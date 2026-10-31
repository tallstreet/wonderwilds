//! Arrival-time generation outside database locks. No identity or learner records
//! enter the prompt; validated results are committed with the travel receipt.
use super::*;
use wonderwilds_domain::story::exploration::{PLACES, place};
pub trait ExplorationProvider: Send + Sync {
    fn propose(&self, input: &Value) -> Result<Value, director::Error>;
}
fn brief(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let index = PLACES
        .iter()
        .position(|other| other.character == p.character)
        .unwrap();
    let visit = state.current_neighbour().map_or(0, |q| q.generation) as usize;
    let nearby = &PLACES[if index + 1 < PLACES.len() {
        index + 1
    } else {
        index - 1
    }];
    let (kind, shape, route) = match (visit + index) % 4 {
        0 => (
            "A sound mystery: trace an unexpected rhythm to its gentle source",
            "instrument",
            vec![p.location.as_str(), nearby.location.as_str()],
        ),
        1 => (
            "A nature surprise: one close observation changes a friend's understanding",
            "leaf",
            vec![p.location.as_str()],
        ),
        2 => (
            "A welcoming trail: compare landmarks in two neighbourhoods to help visitors",
            "trail",
            vec![nearby.location.as_str(), p.location.as_str()],
        ),
        _ => (
            "An evening celebration: discover how two places contribute to a shared idea",
            "stones",
            vec![
                p.location.as_str(),
                nearby.location.as_str(),
                p.location.as_str(),
            ],
        ),
    };
    Ok(json!({"kind":kind,"openingShape":shape,"route":route}))
}
pub fn input(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let previous = state.current_neighbour();
    Ok(
        json!({"area":p.name,"visitStoryNumber":previous.map_or(1, |q| q.generation + 1),
        "previousStory":previous.and_then(|q| q.story.as_ref()).map(|s| json!({"name":s.content.name,"title":s.content.title,"premise":s.content.premise,"steps":s.content.steps,"payoff":s.content.payoff})),
        "allowedLocations":PLACES.iter().map(|p| json!({"id":p.location.as_str(),"name":p.name})).collect::<Vec<_>>(),
        "storyBrief":brief(state)?,
        "capabilities":"One new character at the starting area. One to three sequential investigations, optionally walking between the connected neighbourhoods, then return to that character. Each step supplies its locationId, discovery and learningBridge. The server assesses the preparation activity. No inventory, combat, currency or invented game commands.",
        "editorialBrief":"Ages 6–8, short warm sentences, concrete clues that develop a story and a satisfying resolution. No personal data, external links, threats, manipulation or unsafe activities."}),
    )
}
pub fn fallback(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let brief = brief(state)?;
    let route = brief["route"].as_array().ok_or(Error::InvalidInput)?;
    let shape = brief["openingShape"].as_str().ok_or(Error::InvalidInput)?;
    let (title, premise, labels, discoveries, payoff) = match shape {
        "instrument" => (
            "The travelling tune",
            "A tune drifts along our paths. Follow its echoes into the next neighbourhood, then tell me where it comes from.",
            vec![
                "Listen for the little tune",
                "Find where the breeze makes music",
            ],
            vec![
                "A soft whistle answers the breeze. The same tune drifts in from the next neighbourhood.",
                "The wind hums through a hollow stem! Our mysterious musician is the breeze.",
            ],
            "Now we know the breeze's song. We can listen for it together on our walks.",
        ),
        "leaf" => (
            "A tiny umbrella",
            "I wonder why one patch stays dry after rain. Could a leaf be sheltering something small? Take a closer look and tell me.",
            vec!["Look beneath the leafy umbrella"],
            vec!["A broad leaf shelters a little dry patch. Tiny seedlings have a leafy umbrella!"],
            "We found the seedlings' tiny umbrella and left their cosy shelter undisturbed.",
        ),
        "trail" => (
            "The welcome way",
            "Visitors need landmarks they can recognise. Compare the neighbouring path with ours, then bring back an idea for welcoming them.",
            vec![
                "Notice the neighbouring path's landmark",
                "Compare the landmark beside our path",
            ],
            vec![
                "The path bends beside a leafy arch. That is an easy place for visitors to recognise.",
                "Our path has a curve too! The arch and this bend make a memorable route between friends.",
            ],
            "We can describe a friendly route: look for the arch, then follow the curving path.",
        ),
        _ => (
            "An evening of little wonders",
            "Let's imagine an evening walk for our friends. Notice a sparkle, a rustle next door and a quiet place here, then share your idea.",
            vec![
                "Notice how the smooth stones catch the light",
                "Listen to leaves along the neighbouring path",
                "Find a quiet spot for our evening walk",
            ],
            vec![
                "The smooth stones catch the soft light. Next, listen for a sound to go with their sparkle.",
                "The leaves make a gentle rustling rhythm. A quiet ending back home would complete our walk.",
                "A sheltered hollow softens the sounds. Our walk can end with a peaceful listening pause.",
            ],
            "A sparkle, a rustle and a peaceful pause: we dreamed up an evening walk to share with friends.",
        ),
    };
    let steps: Vec<_> = route.iter().enumerate().map(|(i, location)| json!({
        "label":labels[i], "shape":if i == 0 { shape } else if i == 1 { "leaf" } else { "hollow" },
        "locationId":location,"discovery":discoveries[i],
        "learningBridge":"Let's organise discovery markers to keep track of this clue."
    })).collect();
    Ok(
        json!({"name":format!("{}{}",p.character[..1].to_uppercase(),&p.character[1..]),
        "personality":"A friendly neighbour who enjoys sharing woodland discoveries.","sprite":"patter",
        "greeting":"Hello! I have something curious to share with you.",
        "title":title,"premise":premise,"steps":steps,"payoff":payoff}),
    )
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
        assert!(
            stories
                .iter()
                .any(|s| s["steps"].as_array().unwrap().len() == 1)
        );
        assert!(
            stories
                .iter()
                .any(|s| s["steps"].as_array().unwrap().len() == 3)
        );
        assert!(stories.iter().any(
            |s| s["steps"][0]["locationId"] != s["steps"][1]["locationId"]
                && s["steps"].as_array().unwrap().len() > 1
        ));
    }
}
