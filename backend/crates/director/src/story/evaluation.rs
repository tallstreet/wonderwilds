//! Offline synthetic scene evaluation. No model, persistence or gameplay side effects.
use super::{authored_scene, build_scene_input, resolve_action, validate_scene};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wonderwilds_domain::story::{Action, Command, InitialConditions, Location, State};

pub const FORMAT: &str = "story-eval-1";
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: String,
    pub scenario_id: String,
    /// Keep the entire journey together when splitting training/evaluation data.
    pub family_id: String,
    pub content_version: String,
    pub state_revision: u64,
    pub checkpoint: Value,
    pub input: Value,
    /// Engine-owned metadata; unlike reference, this is required generation input.
    pub presentation: Value,
    /// Authored comparison baseline, not a human-approved training target.
    pub reference: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Candidate {
    pub scenario_id: String,
    pub candidate_id: String,
    pub output: Value,
    /// Optional provider provenance is recorded by the orchestration layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Grade {
    pub scenario_id: String,
    pub candidate_id: String,
    pub passed: bool,
    pub failures: Vec<String>,
    pub simulated_actions: usize,
}

fn id_valid(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_:".contains(&c))
}

fn scenario(state: &State, family: &str) -> Result<Scenario> {
    let input = build_scene_input(state).map_err(|e| format!("Input: {e:?}"))?;
    let reference = authored_scene(state).map_err(|e| format!("Reference: {e:?}"))?;
    let required = if state.location() == Location::Workshop && !state.met_woodcutter() {
        vec![reference["presentation"][0].clone()]
    } else {
        vec![]
    };
    Ok(Scenario {
        schema_version: FORMAT.into(),
        scenario_id: format!("{family}-{:03}", state.revision()),
        family_id: family.into(),
        content_version: wonderwilds_domain::story::CONTENT_VERSION.into(),
        state_revision: state.revision(),
        checkpoint: state.checkpoint(),
        presentation: json!({"sceneId":reference["sceneId"],"requiredEntries":required}),
        input,
        reference,
    })
}

/// Replay validates consistency, not authenticity. Never import these synthetic
/// checkpoints into authenticated game progress.
pub fn validate_scenario(value: &Scenario) -> Result<State> {
    if value.content_version != wonderwilds_domain::story::CONTENT_VERSION {
        return Err(format!(
            "Corpus content version {} differs from current {}. Use archived dataset review for historical evidence, or prepare a new corpus; do not transfer old approvals to changed inputs.",
            value.content_version,
            wonderwilds_domain::story::CONTENT_VERSION
        ));
    }
    if !id_valid(&value.scenario_id) || !id_valid(&value.family_id) {
        return Err("Invalid scenario/family identifier".into());
    }
    let state = State::from_checkpoint(&value.checkpoint)
        .map_err(|e| format!("Checkpoint replay failed: {e:?}"))?;
    let expected = scenario(&state, &value.family_id)?;
    if value.schema_version != expected.schema_version
        || value.content_version != expected.content_version
        || value.state_revision != expected.state_revision
        || value.input != expected.input
        || value.presentation != expected.presentation
        || value.reference != expected.reference
    {
        return Err("Scenario projection/version differs from replayed runtime state".into());
    }
    Ok(state)
}

/// Checks structure/bindings and independently executes each offered action on a
/// cloned state. Does not grade prose truth, suitability, fun or novel missions.
pub fn grade_candidate(scenario: &Scenario, candidate: &Candidate) -> Result<Grade> {
    let state = validate_scenario(scenario)?;
    if !id_valid(&candidate.candidate_id) || candidate.scenario_id != scenario.scenario_id {
        return Err("Candidate identifier or scenario binding is invalid".into());
    }
    let mut grade = Grade {
        scenario_id: candidate.scenario_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        passed: false,
        failures: vec![],
        simulated_actions: 0,
    };
    if let Err(error) = validate_scene(&scenario.input, &scenario.reference, &candidate.output) {
        grade.failures.push(format!("scene_validation: {error:?}"));
        return Ok(grade);
    }
    // A valid imported trace may already use our preferred command IDs.
    let command_id = (0..=state.revision())
        .map(|n| format!("eval-check-{n}"))
        .find(|id| scenario.checkpoint["receipts"].get(id).is_none())
        .expect("More candidate IDs than existing receipts");
    for choice in candidate.output["choices"].as_array().unwrap() {
        let action = resolve_action(&state, choice["actionId"].as_str().unwrap())
            .map_err(|e| format!("Runtime handle resolution failed: {e:?}"))?;
        let mut next = state.clone();
        let command = Command {
            command_id: command_id.clone(),
            expected_revision: state.revision(),
            action,
        };
        match next.apply(&command) {
            Ok(receipt) => {
                let checkpoint = next.checkpoint();
                if next.apply(&command) != Ok(receipt)
                    || next.checkpoint() != checkpoint
                    || State::from_checkpoint(&checkpoint).is_err()
                {
                    grade.failures.push("runtime_replay_or_retry_failed".into());
                }
                grade.simulated_actions += 1;
            }
            Err(error) => grade
                .failures
                .push(format!("runtime_action_rejected: {error:?}")),
        }
    }
    grade.passed = grade.failures.is_empty();
    Ok(grade)
}

