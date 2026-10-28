//! Authoritative authored-game application service. Incoming JSON never supplies
//! assessment, rewards, identity, support history or a domain transition.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use wonderwilds_contracts as wire;
use wonderwilds_domain::engine::{
    self, Action, Command, Content, Context, State, TrustedFacts, VerifiedCompletion,
};
use wonderwilds_learning::assessment::{
    self,
    wire::{assess_wire, authored_encounter},
};

pub mod http;
pub mod repository;

pub const MAX_COMMANDS: usize = 2000;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    Unauthenticated,
    Forbidden,
    NotFound,
    RevisionConflict,
    RateLimited,
    Unavailable,
}
#[derive(Clone, Debug)]
pub struct Identity {
    pub parent_id: String,
    pub profile_id: String,
    pub session_id: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct SavedEncounter {
    id: String,
    quest: String,
    skill: String,
    seed: u64,
    support: wire::SupportState,
    checks: u64,
    completed: bool,
    #[serde(default)]
    abandoned: bool,
    session_id: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Receipt {
    request: Value,
    response: Value,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Aggregate {
    revision: u64,
    domain: State,
    encounters: BTreeMap<String, SavedEncounter>,
    receipts: BTreeMap<String, Receipt>,
    pub(crate) evidence: Vec<wire::LearningEvidence>,
    pub(crate) events: Vec<engine::Event>,
}

pub fn development_content() -> Result<Content, Error> {
    Content::development(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/story/adventure.v1.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/story/help-quests.v1.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/creatures/friends.v1.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/story/progression.v1.json"
        )),
    )
    .map_err(|_| Error::Unavailable)
}

impl Aggregate {
    pub fn new(content: &Content) -> Self {
        Self {
            revision: 0,
            domain: State::new(content),
            encounters: BTreeMap::new(),
            receipts: BTreeMap::new(),
            evidence: Vec::new(),
            events: Vec::new(),
        }
    }
    pub fn snapshot(&self, content: &Content) -> Result<Value, Error> {
        let mut pending = Vec::new();
        for encounter in self
            .encounters
            .values()
            .filter(|e| !e.completed && !e.abandoned)
        {
            pending.push(self.public_encounter(&encounter.id)?);
        }
        use wonderwilds_learning::mastery::{self, Observation, Status};
        let mut observations = Vec::new();
        let mut skills = BTreeMap::new();
        for evidence in &self.evidence {
            let saved = self
                .encounters
                .get(&evidence.encounter_id)
                .ok_or(Error::Unavailable)?;
            if saved.skill != evidence.skill_id {
                return Err(Error::Unavailable);
            }
            let (_, canonical) = authored_encounter(&saved.skill, saved.seed, &saved.id)
                .map_err(|_| Error::Unavailable)?;
            let parameters =
                serde_json::to_string(&canonical.parameters).map_err(|_| Error::Unavailable)?;
            observations.push(Observation {
                evidence: evidence.clone(),
                parameter_fingerprint: digest_id(&parameters),
                // No draft item has an educator-reviewed novel transfer design.
                // Reopening an authored item never invents one.
                novel_transfer: false,
            });
            skills.insert(evidence.skill_id.clone(), Value::Null);
        }
        for (skill_id, value) in &mut skills {
            let summary =
                mastery::recompute(skill_id, &observations, 2).map_err(|_| Error::Unavailable)?;
            let status = match summary.status {
                Status::Unseen => "unseen",
                Status::Introduced => "introduced",
                Status::Practising => "practising",
                Status::Secure => "secure",
            };
            *value = json!({"skillId":skill_id,"status":status,"explanation":summary.explanation});
        }
        let value = json!({"schemaVersion":"1.0","revision":self.revision,"contentVersion":content.version(),"rulesVersion":engine::RULES_VERSION,"storyFlags":self.domain.flags(),"ownedCreatures":self.domain.creatures(),"selectedCompanions":self.domain.team(),"inventory":self.domain.inventory().iter().map(|(id,n)|json!({"itemId":id,"quantity":n})).collect::<Vec<_>>(),"pendingEncounters":pending,"learnerSummaries":skills.into_values().collect::<Vec<_>>()});
        wire::validate("GameSnapshot", &value).map_err(|_| Error::Unavailable)?;
        Ok(value)
    }
    pub fn public_encounter(&self, id: &str) -> Result<Value, Error> {
        let saved = self.encounters.get(id).ok_or(Error::NotFound)?;
        if saved.completed || saved.abandoned {
            return Err(Error::NotFound);
        }
        let (_, mut encounter) = authored_encounter(&saved.skill, saved.seed, &saved.id)
            .map_err(|_| Error::Unavailable)?;
        encounter.support = saved.support.clone();
        serde_json::to_value(encounter).map_err(|_| Error::Unavailable)
    }
    fn domain_apply(
        &mut self,
        action: Action,
        command_id: &str,
        content: &Content,
        facts: TrustedFacts,
        now: u64,
    ) -> Result<(), Error> {
        let command = Command {
            command_id: command_id.into(),
            expected_revision: self.domain.revision(),
            action,
        };
        let applied = engine::apply(
            &self.domain,
            &command,
            content,
            &Context {
                unix_millis: now.saturating_mul(1000),
                recorded_random_choice: 0,
                facts,
            },
        )
        .map_err(|e| match e {
            engine::Rejection::RevisionConflict | engine::Rejection::CommandIdReused => {
                Error::RevisionConflict
            }
            engine::Rejection::NotFound => Error::NotFound,
            engine::Rejection::Locked | engine::Rejection::UnverifiedEncounter => Error::Forbidden,
            _ => Error::InvalidInput,
        })?;
        self.domain = applied.state;
        self.events.extend(applied.events);
        Ok(())
    }
    /// Apply to a private candidate. Repositories persist the candidate and all
    /// side effects together or discard it on any failure.
    pub fn execute(
        &mut self,
        request: &Value,
        content: &Content,
        now: u64,
        session_id: &str,
    ) -> Result<Value, Error> {
        wire::validate("GameCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        if let Some(receipt) = self.receipts.get(id) {
            return if receipt.request == *request {
                Ok(receipt.response.clone())
            } else {
                Err(Error::RevisionConflict)
            };
        }
        if self.receipts.len() >= MAX_COMMANDS {
            return Err(Error::RateLimited);
        }
        if request["expectedRevision"].as_u64() != Some(self.revision) {
            return Err(Error::RevisionConflict);
        }
        let mut next = self.clone();
        let payload = &request["payload"];
        let domain_id = format!("api-{}", digest_id(id));
        match payload["type"].as_str().ok_or(Error::InvalidInput)? {
            "start_encounter" => {
                if next
                    .encounters
                    .values()
                    .any(|e| !e.completed && !e.abandoned)
                {
                    return Err(Error::RevisionConflict);
                }
                let quest = payload["beatId"].as_str().ok_or(Error::InvalidInput)?;
                let skill = content.quest_skill_id(quest).ok_or(Error::NotFound)?;
                // This development API exposes authored menu interactions in Meadow.
                // No browser position is promoted into trusted reachability.
                if next.domain.region() != "meadow" {
                    return Err(Error::Forbidden);
                }
                let encounter_id = format!("enc-{}", digest_id(&format!("{session_id}:{id}")));
                let seed = next.revision;
                let (_, encounter) = authored_encounter(skill, seed, &encounter_id)
                    .map_err(|_| Error::Unavailable)?;
                let template =
                    serde_json::to_value(&encounter.template_id).map_err(|_| Error::Unavailable)?;
                if template.as_str() != content.quest_template_id(quest) {
                    return Err(Error::Unavailable);
                }
                let mut facts = TrustedFacts::default();
                facts.reachable_quests.insert(quest.into());
                facts
                    .issued_encounters
                    .insert(encounter_id.clone(), quest.into());
                next.domain_apply(
                    Action::BeginQuest {
                        quest_id: quest.into(),
                        encounter_id: encounter_id.clone(),
                    },
                    &domain_id,
                    content,
                    facts,
                    now,
                )?;
                next.encounters.insert(
                    encounter_id.clone(),
                    SavedEncounter {
                        id: encounter_id.clone(),
                        quest: quest.into(),
                        skill: skill.into(),
                        seed,
                        support: encounter.support,
                        checks: 0,
                        completed: false,
                        abandoned: false,
                        session_id: session_id.into(),
                    },
                );
            }
            "attempt" | "hint" | "abandon" => {
                let encounter_id = payload["encounterId"].as_str().ok_or(Error::InvalidInput)?;
                let mut saved = next
                    .encounters
                    .get(encounter_id)
                    .cloned()
                    .ok_or(Error::NotFound)?;
                if saved.completed || saved.abandoned {
                    return Err(Error::RevisionConflict);
                }
                let (task, mut encounter) = authored_encounter(&saved.skill, saved.seed, &saved.id)
                    .map_err(|_| Error::Unavailable)?;
                let (kind, result) = if payload["type"] == "abandon" {
                    next.domain_apply(
                        Action::AbandonQuest {
                            quest_id: saved.quest.clone(),
                            encounter_id: saved.id.clone(),
                        },
                        &domain_id,
                        content,
                        TrustedFacts::default(),
                        now,
                    )?;
                    saved.abandoned = true;
                    ("abandonment", "not_assessed")
                } else if payload["type"] == "hint" {
                    let kind = match payload["supportKind"].as_str() {
                        Some("hint") => {
                            saved.support.hint_count = saved
                                .support
                                .hint_count
                                .checked_add(1)
                                .filter(|n| *n <= 20)
                                .ok_or(Error::RateLimited)?;
                            "hint"
                        }
                        Some("worked_example") => {
                            saved.support.worked_example = true;
                            "worked_example"
                        }
                        Some("read_aloud") => {
                            saved.support.read_aloud = true;
                            "hint"
                        }
                        _ => return Err(Error::InvalidInput),
                    };
                    (kind, "not_assessed")
                } else {
                    encounter.support = saved.support.clone();
                    let answer: wire::Answer = serde_json::from_value(payload["answer"].clone())
                        .map_err(|_| Error::InvalidInput)?;
                    let outcome =
                        assess_wire(&task, &encounter, &answer).map_err(|_| Error::InvalidInput)?;
                    saved.checks += 1;
                    if outcome.correct == Some(true) {
                        let mut facts = TrustedFacts::default();
                        facts.completed_encounters.insert(
                            saved.id.clone(),
                            VerifiedCompletion {
                                quest_id: saved.quest.clone(),
                                supported: outcome.classification
                                    != assessment::Classification::Independent,
                            },
                        );
                        next.domain_apply(
                            Action::CompleteQuest {
                                quest_id: saved.quest.clone(),
                                encounter_id: saved.id.clone(),
                            },
                            &domain_id,
                            content,
                            facts,
                            now,
                        )?;
                        saved.completed = true;
                    }
                    (
                        "answer_check",
                        match outcome.correct {
                            Some(true) => "correct",
                            Some(false) => "incorrect",
                            None => "not_assessed",
                        },
                    )
                };
                let sequence = next.evidence.len() as u64 + 1;
                let value = json!({"schemaVersion":"1.0","eventId":format!("evidence-{sequence}-{}",digest_id(&format!("{session_id}:{id}"))),"encounterId":saved.id,"skillId":saved.skill,"variantId":format!("authored-{}",saved.seed),"itemFamily":encounter.template_id,"contextId":saved.quest,"representation":representation(&saved.skill),"result":result,"eventKind":kind,"firstCheck":kind=="answer_check"&&saved.checks==1,"support":saved.support,"sessionId":saved.session_id,"serverSequence":sequence,"provenance":"server_verified"});
                wire::validate("LearningEvidence", &value).map_err(|_| Error::Unavailable)?;
                next.evidence
                    .push(serde_json::from_value(value.clone()).map_err(|_| Error::Unavailable)?);
                if saved.completed && kind == "answer_check" {
                    let mut completion = value;
                    completion["eventId"] = json!(format!(
                        "complete-{}",
                        digest_id(&format!("{session_id}:{id}"))
                    ));
                    completion["eventKind"] = json!("completion");
                    completion["firstCheck"] = json!(false);
                    completion["serverSequence"] = json!(next.evidence.len() as u64 + 1);
                    wire::validate("LearningEvidence", &completion)
                        .map_err(|_| Error::Unavailable)?;
                    next.evidence
                        .push(serde_json::from_value(completion).map_err(|_| Error::Unavailable)?);
                }
                next.encounters.insert(saved.id.clone(), saved);
            }
            "select_companion" => {
                let creature = payload["creatureId"].as_str().ok_or(Error::InvalidInput)?;
                next.domain_apply(
                    Action::SelectTeam {
                        creature_ids: vec![creature.into()],
                    },
                    &domain_id,
                    content,
                    TrustedFacts::default(),
                    now,
                )?;
            }
            _ => return Err(Error::InvalidInput),
        }
        next.revision = next.revision.checked_add(1).ok_or(Error::Unavailable)?;
        let response = next.snapshot(content)?;
        next.receipts.insert(
            id.into(),
            Receipt {
                request: request.clone(),
                response: response.clone(),
            },
        );
        *self = next;
        Ok(response)
    }
}
fn representation(skill: &str) -> &'static str {
    match skill {
        "M06" => "countable_planks",
        "M08" => "countable_objects",
        "M05" => "text_choices",
        "E05" => "letter_tiles",
        _ => "labelled_story_cards",
    }
}
pub fn digest_id(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))[..40].to_string()
}
