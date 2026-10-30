use super::*;

fn command(state: &State, action: Action) -> Command {
    Command {
        command_id: format!("cmd-{}", state.revision()),
        expected_revision: state.revision(),
        action,
    }
}
fn perform(state: &mut State, action: Action) -> Receipt {
    state.apply(&command(state, action)).unwrap()
}
fn travel(destination: &str) -> Action {
    Action::Travel {
        destination_id: destination.into(),
    }
}
fn collect(source: &str, quantity: u32) -> Action {
    Action::Collect {
        source_id: source.into(),
        item_id: "wood".into(),
        quantity,
    }
}
fn exchange() -> Action {
    Action::Exchange {
        character_id: "woodcutter".into(),
        recipe_id: "wood_to_planks".into(),
    }
}
fn assemble() -> Action {
    Action::Assemble {
        object_id: "bridge".into(),
        recipe_id: "repair_bridge".into(),
    }
}
fn cross() -> Action {
    Action::Accompany {
        character_id: "pip".into(),
        destination_id: "far_bank".into(),
    }
}
fn completed(state: &State, id: &str) -> bool {
    state
        .milestones()
        .iter()
        .any(|m| m.condition_id == id && m.complete)
}
fn ready_bridge(source: &str) -> State {
    let mut s = State::new();
    perform(&mut s, Action::AcceptMission);
    if source == "spare_wood" {
        perform(&mut s, travel("workshop"));
    }
    perform(&mut s, collect(source, 3));
    if source == "fallen_wood" {
        perform(&mut s, travel("workshop"));
    }
    perform(&mut s, exchange());
    assert_eq!((s.wood(), s.planks()), (0, 3));
    assert!(completed(&s, "wood_ready"));
    perform(&mut s, travel("bridge_bank"));
    perform(&mut s, assemble());
    s
}

#[test]
fn both_sources_complete_with_conserved_inventory_and_explicit_outcome() {
    for source in ["fallen_wood", "spare_wood"] {
        let mut s = ready_bridge(source);
        assert!(s.bridge_repaired());
        assert_eq!((s.wood(), s.planks()), (0, 0));
        assert_eq!(s.mission_status(), MissionStatus::Active);
        assert!(!s.crossed_with_pip());
        assert_eq!(s.source_remaining(source), Some(0));
        assert!(s.available_actions().contains(&cross()));
        perform(&mut s, cross());
        assert_eq!(s.location(), Location::FarBank);
        assert_eq!(s.pip_location(), Location::FarBank);
        assert_eq!(s.mission_status(), MissionStatus::Completed);
        assert!(s.milestones().iter().all(|m| m.complete));
        assert!(!s.available_actions().contains(&assemble()));
    }
}

#[test]
fn invalid_bindings_locations_resources_and_quantities_are_atomic() {
    let mut s = State::new();
    perform(&mut s, Action::AcceptMission);
    for action in [
        travel("far_bank"),
        travel("invented"),
        collect("spare_wood", 3),
        collect("fallen_wood", 0),
        collect("fallen_wood", 4),
        exchange(),
        assemble(),
        cross(),
        Action::Exchange {
            character_id: "pip".into(),
            recipe_id: "wood_to_planks".into(),
        },
        Action::Assemble {
            object_id: "bridge".into(),
            recipe_id: "magic".into(),
        },
    ] {
        let before = s.clone();
        assert!(s.apply(&command(&s, action)).is_err());
        assert_eq!(s, before);
    }
    perform(&mut s, collect("fallen_wood", 1));
    assert_eq!(s.source_remaining("fallen_wood"), Some(2));
    assert!(s.available_actions().contains(&collect("fallen_wood", 2)));
    let before = s.clone();
    assert!(s.apply(&command(&s, collect("fallen_wood", 3))).is_err());
    assert_eq!(s, before);
    perform(&mut s, collect("fallen_wood", 2));
    assert_eq!(s.wood(), 3);
}

#[test]
fn retries_reuse_receipt_and_stale_or_changed_commands_fail() {
    let mut s = State::new();
    let accept = command(&s, Action::AcceptMission);
    let receipt = s.apply(&accept).unwrap();
    perform(&mut s, collect("fallen_wood", 3));
    let before = s.clone();
    assert_eq!(s.apply(&accept), Ok(receipt));
    assert_eq!(s, before);
    let altered = Command {
        action: travel("workshop"),
        ..accept.clone()
    };
    assert_eq!(s.apply(&altered), Err(Rejection::CommandIdReused));
    let stale = Command {
        command_id: "new-id".into(),
        ..accept
    };
    assert_eq!(s.apply(&stale), Err(Rejection::RevisionConflict));
    assert_eq!(s, before);
}

