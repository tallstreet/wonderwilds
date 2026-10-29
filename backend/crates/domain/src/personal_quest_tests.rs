use super::*;
use crate::personal_quests::{QuestDocument, validate_document};

fn content() -> Content {
    Content::development(
        include_str!("../../../../content/story/adventure.v1.json"),
        include_str!("../../../../content/story/help-quests.v1.json"),
        include_str!("../../../../content/creatures/friends.v1.json"),
        include_str!("../../../../content/story/progression.v1.json"),
    )
    .unwrap()
}
fn initial(content: &Content) -> State {
    let mut state = State::new(content);
    for id in ["pip", "tansy", "nib", "flutter"] {
        state.creatures.insert(id.into());
        state.character_xp.insert(id.into(), 30);
    }
    state.features.insert("team_slot_1".into());
    state
}
fn context() -> Context {
    Context {
        unix_millis: 1000,
        recorded_random_choice: 0,
        facts: TrustedFacts::default(),
    }
}
fn command(state: &State, action: Action) -> Command {
    Command {
        command_id: format!("personal-command-{}", state.revision),
        expected_revision: state.revision,
        action,
    }
}
fn perform(
    state: &mut State,
    action: Action,
    content: &Content,
    context: Context,
    log: &mut Vec<ReplayEntry>,
) {
    let command = command(state, action);
    let result = apply(state, &command, content, &context).unwrap();
    log.push(ReplayEntry {
        command,
        context,
        events: result.events,
    });
    *state = result.state;
}
fn choose(state: &mut State, content: &Content, index: usize, log: &mut Vec<ReplayEntry>) {
    let QuestNode::Choice { choices, .. } = state.personal_quest_view().unwrap().node else {
        panic!("choice expected")
    };
    perform(
        state,
        Action::ChoosePersonalPath {
            choice_id: choices[index].id.clone(),
        },
        content,
        context(),
        log,
    );
}
fn begin(state: &mut State, content: &Content, log: &mut Vec<ReplayEntry>) -> (String, String) {
    let binding = state.active_personal_activity().unwrap();
    let encounter = format!("personal-encounter-{}", state.revision);
    let mut ctx = context();
    ctx.facts.reachable_quests.insert(binding.quest_id.clone());
    ctx.facts
        .issued_encounters
        .insert(encounter.clone(), binding.quest_id.clone());
    perform(
        state,
        Action::BeginQuest {
            quest_id: binding.quest_id.clone(),
            encounter_id: encounter.clone(),
        },
        content,
        ctx,
        log,
    );
    (binding.quest_id, encounter)
}
fn complete(state: &mut State, content: &Content, supported: bool, log: &mut Vec<ReplayEntry>) {
    let (quest_id, encounter_id) = begin(state, content, log);
    let mut ctx = context();
    ctx.facts.completed_encounters.insert(
        encounter_id.clone(),
        VerifiedCompletion {
            quest_id: quest_id.clone(),
            supported,
        },
    );
    perform(
        state,
        Action::CompleteQuest {
            quest_id,
            encounter_id,
        },
        content,
        ctx,
        log,
    );
}
fn episode(
    state: &mut State,
    content: &Content,
    quest_id: &str,
    path: usize,
    reflection: usize,
    supported: bool,
    log: &mut Vec<ReplayEntry>,
) {
    perform(
        state,
        Action::StartPersonalQuest {
            quest_id: quest_id.into(),
        },
        content,
        context(),
        log,
    );
    choose(state, content, path, log);
    complete(state, content, supported, log);
    choose(state, content, reflection, log);
    assert!(state.personal_quest_view().unwrap().node.is_handoff());
}

