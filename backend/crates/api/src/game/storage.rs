//! Bounded in-memory aggregate backed by transactionally durable receipts/history.
//!
//! These methods are internal repository boundaries, never browser inputs. The
//! repository must check the durable API receipt before calling `execute_persisted`,
//! insert novelty keys in the same transaction, and persist every returned delta
//! together with the compact aggregate before publishing its response.
use super::*;
use std::collections::BTreeSet;
use wonderwilds_learning::mastery::{self, IncrementalMastery, Observation, Status};

// A versioned envelope deliberately omits the old reader's required history
// fields. A pre-storage binary must reject compact saves, not silently treat
// emptied receipts/evidence as a fresh history and lose duplicate prevention.
impl Serialize for Aggregate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut output =
            serializer.serialize_struct("Aggregate", if self.storage.is_some() { 5 } else { 6 })?;
        if let Some(storage) = &self.storage {
            let envelope_version = if self.domain.requires_continuity_runtime() {
                CONTINUITY_ENVELOPE_VERSION
            } else if self.domain.requires_personal_quest_runtime() {
                PERSONAL_QUEST_ENVELOPE_VERSION
            } else {
                FORMAT_VERSION
            };
            output.serialize_field("storageFormat", &envelope_version)?;
            output.serialize_field("revision", &self.revision)?;
            output.serialize_field("domain", &self.domain)?;
            output.serialize_field("encounters", &self.encounters)?;
            output.serialize_field("storage", storage)?;
        } else {
            output.serialize_field("revision", &self.revision)?;
            output.serialize_field("domain", &self.domain)?;
            output.serialize_field("encounters", &self.encounters)?;
            output.serialize_field("receipts", &self.receipts)?;
            output.serialize_field("evidence", &self.evidence)?;
            output.serialize_field("events", &self.events)?;
        }
        output.end()
    }
}

impl<'de> Deserialize<'de> for Aggregate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Legacy {
            revision: u64,
            domain: State,
            encounters: BTreeMap<String, SavedEncounter>,
            receipts: BTreeMap<String, Receipt>,
            evidence: Vec<wire::LearningEvidence>,
            events: Vec<engine::Event>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Compact {
            #[serde(rename = "storageFormat")]
            storage_format: u32,
            revision: u64,
            domain: State,
            encounters: BTreeMap<String, SavedEncounter>,
            storage: StorageState,
        }
        let value = Value::deserialize(deserializer)?;
        let (revision, domain, encounters, receipts, evidence, events, storage) =
            if value.get("storageFormat").is_some() {
                let compact: Compact =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                if ![
                    FORMAT_VERSION,
                    PERSONAL_QUEST_ENVELOPE_VERSION,
                    CONTINUITY_ENVELOPE_VERSION,
                ]
                .contains(&compact.storage_format)
                    || compact.storage_format == FORMAT_VERSION
                        && compact.domain.requires_personal_quest_runtime()
                    || compact.storage_format < CONTINUITY_ENVELOPE_VERSION
                        && compact.domain.requires_continuity_runtime()
                {
                    return Err(serde::de::Error::custom(
                        "unsupported aggregate storage format",
                    ));
                }
                (
                    compact.revision,
                    compact.domain,
                    compact.encounters,
                    BTreeMap::new(),
                    Vec::new(),
                    Vec::new(),
                    Some(compact.storage),
                )
            } else {
                let legacy: Legacy =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                (
                    legacy.revision,
                    legacy.domain,
                    legacy.encounters,
                    legacy.receipts,
                    legacy.evidence,
                    legacy.events,
                    None,
                )
            };
        Ok(Self {
            revision,
            domain,
            encounters,
            receipts,
            evidence,
            events,
            storage,
            durable_domain_lookup: None,
            pending_domain_receipts: Vec::new(),
            pending_domain_replays: Vec::new(),
        })
    }
}