struct Journey {
    state: State,
    family: &'static str,
    rows: Vec<Scenario>,
}
impl Journey {
    fn new(family: &'static str, initial: InitialConditions) -> Result<Self> {
        let state = State::with_initial(initial).map_err(|e| format!("Initial state: {e:?}"))?;
        let first = scenario(&state, family)?;
        Ok(Self {
            state,
            family,
            rows: vec![first],
        })
    }
    fn step(&mut self, action: Action) -> Result<()> {
        self.state
            .apply(&Command {
                command_id: format!("eval-{}", self.state.revision()),
                expected_revision: self.state.revision(),
                action,
            })
            .map_err(|e| format!("{} revision {}: {e:?}", self.family, self.state.revision()))?;
        self.rows.push(scenario(&self.state, self.family)?);
        Ok(())
    }
    fn travel(&mut self, to: &str) -> Result<()> {
        if self.state.location().as_str() != to {
            self.step(Action::Travel {
                destination_id: to.into(),
            })?;
        }
        if to == "workshop" && !self.state.met_woodcutter() {
            let p = self.state.presentation();
            self.step(Action::Acknowledge {
                scene_id: p.scene_id,
                entry_id: p.entries[0].entry_id.clone(),
            })?;
        }
        Ok(())
    }
    fn gather(&mut self, source: &str, item: &str) -> Result<()> {
        for quantity in [1, 2] {
            self.step(Action::Collect {
                source_id: source.into(),
                item_id: item.into(),
                quantity,
            })?;
            if quantity == 1 {
                self.step(Action::CancelMission)?;
                self.step(Action::AcceptMission)?;
            }
        }
        Ok(())
    }
    fn exchange(&mut self, recipe: &str) -> Result<()> {
        self.step(Action::Exchange {
            character_id: "woodcutter".into(),
            recipe_id: recipe.into(),
        })
    }
    fn assemble(&mut self, object: &str, recipe: &str) -> Result<()> {
        self.step(Action::Assemble {
            object_id: object.into(),
            recipe_id: recipe.into(),
        })
    }
    fn accompany(&mut self, to: &str) -> Result<()> {
        self.step(Action::Accompany {
            character_id: "pip".into(),
            destination_id: to.into(),
        })
    }
    fn talk(&mut self, who: &str, topic: &str) -> Result<()> {
        self.step(Action::Talk {
            character_id: who.into(),
            topic_id: topic.into(),
        })
    }
    fn inspect(&mut self, object: &str) -> Result<()> {
        self.step(Action::Inspect {
            object_id: object.into(),
        })
    }
    fn choose(&mut self, choice: &str, option: &str) -> Result<()> {
        self.step(Action::Choose {
            choice_id: choice.into(),
            option_id: option.into(),
        })
    }
    fn pause_resume(&mut self) -> Result<()> {
        self.step(Action::CancelMission)?;
        self.step(Action::AcceptMission)
    }
    fn broader(&mut self, variant: usize) -> Result<()> {
        // Choices vary independently: all eight combinations appear in the corpus.
        let instrument_gathering = variant & 1 != 0;
        let star_corner = variant & 2 != 0;
        let far_gathering = variant & 4 != 0;
        self.travel("bridge_bank")?;
        self.step(Action::AcceptMission)?;
        self.talk("pip", "humming_hollow")?;
        self.travel("far_bank")?;
        self.inspect("humming_hollow")?;
        if variant.is_multiple_of(2) {
            self.inspect("mossy_stump")?;
        }
        self.inspect("trembling_leaf")?;
        self.pause_resume()?;
        self.inspect("hollow_branch")?;
        self.choose(
            "instrument_home",
            if instrument_gathering {
                "gathering"
            } else {
                "woods"
            },
        )?;
        if instrument_gathering {
            self.travel("bridge_bank")?;
        }
        self.assemble("wind_instrument", "place_instrument")?;
        self.travel("bridge_bank")?;
        self.talk("pip", "share_discovery")?;

        self.step(Action::AcceptMission)?;
        self.talk("pip", "cosy_preferences")?;
        self.inspect("corner_site")?;
        self.choose(
            "corner_style",
            if star_corner {
                "star_canopy"
            } else {
                "leafy_nook"
            },
        )?;
        self.assemble("corner_frame", "build_corner_frame")?;
        self.pause_resume()?;
        self.assemble("corner_cushions", "arrange_cushions")?;
        self.talk("pip", "enjoy_corner")?;

        self.step(Action::AcceptMission)?;
        self.travel("workshop")?;
        self.talk("bramble", "visit_wish")?;
        self.inspect("workshop_path")?;
        self.assemble("workshop_path", "clear_path")?;
        self.talk("bramble", "ready_to_visit")?;
        self.step(Action::Accompany {
            character_id: "bramble".into(),
            destination_id: "bridge_bank".into(),
        })?;
        self.talk("bramble", "visit_arrival")?;

        self.step(Action::AcceptMission)?;
        self.choose(
            "gathering_place",
            if far_gathering {
                "far_bank"
            } else {
                "near_bank"
            },
        )?;
        let friends = if variant.is_multiple_of(2) {
            ["pip", "bramble"]
        } else {
            ["bramble", "pip"]
        };
        self.step(Action::Invite {
            character_id: friends[0].into(),
        })?;
        self.pause_resume()?;
        self.step(Action::Invite {
            character_id: friends[1].into(),
        })?;
        if far_gathering {
            self.travel("far_bank")?;
        }
        let preparations = if star_corner {
            [
                ("gathering_music", "prepare_music"),
                ("gathering_seats", "arrange_seats"),
            ]
        } else {
            [
                ("gathering_seats", "arrange_seats"),
                ("gathering_music", "prepare_music"),
            ]
        };
        for (object, recipe) in preparations {
            self.assemble(object, recipe)?;
        }
        self.talk("pip", "welcome_friends")?;
        Ok(())
    }
    fn play(
        mut self,
        workshop_source: bool,
        far_first: bool,
        variant: usize,
    ) -> Result<Vec<Scenario>> {
        if workshop_source {
            self.travel("workshop")?;
            self.travel("bridge_bank")?;
        }
        self.step(Action::AcceptMission)?;
        if !self.state.bridge_repaired() {
            if self.state.planks() < 3 {
                if workshop_source {
                    self.travel("workshop")?;
                }
                self.gather(
                    if workshop_source {
                        "spare_wood"
                    } else {
                        "fallen_wood"
                    },
                    "wood",
                )?;
                self.travel("workshop")?;
                self.exchange("wood_to_planks")?;
            }
            self.travel("bridge_bank")?;
            self.assemble("bridge", "repair_bridge")?;
        }
        self.accompany("far_bank")?;
        self.step(Action::AcceptMission)?;
        self.gather("berry_bush", "berries")?;
        self.travel("bridge_bank")?;
        self.travel("workshop")?;
        self.exchange("berries_to_picnic")?;
        self.travel("bridge_bank")?;
        self.assemble("picnic_table", "prepare_picnic")?;
        self.travel("far_bank")?;
        self.accompany("bridge_bank")?;
        self.step(Action::AcceptMission)?;
        self.travel("far_bank")?;
        self.gather("glow_stones", "glow_pebbles")?;
        self.travel("bridge_bank")?;
        self.travel("workshop")?;
        self.exchange("glow_to_lanterns")?;
        self.travel("bridge_bank")?;
        if far_first {
            self.travel("far_bank")?;
            self.assemble("far_lantern", "light_far_bank")?;
            self.travel("bridge_bank")?;
            self.assemble("near_lanterns", "light_near_bank")?;
        } else {
            self.assemble("near_lanterns", "light_near_bank")?;
            self.travel("far_bank")?;
            self.assemble("far_lantern", "light_far_bank")?;
        }
        self.broader(variant)?;
        Ok(self.rows)
    }
}

/// Reproducible seven-mission corpus with eight combinations of lasting choices.
/// These are authored journeys, not a benchmark for arbitrary generated missions.
pub fn corpus() -> Result<Vec<Scenario>> {
    let mut rows = vec![];
    for (variant, (family, initial, workshop, far_first)) in [
        ("bank-series", InitialConditions::default(), false, false),
        ("workshop-series", InitialConditions::default(), true, true),
        (
            "supplied-series",
            InitialConditions {
                planks: 3,
                ..Default::default()
            },
            false,
            true,
        ),
        (
            "repaired-series",
            InitialConditions {
                bridge_repaired: true,
                ..Default::default()
            },
            false,
            false,
        ),
        (
            "far-gathering-woods-leafy",
            InitialConditions::default(),
            false,
            true,
        ),
        (
            "far-gathering-instrument-leafy",
            InitialConditions::default(),
            true,
            false,
        ),
        (
            "far-gathering-woods-stars",
            InitialConditions::default(),
            false,
            false,
        ),
        (
            "far-gathering-instrument-stars",
            InitialConditions::default(),
            true,
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        rows.extend(Journey::new(family, initial)?.play(workshop, far_first, variant)?);
    }
    Ok(rows)
}