#[test]
fn personal_episodes_branch_reward_once_and_replay_without_a_game_ending() {
    let c = content();
    let start = initial(&c);
    let mut state = start.clone();
    let mut log = Vec::new();
    episode(&mut state, &c, "pip-lantern-trail", 0, 0, true, &mut log);
    let first = state.personal_quest_view().unwrap();
    assert_eq!(state.inventory.get("pip-personal-memory"), Some(&1));
    assert_eq!(state.xp(), 0);
    assert_eq!(state.character_xp("pip"), 30);
    assert!(state.flags.is_empty() && state.completed.is_empty());
    assert!(
        state
            .personal_quest_offers(&c)
            .iter()
            .any(|offer| offer.quest_id == "pip-lantern-trail"
                && offer.available
                && offer.completed)
    );
    episode(&mut state, &c, "pip-lantern-trail", 1, 1, false, &mut log);
    let second = state.personal_quest_view().unwrap();
    assert_ne!(first.episode_id, second.episode_id);
    assert_ne!(
        first.node, second.node,
        "reflection selects an actual continuation thread"
    );
    assert_eq!(state.inventory.get("pip-personal-memory"), Some(&1));
    assert_eq!(replay(&start, &c, &log).unwrap(), state);
    let serialized = serde_json::to_value(&state).unwrap();
    assert_eq!(serde_json::from_value::<State>(serialized).unwrap(), state);
    let last = log.last().unwrap();
    assert!(
        apply(&state, &last.command, &c, &last.context)
            .unwrap()
            .duplicate
    );
    assert_eq!(state.inventory.get("pip-personal-memory"), Some(&1));
}

#[test]
fn all_authored_friends_and_paths_have_equal_supported_memory_rewards() {
    let c = content();
    for document in c.personal_quests.documents() {
        for path in 0..2 {
            let mut independent = initial(&c);
            let mut supported = initial(&c);
            episode(
                &mut independent,
                &c,
                &document.quest_id,
                path,
                path,
                false,
                &mut Vec::new(),
            );
            episode(
                &mut supported,
                &c,
                &document.quest_id,
                path,
                path,
                true,
                &mut Vec::new(),
            );
            assert_eq!(independent.inventory, supported.inventory);
            assert_eq!(independent.character_xp, supported.character_xp);
            assert_eq!(independent.friendship, supported.friendship);
            assert_eq!(independent.inventory.values().sum::<u32>(), 1);
            independent.validate_personal_state(&c).unwrap();
            supported.validate_personal_state(&c).unwrap();
        }
    }
}

#[test]
fn personal_unlocks_and_choice_transitions_are_server_owned() {
    let c = content();
    let mut state = State::new(&c);
    let start = Action::StartPersonalQuest {
        quest_id: "pip-lantern-trail".into(),
    };
    assert_eq!(
        apply(&state, &command(&state, start.clone()), &c, &context()),
        Err(Rejection::Locked)
    );
    state.creatures.insert("pip".into());
    state.character_xp.insert("pip".into(), 20);
    assert_eq!(
        apply(&state, &command(&state, start.clone()), &c, &context()),
        Err(Rejection::Locked)
    );
    state.character_xp.insert("pip".into(), 30);
    perform(&mut state, start, &c, context(), &mut Vec::new());
    assert_eq!(
        apply(
            &state,
            &command(
                &state,
                Action::ChoosePersonalPath {
                    choice_id: "invented-choice".into()
                }
            ),
            &c,
            &context()
        ),
        Err(Rejection::InvalidCommand)
    );
    assert_eq!(state.inventory.len(), 0);
    assert_eq!(
        apply(
            &state,
            &command(
                &state,
                Action::StartPersonalQuest {
                    quest_id: "pip-lantern-trail".into()
                }
            ),
            &c,
            &context()
        ),
        Err(Rejection::Locked)
    );
    let before = state.clone();
    let mut corrupted = state;
    corrupted.personal_quest.as_mut().unwrap().node_id = "unknown-node".into();
    assert!(corrupted.validate_personal_state(&c).is_err());
    assert_eq!(before.personal_quest_view().unwrap().node.id(), "start");
}

