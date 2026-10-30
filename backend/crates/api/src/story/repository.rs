//! Separate profile-owned story persistence. Never imports browser checkpoints.
use super::planning::ValidatedPlan;
use crate::game::{Error, Identity};
use postgres::{Client, NoTls, Transaction};
use serde_json::Value;
use wonderwilds_contracts::validate;
use wonderwilds_domain::story::{CONTENT_VERSION, State};

pub trait Repository: Send + Sync {
    fn readiness(&self) -> Result<(), Error> {
        Err(Error::Unavailable)
    }
    fn read(&self, identity: &Identity) -> Result<Value, Error>;
    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error>;
    fn planning_state(&self, _: &Identity, _: &Value) -> Result<Option<State>, Error> {
        Ok(None)
    }
    fn execute_planned(
        &self,
        identity: &Identity,
        request: &Value,
        plan: Option<&ValidatedPlan>,
    ) -> Result<Value, Error> {
        if plan.is_some() {
            return Err(Error::Unavailable);
        }
        self.execute(identity, request)
    }
}

#[derive(Clone)]
pub struct PostgresRepository {
    connection: String,
    #[cfg(test)]
    pub fail_before_commit: bool,
}

impl PostgresRepository {
    pub fn new(connection: String) -> Self {
        Self {
            connection,
            #[cfg(test)]
            fail_before_commit: false,
        }
    }

    fn connect(&self) -> Result<Client, Error> {
        Client::connect(&self.connection, NoTls).map_err(|_| Error::Unavailable)
    }
}

fn own_profile(tx: &mut Transaction<'_>, identity: &Identity, write: bool) -> Result<(), Error> {
    tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
        .map_err(|_| Error::Unavailable)?;
    tx.query_one(
        "SELECT set_config('app.parent_id', $1, true)",
        &[&identity.parent_id],
    )
    .map_err(|_| Error::Unavailable)?;
    // All story/game writers and deletion share this row lock. Readers use a
    // shared lock so their state/receipt consistency check cannot straddle a write.
    let query = if write {
        "SELECT id FROM wonderwilds.profiles WHERE id::text=$1 AND parent_id::text=$2 FOR UPDATE"
    } else {
        "SELECT id FROM wonderwilds.profiles WHERE id::text=$1 AND parent_id::text=$2 FOR SHARE"
    };
    if tx
        .query_opt(query, &[&identity.profile_id, &identity.parent_id])
        .map_err(|_| Error::Unavailable)?
        .is_none()
    {
        return Err(Error::Forbidden);
    }
    Ok(())
}

fn load(tx: &mut Transaction<'_>, identity: &Identity) -> Result<State, Error> {
    let saved = tx
        .query_opt(
            "SELECT revision,checkpoint FROM wonderwilds.story_progress WHERE profile_id::text=$1",
            &[&identity.profile_id],
        )
        .map_err(|_| Error::Unavailable)?;
    let state = match saved {
        Some(row) => {
            let revision: i64 = row.get(0);
            let checkpoint: Value = row.get(1);
            let state = State::from_checkpoint(&checkpoint).map_err(|_| Error::Unavailable)?;
            if i64::try_from(state.revision()).ok() != Some(revision) {
                return Err(Error::Unavailable);
            }
            state
        }
        None => State::new(),
    };
    let receipts: i64 = tx
        .query_one(
            "SELECT count(*) FROM wonderwilds.story_command_receipts WHERE profile_id::text=$1",
            &[&identity.profile_id],
        )
        .map_err(|_| Error::Unavailable)?
        .get(0);
    if receipts != i64::try_from(state.revision()).map_err(|_| Error::Unavailable)? {
        // Missing progress or missing receipt is corruption, not a new game.
        return Err(Error::Unavailable);
    }
    Ok(state)
}

