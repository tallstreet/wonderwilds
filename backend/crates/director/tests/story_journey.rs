//! Integrate the authored runtime with every reachable payload/scene shape.
use serde_json::json;
use std::collections::{BTreeSet, VecDeque};
use wonderwilds_director::story::{
    authored_scene, build_plan_input, build_scene_input, resolve_action,
};
use wonderwilds_domain::story::{Action, Command, MissionStatus, State};

fn key(state: &State) -> String {
    // Ignore command history/revision, retain every semantic fact that affects play.
    json!({
        "location": state.location(),
        "pip": state.pip_location(),
        "wood": state.wood(),
        "planks": state.planks(),
        "fallen": state.source_remaining("fallen_wood"),
        "spare": state.source_remaining("spare_wood"),
        "bridge": state.bridge_repaired(),
        "crossed": state.crossed_with_pip(),
        "mission": state.mission_status(),
        "milestones": state.milestones(),
        "met": state.met_woodcutter()
    })
    .to_string()
}

#[test]
fn reachable_authored_states_have_consistent_playable_payloads() {
    let mut queue = VecDeque::from([State::new()]);
    let mut visited = BTreeSet::new();
    let mut saw_completion = false;
    let mut saw_introduction = false;
    while let Some(state) = queue.pop_front() {
        if !visited.insert(key(&state)) {
            continue;
        }
        assert!(visited.len() < 1000, "State exploration exceeded its bound");
        let before = state.checkpoint();
        let plan = build_plan_input(&state).unwrap();
        let input = build_scene_input(&state).unwrap();
        let scene = authored_scene(&state).unwrap();
        assert_eq!(state.checkpoint(), before, "Projection changed game state");
        let restored = State::from_checkpoint(&before).unwrap();
        assert_eq!(build_plan_input(&restored).unwrap(), plan);
        assert_eq!(build_scene_input(&restored).unwrap(), input);
        assert_eq!(authored_scene(&restored).unwrap(), scene);
        for entry in scene["presentation"].as_array().unwrap() {
            if entry["type"] == "dialogue" {
                assert!(
                    input["here"]["characters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|c| c["id"] == entry["speakerId"])
                );
            }
        }
        for character in input["here"]["characters"].as_array().unwrap() {
            assert!(
                scene["presentation"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry["type"] == "dialogue"
                        && entry["speakerId"] == character["id"]),
                "Present character has no conversation: {character} in {scene}"
            );
        }
        assert!(!scene["choices"].as_array().unwrap().is_empty());
        for choice in scene["choices"].as_array().unwrap() {
            assert!(
                input["availableActions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a["id"] == choice["actionId"])
            );
        }
        for advertised in input["availableActions"].as_array().unwrap() {
            let handle = advertised["id"].as_str().unwrap();
            let action = resolve_action(&state, handle).unwrap();
            assert_eq!(serde_json::to_value(&action).unwrap(), advertised["action"]);
            // Exhaust this first mission's detours without expanding the product of
            // three mission state spaces. The sequence has a separate targeted test.
            if state.mission_status() == MissionStatus::Completed && action == Action::AcceptMission
            {
                continue;
            }
            let mut next = state.clone();
            let command = Command {
                command_id: format!("explore-{}", state.revision()),
                expected_revision: state.revision(),
                action,
            };
            let receipt = next.apply(&command).unwrap();
            assert!(
                resolve_action(&next, handle).is_err(),
                "Old handle remained valid"
            );
            let after = next.checkpoint();
            assert_eq!(next.apply(&command), Ok(receipt));
            assert_eq!(next.checkpoint(), after);
            queue.push_back(next);
        }
        // Presenting a line is distinct from merely building/returning its scene.
        let presentation = state.presentation();
        let mut presented = state.clone();
        presented
            .apply(&Command {
                command_id: format!("present-{}", state.revision()),
                expected_revision: state.revision(),
                action: Action::Acknowledge {
                    scene_id: presentation.scene_id,
                    entry_id: presentation.entries[0].entry_id.clone(),
                },
            })
            .unwrap();
        queue.push_back(presented);
        saw_introduction |= state.met_woodcutter();
        if state.mission_status() == MissionStatus::Completed {
            saw_completion = true;
            assert!(state.bridge_repaired() && state.crossed_with_pip());
            assert!(input["mission"]["current"].is_null());
            assert_eq!(input["mission"]["remaining"], json!([]));
        }
        // Each wood unit becomes one plank; one repair consumes exactly three.
        let accounted = state.wood()
            + state.planks()
            + state.source_remaining("fallen_wood").unwrap()
            + state.source_remaining("spare_wood").unwrap()
            + if state.bridge_repaired() { 3 } else { 0 };
        assert_eq!(accounted, 6);
    }
    assert!(saw_completion && saw_introduction);
    assert!(
        visited.len() > 50,
        "Search failed to explore alternate journeys"
    );
}

#[test]
fn three_connected_stories_keep_history_and_current_bindings_through_reload() {
    let legacy: serde_json::Value = serde_json::from_str(include_str!(
        "../../domain/src/story/fixtures/bridge-completed-v1.json"
    ))
    .unwrap();
    let mut state = State::from_checkpoint(&legacy).unwrap();
    assert_eq!(state.completed_missions().len(), 1);
    assert!(
        resolve_action(&state, "story-7-0").is_err(),
        "Old handles must not become a new mission acceptance"
    );
    for (kind, destination) in [
        ("accept_mission", None),
        ("collect", None),
        ("cancel_mission", None),
        ("accept_mission", None),
        ("travel", Some("bridge_bank")),
        ("travel", Some("workshop")),
        ("exchange", None),
        ("travel", Some("bridge_bank")),
        ("assemble", None),
        ("travel", Some("far_bank")),
        ("accompany", Some("bridge_bank")),
        ("accept_mission", None),
        ("travel", Some("far_bank")),
        ("collect", None),
        ("travel", Some("bridge_bank")),
        ("travel", Some("workshop")),
        ("exchange", None),
        ("travel", Some("bridge_bank")),
        ("assemble", None),
        ("travel", Some("far_bank")),
        ("assemble", None),
    ] {
        let input = build_scene_input(&state).unwrap();
        let scene = authored_scene(&state).unwrap();
        assert_eq!(input["mission"]["missionId"], state.mission_id());
        assert_eq!(
            input["context"]["memory"]["recentMissions"]
                .as_array()
                .unwrap()
                .len(),
            state.completed_missions().len()
        );
        for c in input["here"]["characters"].as_array().unwrap() {
            assert!(
                scene["presentation"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|e| e["speakerId"] == c["id"])
            );
        }
        for e in scene["presentation"].as_array().unwrap() {
            if e["type"] == "dialogue" {
                assert!(
                    input["here"]["characters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|c| e["speakerId"] == c["id"])
                );
            }
        }
        let binding = input["availableActions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["action"]["type"] == kind
                    && destination.is_none_or(|d| a["action"]["destinationId"] == d)
            })
            .unwrap();
        let action = resolve_action(&state, binding["id"].as_str().unwrap()).unwrap();
        state
            .apply(&Command {
                command_id: format!("series-{}", state.revision()),
                expected_revision: state.revision(),
                action,
            })
            .unwrap();
        let reloaded = State::from_checkpoint(&state.checkpoint()).unwrap();
        assert_eq!(
            authored_scene(&reloaded).unwrap(),
            authored_scene(&state).unwrap()
        );
        build_plan_input(&state).unwrap();
        state = reloaded;
    }
    assert_eq!(state.completed_missions().len(), 3);
    assert!(
        state.bridge_repaired() && state.picnic_shared() && state.near_lit() && state.far_lit()
    );
    assert!(
        state
            .available_actions()
            .iter()
            .all(|a| *a != Action::AcceptMission)
    );
    assert_eq!(
        build_plan_input(&state).unwrap()["opportunities"][0]["id"],
        "humming_hollow"
    );
    let input = build_scene_input(&state).unwrap();
    let facts = input["context"]["memory"]["facts"].as_array().unwrap();
    for fact in [
        "The bridge is repaired.",
        "The player and Pip shared a picnic.",
        "Two lanterns light the near bank.",
        "One lantern lights the far bank.",
    ] {
        assert!(facts.contains(&json!(fact)));
    }
}
