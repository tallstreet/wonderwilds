use serde_json::json;
use std::collections::BTreeSet;
use wonderwilds_director::story::evaluation::{
    Candidate, corpus, grade_candidate, validate_scenario,
};

#[test]
fn corpus_is_reproducible_replayable_and_covers_seven_missions_and_choices() {
    let rows = corpus().unwrap();
    assert_eq!(
        serde_json::to_value(&rows).unwrap(),
        serde_json::to_value(corpus().unwrap()).unwrap()
    );
    assert!((500..=800).contains(&rows.len()), "{} rows", rows.len());
    let mut ids = BTreeSet::new();
    let mut missions = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut partial = false;
    let mut paused = false;
    let mut required_intro = false;
    let mut seven_completed = false;
    let mut outcomes = BTreeSet::new();
    let mut optional_clue = false;
    let mut paused_with_invitation = false;
    for row in &rows {
        assert!(ids.insert(row.scenario_id.clone()));
        families.insert(row.family_id.clone());
        let state = validate_scenario(row).unwrap();
        missions.insert(state.mission_id());
        partial |= state.wood() == 1 || state.berries() == 1 || state.glow_pebbles() == 1;
        paused |= row.input["mission"]["status"] == "abandoned";
        required_intro |= !row.presentation["requiredEntries"]
            .as_array()
            .unwrap()
            .is_empty();
        seven_completed |= state.completed_missions().len() == 7;
        optional_clue |= state.has_story_flag("mossy_stump_checked");
        paused_with_invitation |= row.input["mission"]["status"] == "abandoned"
            && (state.has_story_flag("pip_invited") || state.has_story_flag("bramble_invited"));
        if state.completed_missions().len() == 7 {
            outcomes.insert((
                state.instrument_home().unwrap().as_str(),
                state.corner_style().unwrap().to_owned(),
                state.gathering_place().unwrap().as_str(),
            ));
        }
        for action in row.input["availableActions"].as_array().unwrap() {
            if action["action"]["type"] == "cancel_mission" {
                assert_eq!(action["description"], "Take a break from this mission.");
            }
        }
        let grade = grade_candidate(
            row,
            &Candidate {
                scenario_id: row.scenario_id.clone(),
                candidate_id: "reference".into(),
                output: row.reference.clone(),
                generator: None,
            },
        )
        .unwrap();
        assert!(grade.passed, "{:?}", grade.failures);
        assert_eq!(
            grade.simulated_actions,
            row.input["availableActions"].as_array().unwrap().len()
        );
    }
    assert_eq!(families.len(), 8);
    assert_eq!(
        outcomes.len(),
        8,
        "All lasting-choice combinations are captured"
    );
    assert_eq!(
        missions,
        BTreeSet::from([
            "pips_bridge",
            "pips_picnic",
            "welcome_lights",
            "humming_hollow",
            "cosy_corner",
            "brambles_visit",
            "moonlight_gathering"
        ])
    );
    assert!(
        partial
            && paused
            && required_intro
            && seven_completed
            && optional_clue
            && paused_with_invitation
    );
}

#[test]
fn mutations_fail_closed_but_prose_truth_needs_a_separate_judge() {
    let row = corpus().unwrap().remove(0);
    let candidate = |output| Candidate {
        scenario_id: row.scenario_id.clone(),
        candidate_id: "mutation".into(),
        output,
        generator: None,
    };
    for mutation in [
        "unknown_action",
        "absent_speaker",
        "state_grant",
        "missing_choice",
        "wrong_scene",
    ] {
        let mut output = row.reference.clone();
        match mutation {
            "unknown_action" => output["choices"][0]["actionId"] = json!("story-v2-0-999"),
            "absent_speaker" => output["presentation"][0]["speakerId"] = json!("woodcutter"),
            "state_grant" => output["inventory"] = json!([{"itemId":"planks","quantity":3}]),
            "missing_choice" => {
                output["choices"].as_array_mut().unwrap().pop();
            }
            _ => output["sceneId"] = json!("wrong-scene"),
        }
        assert!(
            !grade_candidate(&row, &candidate(output)).unwrap().passed,
            "{mutation}"
        );
    }
    assert!(
        !grade_candidate(&row, &candidate(json!("not scene JSON")))
            .unwrap()
            .passed
    );
    let mut false_prose = row.reference.clone();
    false_prose["presentation"][0]["text"] =
        json!("The bridge is finished! You have already crossed with Pip.");
    assert!(
        grade_candidate(&row, &candidate(false_prose))
            .unwrap()
            .passed,
        "Hard validation must not pretend to detect natural-language falsehoods"
    );
    let mut forged = row.clone();
    forged.input["context"]["inventory"] = json!([{"itemId":"planks","quantity":3}]);
    assert!(validate_scenario(&forged).is_err());
    let mut forged = row.clone();
    forged.reference["presentation"][0]["text"] = json!("Forged reference");
    assert!(validate_scenario(&forged).is_err());
}

#[test]
fn grader_command_ids_do_not_collide_with_valid_trace_commands() {
    let mut row = corpus().unwrap().remove(1);
    let old = row.checkpoint["commands"][0]["commandId"]
        .as_str()
        .unwrap()
        .to_owned();
    row.checkpoint["commands"][0]["commandId"] = json!("eval-check-0");
    let receipt = row.checkpoint["receipts"]
        .as_object_mut()
        .unwrap()
        .remove(&old)
        .unwrap();
    row.checkpoint["receipts"]["eval-check-0"] = receipt;
    row.checkpoint["receipts"]["eval-check-0"]["commandId"] = json!("eval-check-0");
    validate_scenario(&row).unwrap();
    assert!(
        grade_candidate(
            &row,
            &Candidate {
                scenario_id: row.scenario_id.clone(),
                candidate_id: "reference".into(),
                output: row.reference.clone(),
                generator: None,
            }
        )
        .unwrap()
        .passed
    );
}