#[test]
fn personal_activity_abandonment_and_wrong_completion_never_advance_or_grant() {
    let c = content();
    let mut state = initial(&c);
    let mut log = Vec::new();
    perform(
        &mut state,
        Action::StartPersonalQuest {
            quest_id: "pip-lantern-trail".into(),
        },
        &c,
        context(),
        &mut log,
    );
    choose(&mut state, &c, 0, &mut log);
    let expected = state.personal_quest_view().unwrap();
    let (quest_id, encounter_id) = begin(&mut state, &c, &mut log);
    assert_eq!(
        apply(
            &state,
            &command(&state, Action::AbandonPersonalQuest),
            &c,
            &context()
        ),
        Err(Rejection::Locked)
    );
    let mut wrong = context();
    wrong.facts.completed_encounters.insert(
        encounter_id.clone(),
        VerifiedCompletion {
            quest_id: "training-pip-1".into(),
            supported: false,
        },
    );
    assert_eq!(
        apply(
            &state,
            &command(
                &state,
                Action::CompleteQuest {
                    quest_id: quest_id.clone(),
                    encounter_id: encounter_id.clone()
                }
            ),
            &c,
            &wrong
        ),
        Err(Rejection::UnverifiedEncounter)
    );
    perform(
        &mut state,
        Action::AbandonQuest {
            quest_id,
            encounter_id,
        },
        &c,
        context(),
        &mut log,
    );
    assert_eq!(state.personal_quest_view().unwrap().node, expected.node);
    assert!(state.inventory.is_empty());
    let reloaded: State = serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
    state = reloaded;
    complete(&mut state, &c, true, &mut log);
    choose(&mut state, &c, 1, &mut log);
    assert_eq!(state.inventory.get("pip-personal-memory"), Some(&1));
}

#[test]
fn active_document_and_activity_policy_are_pinned_across_catalog_edits() {
    let c = content();
    let mut state = initial(&c);
    let mut log = Vec::new();
    perform(
        &mut state,
        Action::StartPersonalQuest {
            quest_id: "pip-lantern-trail".into(),
        },
        &c,
        context(),
        &mut log,
    );
    choose(&mut state, &c, 0, &mut log);
    let old = state.personal_quest_view().unwrap();
    let binding = state.active_personal_activity().unwrap();
    let mut catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../../content/quests/catalog.v1.json")).unwrap();
    catalog["activitySlots"][0]["skillId"] = serde_json::json!("M08");
    catalog["activitySlots"][0]["templateId"] = serde_json::json!("equal_sharing");
    let mut changed_doc: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../content/quests/pip-lantern-trail.v1.json"
    ))
    .unwrap();
    changed_doc["documentVersion"] = serde_json::json!("2");
    changed_doc["title"] = serde_json::json!("An edited future story");
    let mut changed = c.clone();
    changed.personal_quests = Catalog::load(
        &catalog.to_string(),
        &[
            &changed_doc.to_string(),
            include_str!("../../../../content/quests/tansy-welcome-garden.v1.json"),
            include_str!("../../../../content/quests/nib-sharing-picnic.v1.json"),
            include_str!("../../../../content/quests/flutter-pattern-parade.v1.json"),
        ],
    )
    .unwrap();
    assert_eq!(state.personal_quest_view().unwrap().node, old.node);
    assert_eq!(state.personal_quest_view().unwrap().document_version, "1");
    assert_eq!(state.active_personal_activity().unwrap(), binding);
    complete(&mut state, &changed, true, &mut log);
    choose(&mut state, &changed, 0, &mut log);
    assert_eq!(state.inventory.get("pip-personal-memory"), Some(&1));
}

#[test]
fn graph_validation_rejects_cycles_dangling_and_unassessed_reward_routes() {
    let c = content();
    let document = c.personal_quests.document("pip-lantern-trail").unwrap();
    let base = serde_json::to_value(document).unwrap();
    for (path, value) in [
        ("/nodes/0/choices/0/nextNodeId", "start"),
        ("/nodes/0/choices/0/nextNodeId", "missing-node"),
        ("/nodes/0/choices/0/nextNodeId", "handoff-stars"),
    ] {
        let mut malformed = base.clone();
        *malformed.pointer_mut(path).unwrap() = serde_json::json!(value);
        let malformed: QuestDocument = serde_json::from_value(malformed).unwrap();
        assert!(validate_document(&malformed, c.personal_quests.activity_slots()).is_err());
    }
    let mut model_grant = base;
    model_grant["rewardXp"] = serde_json::json!(999);
    assert!(serde_json::from_value::<QuestDocument>(model_grant).is_err());
}

