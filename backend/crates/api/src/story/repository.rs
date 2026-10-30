//! Separate profile-owned story persistence. Never imports browser checkpoints.
use super::planning::ValidatedPlan;
use crate::game::{Error, Identity};
use postgres::{Client, NoTls, Transaction};
use serde_json::Value;
use wonderwilds_contracts::validate;
use wonderwilds_domain::story::{CONTENT_VERSION, State};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissionLearningContext {
    pub mission_id: String,
    pub location_id: String,
    pub story_revision: u64,
}

/// Read the verified active story context inside a caller-owned transaction.
/// Mission-learning commands call this after locking the same profile row used
/// by story writers, so the issued activity cannot bind to stale client state.
pub(crate) fn mission_learning_context(
    tx: &mut Transaction<'_>,
    identity: &Identity,
) -> Result<Option<MissionLearningContext>, Error> {
    let state = load(tx, identity)?;
    if state.mission_status() != wonderwilds_domain::story::MissionStatus::Active {
        return Ok(None);
    }
    Ok(Some(MissionLearningContext {
        mission_id: state.mission_id().to_owned(),
        location_id: state.location().as_str().to_owned(),
        story_revision: state.revision(),
    }))
}

/// Frozen, server-resolved joining point. It is never accepted from browser prose.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct InlineStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<wonderwilds_domain::story::Action>,
    pub action_id: String,
    pub revision: u64,
    pub mission_id: String,
    pub location_id: String,
    pub label: String,
    pub introduction: String,
}

pub(crate) fn inline_step(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    action_id: &str,
    revision: u64,
) -> Result<InlineStep, Error> {
    let state = load(tx, identity)?;
    if state.revision() != revision {
        return Err(Error::RevisionConflict);
    }
    let action = wonderwilds_director::story::resolve_action(&state, action_id)
        .map_err(|_| Error::RevisionConflict)?;
    if !matches!(
        action,
        wonderwilds_domain::story::Action::Inspect { .. }
            | wonderwilds_domain::story::Action::Collect { .. }
            | wonderwilds_domain::story::Action::Exchange { .. }
            | wonderwilds_domain::story::Action::Assemble { .. }
    ) {
        return Err(Error::InvalidInput);
    }
    let neighbour = if matches!(&action, wonderwilds_domain::story::Action::Inspect { object_id } if object_id.starts_with("neighbour_"))
    {
        Some(
            state
                .current_neighbour()
                .filter(|q| q.status == "open")
                .ok_or(Error::NotFound)?,
        )
    } else {
        if state.mission_status() != wonderwilds_domain::story::MissionStatus::Active {
            return Err(Error::NotFound);
        }
        None
    };
    let snapshot = saved_snapshot(tx, identity, &state)?;
    let label = snapshot["sceneInput"]["availableActions"]
        .as_array()
        .ok_or(Error::Unavailable)?
        .iter()
        .find(|a| a["id"] == action_id)
        .and_then(|a| a["description"].as_str())
        .ok_or(Error::Unavailable)?
        .to_owned();
    let introduction = neighbour
        .and_then(|q| {
            q.story
                .as_ref()
                .and_then(|s| s.content.steps.get(q.step as usize))
        })
        .and_then(|step| step.learning_bridge.clone())
        .unwrap_or_else(|| {
            match action {
                wonderwilds_domain::story::Action::Collect { .. } => {
                    "Let's organise discovery markers so we can mark where we found these supplies."
                }
                wonderwilds_domain::story::Action::Exchange { .. } => {
                    "Help organise our discovery markers so we can keep track of the workshop jobs."
                }
                wonderwilds_domain::story::Action::Assemble { .. } => {
                    "Let's organise discovery markers to plan where the pieces will go."
                }
                _ => "Let's organise our discovery markers to keep track of what we notice here.",
            }
            .to_owned()
        });
    Ok(InlineStep {
        action: Some(action),
        action_id: action_id.to_owned(),
        revision,
        mission_id: neighbour.map_or_else(
            || state.mission_id().to_owned(),
            |q| format!("neighbour-{}-{}", q.character, q.generation),
        ),
        location_id: state.location().as_str().to_owned(),
        label,
        introduction,
    })
}

