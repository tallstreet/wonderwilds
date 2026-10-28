use super::*;

#[test]
fn authored_pack_solutions_match_actual_rust_assessment() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../content/activities/draft-v1.json"
    ))
    .unwrap();
    assert_eq!(pack["releaseEnabled"], false);
    assert_eq!(pack["reviewStatus"], "pending");
    let strings = |value: &serde_json::Value| {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    let activities = pack["activities"].as_array().unwrap();
    assert_eq!(activities.len(), 6);
    for activity in activities {
        assert!(activity["contexts"].as_array().unwrap().len() >= 2);
        assert_eq!(activity["hints"].as_array().unwrap().len(), 3);
        assert!(activity["variants"].as_array().unwrap().len() >= 3);
        for variant in activity["variants"].as_array().unwrap() {
            let number = |field: &str| u8::try_from(variant[field].as_u64().unwrap()).unwrap();
            let (task, answer) = match activity["family"].as_str().unwrap() {
                "quantity" => (
                    Task::MissingQuantity {
                        total: number("total"),
                        given: number("given"),
                    },
                    Answer::Number(number("solution")),
                ),
                "sharing" => (
                    Task::EqualSharing {
                        total: number("total"),
                        recipients: number("recipients"),
                    },
                    Answer::Groups(
                        variant["solution"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
                            .collect(),
                    ),
                ),
                "pattern_sort" => (
                    Task::Pattern {
                        unit: strings(&variant["unit"]),
                        prefix_len: number("prefixLength"),
                    },
                    Answer::Text(variant["solution"].as_str().unwrap().into()),
                ),
                "spatial_measurement" => (
                    Task::Position {
                        relation: variant["relation"].as_str().unwrap().into(),
                        choices: strings(&variant["choices"]),
                    },
                    Answer::Text(variant["solution"].as_str().unwrap().into()),
                ),
                "phonics_word" => (
                    Task::Word {
                        tiles: strings(&variant["tiles"]),
                        accepted: strings(&variant["accepted"]),
                        independently_read: true,
                    },
                    Answer::Tiles(
                        variant["solution"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|n| usize::try_from(n.as_u64().unwrap()).unwrap())
                            .collect(),
                    ),
                ),
                "story_message" => (
                    Task::StorySequence {
                        event_ids: strings(&variant["events"]),
                    },
                    Answer::EventIds(strings(&variant["solution"])),
                ),
                family => panic!("unsupported family {family}"),
            };
            assert_eq!(
                task.validate_skill(activity["skillId"].as_str().unwrap()),
                Ok(())
            );
            assert_eq!(
                task.validate_representation(activity["representation"].as_str().unwrap()),
                Ok(())
            );
            assert_eq!(
                assess(&task, &answer, Support::default()).unwrap().correct,
                Some(true)
            );
        }
    }
}

#[test]
fn curriculum_binding_rejects_wrong_node_and_out_of_band_items() {
    assert_eq!(
        Task::MissingQuantity {
            total: 11,
            given: 4
        }
        .validate_skill("M06"),
        Err(Error::InvalidTask)
    );
    assert_eq!(
        Task::EqualSharing {
            total: 16,
            recipients: 4
        }
        .validate_skill("M08"),
        Err(Error::InvalidTask)
    );
    assert_eq!(
        generate(Family::Quantity, 1).validate_skill("E06"),
        Err(Error::InvalidTask)
    );
    assert_eq!(
        generate(Family::PhonicsWord, 1).validate_skill("E14"),
        Err(Error::InvalidTask)
    );
}

fn solve(task: &Task) -> Answer {
    match task {
        Task::NumberSequence { start, step, shown } => Answer::Number(start + step * shown),
        Task::MissingQuantity { total, given } => Answer::Number(total - given),
        Task::EqualSharing { total, recipients } => {
            Answer::Groups(vec![total / recipients; usize::from(*recipients)])
        }
        Task::Pattern { unit, prefix_len } => {
            Answer::Text(unit[usize::from(*prefix_len) % unit.len()].clone())
        }
        Task::Sort { attributes, target } => Answer::Indices(
            attributes
                .iter()
                .enumerate()
                .filter_map(|(i, a)| (a == target).then_some(i))
                .collect(),
        ),
        Task::Position { relation, .. } => Answer::Text(relation.clone()),
        Task::Measure { lengths } => Answer::Indices(vec![
            lengths
                .iter()
                .position(|n| Some(n) == lengths.iter().max())
                .unwrap(),
        ]),
        Task::Word {
            tiles, accepted, ..
        } => {
            let mut used = vec![];
            for c in accepted[0].chars() {
                used.push(
                    tiles
                        .iter()
                        .enumerate()
                        .find(|(i, t)| !used.contains(i) && *t == &c.to_string())
                        .unwrap()
                        .0,
                );
            }
            Answer::Tiles(used)
        }
        Task::StorySequence { event_ids } => Answer::EventIds(event_ids.clone()),
        Task::MessageChoice { accepted, .. } => Answer::Text(accepted[0].clone()),
        Task::AdultReviewed { .. } => Answer::AdultResponse,
    }
}

#[test]
fn exhaustive_missing_quantity_domain() {
    for total in 0..=20 {
        for given in 0..=total {
            let task = Task::MissingQuantity { total, given };
            for value in 0..=21 {
                assert_eq!(
                    assess(&task, &Answer::Number(value), Support::default())
                        .unwrap()
                        .correct,
                    Some(value == total - given)
                );
            }
        }
    }
}

#[test]
fn exhaustive_sharing_rejects_unequal_equal_totals() {
    for total in 0..=20 {
        for recipients in 2..=5 {
            let task = Task::EqualSharing { total, recipients };
            if total % recipients != 0 {
                assert_eq!(task.validate(), Err(Error::InvalidTask));
                continue;
            }
            assert_eq!(
                assess(&task, &solve(&task), Support::default())
                    .unwrap()
                    .correct,
                Some(true)
            );
            if total > 0 {
                let mut groups = vec![total / recipients; usize::from(recipients)];
                groups[0] += 1;
                groups[1] -= 1;
                assert_eq!(groups.iter().sum::<u8>(), total);
                assert_eq!(
                    assess(&task, &Answer::Groups(groups), Support::default())
                        .unwrap()
                        .correct,
                    Some(false)
                );
            }
        }
    }
}

#[test]
fn deterministic_factories_are_solvable_and_bounded() {
    for family in [
        Family::Quantity,
        Family::Sharing,
        Family::PatternSort,
        Family::SpatialMeasurement,
        Family::PhonicsWord,
        Family::StoryMessage,
    ] {
        for seed in 0..512 {
            let task = generate(family, seed);
            assert_eq!(task, generate(family, seed));
            assert_eq!(task.validate(), Ok(()));
            assert_eq!(
                assess(&task, &solve(&task), Support::default()).unwrap(),
                Outcome {
                    correct: Some(true),
                    classification: Classification::Independent
                }
            );
            match task {
                Task::MissingQuantity { total, .. } => assert!((2..=10).contains(&total)),
                Task::EqualSharing { total, .. } => assert!((2..=12).contains(&total)),
                _ => (),
            }
        }
    }
}

#[test]
fn malformed_tasks_and_answers_fail_closed() {
    for task in [
        Task::EqualSharing {
            total: 6,
            recipients: 0,
        },
        Task::MissingQuantity { total: 3, given: 4 },
        Task::Pattern {
            unit: vec![],
            prefix_len: 5,
        },
        Task::Measure {
            lengths: vec![4, 4],
        },
        Task::Word {
            tiles: vec!["a".into()],
            accepted: vec!["cat".into()],
            independently_read: true,
        },
        Task::StorySequence {
            event_ids: vec!["same".into(), "same".into()],
        },
    ] {
        assert_eq!(
            assess(&task, &Answer::Number(0), Support::default()),
            Err(Error::InvalidTask)
        );
    }
    let sort = Task::Sort {
        attributes: vec!["red".into(), "blue".into()],
        target: "red".into(),
    };
    for indices in [vec![0, 0], vec![2], vec![usize::MAX]] {
        assert_eq!(
            assess(&sort, &Answer::Indices(indices), Support::default()),
            Err(Error::InvalidAnswer)
        );
    }
    let word = generate(Family::PhonicsWord, 0);
    assert_eq!(
        assess(&word, &Answer::Tiles(vec![0, 0]), Support::default()),
        Err(Error::InvalidAnswer)
    );
    assert_eq!(
        word.validate_representation("unlabelled_image"),
        Err(Error::UnsupportedRepresentation)
    );
    assert_eq!(word.validate_representation("keyboard_tiles"), Ok(()));
}

#[test]
fn hints_examples_and_assessment_audio_classify_support_honestly() {
    let reading = generate(Family::PhonicsWord, 0);
    for support in [
        Support {
            hint_count: 1,
            ..Support::default()
        },
        Support {
            worked_example: true,
            ..Support::default()
        },
        Support {
            read_aloud: true,
            ..Support::default()
        },
    ] {
        assert_eq!(
            assess(&reading, &solve(&reading), support)
                .unwrap()
                .classification,
            Classification::Supported
        );
    }
    let quantity = generate(Family::Quantity, 0);
    assert_eq!(
        assess(
            &quantity,
            &solve(&quantity),
            Support {
                read_aloud: true,
                motor_access: true,
                ..Support::default()
            }
        )
        .unwrap()
        .classification,
        Classification::Independent
    );
    assert_eq!(
        assess(
            &reading,
            &solve(&reading),
            Support {
                motor_access: true,
                ..Support::default()
            }
        )
        .unwrap()
        .classification,
        Classification::Independent
    );
}

#[test]
fn language_alternatives_are_authored_and_subjective_outcomes_unassessed() {
    let task = Task::Word {
        tiles: vec!["t".into(), "a".into(), "p".into()],
        accepted: vec!["tap".into(), "pat".into()],
        independently_read: true,
    };
    for answer in [vec![0, 1, 2], vec![2, 1, 0]] {
        assert_eq!(
            assess(&task, &Answer::Tiles(answer), Support::default())
                .unwrap()
                .correct,
            Some(true)
        );
    }
    assert_eq!(
        assess(&task, &Answer::Tiles(vec![0, 2, 1]), Support::default())
            .unwrap()
            .correct,
        Some(false)
    );
    let adult = Task::AdultReviewed {
        prompt_id: "E14-kind-response-reason".into(),
    };
    assert_eq!(
        assess(&adult, &Answer::AdultResponse, Support::default()).unwrap(),
        Outcome {
            correct: None,
            classification: Classification::Unassessed
        }
    );
}
