//! WW-05 headless gameplay and replay, independent of HTTP, clocks and models.
//!
//! `TrustedFacts` is deliberately not deserializable. The integration layer must
//! construct it from server-issued encounters and persisted assessment, in the
//! same transaction as state/evidence/reward writes. Never build it from a JSON
//! request, guest import, director output or a client correctness assertion.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const RULES_VERSION: &str = "domain-1.0.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    InvalidContent(String),
    InvalidCommand,
    RevisionConflict,
    CommandIdReused,
    Locked,
    NotFound,
    AlreadyCompleted,
    UnverifiedEncounter,
    InsufficientInventory,
    Overflow,
    ReplayMismatch,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoryPack {
    schema_version: String,
    content_version: String,
    review_status: String,
    chapters: Vec<Chapter>,
    beats: Vec<Beat>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Chapter {
    id: String,
    entry_beat_id: String,
    finale_beat_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Beat {
    id: String,
    skill_id: String,
    template_id: String,
    chapter_id: String,
    requires_beat_ids: Vec<String>,
    required_creature_ids: Vec<String>,
    required_mastery_ids: Vec<String>,
    completion_flags: Vec<String>,
    rewards: Vec<Reward>,
    finale_steps: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HelpPack {
    schema_version: String,
    content_version: String,
    review_status: String,
    quests: Vec<HelpQuest>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HelpQuest {
    id: String,
    skill_id: String,
    template_id: String,
    creature_id: String,
    requires_beat_ids: Vec<String>,
    support_available: bool,
    supported_completion_befriends: bool,
    rewards: Vec<Reward>,
    repeat_rewards: Vec<Reward>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreaturePack {
    schema_version: String,
    content_version: String,
    review_status: String,
    creatures: Vec<Creature>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Creature {
    id: String,
    region_id: String,
    help_quest_id: String,
    discovery: Discovery,
    transformation: Transformation,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Discovery {
    eligible_after_beat: String,
    guaranteed_first: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transformation {
    id: String,
    minimum_friendship: u32,
    required_story_flag: String,
    reversible: bool,
    cosmetic_only: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Reward {
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
    fn id(&self) -> &str {
        match self {
            Self::ExplorerXp { id, .. }
            | Self::StoryFlag { id, .. }
            | Self::Feature { id, .. }
            | Self::Creature { id, .. }
            | Self::InventoryItem { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Content {
    version: String,
    chapters: BTreeMap<String, Chapter>,
    beats: BTreeMap<String, Beat>,
    help: BTreeMap<String, HelpQuest>,
    creatures: BTreeMap<String, Creature>,
    levels: Vec<u32>,
    max_friendship: u32,
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
    }
    pub fn quest_template_id(&self, quest_id: &str) -> Option<&str> {
        self.beats
            .get(quest_id)
            .map(|b| b.template_id.as_str())
            .or_else(|| self.help.get(quest_id).map(|q| q.template_id.as_str()))
    }
}

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
    revision: u64,
    sequence: u64,
    content_version: String,
    region: String,
    completed: BTreeSet<String>,
    pending: BTreeMap<String, String>,
    flags: BTreeSet<String>,
    features: BTreeSet<String>,
    inventory: BTreeMap<String, u32>,
    creatures: BTreeSet<String>,
    friendship: BTreeMap<String, u32>,
    helping_moments: BTreeSet<(String, String)>,
    transformed: BTreeSet<String>,
    team: Vec<String>,
    xp: u32,
    grants: BTreeSet<(String, String)>,
    discoveries: BTreeMap<String, String>,
    last_discovery: BTreeMap<String, String>,
    receipts: BTreeMap<String, Receipt>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Receipt {
    command: Command,
    events: Vec<Event>,
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
            grants: BTreeSet::new(),
            discoveries: BTreeMap::new(),
            last_discovery: BTreeMap::new(),
            receipts: BTreeMap::new(),
        }
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

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:./".contains(&b))
}
fn reward(
    state: &mut State,
    quest: &str,
    r: &Reward,
    outcomes: &mut Vec<Outcome>,
) -> Result<(), Rejection> {
    if !state.grants.insert((r.id().into(), quest.into())) {
        return Ok(());
    }
    match r {
        Reward::ExplorerXp { amount, .. } => {
            state.xp = state.xp.checked_add(*amount).ok_or(Rejection::Overflow)?
        }
        Reward::StoryFlag { value, .. } => {
            state.flags.insert(value.clone());
        }
        Reward::Feature { value, .. } => {
            state.features.insert(value.clone());
        }
        Reward::Creature { value, .. } => {
            state.creatures.insert(value.clone());
        }
        Reward::InventoryItem { value, amount, .. } => {
            let n = state.inventory.entry(value.clone()).or_default();
            *n = n.checked_add(*amount).ok_or(Rejection::Overflow)?;
        }
    }
    outcomes.push(Outcome::RewardGranted {
        reward_id: r.id().into(),
        quest_id: quest.into(),
    });
    Ok(())
}
fn quest<'a>(
    content: &'a Content,
    id: &str,
) -> Result<(&'a str, &'a [String], &'a [Reward]), Rejection> {
    if let Some(b) = content.beats.get(id) {
        return Ok((&b.chapter_id, &b.requires_beat_ids, &b.rewards));
    }
    let q = content.help.get(id).ok_or(Rejection::NotFound)?;
    let c = &content.creatures[&q.creature_id];
    Ok((&c.region_id, &q.requires_beat_ids, &q.rewards))
}
fn eligible(state: &State, content: &Content, id: &str) -> Result<(), Rejection> {
    let (region, requires, _) = quest(content, id)?;
    if region != state.region || requires.iter().any(|r| !state.completed.contains(r)) {
        return Err(Rejection::Locked);
    }
    if let Some(b) = content.beats.get(id)
        && (b.finale_steps.iter().any(|r| !state.completed.contains(r))
            || b.required_creature_ids
                .iter()
                .any(|r| !state.creatures.contains(r)))
    {
        return Err(Rejection::Locked);
    }
    Ok(())
}

/// Pure transition; errors leave the input untouched. Duplicate retries return
/// the original receipt before revision checking, even if the current revision
/// has advanced. Same ID with any different body is always a conflict.
pub fn apply(
    state: &State,
    command: &Command,
    content: &Content,
    context: &Context,
) -> Result<Applied, Rejection> {
    if !valid_id(&command.command_id) {
        return Err(Rejection::InvalidCommand);
    }
    if let Some(receipt) = state.receipts.get(&command.command_id) {
        return if receipt.command == *command {
            Ok(Applied {
                state: state.clone(),
                events: receipt.events.clone(),
                duplicate: true,
            })
        } else {
            Err(Rejection::CommandIdReused)
        };
    }
    if command.expected_revision != state.revision {
        return Err(Rejection::RevisionConflict);
    }
    if state.content_version != content.version {
        return Err(Rejection::InvalidContent("state/content mismatch".into()));
    }
    let mut next = state.clone();
    let mut outcomes = Vec::new();
    match &command.action {
        Action::BeginQuest {
            quest_id,
            encounter_id,
        } => {
            eligible(state, content, quest_id)?;
            if state.completed.contains(quest_id) {
                return Err(Rejection::AlreadyCompleted);
            }
            if !valid_id(encounter_id)
                || !context.facts.reachable_quests.contains(quest_id)
                || context.facts.issued_encounters.get(encounter_id) != Some(quest_id)
                || state.pending.contains_key(quest_id)
                || state.pending.values().any(|e| e == encounter_id)
            {
                return Err(Rejection::UnverifiedEncounter);
            }
            next.pending.insert(quest_id.clone(), encounter_id.clone());
            outcomes.push(Outcome::QuestStarted {
                quest_id: quest_id.clone(),
                encounter_id: encounter_id.clone(),
            });
        }
        Action::CompleteQuest {
            quest_id,
            encounter_id,
        } => {
            eligible(state, content, quest_id)?;
            if state.completed.contains(quest_id) {
                return Err(Rejection::AlreadyCompleted);
            }
            if state.pending.get(quest_id) != Some(encounter_id) {
                return Err(Rejection::UnverifiedEncounter);
            }
            let verified = context
                .facts
                .completed_encounters
                .get(encounter_id)
                .filter(|v| v.quest_id == *quest_id)
                .ok_or(Rejection::UnverifiedEncounter)?;
            next.completed.insert(quest_id.clone());
            next.pending.remove(quest_id);
            outcomes.push(Outcome::QuestCompleted {
                quest_id: quest_id.clone(),
                supported: verified.supported,
            });
            let (_, _, rewards) = quest(content, quest_id)?;
            for r in rewards {
                reward(&mut next, quest_id, r, &mut outcomes)?;
            }
            if let Some(b) = content.beats.get(quest_id) {
                next.flags.extend(b.completion_flags.clone());
            }
            if let Some(q) = content.help.get(quest_id) {
                next.friendship.insert(q.creature_id.clone(), 1);
                next.helping_moments
                    .insert((q.creature_id.clone(), quest_id.clone()));
                next.discoveries.retain(|_, v| v != &q.creature_id);
                outcomes.push(Outcome::FriendshipIncreased {
                    creature_id: q.creature_id.clone(),
                    points: 1,
                });
            }
        }
        Action::AbandonQuest {
            quest_id,
            encounter_id,
        } => {
            if state.pending.get(quest_id) != Some(encounter_id) {
                return Err(Rejection::UnverifiedEncounter);
            }
            next.pending.remove(quest_id);
            outcomes.push(Outcome::QuestAbandoned {
                quest_id: quest_id.clone(),
                encounter_id: encounter_id.clone(),
            });
        }
        Action::Travel { region_id } => {
            let chapter = content.chapters.get(region_id).ok_or(Rejection::NotFound)?;
            let entry = &content.beats[&chapter.entry_beat_id];
            if entry
                .requires_beat_ids
                .iter()
                .any(|id| !state.completed.contains(id))
            {
                return Err(Rejection::Locked);
            }
            next.region = region_id.clone();
            outcomes.push(Outcome::Travelled {
                region_id: region_id.clone(),
            });
        }
        Action::Discover => {
            let selected = if let Some(id) = state.discoveries.get(&state.region) {
                id.clone()
            } else {
                let candidates: Vec<_> = content
                    .creatures
                    .values()
                    .filter(|c| {
                        c.region_id == state.region
                            && state.completed.contains(&c.discovery.eligible_after_beat)
                    })
                    .collect();
                if candidates.is_empty() {
                    return Err(Rejection::Locked);
                }
                let missing: Vec<_> = candidates
                    .iter()
                    .filter(|c| !state.creatures.contains(&c.id))
                    .copied()
                    .collect();
                let pool = if missing.is_empty() {
                    let alternate: Vec<_> = candidates
                        .iter()
                        .filter(|c| state.last_discovery.get(&state.region) != Some(&c.id))
                        .copied()
                        .collect();
                    if alternate.is_empty() {
                        candidates
                    } else {
                        alternate
                    }
                } else {
                    missing
                };
                if let Some(first) = pool
                    .iter()
                    .find(|c| c.discovery.guaranteed_first && !state.creatures.contains(&c.id))
                {
                    first.id.clone()
                } else {
                    pool[context.recorded_random_choice as usize % pool.len()]
                        .id
                        .clone()
                }
            };
            next.discoveries
                .insert(state.region.clone(), selected.clone());
            next.last_discovery
                .insert(state.region.clone(), selected.clone());
            outcomes.push(Outcome::CreatureOffered {
                creature_id: selected,
            });
        }
        Action::SelectTeam { creature_ids } => {
            let slots = usize::from(state.features.contains("team_slot_1"))
                + usize::from(state.features.contains("team_slot_2"));
            if creature_ids.len() > slots
                || creature_ids.iter().collect::<BTreeSet<_>>().len() != creature_ids.len()
                || creature_ids.iter().any(|id| !state.creatures.contains(id))
            {
                return Err(Rejection::Locked);
            }
            next.team = creature_ids.clone();
            outcomes.push(Outcome::TeamSelected {
                creature_ids: creature_ids.clone(),
            });
        }
        Action::HelpFriend {
            creature_id,
            completed_quest_id,
        } => {
            let c = content
                .creatures
                .get(creature_id)
                .ok_or(Rejection::NotFound)?;
            let b = content
                .beats
                .get(completed_quest_id)
                .ok_or(Rejection::NotFound)?;
            let moment = (creature_id.clone(), completed_quest_id.clone());
            if !state.creatures.contains(creature_id)
                || !state.completed.contains(completed_quest_id)
                || b.chapter_id != c.region_id
                || state.region != c.region_id
                || state.helping_moments.contains(&moment)
            {
                return Err(Rejection::Locked);
            }
            let points = state
                .friendship(creature_id)
                .saturating_add(1)
                .min(content.max_friendship);
            next.friendship.insert(creature_id.clone(), points);
            next.helping_moments.insert(moment);
            outcomes.push(Outcome::FriendshipIncreased {
                creature_id: creature_id.clone(),
                points,
            });
        }
        Action::Transform {
            creature_id,
            enabled,
        } => {
            let c = content
                .creatures
                .get(creature_id)
                .ok_or(Rejection::NotFound)?;
            if !state.creatures.contains(creature_id)
                || (*enabled
                    && (state.friendship(creature_id) < c.transformation.minimum_friendship
                        || !state.flags.contains(&c.transformation.required_story_flag)))
            {
                return Err(Rejection::Locked);
            }
            if *enabled {
                next.transformed.insert(c.transformation.id.clone());
            } else {
                next.transformed.remove(&c.transformation.id);
            }
            outcomes.push(Outcome::TransformationSet {
                creature_id: creature_id.clone(),
                enabled: *enabled,
            });
        }
        Action::UseItem { item_id, amount } => {
            if *amount == 0 {
                return Err(Rejection::InvalidCommand);
            }
            let n = next
                .inventory
                .get_mut(item_id)
                .ok_or(Rejection::InsufficientInventory)?;
            *n = n
                .checked_sub(*amount)
                .ok_or(Rejection::InsufficientInventory)?;
            outcomes.push(Outcome::ItemUsed {
                item_id: item_id.clone(),
                amount: *amount,
            });
        }
    }
    next.revision = next.revision.checked_add(1).ok_or(Rejection::Overflow)?;
    let mut events = Vec::new();
    for outcome in outcomes {
        next.sequence = next.sequence.checked_add(1).ok_or(Rejection::Overflow)?;
        events.push(Event {
            sequence: next.sequence,
            revision: next.revision,
            command_id: command.command_id.clone(),
            rules_version: RULES_VERSION.into(),
            content_version: content.version.clone(),
            unix_millis: context.unix_millis,
            recorded_random_choice: context.recorded_random_choice,
            outcome,
        });
    }
    next.receipts.insert(
        command.command_id.clone(),
        Receipt {
            command: command.clone(),
            events: events.clone(),
        },
    );
    Ok(Applied {
        state: next,
        events,
        duplicate: false,
    })
}

/// Persist with the event stream. Context contains only recorded server facts;
/// replay never contacts an assessment service, random source, clock or model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayEntry {
    pub command: Command,
    pub context: Context,
    pub events: Vec<Event>,
}
pub fn replay(
    initial: &State,
    content: &Content,
    entries: &[ReplayEntry],
) -> Result<State, Rejection> {
    let mut state = initial.clone();
    for entry in entries {
        let applied = apply(&state, &entry.command, content, &entry.context)?;
        if applied.events != entry.events {
            return Err(Rejection::ReplayMismatch);
        }
        state = applied.state;
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    const STORY: &str = include_str!("../../../../content/story/adventure.v1.json");
    const HELP: &str = include_str!("../../../../content/story/help-quests.v1.json");
    const FRIENDS: &str = include_str!("../../../../content/creatures/friends.v1.json");
    const PROGRESSION: &str = include_str!("../../../../content/story/progression.v1.json");
    fn content() -> Content {
        Content::development(STORY, HELP, FRIENDS, PROGRESSION).unwrap()
    }
    fn context() -> Context {
        Context {
            unix_millis: 1793145600000,
            recorded_random_choice: 73,
            facts: TrustedFacts::default(),
        }
    }
    fn command(state: &State, action: Action) -> Command {
        Command {
            command_id: format!("command-{}", state.revision),
            expected_revision: state.revision,
            action,
        }
    }
    fn execute(
        state: &mut State,
        action: Action,
        content: &Content,
        ctx: Context,
        log: &mut Vec<ReplayEntry>,
    ) {
        let cmd = command(state, action);
        let applied = apply(state, &cmd, content, &ctx).unwrap();
        assert!(!applied.duplicate);
        assert_eq!(applied.state.revision, state.revision + 1);
        assert!(
            applied
                .events
                .windows(2)
                .all(|w| w[1].sequence == w[0].sequence + 1)
        );
        log.push(ReplayEntry {
            command: cmd,
            context: ctx,
            events: applied.events,
        });
        *state = applied.state;
    }
    fn finish(
        state: &mut State,
        id: &str,
        content: &Content,
        supported: bool,
        log: &mut Vec<ReplayEntry>,
    ) {
        let encounter = format!("encounter-{id}");
        let mut ctx = context();
        ctx.facts
            .issued_encounters
            .insert(encounter.clone(), id.into());
        ctx.facts.reachable_quests.insert(id.into());
        execute(
            state,
            Action::BeginQuest {
                quest_id: id.into(),
                encounter_id: encounter.clone(),
            },
            content,
            ctx,
            log,
        );
        let mut ctx = context();
        ctx.facts.completed_encounters.insert(
            encounter.clone(),
            VerifiedCompletion {
                quest_id: id.into(),
                supported,
            },
        );
        execute(
            state,
            Action::CompleteQuest {
                quest_id: id.into(),
                encounter_id: encounter,
            },
            content,
            ctx,
            log,
        );
    }
    fn chapter(
        state: &mut State,
        region: &str,
        c: &Content,
        supported: bool,
        log: &mut Vec<ReplayEntry>,
    ) {
        if state.region() != region {
            execute(
                state,
                Action::Travel {
                    region_id: region.into(),
                },
                c,
                context(),
                log,
            );
        }
        let ids = match region {
            "meadow" => vec![
                "meadow-welcome",
                "meadow-bridge",
                "meadow-picnic",
                "meadow-garden",
                "meadow-letter",
                "meadow-finale",
            ],
            "orchard" => vec![
                "orchard-welcome",
                "orchard-seeds",
                "orchard-ropes",
                "orchard-delivery",
                "orchard-instructions",
                "orchard-finale",
            ],
            _ => vec![
                "cove-welcome",
                "cove-observatory",
                "cove-shells",
                "cove-message",
                "cove-storybook",
                "cove-finale",
            ],
        };
        for id in ids {
            finish(state, id, c, supported, log);
        }
    }
    #[test]
    fn abandonment_is_replayable_without_rewards_and_can_restart() {
        let c = content();
        let initial = State::new(&c);
        let mut state = initial.clone();
        let mut log = Vec::new();
        let mut ctx = context();
        ctx.facts
            .issued_encounters
            .insert("enc-1".into(), "meadow-welcome".into());
        ctx.facts.reachable_quests.insert("meadow-welcome".into());
        execute(
            &mut state,
            Action::BeginQuest {
                quest_id: "meadow-welcome".into(),
                encounter_id: "enc-1".into(),
            },
            &c,
            ctx.clone(),
            &mut log,
        );
        let abandon = command(
            &state,
            Action::AbandonQuest {
                quest_id: "meadow-welcome".into(),
                encounter_id: "enc-1".into(),
            },
        );
        let applied = apply(&state, &abandon, &c, &context()).unwrap();
        assert!(matches!(
            &applied.events[0].outcome,
            Outcome::QuestAbandoned { .. }
        ));
        assert!(applied.state.grants.is_empty());
        assert!(applied.state.completed.is_empty());
        assert_eq!(applied.state.xp, 0);
        assert!(applied.state.pending.is_empty());
        assert!(
            apply(&applied.state, &abandon, &c, &context())
                .unwrap()
                .duplicate
        );
        execute(&mut state, abandon.action, &c, context(), &mut log);
        ctx.facts
            .issued_encounters
            .insert("enc-2".into(), "meadow-welcome".into());
        execute(
            &mut state,
            Action::BeginQuest {
                quest_id: "meadow-welcome".into(),
                encounter_id: "enc-2".into(),
            },
            &c,
            ctx,
            &mut log,
        );
        assert_eq!(replay(&initial, &c, &log).unwrap(), state);
    }
    #[test]
    fn golden_welcome_events_keep_rules_content_and_replay_compatible() {
        let c = content();
        let initial = State::new(&c);
        let mut state = initial.clone();
        let mut log = Vec::new();
        finish(&mut state, "meadow-welcome", &c, true, &mut log);
        let events: Vec<_> = log.iter().flat_map(|entry| entry.events.clone()).collect();
        let golden: Vec<Event> = serde_json::from_str(include_str!(
            "../tests/fixtures/meadow-welcome-events.v1.json"
        ))
        .unwrap();
        assert_eq!(events, golden);
        assert_eq!(replay(&initial, &c, &log).unwrap(), state);
    }
    #[test]
    fn small_domain_properties_hold_for_many_recorded_discovery_choices() {
        let c = content();
        let mut base = State::new(&c);
        chapter(&mut base, "meadow", &c, true, &mut Vec::new());
        for seed in 0..64 {
            let mut state = base.clone();
            let mut log = Vec::new();
            let mut choices = BTreeSet::new();
            for n in 0..4 {
                let mut ctx = context();
                ctx.recorded_random_choice = seed * 17 + n;
                let old_revision = state.revision();
                execute(&mut state, Action::Discover, &c, ctx, &mut log);
                assert_eq!(state.revision(), old_revision + 1);
                let creature = state.offered_creature().unwrap().to_string();
                assert!(choices.insert(creature.clone()));
                finish(
                    &mut state,
                    &format!("befriend-{creature}"),
                    &c,
                    n % 2 == 0,
                    &mut log,
                );
                assert!(
                    state
                        .friendship
                        .values()
                        .all(|&points| points <= c.max_friendship)
                );
                assert_eq!(state.grants.len(), 9 + (n as usize + 1) * 2);
            }
            assert_eq!(state.creatures.len(), 4);
            assert_eq!(state.xp(), 65);
            assert_eq!(replay(&base, &c, &log).unwrap(), state);
        }
    }
    #[test]
    fn authored_packs_load_but_unreviewed_content_cannot_activate() {
        assert_eq!(content().version(), "wonderwilds-story-1.0.0");
        assert_eq!(content().quest_skill_id("meadow-welcome"), Some("E01"));
        assert_eq!(
            content().quest_template_id("meadow-welcome"),
            Some("story_message")
        );
        assert_eq!(content().quest_skill_id("befriend-pip"), Some("M06"));
        assert_eq!(content().quest_skill_id("invented"), None);
        assert!(matches!(
            Content::released(STORY, HELP, FRIENDS, PROGRESSION),
            Err(Rejection::InvalidContent(_))
        ));
        let invalid = STORY.replace("\"schemaVersion\": \"1.0\"", "\"schemaVersion\": \"2.0\"");
        assert!(Content::development(&invalid, HELP, FRIENDS, PROGRESSION).is_err());
        let mut story: serde_json::Value = serde_json::from_str(STORY).unwrap();
        story["beats"][0]["requiresBeatIds"] = serde_json::json!(["cove-finale"]);
        assert!(Content::development(&story.to_string(), HELP, FRIENDS, PROGRESSION).is_err());
    }
    #[test]
    fn supported_journey_completes_every_finale_and_replays_exactly() {
        let c = content();
        let initial = State::new(&c);
        let mut state = initial.clone();
        let mut log = Vec::new();
        for region in ["meadow", "orchard", "cove"] {
            chapter(&mut state, region, &c, true, &mut log);
            assert!(state.flags().contains(&format!("{region}_light")));
        }
        assert_eq!(state.completed().len(), 18);
        assert_eq!(state.xp(), 135);
        assert_eq!(state.level(&c), 4);
        assert!(state.features.contains("ending"));
        assert_eq!(replay(&initial, &c, &log).unwrap(), state);
        assert!(
            log.iter()
                .flat_map(|e| &e.events)
                .filter_map(
                    |e| if let Outcome::QuestCompleted { supported, .. } = &e.outcome {
                        Some(supported)
                    } else {
                        None
                    }
                )
                .all(|s| *s)
        );
        let mut bad = log.clone();
        bad[1].events[0].unix_millis += 1;
        assert_eq!(replay(&initial, &c, &bad), Err(Rejection::ReplayMismatch));
    }
    #[test]
    fn idempotent_retries_return_original_events_and_reused_id_rejects_different_body() {
        let c = content();
        let mut s = State::new(&c);
        let mut log = Vec::new();
        finish(&mut s, "meadow-welcome", &c, false, &mut log);
        let receipt = log.last().unwrap();
        let xp = s.xp();
        let revision = s.revision();
        let retry = apply(&s, &receipt.command, &c, &context()).unwrap();
        assert!(retry.duplicate);
        assert_eq!(retry.events, receipt.events);
        assert_eq!(retry.state.xp(), xp);
        assert_eq!(retry.state.revision(), revision);
        let mut reused = receipt.command.clone();
        reused.action = Action::Discover;
        assert_eq!(
            apply(&s, &reused, &c, &context()),
            Err(Rejection::CommandIdReused)
        );
        let cmd = command(
            &s,
            Action::CompleteQuest {
                quest_id: "meadow-welcome".into(),
                encounter_id: "encounter-meadow-welcome".into(),
            },
        );
        assert_eq!(
            apply(&s, &cmd, &c, &context()),
            Err(Rejection::AlreadyCompleted)
        );
    }
    #[test]
    fn client_claims_and_cross_quest_assessments_cannot_grant_rewards() {
        let c = content();
        let mut s = State::new(&c);
        let mut log = Vec::new();
        let id = "meadow-welcome";
        let begin = command(
            &s,
            Action::BeginQuest {
                quest_id: id.into(),
                encounter_id: "fake".into(),
            },
        );
        assert_eq!(
            apply(&s, &begin, &c, &context()),
            Err(Rejection::UnverifiedEncounter)
        );
        let mut ctx = context();
        ctx.facts.issued_encounters.insert("real".into(), id.into());
        ctx.facts.reachable_quests.insert(id.into());
        execute(
            &mut s,
            Action::BeginQuest {
                quest_id: id.into(),
                encounter_id: "real".into(),
            },
            &c,
            ctx,
            &mut log,
        );
        let complete = command(
            &s,
            Action::CompleteQuest {
                quest_id: id.into(),
                encounter_id: "real".into(),
            },
        );
        assert_eq!(
            apply(&s, &complete, &c, &context()),
            Err(Rejection::UnverifiedEncounter)
        );
        let mut ctx = context();
        ctx.facts.completed_encounters.insert(
            "real".into(),
            VerifiedCompletion {
                quest_id: "meadow-bridge".into(),
                supported: false,
            },
        );
        assert_eq!(
            apply(&s, &complete, &c, &ctx),
            Err(Rejection::UnverifiedEncounter)
        );
        assert_eq!(s.xp(), 0);
        assert!(s.completed.is_empty());
        let injected = serde_json::json!({"commandId":"forged","expectedRevision":0,"action":{"type":"complete_quest","quest_id":id,"encounter_id":"real","correct":true,"supported":false}});
        assert!(serde_json::from_value::<Command>(injected).is_err());
    }
    #[test]
    fn stale_revision_unknown_ids_and_unreachable_interactions_are_rejected_atomically() {
        let c = content();
        let state = State::new(&c);
        for (action, expected) in [
            (
                Action::Travel {
                    region_id: "orchard".into(),
                },
                Rejection::Locked,
            ),
            (
                Action::Travel {
                    region_id: "invented".into(),
                },
                Rejection::NotFound,
            ),
            (
                Action::BeginQuest {
                    quest_id: "meadow-finale".into(),
                    encounter_id: "enc".into(),
                },
                Rejection::Locked,
            ),
            (
                Action::SelectTeam {
                    creature_ids: vec!["pip".into()],
                },
                Rejection::Locked,
            ),
        ] {
            assert_eq!(
                apply(&state, &command(&state, action), &c, &context()),
                Err(expected)
            );
        }
        let mut stale = command(&state, Action::Discover);
        stale.expected_revision = 1;
        assert_eq!(
            apply(&state, &stale, &c, &context()),
            Err(Rejection::RevisionConflict)
        );
        assert_eq!(state, State::new(&c));
    }
    #[test]
    fn independent_and_supported_completions_have_identical_story_rewards() {
        let c = content();
        let mut a = State::new(&c);
        let mut b = a.clone();
        chapter(&mut a, "meadow", &c, false, &mut Vec::new());
        chapter(&mut b, "meadow", &c, true, &mut Vec::new());
        assert_eq!(a.xp(), b.xp());
        assert_eq!(a.flags(), b.flags());
        assert_eq!(a.completed(), b.completed());
        assert_eq!(a.grants, b.grants);
    }
    #[test]
    fn collection_friendship_cosmetics_and_team_have_independent_gates() {
        let c = content();
        let mut s = State::new(&c);
        let mut log = Vec::new();
        chapter(&mut s, "meadow", &c, true, &mut log);
        finish(&mut s, "befriend-pip", &c, true, &mut log);
        assert!(s.creatures().contains("pip"));
        assert_eq!(s.friendship("pip"), 1);
        let transform = Action::Transform {
            creature_id: "pip".into(),
            enabled: true,
        };
        assert_eq!(
            apply(&s, &command(&s, transform.clone()), &c, &context()),
            Err(Rejection::Locked)
        );
        for quest in ["meadow-bridge", "meadow-garden"] {
            execute(
                &mut s,
                Action::HelpFriend {
                    creature_id: "pip".into(),
                    completed_quest_id: quest.into(),
                },
                &c,
                context(),
                &mut log,
            );
        }
        assert_eq!(s.friendship("pip"), 3);
        execute(&mut s, transform, &c, context(), &mut log);
        assert!(s.transformed().contains("pip-glow"));
        let repeated = command(
            &s,
            Action::HelpFriend {
                creature_id: "pip".into(),
                completed_quest_id: "meadow-bridge".into(),
            },
        );
        assert_eq!(apply(&s, &repeated, &c, &context()), Err(Rejection::Locked));
        execute(
            &mut s,
            Action::SelectTeam {
                creature_ids: vec!["pip".into()],
            },
            &c,
            context(),
            &mut log,
        );
        assert_eq!(s.team(), ["pip"]);
        let xp = s.xp();
        execute(
            &mut s,
            Action::Transform {
                creature_id: "pip".into(),
                enabled: false,
            },
            &c,
            context(),
            &mut log,
        );
        assert!(s.transformed().is_empty());
        assert_eq!(s.xp(), xp);
    }
    #[test]
    fn discovery_is_stable_when_declined_and_missing_friends_are_bounded() {
        let c = content();
        let mut s = State::new(&c);
        let mut log = Vec::new();
        chapter(&mut s, "meadow", &c, true, &mut log);
        let mut seen = BTreeSet::new();
        for index in 0..4 {
            execute(&mut s, Action::Discover, &c, context(), &mut log);
            let id = s.offered_creature().unwrap().to_string();
            if index == 0 {
                assert_eq!(id, "pip");
            }
            let mut changed = context();
            changed.recorded_random_choice = 999;
            execute(&mut s, Action::Discover, &c, changed, &mut log);
            assert_eq!(s.offered_creature(), Some(id.as_str()));
            assert!(seen.insert(id.clone()));
            finish(&mut s, &format!("befriend-{id}"), &c, true, &mut log);
        }
        assert_eq!(seen.len(), 4);
        assert_eq!(s.creatures().len(), 4);
        execute(&mut s, Action::Discover, &c, context(), &mut log);
    }
    #[test]
    fn inventory_subtraction_is_checked_and_failed_reward_batches_rollback() {
        let mut c = content();
        c.beats
            .get_mut("meadow-welcome")
            .unwrap()
            .rewards
            .push(Reward::InventoryItem {
                id: "planks".into(),
                value: "wood".into(),
                amount: 3,
            });
        let mut s = State::new(&c);
        finish(&mut s, "meadow-welcome", &c, false, &mut Vec::new());
        for amount in [0, 4, u32::MAX] {
            let before = s.clone();
            assert!(
                apply(
                    &s,
                    &command(
                        &s,
                        Action::UseItem {
                            item_id: "wood".into(),
                            amount
                        }
                    ),
                    &c,
                    &context()
                )
                .is_err()
            );
            assert_eq!(s, before);
        }
        execute(
            &mut s,
            Action::UseItem {
                item_id: "wood".into(),
                amount: 3,
            },
            &c,
            context(),
            &mut Vec::new(),
        );
        assert_eq!(s.inventory()["wood"], 0);
        let mut overflow = State::new(&c);
        overflow.xp = u32::MAX;
        let mut ctx = context();
        ctx.facts
            .issued_encounters
            .insert("overflow".into(), "meadow-welcome".into());
        ctx.facts.reachable_quests.insert("meadow-welcome".into());
        execute(
            &mut overflow,
            Action::BeginQuest {
                quest_id: "meadow-welcome".into(),
                encounter_id: "overflow".into(),
            },
            &c,
            ctx,
            &mut Vec::new(),
        );
        let before = overflow.clone();
        let mut ctx = context();
        ctx.facts.completed_encounters.insert(
            "overflow".into(),
            VerifiedCompletion {
                quest_id: "meadow-welcome".into(),
                supported: false,
            },
        );
        assert_eq!(
            apply(
                &overflow,
                &command(
                    &overflow,
                    Action::CompleteQuest {
                        quest_id: "meadow-welcome".into(),
                        encounter_id: "overflow".into()
                    }
                ),
                &c,
                &ctx
            ),
            Err(Rejection::Overflow)
        );
        assert_eq!(overflow, before);
    }
}
