//! Arrival-time generation outside database locks. No identity or learner records
//! enter the prompt; validated results are committed with the travel receipt.
use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use wonderwilds_domain::story::exploration::{PLACES, place};
mod seeds;
use seeds::{PROJECTS, SEEDS, Seed};

pub trait ExplorationProvider: Send + Sync {
    fn propose(&self, input: &Value) -> Result<Value, director::Error>;
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}
fn fingerprint(value: &Value) -> String {
    let normalized = words(&value.to_string()).join(" ");
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}
fn motifs(text: &str) -> BTreeSet<String> {
    words(text)
        .into_iter()
        .filter(|word| {
            word.len() >= 5
                && ![
                    "their",
                    "there",
                    "these",
                    "those",
                    "where",
                    "which",
                    "could",
                    "would",
                    "about",
                    "along",
                    "after",
                    "before",
                    "nearby",
                    "neighbour",
                    "friend",
                    "little",
                    "discovery",
                    "notice",
                    "compare",
                    "discover",
                    "shared",
                ]
                .contains(&word.as_str())
        })
        .collect()
}

// Only the latest saved story in each area is available here. Their insertion
// order is generation order, so the last one is the most recently generated.
fn selected(state: &State) -> Result<(&'static Seed, Vec<&'static str>), Error> {
    let home = PLACES
        .iter()
        .position(|p| p.location == state.location())
        .ok_or(Error::InvalidInput)?;
    let stories: Vec<_> = state
        .exploration_quests()
        .iter()
        .filter_map(|q| q.story.as_ref())
        .collect();
    let total: u64 = state
        .exploration_quests()
        .iter()
        .map(|q| u64::from(q.generation))
        .sum();
    // Most visits offer hands-on projects, interspersed with discovery stories.
    let candidates = if total % 4 == 3 { SEEDS } else { PROJECTS };
    let start = (total as usize + home) % candidates.len();
    let seed = (0..candidates.len())
        .map(|offset| &candidates[(start + offset) % candidates.len()])
        .min_by_key(|seed| {
            let terms = motifs(&format!("{} {} {}", seed.kind, seed.wish, seed.title));
            stories
                .iter()
                .enumerate()
                .map(|(i, story)| {
                    let content = &story.content;
                    let overlap = terms
                        .intersection(&motifs(&format!(
                            "{} {} {}",
                            content.title, content.premise, content.payoff
                        )))
                        .count();
                    let same_opening = content
                        .steps
                        .first()
                        .is_some_and(|s| s.shape == seed.steps[0].0);
                    usize::from(
                        fingerprint(&json!(content.title)) == fingerprint(&json!(seed.title)),
                    ) * 1000
                        + usize::from(content.steps.last().is_some_and(|s| {
                            serde_json::to_value(s.mechanic)
                                .ok()
                                .and_then(|v| v.as_str().map(str::to_owned))
                                .as_deref()
                                == seed.mechanics.last().copied()
                        })) * 15
                        + overlap * 3
                        + usize::from(same_opening) * if i + 1 == stories.len() { 30 } else { 2 }
                })
                .sum::<usize>()
        })
        .unwrap();
    // Alternate both sides of interior areas. Never route outside the map.
    let right = home == 0 || (home + 1 < PLACES.len() && (total + home as u64).is_multiple_of(2));
    let neighbour = if right { home + 1 } else { home - 1 };
    let mut route = vec![PLACES[home].location.as_str(); seed.steps.len()];
    if seed.mechanics.is_empty() {
        if route.len() > 1 {
            route[1] = PLACES[neighbour].location.as_str();
        }
    } else {
        // Local preparation followed by a useful outcome at the destination.
        *route.last_mut().unwrap() = PLACES[neighbour].location.as_str();
    }
    Ok((seed, route))
}

fn brief(state: &State) -> Result<Value, Error> {
    let (seed, route) = selected(state)?;
    Ok(
        json!({"kind":seed.kind,"characterWish":seed.wish,"storyStructure":seed.structure,
        "openingShape":seed.steps[0].0,"route":route,"stepCount":route.len(),
        "mechanics":if seed.mechanics.is_empty() { vec!["inspect";seed.steps.len()] } else { seed.mechanics.to_vec() },
        "completeOnLastStep":!seed.mechanics.is_empty()}),
    )
}

