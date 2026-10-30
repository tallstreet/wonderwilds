use serde_json::json;
use std::collections::BTreeSet;
use wonderwilds_director::story::evaluation::{
    Candidate, corpus, grade_candidate, validate_scenario,
};

#[test]
fn corpus_is_reproducible_replayable_and_covers_three_missions_and_detours() {
    let rows = corpus().unwrap();
    assert_eq!(
        serde_json::to_value(&rows).unwrap(),
        serde_json::to_value(corpus().unwrap()).unwrap()
    );
    assert!((100..=200).contains(&rows.len()), "{} rows", rows.len());
    let mut ids = BTreeSet::new();
    let mut missions = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut partial = false;
    let mut paused = false;
    let mut required_intro = false;
    let mut three_completed = false;
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
        three_completed |= state.completed_missions().len() == 3;
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
    assert_eq!(families.len(), 4);
    assert_eq!(
        missions,
        BTreeSet::from(["pips_bridge", "pips_picnic", "welcome_lights"])
    );
    assert!(partial && paused && required_intro && three_completed);
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
