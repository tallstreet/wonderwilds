//! Pure authored story sandbox. Independent of the assessed adventure and providers.
//!
//! Coarse travel validates a graph transition; it does not prove browser movement.
//! Checkpoints are bounded local developer records, not authenticated server saves.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod broader;
pub mod exploration;
pub mod items;
mod missions;
use broader::BroaderProgress;
pub use missions::Mission;

pub const CONTENT_VERSION: &str = "pip-stories-3";
const SERIES_V2: &str = "pip-stories-2";
const LEGACY_CONTENT_VERSION: &str = "pip-bridge-1";
pub const MAX_COMMANDS: usize = 256;
const MAX_CHECKPOINT_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    BridgeBank,
    Workshop,
    FarBank,
    Cove,
    FernHollow,
    MossyClearing,
    OwlGrove,
    AcornTrail,
}
impl Location {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BridgeBank => "bridge_bank",
            Self::Workshop => "workshop",
            Self::FarBank => "far_bank",
            Self::Cove => "cove",
            Self::FernHollow => "fern_hollow",
            Self::MossyClearing => "mossy_clearing",
            Self::OwlGrove => "owl_grove",
            Self::AcornTrail => "acorn_trail",
        }
    }
    fn parse(value: &str) -> Result<Self, Rejection> {
        match value {
            "bridge_bank" => Ok(Self::BridgeBank),
            "workshop" => Ok(Self::Workshop),
            "far_bank" => Ok(Self::FarBank),
            "cove" => Ok(Self::Cove),
            "fern_hollow" => Ok(Self::FernHollow),
            "mossy_clearing" => Ok(Self::MossyClearing),
            "owl_grove" => Ok(Self::OwlGrove),
            "acorn_trail" => Ok(Self::AcornTrail),
            _ => Err(Rejection::InvalidAction),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Offered,
    Active,
    Abandoned,
    Completed,
}
impl MissionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Offered => "offered",
            Self::Active => "active",
            Self::Abandoned => "abandoned",
            Self::Completed => "completed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Action {
    AcceptMission,
    // Internal, server-validated enrichment of a public travel choice. Never offered
    // as a browser action; saved in the durable command log for exact replay.
    GeneratedTravel {
        destination_id: String,
        story: exploration::GeneratedStory,
    },
    Inspect {
        object_id: String,
    },
    Talk {
        character_id: String,
        topic_id: String,
    },
    Choose {
        choice_id: String,
        option_id: String,
    },
    Invite {
        character_id: String,
    },
    CancelMission,
    Collect {
        source_id: String,
        item_id: String,
        quantity: u32,
    },
    Travel {
        destination_id: String,
    },
    Exchange {
        character_id: String,
        recipe_id: String,
    },
    Assemble {
        object_id: String,
        recipe_id: String,
    },
    Accompany {
        character_id: String,
        destination_id: String,
    },
    Acknowledge {
        scene_id: String,
        entry_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Command {
    pub command_id: String,
    pub expected_revision: u64,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
    pub command_id: String,
    pub revision: u64,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    InvalidAction,
    InvalidCommand,
    RevisionConflict,
    CommandIdReused,
    NotAvailable,
    InsufficientInventory,
    LimitReached,
    InvalidCheckpoint,
}

/// Trusted harness initialization, never a game command or browser import.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialConditions {
    pub wood: u32,
    pub planks: u32,
    pub bridge_repaired: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub condition_id: String,
    pub label: String,
    pub complete: bool,
}
impl Milestone {
    pub fn as_str(&self) -> &str {
        &self.condition_id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub revision: u64,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentationEntry {
    pub entry_id: String,
    pub speaker_id: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Presentation {
    pub scene_id: String,
    pub entries: Vec<PresentationEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(default)]
    durable: bool,
    initial: InitialConditions,
    revision: u64,
    location: Location,
    wood: u32,
    planks: u32,
    sources: BTreeMap<String, u32>,
    bridge_repaired: bool,
    crossed_with_pip: bool,
    pip_location: Location,
    mission_status: MissionStatus,
    milestones: Vec<Milestone>,
    met_woodcutter: bool,
    mission: Mission,
    completed_missions: Vec<Mission>,
    berries: u32,
    picnic_basket: u32,
    glow_pebbles: u32,
    lanterns: u32,
    picnic_ready: bool,
    picnic_shared: bool,
    near_lit: bool,
    far_lit: bool,
    broader: BroaderProgress,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exploration: Vec<exploration::Quest>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    finds: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    neighbourhood_projects: BTreeMap<String, exploration::Project>,
    events: Vec<Event>,
    history: Vec<Command>,
    receipts: BTreeMap<String, Receipt>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self::with_initial(InitialConditions::default()).expect("valid authored initial state")
    }
    pub fn with_initial(initial: InitialConditions) -> Result<Self, Rejection> {
        if initial.wood > 3 || initial.planks > 3 {
            return Err(Rejection::InvalidCheckpoint);
        }
        Ok(Self {
            durable: false,
            initial,
            revision: 0,
            location: Location::BridgeBank,
            wood: initial.wood,
            planks: initial.planks,
            sources: BTreeMap::from([
                ("fallen_wood".into(), 3),
                ("spare_wood".into(), 3),
                ("berry_bush".into(), 3),
                ("glow_stones".into(), 3),
            ]),
            bridge_repaired: initial.bridge_repaired,
            crossed_with_pip: false,
            pip_location: Location::BridgeBank,
            mission_status: MissionStatus::Offered,
            milestones: Vec::new(),
            met_woodcutter: false,
            mission: Mission::Bridge,
            completed_missions: Vec::new(),
            berries: 0,
            picnic_basket: 0,
            glow_pebbles: 0,
            lanterns: 0,
            picnic_ready: false,
            picnic_shared: false,
            near_lit: false,
            far_lit: false,
            broader: BroaderProgress::default(),
            exploration: Vec::new(),
            finds: BTreeMap::new(),
            neighbourhood_projects: BTreeMap::new(),
            events: Vec::new(),
            history: Vec::new(),
            receipts: BTreeMap::new(),
        })
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn location(&self) -> Location {
        self.location
    }
    pub fn wood(&self) -> u32 {
        self.wood
    }
    pub fn planks(&self) -> u32 {
        self.planks
    }
    pub fn source_remaining(&self, id: &str) -> Option<u32> {
        self.sources.get(id).copied()
    }
    pub fn bridge_repaired(&self) -> bool {
        self.bridge_repaired
    }
    pub fn crossed_with_pip(&self) -> bool {
        self.crossed_with_pip
    }
    pub fn pip_location(&self) -> Location {
        self.pip_location
    }
    pub fn mission_status(&self) -> MissionStatus {
        self.mission_status
    }
    pub fn milestones(&self) -> &[Milestone] {
        &self.milestones
    }
    pub fn met_woodcutter(&self) -> bool {
        self.met_woodcutter
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn mission(&self) -> Mission {
        self.mission
    }
    pub fn mission_id(&self) -> &'static str {
        self.mission.as_str()
    }
    pub fn next_mission(&self) -> Option<Mission> {
        if self.mission_status == MissionStatus::Completed {
            self.mission.next()
        } else {
            None
        }
    }
    pub fn completed_missions(&self) -> &[Mission] {
        &self.completed_missions
    }
    pub fn berries(&self) -> u32 {
        self.berries
    }
    pub fn picnic_basket(&self) -> u32 {
        self.picnic_basket
    }
    pub fn glow_pebbles(&self) -> u32 {
        self.glow_pebbles
    }
    pub fn lanterns(&self) -> u32 {
        self.lanterns
    }
    pub fn picnic_ready(&self) -> bool {
        self.picnic_ready
    }
    pub fn picnic_shared(&self) -> bool {
        self.picnic_shared
    }
    pub fn near_lit(&self) -> bool {
        self.near_lit
    }
    pub fn far_lit(&self) -> bool {
        self.far_lit
    }
    pub fn reserved_quantity(&self, item: &str) -> u32 {
        self.exploration
            .iter()
            .map(|q| q.supplies.get(item).copied().unwrap_or(0))
            .sum()
    }
    pub fn quantity(&self, item: &str) -> u32 {
        self.reserved_quantity(item)
            + match item {
                "wood" => self.wood,
                "planks" => self.planks,
                "berries" => self.berries,
                "picnic_basket" => self.picnic_basket,
                "glow_pebbles" => self.glow_pebbles,
                "lanterns" => self.lanterns,
                "wind_instrument" => u32::from(
                    self.instrument_home() == Some(Location::BridgeBank)
                        && !self.has_story_flag("instrument_placed"),
                ),
                _ => self.finds.get(item).copied().unwrap_or(0),
            }
    }
    pub fn inventory(&self) -> BTreeMap<String, u32> {
        [
            "wood",
            "planks",
            "berries",
            "picnic_basket",
            "glow_pebbles",
            "lanterns",
            "wind_instrument",
        ]
        .into_iter()
        .chain(items::FINDS.iter().map(|item| item.id))
        .filter_map(|id| {
            let quantity = self.quantity(id);
            (quantity > 0).then(|| (id.to_owned(), quantity))
        })
        .collect()
    }

    /// Commit a validated action to a private candidate. Every failure is atomic.
    pub fn apply(&mut self, command: &Command) -> Result<Receipt, Rejection> {
        if !valid_id(&command.command_id) {
            return Err(Rejection::InvalidCommand);
        }
        if let Some(previous) = self
            .history
            .iter()
            .find(|c| c.command_id == command.command_id)
        {
            return if previous == command {
                Ok(self.receipts[&command.command_id].clone())
            } else {
                Err(Rejection::CommandIdReused)
            };
        }
        if command.expected_revision != self.revision {
            return Err(Rejection::RevisionConflict);
        }
        if self.command_budget_exhausted() {
            return Err(Rejection::LimitReached);
        }
        let mut candidate = self.clone();
        if candidate.durable {
            candidate.history.clear();
            candidate.receipts.clear();
        }
        let description = candidate.transition(&command.action)?;
        candidate.evaluate_milestones();
        candidate.revision = candidate
            .revision
            .checked_add(1)
            .ok_or(Rejection::LimitReached)?;
        let receipt = Receipt {
            command_id: command.command_id.clone(),
            revision: candidate.revision,
            description,
        };
        candidate.events.push(Event {
            revision: candidate.revision,
            description: receipt.description.clone(),
        });
        if candidate.durable && candidate.events.len() > 8 {
            candidate.events.remove(0);
        }
        candidate.history.push(command.clone());
        candidate
            .receipts
            .insert(command.command_id.clone(), receipt.clone());
        *self = candidate;
        Ok(receipt)
    }

    fn transition(&mut self, action: &Action) -> Result<String, Rejection> {
        if let Some(result) = self.exploration_transition(action) {
            return result;
        }
        match action {
            Action::GeneratedTravel {
                destination_id,
                story,
            } => {
                let description = self.transition(&Action::Travel {
                    destination_id: destination_id.clone(),
                })?;
                self.install_generated_story(story)?;
                Ok(description)
            }
            Action::AcceptMission => {
                if self.mission_status == MissionStatus::Completed {
                    let next = self.next_mission().ok_or(Rejection::NotAvailable)?;
                    if self.location != self.pip_location {
                        return Err(Rejection::NotAvailable);
                    }
                    self.mission = next;
                    self.milestones.clear();
                } else if !matches!(
                    self.mission_status,
                    MissionStatus::Offered | MissionStatus::Abandoned
                ) {
                    return Err(Rejection::NotAvailable);
                }
                if self.milestones.is_empty() {
                    self.initialize_milestones();
                }
                self.mission_status = MissionStatus::Active;
                Ok(match self.mission {
                    Mission::Bridge => {
                        "The player accepted the mission to cross the bridge with Pip."
                    }
                    Mission::Picnic => {
                        "The player accepted the mission to prepare and share a picnic with Pip."
                    }
                    Mission::WelcomeLights => {
                        "The player accepted the mission to light both banks with welcome lanterns."
                    }
                    Mission::HummingHollow => {
                        "The player accepted the mission to discover the humming hollow."
                    }
                    Mission::CosyCorner => {
                        "The player accepted the mission to make a cosy corner for Pip."
                    }
                    Mission::BramblesVisit => {
                        "The player accepted the mission to help Bramble visit the bridge bank."
                    }
                    Mission::MoonlightGathering => {
                        "The player accepted the mission to prepare a moonlight gathering."
                    }
                }
                .into())
            }
            Action::CancelMission => {
                self.require_active()?;
                self.mission_status = MissionStatus::Abandoned;
                Ok(if matches!(self.mission, Mission::Bridge | Mission::Picnic | Mission::WelcomeLights) {
                    "The player paused Pip's mission; items and world changes were preserved."
                } else {
                    "The player paused the mission; discoveries, choices and preparations were preserved."
                }.into())
            }
            Action::Travel { destination_id } => {
                let destination = Location::parse(destination_id)?;
                if !self.route_open(destination) {
                    return Err(Rejection::NotAvailable);
                }
                self.location = destination;
                Ok(format!("The player arrived at {}.", destination.as_str()))
            }
            Action::Collect {
                source_id,
                item_id,
                quantity,
            } => {
                if let Some(find) = items::FINDS.iter().find(|find| find.source == source_id) {
                    if item_id != find.id || *quantity != 1 {
                        return Err(Rejection::InvalidAction);
                    }
                    if self.location != find.location || self.quantity(find.id) > 0 {
                        return Err(Rejection::NotAvailable);
                    }
                    self.finds.insert(find.id.into(), 1);
                    return Ok(format!("Picked up {} and added it to the bag.", find.name));
                }
                self.require_active()?;
                let (source_location, item, mission) = match source_id.as_str() {
                    "fallen_wood" => (Location::BridgeBank, "wood", Mission::Bridge),
                    "spare_wood" => (Location::Workshop, "wood", Mission::Bridge),
                    "berry_bush" => (Location::FarBank, "berries", Mission::Picnic),
                    "glow_stones" => (Location::FarBank, "glow_pebbles", Mission::WelcomeLights),
                    _ => return Err(Rejection::InvalidAction),
                };
                if item_id != item || !(1..=3).contains(quantity) {
                    return Err(Rejection::InvalidAction);
                }
                if self.location != source_location
                    || self.mission != mission
                    || (mission == Mission::Bridge && self.bridge_repaired)
                {
                    return Err(Rejection::NotAvailable);
                }
                let supply = self
                    .sources
                    .get_mut(source_id)
                    .ok_or(Rejection::InvalidAction)?;
                if *supply < *quantity {
                    return Err(Rejection::InsufficientInventory);
                }
                *supply -= quantity;
                match mission {
                    Mission::Bridge => self.wood += quantity,
                    Mission::Picnic => self.berries += quantity,
                    Mission::WelcomeLights => self.glow_pebbles += quantity,
                    _ => return Err(Rejection::InvalidAction),
                }
                Ok(format!(
                    "The player collected {quantity} {item} from {source_id}."
                ))
            }
            Action::Exchange {
                character_id,
                recipe_id,
            } => {
                self.require_active()?;
                if character_id != "woodcutter" {
                    return Err(Rejection::InvalidAction);
                }
                if self.location != Location::Workshop {
                    return Err(Rejection::NotAvailable);
                }
                match recipe_id.as_str() {
                    "wood_to_planks" => {
                        if self.mission != Mission::Bridge || self.bridge_repaired {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.wood < 3 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.wood -= 3;
                        self.planks += 3;
                        Ok("The woodcutter exchanged three wood for three planks.".into())
                    }
                    "berries_to_picnic" => {
                        if self.mission != Mission::Picnic {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.berries < 3 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.berries -= 3;
                        self.picnic_basket += 1;
                        Ok("The woodcutter packed three berries into a picnic basket.".into())
                    }
                    "glow_to_lanterns" => {
                        if self.mission != Mission::WelcomeLights {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.glow_pebbles < 3 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.glow_pebbles -= 3;
                        self.lanterns += 3;
                        Ok(
                            "The woodcutter made three welcome lanterns from three glow pebbles."
                                .into(),
                        )
                    }
                    _ => Err(Rejection::InvalidAction),
                }
            }
            Action::Assemble {
                object_id,
                recipe_id,
            } => {
                self.require_active()?;
                match (object_id.as_str(), recipe_id.as_str()) {
                    ("bridge", "repair_bridge") => {
                        if self.mission != Mission::Bridge
                            || self.location != Location::BridgeBank
                            || self.bridge_repaired
                        {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.planks < 3 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.planks -= 3;
                        self.bridge_repaired = true;
                        Ok("The player used three planks to repair the bridge.".into())
                    }
                    ("picnic_table", "prepare_picnic") => {
                        if self.mission != Mission::Picnic
                            || self.location != Location::BridgeBank
                            || self.picnic_ready
                        {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.picnic_basket < 1 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.picnic_basket -= 1;
                        self.picnic_ready = true;
                        Ok("The player unpacked the basket and prepared a picnic beside the bridge.".into())
                    }
                    ("near_lanterns", "light_near_bank") => {
                        if self.mission != Mission::WelcomeLights
                            || self.location != Location::BridgeBank
                            || self.near_lit
                        {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.lanterns < 2 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.lanterns -= 2;
                        self.near_lit = true;
                        Ok("The player placed two welcome lanterns on the near bank.".into())
                    }
                    ("far_lantern", "light_far_bank") => {
                        if self.mission != Mission::WelcomeLights
                            || self.location != Location::FarBank
                            || self.far_lit
                        {
                            return Err(Rejection::NotAvailable);
                        }
                        if self.lanterns < 1 {
                            return Err(Rejection::InsufficientInventory);
                        }
                        self.lanterns -= 1;
                        self.far_lit = true;
                        Ok("The player placed a welcome lantern on the far bank.".into())
                    }
                    _ => self.broader_transition(action),
                }
            }
            Action::Accompany {
                character_id,
                destination_id,
            } => {
                self.require_active()?;
                if character_id == "bramble" {
                    return self.broader_transition(action);
                }
                if character_id != "pip"
                    || !matches!(destination_id.as_str(), "bridge_bank" | "far_bank")
                {
                    return Err(Rejection::InvalidAction);
                }
                if !self.bridge_repaired || self.pip_location != self.location {
                    return Err(Rejection::NotAvailable);
                }
                match self.mission {
                    Mission::Bridge
                        if self.location == Location::BridgeBank
                            && destination_id == "far_bank" =>
                    {
                        self.location = Location::FarBank;
                        self.pip_location = Location::FarBank;
                        self.crossed_with_pip = true;
                        Ok("The player and Pip crossed the repaired bridge together.".into())
                    }
                    Mission::Picnic
                        if self.location == Location::FarBank
                            && destination_id == "bridge_bank"
                            && self.picnic_ready =>
                    {
                        self.location = Location::BridgeBank;
                        self.pip_location = Location::BridgeBank;
                        self.picnic_shared = true;
                        Ok("The player brought Pip back to the bridge bank and shared the prepared picnic.".into())
                    }
                    _ => Err(Rejection::NotAvailable),
                }
            }
            Action::Inspect { .. }
            | Action::Talk { .. }
            | Action::Choose { .. }
            | Action::Invite { .. } => self.broader_transition(action),
            Action::Acknowledge { scene_id, entry_id } => {
                let scene = self.presentation();
                if scene.scene_id != *scene_id {
                    return Err(Rejection::NotAvailable);
                }
                let entry = scene
                    .entries
                    .iter()
                    .find(|e| e.entry_id == *entry_id)
                    .ok_or(Rejection::InvalidAction)?;
                if entry.speaker_id == "woodcutter" {
                    self.met_woodcutter = true;
                }
                Ok(if entry.speaker_id.is_empty() {
                    "The game presented the authored scene narration.".into()
                } else {
                    format!(
                        "The game presented {}'s authored greeting.",
                        entry.speaker_id
                    )
                })
            }
        }
    }

    fn require_active(&self) -> Result<(), Rejection> {
        if self.mission_status == MissionStatus::Active {
            Ok(())
        } else {
            Err(Rejection::NotAvailable)
        }
    }
    fn route_open(&self, to: Location) -> bool {
        exploration::connected(self.location, to)
            || matches!(
                (self.location, to),
                (Location::BridgeBank, Location::Workshop)
                    | (Location::Workshop, Location::BridgeBank)
            )
            || self.bridge_repaired
                && matches!(
                    (self.location, to),
                    (Location::BridgeBank, Location::FarBank)
                        | (Location::FarBank, Location::BridgeBank)
                )
    }
    fn initialize_milestones(&mut self) {
        let mut definitions = Vec::new();
        match self.mission {
            Mission::Picnic => definitions.extend([
                ("berries_ready", "Gather berries for the picnic."),
                (
                    "picnic_basket_ready",
                    "Ask the woodcutter to pack the picnic.",
                ),
                ("picnic_prepared", "Prepare the picnic beside the bridge."),
                ("picnic_shared", "Bring Pip back and share the picnic."),
            ]),
            Mission::WelcomeLights => definitions.extend([
                ("glow_pebbles_ready", "Gather glow pebbles on the far bank."),
                ("lanterns_ready", "Ask the woodcutter to make lanterns."),
                ("near_bank_lit", "Place two lanterns on the near bank."),
                ("far_bank_lit", "Place a lantern on the far bank."),
            ]),
            Mission::HummingHollow
            | Mission::CosyCorner
            | Mission::BramblesVisit
            | Mission::MoonlightGathering => definitions.extend(self.broader_milestones()),
            Mission::Bridge => {
                if !self.bridge_repaired {
                    if self.planks < 3 {
                        definitions.push(("wood_ready", "Find suitable wood."));
                    }
                    definitions.push(("planks_ready", "Get the planks ready."));
                    definitions.push(("planks_at_bridge", "Bring the planks to the bridge."));
                }
                definitions.push(("bridge_repaired", "Repair the bridge."));
                definitions.push(("crossed_with_pip", "Cross with Pip."));
            }
        }
        self.milestones = definitions
            .into_iter()
            .map(|(id, label)| Milestone {
                condition_id: id.into(),
                label: label.into(),
                complete: false,
            })
            .collect();
    }
    fn evaluate_milestones(&mut self) {
        if self.mission_status != MissionStatus::Active {
            return;
        }
        for milestone in &mut self.milestones {
            milestone.complete |= match milestone.condition_id.as_str() {
                "wood_ready" => self.wood >= 3,
                "planks_ready" => self.planks >= 3,
                "planks_at_bridge" => self.planks >= 3 && self.location == Location::BridgeBank,
                "bridge_repaired" => self.bridge_repaired,
                "crossed_with_pip" => self.crossed_with_pip,
                "berries_ready" => self.berries >= 3,
                "picnic_basket_ready" => self.picnic_basket >= 1,
                "picnic_prepared" => self.picnic_ready,
                "picnic_shared" => self.picnic_shared,
                "glow_pebbles_ready" => self.glow_pebbles >= 3,
                "lanterns_ready" => self.lanterns >= 3,
                "near_bank_lit" => self.near_lit,
                "far_bank_lit" => self.far_lit,
                "instrument_home_chosen" => self.broader.instrument_home.is_some(),
                "corner_style_chosen" => self.broader.corner_style.is_some(),
                "gathering_place_chosen" => self.broader.gathering_place.is_some(),
                id => self.broader.flags.contains(id),
            };
        }
        let outcome = match self.mission {
            Mission::Bridge => self.bridge_repaired && self.crossed_with_pip,
            Mission::Picnic => self.picnic_ready && self.picnic_shared,
            Mission::WelcomeLights => self.near_lit && self.far_lit,
            Mission::HummingHollow => self.has_story_flag("hollow_shared"),
            Mission::CosyCorner => self.has_story_flag("corner_shared"),
            Mission::BramblesVisit => self.has_story_flag("bramble_visit_shared"),
            Mission::MoonlightGathering => self.has_story_flag("gathering_shared"),
        };
        if outcome {
            self.mission_status = MissionStatus::Completed;
            if !self.completed_missions.contains(&self.mission) {
                self.completed_missions.push(self.mission);
            }
        }
    }

    /// Canonical authored lines. Acknowledgements are bound to this revision and entry.
    /// Calling this read-only method does not establish a meeting or any progress.
    pub fn presentation(&self) -> Presentation {
        let (speaker, text) =
            if let Some(q) = self.current_neighbour().filter(|q| q.story.is_some()) {
                let story = &q.story.as_ref().unwrap().content;
                (
                    q.character.as_str(),
                    if q.status == "completed" {
                        story.payoff.as_str()
                    } else if q.status == "open" && !q.inspected {
                        story.steps[q.step as usize].label.as_str()
                    } else if q.inspected {
                        "Thank you for exploring! Come and share what you found."
                    } else {
                        story.greeting.as_str()
                    },
                )
            } else if let Some(place) = exploration::place(self.location) {
                (
                    place.character,
                    if self
                        .exploration
                        .iter()
                        .any(|q| q.character == place.character && q.status == "completed")
                    {
                        place.payoff
                    } else {
                        place.premise
                    },
                )
            } else if matches!(
                self.mission,
                Mission::HummingHollow
                    | Mission::CosyCorner
                    | Mission::BramblesVisit
                    | Mission::MoonlightGathering
            ) {
                self.broader_presentation()
            } else if self.mission != Mission::Bridge {
                self.series_presentation()
            } else {
                match self.location {
                    Location::Workshop if self.met_woodcutter => {
                        ("woodcutter", "I can help shape wood into sturdy planks.")
                    }
                    Location::Workshop => (
                        "woodcutter",
                        "Hello! I am the woodcutter. I can shape wood into sturdy planks.",
                    ),
                    Location::BridgeBank if self.pip_location != self.location => (
                        "",
                        "The repaired bridge leads to the far bank, where Pip is waiting.",
                    ),
                    Location::BridgeBank if self.bridge_repaired => {
                        ("pip", "The bridge is ready! We can cross together.")
                    }
                    Location::BridgeBank => (
                        "pip",
                        "Shall we repair this bridge and explore the far bank together?",
                    ),
                    Location::FarBank if self.crossed_with_pip => {
                        ("pip", "We made it! Thank you for crossing with me.")
                    }
                    _ => (
                        "",
                        "The player reached the far bank. Pip is waiting across the bridge.",
                    ),
                }
            };
        Presentation {
            scene_id: format!("pip-bridge-scene-{}", self.revision),
            entries: vec![PresentationEntry {
                entry_id: "greeting".into(),
                speaker_id: speaker.into(),
                text: text.into(),
            }],
        }
    }

    fn series_presentation(&self) -> (&'static str, &'static str) {
        if self.location == Location::Workshop {
            if !self.met_woodcutter {
                return (
                    "woodcutter",
                    "Hello! I am the woodcutter. I help neighbours prepare supplies and craftwork.",
                );
            }
            return match self.mission {
                Mission::Picnic if self.picnic_shared => {
                    ("woodcutter", "I'm glad you and Pip could share the picnic.")
                }
                Mission::Picnic if self.picnic_basket > 0 || self.picnic_ready => (
                    "woodcutter",
                    "Your picnic basket is packed. Pip is waiting on the far bank.",
                ),
                Mission::Picnic => (
                    "woodcutter",
                    "Bring me three berries, and I can pack a picnic basket for you and Pip.",
                ),
                _ if self.near_lit && self.far_lit => (
                    "woodcutter",
                    "Both banks are glowing with the lanterns we made.",
                ),
                _ if self.lanterns > 0 || self.near_lit || self.far_lit => (
                    "woodcutter",
                    "The lanterns are ready to place along the stream.",
                ),
                _ => (
                    "woodcutter",
                    "Bring me three glow pebbles, and I can make welcome lanterns for both banks.",
                ),
            };
        }
        if self.location == self.pip_location {
            return match (self.mission, self.mission_status) {
                (Mission::Picnic, MissionStatus::Completed) => (
                    "pip",
                    "What a lovely picnic! Thank you for bringing everything back.",
                ),
                (Mission::Picnic, _) if self.picnic_ready => (
                    "pip",
                    "The picnic is ready across the bridge. Shall we go together?",
                ),
                (Mission::Picnic, _) if self.picnic_basket > 0 => (
                    "pip",
                    "The basket is ready! There is a picnic place beside the bridge.",
                ),
                (Mission::Picnic, _) if self.berries >= 3 => (
                    "pip",
                    "You found the berries! The woodcutter can help pack our picnic.",
                ),
                (Mission::Picnic, _) => (
                    "pip",
                    "Let's gather berries and prepare a picnic beside our bridge.",
                ),
                (Mission::WelcomeLights, MissionStatus::Completed) => (
                    "pip",
                    "Both banks are glowing! Our path feels so welcoming.",
                ),
                (Mission::WelcomeLights, _) if self.near_lit || self.far_lit => (
                    "pip",
                    "One bank is glowing. We can light the other bank too.",
                ),
                _ => (
                    "pip",
                    "We can use glow pebbles to make welcome lanterns for both banks.",
                ),
            };
        }
        match self.mission {
            Mission::Picnic => (
                "",
                "The picnic place is beside the bridge. Pip is waiting on the far bank.",
            ),
            Mission::WelcomeLights if self.near_lit && self.far_lit => (
                "",
                "Welcome lanterns glow on both banks. The bridge and picnic are still here.",
            ),
            Mission::WelcomeLights if self.far_lit => (
                "",
                "A welcome lantern glows on the far bank. Pip is waiting near the picnic.",
            ),
            _ => (
                "",
                "The far-bank path leads back across the bridge. Pip is waiting near the picnic.",
            ),
        }
    }

    /// Mechanically legal bindings. Descriptions belong in a read-only projection.
    /// The developer history budget is checked separately by `apply`.
    pub fn available_actions(&self) -> Vec<Action> {
        let mut candidates = vec![Action::AcceptMission, Action::CancelMission];
        for destination in [Location::BridgeBank, Location::Workshop, Location::FarBank] {
            candidates.push(Action::Travel {
                destination_id: destination.as_str().into(),
            });
        }
        for (source, item) in [
            ("fallen_wood", "wood"),
            ("spare_wood", "wood"),
            ("berry_bush", "berries"),
            ("glow_stones", "glow_pebbles"),
        ] {
            let quantity = self.source_remaining(source).unwrap_or(0);
            if quantity > 0 {
                candidates.push(Action::Collect {
                    source_id: source.into(),
                    item_id: item.into(),
                    quantity,
                });
            }
        }
        candidates.extend([
            Action::Exchange {
                character_id: "woodcutter".into(),
                recipe_id: "wood_to_planks".into(),
            },
            Action::Assemble {
                object_id: "bridge".into(),
                recipe_id: "repair_bridge".into(),
            },
            Action::Accompany {
                character_id: "pip".into(),
                destination_id: "far_bank".into(),
            },
        ]);
        candidates.extend([
            Action::Exchange {
                character_id: "woodcutter".into(),
                recipe_id: "berries_to_picnic".into(),
            },
            Action::Exchange {
                character_id: "woodcutter".into(),
                recipe_id: "glow_to_lanterns".into(),
            },
            Action::Assemble {
                object_id: "picnic_table".into(),
                recipe_id: "prepare_picnic".into(),
            },
            Action::Assemble {
                object_id: "near_lanterns".into(),
                recipe_id: "light_near_bank".into(),
            },
            Action::Assemble {
                object_id: "far_lantern".into(),
                recipe_id: "light_far_bank".into(),
            },
            Action::Accompany {
                character_id: "pip".into(),
                destination_id: "bridge_bank".into(),
            },
        ]);
        candidates.extend(self.broader_actions());
        candidates.extend(self.exploration_actions());
        candidates.extend(items::FINDS.iter().map(|find| Action::Collect {
            source_id: find.source.into(),
            item_id: find.id.into(),
            quantity: 1,
        }));
        candidates
            .into_iter()
            .filter(|action| self.clone().transition(action).is_ok())
            .collect()
    }

    pub fn command_budget_exhausted(&self) -> bool {
        if self.durable {
            self.revision >= 1_000_000_000
        } else {
            self.revision >= MAX_COMMANDS as u64
        }
    }
    /// Only for a trusted repository with durable command receipts. This is not a
    /// browser/import checkpoint and is never accepted by from_checkpoint.
    pub fn compact(&self) -> Self {
        let mut state = self.clone();
        state.durable = true;
        state.history.clear();
        state.receipts.clear();
        let keep = state.events.len().saturating_sub(8);
        state.events.drain(..keep);
        state
    }
    pub fn durable_checkpoint(&self) -> Value {
        json!({"format":"story-current-1","contentVersion":CONTENT_VERSION,"state":self.compact()})
    }
    pub fn from_durable_checkpoint(value: &Value) -> Result<Self, Rejection> {
        if value["format"] != "story-current-1" || value["contentVersion"] != CONTENT_VERSION {
            return Err(Rejection::InvalidCheckpoint);
        }
        let state: Self = serde_json::from_value(value["state"].clone())
            .map_err(|_| Rejection::InvalidCheckpoint)?;
        if state
            .finds
            .iter()
            .any(|(id, quantity)| *quantity != 1 || !items::FINDS.iter().any(|find| find.id == id))
            || !state.valid_exploration()
            || state.durable_checkpoint() != *value
            || state.events.len() > 8
            || state.revision > 1_000_000_000
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        Ok(state)
    }
    pub fn last_command(&self) -> Option<&Command> {
        self.history.last()
    }
    /// Local developer checkpoint containing a replay log plus checked final projection.
    pub fn checkpoint(&self) -> Value {
        if self.durable {
            return self.durable_checkpoint();
        }
        json!({"version":CONTENT_VERSION,"initial":self.initial,"commands":self.history,"receipts":self.receipts,"state":self.projection()})
    }
    pub fn from_checkpoint(value: &Value) -> Result<Self, Rejection> {
        if serde_json::to_vec(value)
            .map_err(|_| Rejection::InvalidCheckpoint)?
            .len()
            > MAX_CHECKPOINT_BYTES
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Checkpoint {
            version: String,
            initial: InitialConditions,
            commands: Vec<Command>,
            receipts: BTreeMap<String, Receipt>,
            state: Value,
        }
        let checkpoint: Checkpoint =
            serde_json::from_value(value.clone()).map_err(|_| Rejection::InvalidCheckpoint)?;
        let legacy = checkpoint.version == LEGACY_CONTENT_VERSION;
        let legacy_v2 = checkpoint.version == SERIES_V2;
        if (!legacy && !legacy_v2 && checkpoint.version != CONTENT_VERSION)
            || checkpoint.commands.len() > MAX_COMMANDS
            || checkpoint.receipts.len() != checkpoint.commands.len()
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        let mut state = Self::with_initial(checkpoint.initial)?;
        for command in checkpoint.commands {
            // v1 had no transition from a completed bridge into another mission.
            // Do not reinterpret a forged legacy log using expanded v2 semantics.
            if ((legacy || legacy_v2)
                && matches!(&command.action, Action::Collect { source_id, .. } if source_id.starts_with("find_")))
                || ((legacy || legacy_v2)
                    && matches!(&command.action, Action::Travel { destination_id }
                    if !["bridge_bank", "workshop", "far_bank"].contains(&destination_id.as_str())))
                || ((legacy || legacy_v2)
                    && matches!(
                        command.action,
                        Action::GeneratedTravel { .. }
                            | Action::Inspect { .. }
                            | Action::Talk { .. }
                            | Action::Choose { .. }
                            | Action::Invite { .. }
                    ))
                || ((legacy || legacy_v2 && state.mission == Mission::WelcomeLights)
                    && state.mission_status == MissionStatus::Completed
                    && command.action == Action::AcceptMission)
            {
                return Err(Rejection::InvalidCheckpoint);
            }
            if state.receipts.contains_key(&command.command_id) {
                return Err(Rejection::InvalidCheckpoint);
            }
            state
                .apply(&command)
                .map_err(|_| Rejection::InvalidCheckpoint)?;
        }
        let expected = if legacy {
            state.legacy_checkpoint()
        } else if legacy_v2 {
            state.v2_checkpoint()
        } else {
            state.checkpoint()
        };
        if expected["state"] != checkpoint.state
            || state.receipts != checkpoint.receipts
            || expected != *value
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        Ok(state)
    }
    fn projection(&self) -> Value {
        let mut value = json!({"revision":self.revision,"location":self.location,"wood":self.wood,"planks":self.planks,"sources":self.sources,"bridgeRepaired":self.bridge_repaired,"crossedWithPip":self.crossed_with_pip,"pipLocation":self.pip_location,"missionStatus":self.mission_status,"milestones":self.milestones,"metWoodcutter":self.met_woodcutter,"events":self.events,
        "mission":self.mission,"completedMissions":self.completed_missions,"berries":self.berries,"picnicBasket":self.picnic_basket,"glowPebbles":self.glow_pebbles,"lanterns":self.lanterns,"picnicReady":self.picnic_ready,"picnicShared":self.picnic_shared,"nearLit":self.near_lit,"farLit":self.far_lit,"broader":self.broader});
        if !self.finds.is_empty() {
            value["finds"] = json!(self.finds);
        }
        if !self.neighbourhood_projects.is_empty() {
            value["neighbourhoodProjects"] = json!(self.neighbourhood_projects);
        }
        if !self.exploration.is_empty() {
            value["exploration"] = json!(self.exploration);
        }
        value
    }
    fn v2_checkpoint(&self) -> Value {
        let mut state = self.projection();
        state.as_object_mut().unwrap().remove("broader");
        json!({"version":SERIES_V2,"initial":self.initial,"commands":self.history,"receipts":self.receipts,"state":state})
    }
    fn legacy_checkpoint(&self) -> Value {
        let sources: BTreeMap<_, _> = self
            .sources
            .iter()
            .filter(|(id, _)| matches!(id.as_str(), "fallen_wood" | "spare_wood"))
            .collect();
        let state = json!({"revision":self.revision,"location":self.location,"wood":self.wood,"planks":self.planks,"sources":sources,"bridgeRepaired":self.bridge_repaired,"crossedWithPip":self.crossed_with_pip,"pipLocation":self.pip_location,"missionStatus":self.mission_status,"milestones":self.milestones,"metWoodcutter":self.met_woodcutter,"events":self.events});
        json!({"version":LEGACY_CONTENT_VERSION,"initial":self.initial,"commands":self.history,"receipts":self.receipts,"state":state})
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
}

#[cfg(test)]
mod tests;
