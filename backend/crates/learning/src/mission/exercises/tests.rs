use super::*;
#[test]
fn all_current_questions_validate_and_accept_only_private_solutions() {
    let bank = bank();
    assert_eq!(bank.schema_version, "1.0");
    assert_eq!(bank.review_status, "pending");
    assert!(!bank.release_enabled);
    assert_eq!(bank.skills.len(), 120);
    for skill in &bank.skills {
        for exercise in &skill.exercises {
            assert!(!exercise.facet.is_empty());
            for item in &exercise.items {
                let task = PublicTask::Question {
                    exercise_id: exercise.id.clone(),
                    interaction: item.interaction.clone(),
                    stimuli: item.stimuli.clone(),
                };
                let wire = serde_json::to_value(&task).unwrap();
                wonderwilds_contracts::validate("MissionTask", &wire)
                    .unwrap_or_else(|e| panic!("{}: {:?}", exercise.id, e));
                assert!(item.hints.len() >= 2);
                if let Some(answer) = &item.answer {
                    let answer = if answer.get("rule").is_some() {
                        let objects = item.interaction["items"].as_array().unwrap();
                        let targets = item.interaction["targets"].as_array().unwrap();
                        json!({"kind":"assign","assignments":objects.iter().enumerate().map(|(i,o)|json!({"objectId":o["id"],"targetId":targets[i%targets.len()]["id"]})).collect::<Vec<_>>()})
                    } else {
                        answer.clone()
                    };
                    wonderwilds_contracts::validate("MissionLearningAnswer", &answer).unwrap();
                    let answer: MissionAnswer = serde_json::from_value(answer).unwrap();
                    assert_eq!(
                        assess(item, &answer),
                        Ok(true),
                        "{} {}",
                        exercise.id,
                        item.semantic_id
                    );
                } else {
                    assert_eq!(item.interaction["kind"], "workshop");
                }
            }
        }
    }
}
#[test]
fn full_seeds_are_stable_and_do_not_reinterpret_variants() {
    for node in CurriculumGraph::load().unwrap().nodes() {
        let first = activity(&node.id, 0, "a").unwrap();
        let again = activity(&node.id, 0, "a").unwrap();
        assert_eq!(first.view, again.view);
        assert_ne!(
            first.view.variant_id,
            activity(&node.id, 24, "a").unwrap().view.variant_id
        );
    }
}
#[test]
fn numeric_equivalence_is_exact_and_malformed_answers_are_rejected() {
    let item = Item {
        title: "Test".into(),
        prompt: "Test".into(),
        representation: "text".into(),
        stimuli: vec![],
        interaction: json!({"kind":"number_entry","fields":[{"id":"a","label":"A","format":"fraction"}]}),
        answer: Some(json!({"kind":"numbers","values":[{"fieldId":"a","value":"1/2"}]})),
        hints: vec!["Hint".into()],
        explanation: "Half".into(),
        semantic_id: "half".into(),
    };
    for value in ["2/4", "0.5", "0.50"] {
        assert_eq!(
            assess(
                &item,
                &MissionAnswer::Numbers {
                    values: vec![NumberEntry {
                        field_id: "a".into(),
                        value: value.into()
                    }]
                }
            ),
            Ok(true)
        );
    }
    for value in ["1/0", "NaN", "Infinity", "1+1", "0.5.0", ""] {
        assert_eq!(
            assess(
                &item,
                &MissionAnswer::Numbers {
                    values: vec![NumberEntry {
                        field_id: "a".into(),
                        value: value.into()
                    }]
                }
            ),
            Err(MissionError::InvalidAnswer)
        );
    }
    assert_eq!(
        assess(
            &item,
            &MissionAnswer::Numbers {
                values: vec![NumberEntry {
                    field_id: "other".into(),
                    value: "1/2".into()
                }]
            }
        ),
        Err(MissionError::InvalidAnswer)
    );
}

#[test]
fn public_order_and_choice_ids_do_not_encode_the_private_solution() {
    for skill in ["M05", "M10", "M13", "EY106"] {
        let mut positions = BTreeSet::new();
        for seed in 0..36 {
            let activity = activity(skill, seed, "runtime-keys").unwrap();
            if let AssessmentKey::Question(item) = &activity.key
                && let Some(answer) = &item.answer
            {
                if answer.get("rule").is_some() {
                    continue;
                }
                let typed: MissionAnswer = serde_json::from_value(answer.clone()).unwrap();
                assert_eq!(assess(item, &typed), Ok(true));
                if let Some(id) = answer.get("choiceId").and_then(Value::as_str) {
                    positions.insert(id.to_owned());
                }
            }
        }
        if skill == "M05" {
            assert!(
                positions.len() > 1,
                "correct choice stuck in one public position"
            );
        }
    }
}
#[test]
fn mixed_numbers_accept_equivalent_fractional_parts() {
    let skill = bank()
        .skills
        .iter()
        .find(|s| s.skill_id == "MY502")
        .unwrap();
    let item = &skill
        .exercises
        .iter()
        .find(|e| e.id.ends_with(".mixed_number"))
        .unwrap()
        .items[0];
    let expected = item.answer.as_ref().unwrap();
    let mut value = expected.clone();
    value["values"][1]["value"] = json!(
        (expected["values"][1]["value"]
            .as_str()
            .unwrap()
            .parse::<u32>()
            .unwrap()
            * 2)
        .to_string()
    );
    value["values"][2]["value"] = json!(
        (expected["values"][2]["value"]
            .as_str()
            .unwrap()
            .parse::<u32>()
            .unwrap()
            * 2)
        .to_string()
    );
    assert_eq!(
        assess(item, &serde_json::from_value(value).unwrap()),
        Ok(true)
    );
}
