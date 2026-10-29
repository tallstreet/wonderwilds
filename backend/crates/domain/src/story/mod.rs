//! Pure authored story sandbox. Independent of the assessed adventure and providers.
//!
//! Coarse travel validates a graph transition; it does not prove browser movement.
//! Checkpoints are bounded local developer records, not authenticated server saves.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const CONTENT_VERSION: &str = "pip-bridge-1";
pub const MAX_COMMANDS: usize = 256;
const MAX_CHECKPOINT_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    BridgeBank,
    Workshop,
    FarBank,
}
impl Location {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BridgeBank => "bridge_bank",
            Self::Workshop => "workshop",
            Self::FarBank => "far_bank",
        }
    }
    fn parse(value: &str) -> Result<Self, Rejection> {
        match value {
            "bridge_bank" => Ok(Self::BridgeBank),
            "workshop" => Ok(Self::Workshop),
            "far_bank" => Ok(Self::FarBank),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
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
            initial,
            revision: 0,
            location: Location::BridgeBank,
            wood: initial.wood,
            planks: initial.planks,
            sources: BTreeMap::from([("fallen_wood".into(), 3), ("spare_wood".into(), 3)]),
            bridge_repaired: initial.bridge_repaired,
            crossed_with_pip: false,
            pip_location: Location::BridgeBank,
            mission_status: MissionStatus::Offered,
            milestones: Vec::new(),
            met_woodcutter: false,
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
        if self.history.len() >= MAX_COMMANDS {
            return Err(Rejection::LimitReached);
        }
        let mut candidate = self.clone();
        let description = candidate.transition(&command.action)?;
        candidate.evaluate_milestones();
        candidate.revision += 1;
        let receipt = Receipt {
            command_id: command.command_id.clone(),
            revision: candidate.revision,
            description,
        };
        candidate.events.push(Event {
            revision: candidate.revision,
            description: receipt.description.clone(),
        });
        candidate.history.push(command.clone());
        candidate
            .receipts
            .insert(command.command_id.clone(), receipt.clone());
        *self = candidate;
        Ok(receipt)
    }

    fn transition(&mut self, action: &Action) -> Result<String, Rejection> {
        match action {
            Action::AcceptMission => {
                if !matches!(
                    self.mission_status,
                    MissionStatus::Offered | MissionStatus::Abandoned
                ) {
                    return Err(Rejection::NotAvailable);
                }
                if self.milestones.is_empty() {
                    self.initialize_milestones();
                }
                self.mission_status = MissionStatus::Active;
                Ok("The player accepted the mission to cross the bridge with Pip.".into())
            }
            Action::CancelMission => {
                self.require_active()?;
                self.mission_status = MissionStatus::Abandoned;
                Ok(
                    "The player paused Pip's mission; items and world changes were preserved."
                        .into(),
                )
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
                self.require_active()?;
                let source_location = match source_id.as_str() {
                    "fallen_wood" => Location::BridgeBank,
                    "spare_wood" => Location::Workshop,
                    _ => return Err(Rejection::InvalidAction),
                };
                if item_id != "wood" || !(1..=3).contains(quantity) {
                    return Err(Rejection::InvalidAction);
                }
                if self.location != source_location || self.bridge_repaired {
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
                self.wood += quantity;
                Ok(format!(
                    "The player collected {quantity} wood from {source_id}."
                ))
            }
            Action::Exchange {
                character_id,
                recipe_id,
            } => {
                self.require_active()?;
                if character_id != "woodcutter" || recipe_id != "wood_to_planks" {
                    return Err(Rejection::InvalidAction);
                }
                if self.location != Location::Workshop || self.bridge_repaired {
                    return Err(Rejection::NotAvailable);
                }
                if self.wood < 3 {
                    return Err(Rejection::InsufficientInventory);
                }
                self.wood -= 3;
                self.planks += 3;
                Ok("The woodcutter exchanged three wood for three planks.".into())
            }
            Action::Assemble {
                object_id,
                recipe_id,
            } => {
                self.require_active()?;
                if object_id != "bridge" || recipe_id != "repair_bridge" {
                    return Err(Rejection::InvalidAction);
                }
                if self.location != Location::BridgeBank || self.bridge_repaired {
                    return Err(Rejection::NotAvailable);
                }
                if self.planks < 3 {
                    return Err(Rejection::InsufficientInventory);
                }
                self.planks -= 3;
                self.bridge_repaired = true;
                Ok("The player used three planks to repair the bridge.".into())
            }
            Action::Accompany {
                character_id,
                destination_id,
            } => {
                self.require_active()?;
                if character_id != "pip" || destination_id != "far_bank" {
                    return Err(Rejection::InvalidAction);
                }
                if !self.bridge_repaired
                    || self.location != Location::BridgeBank
                    || self.pip_location != self.location
                {
                    return Err(Rejection::NotAvailable);
                }
                self.location = Location::FarBank;
                self.pip_location = Location::FarBank;
                self.crossed_with_pip = true;
                Ok("The player and Pip crossed the repaired bridge together.".into())
            }
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
        matches!(
            (self.location, to),
            (Location::BridgeBank, Location::Workshop) | (Location::Workshop, Location::BridgeBank)
        ) || self.bridge_repaired
            && matches!(
                (self.location, to),
                (Location::BridgeBank, Location::FarBank)
                    | (Location::FarBank, Location::BridgeBank)
            )
    }
    fn initialize_milestones(&mut self) {
        let mut definitions = Vec::new();
        if !self.bridge_repaired {
            if self.planks < 3 {
                definitions.push(("wood_ready", "Find suitable wood."));
            }
            definitions.push(("planks_ready", "Get the planks ready."));
            definitions.push(("planks_at_bridge", "Bring the planks to the bridge."));
        }
        definitions.push(("bridge_repaired", "Repair the bridge."));
        definitions.push(("crossed_with_pip", "Cross with Pip."));
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
                _ => false,
            };
        }
        if self.bridge_repaired && self.crossed_with_pip {
            self.mission_status = MissionStatus::Completed;
        }
    }

    /// Canonical authored lines. Acknowledgements are bound to this revision and entry.
    /// Calling this read-only method does not establish a meeting or any progress.
    pub fn presentation(&self) -> Presentation {
        let (speaker, text) = match self.location {
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
            Location::FarBank => (
                "",
                "The player reached the far bank. Pip is waiting across the bridge.",
            ),
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

    /// Mechanically legal bindings. Descriptions belong in a read-only projection.
    /// The developer history budget is checked separately by `apply`.
    pub fn available_actions(&self) -> Vec<Action> {
        let mut candidates = vec![Action::AcceptMission, Action::CancelMission];
        for destination in [Location::BridgeBank, Location::Workshop, Location::FarBank] {
            candidates.push(Action::Travel {
                destination_id: destination.as_str().into(),
            });
        }
        for source in ["fallen_wood", "spare_wood"] {
            let quantity = self.source_remaining(source).unwrap_or(0);
            if quantity > 0 {
                candidates.push(Action::Collect {
                    source_id: source.into(),
                    item_id: "wood".into(),
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
        candidates
            .into_iter()
            .filter(|action| self.clone().transition(action).is_ok())
            .collect()
    }

    /// Local developer checkpoint containing a replay log plus checked final projection.
    pub fn checkpoint(&self) -> Value {
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
        if checkpoint.version != CONTENT_VERSION
            || checkpoint.commands.len() > MAX_COMMANDS
            || checkpoint.receipts.len() != checkpoint.commands.len()
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        let mut state = Self::with_initial(checkpoint.initial)?;
        for command in checkpoint.commands {
            if state.receipts.contains_key(&command.command_id) {
                return Err(Rejection::InvalidCheckpoint);
            }
            state
                .apply(&command)
                .map_err(|_| Rejection::InvalidCheckpoint)?;
        }
        if state.projection() != checkpoint.state
            || state.receipts != checkpoint.receipts
            || state.checkpoint() != *value
        {
            return Err(Rejection::InvalidCheckpoint);
        }
        Ok(state)
    }
    fn projection(&self) -> Value {
        json!({"revision":self.revision,"location":self.location,"wood":self.wood,"planks":self.planks,"sources":self.sources,"bridgeRepaired":self.bridge_repaired,"crossedWithPip":self.crossed_with_pip,"pipLocation":self.pip_location,"missionStatus":self.mission_status,"milestones":self.milestones,"metWoodcutter":self.met_woodcutter,"events":self.events})
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
