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

#[cfg(test)]
mod companion_tests;
pub mod http;
pub mod repository;
mod snapshot;
mod storage;

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
    #[serde(default)]
    training: Option<wire::TrainingBinding>,
    #[serde(default)]
    training_policy: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Receipt {
    request: Value,
    response: Value,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Aggregate {
    revision: u64,
    #[serde(default)]
    evidence_sequence: u64,
    #[serde(default)]
    compact_policy: Option<String>,
    #[serde(default)]
    authored_mastery: BTreeMap<String, bool>,
    #[serde(skip)]
    domain_records: Vec<Value>,
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
            evidence_sequence: 0,
            compact_policy: None,
            authored_mastery: BTreeMap::new(),
            domain_records: Vec::new(),
            domain: State::new(content),
            encounters: BTreeMap::new(),
            receipts: BTreeMap::new(),
            evidence: Vec::new(),
            events: Vec::new(),
        }
    }
    const COMPACT_POLICY: &str = "authored-no-transfer-independent-transfer-v1";
    fn update_authored_mastery(&mut self) -> Result<(), Error> {
        if self.compact_policy.is_none() {
            return Ok(());
        }
        if self.compact_policy.as_deref() != Some(Self::COMPACT_POLICY) {
            return Err(Error::Unavailable);
        }
        // This API factory always supplies novel_transfer=false. Under the pinned
        // policy secure is impossible; the public projection needs only these two
        // sufficient statistics. Full observations remain durable for policy replay.
        for e in &self.evidence {
            if e.provenance == wire::LearningEvidenceProvenance::ServerVerified {
                let attempted = self.authored_mastery.entry(e.skill_id.clone()).or_default();
                *attempted |= e.event_kind == wire::LearningEvidenceEventKind::AnswerCheck;
            }
        }
        Ok(())
    }
    fn compact(&self) -> Result<Self, Error> {
        let mut state = self.clone();
        state.compact_policy = Some(Self::COMPACT_POLICY.into());
        state.update_authored_mastery()?;
        state.evidence_sequence = state
            .evidence_sequence
            .max(state.evidence.last().map_or(0, |e| e.server_sequence));
        state.receipts.clear();
        state.events.clear();
        state.evidence.clear();
        state.domain_records.clear();
        state.encounters.retain(|_, e| !e.completed && !e.abandoned);
        state.domain = state.domain.compact();
        Ok(state)
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
        let context = Context {
            unix_millis: now.saturating_mul(1000),
            recorded_random_choice: 0,
            facts,
        };
        let applied = if self.compact_policy.is_some() {
            engine::apply_checkpointed(&self.domain, &command, content, &context)
        } else {
            engine::apply(&self.domain, &command, content, &context)
        }
        .map_err(|e| match e {
            engine::Rejection::RevisionConflict | engine::Rejection::CommandIdReused => {
                Error::RevisionConflict
            }
            engine::Rejection::NotFound => Error::NotFound,
            engine::Rejection::Locked | engine::Rejection::UnverifiedEncounter => Error::Forbidden,
            _ => Error::InvalidInput,
        })?;
        self.domain_records.push(json!({"command":command,"context":context,"events":applied.events,"rulesVersion":engine::RULES_VERSION,"contentVersion":content.version()}));
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
        let command: wire::GameCommand =
            serde_json::from_value(request.clone()).map_err(|_| Error::InvalidInput)?;
        let id = command.command_id.as_str();
        if let Some(receipt) = self.receipts.get(id) {
            return if receipt.request == *request {
                Ok(receipt.response.clone())
            } else {
                Err(Error::RevisionConflict)
            };
        }
        if self.compact_policy.is_none() && self.receipts.len() >= MAX_COMMANDS {
            return Err(Error::RateLimited);
        }
        if command.expected_revision != self.revision {
            return Err(Error::RevisionConflict);
        }
        let mut next = self.clone();
        let payload = &command.payload;
        let domain_id = format!("api-{}", digest_id(id));
        match payload {
            wire::CommandPayload::Variant1(payload) => {
                if next
                    .encounters
                    .values()
                    .any(|e| !e.completed && !e.abandoned)
                {
                    return Err(Error::RevisionConflict);
                }
                let quest = payload.beat_id.as_str();
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
                        training_policy: training_policy(content, quest),
                        training: content
                            .training(quest)
                            .map(|training| wire::TrainingBinding {
                                training_id: training.id.clone(),
                                creature_id: training.creature_id.clone(),
                                rules_version: content.training_rules_version().into(),
                            }),
                    },
                );
            }
            wire::CommandPayload::Variant2(payload) => {
                next.apply_encounter(
                    &payload.encounter_id,
                    EncounterAction::Attempt(&payload.answer),
                    id,
                    content,
                    now,
                    session_id,
                )?;
            }
            wire::CommandPayload::Variant3(payload) => {
                next.apply_encounter(
                    &payload.encounter_id,
                    EncounterAction::Support(&payload.support_kind),
                    id,
                    content,
                    now,
                    session_id,
                )?;
            }
            wire::CommandPayload::Variant5(payload) => {
                next.apply_encounter(
                    &payload.encounter_id,
                    EncounterAction::Abandon,
                    id,
                    content,
                    now,
                    session_id,
                )?;
            }
            wire::CommandPayload::Variant4(payload) => {
                let creature = &payload.creature_id;
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
            wire::CommandPayload::Variant8(payload) => {
                let creature_ids = payload.creature_ids.clone();
                next.domain_apply(
                    Action::SelectTeam { creature_ids },
                    &domain_id,
                    content,
                    TrustedFacts::default(),
                    now,
                )?;
            }
            wire::CommandPayload::Variant6(payload) => {
                next.domain_apply(
                    Action::HelpFriend {
                        creature_id: payload.creature_id.clone(),
                        completed_quest_id: payload.completed_quest_id.clone(),
                    },
                    &domain_id,
                    content,
                    TrustedFacts::default(),
                    now,
                )?;
            }
            wire::CommandPayload::Variant7(payload) => {
                next.domain_apply(
                    Action::Transform {
                        creature_id: payload.creature_id.clone(),
                        enabled: payload.enabled,
                    },
                    &domain_id,
                    content,
                    TrustedFacts::default(),
                    now,
                )?;
            }
        }
        next.revision = next.revision.checked_add(1).ok_or(Error::Unavailable)?;
        next.update_authored_mastery()?;
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
    fn apply_encounter(
        &mut self,
        encounter_id: &str,
        action: EncounterAction<'_>,
        id: &str,
        content: &Content,
        now: u64,
        session_id: &str,
    ) -> Result<(), Error> {
        let domain_id = format!("api-{}", digest_id(id));
        let mut saved = self
            .encounters
            .get(encounter_id)
            .cloned()
            .ok_or(Error::NotFound)?;
        if saved.completed || saved.abandoned {
            return Err(Error::RevisionConflict);
        }
        // Never silently evaluate a pending training instance under changed rules
        // or redirect its reward recipient after content or team changes.
        if !matches!(action, EncounterAction::Abandon)
            && let Some(binding) = &saved.training
        {
            let training = content
                .training(&binding.training_id)
                .ok_or(Error::Unavailable)?;
            if saved.training_policy != training_policy(content, &saved.quest)
                || binding.rules_version != content.training_rules_version()
                || binding.creature_id != training.creature_id
                || saved.quest != training.id
                || saved.skill != training.skill_id
            {
                return Err(Error::Unavailable);
            }
        }
        let (task, mut encounter) = authored_encounter(&saved.skill, saved.seed, &saved.id)
            .map_err(|_| Error::Unavailable)?;
        let (kind, result) = match action {
            EncounterAction::Abandon => {
                self.domain_apply(
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
            }
            EncounterAction::Support(support) => {
                let kind = match support {
                    wire::CommandPayloadVariant3SupportKind::Hint => {
                        saved.support.hint_count = saved
                            .support
                            .hint_count
                            .checked_add(1)
                            .filter(|n| *n <= 20)
                            .ok_or(Error::RateLimited)?;
                        "hint"
                    }
                    wire::CommandPayloadVariant3SupportKind::WorkedExample => {
                        saved.support.worked_example = true;
                        "worked_example"
                    }
                    wire::CommandPayloadVariant3SupportKind::ReadAloud => {
                        saved.support.read_aloud = true;
                        "hint"
                    }
                };
                (kind, "not_assessed")
            }
            EncounterAction::Attempt(answer) => {
                encounter.support = saved.support.clone();
                let outcome =
                    assess_wire(&task, &encounter, answer).map_err(|_| Error::InvalidInput)?;
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
                    self.domain_apply(
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
            }
        };
        let sequence = self
            .evidence_sequence
            .max(self.evidence.last().map_or(0, |e| e.server_sequence))
            + 1;
        let value = json!({"schemaVersion":"1.0","eventId":format!("evidence-{sequence}-{}",digest_id(&format!("{session_id}:{id}"))),"encounterId":saved.id,"skillId":saved.skill,"variantId":format!("authored-{}",saved.seed),"itemFamily":encounter.template_id,"contextId":saved.quest,"representation":representation(&saved.skill),"result":result,"eventKind":kind,"firstCheck":kind=="answer_check"&&saved.checks==1,"support":saved.support,"sessionId":saved.session_id,"serverSequence":sequence,"provenance":"server_verified"});
        wire::validate("LearningEvidence", &value).map_err(|_| Error::Unavailable)?;
        self.evidence
            .push(serde_json::from_value(value.clone()).map_err(|_| Error::Unavailable)?);
        if saved.completed && kind == "answer_check" {
            let mut completion = value;
            completion["eventId"] = json!(format!(
                "complete-{}",
                digest_id(&format!("{session_id}:{id}"))
            ));
            completion["eventKind"] = json!("completion");
            completion["firstCheck"] = json!(false);
            completion["serverSequence"] = json!(sequence + 1);
            wire::validate("LearningEvidence", &completion).map_err(|_| Error::Unavailable)?;
            self.evidence
                .push(serde_json::from_value(completion).map_err(|_| Error::Unavailable)?);
        }
        self.evidence_sequence = self
            .evidence
            .last()
            .map_or(self.evidence_sequence, |e| e.server_sequence);
        self.encounters.insert(saved.id.clone(), saved);
        Ok(())
    }
}
enum EncounterAction<'a> {
    Attempt(&'a wire::Answer),
    Support(&'a wire::CommandPayloadVariant3SupportKind),
    Abandon,
}

fn training_policy(content: &Content, quest: &str) -> Option<String> {
    content.training(quest).map(|training| {
        digest_id(
            &json!({
                "rulesVersion": content.training_rules_version(),
                "thresholds": content.character_level_thresholds(),
                "creatureId": training.creature_id,
                "skillId": training.skill_id,
                "templateId": training.template_id,
                "requiredLevel": training.required_level,
                "xp": training.xp,
                "reactionId": training.reaction_id,
            })
            .to_string(),
        )
    })
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
