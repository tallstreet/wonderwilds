//! Commands, persisted state and domain outcomes.
use super::transitions::training_eligible;
use super::{Content, TrainingDefinition};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Command {
    pub command_id: String,
    pub expected_revision: u64,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    BeginQuest {
        quest_id: String,
        encounter_id: String,
    },
    CompleteQuest {
        quest_id: String,
        encounter_id: String,
    },
    AbandonQuest {
        quest_id: String,
        encounter_id: String,
    },
    Travel {
        region_id: String,
    },
    Discover,
    SelectTeam {
        creature_ids: Vec<String>,
    },
    HelpFriend {
        creature_id: String,
        completed_quest_id: String,
    },
    Transform {
        creature_id: String,
        enabled: bool,
    },
    UseItem {
        item_id: String,
        amount: u32,
    },
}
/// Server-owned, persisted encounter completion, not a request body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedCompletion {
    pub quest_id: String,
    pub supported: bool,
}
/// Trusted transaction inputs. Assessment/factory adapters must verify profile,
/// encounter/session ownership, hint history, and server correctness first.
/// `reachable_quests` comes from server world/path validation, never browser XY.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedFacts {
    pub issued_encounters: BTreeMap<String, String>,
    pub completed_encounters: BTreeMap<String, VerifiedCompletion>,
    pub reachable_quests: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub unix_millis: u64,
    pub recorded_random_choice: u32,
    pub facts: TrustedFacts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub(super) revision: u64,
    pub(super) sequence: u64,
    pub(super) content_version: String,
    pub(super) region: String,
    pub(super) completed: BTreeSet<String>,
    pub(super) pending: BTreeMap<String, String>,
    pub(super) flags: BTreeSet<String>,
    pub(super) features: BTreeSet<String>,
    pub(super) inventory: BTreeMap<String, u32>,
    pub(super) creatures: BTreeSet<String>,
    pub(super) friendship: BTreeMap<String, u32>,
    pub(super) helping_moments: BTreeSet<(String, String)>,
    pub(super) transformed: BTreeSet<String>,
    pub(super) team: Vec<String>,
    pub(super) xp: u32,
    #[serde(default)]
    pub(super) character_xp: BTreeMap<String, u32>,
    #[serde(default)]
    pub(super) training_grants: BTreeSet<String>,
    pub(super) grants: BTreeSet<(String, String)>,
    pub(super) discoveries: BTreeMap<String, String>,
    pub(super) last_discovery: BTreeMap<String, String>,
    pub(super) receipts: BTreeMap<String, Receipt>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Receipt {
    pub(super) command: Command,
    pub(super) events: Vec<Event>,
}
impl State {
    pub fn new(content: &Content) -> Self {
        Self {
            revision: 0,
            sequence: 0,
            content_version: content.version.clone(),
            region: "meadow".into(),
            completed: BTreeSet::new(),
            pending: BTreeMap::new(),
            flags: BTreeSet::new(),
            features: BTreeSet::new(),
            inventory: BTreeMap::new(),
            creatures: BTreeSet::new(),
            friendship: BTreeMap::new(),
            helping_moments: BTreeSet::new(),
            transformed: BTreeSet::new(),
            team: Vec::new(),
            xp: 0,
            character_xp: BTreeMap::new(),
            training_grants: BTreeSet::new(),
            grants: BTreeSet::new(),
            discoveries: BTreeMap::new(),
            last_discovery: BTreeMap::new(),
            receipts: BTreeMap::new(),
        }
    }
    pub fn character_xp(&self, id: &str) -> u32 {
        *self.character_xp.get(id).unwrap_or(&0)
    }
    pub fn character_level(&self, content: &Content, id: &str) -> usize {
        content
            .character_level_thresholds()
            .iter()
            .filter(|&&xp| self.character_xp(id) >= xp)
            .count()
    }
    pub fn training_completed(&self, id: &str) -> bool {
        self.training_grants.contains(id)
    }
    pub fn available_training<'a>(&self, content: &'a Content) -> Vec<&'a TrainingDefinition> {
        content
            .training_definitions()
            .filter(|t| training_eligible(self, content, t).is_ok())
            .collect()
    }
    pub fn team_capacity(&self) -> usize {
        usize::from(self.features.contains("team_slot_1"))
            + usize::from(self.features.contains("team_slot_2"))
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn region(&self) -> &str {
        &self.region
    }
    pub fn xp(&self) -> u32 {
        self.xp
    }
    pub fn level(&self, content: &Content) -> usize {
        content
            .levels
            .iter()
            .filter(|&&threshold| self.xp >= threshold)
            .count()
    }
    pub fn completed(&self) -> &BTreeSet<String> {
        &self.completed
    }
    pub fn flags(&self) -> &BTreeSet<String> {
        &self.flags
    }
    pub fn creatures(&self) -> &BTreeSet<String> {
        &self.creatures
    }
    pub fn friendship(&self, creature: &str) -> u32 {
        *self.friendship.get(creature).unwrap_or(&0)
    }
    /// Server-derived distinct completed story moments eligible for friendship.
    pub fn available_helping_moments(&self, content: &Content) -> Vec<(String, String)> {
        content
            .creatures
            .values()
            .filter(|c| self.creatures.contains(&c.id) && c.region_id == self.region)
            .flat_map(|c| {
                content.beats.values().filter_map(move |b| {
                    let moment = (c.id.clone(), b.id.clone());
                    (b.chapter_id == c.region_id
                        && self.completed.contains(&b.id)
                        && !self.helping_moments.contains(&moment))
                    .then_some(moment)
                })
            })
            .collect()
    }
    pub fn inventory(&self) -> &BTreeMap<String, u32> {
        &self.inventory
    }
    pub fn team(&self) -> &[String] {
        &self.team
    }
    pub fn transformed(&self) -> &BTreeSet<String> {
        &self.transformed
    }
    pub fn offered_creature(&self) -> Option<&str> {
        self.discoveries.get(&self.region).map(String::as_str)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub sequence: u64,
    pub revision: u64,
    pub command_id: String,
    pub rules_version: String,
    pub content_version: String,
    pub unix_millis: u64,
    pub recorded_random_choice: u32,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Outcome {
    QuestStarted {
        quest_id: String,
        encounter_id: String,
    },
    QuestCompleted {
        quest_id: String,
        supported: bool,
    },
    TrainingCompleted {
        training_id: String,
        creature_id: String,
        supported: bool,
        xp_granted: u32,
        level: usize,
    },
    QuestAbandoned {
        quest_id: String,
        encounter_id: String,
    },
    RewardGranted {
        reward_id: String,
        quest_id: String,
    },
    Travelled {
        region_id: String,
    },
    CreatureOffered {
        creature_id: String,
    },
    TeamSelected {
        creature_ids: Vec<String>,
    },
    FriendshipIncreased {
        creature_id: String,
        points: u32,
    },
    TransformationSet {
        creature_id: String,
        enabled: bool,
    },
    ItemUsed {
        item_id: String,
        amount: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub state: State,
    pub events: Vec<Event>,
    pub duplicate: bool,
}