#[test]
fn existing_inventory_and_repaired_world_do_not_require_obsolete_work() {
    let mut wood = State::with_initial(InitialConditions {
        wood: 3,
        ..Default::default()
    })
    .unwrap();
    perform(&mut wood, Action::AcceptMission);
    assert!(completed(&wood, "wood_ready"));
    perform(&mut wood, travel("workshop"));
    perform(&mut wood, exchange());
    assert!(completed(&wood, "wood_ready"));
    for initial in [
        InitialConditions {
            planks: 3,
            ..Default::default()
        },
        InitialConditions {
            bridge_repaired: true,
            ..Default::default()
        },
    ] {
        let mut s = State::with_initial(initial).unwrap();
        perform(&mut s, Action::AcceptMission);
        assert!(
            !s.milestones()
                .iter()
                .any(|m| m.condition_id == "wood_ready")
        );
        if !s.bridge_repaired() {
            perform(&mut s, assemble());
        }
        perform(&mut s, cross());
        assert_eq!(s.mission_status(), MissionStatus::Completed);
        assert!(s.milestones().iter().all(|m| m.complete));
    }
}

#[test]
fn cancellation_and_solo_crossing_preserve_world_without_finishing_mission() {
    let mut s = ready_bridge("fallen_wood");
    perform(&mut s, Action::CancelMission);
    assert_eq!(s.mission_status(), MissionStatus::Abandoned);
    assert!(s.bridge_repaired());
    assert_eq!(s.planks(), 0);
    perform(&mut s, travel("far_bank"));
    assert_eq!(s.pip_location(), Location::BridgeBank);
    assert!(!s.crossed_with_pip());
    perform(&mut s, Action::AcceptMission);
    assert_eq!(s.mission_status(), MissionStatus::Active);
    perform(&mut s, travel("bridge_bank"));
    perform(&mut s, cross());
    assert_eq!(s.mission_status(), MissionStatus::Completed);
    assert_eq!(s.planks(), 0);
}

#[test]
fn detour_and_resume_keep_partial_progress_and_supplies() {
    let mut s = State::new();
    perform(&mut s, Action::AcceptMission);
    perform(&mut s, collect("fallen_wood", 3));
    perform(&mut s, Action::CancelMission);
    perform(&mut s, travel("workshop"));
    perform(&mut s, Action::AcceptMission);
    assert_eq!(s.wood(), 3);
    assert!(completed(&s, "wood_ready"));
    perform(&mut s, exchange());
    perform(&mut s, travel("bridge_bank"));
    perform(&mut s, assemble());
    perform(&mut s, cross());
    assert!(s.crossed_with_pip());
}

#[test]
fn introduction_is_recorded_only_by_current_canonical_entry_acknowledgement() {
    let mut s = State::new();
    perform(&mut s, travel("workshop"));
    let scene = s.presentation();
    assert!(!s.met_woodcutter());
    assert_eq!(scene.entries[0].speaker_id, "woodcutter");
    let invalid = Action::Acknowledge {
        scene_id: scene.scene_id.clone(),
        entry_id: "invented".into(),
    };
    let before = s.clone();
    assert_eq!(
        s.apply(&command(&s, invalid)),
        Err(Rejection::InvalidAction)
    );
    assert_eq!(s, before);
    let ack = command(
        &s,
        Action::Acknowledge {
            scene_id: scene.scene_id,
            entry_id: scene.entries[0].entry_id.clone(),
        },
    );
    perform(&mut s, Action::AcceptMission);
    let stale = Command {
        command_id: "stale-scene".into(),
        expected_revision: s.revision(),
        ..ack
    };
    assert_eq!(s.apply(&stale), Err(Rejection::NotAvailable));
    let scene = s.presentation();
    perform(
        &mut s,
        Action::Acknowledge {
            scene_id: scene.scene_id,
            entry_id: scene.entries[0].entry_id.clone(),
        },
    );
    assert!(s.met_woodcutter());
    assert_eq!((s.wood(), s.planks()), (0, 0));
    assert_eq!(
        s.presentation().entries[0].text,
        "I can help shape wood into sturdy planks."
    );
}

