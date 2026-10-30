use super::*;

fn command(state: &State, id: &str, kind: &str) -> Value {
    let view = snapshot(state).unwrap();
    let action = view["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["type"] == kind)
        .unwrap();
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":state.revision(),"payload":{"type":"choose","actionId":action["id"]}})
}

#[test]
fn scene_and_acknowledgement_are_separate_and_current() {
    let state = State::new();
    let initial = snapshot(&state).unwrap();
    assert_eq!(initial["acknowledgeableEntryIds"], json!([]));
    assert_eq!(state.revision(), 0);
    let (state, _) = execute(&state, &command(&state, "accept", "accept_mission")).unwrap();
    let (state, view) = execute(&state, &command(&state, "travel", "travel")).unwrap();
    assert_eq!(state.location(), Location::Workshop);
    assert!(!state.met_woodcutter());
    assert_eq!(view["acknowledgeableEntryIds"], json!(["greeting"]));
    let ack = json!({"schemaVersion":"1.0","commandId":"ack","expectedRevision":state.revision(),"payload":{"type":"acknowledge","sceneId":view["scene"]["sceneId"],"entryId":"greeting"}});
    let (next, after) = execute(&state, &ack).unwrap();
    assert!(next.met_woodcutter());
    assert!(!state.met_woodcutter());
    assert_eq!(after["acknowledgeableEntryIds"], json!([]));
    assert_eq!(next.wood(), state.wood());
    assert_eq!(next.planks(), state.planks());
    let mut repeated = ack;
    repeated["commandId"] = json!("new-ack");
    repeated["expectedRevision"] = json!(next.revision());
    repeated["payload"]["sceneId"] = after["scene"]["sceneId"].clone();
    assert!(matches!(
        execute(&next, &repeated),
        Err(Error::RevisionConflict)
    ));
}

#[test]
fn command_boundary_rejects_arbitrary_state_and_stale_handles() {
    let state = State::new();
    let request = command(&state, "accept", "accept_mission");
    let mut forged = request.clone();
    forged["checkpoint"] = state.checkpoint();
    assert!(matches!(execute(&state, &forged), Err(Error::InvalidInput)));
    let mut forged = request.clone();
    forged["payload"] =
        json!({"type":"collect","sourceId":"fallen_wood","quantity":3,"itemId":"wood"});
    assert!(matches!(execute(&state, &forged), Err(Error::InvalidInput)));
    let (next, _) = execute(&state, &request).unwrap();
    assert!(matches!(
        execute(&next, &request),
        Err(Error::RevisionConflict)
    ));
    let mut stale_handle = request;
    stale_handle["expectedRevision"] = json!(next.revision());
    stale_handle["commandId"] = json!("fresh-id");
    assert!(matches!(
        execute(&next, &stale_handle),
        Err(Error::RevisionConflict)
    ));
    assert_eq!(state.revision(), 0);
}

#[test]
fn exhausted_budget_preserves_readable_terminal_snapshot() {
    let mut state = State::new();
    for i in 0..wonderwilds_domain::story::MAX_COMMANDS {
        state
            .apply(&Command {
                command_id: format!("travel-{i}"),
                expected_revision: state.revision(),
                action: Action::Travel {
                    destination_id: if i % 2 == 0 {
                        "workshop"
                    } else {
                        "bridge_bank"
                    }
                    .into(),
                },
            })
            .unwrap();
    }
    let view = snapshot(&state).unwrap();
    assert_eq!(view["commandBudgetExhausted"], true);
    assert_eq!(view["revision"], 256);
    assert!(view.get("sceneInput").is_none());
    assert!(view.get("scene").is_none());
    assert_eq!(view["acknowledgeableEntryIds"], json!([]));
    let request = json!({"schemaVersion":"1.0","commandId":"new","expectedRevision":256,"payload":{"type":"choose","actionId":"story-256-0"}});
    assert!(matches!(execute(&state, &request), Err(Error::RateLimited)));
}
