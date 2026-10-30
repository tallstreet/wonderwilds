//! Local stand-in for a scene provider. No network, randomness, or state writes.
use super::*;

/// Providers receive compact input and a canonical fallback containing revision-bound
/// presentation IDs. They cannot execute actions or alter mission progress.
pub trait SceneProvider {
    fn propose(&self, input: &Value, canonical: &Value) -> Result<Value, Error>;
}

pub struct MockSceneProvider;

fn count(input: &Value, item: &str) -> u64 {
    input["context"]["inventory"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["itemId"] == item)
        .and_then(|i| i["quantity"].as_u64())
        .unwrap_or(0)
}

/// The compact v1 input carries introduction evidence as an authored memory fact,
/// not a dedicated flag. Keep this exact mapping in one place and test it against
/// the builder; scene prose and relationship descriptions are not evidence.
pub(super) fn needs_introduction(input: &Value) -> bool {
    input["context"]["locationId"] == "workshop"
        && input["here"]["characters"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == "woodcutter")
        && !input["context"]["memory"]["facts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|fact| fact == "The game has presented the woodcutter's introduction.")
}

impl SceneProvider for MockSceneProvider {
    fn propose(&self, input: &Value, canonical: &Value) -> Result<Value, Error> {
        checked("StorySceneInput", input.clone())?;
        checked("StoryScene", canonical.clone())?;
        if exploration::PLACES
            .iter()
            .any(|p| input["context"]["locationId"] == p.location.as_str())
        {
            return Ok(canonical.clone());
        }
        if matches!(
            input["mission"]["missionId"].as_str(),
            Some("pips_picnic" | "welcome_lights")
        ) {
            return sequel_scene(input, canonical);
        }
        if matches!(
            input["mission"]["missionId"].as_str(),
            Some("humming_hollow" | "cosy_corner" | "brambles_visit" | "moonlight_gathering")
        ) {
            return super::broader::scene(input, canonical);
        }
        let actions = input["availableActions"].as_array().unwrap();
        let has = |kind: &str| actions.iter().any(|a| a["action"]["type"] == kind);
        let location = input["context"]["locationId"].as_str().unwrap();
        let status = input["mission"]["status"].as_str();
        let introduction = needs_introduction(input);
        let bridge_ready = input["context"]["memory"]["facts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|fact| fact == "The bridge is repaired.");
        let wood = count(input, "wood");
        let wood_prompt = if wood == 0 {
            "Let's look for loose wood nearby. We need three pieces to make planks for the bridge."
                .to_owned()
        } else {
            format!(
                "You have {wood} {} of wood. Find {} more nearby to make the bridge planks.",
                if wood == 1 { "piece" } else { "pieces" },
                3_u64.saturating_sub(wood)
            )
        };
        let (stage, text) = if introduction {
            (
                "workshop_intro",
                "The workshop is full of tools for shaping wood. The woodcutter is ready to help.",
            )
        } else if status == Some("abandoned") {
            (
                "paused",
                "There is time to explore. Your supplies and bridge progress will be here when you help Pip again.",
            )
        } else if status == Some("completed") {
            (
                "completed",
                "You and Pip have crossed the stream together. The repaired bridge is ready for more exploring.",
            )
        } else if input["mission"].is_null() {
            (
                "bridge_offer",
                if bridge_ready {
                    "The bridge is already repaired. Pip would like to cross it with you."
                } else {
                    "Pip wants to explore across the stream. You can help make a way across together."
                },
            )
        } else if has("accompany") {
            (
                "shared_crossing",
                "The bridge is ready. Pip is waiting beside you to cross together.",
            )
        } else if has("assemble") {
            (
                "repair",
                "The planks are here at the bridge. You can fit them into place to make a safe way across.",
            )
        } else if location == "workshop" && count(input, "planks") >= 3 {
            (
                "return",
                "The planks are ready. Bring them back to the bridge to help Pip.",
            )
        } else if has("exchange") {
            (
                "exchange",
                "You have enough wood. The woodcutter can shape it into three sturdy planks for the bridge.",
            )
        } else if count(input, "wood") >= 3 {
            (
                "gather",
                "You have enough wood for the bridge. Take it to the woodcutter at the workshop.",
            )
        } else if has("collect") {
            ("gather", wood_prompt.as_str())
        } else {
            (
                "explore",
                "The paths are open to explore. You can return to the bridge to help Pip.",
            )
        };
        let mut response = canonical.clone();
        let present = input["here"]["characters"].as_array().unwrap();
        let pip_here = present.iter().any(|c| c["id"] == "pip");
        let woodcutter_here = present.iter().any(|c| c["id"] == "woodcutter");
        let dialogue = match stage {
            "bridge_offer" if pip_here => Some((
                "pip",
                if bridge_ready {
                    "The bridge is ready! Will you cross the stream with me?"
                } else {
                    "I'd love to explore across the stream. Will you help me make a way across?"
                },
            )),
            "gather" if pip_here && count(input, "wood") >= 3 => Some((
                "pip",
                "You found enough wood! Take it to the woodcutter at the workshop to make planks.",
            )),
            "bridge_offer" if woodcutter_here => Some((
                "woodcutter",
                if bridge_ready {
                    "The bridge is ready for crossing. Pip is waiting at the bridge bank."
                } else {
                    "Pip is waiting at the bridge bank. You can help him cross the stream."
                },
            )),
            "gather" if pip_here => Some(("pip", wood_prompt.as_str())),
            "gather" if woodcutter_here => Some(("woodcutter", wood_prompt.as_str())),
            "repair" if pip_here => Some((
                "pip",
                "The planks are here! Let's fit them into place to make a safe way across.",
            )),
            "shared_crossing" if pip_here => {
                Some(("pip", "The bridge is ready! Let's cross it together."))
            }
            "completed" if pip_here => Some((
                "pip",
                "We made it across together! Shall we prepare a picnic next?",
            )),
            "paused" if pip_here => Some((
                "pip",
                "Take your time exploring. I'll be here when you want to help again.",
            )),
            "paused" if woodcutter_here => Some((
                "woodcutter",
                "Enjoy exploring. Your supplies will be ready when you want to help Pip again.",
            )),
            "exchange" if woodcutter_here => Some((
                "woodcutter",
                "You have enough wood. I can shape it into three sturdy planks for the bridge.",
            )),
            "return" if woodcutter_here => Some((
                "woodcutter",
                "These planks are ready. Take them back to Pip at the bridge.",
            )),
            _ => None,
        };
        let entry = if let Some((speaker, dialogue)) = dialogue {
            json!({"type":"dialogue","entryId":stage,"speakerId":speaker,"text":dialogue})
        } else {
            json!({"type":"narration","entryId":stage,"text":text})
        };
        response["presentation"] = if introduction || (dialogue.is_none() && !present.is_empty()) {
            // Keep the exact introduction, or a truthful canonical conversation for
            // present NPCs in detours not covered by a stage-specific dialogue.
            json!([canonical["presentation"][0], entry])
        } else {
            json!([entry])
        };
        Ok(response)
    }
}

fn sequel_scene(input: &Value, canonical: &Value) -> Result<Value, Error> {
    let picnic = input["mission"]["missionId"] == "pips_picnic";
    let status = input["mission"]["status"].as_str().unwrap();
    let actions = input["availableActions"].as_array().unwrap();
    let has = |kind: &str| actions.iter().any(|a| a["action"]["type"] == kind);
    let facts = input["context"]["memory"]["facts"].as_array().unwrap();
    let berries = count(input, "berries");
    let berry_prompt = if berries == 0 {
        "Find three berries on the far bank, then take them to the workshop for our picnic."
            .to_owned()
    } else {
        format!(
            "You have {berries} {}. Find {} more on the far bank, then take them to the workshop for our picnic.",
            if berries == 1 { "berry" } else { "berries" },
            3_u64.saturating_sub(berries)
        )
    };
    let pebbles = count(input, "glow_pebbles");
    let pebble_prompt = if pebbles == 0 {
        "Find three glow pebbles on the far bank so we can make welcoming lanterns.".to_owned()
    } else {
        format!(
            "You have {pebbles} glow {}. Find {} more on the far bank so we can make welcoming lanterns.",
            if pebbles == 1 { "pebble" } else { "pebbles" },
            3_u64.saturating_sub(pebbles)
        )
    };
    let (stage, text) = if status == "completed" {
        if picnic {
            (
                "picnic_completed",
                "The picnic was lovely! Shall we make lanterns to welcome friends on both banks?",
            )
        } else {
            (
                "lights_completed",
                "Both banks are glowing! The bridge, the picnic, and the welcome lights are all ready.",
            )
        }
    } else if status == "abandoned" {
        (
            "paused",
            "Take your time exploring. Your supplies and the things we have made will be here when you return.",
        )
    } else if picnic {
        if has("accompany") {
            (
                "picnic_share",
                "The table is ready. Let's cross back together and share the picnic.",
            )
        } else if has("assemble") {
            (
                "picnic_prepare",
                "The basket is ready. Set it on the picnic table beside the bridge.",
            )
        } else if facts.iter().any(|f| f == "The picnic table is ready.") {
            (
                "picnic_fetch_pip",
                "The picnic table is ready. Go to the far bank to bring Pip back to share it.",
            )
        } else if count(input, "picnic_basket") > 0 {
            (
                "picnic_return",
                "The picnic basket is packed. Bring it to the table on the near bank.",
            )
        } else if count(input, "berries") >= 3 {
            (
                "picnic_pack",
                "These berries are ready for the picnic. The woodcutter can pack them into a basket at the workshop.",
            )
        } else {
            ("picnic_gather", berry_prompt.as_str())
        }
    } else if has("assemble") {
        (
            "lights_install",
            if facts
                .iter()
                .any(|f| f == "Two lanterns light the near bank.")
            {
                "The near bank is already lit. Place the last lantern here on the far bank."
            } else if facts
                .iter()
                .any(|f| f == "One lantern lights the far bank.")
            {
                "The far bank is already lit. Place the remaining two lanterns here on the near bank."
            } else {
                "Place the lanterns here to make the path welcoming. Both banks need their lights."
            },
        )
    } else if count(input, "lanterns") > 0 {
        let message = if facts
            .iter()
            .any(|f| f == "Two lanterns light the near bank.")
        {
            "The near bank is lit. Bring the remaining lantern to the far bank."
        } else if facts
            .iter()
            .any(|f| f == "One lantern lights the far bank.")
        {
            "The far bank is lit. Bring the remaining two lanterns to the near bank."
        } else {
            "Carry the lanterns to the banks. Place two on the near bank and one on the far bank."
        };
        ("lights_carry", message)
    } else if count(input, "glow_pebbles") >= 3 {
        (
            "lights_make",
            "Take these glow pebbles to the workshop. The woodcutter can make three lanterns.",
        )
    } else {
        ("lights_gather", pebble_prompt.as_str())
    };
    let mut response = canonical.clone();
    let present = input["here"]["characters"].as_array().unwrap();
    let mut entries = Vec::new();
    if needs_introduction(input) {
        entries.push(canonical["presentation"][0].clone());
    }
    for character in present {
        if needs_introduction(input) && character["id"] == "woodcutter" {
            continue;
        }
        let line = match (stage, character["id"].as_str().unwrap()) {
            ("picnic_pack", "woodcutter") => {
                "You have three berries. I can pack them into a picnic basket."
            }
            ("picnic_gather", "woodcutter") => berry_prompt.as_str(),
            ("lights_make", "woodcutter") => {
                "I can turn your three glow pebbles into three welcoming lanterns."
            }
            ("lights_gather", "woodcutter") => pebble_prompt.as_str(),
            _ => text,
        };
        entries.push(json!({"type":"dialogue","entryId":format!("{stage}-{}",character["id"].as_str().unwrap()),"speakerId":character["id"],"text":line}));
    }
    if entries.is_empty() || needs_introduction(input) {
        entries.push(json!({"type":"narration","entryId":stage,"text":text}));
    }
    response["presentation"] = json!(entries);
    Ok(response)
}

/// Structural validation plus the runtime bindings that JSON Schema cannot express.
/// This is not a general natural-language truth/safety validator for future models.
pub fn validate_scene(input: &Value, canonical: &Value, proposal: &Value) -> Result<(), Error> {
    checked("StorySceneInput", input.clone())?;
    checked("StoryScene", canonical.clone())?;
    checked("StoryScene", proposal.clone())?;
    let reject = || Error::InvalidPayload("Scene does not match current story bindings".into());
    if proposal["sceneId"] != canonical["sceneId"] {
        return Err(reject());
    }
    let actions = input["availableActions"].as_array().unwrap();
    let choices = proposal["choices"].as_array().unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for choice in choices {
        let handle = choice["actionId"].as_str().unwrap();
        if !seen.insert(handle) || !actions.iter().any(|a| a["id"] == handle) {
            return Err(reject());
        }
    }
    // Keep all current routes available; a mock response must not strand the player.
    if seen.len() != actions.len() {
        return Err(reject());
    }
    let mut entries = std::collections::BTreeSet::new();
    for entry in proposal["presentation"].as_array().unwrap() {
        if !entries.insert(entry["entryId"].as_str().unwrap()) {
            return Err(reject());
        }
        if entry["type"] == "dialogue"
            && !input["here"]["characters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == entry["speakerId"])
        {
            return Err(reject());
        }
    }
    if needs_introduction(input)
        && !proposal["presentation"]
            .as_array()
            .unwrap()
            .contains(&canonical["presentation"][0])
    {
        return Err(reject());
    }
    Ok(())
}

/// Invalid or failed proposals fall back to the canonical authored scene. Both paths
/// preserve the complete legal action list and deterministic revision metadata.
pub fn scene_with_provider(state: &State, provider: &impl SceneProvider) -> Result<Value, Error> {
    let input = build_scene_input(state)?;
    let canonical = super::canonical_scene(state, &input)?;
    match provider.propose(&input, &canonical) {
        Ok(proposal) if validate_scene(&input, &canonical, &proposal).is_ok() => Ok(proposal),
        _ => Ok(canonical),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wonderwilds_domain::story::Command;

    #[test]
    fn introduction_mapping_uses_builder_evidence_not_descriptive_prose() {
        let mut state = State::new();
        state
            .apply(&Command {
                command_id: "travel".into(),
                expected_revision: 0,
                action: Action::Travel {
                    destination_id: "workshop".into(),
                },
            })
            .unwrap();
        let mut input = build_scene_input(&state).unwrap();
        assert!(needs_introduction(&input));
        input["here"]["characters"][0]["relationship"] = json!("Different descriptive wording");
        assert!(needs_introduction(&input));
        state
            .apply(&Command {
                command_id: "intro".into(),
                expected_revision: 1,
                action: Action::Acknowledge {
                    scene_id: state.presentation().scene_id,
                    entry_id: "greeting".into(),
                },
            })
            .unwrap();
        let input = build_scene_input(&state).unwrap();
        assert!(!needs_introduction(&input));
        assert!(!needs_introduction(
            &build_scene_input(&State::new()).unwrap()
        ));
    }
}