#[test]
fn checkpoint_replays_commands_and_rejects_tampered_results_and_unknown_fields() {
    let mut s = ready_bridge("spare_wood");
    let checkpoint = s.checkpoint();
    assert_eq!(State::from_checkpoint(&checkpoint), Ok(s.clone()));
    let mut restored = State::from_checkpoint(&checkpoint).unwrap();
    perform(&mut restored, cross());
    perform(&mut s, cross());
    assert_eq!(restored, s);
    for mutation in [
        "state",
        "receipt",
        "initial",
        "command",
        "extra",
        "duplicate",
    ] {
        let mut bad = checkpoint.clone();
        match mutation {
            "state" => bad["state"]["planks"] = json!(900),
            "receipt" => bad["receipts"]["cmd-0"]["description"] = json!("forged"),
            "initial" => bad["initial"]["wood"] = json!(4),
            "command" => bad["commands"][0]["action"]["extra"] = json!(true),
            "extra" => bad["unknown"] = json!(true),
            "duplicate" => {
                let first = bad["commands"][0].clone();
                bad["commands"].as_array_mut().unwrap().push(first);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            State::from_checkpoint(&bad),
            Err(Rejection::InvalidCheckpoint),
            "{mutation}"
        );
    }
}

#[test]
fn command_and_checkpoint_size_limits_are_enforced() {
    let mut s = State::new();
    for i in 0..MAX_COMMANDS {
        let destination = if i % 2 == 0 {
            "workshop"
        } else {
            "bridge_bank"
        };
        perform(&mut s, travel(destination));
    }
    assert_eq!(
        s.apply(&command(&s, travel("workshop"))),
        Err(Rejection::LimitReached)
    );
    let mut huge = s.checkpoint();
    huge["state"]["extra"] = json!("x".repeat(MAX_CHECKPOINT_BYTES));
    assert_eq!(
        State::from_checkpoint(&huge),
        Err(Rejection::InvalidCheckpoint)
    );
    let first = Command {
        command_id: "cmd-0".into(),
        expected_revision: 0,
        action: travel("workshop"),
    };
    assert!(s.apply(&first).is_ok());
}

#[test]
fn presentation_never_speaks_as_an_absent_pip_or_invents_building_history() {
    let mut s = State::with_initial(InitialConditions {
        bridge_repaired: true,
        ..Default::default()
    })
    .unwrap();
    perform(&mut s, Action::AcceptMission);
    perform(&mut s, cross());
    assert!(s.presentation().entries[0].text.contains("crossing"));
    perform(&mut s, travel("bridge_bank"));
    assert_eq!(s.pip_location(), Location::FarBank);
    assert!(s.presentation().entries[0].speaker_id.is_empty());
}

fn bound_collect(source: &str, item: &str) -> Action {
    Action::Collect {
        source_id: source.into(),
        item_id: item.into(),
        quantity: 3,
    }
}
fn recipe(id: &str) -> Action {
    Action::Exchange {
        character_id: "woodcutter".into(),
        recipe_id: id.into(),
    }
}
fn build(object: &str, recipe: &str) -> Action {
    Action::Assemble {
        object_id: object.into(),
        recipe_id: recipe.into(),
    }
}
fn finish_bridge() -> State {
    let mut state = ready_bridge("fallen_wood");
    perform(&mut state, cross());
    state
}
fn picnic_actions() -> Vec<Action> {
    vec![
        Action::AcceptMission,
        bound_collect("berry_bush", "berries"),
        travel("bridge_bank"),
        travel("workshop"),
        recipe("berries_to_picnic"),
        travel("bridge_bank"),
        build("picnic_table", "prepare_picnic"),
        travel("far_bank"),
        Action::Accompany {
            character_id: "pip".into(),
            destination_id: "bridge_bank".into(),
        },
    ]
}
fn lights_actions() -> Vec<Action> {
    vec![
        Action::AcceptMission,
        travel("far_bank"),
        bound_collect("glow_stones", "glow_pebbles"),
        travel("bridge_bank"),
        travel("workshop"),
        recipe("glow_to_lanterns"),
        travel("bridge_bank"),
        build("near_lanterns", "light_near_bank"),
        travel("far_bank"),
        build("far_lantern", "light_far_bank"),
    ]
}

#[test]
fn three_connected_missions_require_explicit_acceptance_and_preserve_all_outcomes() {
    let mut state = finish_bridge();
    assert_eq!(state.mission(), Mission::Bridge);
    assert_eq!(state.next_mission(), Some(Mission::Picnic));
    assert_eq!(state.completed_missions(), &[Mission::Bridge]);
    assert!(state.available_actions().contains(&Action::AcceptMission));
    perform(&mut state, travel("bridge_bank"));
    let before = state.clone();
    assert_eq!(
        state.apply(&command(&state, Action::AcceptMission)),
        Err(Rejection::NotAvailable)
    );
    assert_eq!(state, before);
    perform(&mut state, travel("far_bank"));
    for action in picnic_actions() {
        perform(&mut state, action);
        state = State::from_checkpoint(&state.checkpoint()).unwrap();
        assert!(state.bridge_repaired());
        assert!(state.crossed_with_pip());
    }
    assert_eq!(state.mission(), Mission::Picnic);
    assert_eq!(state.mission_status(), MissionStatus::Completed);
    assert_eq!(state.next_mission(), Some(Mission::WelcomeLights));
    assert_eq!(
        state.completed_missions(),
        &[Mission::Bridge, Mission::Picnic]
    );
    assert!(state.picnic_ready() && state.picnic_shared());
    assert_eq!((state.berries(), state.picnic_basket()), (0, 0));
    assert!(state.milestones().iter().all(|m| m.complete));
    for action in lights_actions() {
        perform(&mut state, action);
        state = State::from_checkpoint(&state.checkpoint()).unwrap();
        assert!(state.bridge_repaired() && state.picnic_shared());
    }
    assert_eq!(state.mission(), Mission::WelcomeLights);
    assert_eq!(state.mission_status(), MissionStatus::Completed);
    assert_eq!(state.next_mission(), None);
    assert_eq!(
        state.completed_missions(),
        &[Mission::Bridge, Mission::Picnic, Mission::WelcomeLights]
    );
    assert!(state.near_lit() && state.far_lit());
    assert_eq!((state.glow_pebbles(), state.lanterns()), (0, 0));
    assert!(state.inventory().is_empty());
    assert!(state.milestones().iter().all(|m| m.complete));
    assert!(!state.available_actions().contains(&Action::AcceptMission));
    perform(&mut state, travel("bridge_bank"));
    assert_eq!(state.presentation().entries[0].speaker_id, "pip");
    assert!(state.presentation().entries[0].text.contains("Both banks"));
}

#[test]
fn new_resources_recipes_and_world_objects_are_bound_to_their_active_mission() {
    let mut state = State::new();
    perform(&mut state, Action::AcceptMission);
    for action in [
        bound_collect("berry_bush", "berries"),
        bound_collect("glow_stones", "glow_pebbles"),
        recipe("berries_to_picnic"),
        build("picnic_table", "prepare_picnic"),
        build("near_lanterns", "light_near_bank"),
    ] {
        let before = state.clone();
        assert!(state.apply(&command(&state, action)).is_err());
        assert_eq!(state, before);
    }
    let mut state = finish_bridge();
    perform(&mut state, Action::AcceptMission);
    for action in [
        bound_collect("glow_stones", "glow_pebbles"),
        bound_collect("berry_bush", "wood"),
        build("picnic_table", "prepare_picnic"),
    ] {
        let before = state.clone();
        assert!(state.apply(&command(&state, action)).is_err());
        assert_eq!(state, before);
    }
    perform(&mut state, bound_collect("berry_bush", "berries"));
    perform(&mut state, travel("bridge_bank"));
    perform(&mut state, travel("workshop"));
    let before = state.clone();
    assert_eq!(
        state.apply(&command(&state, recipe("glow_to_lanterns"))),
        Err(Rejection::NotAvailable)
    );
    assert_eq!(state, before);
    let exchange = command(&state, recipe("berries_to_picnic"));
    let receipt = state.apply(&exchange).unwrap();
    assert_eq!(state.apply(&exchange), Ok(receipt));
    assert_eq!((state.berries(), state.picnic_basket()), (0, 1));
    let after = state.clone();
    assert_eq!(
        state.apply(&command(&state, recipe("berries_to_picnic"))),
        Err(Rejection::InsufficientInventory)
    );
    assert_eq!(state, after);
}

#[test]
fn pauses_preserve_new_milestones_and_partial_world_changes_without_refunds() {
    let mut state = finish_bridge();
    for action in picnic_actions() {
        perform(&mut state, action);
    }
    for action in lights_actions().into_iter().take(8) {
        perform(&mut state, action);
    }
    assert!(state.near_lit());
    assert!(!state.far_lit());
    assert_eq!(state.lanterns(), 1);
    perform(&mut state, Action::CancelMission);
    perform(&mut state, travel("far_bank"));
    let saved = state.checkpoint();
    let mut state = State::from_checkpoint(&saved).unwrap();
    assert_eq!(state.mission_status(), MissionStatus::Abandoned);
    assert_eq!(state.mission(), Mission::WelcomeLights);
    assert_eq!(state.lanterns(), 1);
    assert!(
        state
            .milestones()
            .iter()
            .find(|m| m.condition_id == "lanterns_ready")
            .unwrap()
            .complete
    );
    assert!(
        !state
            .available_actions()
            .contains(&build("far_lantern", "light_far_bank"))
    );
    perform(&mut state, Action::AcceptMission);
    perform(&mut state, build("far_lantern", "light_far_bank"));
    assert!(state.near_lit() && state.far_lit());
    assert_eq!(state.lanterns(), 0);
    assert_eq!(state.mission_status(), MissionStatus::Completed);
    let after = state.clone();
    assert!(
        state
            .apply(&command(&state, build("far_lantern", "light_far_bank")))
            .is_err()
    );
    assert_eq!(state, after);
}

#[test]
fn alternate_lantern_order_and_solo_picnic_return_do_not_skip_outcomes() {
    let mut state = finish_bridge();
    for action in picnic_actions().into_iter().take(8) {
        perform(&mut state, action);
    }
    assert!(state.picnic_ready());
    perform(&mut state, travel("bridge_bank"));
    assert!(!state.picnic_shared());
    assert_eq!(state.mission_status(), MissionStatus::Active);
    perform(&mut state, travel("far_bank"));
    perform(
        &mut state,
        Action::Accompany {
            character_id: "pip".into(),
            destination_id: "bridge_bank".into(),
        },
    );
    for action in lights_actions().into_iter().take(7) {
        perform(&mut state, action);
    }
    perform(&mut state, travel("far_bank"));
    perform(&mut state, build("far_lantern", "light_far_bank"));
    assert!(!state.near_lit());
    assert!(state.far_lit());
    assert_eq!(state.lanterns(), 2);
    assert_eq!(state.mission_status(), MissionStatus::Active);
    perform(&mut state, travel("bridge_bank"));
    perform(&mut state, build("near_lanterns", "light_near_bank"));
    assert_eq!(state.mission_status(), MissionStatus::Completed);
}

#[test]
fn literal_legacy_completed_checkpoint_migrates_without_reset_and_can_continue() {
    let legacy: Value =
        serde_json::from_str(include_str!("fixtures/bridge-completed-v1.json")).unwrap();
    assert_eq!(legacy["version"], "pip-bridge-1");
    let mut state = State::from_checkpoint(&legacy).unwrap();
    assert_eq!(state.mission(), Mission::Bridge);
    assert_eq!(state.mission_status(), MissionStatus::Completed);
    assert_eq!(state.revision(), 7);
    assert!(state.bridge_repaired() && state.crossed_with_pip());
    assert_eq!(state.completed_missions(), &[Mission::Bridge]);
    assert_eq!(state.legacy_checkpoint(), legacy);
    assert_eq!(state.checkpoint()["version"], "pip-stories-2");
    assert_eq!(
        State::from_checkpoint(&state.checkpoint()),
        Ok(state.clone())
    );
    let original: Command = serde_json::from_value(legacy["commands"][0].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(state.apply(&original).unwrap()).unwrap(),
        legacy["receipts"]["cmd-0"]
    );
    for action in picnic_actions() {
        perform(&mut state, action);
    }
    assert_eq!(state.mission(), Mission::Picnic);
    assert!(state.picnic_shared());
    assert!(State::from_checkpoint(&state.checkpoint()).is_ok());
}

#[test]
fn legacy_reader_rejects_new_semantics_and_tampered_legacy_receipts() {
    let mut legacy: Value =
        serde_json::from_str(include_str!("fixtures/bridge-completed-v1.json")).unwrap();
    legacy["receipts"]["cmd-6"]["description"] = json!("invented crossing");
    assert_eq!(
        State::from_checkpoint(&legacy),
        Err(Rejection::InvalidCheckpoint)
    );
    let mut state = finish_bridge();
    perform(&mut state, Action::AcceptMission);
    assert_eq!(state.mission(), Mission::Picnic);
    // Even a legacy-shaped projection and matching receipts cannot admit new v2 actions.
    assert_eq!(
        State::from_checkpoint(&state.legacy_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
    let mut state = finish_bridge();
    for action in picnic_actions() {
        perform(&mut state, action);
    }
    let mut bad = state.checkpoint();
    bad["state"]["completedMissions"] = json!(["pips_bridge"]);
    assert_eq!(
        State::from_checkpoint(&bad),
        Err(Rejection::InvalidCheckpoint)
    );
}
