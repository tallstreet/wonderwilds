//! Authored content loading and validation.
use super::{Rejection, valid_id};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoryPack {
    pub(super) schema_version: String,
    pub(super) content_version: String,
    pub(super) review_status: String,
    pub(super) chapters: Vec<Chapter>,
    pub(super) beats: Vec<Beat>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Chapter {
    pub(super) id: String,
    pub(super) entry_beat_id: String,
    pub(super) finale_beat_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Beat {
    pub(super) id: String,
    pub(super) skill_id: String,
    pub(super) template_id: String,
    pub(super) chapter_id: String,
    pub(super) requires_beat_ids: Vec<String>,
    pub(super) required_creature_ids: Vec<String>,
    pub(super) required_mastery_ids: Vec<String>,
    pub(super) completion_flags: Vec<String>,
    pub(super) rewards: Vec<Reward>,
    pub(super) finale_steps: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HelpPack {
    pub(super) schema_version: String,
    pub(super) content_version: String,
    pub(super) review_status: String,
    pub(super) quests: Vec<HelpQuest>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HelpQuest {
    pub(super) id: String,
    pub(super) skill_id: String,
    pub(super) template_id: String,
    pub(super) creature_id: String,
    pub(super) requires_beat_ids: Vec<String>,
    pub(super) support_available: bool,
    pub(super) supported_completion_befriends: bool,
    pub(super) rewards: Vec<Reward>,
    pub(super) repeat_rewards: Vec<Reward>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreaturePack {
    pub(super) schema_version: String,
    pub(super) content_version: String,
    pub(super) review_status: String,
    pub(super) creatures: Vec<Creature>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Creature {
    pub(super) id: String,
    pub(super) region_id: String,
    pub(super) help_quest_id: String,
    pub(super) discovery: Discovery,
    pub(super) transformation: Transformation,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Discovery {
    pub(super) eligible_after_beat: String,
    pub(super) guaranteed_first: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Transformation {
    pub(super) id: String,
    pub(super) minimum_friendship: u32,
    pub(super) required_story_flag: String,
    pub(super) reversible: bool,
    pub(super) cosmetic_only: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Reward {
    ExplorerXp {
        id: String,
        amount: u32,
    },
    StoryFlag {
        id: String,
        value: String,
    },
    Feature {
        id: String,
        value: String,
    },
    Creature {
        id: String,
        value: String,
    },
    InventoryItem {
        id: String,
        value: String,
        amount: u32,
    },
}
impl Reward {
    pub(super) fn id(&self) -> &str {
        match self {
            Self::ExplorerXp { id, .. }
            | Self::StoryFlag { id, .. }
            | Self::Feature { id, .. }
            | Self::Creature { id, .. }
            | Self::InventoryItem { id, .. } => id,
        }
    }
}

/// Server-internal authored training policy. No client supplies XP or recipient.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingDefinition {
    pub id: String,
    pub creature_id: String,
    pub skill_id: String,
    pub template_id: String,
    pub required_level: usize,
    pub xp: u32,
    pub reaction_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CharacterProgression {
    pub(super) rules_version: String,
    pub(super) levels: Vec<u32>,
    pub(super) supported_completion_multiplier: u32,
    pub(super) repeat_milestone_xp: u32,
    pub(super) training: Vec<TrainingDefinition>,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub(super) version: String,
    pub(super) chapters: BTreeMap<String, Chapter>,
    pub(super) beats: BTreeMap<String, Beat>,
    pub(super) help: BTreeMap<String, HelpQuest>,
    pub(super) creatures: BTreeMap<String, Creature>,
    pub(super) levels: Vec<u32>,
    pub(super) max_friendship: u32,
    pub(super) character_progression: CharacterProgression,
}
impl Content {
    /// Explicit development loader. Pending author/educator reviews cannot be
    /// mistaken for production activation; use `released` at a release boundary.
    pub fn development(
        story: &str,
        help: &str,
        creatures: &str,
        progression: &str,
    ) -> Result<Self, Rejection> {
        Self::load(story, help, creatures, progression, false)
    }
    pub fn released(
        story: &str,
        help: &str,
        creatures: &str,
        progression: &str,
    ) -> Result<Self, Rejection> {
        Self::load(story, help, creatures, progression, true)
    }
    fn load(
        story: &str,
        help: &str,
        creatures: &str,
        progression: &str,
        reviewed: bool,
    ) -> Result<Self, Rejection> {
        fn parse<T: for<'a> Deserialize<'a>>(s: &str) -> Result<T, Rejection> {
            serde_json::from_str(s).map_err(|e| Rejection::InvalidContent(e.to_string()))
        }
        let story: StoryPack = parse(story)?;
        let help: HelpPack = parse(help)?;
        let creatures: CreaturePack = parse(creatures)?;
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Progression {
            schema_version: String,
            content_version: String,
            explorer_levels: Vec<u32>,
            friendship: Friendship,
            character_progression: CharacterProgression,
        }
        #[derive(Deserialize)]
        struct Friendship {
            maximum: u32,
        }
        let progression: Progression = parse(progression)?;
        if [
            &story.schema_version,
            &help.schema_version,
            &creatures.schema_version,
            &progression.schema_version,
        ]
        .iter()
        .any(|v| v.as_str() != "1.0")
            || help.content_version != story.content_version
            || creatures.content_version != story.content_version
            || progression.content_version != story.content_version
            || (reviewed
                && [
                    &story.review_status,
                    &help.review_status,
                    &creatures.review_status,
                ]
                .iter()
                .any(|v| v.as_str() != "approved"))
        {
            return Err(Rejection::InvalidContent("version/review mismatch".into()));
        }
        let nc = story.chapters.len();
        let nb = story.beats.len();
        let nh = help.quests.len();
        let nf = creatures.creatures.len();
        let content = Self {
            version: story.content_version,
            chapters: story
                .chapters
                .into_iter()
                .map(|x| (x.id.clone(), x))
                .collect(),
            beats: story.beats.into_iter().map(|x| (x.id.clone(), x)).collect(),
            help: help.quests.into_iter().map(|x| (x.id.clone(), x)).collect(),
            creatures: creatures
                .creatures
                .into_iter()
                .map(|x| (x.id.clone(), x))
                .collect(),
            levels: progression.explorer_levels,
            max_friendship: progression.friendship.maximum,
            character_progression: progression.character_progression,
        };
        if [nc, nb, nh, nf].contains(&0)
            || (
                content.chapters.len(),
                content.beats.len(),
                content.help.len(),
                content.creatures.len(),
            ) != (nc, nb, nh, nf)
            || content.levels.first() != Some(&0)
            || content.levels.windows(2).any(|x| x[0] >= x[1])
            || content.max_friendship == 0
        {
            return Err(Rejection::InvalidContent(
                "duplicate IDs or invalid levels".into(),
            ));
        }
        let policy = &content.character_progression;
        if policy.rules_version != "character-training-1.0.0"
            || policy.levels.len() != 5
            || policy.levels.first() != Some(&0)
            || policy.levels.windows(2).any(|x| x[0] >= x[1])
            || policy.supported_completion_multiplier != 1
            || policy.repeat_milestone_xp != 0
        {
            return Err(Rejection::InvalidContent(
                "invalid character progression".into(),
            ));
        }
        let mut training_ids = BTreeSet::new();
        for t in &policy.training {
            let mapping = match t.creature_id.as_str() {
                "pip" => ("M06", "missing_quantity"),
                "tansy" => ("M05", "pattern_sort"),
                "nib" => ("M08", "equal_sharing"),
                "flutter" => ("M09", "pattern_sort"),
                _ => {
                    return Err(Rejection::InvalidContent(
                        "unsupported training creature".into(),
                    ));
                }
            };
            if (t.skill_id.as_str(), t.template_id.as_str()) != mapping
                || !valid_id(&t.id)
                || !training_ids.insert(&t.id)
                || content.beats.contains_key(&t.id)
                || content.help.contains_key(&t.id)
                || !content.creatures.contains_key(&t.creature_id)
                || t.required_level == 0
                || t.required_level >= policy.levels.len()
                || t.xp == 0
                || !valid_id(&t.reaction_id)
                || !valid_id(&t.skill_id)
                || !valid_id(&t.template_id)
            {
                return Err(Rejection::InvalidContent(
                    "invalid training definition".into(),
                ));
            }
        }
        for id in ["pip", "tansy", "nib", "flutter"] {
            let mut xp = 0u32;
            for level in 1..5 {
                let milestones: Vec<_> = policy
                    .training
                    .iter()
                    .filter(|t| t.creature_id == id && t.required_level == level)
                    .collect();
                if milestones.len() != 1 || xp < policy.levels[level - 1] {
                    return Err(Rejection::InvalidContent(
                        "unreachable character milestone".into(),
                    ));
                }
                xp = xp
                    .checked_add(milestones[0].xp)
                    .ok_or(Rejection::Overflow)?;
            }
            if xp < policy.levels[4] {
                return Err(Rejection::InvalidContent(
                    "unreachable character level five".into(),
                ));
            }
        }
        let mut reward_ids = BTreeSet::new();
        for beat in content.beats.values() {
            if !content.chapters.contains_key(&beat.chapter_id)
                || !beat.required_mastery_ids.is_empty()
                || beat
                    .required_creature_ids
                    .iter()
                    .any(|id| !content.creatures.contains_key(id))
                || beat
                    .requires_beat_ids
                    .iter()
                    .chain(&beat.finale_steps)
                    .any(|id| !content.beats.contains_key(id))
            {
                return Err(Rejection::InvalidContent(
                    "dangling beat or academic story gate".into(),
                ));
            }
        }
        for chapter in content.chapters.values() {
            if !content
                .beats
                .get(&chapter.entry_beat_id)
                .is_some_and(|b| b.chapter_id == chapter.id)
                || !content
                    .beats
                    .get(&chapter.finale_beat_id)
                    .is_some_and(|b| b.chapter_id == chapter.id)
            {
                return Err(Rejection::InvalidContent("chapter boundary".into()));
            }
        }
        for quest in content.help.values() {
            if !content
                .creatures
                .get(&quest.creature_id)
                .is_some_and(|c| c.help_quest_id == quest.id)
                || !quest.support_available
                || !quest.supported_completion_befriends
                || !quest.repeat_rewards.is_empty()
                || quest
                    .requires_beat_ids
                    .iter()
                    .any(|id| !content.beats.contains_key(id))
            {
                return Err(Rejection::InvalidContent("invalid help quest".into()));
            }
        }
        let flags: BTreeSet<_> = content
            .beats
            .values()
            .flat_map(|b| b.completion_flags.iter())
            .collect();
        for creature in content.creatures.values() {
            if !content.chapters.contains_key(&creature.region_id)
                || !content.help.contains_key(&creature.help_quest_id)
                || !content
                    .beats
                    .contains_key(&creature.discovery.eligible_after_beat)
                || !flags.contains(&creature.transformation.required_story_flag)
                || creature.transformation.minimum_friendship > content.max_friendship
                || !creature.transformation.reversible
                || !creature.transformation.cosmetic_only
            {
                return Err(Rejection::InvalidContent("invalid creature".into()));
            }
        }
        for reward in content
            .beats
            .values()
            .flat_map(|b| b.rewards.iter())
            .chain(content.help.values().flat_map(|q| q.rewards.iter()))
        {
            if !valid_id(reward.id()) || !reward_ids.insert(reward.id()) {
                return Err(Rejection::InvalidContent("duplicate reward".into()));
            }
            if let Reward::Creature { value, .. } = reward
                && !content.creatures.contains_key(value)
            {
                return Err(Rejection::InvalidContent("unknown reward creature".into()));
            }
        }
        // A topological fixed point rejects cycles without a recursive depth hazard.
        let mut seen = BTreeSet::new();
        loop {
            let before = seen.len();
            for b in content.beats.values() {
                if b.requires_beat_ids
                    .iter()
                    .chain(&b.finale_steps)
                    .all(|id| seen.contains(id))
                {
                    seen.insert(b.id.clone());
                }
            }
            if seen.len() == before {
                break;
            }
        }
        if seen.len() != content.beats.len() {
            return Err(Rejection::InvalidContent("cyclic story".into()));
        }
        Ok(content)
    }
    pub fn training(&self, id: &str) -> Option<&TrainingDefinition> {
        self.character_progression
            .training
            .iter()
            .find(|t| t.id == id)
    }
    pub fn training_definitions(&self) -> impl Iterator<Item = &TrainingDefinition> {
        self.character_progression.training.iter()
    }
    pub fn training_rules_version(&self) -> &str {
        &self.character_progression.rules_version
    }
    pub fn character_level_thresholds(&self) -> &[u32] {
        &self.character_progression.levels
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    /// Authoritative authored mapping. API factories must use this value rather
    /// than silently assigning a different learning objective to a story beat.
    pub fn quest_skill_id(&self, quest_id: &str) -> Option<&str> {
        self.beats
            .get(quest_id)
            .map(|b| b.skill_id.as_str())
            .or_else(|| self.help.get(quest_id).map(|q| q.skill_id.as_str()))
            .or_else(|| self.training(quest_id).map(|t| t.skill_id.as_str()))
    }
    pub fn quest_template_id(&self, quest_id: &str) -> Option<&str> {
        self.beats
            .get(quest_id)
            .map(|b| b.template_id.as_str())
            .or_else(|| self.help.get(quest_id).map(|q| q.template_id.as_str()))
            .or_else(|| self.training(quest_id).map(|t| t.template_id.as_str()))
    }
}
