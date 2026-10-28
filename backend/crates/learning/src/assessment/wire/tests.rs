use super::*;
fn answer(value: Value) -> WireAnswer {
    serde_json::from_value(value).unwrap()
}
#[test]
fn every_authored_story_skill_has_a_valid_non_answer_wire_projection() {
    for skill in [
        "M05", "M06", "M08", "M09", "M14", "M11", "M12", "E01", "E05", "E06", "E10", "E11", "E12",
    ] {
        for seed in 0..3 {
            let (_, encounter) = authored_encounter(skill, seed, "server-encounter-1").unwrap();
            let value = serde_json::to_value(encounter).unwrap();
            assert!(wonderwilds_contracts::validate("EncounterInstance", &value).is_ok());
            let serialized = value.to_string();
            for key in ["accepted", "solution", "correctAnswer", "target"] {
                assert!(!serialized.contains(&format!("\"{key}\"")));
            }
        }
    }
    assert!(authored_encounter("E14", 0, "id").is_err());
    assert!(prompt("unknown-prompt").is_none());
}
#[test]
fn answers_map_ids_and_reject_forged_objects_or_duplicates() {
    let (task, mut encounter) = authored_encounter("M05", 0, "sequence").unwrap();
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["number-4"]}))
        )
        .unwrap()
        .correct,
        Some(true)
    );
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["number-3"]}))
        )
        .unwrap()
        .correct,
        Some(false)
    );
    for ids in [
        vec!["shown-0"],
        vec!["forged"],
        vec!["number-4", "number-4"],
    ] {
        assert_eq!(
            assess_wire(
                &task,
                &encounter,
                &answer(json!({"kind":"choices","choiceIds":ids}))
            ),
            Err(Error::InvalidAnswer)
        );
    }
    encounter.support.hint_count = 1;
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["number-4"]}))
        )
        .unwrap()
        .classification,
        Classification::Supported
    );
}
#[test]
fn tile_and_story_order_adapters_do_not_trust_client_order_or_values() {
    let (task, mut encounter) = authored_encounter("E06", 0, "word").unwrap();
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["0","1","2"]}))
        )
        .unwrap()
        .correct,
        Some(true)
    );
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["2","1","0"]}))
        )
        .unwrap()
        .correct,
        Some(false)
    );
    encounter.support.read_aloud = true;
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["0","1","2"]}))
        )
        .unwrap()
        .classification,
        Classification::Supported
    );
    let (task, encounter) = authored_encounter("E11", 0, "story").unwrap();
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["find_seed","plant_seed","water_seed"]}))
        )
        .unwrap()
        .correct,
        Some(true)
    );
    assert_eq!(
        assess_wire(
            &task,
            &encounter,
            &answer(json!({"kind":"choices","choiceIds":["plant_seed","water_seed","find_seed"]}))
        )
        .unwrap()
        .correct,
        Some(false)
    );
}