#[test]
fn pre_personal_state_keeps_its_serialized_shape_and_no_invented_progress() {
    let c = content();
    let state = State::new(&c);
    let saved = serde_json::to_value(&state).unwrap();
    assert!(saved.get("personal_quest").is_none());
    assert!(saved.get("personal_episode_sequence").is_none());
    assert!(saved.get("personal_quest_rewards").is_none());
    let loaded: State = serde_json::from_value(saved.clone()).unwrap();
    assert_eq!(serde_json::to_value(loaded).unwrap(), saved);
}

#[test]
fn shared_destination_graphs_are_memoized_without_losing_activity_path_checks() {
    use crate::personal_quests::QuestChoice;
    let c = content();
    let mut document = c
        .personal_quests
        .document("pip-lantern-trail")
        .unwrap()
        .clone();
    document.nodes = vec![
        QuestNode::Choice {
            id: "start".into(),
            text: "Choose a path to explore together.".into(),
            choices: vec![
                QuestChoice {
                    id: "pond".into(),
                    label: "Pond path".into(),
                    next_node_id: "path-one".into(),
                },
                QuestChoice {
                    id: "hill".into(),
                    label: "Hill path".into(),
                    next_node_id: "path-two".into(),
                },
            ],
        },
        QuestNode::Activity {
            id: "path-one".into(),
            text: "Count with Pip.".into(),
            activity_slot_id: "pip-count".into(),
            next_node_id: "merge-0".into(),
        },
        QuestNode::Activity {
            id: "path-two".into(),
            text: "Count with Pip.".into(),
            activity_slot_id: "pip-count".into(),
            next_node_id: "merge-0".into(),
        },
    ];
    for index in 0..12 {
        let next = if index == 11 {
            "handoff".into()
        } else {
            format!("merge-{}", index + 1)
        };
        document.nodes.push(QuestNode::Choice {
            id: format!("merge-{index}"),
            text: "Which clue shall we remember?".into(),
            choices: ["one", "two", "three"]
                .into_iter()
                .map(|id| QuestChoice {
                    id: id.into(),
                    label: format!("Remember clue {id}"),
                    next_node_id: next.clone(),
                })
                .collect(),
        });
    }
    document.nodes.push(QuestNode::Handoff {
        id: "handoff".into(),
        text: "More paths await another day.".into(),
        continuation_tags: vec!["exploring".into()],
    });
    assert_eq!(document.nodes.len(), 16);
    validate_document(&document, c.personal_quests.activity_slots()).unwrap();
    let QuestNode::Choice { choices, .. } = &mut document.nodes[0] else {
        unreachable!()
    };
    choices.push(QuestChoice {
        id: "bypass".into(),
        label: "A shortcut".into(),
        next_node_id: "merge-0".into(),
    });
    assert!(
        validate_document(&document, c.personal_quests.activity_slots()).is_err(),
        "memoization must distinguish paths with and without assessed activity"
    );
}

#[test]
fn restored_pinned_activity_slots_reject_unknown_mismatched_and_duplicate_bindings() {
    let c = content();
    let mut state = initial(&c);
    perform(
        &mut state,
        Action::StartPersonalQuest {
            quest_id: "pip-lantern-trail".into(),
        },
        &c,
        context(),
        &mut Vec::new(),
    );
    let run = state.personal_quest.unwrap();
    for variant in 0..3 {
        let mut malformed = run.clone();
        match variant {
            0 => malformed.slots[0].skill_id = "M99".into(),
            1 => malformed.slots[0].template_id = "equal_sharing".into(),
            _ => malformed.slots.push(malformed.slots[0].clone()),
        }
        assert!(malformed.validate().is_err());
    }
}
