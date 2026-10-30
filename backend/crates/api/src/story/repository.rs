//! Separate profile-owned story persistence. Never imports browser checkpoints.
use crate::game::{Error, Identity};
use postgres::{Client, NoTls, Transaction};
use serde_json::Value;
use wonderwilds_contracts::validate;
use wonderwilds_domain::story::State;

pub trait Repository: Send + Sync {
    fn readiness(&self) -> Result<(), Error> {
        Err(Error::Unavailable)
    }
    fn read(&self, identity: &Identity) -> Result<Value, Error>;
    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error>;
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

impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let present: bool = client
            .query_one(
                "SELECT to_regclass('wonderwilds.story_progress') IS NOT NULL AND to_regclass('wonderwilds.story_command_receipts') IS NOT NULL",
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
        let response = super::snapshot(&state)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }

    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error> {
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
        let (next, response) = super::execute(&state, request)?;
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