/// Complete learning even when its original story step has expired. Only apply
/// the story effect if the same semantic action is still legal for that mission
/// (including neighbour generation) and location, under the profile lock.
pub(crate) fn complete_inline_step(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    step: &InlineStep,
    learning_command_id: &str,
) -> Result<(), Error> {
    let state = load(tx, identity)?;
    if state.command_budget_exhausted() {
        return Ok(());
    }
    let action_id = if state.revision() == step.revision {
        step.action_id.clone()
    } else {
        let Some(action) = &step.action else {
            return Ok(());
        };
        let scene = wonderwilds_director::story::build_scene_input(&state)
            .map_err(|_| Error::Unavailable)?;
        let encoded = serde_json::to_value(action).map_err(|_| Error::Unavailable)?;
        let Some(candidate) = scene["availableActions"]
            .as_array()
            .ok_or(Error::Unavailable)?
            .iter()
            .find(|candidate| candidate["action"] == encoded)
        else {
            return Ok(());
        };
        candidate["id"]
            .as_str()
            .ok_or(Error::Unavailable)?
            .to_owned()
    };
    let current = match inline_step(tx, identity, &action_id, state.revision()) {
        Ok(current) => current,
        Err(Error::RevisionConflict) => return Ok(()),
        Err(error) => return Err(error),
    };
    if current.mission_id != step.mission_id || current.location_id != step.location_id {
        return Ok(());
    }
    let id = format!(
        "learning-step-{}",
        crate::persistence::digest(&serde_json::json!(learning_command_id))
    );
    let request = serde_json::json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":state.revision(),"payload":{"type":"choose","actionId":action_id}});
    crate::persistence::take_budget(tx, &identity.profile_id, "story")?;
    let (next, _) = super::execute(&state, &request)?;
    let response = saved_snapshot(tx, identity, &next)?;
    save_state(tx, identity, &next)?;
    let command = serde_json::to_value(next.last_command().ok_or(Error::Unavailable)?)
        .map_err(|_| Error::Unavailable)?;
    let digest = crate::persistence::receipt(&request, &response, &command);
    tx.execute("INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,revision,request,response,domain_command,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6,$7)",
        &[&identity.profile_id,&id,&(next.revision() as i64),&request,&response,&command,&digest]).map_err(|_| Error::Unavailable)?;
    Ok(())
}

pub trait Repository: Send + Sync {
    fn readiness(&self) -> Result<(), Error> {
        Err(Error::Unavailable)
    }
    fn read(&self, identity: &Identity) -> Result<Value, Error>;
    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error>;
    fn generation_state(&self, _: &Identity, _: &Value) -> Result<Option<State>, Error> {
        Ok(None)
    }
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

