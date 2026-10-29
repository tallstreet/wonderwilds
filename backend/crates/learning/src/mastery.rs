//! Versioned experimental mastery policy. Inputs are server records, never browser claims.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wonderwilds_contracts::{
    LearningEvidence, LearningEvidenceEventKind as Kind, LearningEvidenceProvenance as Provenance,
    LearningEvidenceResult as ResultKind,
};

pub const POLICY_VERSION: &str = "independent-transfer-v1";

/// Filled by the server encounter factory from canonical parameters and transfer design.
/// This metadata must never be accepted from an attempt request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub evidence: LearningEvidence,
    pub parameter_fingerprint: String,
    pub novel_transfer: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Unseen,
    Introduced,
    Practising,
    Secure,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub status: Status,
    pub eligible_checks: usize,
    pub correct_checks: usize,
    pub explanation: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidRecord,
    DuplicateEvent,
    SequenceOrder,
}

/// A versioned, bounded projection of one skill's complete evidence history.
///
/// The caller must durably validate event identity/order and each encounter's
/// support/first-check history before calling `observe`. Item identity is also
/// external: the storage transaction inserts a unique (profile, skill, family,
/// fingerprint) only when `is_novelty_candidate` is true. Its insertion result is
/// passed as `novel_item`. That durable ledger must outlive this rolling window;
/// remembering only the last eight items would manufacture independent evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IncrementalMastery {
    policy_version: String,
    skill_id: String,
    last_sequence: Option<u64>,
    seen: bool,
    attempted: bool,
    window: Vec<Observation>,
}

/// Whether this observation consumes durable item novelty. Supported practice,
/// unassessed answers and later checks leave a later independent item eligible.
pub fn is_novelty_candidate(record: &Observation) -> bool {
    let e = &record.evidence;
    e.provenance == Provenance::ServerVerified
        && e.event_kind == Kind::AnswerCheck
        && e.first_check
        && e.support.hint_count == 0
        && !e.support.worked_example
        && !e.support.read_aloud
        && e.result != ResultKind::NotAssessed
}

impl IncrementalMastery {
    pub fn new(skill_id: &str) -> Self {
        Self {
            policy_version: POLICY_VERSION.into(),
            skill_id: skill_id.into(),
            last_sequence: None,
            seen: false,
            attempted: false,
            window: Vec::new(),
        }
    }

    pub fn skill_id(&self) -> &str {
        &self.skill_id
    }

    pub fn last_sequence(&self) -> Option<u64> {
        self.last_sequence
    }

    /// Validate every loaded checkpoint before consuming it. Deserialization is
    /// deliberately not treated as validation of bounds or policy compatibility.
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.policy_version != POLICY_VERSION
            || !bounded_text(&self.skill_id, 120)
            || self.window.len() > 8
            || self.last_sequence == Some(0)
            || self.attempted && !self.seen
            || self.seen && self.last_sequence.is_none()
            || !self.window.is_empty() && !self.attempted
        {
            return Err(EvidenceError::InvalidRecord);
        }
        let mut previous = None;
        let mut items = BTreeSet::new();
        let mut encounters = BTreeSet::new();
        let mut events = BTreeSet::new();
        for record in &self.window {
            validate_checkpoint_observation(record)?;
            let e = &record.evidence;
            if e.skill_id != self.skill_id
                || !is_novelty_candidate(record)
                || previous.is_some_and(|sequence| e.server_sequence <= sequence)
                || self
                    .last_sequence
                    .is_none_or(|sequence| e.server_sequence > sequence)
                || !items.insert((&e.item_family, &record.parameter_fingerprint))
                || !encounters.insert(&e.encounter_id)
                || !events.insert(&e.event_id)
            {
                return Err(EvidenceError::InvalidRecord);
            }
            previous = Some(e.server_sequence);
        }
        Ok(())
    }

    /// Consume a server-validated observation and its transactionally durable
    /// item novelty result. Errors leave this projection untouched.
    pub fn observe(&mut self, record: &Observation, novel_item: bool) -> Result<(), EvidenceError> {
        self.validate()?;
        validate_checkpoint_observation(record)?;
        let e = &record.evidence;
        if e.skill_id != self.skill_id || novel_item && !is_novelty_candidate(record) {
            return Err(EvidenceError::InvalidRecord);
        }
        if self
            .last_sequence
            .is_some_and(|previous| e.server_sequence <= previous)
        {
            return Err(EvidenceError::SequenceOrder);
        }
        if novel_item
            && self.window.iter().any(|prior| {
                prior.evidence.encounter_id == e.encounter_id
                    || prior.evidence.event_id == e.event_id
                    || prior.evidence.item_family == e.item_family
                        && prior.parameter_fingerprint == record.parameter_fingerprint
            })
        {
            return Err(EvidenceError::InvalidRecord);
        }
        self.last_sequence = Some(e.server_sequence);
        if e.provenance == Provenance::ServerVerified {
            self.seen = true;
            self.attempted |= e.event_kind == Kind::AnswerCheck;
            if novel_item {
                if self.window.len() == 8 {
                    self.window.remove(0);
                }
                self.window.push(record.clone());
            }
        }
        Ok(())
    }

    pub fn summary(&self, required_representations: usize) -> Result<Summary, EvidenceError> {
        self.validate()?;
        let mut summary = recompute(&self.skill_id, &self.window, required_representations)?;
        if self.window.is_empty() {
            summary.status = if self.attempted {
                Status::Practising
            } else if self.seen {
                Status::Introduced
            } else {
                Status::Unseen
            };
            summary.explanation = status_explanation(summary.status);
        }
        Ok(summary)
    }
}