pub fn input(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let recent: Vec<_> = state.exploration_quests().iter().filter_map(|q| {
        let content = &q.story.as_ref()?.content;
        Some(json!({"title":content.title,
            "motifs":motifs(&format!("{} {}", content.premise, content.payoff)).into_iter().take(16).collect::<Vec<_>>(),
            "openingShape":content.steps.first().map(|s| &s.shape),
            "route":content.steps.iter().map(|s| &s.location_id).collect::<Vec<_>>()}))
    }).collect();
    // Server-only equality checks. Removed before constructing the provider body:
    // the model gets compact avoidance hints, never a whole story to copy.
    let checks: Vec<_> = state
        .exploration_quests()
        .iter()
        .filter_map(|q| {
            let content = &q.story.as_ref()?.content;
            Some(json!({"title":fingerprint(&json!(content.title)),
            "premise":fingerprint(&json!(content.premise)),
            "payoff":fingerprint(&json!(content.payoff)),
            "steps":fingerprint(&json!(content.steps))}))
        })
        .collect();
    Ok(
        json!({"area":p.name,"visitStoryNumber":state.current_neighbour().map_or(1, |q| q.generation + 1),
        "recentStories":recent,"noveltyChecks":checks,
        "allowedLocations":PLACES.iter().map(|p| json!({"id":p.location.as_str(),"name":p.name})).collect::<Vec<_>>(),
        "storyBrief":brief(state)?,
        "capabilities":"One character offers a plan with fixed supported mechanics, recipes and locations. Follow the supplied per-step mechanics. Project supplies are reserved for this mission; gathering, crafting, delivery, building and creative style choices have real server-owned effects. Projects finish at the destination without returning. No invented items, recipes, rewards, escorts or controls.",
        "editorialBrief":"Ages 6–8, short warm sentences, concrete clues that develop a story and a satisfying resolution. No personal data, external links, threats, manipulation or unsafe activities."}),
    )
}

