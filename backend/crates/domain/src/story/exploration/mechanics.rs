//! Fixed project recipes. Model prose cannot supply quantities, effects or rewards.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanic {
    #[default]
    Inspect,
    GatherWood,
    GatherBerries,
    GatherGlowPebbles,
    CraftPlanks,
    PackBasket,
    CraftLantern,
    DeliverBasket,
    DeliverPlanks,
    BuildSeat,
    PlaceLantern,
    ChooseStyle,
}
type Stack = (&'static str, u32);

impl Mechanic {
    pub fn is_inspect(&self) -> bool {
        *self == Self::Inspect
    }
    fn recipe(self) -> (Option<Stack>, Option<Stack>) {
        use Mechanic::*;
        match self {
            GatherWood => (None, Some(("wood", 3))),
            GatherBerries => (None, Some(("berries", 3))),
            GatherGlowPebbles => (None, Some(("glow_pebbles", 3))),
            CraftPlanks => (Some(("wood", 3)), Some(("planks", 3))),
            PackBasket => (Some(("berries", 3)), Some(("picnic_basket", 1))),
            CraftLantern => (Some(("glow_pebbles", 3)), Some(("lanterns", 1))),
            DeliverBasket => (Some(("picnic_basket", 1)), None),
            DeliverPlanks | BuildSeat => (Some(("planks", 3)), None),
            PlaceLantern => (Some(("lanterns", 1)), None),
            Inspect | ChooseStyle => (None, None),
        }
    }
    pub(super) fn apply(self, supplies: &mut BTreeMap<String, u32>) -> Result<(), Rejection> {
        let (input, output) = self.recipe();
        if let Some((item, count)) = input {
            let remaining = supplies
                .get(item)
                .copied()
                .unwrap_or(0)
                .checked_sub(count)
                .ok_or(Rejection::InsufficientInventory)?;
            if remaining == 0 {
                supplies.remove(item);
            } else {
                supplies.insert(item.into(), remaining);
            }
        }
        if let Some((item, count)) = output {
            let total = supplies
                .get(item)
                .copied()
                .unwrap_or(0)
                .checked_add(count)
                .filter(|n| *n <= 18)
                .ok_or(Rejection::LimitReached)?;
            supplies.insert(item.into(), total);
        }
        Ok(())
    }
    pub fn project_kind(self) -> Option<&'static str> {
        match self {
            Self::DeliverBasket => Some("picnic"),
            Self::DeliverPlanks => Some("supplies"),
            Self::BuildSeat => Some("seat"),
            Self::PlaceLantern => Some("lantern"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Project {
    pub location: Location,
    pub kind: String,
    pub style: String,
}

impl GeneratedStory {
    pub(super) fn valid_mechanics(&self) -> bool {
        let mut supplies = BTreeMap::new();
        let mut chosen = false;
        let mut productive = false;
        let has_mechanics = self.content.steps.iter().any(|s| !s.mechanic.is_inspect());
        let mut last = None;
        for step in &self.content.steps {
            if has_mechanics {
                let Some(location) = step
                    .location_id
                    .as_ref()
                    .and_then(|id| PLACES.iter().find(|p| p.location.as_str() == id))
                    .map(|p| p.location)
                else {
                    return false;
                };
                if last.is_some_and(|from| from != location && !connected(from, location)) {
                    return false;
                }
                last = Some(location);
            }
            if step.mechanic == Mechanic::ChooseStyle {
                if chosen {
                    return false;
                }
                chosen = true;
            }
            productive |= step.mechanic.project_kind().is_some();
            if step.mechanic.apply(&mut supplies).is_err() {
                return false;
            }
        }
        !has_mechanics || (self.content.complete_on_last_step && productive && supplies.is_empty())
    }
}

impl Quest {
    pub fn step_object_id(&self) -> String {
        if self
            .story
            .as_ref()
            .is_some_and(|s| s.content.steps.iter().any(|s| !s.mechanic.is_inspect()))
        {
            format!(
                "neighbour_{}_g{}_{}",
                self.character, self.generation, self.step
            )
        } else {
            format!("neighbour_{}_{}", self.character, self.step)
        }
    }
    pub fn step_actions(&self) -> Vec<Action> {
        let Some(step) = self
            .story
            .as_ref()
            .and_then(|s| s.content.steps.get(self.step as usize))
        else {
            return vec![];
        };
        let id = self.step_object_id();
        let action = match step.mechanic {
            Mechanic::Inspect => Action::Inspect { object_id: id },
            Mechanic::GatherWood | Mechanic::GatherBerries | Mechanic::GatherGlowPebbles => {
                let (_, Some((item, count))) = step.mechanic.recipe() else {
                    unreachable!()
                };
                Action::Collect {
                    source_id: id,
                    item_id: item.into(),
                    quantity: count,
                }
            }
            Mechanic::ChooseStyle => {
                return ["leafy", "starry"]
                    .iter()
                    .map(|style| Action::Choose {
                        choice_id: id.clone(),
                        option_id: (*style).into(),
                    })
                    .collect();
            }
            mechanic => Action::Assemble {
                object_id: id,
                recipe_id: serde_json::to_value(mechanic)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .into(),
            },
        };
        vec![action]
    }
    pub(super) fn valid_supplies(&self) -> bool {
        let mut expected = BTreeMap::new();
        let mut chosen = false;
        if let Some(story) = &self.story {
            for step in story.content.steps.iter().take(self.step as usize) {
                if step.mechanic.apply(&mut expected).is_err() {
                    return false;
                }
                chosen |= step.mechanic == Mechanic::ChooseStyle;
            }
        }
        expected == self.supplies
            && if chosen {
                matches!(self.style.as_deref(), Some("leafy" | "starry"))
            } else {
                self.style.is_none()
            }
    }
}

pub fn action_object_id(action: &Action) -> Option<&str> {
    match action {
        Action::Inspect { object_id } | Action::Assemble { object_id, .. } => Some(object_id),
        Action::Collect { source_id, .. } => Some(source_id),
        Action::Choose { choice_id, .. } => Some(choice_id),
        _ => None,
    }
}

impl State {
    pub fn neighbour_for_action(&self, action: &Action) -> Option<&Quest> {
        self.exploration.iter().find(|q| {
            q.status == "open"
                && !q.inspected
                && q.task_location() == self.location
                && (q.step_actions().contains(action) || (q.story.is_none() && matches!(action, Action::Inspect { object_id } if *object_id == format!("neighbour_{}", q.character))))
        })
    }
    pub fn neighbourhood_projects(&self) -> &BTreeMap<String, Project> {
        &self.neighbourhood_projects
    }
    pub(super) fn project_transition(
        &mut self,
        action: &Action,
    ) -> Option<Result<String, Rejection>> {
        if !action_object_id(action).is_some_and(|id| id.starts_with("neighbour_")) {
            return None;
        }
        // Plain inspections retain their historical transition and checkpoint meaning.
        if matches!(action, Action::Inspect { .. }) {
            return None;
        }
        Some((|| {
            let q = self
                .neighbour_for_action(action)
                .ok_or(Rejection::NotAvailable)?;
            let character = q.character.clone();
            let mechanic = q.story.as_ref().unwrap().content.steps[q.step as usize].mechanic;
            let q = self
                .exploration
                .iter_mut()
                .find(|q| q.character == character)
                .unwrap();
            mechanic.apply(&mut q.supplies)?;
            if let Action::Choose { option_id, .. } = action {
                q.style = Some(option_id.clone());
            }
            if let Some(kind) = mechanic.project_kind() {
                let key = format!("{}_{kind}", self.location.as_str());
                self.neighbourhood_projects.insert(
                    key,
                    Project {
                        location: self.location,
                        kind: kind.into(),
                        style: q.style.clone().unwrap_or_else(|| "leafy".into()),
                    },
                );
            }
            q.step += 1;
            let content = &q.story.as_ref().unwrap().content;
            q.inspected = q.step as usize == content.steps.len();
            if q.inspected && content.complete_on_last_step {
                q.status = "completed".into();
                return Ok(content.payoff.clone());
            }
            Ok(content.steps[q.step as usize - 1]
                .discovery
                .clone()
                .unwrap_or_else(|| "Project step completed.".into()))
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::{Command, InitialConditions};
    use serde_json::json;

    fn apply(state: &mut State, action: Action) -> crate::story::Receipt {
        state
            .apply(&Command {
                command_id: format!("project-{}", state.revision()),
                expected_revision: state.revision(),
                action,
            })
            .unwrap()
    }
    fn content(style: bool) -> GeneratedStory {
        let mut steps = vec![];
        if style {
            steps.push(json!({"mechanic":"choose_style","locationId":"cove","label":"Choose a design","shape":"leaf"}));
        }
        for (mechanic, label, location) in [
            ("gather_wood", "Gather wood", "cove"),
            ("craft_planks", "Make planks", "cove"),
            ("build_seat", "Build the seat", "fern_hollow"),
        ] {
            steps.push(
                json!({"mechanic":mechanic,"locationId":location,"label":label,"shape":"hollow"}),
            );
        }
        GeneratedStory { model_generated: false, content: serde_json::from_value(json!({
            "name":"Wren", "personality":"A hopeful maker.","greeting":"Let's build a seat.",
            "sprite":"flutter","title":"A resting seat","premise":"Build a seat beside the path.",
            "payoff":"The new seat is ready.","completeOnLastStep":true,"steps":steps
        })).unwrap() }
    }
    fn start(style: bool) -> State {
        let mut state = State::with_initial(InitialConditions {
            wood: 2,
            planks: 1,
            bridge_repaired: false,
        })
        .unwrap();
        apply(
            &mut state,
            Action::Travel {
                destination_id: "workshop".into(),
            },
        );
        apply(
            &mut state,
            Action::GeneratedTravel {
                destination_id: "cove".into(),
                story: content(style),
            },
        );
        apply(
            &mut state,
            Action::Talk {
                character_id: "marina".into(),
                topic_id: "start_neighbour_story".into(),
            },
        );
        state
    }
    fn step(state: &State) -> Action {
        state
            .exploration
            .iter()
            .find(|q| q.character == "marina")
            .unwrap()
            .step_actions()[0]
            .clone()
    }
    fn reject(state: &mut State, action: Action) {
        let before = state.checkpoint();
        assert!(
            state
                .apply(&Command {
                    command_id: "invalid-project-action".into(),
                    expected_revision: state.revision(),
                    action
                })
                .is_err()
        );
        assert_eq!(state.checkpoint(), before);
    }

    #[test]
    fn project_recipes_are_atomic_reserved_idempotent_and_finish_at_destination() {
        let mut state = start(false);
        let gather = step(&state);
        let mut forged = gather.clone();
        if let Action::Collect { quantity, .. } = &mut forged {
            *quantity = 99;
        }
        reject(&mut state, forged);
        reject(
            &mut state,
            Action::Assemble {
                object_id: "neighbour_marina_g1_1".into(),
                recipe_id: "craft_planks".into(),
            },
        );
        let command = Command {
            command_id: "exact-gather".into(),
            expected_revision: state.revision(),
            action: gather,
        };
        let receipt = state.apply(&command).unwrap();
        assert_eq!(state.apply(&command).unwrap(), receipt);
        assert_eq!(state.quantity("wood"), 5);
        assert_eq!(state.wood(), 2, "Pip's supplies are separate");
        let craft = step(&state);
        apply(
            &mut state,
            Action::Travel {
                destination_id: "fern_hollow".into(),
            },
        );
        reject(&mut state, craft.clone());
        apply(
            &mut state,
            Action::Travel {
                destination_id: "cove".into(),
            },
        );
        apply(&mut state, craft);
        assert_eq!(state.quantity("planks"), 4);
        assert_eq!(state.quantity("wood"), 2);
        state = State::from_durable_checkpoint(&state.durable_checkpoint()).unwrap();
        let build = step(&state);
        reject(&mut state, build.clone());
        apply(
            &mut state,
            Action::Travel {
                destination_id: "fern_hollow".into(),
            },
        );
        let final_command = Command {
            command_id: "exact-build".into(),
            expected_revision: state.revision(),
            action: build.clone(),
        };
        let receipt = state.apply(&final_command).unwrap();
        assert_eq!(state.apply(&final_command).unwrap(), receipt);
        assert_eq!(receipt.description, "The new seat is ready.");
        assert_eq!(state.exploration[0].status, "completed");
        assert!(state.exploration[0].supplies.is_empty());
        assert_eq!(state.quantity("planks"), 1);
        assert!(
            state
                .neighbourhood_projects
                .contains_key("fern_hollow_seat")
        );
        reject(&mut state, build);
        let saved = state.durable_checkpoint();
        assert_eq!(
            State::from_durable_checkpoint(&saved)
                .unwrap()
                .durable_checkpoint(),
            saved
        );
        // A later mission never erases a finished project and cannot reuse old actions.
        apply(
            &mut state,
            Action::GeneratedTravel {
                destination_id: "cove".into(),
                story: content(false),
            },
        );
        assert!(
            state
                .neighbourhood_projects
                .contains_key("fern_hollow_seat")
        );
        apply(
            &mut state,
            Action::Talk {
                character_id: "marina".into(),
                topic_id: "start_neighbour_story".into(),
            },
        );
        reject(&mut state, command.action);
    }

    #[test]
    fn both_creative_choices_are_valid_and_saved_projects_remember_the_design() {
        for style in ["leafy", "starry"] {
            let mut state = start(true);
            let mut choose = step(&state);
            if let Action::Choose { option_id, .. } = &mut choose {
                *option_id = style.into();
            }
            apply(&mut state, choose);
            let action = step(&state);
            apply(&mut state, action);
            let action = step(&state);
            apply(&mut state, action);
            apply(
                &mut state,
                Action::Travel {
                    destination_id: "fern_hollow".into(),
                },
            );
            let action = step(&state);
            apply(&mut state, action);
            assert_eq!(
                state.neighbourhood_projects["fern_hollow_seat"].style,
                style
            );
            let mut corrupt = state.durable_checkpoint();
            corrupt["state"]["exploration"][0]["supplies"] = json!({"planks":99});
            assert_eq!(
                State::from_durable_checkpoint(&corrupt),
                Err(Rejection::InvalidCheckpoint)
            );
            let mut corrupt = state.durable_checkpoint();
            corrupt["state"]["neighbourhoodProjects"]["fern_hollow_seat"]["style"] =
                json!("invented");
            assert_eq!(
                State::from_durable_checkpoint(&corrupt),
                Err(Rejection::InvalidCheckpoint)
            );
        }
    }

    #[test]
    fn impossible_or_farming_recipes_and_disconnected_locations_are_rejected() {
        let valid = content(false);
        assert!(valid.valid());
        for mutation in 0..4 {
            let mut invalid = valid.clone();
            match mutation {
                0 => invalid.content.steps.swap(0, 1),
                1 => invalid.content.steps[2].mechanic = Mechanic::GatherWood,
                2 => invalid.content.steps[2].location_id = Some("acorn_trail".into()),
                _ => invalid.content.complete_on_last_step = false,
            }
            assert!(!invalid.valid());
            let mut state = State::new();
            apply(
                &mut state,
                Action::Travel {
                    destination_id: "workshop".into(),
                },
            );
            reject(
                &mut state,
                Action::GeneratedTravel {
                    destination_id: "cove".into(),
                    story: invalid,
                },
            );
        }
    }
}
