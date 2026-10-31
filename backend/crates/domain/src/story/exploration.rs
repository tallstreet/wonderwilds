//! Bounded, deterministic neighbourhood stories. Models may narrate these legal
//! steps but cannot invent completion, characters, rewards or travel links.
use super::{Action, Location, Rejection, State};
use serde::{Deserialize, Serialize};

pub struct Place {
    pub location: Location,
    pub name: &'static str,
    pub character: &'static str,
    pub title: &'static str,
    pub premise: &'static str,
    pub task: &'static str,
    pub payoff: &'static str,
}
pub const PLACES: [Place; 5] = [
    Place {
        location: Location::Cove,
        name: "Starlight Cove",
        character: "marina",
        title: "Marina's shell song",
        premise: "I'm Marina. The tide left a singing shell beside the path. Will you listen to it and bring its rhythm back to me?",
        task: "Listen to the singing shell",
        payoff: "Marina and the player shared the shell's gentle rhythm.",
    },
    Place {
        location: Location::FernHollow,
        name: "Fern Hollow",
        character: "fern",
        title: "Fern's rain shelter",
        premise: "I'm Fern. A tiny shelter keeps our seed baskets dry. Will you check its leafy roof before the rain?",
        task: "Check the leafy shelter",
        payoff: "Fern's seed baskets have a checked, cosy shelter.",
    },
    Place {
        location: Location::MossyClearing,
        name: "Mossy Clearing",
        character: "moss",
        title: "Moss's hidden trail",
        premise: "I'm Moss. Fallen leaves hide our trail marker. Will you find the marker so visitors can see the way?",
        task: "Find the mossy trail marker",
        payoff: "Moss and the player found the marker that welcomes visitors.",
    },
    Place {
        location: Location::OwlGrove,
        name: "Owl Grove",
        character: "orla",
        title: "Orla's evening melody",
        premise: "I'm Orla. Our grove has a new evening sound. Will you listen near the hollow tree and tell me what you discover?",
        task: "Listen beside the hollow tree",
        payoff: "Orla and the player shared the grove's evening melody.",
    },
    Place {
        location: Location::AcornTrail,
        name: "Acorn Trail",
        character: "hazel",
        title: "Hazel's acorn nook",
        premise: "I'm Hazel. I found a nook for our acorn collection. Will you check that it is dry before we settle in?",
        task: "Check the dry acorn nook",
        payoff: "Hazel's acorn collection has a dry nook beside the trail.",
    },
];
pub fn place(location: Location) -> Option<&'static Place> {
    PLACES.iter().find(|p| p.location == location)
}
pub fn connected(from: Location, to: Location) -> bool {
    let chain = [
        Location::Workshop,
        Location::Cove,
        Location::FernHollow,
        Location::MossyClearing,
        Location::OwlGrove,
        Location::AcornTrail,
    ];
    chain
        .windows(2)
        .any(|pair| pair == [from, to] || pair == [to, from])
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedContent {
    pub name: String,
    pub personality: String,
    pub sprite: String,
    pub greeting: String,
    pub title: String,
    pub premise: String,
    pub steps: Vec<GeneratedStep>,
    pub payoff: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_bridge: Option<String>,
    pub label: String,
    pub shape: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedStory {
    pub content: GeneratedContent,
    pub model_generated: bool,
}
impl GeneratedStory {
    pub fn valid(&self) -> bool {
        wonderwilds_contracts::validate("GeneratedNeighbourhood", &serde_json::json!(self.content))
            .is_ok()
            && self.content.steps.iter().enumerate().all(|(i, s)| {
                !self.content.steps[..i]
                    .iter()
                    .any(|earlier| earlier.label == s.label)
            })
    }
}
fn zero(value: &u32) -> bool {
    *value == 0
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Quest {
    pub character: String,
    pub inspected: bool,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story: Option<GeneratedStory>,
    #[serde(default, skip_serializing_if = "zero")]
    pub generation: u32,
    #[serde(default, skip_serializing_if = "zero")]
    pub step: u32,
}
impl Quest {
    pub fn task_location(&self) -> Location {
        self.story
            .as_ref()
            .and_then(|s| s.content.steps.get(self.step as usize))
            .and_then(|s| s.location_id.as_deref())
            .and_then(|id| PLACES.iter().find(|p| p.location.as_str() == id))
            .unwrap_or_else(|| {
                PLACES
                    .iter()
                    .find(|p| p.character == self.character)
                    .unwrap()
            })
            .location
    }
}
impl State {
    pub fn neighbour_for_object(&self, object: &str) -> Option<&Quest> {
        self.exploration.iter().find(|q| {
            q.status == "open"
                && !q.inspected
                && q.task_location() == self.location
                && object
                    == if q.story.is_some() {
                        format!("neighbour_{}_{}", q.character, q.step)
                    } else {
                        format!("neighbour_{}", q.character)
                    }
        })
    }

    pub(super) fn valid_exploration(&self) -> bool {
        let mut ids = std::collections::BTreeSet::new();
        self.exploration.len() <= PLACES.len()
            && self
                .exploration
                .iter()
                .filter(|q| q.status == "open")
                .count()
                <= 3
            && self.exploration.iter().all(|q| {
                ids.insert(&q.character)
                    && PLACES.iter().any(|p| p.character == q.character)
                    && matches!(
                        q.status.as_str(),
                        "offered" | "open" | "archived" | "completed"
                    )
                    && (q.status != "completed" || q.inspected)
                    && (q.status != "offered" || q.story.is_some() && q.step == 0)
                    && q.story.as_ref().is_none_or(|s| {
                        q.generation > 0
                            && u64::from(q.generation) <= self.revision
                            && s.valid()
                            && q.step as usize <= s.content.steps.len()
                            && q.inspected == (q.step as usize == s.content.steps.len())
                    })
            })
    }
    pub fn current_neighbour(&self) -> Option<&Quest> {
        let p = place(self.location)?;
        self.exploration.iter().find(|q| q.character == p.character)
    }
    pub fn needs_generated_story(&self) -> bool {
        place(self.location).is_some()
            && self
                .current_neighbour()
                .is_none_or(|q| q.status == "completed")
    }
    pub(super) fn install_generated_story(
        &mut self,
        story: &GeneratedStory,
    ) -> Result<(), Rejection> {
        if !self.needs_generated_story() || !story.valid() {
            return Err(Rejection::NotAvailable);
        }
        let p = place(self.location).ok_or(Rejection::NotAvailable)?;
        let previous = self
            .exploration
            .iter()
            .position(|q| q.character == p.character);
        let generation = previous
            .map_or(0, |i| self.exploration[i].generation)
            .checked_add(1)
            .ok_or(Rejection::LimitReached)?;
        if let Some(index) = previous {
            self.exploration.remove(index);
        }
        self.exploration.push(Quest {
            character: p.character.into(),
            inspected: false,
            status: "offered".into(),
            story: Some(story.clone()),
            generation,
            step: 0,
        });
        Ok(())
    }
    pub fn exploration_quests(&self) -> &[Quest] {
        &self.exploration
    }
    pub(super) fn exploration_actions(&self) -> Vec<Action> {
        let mut actions: Vec<_> = PLACES
            .iter()
            .map(|p| Action::Travel {
                destination_id: p.location.as_str().into(),
            })
            .collect();
        // The original three travel candidates already include the workshop.
        if let Some(p) = place(self.location) {
            for topic in ["start_neighbour_story", "finish_neighbour_story"] {
                actions.push(Action::Talk {
                    character_id: p.character.into(),
                    topic_id: topic.into(),
                });
            }
            actions.push(Action::Inspect {
                object_id: if let Some(q) = self.current_neighbour().filter(|q| q.story.is_some()) {
                    format!("neighbour_{}_{}", p.character, q.step)
                } else {
                    format!("neighbour_{}", p.character)
                },
            });
        }
        for q in &self.exploration {
            if q.status == "open"
                && !q.inspected
                && q.story.is_some()
                && q.task_location() == self.location
                && place(self.location).is_none_or(|p| p.character != q.character)
            {
                actions.push(Action::Inspect {
                    object_id: format!("neighbour_{}_{}", q.character, q.step),
                });
            }
        }
        actions
    }
    pub(super) fn exploration_transition(
        &mut self,
        action: &Action,
    ) -> Option<Result<String, Rejection>> {
        let (character, finish, inspect) = match action {
            Action::Talk {
                character_id,
                topic_id,
            } if topic_id == "start_neighbour_story" || topic_id == "finish_neighbour_story" => (
                character_id.as_str(),
                topic_id == "finish_neighbour_story",
                false,
            ),
            Action::Inspect { object_id } if object_id.starts_with("neighbour_") => {
                (object_id[10..].split('_').next().unwrap_or(""), false, true)
            }
            _ => return None,
        };
        Some((|| {
            let p = PLACES
                .iter()
                .find(|p| p.character == character)
                .ok_or(Rejection::NotAvailable)?;
            if !inspect && self.location != p.location {
                return Err(Rejection::NotAvailable);
            }
            let index = self
                .exploration
                .iter()
                .position(|q| q.character == character);
            if inspect || finish {
                let q = index
                    .map(|i| &mut self.exploration[i])
                    .ok_or(Rejection::NotAvailable)?;
                if q.status != "open" || (finish && !q.inspected) || (inspect && q.inspected) {
                    return Err(Rejection::NotAvailable);
                }
                if inspect && q.task_location() != self.location {
                    return Err(Rejection::NotAvailable);
                }
                if inspect && let Some(story) = &q.story {
                    if !matches!(action, Action::Inspect { object_id } if *object_id == format!("neighbour_{}_{}", p.character,q.step))
                    {
                        return Err(Rejection::NotAvailable);
                    }
                    let discovery = story.content.steps[q.step as usize].discovery.clone();
                    q.step += 1;
                    q.inspected = q.step as usize == story.content.steps.len();
                    return Ok(discovery.unwrap_or_else(|| {
                        format!(
                            "Completed step {} of {}.",
                            q.step,
                            story.content.steps.len()
                        )
                    }));
                }
                if finish {
                    q.status = "completed".into();
                    Ok(q.story
                        .as_ref()
                        .map_or(p.payoff, |s| s.content.payoff.as_str())
                        .into())
                } else {
                    q.inspected = true;
                    Ok(format!(
                        "{} is done. Return to {} to share the discovery.",
                        p.task, p.character
                    ))
                }
            } else {
                if index.is_some_and(|i| {
                    !["archived", "offered"].contains(&self.exploration[i].status.as_str())
                }) {
                    return Err(Rejection::NotAvailable);
                }
                // Reserve one slot for Pip, including paused/offered stories. Accepting
                // or resuming Pip can never push the total over four.
                let archived = if self
                    .exploration
                    .iter()
                    .filter(|q| q.status == "open")
                    .count()
                    >= 3
                {
                    let oldest = self
                        .exploration
                        .iter_mut()
                        .find(|q| q.status == "open")
                        .unwrap();
                    oldest.status = "archived".into();
                    Some(oldest.character.clone())
                } else {
                    None
                };
                let mut quest = index.map(|i| self.exploration.remove(i)).unwrap_or(Quest {
                    character: character.into(),
                    inspected: false,
                    status: "open".into(),
                    story: None,
                    generation: 0,
                    step: 0,
                });
                quest.status = "open".into();
                let generated_title = quest.story.as_ref().map(|s| s.content.title.clone());
                self.exploration.push(quest);
                if let Some(title) = generated_title {
                    return Ok(format!(
                        "Started {title}.{}",
                        archived.map_or(String::new(), |name| format!(
                            " Archived {name}'s older mission; visit them to resume."
                        ))
                    ));
                }
                Ok(format!("Started {}.{}", p.title, archived.map_or(String::new(), |name| format!(" {name}'s older mission was archived with its progress saved; visit them to resume."))))
            }
        })())
    }
}