pub(crate) const FORMAT_VERSION: u32 = 1;
const PERSONAL_QUEST_ENVELOPE_VERSION: u32 = 2;
const CONTINUITY_ENVELOPE_VERSION: u32 = 3;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StorageState {
    pub format_version: u32,
    pub evidence_sequence: u64,
    pub domain_revision: u64,
    pub domain_sequence: u64,
    pub mastery: BTreeMap<String, IncrementalMastery>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MasteryItem {
    pub skill_id: String,
    pub item_family: String,
    pub parameter_fingerprint: String,
}
impl MasteryItem {
    fn from_observation(observation: &Observation) -> Self {
        Self {
            skill_id: observation.evidence.skill_id.clone(),
            item_family: observation.evidence.item_family.clone(),
            parameter_fingerprint: observation.parameter_fingerprint.clone(),
        }
    }
}

pub(crate) struct StorageMigration {
    pub aggregate: Aggregate,
    pub api_receipts: Vec<(String, Value, Value)>,
    pub domain_receipts: Vec<engine::DomainReceipt>,
    pub encounters: Vec<SavedEncounter>,
    pub evidence: Vec<wire::LearningEvidence>,
    pub events: Vec<engine::Event>,
    pub mastery_items: Vec<MasteryItem>,
}

pub(crate) struct StorageHistory {
    pub api_receipts: Vec<(String, Value, Value)>,
    pub domain_receipts: Vec<engine::DomainReceipt>,
    pub encounters: Vec<SavedEncounter>,
    pub evidence: Vec<wire::LearningEvidence>,
    pub events: Vec<engine::Event>,
}

pub(crate) struct StorageDelta {
    pub response: Value,
    pub domain_receipts: Vec<engine::DomainReceipt>,
    pub domain_replays: Vec<Value>,
    pub encounters: Vec<SavedEncounter>,
    pub evidence: Vec<wire::LearningEvidence>,
    pub events: Vec<engine::Event>,
}

pub(super) fn replay_value(
    command: &Command,
    context: &Context,
    events: &[engine::Event],
) -> Result<Value, Error> {
    serde_json::to_value(engine::ReplayEntry {
        command: command.clone(),
        context: context.clone(),
        events: events.to_vec(),
    })
    .map_err(|_| Error::Unavailable)
}

fn summary_value(skill_id: &str, summary: mastery::Summary) -> Value {
    let status = match summary.status {
        Status::Unseen => "unseen",
        Status::Introduced => "introduced",
        Status::Practising => "practising",
        Status::Secure => "secure",
    };
    json!({"skillId":skill_id,"status":status,"explanation":summary.explanation})
}

impl Aggregate {
    pub(crate) fn is_storage_checkpoint(&self) -> bool {
        self.storage.is_some()
    }

    pub(crate) fn evidence_sequence(&self) -> u64 {
        self.storage
            .as_ref()
            .map_or(self.evidence.len() as u64, |s| s.evidence_sequence)
    }

    pub(super) fn next_evidence_sequence(&self) -> Result<u64, Error> {
        self.evidence_sequence()
            .checked_add(1)
            .ok_or(Error::Unavailable)
    }

    pub(super) fn append_evidence(
        &mut self,
        evidence: wire::LearningEvidence,
    ) -> Result<(), Error> {
        if evidence.server_sequence != self.next_evidence_sequence()? {
            return Err(Error::Unavailable);
        }
        if let Some(storage) = &mut self.storage {
            storage.evidence_sequence = evidence.server_sequence;
        }
        self.evidence.push(evidence);
        Ok(())
    }

    fn observations_for(
        &self,
        evidence: &[wire::LearningEvidence],
    ) -> Result<Vec<Observation>, Error> {
        evidence
            .iter()
            .map(|evidence| {
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
                Ok(Observation {
                    evidence: evidence.clone(),
                    parameter_fingerprint: digest_id(&parameters),
                    // Draft practice never manufactures educator-approved transfer.
                    novel_transfer: false,
                })
            })
            .collect()
    }

    pub(super) fn learning_summaries(&self) -> Result<Vec<Value>, Error> {
        if let Some(storage) = &self.storage {
            return storage
                .mastery
                .iter()
                .map(|(skill, projection)| {
                    Ok(summary_value(
                        skill,
                        projection.summary(2).map_err(|_| Error::Unavailable)?,
                    ))
                })
                .collect();
        }
        let observations = self.observations_for(&self.evidence)?;
        let skills: BTreeSet<_> = self.evidence.iter().map(|e| &e.skill_id).collect();
        skills
            .into_iter()
            .map(|skill| {
                Ok(summary_value(
                    skill,
                    mastery::recompute(skill, &observations, 2).map_err(|_| Error::Unavailable)?,
                ))
            })
            .collect()
    }

    pub(super) fn update_storage_mastery(
        &mut self,
        novelty: &mut NoveltyLookup<'_>,
    ) -> Result<(), Error> {
        let observations = self.observations_for(&self.evidence)?;
        let storage = self.storage.as_mut().ok_or(Error::Unavailable)?;
        for observation in observations {
            let novel = if mastery::is_novelty_candidate(&observation) {
                novelty(&MasteryItem::from_observation(&observation))?
            } else {
                false
            };
            storage
                .mastery
                .entry(observation.evidence.skill_id.clone())
                .or_insert_with(|| IncrementalMastery::new(&observation.evidence.skill_id))
                .observe(&observation, novel)
                .map_err(|_| Error::Unavailable)?;
        }
        Ok(())
    }

    pub(crate) fn validate_storage(&self, content: &Content) -> Result<(), Error> {
        self.validate_pending()?;
        let storage = self.storage.as_ref().ok_or(Error::Unavailable)?;
        if storage.format_version != FORMAT_VERSION
            || !self.receipts.is_empty()
            || !self.evidence.is_empty()
            || !self.events.is_empty()
            || self.encounters.len() > 1
            || self
                .encounters
                .iter()
                .any(|(id, e)| id != &e.id || e.completed || e.abandoned)
            || storage.mastery.len() > 200
            || storage.domain_revision > self.revision
        {
            return Err(Error::Unavailable);
        }
        if storage
            .mastery
            .values()
            .filter_map(IncrementalMastery::last_sequence)
            .max()
            .unwrap_or(0)
            != storage.evidence_sequence
        {
            return Err(Error::Unavailable);
        }
        self.domain
            .validate_checkpoint(storage.domain_revision, storage.domain_sequence)
            .map_err(|_| Error::Unavailable)?;
        for (skill, projection) in &storage.mastery {
            projection.validate().map_err(|_| Error::Unavailable)?;
            if projection.skill_id() != skill
                || projection
                    .last_sequence()
                    .is_some_and(|s| s > storage.evidence_sequence)
            {
                return Err(Error::Unavailable);
            }
        }
        self.snapshot(content)?;
        Ok(())
    }

    fn validate_pending(&self) -> Result<(), Error> {
        let pending: BTreeMap<_, _> = self
            .encounters
            .values()
            .filter(|encounter| !encounter.completed && !encounter.abandoned)
            .map(|encounter| (encounter.quest.clone(), encounter.id.clone()))
            .collect();
        if &pending != self.domain.pending_encounters() {
            return Err(Error::Unavailable);
        }
        Ok(())
    }

    fn validate_legacy_history(&self) -> Result<(), Error> {
        self.validate_pending()?;
        if self.storage.is_some()
            || self.domain.is_storage_checkpoint()
            || self.receipts.len() as u64 != self.revision
            || self.domain.revision() > self.revision
            || self
                .encounters
                .values()
                .filter(|e| !e.completed && !e.abandoned)
                .count()
                > 1
        {
            return Err(Error::Unavailable);
        }
        let mut revisions = BTreeSet::new();
        for (id, receipt) in &self.receipts {
            let revision = receipt.request["expectedRevision"]
                .as_u64()
                .ok_or(Error::Unavailable)?;
            if receipt.request["commandId"].as_str() != Some(id)
                || revision >= self.revision
                || !revisions.insert(revision)
                || receipt.response["revision"].as_u64() != revision.checked_add(1)
            {
                return Err(Error::Unavailable);
            }
        }
        for (id, encounter) in &self.encounters {
            if id != &encounter.id || encounter.completed && encounter.abandoned {
                return Err(Error::Unavailable);
            }
            let records: Vec<_> = self
                .evidence
                .iter()
                .filter(|e| e.encounter_id == *id)
                .collect();
            if records
                .iter()
                .any(|e| e.session_id != encounter.session_id || e.skill_id != encounter.skill)
                || records
                    .iter()
                    .filter(|e| e.event_kind == wire::LearningEvidenceEventKind::AnswerCheck)
                    .count() as u64
                    != encounter.checks
            {
                return Err(Error::Unavailable);
            }
            if let Some(last) = records.last() {
                if last.support != encounter.support
                    || encounter.completed
                        != (last.event_kind == wire::LearningEvidenceEventKind::Completion)
                    || encounter.abandoned
                        != (last.event_kind == wire::LearningEvidenceEventKind::Abandonment)
                {
                    return Err(Error::Unavailable);
                }
            } else if encounter.checks != 0
                || encounter.support
                    != (wire::SupportState {
                        hint_count: 0,
                        worked_example: false,
                        read_aloud: false,
                    })
                || encounter.completed
                || encounter.abandoned
            {
                return Err(Error::Unavailable);
            }
        }
        Ok(())
    }

    /// Upgrade an old aggregate under the profile lock. The caller inserts all
    /// returned durable records before committing `aggregate` as the new format.
    pub(crate) fn into_storage(mut self, content: &Content) -> Result<StorageMigration, Error> {
        if self.storage.is_some() {
            self.validate_storage(content)?;
            return Ok(StorageMigration {
                aggregate: self,
                api_receipts: Vec::new(),
                domain_receipts: Vec::new(),
                encounters: Vec::new(),
                evidence: Vec::new(),
                events: Vec::new(),
                mastery_items: Vec::new(),
            });
        }
        self.validate_legacy_history()?;
        let previous_snapshot = self.snapshot(content)?;
        let observations = self.observations_for(&self.evidence)?;
        let mut mastery = BTreeMap::new();
        let mut items = BTreeSet::new();
        for (index, observation) in observations.iter().enumerate() {
            wire::validate(
                "LearningEvidence",
                &serde_json::to_value(&observation.evidence).map_err(|_| Error::Unavailable)?,
            )
            .map_err(|_| Error::Unavailable)?;
            if observation.evidence.server_sequence != index as u64 + 1 {
                return Err(Error::Unavailable);
            }
            let novel = mastery::is_novelty_candidate(observation)
                && items.insert(MasteryItem::from_observation(observation));
            mastery
                .entry(observation.evidence.skill_id.clone())
                .or_insert_with(|| IncrementalMastery::new(&observation.evidence.skill_id))
                .observe(observation, novel)
                .map_err(|_| Error::Unavailable)?;
        }
        let mut api_receipts = Vec::new();
        for (id, receipt) in std::mem::take(&mut self.receipts) {
            wire::validate("GameCommand", &receipt.request).map_err(|_| Error::Unavailable)?;
            wire::validate("GameSnapshot", &receipt.response).map_err(|_| Error::Unavailable)?;
            if receipt.request["commandId"].as_str() != Some(&id) {
                return Err(Error::Unavailable);
            }
            api_receipts.push((id, receipt.request, receipt.response));
        }
        let (domain, domain_receipts) = self
            .domain
            .storage_checkpoint()
            .map_err(|_| Error::Unavailable)?;
        let mut receipt_events: Vec<_> = domain_receipts
            .iter()
            .flat_map(|receipt| receipt.events.iter().cloned())
            .collect();
        receipt_events.sort_by_key(|event| event.sequence);
        if receipt_events != self.events {
            return Err(Error::Unavailable);
        }
        self.domain = domain;
        self.storage = Some(StorageState {
            format_version: FORMAT_VERSION,
            evidence_sequence: self.evidence.len() as u64,
            domain_revision: self.domain.revision(),
            domain_sequence: self.domain.event_sequence(),
            mastery,
        });
        let encounters = self.encounters.values().cloned().collect();
        self.encounters
            .retain(|id, e| id == &e.id && !e.completed && !e.abandoned);
        let evidence = std::mem::take(&mut self.evidence);
        let events = std::mem::take(&mut self.events);
        self.validate_storage(content)?;
        if self.snapshot(content)? != previous_snapshot {
            return Err(Error::Unavailable);
        }
        Ok(StorageMigration {
            aggregate: self,
            api_receipts,
            domain_receipts,
            encounters,
            evidence,
            events,
            mastery_items: items.into_iter().collect(),
        })
    }

    /// Reconstruct a legacy-format export/rollback candidate from complete durable
    /// history. The caller must obtain a consistent locked database snapshot.
    pub(crate) fn restore_legacy_history(
        &self,
        content: &Content,
        history: StorageHistory,
    ) -> Result<Self, Error> {
        self.validate_storage(content)?;
        if history.evidence.len() as u64 != self.evidence_sequence()
            || history.events.len() as u64 != self.domain.event_sequence()
        {
            return Err(Error::Unavailable);
        }
        let expected = self.snapshot(content)?;
        let mut restored = self.clone();
        restored.domain = self
            .domain
            .restore_embedded_receipts(history.domain_receipts)
            .map_err(|_| Error::Unavailable)?;
        restored.storage = None;
        restored.encounters.clear();
        for encounter in history.encounters {
            if restored
                .encounters
                .insert(encounter.id.clone(), encounter)
                .is_some()
            {
                return Err(Error::Unavailable);
            }
        }
        for (id, request, response) in history.api_receipts {
            if restored
                .receipts
                .insert(id, Receipt { request, response })
                .is_some()
            {
                return Err(Error::Unavailable);
            }
        }
        restored.evidence = history.evidence;
        restored.events = history.events;
        restored.validate_legacy_history()?;
        // Run the complete migration verifier again, including receipt/event
        // correspondence and golden mastery summary equality.
        restored.clone().into_storage(content)?;
        if restored.snapshot(content)? != expected {
            return Err(Error::Unavailable);
        }
        Ok(restored)
    }

    /// The repository has checked the durable API receipt and must persist the
    /// delta, novelty inserts and new compact checkpoint atomically.
    pub(crate) fn execute_persisted(
        &mut self,
        request: &Value,
        content: &Content,
        now: u64,
        session_id: &str,
        domain_lookup: Option<&engine::DomainReceipt>,
        novelty: &mut NoveltyLookup<'_>,
    ) -> Result<StorageDelta, Error> {
        self.validate_storage(content)?;
        let mut next = self.clone();
        next.durable_domain_lookup = domain_lookup.cloned();
        let response = next.execute_inner(request, content, now, session_id, Some(novelty))?;
        let delta = StorageDelta {
            response,
            domain_receipts: std::mem::take(&mut next.pending_domain_receipts),
            domain_replays: std::mem::take(&mut next.pending_domain_replays),
            encounters: next.encounters.values().cloned().collect(),
            evidence: std::mem::take(&mut next.evidence),
            events: std::mem::take(&mut next.events),
        };
        next.encounters.retain(|_, e| !e.completed && !e.abandoned);
        next.durable_domain_lookup = None;
        next.validate_storage(content)?;
        *self = next;
        Ok(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(game: &Aggregate, id: &str, payload: Value) -> Value {
        json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":game.revision,"payload":payload})
    }

    #[test]
    fn legacy_migration_and_reconstruction_preserve_pending_support_and_exact_snapshot() {
        let content = development_content().unwrap();
        let mut legacy = super::super::companion_tests::recruited_pip_via_api(&content);
        let started = legacy
            .execute(
                &request(
                    &legacy,
                    "pending-start",
                    json!({"type":"start_encounter","beatId":"training-pip-1"}),
                ),
                &content,
                10,
                "session",
            )
            .unwrap();
        let id = started["pendingEncounters"][0]["id"]
            .as_str()
            .unwrap()
            .to_string();
        legacy
            .execute(
                &request(
                    &legacy,
                    "pending-hint",
                    json!({"type":"hint","encounterId":id,"supportKind":"hint"}),
                ),
                &content,
                11,
                "session",
            )
            .unwrap();
        let old = serde_json::to_value(&legacy).unwrap();
        let snapshot = legacy.snapshot(&content).unwrap();
        let migration = legacy.into_storage(&content).unwrap();
        let mut compact = migration.aggregate;
        assert_eq!(compact.snapshot(&content).unwrap(), snapshot);
        assert!(
            compact.evidence.is_empty() && compact.events.is_empty() && compact.receipts.is_empty()
        );
        assert_eq!(compact.encounters.len(), 1);
        let restored = compact
            .restore_legacy_history(
                &content,
                StorageHistory {
                    api_receipts: migration.api_receipts,
                    domain_receipts: migration.domain_receipts,
                    encounters: migration.encounters,
                    evidence: migration.evidence,
                    events: migration.events,
                },
            )
            .unwrap();
        assert_eq!(serde_json::to_value(&restored).unwrap(), old);
        let encounter = &snapshot["pendingEncounters"][0];
        let answer = encounter["parameters"]["total"].as_u64().unwrap()
            - encounter["parameters"]["given"].as_u64().unwrap();
        let completion = request(
            &compact,
            "pending-answer",
            json!({"type":"attempt","encounterId":id,"answer":{"kind":"quantity","value":answer}}),
        );
        let mut normal = restored;
        let expected = normal
            .execute(&completion, &content, 12, "session")
            .unwrap();
        let delta = compact
            .execute_persisted(&completion, &content, 12, "session", None, &mut |_| {
                Ok(true)
            })
            .unwrap();
        assert_eq!(delta.response, expected);
        assert_eq!(delta.evidence.len(), 2);
        assert_eq!(delta.domain_receipts.len(), 1);
        assert_eq!(delta.domain_replays.len(), 1);
        assert!(delta.domain_replays[0]["context"]["facts"]["completed_encounters"].is_object());
        assert_eq!(compact.domain.character_xp("pip"), 10);
        assert!(compact.encounters.is_empty());
        assert!(compact.evidence.is_empty() && compact.events.is_empty());
        assert!(compact.evidence_sequence() > snapshot["revision"].as_u64().unwrap() / 2);
    }

    #[test]
    fn compact_hot_state_stays_bounded_beyond_old_lifetime_cap() {
        let content = development_content().unwrap();
        let mut compact = Aggregate::new(&content)
            .into_storage(&content)
            .unwrap()
            .aggregate;
        let initial_size = serde_json::to_vec(&compact).unwrap().len();
        for n in 0..MAX_COMMANDS + 101 {
            let command = request(
                &compact,
                &format!("long-{n}"),
                json!({"type":"select_team","creatureIds":[]}),
            );
            let delta = compact
                .execute_persisted(&command, &content, n as u64, "session", None, &mut |_| {
                    Ok(true)
                })
                .unwrap();
            assert_eq!(delta.domain_receipts.len(), 1);
            assert_eq!(delta.events.len(), 1);
            assert_eq!(delta.response["revision"], n + 1);
            if n % 256 == 0 {
                compact = serde_json::from_value(serde_json::to_value(&compact).unwrap()).unwrap();
                compact.validate_storage(&content).unwrap();
            }
        }
        assert!(serde_json::to_vec(&compact).unwrap().len() < initial_size + 100);
        assert_eq!(compact.evidence_sequence(), 0);
        assert!(
            compact.receipts.is_empty() && compact.evidence.is_empty() && compact.events.is_empty()
        );
        let legacy_command = request(
            &compact,
            "ordinary-not-allowed",
            json!({"type":"select_team","creatureIds":[]}),
        );
        assert_eq!(
            compact.execute(&legacy_command, &content, 1, "session"),
            Err(Error::Unavailable)
        );
    }

    #[test]
    fn corrupt_legacy_receipts_events_and_encounter_counters_fail_closed() {
        let content = development_content().unwrap();
        let original = super::super::companion_tests::recruited_pip_via_api(&content);
        let mut missing = original.clone();
        missing.receipts.pop_first();
        assert!(matches!(
            missing.into_storage(&content),
            Err(Error::Unavailable)
        ));
        let mut wrong_response = original.clone();
        wrong_response
            .receipts
            .first_entry()
            .unwrap()
            .get_mut()
            .response["revision"] = json!(999);
        assert!(matches!(
            wrong_response.into_storage(&content),
            Err(Error::Unavailable)
        ));
        let mut wrong_events = original.clone();
        wrong_events.events[0].sequence = 99;
        assert!(matches!(
            wrong_events.into_storage(&content),
            Err(Error::Unavailable)
        ));
        let mut wrong_checks = original.clone();
        wrong_checks
            .encounters
            .first_entry()
            .unwrap()
            .get_mut()
            .checks = 0;
        assert!(matches!(
            wrong_checks.into_storage(&content),
            Err(Error::Unavailable)
        ));
        let mut wrong_session = original;
        wrong_session
            .encounters
            .first_entry()
            .unwrap()
            .get_mut()
            .session_id = "wrong".into();
        assert!(matches!(
            wrong_session.into_storage(&content),
            Err(Error::Unavailable)
        ));
    }

    #[test]
    fn incremental_practice_matches_full_history_and_novelty_failure_is_atomic() {
        let content = development_content().unwrap();
        let mut normal = super::super::companion_tests::recruited_pip_via_api(&content);
        let migration = normal.clone().into_storage(&content).unwrap();
        let mut compact = migration.aggregate;
        let mut items: BTreeSet<_> = migration.mastery_items.into_iter().collect();
        for n in 0..40 {
            let start = request(
                &normal,
                &format!("practice-start-{n}"),
                json!({"type":"start_encounter","beatId":"training-pip-1"}),
            );
            let expected = normal.execute(&start, &content, n, "session").unwrap();
            assert_eq!(
                compact
                    .execute_persisted(&start, &content, n, "session", None, &mut |item| Ok(
                        items.insert(item.clone())
                    ))
                    .unwrap()
                    .response,
                expected
            );
            let encounter = &expected["pendingEncounters"][0];
            let answer = encounter["parameters"]["total"].as_u64().unwrap()
                - encounter["parameters"]["given"].as_u64().unwrap();
            let attempt = request(
                &normal,
                &format!("practice-answer-{n}"),
                json!({"type":"attempt","encounterId":encounter["id"],"answer":{"kind":"quantity","value":answer}}),
            );
            if n == 0 {
                let before = serde_json::to_value(&compact).unwrap();
                assert!(matches!(
                    compact.execute_persisted(
                        &attempt,
                        &content,
                        n,
                        "session",
                        None,
                        &mut |_| Err(Error::Unavailable)
                    ),
                    Err(Error::Unavailable)
                ));
                assert_eq!(serde_json::to_value(&compact).unwrap(), before);
            }
            let expected = normal.execute(&attempt, &content, n, "session").unwrap();
            assert_eq!(
                compact
                    .execute_persisted(&attempt, &content, n, "session", None, &mut |item| Ok(
                        items.insert(item.clone())
                    ))
                    .unwrap()
                    .response,
                expected
            );
            compact = serde_json::from_value(serde_json::to_value(&compact).unwrap()).unwrap();
            assert_eq!(
                compact.snapshot(&content).unwrap(),
                normal.snapshot(&content).unwrap()
            );
        }
        assert_eq!(compact.domain.character_xp("pip"), 10);
        assert!(
            compact.evidence.is_empty()
                && compact.events.is_empty()
                && compact.encounters.is_empty()
        );
    }

    #[test]
    fn orphaned_domain_pending_and_mismatched_active_encounters_fail_closed() {
        let content = development_content().unwrap();
        let mut legacy = Aggregate::new(&content);
        let command = request(
            &legacy,
            "pending-story",
            json!({"type":"start_encounter","beatId":"meadow-welcome"}),
        );
        legacy.execute(&command, &content, 1, "session").unwrap();
        let mut orphan = legacy.clone();
        orphan.encounters.clear();
        assert!(matches!(
            orphan.into_storage(&content),
            Err(Error::Unavailable)
        ));
        let compact = legacy.into_storage(&content).unwrap().aggregate;
        let mut orphan = compact.clone();
        orphan.encounters.clear();
        assert_eq!(orphan.validate_storage(&content), Err(Error::Unavailable));
        let mut wrong_quest = compact;
        wrong_quest
            .encounters
            .first_entry()
            .unwrap()
            .get_mut()
            .quest = "meadow-bridge".into();
        assert_eq!(
            wrong_quest.validate_storage(&content),
            Err(Error::Unavailable)
        );
    }

    #[test]
    fn compact_envelope_is_rejected_by_pre_storage_readers_and_legacy_stays_compatible() {
        #[derive(Deserialize)]
        struct PreStorageRequiredHistory {
            #[serde(rename = "receipts")]
            _receipts: BTreeMap<String, Value>,
            #[serde(rename = "evidence")]
            _evidence: Vec<Value>,
            #[serde(rename = "events")]
            _events: Vec<Value>,
        }
        let content = development_content().unwrap();
        let legacy = Aggregate::new(&content);
        let legacy_json = serde_json::to_value(&legacy).unwrap();
        assert!(serde_json::from_value::<PreStorageRequiredHistory>(legacy_json.clone()).is_ok());
        assert!(serde_json::from_value::<Aggregate>(legacy_json.clone()).is_ok());
        let mut missing_legacy = legacy_json;
        missing_legacy.as_object_mut().unwrap().remove("receipts");
        assert!(serde_json::from_value::<Aggregate>(missing_legacy).is_err());
        let compact = legacy.into_storage(&content).unwrap().aggregate;
        let compact_json = serde_json::to_value(&compact).unwrap();
        assert_eq!(compact_json["storageFormat"], FORMAT_VERSION);
        assert!(compact_json.get("receipts").is_none());
        assert!(compact_json.get("evidence").is_none());
        assert!(compact_json.get("events").is_none());
        // Old readers ignore new fields, but still require their original
        // history fields: they must fail rather than load emptied history.
        assert!(serde_json::from_value::<PreStorageRequiredHistory>(compact_json.clone()).is_err());
        let reloaded: Aggregate = serde_json::from_value(compact_json.clone()).unwrap();
        reloaded.validate_storage(&content).unwrap();
        let mut unknown = compact_json.clone();
        unknown["storageFormat"] = json!(999);
        assert!(serde_json::from_value::<Aggregate>(unknown).is_err());
        let mut lost_marker = compact_json;
        lost_marker.as_object_mut().unwrap().remove("storageFormat");
        assert!(serde_json::from_value::<Aggregate>(lost_marker).is_err());
    }

    #[test]
    fn unknown_checkpoint_version_and_counters_fail_closed() {
        let content = development_content().unwrap();
        let compact = Aggregate::new(&content)
            .into_storage(&content)
            .unwrap()
            .aggregate;
        for (field, value) in [
            ("format_version", json!(99)),
            ("domain_sequence", json!(1)),
            ("domain_revision", json!(1)),
        ] {
            let mut saved = serde_json::to_value(&compact).unwrap();
            saved["storage"][field] = value;
            let bad: Aggregate = serde_json::from_value(saved).unwrap();
            assert_eq!(bad.validate_storage(&content), Err(Error::Unavailable));
        }
    }
}
