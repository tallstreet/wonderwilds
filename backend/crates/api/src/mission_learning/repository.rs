//! Durable mission-learning state. This is separate from game/story checkpoints.
use crate::{
    game::{Error, Identity},
    persistence,
    story::repository::{MissionLearningContext, mission_learning_context},
};
use postgres::{Client, NoTls, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use wonderwilds_contracts::{
    LearningEvidence, LearningEvidenceEventKind as EventKind,
    LearningEvidenceProvenance as Provenance, LearningEvidenceResult as ResultKind,
    LearningEvidenceSchemaVersion, SupportState as WireSupport, validate,
};
use wonderwilds_learning::mission::{
    self, MissionActivity, MissionAnswer, MissionObservation, MissionState, Support, SupportKind,
};

const STORAGE_FORMAT: &str = "mission-learning-storage-v1";
const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const POLICY_EXPLANATION: &str = "Practice readiness requires eight independent first checks, at least seven correct, varied task variants and representations, two story contexts, two authenticated sessions, and three recent correct checks. It is experimental practice routing, not calibrated academic mastery.";

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
    fail_before_commit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActiveActivity {
    id: String,
    skill_id: String,
    seed: u64,
    mission_id: String,
    location_id: String,
    story_revision: u64,
    checks: u32,
    hint_count: u8,
    worked_example: bool,
    read_aloud: bool,
    revealed_kind: Option<String>,
    revealed_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LearningState {
    revision: u64,
    content_version: String,
    policy: MissionState,
    journal_digest: String,
    issued_by_skill: BTreeMap<String, u64>,
    active: Option<ActiveActivity>,
    last_feedback: Option<Value>,
}

impl Default for LearningState {
    fn default() -> Self {
        Self {
            revision: 0,
            content_version: mission::CONTENT_VERSION.to_owned(),
            policy: MissionState::default(),
            journal_digest: ZERO_DIGEST.to_owned(),
            issued_by_skill: BTreeMap::new(),
            active: None,
            last_feedback: None,
        }
    }
}

struct PendingEvent {
    sequence: u64,
    event_id: String,
    record: Value,
    previous_digest: String,
    digest: String,
}

#[derive(Debug, Clone)]
struct Invitation {
    title: &'static str,
    relation: String,
    message: &'static str,
}

fn invitation(mission_id: &str, location_id: &str) -> Option<Invitation> {
    // Story copy is authored and bounded here; clients cannot select or edit context.
    let (title, relation, message) = match mission_id {
        "pips_bridge" => (
            "Practise fair sharing",
            "Use the bridge story to practise arranging a small collection fairly.",
            "Would you like to practise fair sharing with this story?",
        ),
        "pips_picnic" => (
            "Count picnic supplies",
            "Use the picnic story to practise counting and comparing supplies.",
            "Would you like to count picnic supplies?",
        ),
        "welcome_lights" => (
            "Count the lights",
            "Use the welcome-lights story to practise counting and number paths.",
            "Would you like to practise with the lights story?",
        ),
        "humming_hollow" => (
            "Follow a clue",
            "Use the humming-hollow story to practise comparing collections and number paths.",
            "Would you like to practise with a clue from this story?",
        ),
        "cosy_corner" => (
            "Plan a cosy corner",
            "Use the cosy-corner story to practise counting and completing a whole.",
            "Would you like to practise with the cosy-corner story?",
        ),
        "brambles_visit" => (
            "Plan a visit",
            "Use Bramble's visit story to practise fair sharing and counting.",
            "Would you like to practise with the visit story?",
        ),
        "moonlight_gathering" => (
            "Moonlight gathering",
            "Use the moonlight-gathering story to practise fair sharing and number paths.",
            "Would you like to practise with the gathering story?",
        ),
        _ => return None,
    };
    let location = match location_id {
        "bridge_bank" => "the bridge bank",
        "workshop" => "the workshop",
        "far_bank" => "the far bank",
        _ => return None,
    };
    Some(Invitation {
        title,
        relation: format!("{relation} The current story scene is at {location}."),
        message,
    })
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

    fn own_profile(tx: &mut Transaction<'_>, identity: &Identity) -> Result<(), Error> {
        tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
            .map_err(|_| Error::Unavailable)?;
        tx.query_one(
            "SELECT set_config('app.parent_id',$1,true)",
            &[&identity.parent_id],
        )
        .map_err(|_| Error::Unavailable)?;
        if tx.query_opt("SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE", &[&identity.profile_id, &identity.parent_id]).map_err(|_| Error::Unavailable)?.is_none() {
            return Err(Error::Forbidden);
        }
        Ok(())
    }

    fn load(tx: &mut Transaction<'_>, profile: &str) -> Result<LearningState, Error> {
        let row = tx.query_opt("SELECT revision,checkpoint,digest FROM wonderwilds.mission_learning_progress WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_| Error::Unavailable)?;
        let state = if let Some(row) = row {
            let revision: i64 = row.get(0);
            let checkpoint: Value = row.get(1);
            let digest: String = row.get(2);
            if digest != persistence::digest(&checkpoint) {
                return Err(Error::Unavailable);
            }
            let value = persistence::unwrap(&checkpoint, STORAGE_FORMAT)?;
            let state: LearningState =
                serde_json::from_value(value).map_err(|_| Error::Unavailable)?;
            if revision < 0
                || revision as u64 != state.revision
                || state.revision > 1_000_000_000
                || state.content_version != mission::CONTENT_VERSION
                || state.policy.policy_version != mission::POLICY_VERSION
            {
                return Err(Error::Unavailable);
            }
            state
        } else {
            LearningState::default()
        };
        let latest = tx.query_opt("SELECT sequence,event_id,command_id,record,previous_digest,digest FROM wonderwilds.mission_learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence DESC LIMIT 1", &[&profile]).map_err(|_| Error::Unavailable)?;
        let (journal_sequence, journal_digest) = if let Some(row) = latest {
            let sequence: i64 = row.get(0);
            let event_id: String = row.get(1);
            let command_id: String = row.get(2);
            let record: Value = row.get(3);
            let previous: String = row.get(4);
            let digest: String = row.get(5);
            if digest
                != persistence::digest(&json!([previous, sequence, event_id, command_id, record]))
            {
                return Err(Error::Unavailable);
            }
            (sequence as u64, digest)
        } else {
            (0, ZERO_DIGEST.to_owned())
        };
        if journal_sequence != state.policy.last_sequence || journal_digest != state.journal_digest
        {
            return Err(Error::Unavailable);
        }
        let latest_receipt = tx.query_opt("SELECT revision FROM wonderwilds.mission_learning_command_receipts WHERE profile_id=$1::text::uuid ORDER BY revision DESC LIMIT 1", &[&profile]).map_err(|_| Error::Unavailable)?;
        if latest_receipt
            .map(|r| r.get::<_, i64>(0) as u64)
            .unwrap_or(0)
            != state.revision
        {
            return Err(Error::Unavailable);
        }
        Ok(state)
    }

    /// Full journal replay is deliberately separate from ordinary hot reads. Tests use it to
    /// prove the bounded policy projection can be independently reconstructed.
    #[cfg(test)]
    fn verify_history(
        tx: &mut Transaction<'_>,
        profile: &str,
        expected: &LearningState,
    ) -> Result<(), Error> {
        let rows = tx.query("SELECT sequence,event_id,command_id,record,previous_digest,digest FROM wonderwilds.mission_learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence", &[&profile]).map_err(|_| Error::Unavailable)?;
        let mut previous = ZERO_DIGEST.to_owned();
        let mut observations = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let sequence: i64 = row.get(0);
            let event_id: String = row.get(1);
            let command_id: String = row.get(2);
            let record: Value = row.get(3);
            let previous_digest: String = row.get(4);
            let digest: String = row.get(5);
            if sequence as usize != index + 1
                || previous_digest != previous
                || digest
                    != persistence::digest(&json!([
                        previous, sequence, event_id, command_id, record
                    ]))
            {
                return Err(Error::Unavailable);
            }
            observations.push(
                serde_json::from_value(record["observation"].clone())
                    .map_err(|_| Error::Unavailable)?,
            );
            previous = digest;
        }
        let replayed = mission::replay(&observations).map_err(|_| Error::Unavailable)?;
        if replayed != expected.policy || previous != expected.journal_digest {
            return Err(Error::Unavailable);
        }
        Ok(())
    }

    fn context(
        tx: &mut Transaction<'_>,
        identity: &Identity,
    ) -> Result<Option<MissionLearningContext>, Error> {
        mission_learning_context(tx, identity)
    }

    fn active_activity(active: &ActiveActivity) -> Result<(MissionActivity, Value), Error> {
        let context_id = format!("{}:{}", active.mission_id, active.location_id);
        let invite =
            invitation(&active.mission_id, &active.location_id).ok_or(Error::Unavailable)?;
        let context_text = format!("{} — {}", invite.title, invite.relation);
        let activity = mission::authored_activity_in_context(
            &active.skill_id,
            active.seed,
            &active.id,
            &context_id,
            &context_text,
        )
        .map_err(|_| Error::Unavailable)?;
        let mut view = serde_json::to_value(&activity.view).map_err(|_| Error::Unavailable)?;
        view["support"] = json!({"hintCount":active.hint_count,"workedExample":active.worked_example,"readAloud":active.read_aloud});
        if let (Some(kind), Some(text)) = (&active.revealed_kind, &active.revealed_text) {
            view["revealedSupport"] = json!({"kind":kind,"text":text});
        }
        Ok((activity, view))
    }

    fn snapshot(
        state: &LearningState,
        context: Option<&MissionLearningContext>,
    ) -> Result<Value, Error> {
        let current_context =
            context.filter(|c| invitation(&c.mission_id, &c.location_id).is_some());
        let current_invite =
            current_context.and_then(|c| invitation(&c.mission_id, &c.location_id));
        let activity_context = state.active.as_ref();
        let mission_id = activity_context
            .map(|a| a.mission_id.as_str())
            .or_else(|| current_context.map(|c| c.mission_id.as_str()));
        let location_id = activity_context
            .map(|a| a.location_id.as_str())
            .or_else(|| current_context.map(|c| c.location_id.as_str()));
        let active_activity = state
            .active
            .as_ref()
            .map(Self::active_activity)
            .transpose()?
            .map(|(_, view)| view);
        let graph =
            mission::all_skill_projections(&state.policy).map_err(|_| Error::Unavailable)?;
        let mut skills = Vec::with_capacity(graph.len());
        for node in graph {
            let summary = mission::summary(&node.skill_id, &state.policy).ok();
            let status = match node.status {
                mission::Status::Locked => "locked",
                mission::Status::Ready | mission::Status::Unseen | mission::Status::Introduced => {
                    "available"
                }
                mission::Status::Practising => "practising",
                mission::Status::PracticeReady => "ready_for_next",
                mission::Status::Unavailable => "unavailable",
            };
            let (
                eligible,
                correct,
                variants,
                contexts,
                representations,
                sessions,
                blockers,
                next_step,
            ) = if status == "locked" {
                let blockers = node
                    .prerequisites
                    .iter()
                    .map(|p| format!("Complete {p} first."))
                    .collect::<Vec<_>>();
                (
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    blockers,
                    format!(
                        "Complete prerequisite skill{} {} first.",
                        if node.prerequisites.len() == 1 {
                            ""
                        } else {
                            "s"
                        },
                        node.prerequisites.join(", ")
                    ),
                )
            } else if let Some(s) = summary {
                (
                    s.eligible_checks,
                    s.correct_checks,
                    s.distinct_variants,
                    s.distinct_contexts,
                    s.distinct_representations,
                    s.distinct_sessions,
                    s.blockers,
                    s.next_step,
                )
            } else {
                (
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    Vec::<String>::new(),
                    "Not available in this mission slice.".to_owned(),
                )
            };
            skills.push(json!({"skillId":node.skill_id,"objective":node.objective,"prerequisites":node.prerequisites,"status":status,"supportedByMission":node.supported_by_mission,"eligibleChecks":eligible,"correctChecks":correct,"distinctVariants":variants,"distinctContexts":contexts,"distinctRepresentations":representations,"distinctSessions":sessions,"blockers":blockers,"nextStep":next_step}));
        }
        let seed = state.revision.saturating_add(1);
        let frontier_skill_id = mission::select(&state.policy, Some("maths"), seed)
            .map_err(|_| Error::Unavailable)?
            .map(|s| s.skill_id);
        let active_invite =
            activity_context.and_then(|a| invitation(&a.mission_id, &a.location_id));
        let can_begin = current_context.is_some() && frontier_skill_id.is_some();
        let mut value = json!({"schemaVersion":"1.0","contentVersion":mission::CONTENT_VERSION,"policyVersion":mission::POLICY_VERSION,"policyExplanation":POLICY_EXPLANATION,"revision":state.revision,"available":can_begin || activity_context.is_some(),"skills":skills});
        if let Some(id) = mission_id {
            value["missionId"] = json!(id);
        }
        if let Some(id) = location_id {
            value["locationId"] = json!(id);
        }
        if let Some(invite) =
            active_invite.or_else(|| can_begin.then_some(current_invite).flatten())
        {
            value["invitation"] =
                json!({"title":invite.title,"relation":invite.relation,"message":invite.message});
        }
        if let Some(activity) = active_activity {
            value["activeActivity"] = activity;
        }
        if let Some(feedback) = &state.last_feedback {
            value["lastFeedback"] = feedback.clone();
        }
        if let Some(skill_id) = frontier_skill_id {
            value["frontierSkillId"] = json!(skill_id);
        }
        validate("MissionLearningSnapshot", &value).map_err(|_| Error::Unavailable)?;
        if value.to_string().len() > 65536 {
            return Err(Error::RateLimited);
        }
        Ok(value)
    }

    fn save(tx: &mut Transaction<'_>, profile: &str, state: &LearningState) -> Result<(), Error> {
        let value = serde_json::to_value(state).map_err(|_| Error::Unavailable)?;
        let checkpoint = persistence::envelope(STORAGE_FORMAT, value);
        if checkpoint.to_string().len() > 262144 {
            return Err(Error::RateLimited);
        }
        let revision = i64::try_from(state.revision).map_err(|_| Error::Unavailable)?;
        let digest = persistence::digest(&checkpoint);
        tx.execute("INSERT INTO wonderwilds.mission_learning_progress(profile_id,revision,checkpoint,digest) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT(profile_id) DO UPDATE SET revision=EXCLUDED.revision,checkpoint=EXCLUDED.checkpoint,digest=EXCLUDED.digest", &[&profile,&revision,&checkpoint,&digest]).map_err(|_| Error::Unavailable)?;
        Ok(())
    }

    fn receipt(
        tx: &mut Transaction<'_>,
        profile: &str,
        request: &Value,
        response: &Value,
        revision: u64,
    ) -> Result<(), Error> {
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let digest = persistence::digest(&json!([request, response]));
        tx.execute("INSERT INTO wonderwilds.mission_learning_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6)", &[&profile,&id,&(revision as i64),request,response,&digest]).map_err(|e| if e.code().is_some_and(|c| c.code()=="23505") { Error::RevisionConflict } else { Error::Unavailable })?;
        Ok(())
    }

    fn retry(
        tx: &mut Transaction<'_>,
        profile: &str,
        request: &Value,
    ) -> Result<Option<Value>, Error> {
        let Some(id) = request["commandId"].as_str() else {
            return Err(Error::InvalidInput);
        };
        let row = tx.query_opt("SELECT request,response,digest FROM wonderwilds.mission_learning_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&profile,&id]).map_err(|_| Error::Unavailable)?;
        let Some(row) = row else { return Ok(None) };
        let saved_request: Value = row.get(0);
        let response: Value = row.get(1);
        let digest: String = row.get(2);
        if saved_request != *request {
            return Err(Error::RevisionConflict);
        }
        if digest != persistence::digest(&json!([saved_request, response])) {
            return Err(Error::Unavailable);
        }
        validate("MissionLearningSnapshot", &response).map_err(|_| Error::Unavailable)?;
        Ok(Some(response))
    }
}

#[cfg(test)]
mod tests;

impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let present: bool = client.query_one("SELECT to_regclass('wonderwilds.mission_learning_progress') IS NOT NULL AND to_regclass('wonderwilds.mission_learning_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.mission_learning_events') IS NOT NULL", &[]).map_err(|_| Error::Unavailable)?.get(0);
        present.then_some(()).ok_or(Error::Unavailable)
    }
    fn read(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        Self::own_profile(&mut tx, identity)?;
        let state = Self::load(&mut tx, &identity.profile_id)?;
        let context = Self::context(&mut tx, identity)?;
        let response = Self::snapshot(&state, context.as_ref())?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
    fn execute(&self, identity: &Identity, request: &Value) -> Result<Value, Error> {
        validate("MissionLearningCommand", request).map_err(|_| Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        Self::own_profile(&mut tx, identity)?;
        if let Some(response) = Self::retry(&mut tx, &identity.profile_id, request)? {
            tx.commit().map_err(|_| Error::Unavailable)?;
            return Ok(response);
        }
        let mut state = Self::load(&mut tx, &identity.profile_id)?;
        let expected = request["expectedRevision"]
            .as_u64()
            .ok_or(Error::InvalidInput)?;
        if expected != state.revision {
            return Err(Error::RevisionConflict);
        }
        persistence::take_budget(&mut tx, &identity.profile_id, "mission_learning")?;
        let context = Self::context(&mut tx, identity)?;
        let payload = &request["payload"];
        let mut event: Option<(u64, String, Value)> = None;
        match payload["type"].as_str().ok_or(Error::InvalidInput)? {
            "begin" => {
                if state.active.is_some() {
                    return Err(Error::RevisionConflict);
                }
                let context = context.as_ref().ok_or(Error::NotFound)?;
                let invite =
                    invitation(&context.mission_id, &context.location_id).ok_or(Error::NotFound)?;
                let selected = mission::select(
                    &state.policy,
                    Some("maths"),
                    state.revision.saturating_add(1),
                )
                .map_err(|_| Error::Unavailable)?
                .ok_or(Error::NotFound)?;
                let issued = state
                    .issued_by_skill
                    .entry(selected.skill_id.clone())
                    .or_default();
                let seed = *issued;
                *issued = issued.saturating_add(1);
                let id = format!(
                    "mission-activity-{}-{}",
                    state.revision.saturating_add(1),
                    seed
                );
                mission::authored_activity_in_context(
                    &selected.skill_id,
                    seed,
                    &id,
                    &format!("{}:{}", context.mission_id, context.location_id),
                    &format!("{} — {}", invite.title, invite.relation),
                )
                .map_err(|_| Error::Unavailable)?;
                state.active = Some(ActiveActivity {
                    id,
                    skill_id: selected.skill_id,
                    seed,
                    mission_id: context.mission_id.clone(),
                    location_id: context.location_id.clone(),
                    story_revision: context.story_revision,
                    checks: 0,
                    hint_count: 0,
                    worked_example: false,
                    read_aloud: false,
                    revealed_kind: None,
                    revealed_text: None,
                });
                state.last_feedback = None;
            }
            "answer" => {
                let active = state.active.as_mut().ok_or(Error::NotFound)?;
                if payload["encounterId"].as_str() != Some(active.id.as_str()) {
                    return Err(Error::NotFound);
                }
                let (activity, _) = Self::active_activity(active)?;
                let answer: MissionAnswer = serde_json::from_value(payload["answer"].clone())
                    .map_err(|_| Error::InvalidInput)?;
                let support = Support {
                    hint_count: active.hint_count,
                    worked_example: active.worked_example,
                    read_aloud: active.read_aloud,
                };
                let outcome = activity
                    .assess(&answer, support)
                    .map_err(|_| Error::InvalidInput)?;
                let first_check = active.checks == 0;
                active.checks = active.checks.saturating_add(1);
                let sequence = state.policy.last_sequence.saturating_add(1);
                let evidence = Self::evidence(
                    identity,
                    active,
                    &activity.view.representation,
                    sequence,
                    EventKind::AnswerCheck,
                    if outcome.correct {
                        ResultKind::Correct
                    } else {
                        ResultKind::Incorrect
                    },
                    first_check,
                );
                let observation = MissionObservation {
                    evidence,
                    parameter_fingerprint: activity.canonical_fingerprint().to_owned(),
                };
                mission::record(&mut state.policy, observation.clone())
                    .map_err(|_| Error::Unavailable)?;
                let record = json!({
                    "observation":observation,
                    "activity":activity.view,
                    "storyContext":{"missionId":active.mission_id,"locationId":active.location_id,"storyRevision":active.story_revision},
                    "request":payload
                });
                let event_id = observation.evidence.event_id.clone();
                event = Some((sequence, event_id, record));
                state.last_feedback = Some(
                    json!({"result":if outcome.correct {"correct"} else {"incorrect"},"message":if outcome.correct {"That works!"} else {"Try another way, or ask for a hint."},"completed":outcome.correct}),
                );
                if outcome.correct {
                    state.active = None;
                }
            }
            "support" => {
                let active = state.active.as_mut().ok_or(Error::NotFound)?;
                if payload["encounterId"].as_str() != Some(active.id.as_str()) {
                    return Err(Error::NotFound);
                }
                let kind = payload["kind"].as_str().ok_or(Error::InvalidInput)?;
                let (activity, _) = Self::active_activity(active)?;
                let (event_kind, text) = match kind {
                    "hint" => {
                        active.hint_count = active.hint_count.saturating_add(1).min(20);
                        (
                            EventKind::Hint,
                            mission::support_content(
                                &active.skill_id,
                                SupportKind::Hint,
                                usize::from(active.hint_count.saturating_sub(1)),
                            ),
                        )
                    }
                    "worked_example" => {
                        active.worked_example = true;
                        (
                            EventKind::WorkedExample,
                            mission::support_content(
                                &active.skill_id,
                                SupportKind::WorkedExample,
                                0,
                            ),
                        )
                    }
                    _ => return Err(Error::InvalidInput),
                };
                let text = text.ok_or(Error::InvalidInput)?;
                active.revealed_kind = Some(kind.to_owned());
                active.revealed_text = Some(text);
                let sequence = state.policy.last_sequence.saturating_add(1);
                let evidence = Self::evidence(
                    identity,
                    active,
                    &activity.view.representation,
                    sequence,
                    event_kind,
                    ResultKind::NotAssessed,
                    false,
                );
                let observation = MissionObservation {
                    evidence,
                    parameter_fingerprint: activity.canonical_fingerprint().to_owned(),
                };
                mission::record(&mut state.policy, observation.clone())
                    .map_err(|_| Error::Unavailable)?;
                event = Some((
                    sequence,
                    observation.evidence.event_id.clone(),
                    json!({
                        "observation":observation,
                        "activity":activity.view,
                        "storyContext":{"missionId":active.mission_id,"locationId":active.location_id,"storyRevision":active.story_revision},
                        "request":payload
                    }),
                ));
            }
            "abandon" => {
                let active = state.active.take().ok_or(Error::NotFound)?;
                if payload["encounterId"].as_str() != Some(active.id.as_str()) {
                    state.active = Some(active);
                    return Err(Error::NotFound);
                }
                let activity = mission::authored_activity_in_context(
                    &active.skill_id,
                    active.seed,
                    &active.id,
                    &format!("{}:{}", active.mission_id, active.location_id),
                    "Abandoned activity",
                )
                .map_err(|_| Error::Unavailable)?;
                let sequence = state.policy.last_sequence.saturating_add(1);
                let evidence = Self::evidence(
                    identity,
                    &active,
                    &activity.view.representation,
                    sequence,
                    EventKind::Abandonment,
                    ResultKind::NotAssessed,
                    active.checks == 0,
                );
                let observation = MissionObservation {
                    evidence,
                    parameter_fingerprint: activity.canonical_fingerprint().to_owned(),
                };
                mission::record(&mut state.policy, observation.clone())
                    .map_err(|_| Error::Unavailable)?;
                event = Some((
                    sequence,
                    observation.evidence.event_id.clone(),
                    json!({
                        "observation":observation,
                        "activity":activity.view,
                        "storyContext":{"missionId":active.mission_id,"locationId":active.location_id,"storyRevision":active.story_revision},
                        "request":payload
                    }),
                ));
                state.last_feedback = None;
            }
            _ => return Err(Error::InvalidInput),
        }
        state.revision = state
            .revision
            .checked_add(1)
            .filter(|r| *r <= 1_000_000_000)
            .ok_or(Error::RateLimited)?;
        let snapshot_context = context.as_ref();
        let response = Self::snapshot(&state, snapshot_context)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let pending_event = event.map(|(sequence, event_id, record)| {
            let previous_digest = state.journal_digest.clone();
            let digest =
                persistence::digest(&json!([previous_digest, sequence, event_id, id, record]));
            state.journal_digest = digest.clone();
            PendingEvent {
                sequence,
                event_id,
                record,
                previous_digest,
                digest,
            }
        });
        Self::receipt(
            &mut tx,
            &identity.profile_id,
            request,
            &response,
            state.revision,
        )?;
        Self::save(&mut tx, &identity.profile_id, &state)?;
        if let Some(pending) = pending_event {
            tx.execute("INSERT INTO wonderwilds.mission_learning_events(profile_id,sequence,event_id,command_id,record,previous_digest,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6,$7)", &[&identity.profile_id,&(pending.sequence as i64),&pending.event_id,&id,&pending.record,&pending.previous_digest,&pending.digest]).map_err(|_| Error::Unavailable)?;
        }
        #[cfg(test)]
        if self.fail_before_commit {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
}

impl PostgresRepository {
    fn evidence(
        identity: &Identity,
        active: &ActiveActivity,
        representation: &str,
        sequence: u64,
        event_kind: EventKind,
        result: ResultKind,
        first_check: bool,
    ) -> LearningEvidence {
        let event_id = format!(
            "mission-event-{sequence}-{}",
            persistence::digest(&json!([identity.profile_id, &active.id, sequence]))[..12]
                .to_owned()
        );
        LearningEvidence {
            schema_version: LearningEvidenceSchemaVersion::V10,
            event_id,
            encounter_id: active.id.clone(),
            skill_id: active.skill_id.clone(),
            variant_id: format!("{}-mission-v1-{:02}", active.skill_id, active.seed % 12),
            item_family: active.skill_id.clone(),
            context_id: format!("{}:{}", active.mission_id, active.location_id),
            representation: representation.to_owned(),
            result,
            event_kind,
            first_check,
            support: WireSupport {
                hint_count: u64::from(active.hint_count),
                worked_example: active.worked_example,
                read_aloud: active.read_aloud,
            },
            session_id: identity.session_id.clone(),
            server_sequence: sequence,
            provenance: Provenance::ServerVerified,
        }
    }
}