fn saved_snapshot(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    state: &State,
) -> Result<Value, Error> {
    let mut snapshot = super::snapshot(state)?;
    for row in tx.query("SELECT mission_id,accepted_revision,plan FROM wonderwilds.story_mission_plans WHERE profile_id::text=$1 AND content_version=$2",
        &[&identity.profile_id,&CONTENT_VERSION]).map_err(|_| Error::Unavailable)? {
        let mission: String = row.get(0);
        let revision: i64 = row.get(1);
        let plan: Value = row.get(2);
        validate("StoryMissionPlan", &plan).map_err(|_| Error::Unavailable)?;
        if revision < 1 || revision > state.revision() as i64 || plan["opportunityId"] != mission {
            return Err(Error::Unavailable);
        }
        if mission == state.mission_id() { super::planning::project(state, &plan, &mut snapshot)?; }
        if snapshot["commandBudgetExhausted"] == false {
            for (index, completed) in state.completed_missions().iter().enumerate() {
                if completed.as_str() == mission {
                    snapshot["sceneInput"]["context"]["memory"]["recentMissions"][index] = serde_json::json!(format!("{} Completed.", plan["goal"].as_str().unwrap().chars().take(189).collect::<String>()));
                }
            }
        }
    }
    validate("StorySnapshot", &snapshot).map_err(|_| Error::Unavailable)?;
    Ok(snapshot)
}

impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let present: bool = client
            .query_one(
                "SELECT to_regclass('wonderwilds.story_progress') IS NOT NULL AND to_regclass('wonderwilds.story_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.story_mission_plans') IS NOT NULL",
                &[],
            )
            .map_err(|_| Error::Unavailable)?
            .get(0);
        if present {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }

    fn read(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, false)?;
        let state = load(&mut tx, identity)?;
        let response = saved_snapshot(&mut tx, identity, &state)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }

    fn planning_state(&self, identity: &Identity, request: &Value) -> Result<Option<State>, Error> {
        validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, false)?;
        let state = load(&mut tx, identity)?;
        // Exact retries need no model request; execute still checks body equality.
        let retry = tx.query_opt("SELECT 1 FROM wonderwilds.story_command_receipts WHERE profile_id::text=$1 AND command_id=$2",
            &[&identity.profile_id, &id]).map_err(|_| Error::Unavailable)?.is_some();
        let result = if retry {
            None
        } else {
            super::planning::acceptance(&state, request)?
        };
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(result)
    }

    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error> {
        self.execute_planned(identity, request, None)
    }

    fn execute_planned(
        &self,
        identity: &Identity,
        request: &Value,
        plan: Option<&ValidatedPlan>,
    ) -> Result<Value, Error> {
        validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, true)?;
        let state = load(&mut tx, identity)?;
        if let Some(row) = tx
            .query_opt(
                "SELECT request,response,revision FROM wonderwilds.story_command_receipts WHERE profile_id::text=$1 AND command_id=$2",
                &[&identity.profile_id, &id],
            )
            .map_err(|_| Error::Unavailable)?
        {
            let original: Value = row.get(0);
            if original != *request {
                return Err(Error::RevisionConflict);
            }
            let response: Value = row.get(1);
            let revision: i64 = row.get(2);
            validate("StorySnapshot", &response).map_err(|_| Error::Unavailable)?;
            if response["revision"].as_u64() != u64::try_from(revision).ok()
                || revision <= 0
                || revision > state.revision() as i64
            {
                return Err(Error::Unavailable);
            }
            tx.commit().map_err(|_| Error::Unavailable)?;
            return Ok(response);
        }
        let (next, _) = super::execute(&state, request)?;
        if let Some(plan) = plan {
            if super::planning::acceptance(&state, request)?.is_none() {
                return Err(Error::InvalidInput);
            }
            let value = plan.for_state(&next)?;
            let revision = next.revision() as i64;
            tx.execute("INSERT INTO wonderwilds.story_mission_plans(profile_id,content_version,mission_id,accepted_revision,plan) VALUES($1::text::uuid,$2,$3,$4,$5)",
                &[&identity.profile_id,&CONTENT_VERSION,&next.mission_id(),&revision,value])
                .map_err(|_| Error::Unavailable)?;
        }
        let response = saved_snapshot(&mut tx, identity, &next)?;
        let revision = i64::try_from(next.revision()).map_err(|_| Error::Unavailable)?;
        let checkpoint = next.checkpoint();
        tx.execute(
            "INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id) DO UPDATE SET revision=EXCLUDED.revision,checkpoint=EXCLUDED.checkpoint",
            &[&identity.profile_id, &revision, &checkpoint],
        )
        .map_err(|_| Error::Unavailable)?;
        tx.execute(
            "INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,revision,request,response) VALUES($1::text::uuid,$2,$3,$4,$5)",
            &[&identity.profile_id, &id, &revision, request, &response],
        )
        .map_err(|_| Error::Unavailable)?;
        #[cfg(test)]
        if self.fail_before_commit {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests;