/// Fresh-generation policy only: historical saves keep their existing reader.
/// This is a mechanical freshness check, not editorial or educational approval.
pub(super) fn validate_proposal(input: &Value, proposal: &Value) -> Result<(), Error> {
    let steps = proposal["steps"].as_array().ok_or(Error::InvalidInput)?;
    let route = input["storyBrief"]["route"]
        .as_array()
        .ok_or(Error::InvalidInput)?;
    if steps.len() != route.len()
        || steps.first().map(|s| &s["shape"]) != Some(&input["storyBrief"]["openingShape"])
        || steps
            .iter()
            .zip(route)
            .any(|(step, location)| &step["locationId"] != location)
        || proposal["completeOnLastStep"].as_bool().unwrap_or(false)
            != input["storyBrief"]["completeOnLastStep"]
                .as_bool()
                .unwrap_or(false)
        || steps.iter().enumerate().any(|(i, step)| {
            step["mechanic"].as_str().unwrap_or("inspect")
                != input["storyBrief"]["mechanics"][i]
                    .as_str()
                    .unwrap_or("inspect")
        })
        || input["noveltyChecks"].as_array().is_some_and(|checks| {
            checks.iter().any(|previous| {
                ["title", "premise", "payoff", "steps"].iter().any(|field| {
                    previous[*field].as_str() == Some(fingerprint(&proposal[*field]).as_str())
                })
            })
        })
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn fallback(state: &State) -> Result<Value, Error> {
    let (seed, route) = selected(state)?;
    fallback_for(state, seed, &route)
}

fn fallback_for(state: &State, seed: &Seed, route: &[&str]) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let bridges = [
        "Let's prepare to keep track of this clue.",
        "Let's get ready to compare what we notice.",
        "Let's prepare for a closer look at the discovery.",
        "Let's get ready to explore what this clue could mean.",
        "Let's prepare before we share the next discovery.",
    ];
    let bridge_start = SEEDS
        .iter()
        .position(|candidate| candidate.title == seed.title)
        .unwrap_or(0);
    let steps: Vec<_> = seed
        .steps
        .iter()
        .zip(route)
        .enumerate()
        .map(|(i, ((shape, label, discovery), location))| {
            let destination = PLACES
                .iter()
                .find(|p| p.location.as_str() == *location)
                .unwrap();
            let mut step = json!({"label":format!("{}: {}",destination.name,label),"shape":shape,
            "locationId":location,"discovery":discovery,"learningBridge":bridges[(bridge_start + i) % bridges.len()]});
            if let Some(mechanic) = seed.mechanics.get(i) { step["mechanic"] = json!(mechanic); }
            step
        })
        .collect();
    let mut content = json!({"name":format!("{}{}",p.character[..1].to_uppercase(),&p.character[1..]),
        "personality":"A curious neighbour with a particular woodland question to explore.","sprite":"patter",
        "greeting":"Hello! Could you help me understand something surprising?",
        "title":seed.title,"premise":seed.premise,"steps":steps,"payoff":seed.payoff});
    if !seed.mechanics.is_empty() {
        content["completeOnLastStep"] = json!(true);
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offline_stories_vary_in_length_shape_and_route_and_validate() {
        let mut stories = Vec::new();
        for p in &PLACES {
            let mut state = State::new();
            for id in std::iter::once("workshop").chain(PLACES.iter().map(|p| p.location.as_str()))
            {
                let command = wonderwilds_domain::story::Command {
                    command_id: format!("travel-{id}"),
                    expected_revision: state.revision(),
                    action: Action::Travel {
                        destination_id: id.into(),
                    },
                };
                state.apply(&command).unwrap();
                if id == p.location.as_str() {
                    break;
                }
            }
            let story = fallback(&state).unwrap();
            wire::validate("GeneratedNeighbourhood", &story).unwrap();
            assert!(
                input(&state).unwrap()["allowedLocations"]
                    .as_array()
                    .unwrap()
                    .len()
                    == 5
            );
            stories.push(story);
        }
        assert!(stories.iter().all(|s| s["completeOnLastStep"] == true));
        assert!(
            stories
                .iter()
                .any(|s| s["steps"].as_array().unwrap().len() == 4)
        );
        assert!(
            stories
                .iter()
                .any(|s| s["steps"].as_array().unwrap().len() == 3)
        );
        assert!(stories.iter().any(|s| s["steps"][0]["locationId"]
            != s["steps"].as_array().unwrap().last().unwrap()["locationId"]
            && s["steps"].as_array().unwrap().len() > 1));
    }

    fn apply(state: &mut State, action: Action) {
        state
            .apply(&Command {
                command_id: format!("variety-{}", state.revision()),
                expected_revision: state.revision(),
                action,
            })
            .unwrap();
    }

    fn travel(state: &mut State, destination: &str) {
        let chain = [
            "bridge_bank",
            "workshop",
            "cove",
            "fern_hollow",
            "mossy_clearing",
            "owl_grove",
            "acorn_trail",
        ];
        while state.location().as_str() != destination {
            let from = chain
                .iter()
                .position(|id| *id == state.location().as_str())
                .unwrap();
            let to = chain.iter().position(|id| *id == destination).unwrap();
            let next = if from < to { from + 1 } else { from - 1 };
            apply(
                state,
                Action::Travel {
                    destination_id: chain[next].into(),
                },
            );
        }
    }

    #[test]
    fn every_offline_arc_fits_the_contract_in_every_area() {
        for p in &PLACES {
            let mut state = State::new();
            travel(&mut state, p.location.as_str());
            for seed in SEEDS.iter().chain(PROJECTS) {
                let route = vec![p.location.as_str(); seed.steps.len()];
                let story = fallback_for(&state, seed, &route).unwrap();
                wire::validate("GeneratedNeighbourhood", &story)
                    .unwrap_or_else(|error| panic!("{} in {}: {error:?}", seed.title, p.name));
                let generated = wonderwilds_domain::story::exploration::GeneratedStory {
                    content: serde_json::from_value(story).unwrap(),
                    model_generated: false,
                };
                assert!(generated.valid(), "{} in {}", seed.title, p.name);
            }
        }
    }

    #[test]
    fn repeat_visits_have_varied_playable_stable_stories_and_cross_area_memory() {
        let mut state = State::new();
        travel(&mut state, "workshop");
        let mut titles = BTreeSet::new();
        let mut lengths = BTreeSet::new();
        let mut openings = BTreeSet::new();
        for cycle in 0..16 {
            let home = if cycle < 8 {
                "cove"
            } else {
                PLACES[(cycle - 8) % PLACES.len()].location.as_str()
            };
            // Stop one area short of home so the enriched arrival is real travel.
            travel(
                &mut state,
                if home == "cove" {
                    "workshop"
                } else {
                    PLACES
                        .iter()
                        .position(|p| p.location.as_str() == home)
                        .map(|i| PLACES[i - 1].location.as_str())
                        .unwrap()
                },
            );
            let mut arrived = state.clone();
            apply(
                &mut arrived,
                Action::Travel {
                    destination_id: home.into(),
                },
            );
            let request = input(&arrived).unwrap();
            assert!(request.get("previousStory").is_none());
            let content = fallback(&arrived).unwrap();
            assert_eq!(content, fallback(&arrived).unwrap());
            let restored = State::from_durable_checkpoint(&arrived.durable_checkpoint()).unwrap();
            assert_eq!(request, input(&restored).unwrap());
            assert_eq!(content, fallback(&restored).unwrap());
            validate_proposal(&request, &content).unwrap();
            wire::validate("GeneratedNeighbourhood", &content).unwrap();
            titles.insert(content["title"].as_str().unwrap().to_owned());
            lengths.insert(content["steps"].as_array().unwrap().len());
            openings.insert(content["steps"][0]["shape"].as_str().unwrap().to_owned());
            let story = wonderwilds_domain::story::exploration::GeneratedStory {
                content: serde_json::from_value(content.clone()).unwrap(),
                model_generated: false,
            };
            apply(
                &mut state,
                Action::GeneratedTravel {
                    destination_id: home.into(),
                    story,
                },
            );
            let character = place(state.location()).unwrap().character;
            apply(
                &mut state,
                Action::Talk {
                    character_id: character.into(),
                    topic_id: "start_neighbour_story".into(),
                },
            );
            for (i, step) in content["steps"].as_array().unwrap().iter().enumerate() {
                travel(&mut state, step["locationId"].as_str().unwrap());
                let action = state
                    .exploration_quests()
                    .iter()
                    .find(|q| q.character == character)
                    .unwrap()
                    .step_actions()[0]
                    .clone();
                assert!(
                    state.available_actions().contains(&action),
                    "step {i}: {action:?}"
                );
                apply(&mut state, action);
            }
            if content["completeOnLastStep"] != true {
                travel(&mut state, home);
                apply(
                    &mut state,
                    Action::Talk {
                        character_id: character.into(),
                        topic_id: "finish_neighbour_story".into(),
                    },
                );
            }
            let q = state
                .exploration_quests()
                .iter()
                .find(|q| q.character == character)
                .unwrap();
            assert_eq!(q.status, "completed");
            assert!(q.supplies.is_empty());
            if content["completeOnLastStep"] == true {
                assert_ne!(
                    state.location().as_str(),
                    home,
                    "project finishes without return"
                );
                assert!(!state.neighbourhood_projects().is_empty());
            }
            state = State::from_durable_checkpoint(&state.durable_checkpoint()).unwrap();
            assert_eq!(
                state
                    .exploration_quests()
                    .iter()
                    .find(|q| q.character == character)
                    .unwrap()
                    .status,
                "completed"
            );
            if cycle > 10 {
                assert!(
                    input(&state).unwrap()["recentStories"]
                        .as_array()
                        .unwrap()
                        .len()
                        > 1
                );
            }
        }
        assert!(
            titles.len() >= 8,
            "Only {} different stories: {titles:?}",
            titles.len()
        );
        assert!(lengths.contains(&3) && lengths.contains(&4));
        assert!(lengths.contains(&1) || lengths.contains(&2));
        assert!(openings.len() >= 4);
    }

    #[test]
    fn fresh_policy_rejects_copied_fields_and_wrong_routes_without_changing_old_saves() {
        let mut state = State::new();
        travel(&mut state, "cove");
        let request = input(&state).unwrap();
        let content = fallback(&state).unwrap();
        validate_proposal(&request, &content).unwrap();
        for field in ["title", "premise", "payoff", "steps"] {
            let mut request = request.clone();
            request["noveltyChecks"] = json!([{field: fingerprint(&content[field])}]);
            assert!(
                validate_proposal(&request, &content).is_err(),
                "copied {field}"
            );
        }
        let mut copied_title = request.clone();
        copied_title["noveltyChecks"] = json!([{"title":fingerprint(&json!(format!("{}!!!", content["title"].as_str().unwrap().to_uppercase())))}]);
        assert!(validate_proposal(&copied_title, &content).is_err());
        for field in ["shape", "locationId"] {
            let mut wrong = content.clone();
            wrong["steps"][0][field] = json!(if field == "shape" {
                "trail"
            } else {
                "acorn_trail"
            });
            wire::validate("GeneratedNeighbourhood", &wrong).unwrap();
            assert!(validate_proposal(&request, &wrong).is_err());
        }
        let mut wrong = content.clone();
        wrong["steps"].as_array_mut().unwrap().pop();
        assert!(validate_proposal(&request, &wrong).is_err());
        // Validation of already-saved content still uses the compatible contract.
        wire::validate("GeneratedNeighbourhood", &content).unwrap();
    }
}