fn bounded_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.chars().count() <= maximum
}

fn validate_checkpoint_observation(record: &Observation) -> Result<(), EvidenceError> {
    let e = &record.evidence;
    if [
        &e.event_id,
        &e.session_id,
        &e.encounter_id,
        &e.skill_id,
        &e.variant_id,
        &e.item_family,
        &e.context_id,
        &e.representation,
    ]
    .iter()
    .any(|value| !bounded_text(value, 120))
        || !bounded_text(&record.parameter_fingerprint, 256)
        || e.server_sequence == 0
        || e.support.hint_count > 20
        || e.event_kind == Kind::Hint && e.support.hint_count == 0 && !e.support.read_aloud
        || e.event_kind == Kind::WorkedExample && !e.support.worked_example
    {
        return Err(EvidenceError::InvalidRecord);
    }
    Ok(())
}

fn status_explanation(status: Status) -> &'static str {
    match status {
        Status::Secure => {
            "Secure in these activities under the experimental transfer policy; optional enrichment is available."
        }
        Status::Practising => {
            "Practised; support and repeated items do not establish independent understanding."
        }
        Status::Introduced => "Introduced; no qualifying independent answer checks yet.",
        Status::Unseen => "No verified server evidence; imported practice remains unverified.",
    }
}

/// Recompute from ordered, immutable server history. No elapsed-time demotion.
/// Representations are a reviewed per-node choice: use one only where a second is invalid.
pub fn recompute(
    skill_id: &str,
    records: &[Observation],
    required_representations: usize,
) -> Result<Summary, EvidenceError> {
    if skill_id.is_empty() || !(1..=2).contains(&required_representations) {
        return Err(EvidenceError::InvalidRecord);
    }
    let mut ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    let mut previous = None;
    let mut items = BTreeSet::new();
    let mut encounters = BTreeSet::new();
    let mut support_history: BTreeMap<&str, (u64, bool, bool)> = BTreeMap::new();
    let mut eligible = Vec::new();
    let mut seen = false;
    let mut attempted = false;
    for record in records {
        let e = &record.evidence;
        if e.event_id.is_empty()
            || e.session_id.is_empty()
            || e.encounter_id.is_empty()
            || e.skill_id.is_empty()
            || e.variant_id.is_empty()
            || e.item_family.is_empty()
            || e.context_id.is_empty()
            || e.representation.is_empty()
            || record.parameter_fingerprint.is_empty()
        {
            return Err(EvidenceError::InvalidRecord);
        }
        if !ids.insert(&e.event_id) || !sequences.insert(e.server_sequence) {
            return Err(EvidenceError::DuplicateEvent);
        }
        if previous.is_some_and(|p| e.server_sequence <= p) {
            return Err(EvidenceError::SequenceOrder);
        }
        previous = Some(e.server_sequence);
        if e.provenance != Provenance::ServerVerified || e.skill_id != skill_id {
            continue;
        }
        seen = true;
        let support = (
            e.support.hint_count,
            e.support.worked_example,
            e.support.read_aloud,
        );
        if support_history
            .get(e.encounter_id.as_str())
            .is_some_and(|prior| {
                support.0 < prior.0 || prior.1 && !support.1 || prior.2 && !support.2
            })
            || e.event_kind == Kind::Hint && support.0 == 0 && !support.2
            || e.event_kind == Kind::WorkedExample && !support.1
        {
            return Err(EvidenceError::InvalidRecord);
        }
        support_history.insert(&e.encounter_id, support);
        if e.event_kind != Kind::AnswerCheck {
            continue;
        }
        attempted = true;
        // A later attempt cannot pretend to be the encounter's first check.
        let first_encounter_check = encounters.insert(&e.encounter_id);
        if e.first_check != first_encounter_check {
            return Err(EvidenceError::InvalidRecord);
        }
        if !first_encounter_check {
            continue;
        }
        // Reopening the same parameters cannot manufacture independent evidence.
        if e.support.hint_count > 0
            || e.support.worked_example
            || e.support.read_aloud
            || e.result == ResultKind::NotAssessed
        {
            continue;
        }
        let novel_item = items.insert((&e.item_family, &record.parameter_fingerprint));
        if !novel_item {
            continue;
        }
        eligible.push(record);
    }
    let window = &eligible[eligible.len().saturating_sub(8)..];
    let correct = window
        .iter()
        .filter(|r| r.evidence.result == ResultKind::Correct)
        .count();
    let variants: BTreeSet<_> = window.iter().map(|r| &r.evidence.variant_id).collect();
    let representations: BTreeSet<_> = window.iter().map(|r| &r.evidence.representation).collect();
    let sessions: BTreeSet<_> = window.iter().map(|r| &r.evidence.session_id).collect();
    let contexts: BTreeSet<_> = window.iter().map(|r| &r.evidence.context_id).collect();
    let secure = window.len() == 8
        && correct >= 7
        && variants.len() >= 3
        && representations.len() >= required_representations
        && sessions.len() >= 2
        && contexts.len() >= 2
        && window
            .iter()
            .any(|r| r.novel_transfer && r.evidence.result == ResultKind::Correct)
        && window
            .iter()
            .rev()
            .take(3)
            .all(|r| r.evidence.result == ResultKind::Correct);
    let status = if secure {
        Status::Secure
    } else if attempted {
        Status::Practising
    } else if seen {
        Status::Introduced
    } else {
        Status::Unseen
    };
    Ok(Summary {
        status,
        eligible_checks: window.len(),
        correct_checks: correct,
        explanation: status_explanation(status),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub subject: String,
    pub prerequisites: Vec<String>,
    /// Only an explicit educator-approved probe may bypass prerequisite readiness.
    pub approved_entry_probe: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Practice {
        skill_id: String,
        explanation: &'static str,
    },
    Enrichment {
        explanation: &'static str,
    },
    PrerequisiteReview {
        explanation: &'static str,
    },
}
/// Seeded tie-breaking never depends on process hash order, wall time or model output.
pub fn select(
    nodes: &[Node],
    summaries: &BTreeMap<String, Summary>,
    subject: Option<&str>,
    seed: u64,
) -> Selection {
    let configured: Vec<_> = nodes
        .iter()
        .filter(|n| subject.is_none_or(|s| s == n.subject))
        .collect();
    let is_secure = |id: &str| {
        summaries
            .get(id)
            .is_some_and(|s| s.status == Status::Secure)
    };
    if !configured.is_empty() && configured.iter().all(|n| is_secure(&n.id)) {
        return Selection::Enrichment {
            explanation: "This configured pack is complete; choose enrichment or optional revision.",
        };
    }
    let mut frontier: Vec<_> = configured
        .into_iter()
        .filter(|n| {
            !is_secure(&n.id)
                && (n.approved_entry_probe || n.prerequisites.iter().all(|p| is_secure(p)))
        })
        .collect();
    frontier.sort_by_key(|n| (summaries.get(&n.id).map_or(0, |s| s.eligible_checks), &n.id));
    let Some(first) = frontier.first() else {
        return Selection::PrerequisiteReview {
            explanation: "No eligible frontier in this subject; offer prerequisite support without removing story access.",
        };
    };
    let minimum = summaries.get(&first.id).map_or(0, |s| s.eligible_checks);
    frontier.retain(|n| summaries.get(&n.id).map_or(0, |s| s.eligible_checks) == minimum);
    let selected = frontier[(seed % frontier.len() as u64) as usize];
    Selection::Practice {
        skill_id: selected.id.clone(),
        explanation: "Prerequisites are ready; this skill has relatively little independent evidence. Selection does not gate story rewards.",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportOffer {
    None,
    ConcreteHint,
    WorkedExampleOrPrerequisite,
}
pub fn support_offer(unsuccessful_checks: usize) -> SupportOffer {
    match unsuccessful_checks {
        0..=1 => SupportOffer::None,
        2 => SupportOffer::ConcreteHint,
        _ => SupportOffer::WorkedExampleOrPrerequisite,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyAdvice {
    Stay,
    HarderProbe(usize),
    OfferSupportBeforeSimplerProbe(usize),
}

/// The caller supplies distinct, server-verified first checks at this rung only.
/// A probe does not change mastery or lock access to the story.
pub fn difficulty_advice(
    current: usize,
    rung_count: usize,
    checks: &[(bool, bool)],
) -> Result<DifficultyAdvice, EvidenceError> {
    if rung_count == 0 || current >= rung_count {
        return Err(EvidenceError::InvalidRecord);
    }
    let last_two: Vec<_> = checks.iter().rev().take(2).collect();
    if last_two.len() < 2 {
        return Ok(DifficultyAdvice::Stay);
    }
    if last_two
        .iter()
        .all(|(correct, supported)| *correct && !*supported)
        && current + 1 < rung_count
    {
        Ok(DifficultyAdvice::HarderProbe(current + 1))
    } else if last_two.iter().all(|(correct, _)| !*correct) && current > 0 {
        Ok(DifficultyAdvice::OfferSupportBeforeSimplerProbe(
            current - 1,
        ))
    } else {
        Ok(DifficultyAdvice::Stay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn records() -> Vec<Observation> {
        (0..8).map(|i| Observation { evidence: serde_json::from_value(json!({
            "schemaVersion":"1.0", "eventId":format!("event-{i}"), "encounterId":format!("item-{i}"),
            "skillId":"M06", "variantId":format!("variant-{}", i % 3), "itemFamily":"missing_quantity",
            "contextId":format!("context-{}", i % 2), "representation":format!("representation-{}", i % 2),
            "result":"correct", "eventKind":"answer_check", "firstCheck":true,
            "support":{"hintCount":0,"workedExample":false,"readAloud":false},
            "sessionId":format!("session-{}",i % 2), "serverSequence":i+1,"provenance":"server_verified"
        })).unwrap(), parameter_fingerprint: format!("total-{i}"), novel_transfer: i == 7 }).collect()
    }

    fn assert_incremental_matches(history: &[Observation]) {
        let mut state = IncrementalMastery::new("M06");
        let mut durable_items = BTreeSet::new();
        for (index, record) in history.iter().enumerate() {
            let novel = is_novelty_candidate(record)
                && durable_items.insert((
                    record.evidence.item_family.clone(),
                    record.parameter_fingerprint.clone(),
                ));
            state.observe(record, novel).unwrap();
            // A database checkpoint/restart after every record must preserve
            // the same projection without loading the complete evidence history.
            let value = serde_json::to_value(&state).unwrap();
            assert!(value["window"].as_array().unwrap().len() <= 8);
            state = serde_json::from_value(value).unwrap();
            for representations in 1..=2 {
                assert_eq!(
                    state.summary(representations).unwrap(),
                    recompute("M06", &history[..=index], representations).unwrap(),
                    "record {index}, representations {representations}"
                );
            }
        }
    }

    #[test]
    fn incremental_checkpoints_match_full_policy_across_support_and_old_item_repeats() {
        let samples = records();
        let mut history = Vec::new();
        let push = |history: &mut Vec<Observation>, mut record: Observation| {
            record.evidence.server_sequence = history.len() as u64 + 1;
            record.evidence.event_id = format!("incremental-event-{}", history.len());
            history.push(record);
        };
        for index in 0..160 {
            let mut answer = samples[index % 8].clone();
            answer.evidence.encounter_id = format!("encounter-{index}");
            answer.evidence.session_id = format!("session-{}", index / 11);
            answer.evidence.context_id = format!("context-{}", index % 3);
            answer.evidence.variant_id = format!("variant-{}", index % 5);
            answer.parameter_fingerprint = if index % 4 == 0 {
                // The first item can have left the eight-item checkpoint long
                // ago, but durable novelty must still suppress a later repeat.
                "original-item".into()
            } else {
                format!("item-parameters-{index}")
            };
            answer.novel_transfer = index % 7 == 0;
            if index % 11 == 0 {
                answer.evidence.result = ResultKind::Incorrect;
            }
            match index % 13 {
                1 => {
                    answer.evidence.support.hint_count = 1;
                    let mut hint = answer.clone();
                    hint.evidence.event_kind = Kind::Hint;
                    hint.evidence.first_check = false;
                    hint.evidence.result = ResultKind::NotAssessed;
                    push(&mut history, hint);
                }
                2 => {
                    answer.evidence.support.worked_example = true;
                    let mut example = answer.clone();
                    example.evidence.event_kind = Kind::WorkedExample;
                    example.evidence.first_check = false;
                    example.evidence.result = ResultKind::NotAssessed;
                    push(&mut history, example);
                }
                3 => answer.evidence.support.read_aloud = true,
                4 => answer.evidence.result = ResultKind::NotAssessed,
                5 => answer.evidence.provenance = Provenance::LegacyUnverified,
                _ => {}
            }
            push(&mut history, answer.clone());
            if index % 3 == 0 {
                let mut later = answer.clone();
                later.evidence.first_check = false;
                later.evidence.result = ResultKind::Correct;
                push(&mut history, later);
            }
            let mut completed = answer;
            completed.evidence.first_check = false;
            completed.evidence.event_kind = Kind::Completion;
            push(&mut history, completed);
        }
        assert_incremental_matches(&history);
    }

    #[test]
    fn incremental_supported_introductions_and_abandonment_preserve_exact_summary() {
        let mut history = records();
        for record in &mut history {
            record.evidence.support.hint_count = 1;
        }
        let mut independent = records();
        for record in &mut independent {
            record.evidence.event_id.push_str("-independent");
            record.evidence.encounter_id.push_str("-independent");
            record.evidence.server_sequence += 8;
        }
        history.extend(independent);
        assert_incremental_matches(&history);
        assert_eq!(
            recompute("M06", &history, 2).unwrap().status,
            Status::Secure
        );

        for record in &mut history {
            record.evidence.event_kind = Kind::Abandonment;
        }
        assert_incremental_matches(&history);
        assert_eq!(
            recompute("M06", &history, 2).unwrap().status,
            Status::Introduced
        );
    }

    #[test]
    fn incremental_state_rejects_corruption_and_bad_inputs_without_mutation() {
        let empty = IncrementalMastery::new("M06");
        assert_eq!(empty.summary(2), recompute("M06", &[], 2));
        assert_eq!(empty.summary(0), Err(EvidenceError::InvalidRecord));
        let mut state = empty;
        for record in records() {
            state.observe(&record, true).unwrap();
        }
        let original = state.clone();
        assert_eq!(
            state.observe(&records()[0], true),
            Err(EvidenceError::SequenceOrder)
        );
        assert_eq!(state, original);
        let mut supported = records()[0].clone();
        supported.evidence.server_sequence = 9;
        supported.evidence.support.hint_count = 1;
        assert_eq!(
            state.observe(&supported, true),
            Err(EvidenceError::InvalidRecord)
        );
        assert_eq!(state, original);

        let valid = serde_json::to_value(&state).unwrap();
        for mode in 0..9 {
            let mut corrupt = valid.clone();
            match mode {
                0 => corrupt["policyVersion"] = json!("unsupported-policy"),
                1 => corrupt["window"]
                    .as_array_mut()
                    .unwrap()
                    .push(valid["window"][0].clone()),
                2 => corrupt["attempted"] = json!(false),
                3 => corrupt["seen"] = json!(false),
                4 => corrupt["lastSequence"] = json!(0),
                5 => corrupt["window"][0]["evidence"]["support"]["hintCount"] = json!(1),
                6 => corrupt["window"][0]["evidence"]["skillId"] = json!("E01"),
                7 => corrupt["window"][0]["evidence"]["serverSequence"] = json!(100),
                _ => corrupt["skillId"] = json!("x".repeat(121)),
            }
            let loaded: IncrementalMastery = serde_json::from_value(corrupt).unwrap();
            assert_eq!(loaded.validate(), Err(EvidenceError::InvalidRecord));
            assert_eq!(loaded.summary(2), Err(EvidenceError::InvalidRecord));
        }
    }
    #[test]
    fn diverse_independent_transfer_qualifies_and_recomputes() {
        let records = records();
        assert_eq!(
            recompute("M06", &records, 2).unwrap().status,
            Status::Secure
        );
        assert_eq!(recompute("M06", &records, 2), recompute("M06", &records, 2));
        assert_eq!(
            recompute("M06", &records[..7], 2).unwrap().status,
            Status::Practising
        );
    }
    #[test]
    fn support_imports_repetition_and_no_transfer_never_qualify() {
        for mode in 0..6 {
            let mut observations = records();
            for r in &mut observations {
                match mode {
                    0 => r.evidence.support.hint_count = 1,
                    1 => r.evidence.support.read_aloud = true,
                    2 => r.evidence.provenance = Provenance::LegacyUnverified,
                    3 => r.parameter_fingerprint = "fixed".into(),
                    4 => r.novel_transfer = false,
                    _ => r.evidence.session_id = "one-session".into(),
                }
            }
            assert_ne!(
                recompute("M06", &observations, 2).unwrap().status,
                Status::Secure
            );
        }
    }
    #[test]
    fn seven_correct_needs_latest_three_and_representation_diversity() {
        let mut r = records();
        r[0].evidence.result = ResultKind::Incorrect;
        assert_eq!(recompute("M06", &r, 2).unwrap().status, Status::Secure);
        r[0].evidence.result = ResultKind::Correct;
        r[7].evidence.result = ResultKind::Incorrect;
        assert_eq!(recompute("M06", &r, 2).unwrap().status, Status::Practising);
        let mut r = records();
        for o in &mut r {
            o.evidence.representation = "objects".into();
        }
        assert_eq!(recompute("M06", &r, 2).unwrap().status, Status::Practising);
        assert_eq!(recompute("M06", &r, 1).unwrap().status, Status::Secure);
    }
    #[test]
    fn corrupted_duplicate_or_out_of_order_events_are_rejected() {
        let mut r = records();
        r[1].evidence.event_id = r[0].evidence.event_id.clone();
        assert_eq!(recompute("M06", &r, 2), Err(EvidenceError::DuplicateEvent));
        let mut r = records();
        r.swap(1, 2);
        assert_eq!(recompute("M06", &r, 2), Err(EvidenceError::SequenceOrder));
        let mut r = records();
        r[1].evidence.encounter_id = r[0].evidence.encounter_id.clone();
        assert_eq!(recompute("M06", &r, 2), Err(EvidenceError::InvalidRecord));
    }
    #[test]
    fn abandonment_does_not_imply_failure_or_mastery() {
        let mut r = records();
        for o in &mut r {
            o.evidence.event_kind = Kind::Abandonment;
        }
        let summary = recompute("M06", &r, 2).unwrap();
        assert_eq!(summary.status, Status::Introduced);
        assert_eq!(summary.eligible_checks, 0);
    }

    #[test]
    fn replay_cannot_erase_preceding_support() {
        let mut history = Vec::new();
        for (index, mut answer) in records().into_iter().enumerate() {
            let mut hint = answer.clone();
            hint.evidence.event_id = format!("hint-{index}");
            hint.evidence.event_kind = Kind::Hint;
            hint.evidence.first_check = false;
            hint.evidence.result = ResultKind::NotAssessed;
            hint.evidence.support.hint_count = 1;
            hint.evidence.server_sequence = index as u64 * 2 + 1;
            answer.evidence.server_sequence = index as u64 * 2 + 2;
            history.extend([hint, answer]);
        }
        assert_eq!(
            recompute("M06", &history, 2),
            Err(EvidenceError::InvalidRecord)
        );
        for record in &mut history {
            record.evidence.support.hint_count = 1;
        }
        let summary = recompute("M06", &history, 2).unwrap();
        assert_eq!(summary.status, Status::Practising);
        assert_eq!(summary.eligible_checks, 0);
    }

    #[test]
    fn supported_introduction_does_not_exhaust_later_independent_items() {
        let mut supported = records();
        for record in &mut supported {
            record.evidence.support.hint_count = 1;
        }
        let mut later = records();
        for record in &mut later {
            record.evidence.event_id.push_str("-later");
            record.evidence.encounter_id.push_str("-later");
            record.evidence.session_id.push_str("-later");
            record.evidence.server_sequence += 8;
        }
        supported.extend(later);
        assert_eq!(
            recompute("M06", &supported, 2).unwrap().status,
            Status::Secure
        );
    }
    #[test]
    fn frontier_excludes_secure_nodes_and_handles_subject_switch() {
        let nodes = vec![
            Node {
                id: "M06".into(),
                subject: "maths".into(),
                prerequisites: vec![],
                approved_entry_probe: false,
            },
            Node {
                id: "M07".into(),
                subject: "maths".into(),
                prerequisites: vec!["M06".into()],
                approved_entry_probe: false,
            },
            Node {
                id: "E01".into(),
                subject: "english".into(),
                prerequisites: vec![],
                approved_entry_probe: false,
            },
        ];
        let mut state = BTreeMap::new();
        let secure = recompute("M06", &records(), 2).unwrap();
        state.insert("M06".into(), secure.clone());
        assert!(
            matches!(select(&nodes,&state,Some("maths"),0),Selection::Practice{skill_id,..} if skill_id=="M07")
        );
        assert!(
            matches!(select(&nodes,&state,Some("english"),0),Selection::Practice{skill_id,..} if skill_id=="E01")
        );
        state.insert("M07".into(), secure);
        assert!(matches!(
            select(&nodes, &state, Some("maths"), 0),
            Selection::Enrichment { .. }
        ));
        assert_eq!(
            select(&nodes, &state, None, 42),
            select(&nodes, &state, None, 42)
        );
    }
    #[test]
    fn cold_start_and_support_offers_are_useful() {
        let nodes = vec![Node {
            id: "M03".into(),
            subject: "maths".into(),
            prerequisites: vec!["M01".into()],
            approved_entry_probe: false,
        }];
        assert!(matches!(
            select(&nodes, &BTreeMap::new(), None, 0),
            Selection::PrerequisiteReview { .. }
        ));
        assert_eq!(support_offer(2), SupportOffer::ConcreteHint);
        assert_eq!(support_offer(3), SupportOffer::WorkedExampleOrPrerequisite);
        assert_eq!(
            difficulty_advice(0, 3, &[(true, false), (true, false)]),
            Ok(DifficultyAdvice::HarderProbe(1))
        );
        assert_eq!(
            difficulty_advice(0, 3, &[(true, true), (true, false)]),
            Ok(DifficultyAdvice::Stay)
        );
        assert_eq!(
            difficulty_advice(1, 3, &[(false, false), (false, true)]),
            Ok(DifficultyAdvice::OfferSupportBeforeSimplerProbe(0))
        );
        assert_eq!(
            difficulty_advice(3, 3, &[]),
            Err(EvidenceError::InvalidRecord)
        );
    }
}
