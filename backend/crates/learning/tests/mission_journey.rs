//! Integration-owner checks using real generated tasks, not synthetic mastery flags.
use serde_json::json;
use wonderwilds_learning::{
    curriculum::CurriculumGraph,
    mission::{
        self, MissionAnswer, MissionObservation, MissionState, PairAssignment, PublicTask, Status,
        Support,
    },
};

fn answer(task: &PublicTask) -> MissionAnswer {
    match task {
        PublicTask::Pairing {
            objects,
            recipients,
            ..
        } => MissionAnswer::Pairing {
            assignments: objects
                .iter()
                .zip(recipients.iter().rev())
                .map(|(object, recipient)| PairAssignment {
                    object_id: object.id.clone(),
                    recipient_id: recipient.id.clone(),
                })
                .collect(),
        },
        PublicTask::CountCollection {
            objects, choices, ..
        } => MissionAnswer::Choice {
            choice_id: choices
                .iter()
                .find(|c| c.value == objects.len() as i8)
                .unwrap()
                .id
                .clone(),
        },
        PublicTask::CompareCollections {
            collections,
            prompt_mode,
            ..
        } => {
            let selected = if prompt_mode == "more" {
                collections.iter().max_by_key(|c| c.items.len())
            } else {
                collections.iter().min_by_key(|c| c.items.len())
            }
            .unwrap();
            MissionAnswer::Choice {
                choice_id: selected.id.clone(),
            }
        }
        PublicTask::NumberPath {
            shown,
            direction,
            choices,
        } => {
            let delta = if direction == "forward" { 1 } else { -1 };
            assert!(shown.iter().all(|n| (1..=20).contains(n)));
            assert!(shown.windows(2).all(|w| w[1] - w[0] == delta));
            let expected = shown.last().unwrap() + delta;
            assert!((1..=20).contains(&expected));
            MissionAnswer::Choice {
                choice_id: choices
                    .iter()
                    .find(|c| c.value == expected)
                    .unwrap()
                    .id
                    .clone(),
            }
        }
        PublicTask::PartWhole {
            whole,
            shown_part,
            choices,
        } => {
            assert!((2..=10).contains(whole));
            assert!(shown_part < whole);
            MissionAnswer::Choice {
                choice_id: choices
                    .iter()
                    .find(|c| c.value == (whole - shown_part) as i8)
                    .unwrap()
                    .id
                    .clone(),
            }
        }
    }
}

#[test]
fn actual_items_advance_the_graph_and_replay_the_same_bounded_progress() {
    let graph = CurriculumGraph::load().unwrap();
    let mut state = MissionState::default();
    let mut history = Vec::new();
    for skill in ["M01", "M03", "M04", "M05", "M06"] {
        assert!(
            mission::frontier(&state, Some("maths"))
                .unwrap()
                .iter()
                .any(|n| n.skill_id == skill)
        );
        for seed in 0..12 {
            let sequence = history.len() as u64 + 1;
            let context = if seed < 6 { "pip-bridge" } else { "pip-picnic" };
            let activity = mission::authored_activity_in_context(
                skill,
                seed,
                &format!("item-{sequence}"),
                context,
                "Prepare supplies for the mission",
            )
            .unwrap();
            assert!(
                graph
                    .get(skill)
                    .unwrap()
                    .allowed_representations
                    .contains(&activity.view.representation)
            );
            let outcome = activity
                .assess(
                    &answer(&activity.view.task),
                    Support {
                        hint_count: 0,
                        worked_example: false,
                        read_aloud: false,
                    },
                )
                .unwrap();
            assert!(outcome.correct, "{skill} seed {seed}");
            let observation = MissionObservation {
                evidence: serde_json::from_value(json!({
                    "schemaVersion":"1.0", "eventId":format!("event-{sequence}"),
                    "encounterId":activity.view.id, "skillId":skill,
                    "variantId":activity.view.variant_id, "itemFamily":"mission_maths",
                    "contextId":activity.view.context_id, "representation":activity.view.representation,
                    "result":"correct", "eventKind":"answer_check", "firstCheck":true,
                    "support":{"hintCount":0,"workedExample":false,"readAloud":false},
                    "sessionId":if seed < 6 { "verified-visit-one" } else { "verified-visit-two" },
                    "serverSequence":sequence,"provenance":"server_verified"
                })).unwrap(),
                parameter_fingerprint: activity.canonical_fingerprint().to_owned(),
            };
            mission::record(&mut state, observation.clone()).unwrap();
            history.push(observation);
        }
        assert_eq!(
            mission::summary(skill, &state).unwrap().status,
            Status::PracticeReady,
            "{skill}"
        );
    }
    assert!(mission::frontier(&state, Some("maths")).unwrap().is_empty());
    assert_eq!(mission::replay(&history).unwrap(), state);
    assert!(state.observations.len() <= 60);
    assert_eq!(mission::all_skill_projections(&state).unwrap().len(), 120);
    assert!(serde_json::to_vec(&state).unwrap().len() < 64 * 1024);
}