    /// Offline replay from the original compatible checkpoint and durable log.
    pub fn verify_history(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, true)?;
        let rebuilt = rebuild(&mut tx, identity, false)?;
        let current = load(&mut tx, identity)?;
        if persisted(&rebuilt) != persisted(&current) {
            return Err(Error::Unavailable);
        }
        let response = saved_snapshot(&mut tx, identity, &rebuilt)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
    pub fn recover_current(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, true)?;
        let rebuilt = rebuild(&mut tx, identity, true)?;
        save_state(&mut tx, identity, &rebuilt)?;
        let response = saved_snapshot(&mut tx, identity, &rebuilt)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
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
        "SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE"
    } else {
        "SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR SHARE"
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

fn persisted(state: &State) -> Value {
    crate::persistence::envelope("story-storage-1", state.durable_checkpoint())
}
fn restored(value: &Value) -> Result<State, Error> {
    State::from_durable_checkpoint(&crate::persistence::unwrap(value, "story-storage-1")?)
        .map_err(|_| Error::Unavailable)
}
fn save_state(tx: &mut Transaction<'_>, identity: &Identity, state: &State) -> Result<(), Error> {
    let value = persisted(state);
    if value.to_string().len() > 65536 {
        return Err(Error::RateLimited);
    }
    crate::persistence::save(tx, &identity.profile_id, "story", state.revision(), &value)?;
    if state.revision() > 0 {
        tx.execute("INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id) DO UPDATE SET revision=EXCLUDED.revision,checkpoint=EXCLUDED.checkpoint", &[&identity.profile_id,&(state.revision() as i64),&value]).map_err(|_|Error::Unavailable)?;
    }
    Ok(())
}
fn load(tx: &mut Transaction<'_>, identity: &Identity) -> Result<State, Error> {
    let saved=tx.query_opt("SELECT revision,checkpoint FROM wonderwilds.story_progress WHERE profile_id=$1::text::uuid", &[&identity.profile_id]).map_err(|_|Error::Unavailable)?;
    if let Some(row) = &saved {
        let value: Value = row.get(1);
        if value.get("format").is_some() {
            let state = restored(&value)?;
            if row.get::<_, i64>(0) != state.revision() as i64 {
                return Err(Error::Unavailable);
            }
            if let Some(previous) = crate::persistence::previous(
                tx,
                &identity.profile_id,
                "story",
                state.revision(),
                &value,
            )? {
                let mut prior = restored(&previous)?;
                let row=tx.query_opt("SELECT domain_command,request,response,digest FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND revision=$2", &[&identity.profile_id,&(state.revision() as i64)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
                let command: Value = row.get(0);
                if row.get::<_, Option<String>>(3).as_deref()
                    != Some(
                        crate::persistence::receipt(&row.get(1), &row.get(2), &command).as_str(),
                    )
                {
                    return Err(Error::Unavailable);
                }
                let command = serde_json::from_value(command).map_err(|_| Error::Unavailable)?;
                prior.apply(&command).map_err(|_| Error::Unavailable)?;
                if persisted(&prior) != value {
                    return Err(Error::Unavailable);
                }
            } else {
                let origin = crate::persistence::original(tx, &identity.profile_id, "story")?;
                let initial = State::from_checkpoint(&origin).map_err(|_| Error::Unavailable)?;
                if persisted(&initial) != value {
                    return Err(Error::Unavailable);
                }
            }
            return Ok(state);
        }
    }
    // Legacy full-log checkpoints are replay-validated under their original
    // version before converting. Only this one-time path scans old receipts.
    let (state, original) = match saved {
        Some(row) => {
            let value: Value = row.get(1);
            let state = State::from_checkpoint(&value).map_err(|_| Error::Unavailable)?;
            if row.get::<_, i64>(0) != state.revision() as i64 {
                return Err(Error::Unavailable);
            }
            (state, value)
        }
        None => {
            let latest:i64=tx.query_one("SELECT coalesce(max(revision),0) FROM wonderwilds.story_checkpoints WHERE profile_id=$1::text::uuid", &[&identity.profile_id]).map_err(|_|Error::Unavailable)?.get(0);
            if latest != 0 {
                return Err(Error::Unavailable);
            }
            let state = State::new();
            let value = state.checkpoint();
            (state, value)
        }
    };
    let receipts=tx.query("SELECT command_id,revision,request,response FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid ORDER BY revision", &[&identity.profile_id]).map_err(|_|Error::Unavailable)?;
    if receipts.len() as u64 != state.revision() {
        return Err(Error::Unavailable);
    }
    for (index, row) in receipts.iter().enumerate() {
        let command = &original["commands"][index];
        let request: Value = row.get(2);
        let response: Value = row.get(3);
        validate("StoryCommand", &request).map_err(|_| Error::Unavailable)?;
        validate("StorySnapshot", &response).map_err(|_| Error::Unavailable)?;
        if request["commandId"] != row.get::<_, String>(0)
            || request["expectedRevision"] != index as u64
            || response["revision"] != index as u64 + 1
        {
            return Err(Error::Unavailable);
        }

        if command["commandId"] != row.get::<_, String>(0)
            || row.get::<_, i64>(1) != index as i64 + 1
        {
            return Err(Error::Unavailable);
        }
        tx.execute("UPDATE wonderwilds.story_command_receipts SET domain_command=$3,digest=$4 WHERE profile_id=$1::text::uuid AND revision=$2", &[&identity.profile_id,&(index as i64+1),command,&crate::persistence::receipt(&row.get(2),&row.get(3),command)]).map_err(|_|Error::Unavailable)?;
    }
    crate::persistence::origin(tx, &identity.profile_id, "story", &original)?;
    let state = state.compact();
    save_state(tx, identity, &state)?;
    Ok(state)
}

fn rebuild(tx: &mut Transaction<'_>, identity: &Identity, restore: bool) -> Result<State, Error> {
    let original = crate::persistence::original(tx, &identity.profile_id, "story")?;
    let mut state = State::from_checkpoint(&original)
        .map_err(|_| Error::Unavailable)?
        .compact();
    for (index, command) in original["commands"]
        .as_array()
        .ok_or(Error::Unavailable)?
        .iter()
        .enumerate()
    {
        let row=tx.query_opt("SELECT domain_command,request,response,digest FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND revision=$2", &[&identity.profile_id,&(index as i64+1)]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
        if row.get::<_, Value>(0) != *command
            || row.get::<_, Option<String>>(3).as_deref()
                != Some(crate::persistence::receipt(&row.get(1), &row.get(2), command).as_str())
        {
            return Err(Error::Unavailable);
        }
    }
    let mut verified = vec![(state.revision(), persisted(&state))];
    loop {
        let rows=tx.query("SELECT revision,domain_command,request,response,digest FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND revision>$2 ORDER BY revision LIMIT 256", &[&identity.profile_id,&(state.revision() as i64)]).map_err(|_|Error::Unavailable)?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            if row.get::<_, i64>(0) != state.revision() as i64 + 1 {
                return Err(Error::Unavailable);
            }
            let command: Value = row.get(1);
            if row.get::<_, Option<String>>(4).as_deref()
                != Some(crate::persistence::receipt(&row.get(2), &row.get(3), &command).as_str())
            {
                return Err(Error::Unavailable);
            }
            state
                .apply(&serde_json::from_value(command).map_err(|_| Error::Unavailable)?)
                .map_err(|_| Error::Unavailable)?;
            state = state.compact();
            verified.push((state.revision(), persisted(&state)));
            if verified.len() > 2 {
                verified.remove(0);
            }
        }
    }
    let row = tx
        .query_opt(
            "SELECT revision FROM wonderwilds.story_progress WHERE profile_id=$1::text::uuid",
            &[&identity.profile_id],
        )
        .map_err(|_| Error::Unavailable)?;
    if row.map_or(0, |r| r.get::<_, i64>(0)) != state.revision() as i64 {
        return Err(Error::Unavailable);
    }
    if restore {
        crate::persistence::replace(tx, &identity.profile_id, "story", &verified)?;
    }
    Ok(state)
}

fn saved_snapshot(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    state: &State,
) -> Result<Value, Error> {
    let mut snapshot = super::snapshot(state)?;
    for row in tx.query("SELECT mission_id,accepted_revision,plan FROM wonderwilds.story_mission_plans WHERE profile_id=$1::text::uuid AND content_version=$2",
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
                "SELECT to_regclass('wonderwilds.story_progress') IS NOT NULL AND to_regclass('wonderwilds.story_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.story_mission_plans') IS NOT NULL AND to_regclass('wonderwilds.story_checkpoints') IS NOT NULL AND to_regclass('wonderwilds.persistence_origins') IS NOT NULL AND to_regclass('wonderwilds.command_windows') IS NOT NULL",
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
        own_profile(&mut tx, identity, true)?;
        let state = load(&mut tx, identity)?;
        let response = saved_snapshot(&mut tx, identity, &state)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }

    fn generation_state(
        &self,
        identity: &Identity,
        request: &Value,
    ) -> Result<Option<State>, Error> {
        validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, true)?;
        let state = load(&mut tx, identity)?;
        let retry = tx.query_opt("SELECT 1 FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&identity.profile_id, &id]).map_err(|_| Error::Unavailable)?.is_some();
        let result = if retry {
            None
        } else {
            let (next, _) = super::execute(&state, request)?;
            (next.needs_generated_story()
                && matches!(
                    next.last_command().map(|c| &c.action),
                    Some(wonderwilds_domain::story::Action::Travel { .. })
                ))
            .then_some(next)
        };
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(result)
    }

    fn planning_state(&self, identity: &Identity, request: &Value) -> Result<Option<State>, Error> {
        validate("StoryCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own_profile(&mut tx, identity, true)?;
        let state = load(&mut tx, identity)?;
        // Exact retries need no model request; execute still checks body equality.
        let retry = tx.query_opt("SELECT 1 FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",
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
                "SELECT request,response,revision,domain_command,digest FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",
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
            if row.get::<_,Option<String>>(4).as_deref()!=Some(crate::persistence::receipt(request,&response,&row.get(3)).as_str()) {return Err(Error::Unavailable);}
            validate("StorySnapshot", &response).map_err(|_| Error::Unavailable)?;
            if response["revision"].as_u64() != u64::try_from(revision).ok()
                || revision <= 0
            {
                return Err(Error::Unavailable);
            }
            tx.commit().map_err(|_| Error::Unavailable)?;
            return Ok(response);
        }
        crate::persistence::take_budget(&mut tx, &identity.profile_id, "story")?;
        let (mut next, _) = super::execute(&state, request)?;
        if let Some(plan) = plan.filter(|p| p.is_generated()) {
            let value = plan.for_state(&next)?;
            let story = serde_json::from_value(value.clone()).map_err(|_| Error::InvalidInput)?;
            let mut command = next.last_command().ok_or(Error::InvalidInput)?.clone();
            let wonderwilds_domain::story::Action::Travel { destination_id } = command.action
            else {
                return Err(Error::InvalidInput);
            };
            command.action = wonderwilds_domain::story::Action::GeneratedTravel {
                destination_id,
                story,
            };
            next = state.clone();
            next.apply(&command).map_err(|_| Error::InvalidInput)?;
        }
        if let Some(plan) = plan.filter(|p| !p.is_generated()) {
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
        save_state(&mut tx, identity, &next)?;
        let domain_command = serde_json::to_value(next.last_command().ok_or(Error::Unavailable)?)
            .map_err(|_| Error::Unavailable)?;
        tx.execute(
            "INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,revision,request,response,domain_command,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6,$7)",
            &[&identity.profile_id, &id, &revision, request, &response, &domain_command,&crate::persistence::receipt(request,&response,&domain_command)],
        ).map_err(|_| Error::Unavailable)?;
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
