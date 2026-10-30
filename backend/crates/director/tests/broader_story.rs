use serde_json::{Value, json};
use wonderwilds_director::story::{
    MockSceneProvider, SceneProvider, authored_scene, build_plan_input, build_scene_input,
    resolve_action, validate_scene,
};
use wonderwilds_domain::story::{Command, Location, State};

fn check(state: &State) {
    let input = build_scene_input(state).unwrap();
    build_plan_input(state).unwrap();
    let p = state.presentation();
    let entries:Vec<_>=p.entries.iter().map(|e|if e.speaker_id.is_empty(){json!({"type":"narration","entryId":e.entry_id,"text":e.text})}else{json!({"type":"dialogue","entryId":e.entry_id,"speakerId":e.speaker_id,"text":e.text})}).collect();
    let choices: Vec<_> = input["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| json!({"label":a["description"],"actionId":a["id"]}))
        .collect();
    let canonical = json!({"sceneId":p.scene_id,"presentation":entries,"choices":choices});
    let proposed = MockSceneProvider.propose(&input, &canonical).unwrap();
    validate_scene(&input, &canonical, &proposed).unwrap();
    assert_eq!(
        authored_scene(state).unwrap(),
        proposed,
        "Mock unexpectedly used fallback"
    );
    for c in input["here"]["characters"].as_array().unwrap() {
        assert!(
            proposed["presentation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["speakerId"] == c["id"]),
            "Missing conversation for {c}"
        );
    }
    let restored = State::from_checkpoint(&state.checkpoint()).unwrap();
    assert_eq!(build_scene_input(&restored).unwrap(), input);
}
fn step(state: &mut State, action: Value) {
    check(state);
    let input = build_scene_input(state).unwrap();
    let advertised = input["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"] == action)
        .unwrap_or_else(|| panic!("Unavailable {action} in {input}"));
    let bound = resolve_action(state, advertised["id"].as_str().unwrap()).unwrap();
    let command = Command {
        command_id: format!("broader-{}", state.revision()),
        expected_revision: state.revision(),
        action: bound,
    };
    state.apply(&command).unwrap();
    check(state);
    if action["type"] == "inspect" && action["objectId"] == "hollow_branch" {
        let scene = authored_scene(state).unwrap();
        assert!(
            scene["presentation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["entryId"] == "humming_hollow-result"
                    && e["text"].as_str().unwrap().contains("wind"))
        );
    }
    if action["type"] == "talk" && action["topicId"] == "cosy_preferences" {
        let scene = authored_scene(state).unwrap();
        assert!(
            scene["presentation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["speakerId"] == "pip" && e["text"].as_str().unwrap().contains("stream"))
        );
    }
}
fn travel(state: &mut State, to: &str) {
    if state.location().as_str() != to {
        step(state, json!({"type":"travel","destinationId":to}));
    }
}

#[test]
fn four_new_missions_project_both_outcomes_and_parallel_gathering_orders() {
    for alternate in [false, true] {
        let saved: Value = serde_json::from_str(include_str!(
            "../../domain/src/story/fixtures/lights-completed-v2.json"
        ))
        .unwrap();
        let mut s = State::from_checkpoint(&saved).unwrap();
        assert!(resolve_action(&s, &format!("story-v2-{}-0", s.revision())).is_err());
        travel(&mut s, "bridge_bank");
        step(&mut s, json!({"type":"accept_mission"}));
        step(
            &mut s,
            json!({"type":"talk","characterId":"pip","topicId":"humming_hollow"}),
        );
        travel(&mut s, "far_bank");
        for object in [
            "humming_hollow",
            "trembling_leaf",
            "mossy_stump",
            "hollow_branch",
        ] {
            step(&mut s, json!({"type":"inspect","objectId":object}));
        }
        step(
            &mut s,
            json!({"type":"choose","choiceId":"instrument_home","optionId":if alternate{"gathering"}else{"woods"}}),
        );
        let place = if alternate { "bridge_bank" } else { "far_bank" };
        travel(&mut s, place);
        assert!(
            !build_scene_input(&s).unwrap()["context"]["memory"]["facts"]
                .as_array()
                .unwrap()
                .contains(&json!(format!("The wind instrument is at {place}.")))
        );
        step(
            &mut s,
            json!({"type":"assemble","objectId":"wind_instrument","recipeId":"place_instrument"}),
        );
        travel(&mut s, "bridge_bank");
        step(
            &mut s,
            json!({"type":"talk","characterId":"pip","topicId":"share_discovery"}),
        );
        step(&mut s, json!({"type":"accept_mission"}));
        step(
            &mut s,
            json!({"type":"talk","characterId":"pip","topicId":"cosy_preferences"}),
        );
        step(&mut s, json!({"type":"inspect","objectId":"corner_site"}));
        step(
            &mut s,
            json!({"type":"choose","choiceId":"corner_style","optionId":if alternate{"star_canopy"}else{"leafy_nook"}}),
        );
        for (object, recipe) in [
            ("corner_frame", "build_corner_frame"),
            ("corner_cushions", "arrange_cushions"),
        ] {
            step(
                &mut s,
                json!({"type":"assemble","objectId":object,"recipeId":recipe}),
            );
        }
        step(
            &mut s,
            json!({"type":"talk","characterId":"pip","topicId":"enjoy_corner"}),
        );
        step(&mut s, json!({"type":"accept_mission"}));
        travel(&mut s, "workshop");
        step(
            &mut s,
            json!({"type":"talk","characterId":"bramble","topicId":"visit_wish"}),
        );
        step(&mut s, json!({"type":"inspect","objectId":"workshop_path"}));
        step(
            &mut s,
            json!({"type":"assemble","objectId":"workshop_path","recipeId":"clear_path"}),
        );
        step(
            &mut s,
            json!({"type":"talk","characterId":"bramble","topicId":"ready_to_visit"}),
        );
        step(
            &mut s,
            json!({"type":"accompany","characterId":"bramble","destinationId":"bridge_bank"}),
        );
        assert_eq!(s.bramble_location(), Location::BridgeBank);
        step(
            &mut s,
            json!({"type":"talk","characterId":"bramble","topicId":"visit_arrival"}),
        );
        step(&mut s, json!({"type":"accept_mission"}));
        step(
            &mut s,
            json!({"type":"choose","choiceId":"gathering_place","optionId":if alternate{"far_bank"}else{"near_bank"}}),
        );
        let location = if alternate { "far_bank" } else { "bridge_bank" };
        if !alternate {
            for id in ["pip", "bramble"] {
                step(&mut s, json!({"type":"invite","characterId":id}));
            }
        }
        travel(&mut s, location);
        for (object, recipe) in [
            ("gathering_music", "prepare_music"),
            ("gathering_seats", "arrange_seats"),
        ] {
            step(
                &mut s,
                json!({"type":"assemble","objectId":object,"recipeId":recipe}),
            );
        }
        if alternate {
            travel(&mut s, "bridge_bank");
            for id in ["bramble", "pip"] {
                step(&mut s, json!({"type":"invite","characterId":id}));
            }
            travel(&mut s, location);
        }
        step(
            &mut s,
            json!({"type":"talk","characterId":"pip","topicId":"welcome_friends"}),
        );
        assert_eq!(s.completed_missions().len(), 7);
        let home = s.instrument_home();
        travel(&mut s, if alternate { "bridge_bank" } else { "far_bank" });
        assert_eq!(home, s.instrument_home());
        let scene = authored_scene(&s).unwrap();
        assert!(!scene.to_string().contains("Everyone is here"));
        assert_eq!(
            build_scene_input(&s).unwrap()["context"]["memory"]["recentMissions"]
                .as_array()
                .unwrap()
                .len(),
            7
        );
    }
}
