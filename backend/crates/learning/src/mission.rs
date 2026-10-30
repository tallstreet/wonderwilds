//! Original, bounded maths activities and a separate provisional mission progression policy.
//! This does not alter the established independent-transfer mastery policy.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use wonderwilds_contracts::{
    LearningEvidence, LearningEvidenceEventKind as EventKind,
    LearningEvidenceProvenance as Provenance, LearningEvidenceResult as ResultKind,
};

use crate::curriculum::{CurriculumGraph, SkillNode};

pub const POLICY_VERSION: &str = "provisional-mission-v1";
pub const CONTENT_VERSION: &str = "mission-maths-draft-1";
const QUALIFYING_CHECKS: usize = 8;
const REQUIRED_CORRECT: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MissionState {
    pub policy_version: String,
    /// Monotonic command/evidence sequence; observations below are a bounded latest-item projection.
    pub last_sequence: u64,
    #[serde(default)]
    pub introduced_skills: BTreeSet<String>,
    #[serde(default)]
    pub attempted_skills: BTreeSet<String>,
    #[serde(default)]
    pub observations: Vec<MissionObservation>,
}

impl Default for MissionState {
    fn default() -> Self {
        Self {
            policy_version: POLICY_VERSION.to_owned(),
            last_sequence: 0,
            introduced_skills: BTreeSet::new(),
            attempted_skills: BTreeSet::new(),
            observations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MissionObservation {
    pub evidence: LearningEvidence,
    /// Fingerprint of canonical task content; excludes IDs, labels, mission and session names.
    pub parameter_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionError {
    UnsupportedSkill,
    InvalidState,
    DuplicateEvent,
    SequenceOrder,
    InvalidAnswer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Unseen,
    Introduced,
    Practising,
    Locked,
    Ready,
    #[serde(rename = "ready_for_next")]
    PracticeReady,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillSummary {
    pub skill_id: String,
    pub status: Status,
    pub eligible_checks: usize,
    pub correct_checks: usize,
    pub distinct_variants: usize,
    pub distinct_contexts: usize,
    pub distinct_representations: usize,
    pub distinct_sessions: usize,
    pub blockers: Vec<String>,
    pub next_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillProjection {
    pub skill_id: String,
    pub subject: String,
    pub objective: String,
    pub prerequisites: Vec<String>,
    pub status: Status,
    pub supported_by_mission: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MissionPresentation {
    pub title: String,
    pub objective: String,
    pub lesson_text: String,
    pub prompt: String,
    pub context: String,
    pub representation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamedItem {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum PublicTask {
    Pairing {
        layout: String,
        objects: Vec<NamedItem>,
        recipients: Vec<NamedItem>,
    },
    CountCollection {
        layout: String,
        objects: Vec<NamedItem>,
        choices: Vec<NumberChoice>,
    },
    CompareCollections {
        collections: Vec<Collection>,
        prompt_mode: String,
        choices: Vec<NamedItem>,
    },
    NumberPath {
        shown: Vec<i8>,
        direction: String,
        choices: Vec<NumberChoice>,
    },
    PartWhole {
        whole: u8,
        shown_part: u8,
        choices: Vec<NumberChoice>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumberChoice {
    pub id: String,
    pub value: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    pub id: String,
    pub label: String,
    pub items: Vec<NamedItem>,
}

/// Client-safe activity view. The assessment key is held in `MissionActivity` and is not serialized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MissionActivityView {
    pub schema_version: String,
    pub id: String,
    pub skill_id: String,
    pub content_version: String,
    pub variant_id: String,
    pub context_id: String,
    pub representation: String,
    pub presentation: MissionPresentation,
    pub task: PublicTask,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
pub enum MissionAnswer {
    Pairing {
        assignments: Vec<PairAssignment>,
    },
    Choice {
        #[serde(rename = "choiceId")]
        choice_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PairAssignment {
    pub object_id: String,
    pub recipient_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Independent,
    Supported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub correct: bool,
    pub classification: Classification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Support {
    pub hint_count: u8,
    pub worked_example: bool,
    pub read_aloud: bool,
}

#[derive(Debug, Clone)]
enum AssessmentKey {
    Pairing {
        object_ids: Vec<String>,
        recipient_ids: Vec<String>,
    },
    Choice {
        choice_ids: Vec<String>,
        correct_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct MissionActivity {
    pub view: MissionActivityView,
    key: AssessmentKey,
    canonical_fingerprint: String,
}

impl MissionActivity {
    pub fn canonical_fingerprint(&self) -> &str {
        &self.canonical_fingerprint
    }

    pub fn assess(
        &self,
        answer: &MissionAnswer,
        support: Support,
    ) -> Result<Outcome, MissionError> {
        let correct = match (&self.key, answer) {
            (
                AssessmentKey::Pairing {
                    object_ids,
                    recipient_ids,
                },
                MissionAnswer::Pairing { assignments },
            ) => {
                if assignments.len() != object_ids.len()
                    || object_ids.len() != recipient_ids.len()
                    || assignments.iter().any(|a| {
                        !object_ids.contains(&a.object_id)
                            || !recipient_ids.contains(&a.recipient_id)
                    })
                {
                    return Err(MissionError::InvalidAnswer);
                }
                let objects: BTreeSet<_> =
                    assignments.iter().map(|a| a.object_id.as_str()).collect();
                let recipients: BTreeSet<_> = assignments
                    .iter()
                    .map(|a| a.recipient_id.as_str())
                    .collect();
                objects.len() == assignments.len() && recipients.len() == assignments.len()
            }
            (
                AssessmentKey::Choice {
                    choice_ids,
                    correct_id,
                },
                MissionAnswer::Choice { choice_id },
            ) => {
                if !choice_ids.contains(choice_id) {
                    return Err(MissionError::InvalidAnswer);
                }
                choice_id == correct_id
            }
            _ => return Err(MissionError::InvalidAnswer),
        };
        Ok(Outcome {
            correct,
            classification: if support.hint_count > 0
                || support.worked_example
                || support.read_aloud
            {
                Classification::Supported
            } else {
                Classification::Independent
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportKind {
    Hint,
    WorkedExample,
}

const SUPPORTED_SKILLS: [&str; 5] = ["M01", "M03", "M04", "M05", "M06"];

pub fn supported_skill(skill_id: &str) -> bool {
    SUPPORTED_SKILLS.contains(&skill_id)
}

pub fn authored_activity(
    skill_id: &str,
    seed: u64,
    activity_id: &str,
) -> Result<MissionActivity, MissionError> {
    if !supported_skill(skill_id) || activity_id.is_empty() {
        return Err(MissionError::UnsupportedSkill);
    }
    let slot = (seed % 12) as u8;
    let variant_id = format!("{skill_id}-mission-v1-{slot:02}");
    let (task, key, context, representation, prompt) = match skill_id {
        "M01" => {
            let count = 1 + (slot % 6);
            let layout = if slot < 6 {
                "object_cards"
            } else {
                "recipient_slots"
            };
            let objects = (0..count)
                .map(|i| NamedItem {
                    id: format!("object-{i}"),
                    label: object_label(slot, i),
                })
                .collect::<Vec<_>>();
            let recipients = (0..count)
                .map(|i| NamedItem {
                    id: format!("recipient-{i}"),
                    label: recipient_label(slot, i),
                })
                .collect::<Vec<_>>();
            let key = AssessmentKey::Pairing {
                object_ids: objects.iter().map(|v| v.id.clone()).collect(),
                recipient_ids: recipients.iter().map(|v| v.id.clone()).collect(),
            };
            (
                PublicTask::Pairing {
                    layout: layout.to_owned(),
                    objects,
                    recipients,
                },
                key,
                context_for(slot),
                if slot < 6 {
                    "object_pairing"
                } else {
                    "recipient_slots"
                }
                .to_owned(),
                "Give one object to each place. Pair every object with a different place."
                    .to_owned(),
            )
        }
        "M03" => {
            let count = 1 + (slot % 6);
            let layout = if slot < 6 {
                "movable_objects"
            } else {
                "ten_frame"
            };
            let objects = (0..count)
                .map(|i| NamedItem {
                    id: format!("counted-{i}"),
                    label: countable_label(slot, i),
                })
                .collect::<Vec<_>>();
            let correct_id = format!("number-{count}");
            let choices = number_choices(count, slot);
            let key = AssessmentKey::Choice {
                choice_ids: choices.iter().map(|v| v.id.clone()).collect(),
                correct_id,
            };
            (
                PublicTask::CountCollection {
                    layout: layout.to_owned(),
                    objects,
                    choices,
                },
                key,
                context_for(slot),
                layout.to_owned(),
                "Count the collection, then choose the number that shows how many there are."
                    .to_owned(),
            )
        }
        "M04" => {
            let left = 2 + (slot % 6);
            let right = left + 1;
            let mode = if slot < 6 { "more" } else { "fewer" };
            let collections = vec![
                make_collection("garden", left, slot),
                make_collection("pond", right, slot + 3),
            ];
            let target = if (mode == "more" && left > right) || (mode == "fewer" && left < right) {
                "garden"
            } else {
                "pond"
            };
            let choices = collections
                .iter()
                .map(|c| NamedItem {
                    id: c.id.clone(),
                    label: c.label.clone(),
                })
                .collect::<Vec<_>>();
            let key = AssessmentKey::Choice {
                choice_ids: choices.iter().map(|v| v.id.clone()).collect(),
                correct_id: target.to_owned(),
            };
            (
                PublicTask::CompareCollections {
                    collections,
                    prompt_mode: mode.to_owned(),
                    choices,
                },
                key,
                context_for(slot),
                if slot < 6 {
                    "paired_collections"
                } else {
                    "aligned_rows"
                }
                .to_owned(),
                format!("Which collection has {mode} items?"),
            )
        }
        "M05" => {
            let backwards = slot % 2 == 1;
            let step = 1;
            let start = if backwards {
                4 + (slot / 2) as i8
            } else {
                1 + (slot / 2) as i8
            };
            let shown = (0..3)
                .map(|i| {
                    if backwards {
                        start - i * step
                    } else {
                        start + i * step
                    }
                })
                .collect::<Vec<_>>();
            let answer = if backwards {
                start - 3 * step
            } else {
                start + 3 * step
            };
            let choices = number_choices_i8(answer, slot);
            let correct_id = format!("number-{answer}");
            let key = AssessmentKey::Choice {
                choice_ids: choices.iter().map(|v| v.id.clone()).collect(),
                correct_id,
            };
            (
                PublicTask::NumberPath {
                    shown,
                    direction: if backwards { "backward" } else { "forward" }.to_owned(),
                    choices,
                },
                key,
                context_for(slot),
                if slot < 6 {
                    "number_path"
                } else {
                    "countable_flower_sequence"
                }
                .to_owned(),
                format!(
                    "Follow the numbers {} and choose what comes next.",
                    if backwards { "backward" } else { "forward" }
                ),
            )
        }
        "M06" => {
            let whole = 4 + (slot % 7);
            let shown_part = 1 + (slot % (whole - 1));
            let answer = whole - shown_part;
            let choices = number_choices(answer, slot);
            let correct_id = format!("number-{answer}");
            let key = AssessmentKey::Choice {
                choice_ids: choices.iter().map(|v| v.id.clone()).collect(),
                correct_id,
            };
            (
                PublicTask::PartWhole {
                    whole,
                    shown_part,
                    choices,
                },
                key,
                context_for(slot),
                if slot < 6 {
                    "countable_planks"
                } else {
                    "part_whole_objects"
                }
                .to_owned(),
                "Some objects are shown. Choose how many more complete the whole.".to_owned(),
            )
        }
        _ => return Err(MissionError::UnsupportedSkill),
    };
    let node = CurriculumGraph::load()
        .map_err(|_| MissionError::InvalidState)?
        .get(skill_id)
        .cloned()
        .ok_or(MissionError::UnsupportedSkill)?;
    let presentation = presentation(&node, context, &representation, &prompt);
    let canonical_fingerprint = canonical_fingerprint(skill_id, &task);
    let view = MissionActivityView {
        schema_version: "1.0".to_owned(),
        id: activity_id.to_owned(),
        skill_id: skill_id.to_owned(),
        content_version: CONTENT_VERSION.to_owned(),
        variant_id: variant_id.clone(),
        context_id: format!("{skill_id}-mission-context-{}", slot % 3),
        representation,
        presentation,
        task,
    };
    // Include authored task shape but not generated ordinal IDs/labels in the dedup fingerprint.
    Ok(MissionActivity {
        view,
        key,
        canonical_fingerprint,
    })
}

/// Apply server-derived mission context supplied from the locked story snapshot.
/// This only changes safe display text; it never changes task parameters or the answer key.
pub fn authored_activity_in_context(
    skill_id: &str,
    seed: u64,
    activity_id: &str,
    context_id: &str,
    context_text: &str,
) -> Result<MissionActivity, MissionError> {
    if context_id.is_empty() || context_text.is_empty() || context_text.len() > 160 {
        return Err(MissionError::InvalidState);
    }
    let mut activity = authored_activity(skill_id, seed, activity_id)?;
    activity.view.context_id = context_id.to_owned();
    activity.view.presentation.context = context_text.to_owned();
    Ok(activity)
}

fn canonical_fingerprint(skill_id: &str, task: &PublicTask) -> String {
    let shape = match task {
        PublicTask::Pairing {
            layout,
            objects,
            recipients,
        } => {
            format!("pairing:{layout}:{}:{}", objects.len(), recipients.len())
        }
        PublicTask::CountCollection {
            layout,
            objects,
            choices,
        } => {
            format!("count:{layout}:{}:{}", objects.len(), choices.len())
        }
        PublicTask::CompareCollections {
            collections,
            prompt_mode,
            ..
        } => {
            let mut counts = collections
                .iter()
                .map(|c| c.items.len())
                .collect::<Vec<_>>();
            counts.sort_unstable();
            format!("compare:{prompt_mode}:{counts:?}")
        }
        PublicTask::NumberPath {
            shown, direction, ..
        } => format!("path:{direction}:{shown:?}"),
        PublicTask::PartWhole {
            whole, shown_part, ..
        } => format!("part-whole:{whole}:{shown_part}"),
    };
    format!("{skill_id}:{shape}")
}

fn presentation(
    node: &SkillNode,
    context: &str,
    representation: &str,
    prompt: &str,
) -> MissionPresentation {
    let (title, lesson_text) = match node.id.as_str() {
        "M01" => (
            "Make a fair delivery",
            "Every friend should receive one object. Check that no friend is missed and no friend gets two.",
        ),
        "M03" => (
            "Count the collection",
            "Touch or track each object once. The last number you say tells how many objects there are.",
        ),
        "M04" => (
            "Compare collections",
            "Count each group. The group with the larger count has more; the smaller count has fewer.",
        ),
        "M05" => (
            "Follow a number path",
            "Look at how the numbers change each time, then continue in the same direction.",
        ),
        "M06" => (
            "Complete the whole",
            "A whole can be split into parts. The shown part and the missing part make the whole.",
        ),
        _ => ("Maths mission", "Try the activity and check your idea."),
    };
    MissionPresentation {
        title: title.to_owned(),
        objective: node.objective.clone(),
        lesson_text: lesson_text.to_owned(),
        prompt: prompt.to_owned(),
        context: context.to_owned(),
        representation: representation.to_owned(),
    }
}

fn context_for(slot: u8) -> &'static str {
    [
        "a garden collection",
        "a picnic collection",
        "a number path",
    ][usize::from(slot % 3)]
}

fn object_label(slot: u8, index: u8) -> String {
    let kinds = ["acorn", "berry", "flower", "shell", "seed"];
    format!("{} {}", kinds[usize::from((slot + index) % 5)], index + 1)
}

fn recipient_label(slot: u8, index: u8) -> String {
    if slot < 6 {
        format!("place mat {}", index + 1)
    } else {
        format!("basket slot {}", index + 1)
    }
}

fn countable_label(slot: u8, index: u8) -> String {
    let kinds = ["leaf", "pebble", "berry", "seed", "star", "flower"];
    kinds[usize::from((slot + index) % kinds.len() as u8)].to_owned()
}

fn number_choices(correct: u8, seed: u8) -> Vec<NumberChoice> {
    number_choices_i8(correct as i8, seed)
}

fn number_choices_i8(correct: i8, seed: u8) -> Vec<NumberChoice> {
    let mut values = vec![correct];
    for candidate in [
        correct - 1,
        correct + 1,
        correct - 2,
        correct + 2,
        correct - 3,
        correct + 3,
    ] {
        if (1..=20).contains(&candidate) && !values.contains(&candidate) && values.len() < 3 {
            values.push(candidate);
        }
    }
    values.sort_unstable();
    if seed % 2 == 1 {
        values.reverse();
    }
    values
        .into_iter()
        .map(|value| NumberChoice {
            id: format!("number-{value}"),
            value,
        })
        .collect()
}

fn make_collection(id: &str, count: u8, seed: u8) -> Collection {
    let items = (0..count)
        .map(|i| NamedItem {
            id: format!("{id}-{i}"),
            label: countable_label(seed, i),
        })
        .collect();
    Collection {
        id: id.to_owned(),
        label: if id == "garden" {
            "Garden basket"
        } else {
            "Pond basket"
        }
        .to_owned(),
        items,
    }
}

/// Return support text only after the API has accepted and persisted a support command.
pub fn support_content(skill_id: &str, kind: SupportKind, hint_index: usize) -> Option<String> {
    if !supported_skill(skill_id) {
        return None;
    }
    let hints: &[&str] = match skill_id {
        "M01" => &[
            "Give one object to one place.",
            "After each delivery, check whether that place already has an object.",
            "Count the objects and places. Make sure every place has exactly one.",
        ],
        "M03" => &[
            "Point to each object as you count.",
            "Keep track so you do not count an object twice.",
            "Say the count again slowly, then choose that number.",
        ],
        "M04" => &[
            "Count the objects in each collection.",
            "Compare the two counts.",
            "Choose the collection with the requested larger or smaller count.",
        ],
        "M05" => &[
            "Compare the first two numbers.",
            "Find how much the number changes by.",
            "Keep the same change as you follow the path.",
        ],
        "M06" => &[
            "The whole is made from the shown part and one missing part.",
            "Think: shown part plus missing part equals the whole.",
            "Count on from the shown part until you reach the whole.",
        ],
        _ => return None,
    };
    match kind {
        SupportKind::Hint => hints.get(hint_index.min(hints.len() - 1)).map(|v| (*v).to_owned()),
        SupportKind::WorkedExample => Some(match skill_id {
            "M01" => "For two friends and two objects, give one object to each friend and check that each friend has one.".to_owned(),
            "M03" => "If you count 1, 2, 3, 4 objects, there are 4 objects altogether.".to_owned(),
            "M04" => "A group of 5 has more than a group of 3 because 5 is greater than 3.".to_owned(),
            "M05" => "On 8, 7, 6, the path goes back by 1, so the next number is 5.".to_owned(),
            "M06" => "If the whole is 7 and 3 are shown, 4 more make 7.".to_owned(),
            _ => return None,
        }),
    }
}

pub fn record(
    state: &mut MissionState,
    observation: MissionObservation,
) -> Result<(), MissionError> {
    if state.policy_version != POLICY_VERSION
        || !valid_observation(&observation)
        || !supported_skill(&observation.evidence.skill_id)
    {
        return Err(MissionError::InvalidState);
    }
    let e = &observation.evidence;
    if state
        .observations
        .iter()
        .any(|prior| prior.evidence.event_id == e.event_id)
    {
        return Err(MissionError::DuplicateEvent);
    }
    if e.server_sequence <= state.last_sequence {
        return Err(MissionError::SequenceOrder);
    }
    let eligible = e.provenance == Provenance::ServerVerified
        && e.event_kind == EventKind::AnswerCheck
        && e.first_check
        && e.result != ResultKind::NotAssessed
        && e.support.hint_count == 0
        && !e.support.worked_example
        && !e.support.read_aloud;
    let same_item = state.observations.iter().any(|prior| {
        prior.evidence.skill_id == e.skill_id
            && prior.evidence.item_family == e.item_family
            && prior.parameter_fingerprint == observation.parameter_fingerprint
    });
    if eligible && !same_item && state.observations.len() >= 60 {
        return Err(MissionError::InvalidState);
    }
    state.last_sequence = e.server_sequence;
    if e.provenance == Provenance::ServerVerified
        && matches!(
            e.event_kind,
            EventKind::Hint | EventKind::WorkedExample | EventKind::AnswerCheck
        )
    {
        state.introduced_skills.insert(e.skill_id.clone());
    }
    if e.provenance == Provenance::ServerVerified && e.event_kind == EventKind::AnswerCheck {
        state.attempted_skills.insert(e.skill_id.clone());
    }
    // Store only the latest independently eligible result per canonical task. Full immutable
    // observations and receipts stay in the API evidence journal; this policy projection is bounded.
    if eligible {
        state.observations.retain(|prior| {
            prior.evidence.skill_id != e.skill_id
                || prior.evidence.item_family != e.item_family
                || prior.parameter_fingerprint != observation.parameter_fingerprint
        });
        state.observations.push(observation);
        state
            .observations
            .sort_by_key(|o| o.evidence.server_sequence);
    }
    Ok(())
}

pub fn mark_introduced(state: &mut MissionState, skill_id: &str) -> Result<(), MissionError> {
    if state.policy_version != POLICY_VERSION || !supported_skill(skill_id) {
        return Err(MissionError::UnsupportedSkill);
    }
    state.introduced_skills.insert(skill_id.to_owned());
    Ok(())
}

/// Replay the immutable API evidence journal into the bounded policy checkpoint.
/// This verifies server sequence, event uniqueness, first-check flags and support monotonicity.
pub fn replay(history: &[MissionObservation]) -> Result<MissionState, MissionError> {
    let mut state = MissionState::default();
    let mut event_ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    let mut checked_encounters = BTreeSet::new();
    let mut support_by_encounter: BTreeMap<&str, (u64, bool, bool)> = BTreeMap::new();
    let mut previous = None;
    for observation in history {
        if !valid_observation(observation) {
            return Err(MissionError::InvalidState);
        }
        let e = &observation.evidence;
        if !event_ids.insert(e.event_id.as_str()) || !sequences.insert(e.server_sequence) {
            return Err(MissionError::DuplicateEvent);
        }
        if previous.is_some_and(|p| e.server_sequence <= p) {
            return Err(MissionError::SequenceOrder);
        }
        previous = Some(e.server_sequence);
        if e.provenance != Provenance::ServerVerified {
            continue;
        }
        let current = (
            e.support.hint_count,
            e.support.worked_example,
            e.support.read_aloud,
        );
        if support_by_encounter
            .get(e.encounter_id.as_str())
            .is_some_and(|old| current.0 < old.0 || old.1 && !current.1 || old.2 && !current.2)
            || (e.event_kind == EventKind::Hint && current.0 == 0 && !current.2)
            || (e.event_kind == EventKind::WorkedExample && !current.1)
        {
            return Err(MissionError::InvalidState);
        }
        support_by_encounter.insert(&e.encounter_id, current);
        if e.event_kind == EventKind::AnswerCheck {
            let first = checked_encounters.insert(e.encounter_id.as_str());
            if first != e.first_check {
                return Err(MissionError::InvalidState);
            }
        }
        if supported_skill(&e.skill_id) {
            record(&mut state, observation.clone())?;
        } else {
            state.last_sequence = e.server_sequence;
        }
    }
    Ok(state)
}

fn valid_observation(observation: &MissionObservation) -> bool {
    let e = &observation.evidence;
    !e.event_id.is_empty()
        && !e.encounter_id.is_empty()
        && !e.skill_id.is_empty()
        && !e.variant_id.is_empty()
        && !e.item_family.is_empty()
        && !e.context_id.is_empty()
        && !e.representation.is_empty()
        && !e.session_id.is_empty()
        && !observation.parameter_fingerprint.is_empty()
}

fn qualifying<'a>(
    skill_id: &str,
    state: &'a MissionState,
) -> Result<Vec<&'a MissionObservation>, MissionError> {
    if state.policy_version != POLICY_VERSION
        || state.observations.iter().any(|o| !valid_observation(o))
    {
        return Err(MissionError::InvalidState);
    }
    let mut event_ids = BTreeSet::new();
    let mut seqs = BTreeSet::new();
    let mut previous = None;
    let mut eligible_by_item = BTreeMap::new();
    for o in &state.observations {
        let e = &o.evidence;
        if !event_ids.insert(e.event_id.as_str()) || !seqs.insert(e.server_sequence) {
            return Err(MissionError::DuplicateEvent);
        }
        if previous.is_some_and(|p| e.server_sequence <= p) {
            return Err(MissionError::SequenceOrder);
        }
        previous = Some(e.server_sequence);
        if e.skill_id != skill_id
            || !supported_skill(&e.skill_id)
            || e.provenance != Provenance::ServerVerified
            || e.event_kind != EventKind::AnswerCheck
            || !e.first_check
        {
            continue;
        }
        if e.result == ResultKind::NotAssessed
            || e.support.hint_count > 0
            || e.support.worked_example
            || e.support.read_aloud
        {
            continue;
        }
        // Exactly one observation per canonical task is retained. This newest result can
        // move an item between sessions but cannot increase the number of distinct checks.
        eligible_by_item.insert(
            (e.item_family.as_str(), o.parameter_fingerprint.as_str()),
            o,
        );
    }
    let mut items = eligible_by_item.into_values().collect::<Vec<_>>();
    items.sort_by_key(|o| o.evidence.server_sequence);
    Ok(items)
}

pub fn summary(skill_id: &str, state: &MissionState) -> Result<SkillSummary, MissionError> {
    if !supported_skill(skill_id) {
        return Err(MissionError::UnsupportedSkill);
    }
    let eligible = qualifying(skill_id, state)?;
    let window = &eligible[eligible.len().saturating_sub(QUALIFYING_CHECKS)..];
    let correct = window
        .iter()
        .filter(|o| o.evidence.result == ResultKind::Correct)
        .count();
    let variants: BTreeSet<_> = window.iter().map(|o| &o.evidence.variant_id).collect();
    let contexts: BTreeSet<_> = window.iter().map(|o| &o.evidence.context_id).collect();
    let representations: BTreeSet<_> = window.iter().map(|o| &o.evidence.representation).collect();
    let sessions: BTreeSet<_> = window.iter().map(|o| &o.evidence.session_id).collect();
    let ready = window.len() == QUALIFYING_CHECKS
        && correct >= REQUIRED_CORRECT
        && variants.len() >= 3
        && contexts.len() >= 2
        && representations.len() >= 2
        && sessions.len() >= 2
        && window
            .iter()
            .rev()
            .take(3)
            .all(|o| o.evidence.result == ResultKind::Correct);
    let status = if ready {
        Status::PracticeReady
    } else if !eligible.is_empty() || state.attempted_skills.contains(skill_id) {
        Status::Practising
    } else if state.introduced_skills.contains(skill_id) {
        Status::Introduced
    } else {
        Status::Unseen
    };
    let blockers = if ready {
        Vec::new()
    } else {
        let mut items = Vec::new();
        if window.len() < QUALIFYING_CHECKS {
            items.push(format!(
                "{} more independent first checks needed",
                QUALIFYING_CHECKS - window.len()
            ));
        }
        if correct < REQUIRED_CORRECT {
            items.push(format!(
                "need at least {REQUIRED_CORRECT} correct checks in the qualifying window"
            ));
        }
        if variants.len() < 3 {
            items.push("need at least 3 distinct task variants".to_owned());
        }
        if contexts.len() < 2 {
            items.push("need at least 2 authored contexts".to_owned());
        }
        if representations.len() < 2 {
            items.push("need at least 2 representations".to_owned());
        }
        if sessions.len() < 2 {
            items.push("need checks from 2 authenticated sessions".to_owned());
        }
        if !window
            .iter()
            .rev()
            .take(3)
            .all(|o| o.evidence.result == ResultKind::Correct)
        {
            items.push("the latest 3 first checks must be correct".to_owned());
        }
        items
    };
    let next_step = if ready {
        "Ready for the next supported mission skill under the provisional mission policy. This is not a calibrated academic mastery claim.".to_owned()
    } else if window.len() < QUALIFYING_CHECKS {
        format!(
            "Keep practising: {} of {} independent first checks are available; vary the task and return in another session.",
            window.len(),
            QUALIFYING_CHECKS
        )
    } else if sessions.len() < 2 {
        "Keep practising in another authenticated session; a page reload does not create a new session.".to_owned()
    } else if contexts.len() < 2 || representations.len() < 2 || variants.len() < 3 {
        "Try different activity contexts, representations and task variants before advancing."
            .to_owned()
    } else {
        "Keep practising this skill; the latest checks need more correct answers before the next skill unlocks.".to_owned()
    };
    Ok(SkillSummary {
        skill_id: skill_id.to_owned(),
        status,
        eligible_checks: window.len(),
        correct_checks: correct,
        distinct_variants: variants.len(),
        distinct_contexts: contexts.len(),
        distinct_representations: representations.len(),
        distinct_sessions: sessions.len(),
        blockers,
        next_step,
    })
}

pub fn all_skill_projections(state: &MissionState) -> Result<Vec<SkillProjection>, MissionError> {
    let graph = CurriculumGraph::load().map_err(|_| MissionError::InvalidState)?;
    graph.nodes().map(|node| projection(node, state)).collect()
}

pub fn frontier(
    state: &MissionState,
    subject: Option<&str>,
) -> Result<Vec<SkillProjection>, MissionError> {
    let graph = CurriculumGraph::load().map_err(|_| MissionError::InvalidState)?;
    let mut ready = Vec::new();
    for node in graph
        .nodes()
        .filter(|n| subject.is_none_or(|s| n.subject == s) && supported_skill(&n.id))
    {
        if summary(&node.id, state)?.status == Status::PracticeReady {
            continue;
        }
        if node
            .prerequisites
            .iter()
            .all(|pre| summary(pre, state).is_ok_and(|s| s.status == Status::PracticeReady))
        {
            let mut item = projection(node, state)?;
            if matches!(item.status, Status::Ready | Status::Unseen) {
                item.status = Status::Ready;
            }
            ready.push(item);
        }
    }
    Ok(ready)
}

pub fn select(
    state: &MissionState,
    subject: Option<&str>,
    seed: u64,
) -> Result<Option<SkillProjection>, MissionError> {
    let mut frontier = frontier(state, subject)?;
    if frontier.is_empty() {
        return Ok(None);
    }
    frontier.sort_by(|a, b| a.skill_id.cmp(&b.skill_id));
    let least_practised = frontier
        .iter()
        .map(|s| summary(&s.skill_id, state).map(|v| v.eligible_checks))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .min()
        .unwrap_or(0);
    frontier.retain(|s| {
        summary(&s.skill_id, state).is_ok_and(|v| v.eligible_checks == least_practised)
    });
    Ok(Some(
        frontier[(seed % frontier.len() as u64) as usize].clone(),
    ))
}

fn projection(node: &SkillNode, state: &MissionState) -> Result<SkillProjection, MissionError> {
    let supported = supported_skill(&node.id);
    let status = if !supported {
        Status::Unavailable
    } else {
        match summary(&node.id, state)?.status {
            Status::PracticeReady => Status::PracticeReady,
            Status::Practising => Status::Practising,
            Status::Unseen | Status::Introduced
                if node.prerequisites.is_empty()
                    || node.prerequisites.iter().all(|p| {
                        summary(p, state).is_ok_and(|s| s.status == Status::PracticeReady)
                    }) =>
            {
                Status::Ready
            }
            Status::Unseen => Status::Locked,
            other => other,
        }
    };
    Ok(SkillProjection {
        skill_id: node.id.clone(),
        subject: node.subject.clone(),
        objective: node.objective.clone(),
        prerequisites: node.prerequisites.clone(),
        status,
        supported_by_mission: supported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderwilds_contracts::{
        LearningEvidenceProvenance, LearningEvidenceResult, LearningEvidenceSchemaVersion,
        SupportState,
    };

    #[test]
    fn mission_answer_deserializes_camel_case_choice_ids_from_the_wire() {
        let answer: MissionAnswer =
            serde_json::from_value(serde_json::json!({"kind":"choice","choiceId":"number-1"}))
                .unwrap();
        assert_eq!(
            answer,
            MissionAnswer::Choice {
                choice_id: "number-1".into()
            }
        );
    }

    fn answer_from_public_view(view: &MissionActivityView) -> MissionAnswer {
        let choice = |choices: &[NumberChoice], value: i8| {
            choices
                .iter()
                .find(|candidate| candidate.value == value)
                .unwrap()
                .id
                .clone()
        };
        match &view.task {
            PublicTask::Pairing {
                objects,
                recipients,
                ..
            } => MissionAnswer::Pairing {
                assignments: objects
                    .iter()
                    .zip(recipients)
                    .map(|(object, recipient)| PairAssignment {
                        object_id: object.id.clone(),
                        recipient_id: recipient.id.clone(),
                    })
                    .collect(),
            },
            PublicTask::CountCollection {
                objects, choices, ..
            } => MissionAnswer::Choice {
                choice_id: choice(choices, objects.len() as i8),
            },
            PublicTask::CompareCollections {
                collections,
                prompt_mode,
                choices,
            } => {
                let target = if prompt_mode == "more" {
                    collections.iter().max_by_key(|group| group.items.len())
                } else {
                    collections.iter().min_by_key(|group| group.items.len())
                }
                .unwrap();
                MissionAnswer::Choice {
                    choice_id: choices
                        .iter()
                        .find(|item| item.id == target.id)
                        .unwrap()
                        .id
                        .clone(),
                }
            }
            PublicTask::NumberPath {
                shown,
                direction,
                choices,
            } => {
                let last = *shown.last().unwrap();
                let next = last + if direction == "forward" { 1 } else { -1 };
                MissionAnswer::Choice {
                    choice_id: choice(choices, next),
                }
            }
            PublicTask::PartWhole {
                whole,
                shown_part,
                choices,
            } => MissionAnswer::Choice {
                choice_id: choice(choices, *whole as i8 - *shown_part as i8),
            },
        }
    }

    #[test]
    fn every_supported_authored_task_accepts_a_wire_answer_derived_from_its_public_view() {
        for skill in SUPPORTED_SKILLS {
            for seed in 0..12 {
                let activity = authored_activity(skill, seed, "test-activity").unwrap();
                let generated_answer = answer_from_public_view(&activity.view);
                let wire = serde_json::to_value(generated_answer).unwrap();
                wonderwilds_contracts::validate("MissionLearningAnswer", &wire)
                    .unwrap_or_else(|error| panic!("{skill} seed {seed}: {error}"));
                let answer: MissionAnswer = serde_json::from_value(wire).unwrap();
                let outcome = activity
                    .assess(
                        &answer,
                        Support {
                            hint_count: 0,
                            worked_example: false,
                            read_aloud: false,
                        },
                    )
                    .unwrap();
                assert!(outcome.correct, "{skill} seed {seed}");
            }
        }
    }

    fn observation(skill: &str, i: usize, correct: bool) -> MissionObservation {
        MissionObservation {
            evidence: LearningEvidence {
                schema_version: LearningEvidenceSchemaVersion::V10,
                event_id: format!("event-{i}"),
                encounter_id: format!("encounter-{i}"),
                skill_id: skill.into(),
                variant_id: format!("variant-{}", i % 4),
                item_family: skill.into(),
                context_id: format!("context-{}", i % 2),
                representation: format!("representation-{}", i % 2),
                result: if correct {
                    LearningEvidenceResult::Correct
                } else {
                    LearningEvidenceResult::Incorrect
                },
                event_kind: EventKind::AnswerCheck,
                first_check: true,
                support: SupportState {
                    hint_count: 0,
                    worked_example: false,
                    read_aloud: false,
                },
                session_id: format!("session-{}", i % 2),
                server_sequence: i as u64 + 1,
                provenance: LearningEvidenceProvenance::ServerVerified,
            },
            parameter_fingerprint: format!("item-{i}"),
        }
    }

    #[test]
    fn graph_frontier_starts_at_m01_and_unlocks_only_after_each_prerequisite() {
        let state = MissionState::default();
        let initial_frontier = frontier(&state, Some("maths")).unwrap();
        assert_eq!(
            initial_frontier
                .iter()
                .map(|s| s.skill_id.as_str())
                .collect::<Vec<_>>(),
            ["M01"]
        );
        assert_eq!(
            all_skill_projections(&state)
                .unwrap()
                .iter()
                .filter(|s| s.status == Status::Unavailable)
                .count(),
            115
        );
        let mut state = state;
        for i in 0..8 {
            record(&mut state, observation("M01", i, i != 3)).unwrap();
        }
        assert_eq!(
            summary("M01", &state).unwrap().status,
            Status::PracticeReady
        );
        assert_eq!(
            frontier(&state, Some("maths"))
                .unwrap()
                .iter()
                .map(|s| s.skill_id.as_str())
                .collect::<Vec<_>>(),
            ["M03"]
        );
    }

    #[test]
    fn threshold_requires_varied_independent_checks_and_two_sessions() {
        let mut state = MissionState::default();
        for i in 0..8 {
            record(&mut state, observation("M01", i, i != 4)).unwrap();
        }
        let result = summary("M01", &state).unwrap();
        assert_eq!(result.status, Status::PracticeReady);
        assert!(
            result
                .next_step
                .contains("not a calibrated academic mastery claim")
        );
        for o in &mut state.observations {
            o.evidence.session_id = "one-session".into();
        }
        assert_eq!(summary("M01", &state).unwrap().status, Status::Practising);
    }

    #[test]
    fn imported_supported_repeated_and_duplicate_events_cannot_establish_progress() {
        let mut state = MissionState::default();
        let mut imported = observation("M01", 0, true);
        imported.evidence.provenance = LearningEvidenceProvenance::LegacyUnverified;
        record(&mut state, imported.clone()).unwrap();
        for i in 1..9 {
            let mut o = observation("M01", i, true);
            o.evidence.support.hint_count = 1;
            record(&mut state, o).unwrap();
        }
        assert_eq!(summary("M01", &state).unwrap().status, Status::Practising);
        assert_eq!(summary("M01", &state).unwrap().eligible_checks, 0);
        assert_eq!(
            record(&mut state, imported),
            Err(MissionError::SequenceOrder)
        );
    }

    #[test]
    fn replay_replaces_same_item_across_sessions_without_farming_check_count() {
        let mut history = (0..12)
            .map(|i| observation("M01", i, true))
            .collect::<Vec<_>>();
        for item in &mut history {
            item.evidence.session_id = "session-first".into();
        }
        let first = replay(&history).unwrap();
        assert_eq!(summary("M01", &first).unwrap().status, Status::Practising);
        // Four canonical items rechecked in another session still contribute once each.
        for i in 0..4 {
            let mut later = observation("M01", 20 + i, true);
            later.parameter_fingerprint = format!("item-{i}");
            later.evidence.variant_id = format!("variant-{}", i % 4);
            later.evidence.session_id = "session-second".into();
            history.push(later);
        }
        let replayed = replay(&history).unwrap();
        assert_eq!(replayed.observations.len(), 12);
        assert_eq!(replayed.last_sequence, 24);
        let ready = summary("M01", &replayed).unwrap();
        assert_eq!(ready.status, Status::PracticeReady);
        assert_eq!(ready.eligible_checks, 8);
        assert_eq!(ready.distinct_sessions, 2);
    }

    #[test]
    fn replay_rejects_forged_duplicate_first_checks_and_support_regression() {
        let first = observation("M01", 0, true);
        let mut second = observation("M01", 1, true);
        second.evidence.encounter_id = first.evidence.encounter_id.clone();
        assert_eq!(
            replay(&[first.clone(), second]),
            Err(MissionError::InvalidState)
        );

        let mut hint = observation("M01", 0, true);
        hint.evidence.event_kind = EventKind::Hint;
        hint.evidence.first_check = false;
        hint.evidence.support.hint_count = 1;
        let mut regressed = observation("M01", 1, true);
        regressed.evidence.encounter_id = hint.evidence.encounter_id.clone();
        regressed.evidence.support.hint_count = 0;
        assert_eq!(replay(&[hint, regressed]), Err(MissionError::InvalidState));
    }

    #[test]
    fn activities_are_deterministic_and_pairing_checks_a_full_bijection() {
        let a = authored_activity("M01", 7, "task-1").unwrap();
        let b = authored_activity("M01", 7, "task-1").unwrap();
        assert_eq!(a.view, b.view);
        assert!(!serde_json::to_string(&a.view).unwrap().contains("correct"));
        let PublicTask::Pairing {
            objects,
            recipients,
            ..
        } = &a.view.task
        else {
            panic!()
        };
        let assignments: Vec<PairAssignment> = objects
            .iter()
            .zip(recipients)
            .map(|(o, r)| PairAssignment {
                object_id: o.id.clone(),
                recipient_id: r.id.clone(),
            })
            .collect();
        assert!(
            a.assess(
                &MissionAnswer::Pairing {
                    assignments: assignments.clone()
                },
                Support {
                    hint_count: 0,
                    worked_example: false,
                    read_aloud: false
                }
            )
            .unwrap()
            .correct
        );
        let duplicate_recipient = assignments
            .iter()
            .enumerate()
            .map(|(i, a)| PairAssignment {
                object_id: a.object_id.clone(),
                recipient_id: if i == 0 {
                    assignments[1].recipient_id.clone()
                } else {
                    a.recipient_id.clone()
                },
            })
            .collect();
        assert!(
            !a.assess(
                &MissionAnswer::Pairing {
                    assignments: duplicate_recipient
                },
                Support {
                    hint_count: 0,
                    worked_example: false,
                    read_aloud: false
                }
            )
            .unwrap()
            .correct
        );
        assert_eq!(
            a.assess(
                &MissionAnswer::Pairing {
                    assignments: vec![]
                },
                Support {
                    hint_count: 0,
                    worked_example: false,
                    read_aloud: false
                }
            ),
            Err(MissionError::InvalidAnswer)
        );
    }

    #[test]
    fn finite_task_bank_has_eight_distinct_canonical_variants() {
        for skill in SUPPORTED_SKILLS {
            let unique: BTreeSet<_> = (0..12)
                .map(|seed| {
                    authored_activity(skill, seed, "x")
                        .unwrap()
                        .canonical_fingerprint()
                        .to_owned()
                })
                .collect();
            assert!(
                unique.len() >= 8,
                "{skill}: only {} unique canonical variants",
                unique.len()
            );
        }
    }

    #[test]
    fn mission_representations_match_the_graph_and_number_paths_are_bounded() {
        let graph = CurriculumGraph::load().unwrap();
        for skill in SUPPORTED_SKILLS {
            let node = graph.get(skill).unwrap();
            for seed in 0..12 {
                let activity = authored_activity(skill, seed, "bounded-item").unwrap();
                assert!(
                    node.allowed_representations
                        .contains(&activity.view.representation)
                );
                let mut wire = serde_json::to_value(&activity.view).unwrap();
                wire["support"] = serde_json::json!({
                    "hintCount": 0,
                    "workedExample": false,
                    "readAloud": false
                });
                wonderwilds_contracts::validate("MissionActivity", &wire).unwrap_or_else(|error| {
                    panic!("{skill} seed {seed} did not match MissionActivity contract: {error:?}")
                });
                if let PublicTask::NumberPath { shown, choices, .. } = &activity.view.task {
                    assert!(shown.iter().all(|n| (1..=20).contains(n)));
                    assert!(choices.iter().all(|n| (1..=20).contains(&n.value)));
                }
            }
        }
    }
}
