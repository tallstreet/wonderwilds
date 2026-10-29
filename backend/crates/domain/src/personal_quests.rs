//! Bounded authored quest documents and their deterministic interpreter data.
//! Documents provide narrative and allowlisted activity references. The catalog,
//! not a document or future generator, owns unlock and cosmetic reward policy.
use crate::engine::Rejection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestDocument {
    pub schema_version: String,
    pub quest_id: String,
    pub document_version: String,
    pub title: String,
    pub companion_id: String,
    pub region_id: String,
    pub interest_tags: Vec<String>,
    pub entry_node_id: String,
    pub nodes: Vec<QuestNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum QuestNode {
    Choice {
        id: String,
        text: String,
        choices: Vec<QuestChoice>,
    },
    Activity {
        id: String,
        text: String,
        #[serde(rename = "activitySlotId")]
        activity_slot_id: String,
        #[serde(rename = "nextNodeId")]
        next_node_id: String,
    },
    Handoff {
        id: String,
        text: String,
        #[serde(rename = "continuationTags")]
        continuation_tags: Vec<String>,
    },
}
impl QuestNode {
    pub fn id(&self) -> &str {
        match self {
            Self::Choice { id, .. } | Self::Activity { id, .. } | Self::Handoff { id, .. } => id,
        }
    }
    pub fn is_handoff(&self) -> bool {
        matches!(self, Self::Handoff { .. })
    }
    fn text(&self) -> &str {
        match self {
            Self::Choice { text, .. }
            | Self::Activity { text, .. }
            | Self::Handoff { text, .. } => text,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestChoice {
    pub id: String,
    pub label: String,
    pub next_node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActivitySlot {
    pub id: String,
    pub skill_id: String,
    pub template_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CatalogDocument {
    schema_version: String,
    content_version: String,
    review_status: String,
    release_enabled: bool,
    activity_slots: Vec<ActivitySlot>,
    quests: Vec<CatalogEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogEntry {
    #[serde(default)]
    pub continuation: Option<ContinuationPolicy>,
    pub quest_id: String,
    pub document_path: String,
    pub minimum_character_level: usize,
    pub reward_item_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContinuationPolicy {
    pub tag: String,
    pub source_quest_ids: Vec<String>,
    pub acknowledgement: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrativeMemory {
    pub recent_choices: Vec<NarrativeChoice>,
    pub recent_outcomes: Vec<NarrativeOutcome>,
    pub threads: Vec<NarrativeThread>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrativeChoice {
    pub document_version: String,
    pub quest_id: String,
    pub episode_id: String,
    pub node_id: String,
    pub choice_id: String,
    pub label: String,
    pub sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrativeOutcome {
    pub document_version: String,
    pub quest_id: String,
    pub episode_id: String,
    pub summary: String,
    pub sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrativeThread {
    pub source_choice_label: String,
    pub source_document_version: String,
    pub id: String,
    pub tag: String,
    pub source_quest_id: String,
    pub source_episode_id: String,
    pub source_node_id: String,
    pub source_choice_id: String,
    pub summary: String,
    pub status: String,
}
impl NarrativeMemory {
    pub fn is_empty(&self) -> bool {
        self.recent_choices.is_empty() && self.recent_outcomes.is_empty() && self.threads.is_empty()
    }
    pub fn validate(&self) -> Result<(), Rejection> {
        wonderwilds_contracts::validate(
            "NarrativeMemory",
            &serde_json::to_value(self).map_err(|_| invalid("narrative memory JSON"))?,
        )
        .map_err(|_| invalid("narrative memory bounds"))?;
        if self
            .threads
            .iter()
            .map(|t| &t.id)
            .collect::<BTreeSet<_>>()
            .len()
            != self.threads.len()
        {
            return Err(invalid("duplicate narrative thread"));
        }
        Ok(())
    }
}
fn bounded_push<T>(items: &mut Vec<T>, item: T, maximum: usize) {
    if items.len() == maximum {
        items.remove(0);
    }
    items.push(item);
}
pub fn remember_choice(memory: &mut NarrativeMemory, choice: NarrativeChoice) {
    bounded_push(&mut memory.recent_choices, choice, 16);
}
pub fn remember_outcome(memory: &mut NarrativeMemory, outcome: NarrativeOutcome) {
    bounded_push(&mut memory.recent_outcomes, outcome, 8);
}
pub fn remember_thread(memory: &mut NarrativeMemory, thread: NarrativeThread) {
    bounded_push(&mut memory.threads, thread, 8);
}

#[derive(Clone, Debug)]
pub struct Catalog {
    version: String,
    slots: Vec<ActivitySlot>,
    entries: BTreeMap<String, CatalogEntry>,
    documents: BTreeMap<String, QuestDocument>,
}

fn invalid(reason: &str) -> Rejection {
    Rejection::InvalidContent(reason.into())
}
fn id(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= maximum
}
fn tags(values: &[String]) -> bool {
    !values.is_empty()
        && values.len() <= 8
        && values.iter().all(|v| id(v, 48))
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

/// Validate the server-issued skill/template vocabulary before pinning it in a
/// run. IDs alone are insufficient when restoring older, untrusted JSON saves.
pub fn validate_slots(slots: &[ActivitySlot]) -> Result<(), Rejection> {
    if slots.is_empty()
        || slots.len() > 16
        || slots
            .iter()
            .map(|slot| &slot.id)
            .collect::<BTreeSet<_>>()
            .len()
            != slots.len()
    {
        return Err(invalid("invalid activity slot bounds"));
    }
    for slot in slots {
        let value = serde_json::to_value(slot).map_err(|_| invalid("invalid activity slot"))?;
        wonderwilds_contracts::validate("QuestActivitySlot", &value)
            .map_err(|_| invalid("activity slot schema"))?;
        if !id(&slot.id, 40)
            || !matches!(
                (slot.skill_id.as_str(), slot.template_id.as_str()),
                ("M06", "missing_quantity")
                    | ("M08", "equal_sharing")
                    | ("M05" | "M09" | "M14", "pattern_sort")
                    | ("M11" | "M12", "spatial_measurement")
                    | ("E05" | "E06" | "E07", "phonics_word_building")
                    | ("E01" | "E10" | "E11" | "E12", "story_message")
            )
        {
            return Err(invalid("invalid activity skill/template binding"));
        }
    }
    Ok(())
}

/// Structural graph validation shared by authored content and future proposals.
/// Context-specific actor/region/interest allowlists remain the director's job.
pub fn validate_document(
    document: &QuestDocument,
    slots: &[ActivitySlot],
) -> Result<(), Rejection> {
    validate_slots(slots)?;
    let wire = serde_json::to_value(document).map_err(|_| invalid("invalid quest wire value"))?;
    wonderwilds_contracts::validate("QuestDocument", &wire)
        .map_err(|_| invalid("quest document schema"))?;
    if document.schema_version != "1.0"
        || !id(&document.quest_id, 48)
        || !text(&document.document_version, 40)
        || !text(&document.title, 120)
        || !id(&document.companion_id, 40)
        || !id(&document.region_id, 40)
        || !id(&document.entry_node_id, 24)
        || !tags(&document.interest_tags)
        || document.nodes.len() < 3
        || document.nodes.len() > 16
    {
        return Err(invalid("invalid quest document bounds"));
    }
    let mut nodes = BTreeMap::new();
    let mut branches = false;
    for node in &document.nodes {
        if !id(node.id(), 24) || !text(node.text(), 600) || nodes.insert(node.id(), node).is_some()
        {
            return Err(invalid("invalid or duplicate quest node"));
        }
        match node {
            QuestNode::Choice { choices, .. } => {
                if !(2..=3).contains(&choices.len())
                    || choices
                        .iter()
                        .any(|c| !id(&c.id, 40) || !text(&c.label, 120) || !id(&c.next_node_id, 24))
                    || choices.iter().map(|c| &c.id).collect::<BTreeSet<_>>().len() != choices.len()
                {
                    return Err(invalid("invalid quest choices"));
                }
                branches |= choices
                    .iter()
                    .map(|c| &c.next_node_id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    > 1;
            }
            QuestNode::Activity {
                activity_slot_id,
                next_node_id,
                ..
            } => {
                if !slots.iter().any(|slot| slot.id == *activity_slot_id) || !id(next_node_id, 24) {
                    return Err(invalid("unknown activity slot"));
                }
            }
            QuestNode::Handoff {
                continuation_tags, ..
            } => {
                if !tags(continuation_tags) {
                    return Err(invalid("invalid continuation tags"));
                }
            }
        }
    }
    if !branches {
        return Err(invalid("quest needs a meaningful branch"));
    }
    fn visit<'a>(
        key: &'a str,
        activity: bool,
        nodes: &BTreeMap<&'a str, &'a QuestNode>,
        visiting: &mut BTreeSet<&'a str>,
        seen: &mut BTreeSet<&'a str>,
        validated: &mut BTreeSet<(&'a str, bool)>,
    ) -> Result<(), Rejection> {
        if visiting.contains(key) {
            return Err(invalid("quest node cycle"));
        }
        if validated.contains(&(key, activity)) {
            return Ok(());
        }
        visiting.insert(key);
        let node = nodes
            .get(key)
            .ok_or_else(|| invalid("dangling quest node"))?;
        seen.insert(key);
        match node {
            QuestNode::Choice { choices, .. } => {
                for choice in choices {
                    visit(
                        &choice.next_node_id,
                        activity,
                        nodes,
                        visiting,
                        seen,
                        validated,
                    )?;
                }
            }
            QuestNode::Activity { next_node_id, .. } => {
                visit(next_node_id, true, nodes, visiting, seen, validated)?
            }
            QuestNode::Handoff { .. } if !activity => {
                return Err(invalid("handoff bypasses activity"));
            }
            QuestNode::Handoff { .. } => (),
        }
        visiting.remove(key);
        validated.insert((key, activity));
        Ok(())
    }
    let mut seen = BTreeSet::new();
    visit(
        &document.entry_node_id,
        false,
        &nodes,
        &mut BTreeSet::new(),
        &mut seen,
        &mut BTreeSet::new(),
    )?;
    if seen.len() != nodes.len() {
        return Err(invalid("unreachable quest node"));
    }
    Ok(())
}

impl Catalog {
    pub fn bundled() -> Result<Self, Rejection> {
        Self::load(
            include_str!("../../../../content/quests/catalog.v1.json"),
            &[
                include_str!("../../../../content/quests/pip-lantern-trail.v1.json"),
                include_str!("../../../../content/quests/tansy-welcome-garden.v1.json"),
                include_str!("../../../../content/quests/nib-sharing-picnic.v1.json"),
                include_str!("../../../../content/quests/flutter-pattern-parade.v1.json"),
                include_str!("../../../../content/quests/pip-woodland-whispers.v1.json"),
                include_str!("../../../../content/quests/pip-star-pebble.v1.json"),
            ],
        )
    }
    pub fn load(catalog_json: &str, documents: &[&str]) -> Result<Self, Rejection> {
        let raw: CatalogDocument =
            serde_json::from_str(catalog_json).map_err(|_| invalid("invalid quest catalog"))?;
        if raw.schema_version != "1.0"
            || raw.review_status != "pending"
            || raw.release_enabled
            || !text(&raw.content_version, 120)
            || !(4..=32).contains(&raw.quests.len())
            || raw.activity_slots.is_empty()
            || raw.activity_slots.len() > 16
            || raw
                .activity_slots
                .iter()
                .map(|s| &s.id)
                .collect::<BTreeSet<_>>()
                .len()
                != raw.activity_slots.len()
        {
            return Err(invalid("quest catalog review/version policy"));
        }
        let mut parsed = BTreeMap::new();
        for data in documents {
            let document: QuestDocument =
                serde_json::from_str(data).map_err(|_| invalid("invalid quest document JSON"))?;
            validate_document(&document, &raw.activity_slots)?;
            if parsed.insert(document.quest_id.clone(), document).is_some() {
                return Err(invalid("duplicate quest document"));
            }
        }
        let mut entries = BTreeMap::new();
        let mut companions = BTreeSet::new();
        for entry in raw.quests {
            let document = parsed
                .get(&entry.quest_id)
                .ok_or_else(|| invalid("missing quest document"))?;
            companions.insert(document.companion_id.clone());
            if entry.minimum_character_level != 4
                || !id(&entry.reward_item_id, 80)
                || entry.document_path != format!("content/quests/{}.v1.json", entry.quest_id)
                || !["pip", "tansy", "nib", "flutter"].contains(&document.companion_id.as_str())
                || document.region_id != "meadow"
                || entries.insert(entry.quest_id.clone(), entry).is_some()
            {
                return Err(invalid("invalid catalog quest policy"));
            }
        }
        for entry in entries.values() {
            if let Some(policy) = &entry.continuation {
                let target = parsed
                    .get(&entry.quest_id)
                    .ok_or_else(|| invalid("missing continuation target"))?;
                if policy.source_quest_ids.is_empty()
                    || policy.source_quest_ids.len() > 8
                    || policy
                        .source_quest_ids
                        .iter()
                        .collect::<BTreeSet<_>>()
                        .len()
                        != policy.source_quest_ids.len()
                    || !id(&policy.tag, 48)
                    || !text(&policy.acknowledgement, 600)
                {
                    return Err(invalid("invalid continuation policy"));
                }
                for source_id in &policy.source_quest_ids {
                    let source = parsed
                        .get(source_id)
                        .ok_or_else(|| invalid("missing continuation source"))?;
                    if source.companion_id!=target.companion_id || !source.nodes.iter().any(|node|matches!(node,QuestNode::Handoff{continuation_tags,..} if continuation_tags.contains(&policy.tag))) { return Err(invalid("invalid continuation policy")); }
                }
            }
        }
        let mut reachable: BTreeSet<String> = entries
            .values()
            .filter(|entry| entry.continuation.is_none())
            .map(|entry| entry.quest_id.clone())
            .collect();
        loop {
            let before = reachable.len();
            for entry in entries.values() {
                if entry.continuation.as_ref().is_some_and(|policy| {
                    policy
                        .source_quest_ids
                        .iter()
                        .any(|source| reachable.contains(source))
                }) {
                    reachable.insert(entry.quest_id.clone());
                }
            }
            if reachable.len() == before {
                break;
            }
        }
        if reachable.len() != entries.len()
            || parsed
                .values()
                .flat_map(|document| document.interest_tags.iter())
                .collect::<BTreeSet<_>>()
                .len()
                > 32
        {
            return Err(invalid(
                "unreachable continuation bank or oversized interests",
            ));
        }
        if companions.len() != 4
            || entries
                .values()
                .map(|e| &e.reward_item_id)
                .collect::<BTreeSet<_>>()
                .len()
                != entries.len()
        {
            return Err(invalid("catalog coverage/reward uniqueness"));
        }
        if entries.len() != parsed.len() {
            return Err(invalid("unlisted quest document"));
        }
        Ok(Self {
            version: raw.content_version,
            slots: raw.activity_slots,
            entries,
            documents: parsed,
        })
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn activity_slots(&self) -> &[ActivitySlot] {
        &self.slots
    }
    pub fn interest_options(&self) -> Vec<String> {
        self.documents
            .values()
            .flat_map(|d| d.interest_tags.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn documents(&self) -> impl Iterator<Item = &QuestDocument> {
        self.documents.values()
    }
    pub fn document(&self, quest_id: &str) -> Option<&QuestDocument> {
        self.documents.get(quest_id)
    }
    pub fn entry(&self, quest_id: &str) -> Option<&CatalogEntry> {
        self.entries.get(quest_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersonalQuestRun {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_thread: Option<NarrativeThread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledgement: Option<String>,
    pub document: QuestDocument,
    pub episode_id: String,
    pub episode_sequence: u64,
    pub node_id: String,
    pub slots: Vec<ActivitySlot>,
    pub reward_item_id: String,
    pub choices: BTreeMap<String, String>,
}
impl PersonalQuestRun {
    pub fn node(&self) -> Option<&QuestNode> {
        self.document.nodes.iter().find(|n| n.id() == self.node_id)
    }
    pub fn last_choice(&self) -> Option<(String, String)> {
        let mut node_id = self.document.entry_node_id.as_str();
        let mut last = None;
        for _ in 0..self.document.nodes.len() {
            if node_id == self.node_id {
                return last;
            }
            let node = self
                .document
                .nodes
                .iter()
                .find(|node| node.id() == node_id)?;
            node_id = match node {
                QuestNode::Choice { id, choices, .. } => {
                    let selected = self.choices.get(id)?;
                    let choice = choices.iter().find(|c| c.id == *selected)?;
                    last = Some((id.clone(), selected.clone()));
                    &choice.next_node_id
                }
                QuestNode::Activity { next_node_id, .. } => next_node_id,
                QuestNode::Handoff { .. } => return None,
            };
        }
        None
    }
    pub fn activity(&self) -> Option<PersonalActivityBinding> {
        let QuestNode::Activity {
            activity_slot_id, ..
        } = self.node()?
        else {
            return None;
        };
        let slot = self
            .slots
            .iter()
            .find(|slot| slot.id == *activity_slot_id)?;
        Some(PersonalActivityBinding {
            quest_id: format!(
                "personal-{}-{}-{}",
                self.document.quest_id, self.episode_sequence, self.node_id
            ),
            document_quest_id: self.document.quest_id.clone(),
            document_version: self.document.document_version.clone(),
            episode_id: self.episode_id.clone(),
            node_id: self.node_id.clone(),
            activity_slot_id: slot.id.clone(),
            skill_id: slot.skill_id.clone(),
            template_id: slot.template_id.clone(),
        })
    }
    pub fn validate(&self) -> Result<(), Rejection> {
        validate_document(&self.document, &self.slots)?;
        if self.slots.len() > 16
            || self.node().is_none()
            || self.episode_sequence == 0
            || self.episode_id != format!("episode-{}", self.episode_sequence)
            || !id(&self.reward_item_id, 80)
            || self.choices.len() > 16
        {
            return Err(invalid("invalid pinned personal quest"));
        }
        if self.source_thread.is_some() != self.acknowledgement.is_some() {
            return Err(invalid("incomplete continuity binding"));
        }
        if let Some(thread) = &self.source_thread {
            NarrativeMemory {
                threads: vec![thread.clone()],
                ..Default::default()
            }
            .validate()?;
            if !text(self.acknowledgement.as_deref().unwrap_or(""), 600) {
                return Err(invalid("invalid acknowledgement"));
            }
        }
        for (node_id, choice_id) in &self.choices {
            if !self.document.nodes.iter().any(|n| matches!(n, QuestNode::Choice { id, choices, .. } if id == node_id && choices.iter().any(|c| c.id == *choice_id))) {
                return Err(invalid("invalid personal choice memory"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalActivityBinding {
    pub quest_id: String,
    pub document_quest_id: String,
    pub document_version: String,
    pub episode_id: String,
    pub node_id: String,
    pub activity_slot_id: String,
    pub skill_id: String,
    pub template_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalQuestOffer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    pub quest_id: String,
    pub document_version: String,
    pub title: String,
    pub companion_id: String,
    pub minimum_character_level: usize,
    pub available: bool,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalQuestView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledgement: Option<String>,
    pub quest_id: String,
    pub document_version: String,
    pub episode_id: String,
    pub title: String,
    pub companion_id: String,
    pub node: QuestNode,
}
