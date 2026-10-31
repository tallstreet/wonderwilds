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
    assert_eq!(state.next_mission(), Some(Mission::HummingHollow));
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
    assert_eq!(state.checkpoint()["version"], "pip-stories-3");
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

fn inspect(object: &str) -> Action {
    Action::Inspect {
        object_id: object.into(),
    }
}
fn talk(character: &str, topic: &str) -> Action {
    Action::Talk {
        character_id: character.into(),
        topic_id: topic.into(),
    }
}
fn choose(choice: &str, option: &str) -> Action {
    Action::Choose {
        choice_id: choice.into(),
        option_id: option.into(),
    }
}
fn invite(character: &str) -> Action {
    Action::Invite {
        character_id: character.into(),
    }
}
fn finish_lights_v2() -> State {
    let legacy: Value =
        serde_json::from_str(include_str!("fixtures/lights-completed-v2.json")).unwrap();
    State::from_checkpoint(&legacy).unwrap()
}
fn replay_step(state: &mut State, action: Action) {
    let command = command(state, action);
    let receipt = state.apply(&command).unwrap();
    let after = state.clone();
    assert_eq!(state.apply(&command), Ok(receipt));
    assert_eq!(*state, after);
    *state = State::from_checkpoint(&state.checkpoint()).unwrap();
    assert!(state.available_actions().len() <= 12);
}
fn humming_actions(home: &str) -> Vec<Action> {
    let mut actions = vec![
        travel("bridge_bank"),
        Action::AcceptMission,
        talk("pip", "humming_hollow"),
        travel("far_bank"),
        inspect("humming_hollow"),
        inspect("mossy_stump"),
        inspect("trembling_leaf"),
        inspect("hollow_branch"),
        choose("instrument_home", home),
    ];
    if home == "gathering" {
        actions.push(travel("bridge_bank"));
    }
    actions.push(build("wind_instrument", "place_instrument"));
    if home == "woods" {
        actions.push(travel("bridge_bank"));
    }
    actions.push(talk("pip", "share_discovery"));
    actions
}

fn corner_actions(style: &str) -> Vec<Action> {
    vec![
        Action::AcceptMission,
        talk("pip", "cosy_preferences"),
        inspect("corner_site"),
        choose("corner_style", style),
        build("corner_frame", "build_corner_frame"),
        build("corner_cushions", "arrange_cushions"),
        talk("pip", "enjoy_corner"),
    ]
}
fn visit_actions() -> Vec<Action> {
    vec![
        Action::AcceptMission,
        travel("workshop"),
        talk("bramble", "visit_wish"),
        inspect("workshop_path"),
        build("workshop_path", "clear_path"),
        talk("bramble", "ready_to_visit"),
        Action::Accompany {
            character_id: "bramble".into(),
            destination_id: "bridge_bank".into(),
        },
        talk("bramble", "visit_arrival"),
    ]
}

#[test]
fn all_four_broader_missions_replay_choices_orders_and_distinct_outcomes() {
    for home in ["woods", "gathering"] {
        for style in ["leafy_nook", "star_canopy"] {
            for venue in ["near_bank", "far_bank"] {
                for reverse in [false, true] {
                    let mut state = finish_lights_v2();
                    for action in humming_actions(home) {
                        replay_step(&mut state, action);
                    }
                    assert_eq!(state.mission(), Mission::HummingHollow);
                    assert_eq!(state.mission_status(), MissionStatus::Completed);
                    let instrument = state.instrument_home();
                    assert_eq!(
                        instrument,
                        Some(if home == "woods" {
                            Location::FarBank
                        } else {
                            Location::BridgeBank
                        })
                    );
                    for action in corner_actions(style) {
                        replay_step(&mut state, action);
                    }
                    assert_eq!(state.corner_style(), Some(style));
                    assert!(state.has_story_flag("corner_shared"));
                    for action in visit_actions() {
                        replay_step(&mut state, action);
                    }
                    assert_eq!(state.bramble_location(), Location::BridgeBank);
                    assert!(state.has_story_flag("bramble_visit_shared"));
                    replay_step(&mut state, Action::AcceptMission);
                    replay_step(&mut state, choose("gathering_place", venue));
                    let order = if reverse {
                        ["bramble", "pip"]
                    } else {
                        ["pip", "bramble"]
                    };
                    for character in order {
                        replay_step(&mut state, invite(character));
                    }
                    if venue == "far_bank" {
                        replay_step(&mut state, travel("far_bank"));
                    }
                    let preparations = if reverse {
                        [
                            ("gathering_music", "prepare_music"),
                            ("gathering_seats", "arrange_seats"),
                        ]
                    } else {
                        [
                            ("gathering_seats", "arrange_seats"),
                            ("gathering_music", "prepare_music"),
                        ]
                    };
                    for (object, recipe) in preparations {
                        replay_step(&mut state, build(object, recipe));
                    }
                    assert_eq!(state.mission_status(), MissionStatus::Active);
                    replay_step(&mut state, talk("pip", "welcome_friends"));
                    assert_eq!(state.completed_missions().len(), 7);
                    assert_eq!(state.next_mission(), None);
                    assert_eq!(state.mission_status(), MissionStatus::Completed);
                    assert!(state.milestones().iter().all(|m| m.complete));
                    assert_eq!(
                        state.instrument_home(),
                        instrument,
                        "Gathering music must not move the chosen woodland instrument"
                    );
                    assert_eq!(state.corner_style(), Some(style));
                    assert_eq!(state.pip_location(), state.location());
                    assert_eq!(state.bramble_location(), state.location());
                    assert!(state.near_lit() && state.far_lit() && state.picnic_shared());
                }
            }
        }
    }
}

#[test]
fn broader_prerequisites_pause_and_immutable_choices_fail_atomically() {
    let mut state = finish_lights_v2();
    replay_step(&mut state, travel("bridge_bank"));
    replay_step(&mut state, Action::AcceptMission);
    let reject = |state: &mut State, action| {
        let before = state.clone();
        assert!(state.apply(&command(state, action)).is_err());
        assert_eq!(*state, before);
    };
    reject(&mut state, talk("pip", "share_discovery"));
    reject(&mut state, choose("instrument_home", "woods"));
    reject(&mut state, talk("woodcutter", "humming_hollow"));
    replay_step(&mut state, talk("pip", "humming_hollow"));
    replay_step(&mut state, travel("far_bank"));
    reject(&mut state, inspect("hollow_branch"));
    replay_step(&mut state, inspect("humming_hollow"));
    replay_step(&mut state, Action::CancelMission);
    reject(&mut state, inspect("trembling_leaf"));
    replay_step(&mut state, Action::AcceptMission);
    assert!(state.has_story_flag("hollow_listened"));
    replay_step(&mut state, inspect("trembling_leaf"));
    replay_step(&mut state, inspect("hollow_branch"));
    reject(&mut state, choose("instrument_home", "unknown"));
    replay_step(&mut state, choose("instrument_home", "woods"));
    reject(&mut state, choose("instrument_home", "gathering"));
    replay_step(&mut state, build("wind_instrument", "place_instrument"));
    replay_step(&mut state, travel("bridge_bank"));
    replay_step(&mut state, talk("pip", "share_discovery"));
    for action in corner_actions("leafy_nook") {
        replay_step(&mut state, action);
    }
    replay_step(&mut state, Action::AcceptMission);
    replay_step(&mut state, travel("workshop"));
    reject(&mut state, talk("bramble", "ready_to_visit"));
    reject(
        &mut state,
        Action::Accompany {
            character_id: "bramble".into(),
            destination_id: "bridge_bank".into(),
        },
    );
    for action in visit_actions().into_iter().skip(2) {
        replay_step(&mut state, action);
    }
    replay_step(&mut state, Action::AcceptMission);
    reject(&mut state, invite("pip"));
    replay_step(&mut state, choose("gathering_place", "far_bank"));
    replay_step(&mut state, invite("pip"));
    reject(&mut state, invite("pip"));
    replay_step(&mut state, Action::CancelMission);
    reject(&mut state, invite("bramble"));
    replay_step(&mut state, Action::AcceptMission);
    assert!(state.has_story_flag("pip_invited"));
    assert_eq!(state.pip_location(), Location::FarBank);
    replay_step(&mut state, travel("far_bank"));
    reject(&mut state, talk("pip", "welcome_friends"));
    reject(&mut state, invite("bramble"));
    replay_step(&mut state, build("gathering_seats", "arrange_seats"));
    replay_step(&mut state, build("gathering_music", "prepare_music"));
    reject(&mut state, talk("pip", "welcome_friends"));
    replay_step(&mut state, travel("bridge_bank"));
    replay_step(&mut state, invite("bramble"));
    replay_step(&mut state, travel("far_bank"));
    replay_step(&mut state, talk("pip", "welcome_friends"));
    assert_eq!(state.mission_status(), MissionStatus::Completed);
}

#[test]
fn literal_v2_lights_save_upgrades_preserving_receipts_and_rejects_new_commands_as_v2() {
    let legacy: Value =
        serde_json::from_str(include_str!("fixtures/lights-completed-v2.json")).unwrap();
    assert_eq!(legacy["version"], "pip-stories-2");
    let mut state = State::from_checkpoint(&legacy).unwrap();
    assert_eq!(
        state.revision(),
        legacy["state"]["revision"].as_u64().unwrap()
    );
    let original: Command = serde_json::from_value(legacy["commands"][0].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(state.apply(&original).unwrap()).unwrap(),
        legacy["receipts"][&original.command_id]
    );
    assert_eq!(state.completed_missions().len(), 3);
    assert_eq!(state.checkpoint()["version"], "pip-stories-3");
    replay_step(&mut state, travel("bridge_bank"));
    replay_step(&mut state, Action::AcceptMission);
    assert_eq!(state.mission(), Mission::HummingHollow);
    assert_eq!(
        State::from_checkpoint(&state.v2_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
    let mut corrupt = legacy.clone();
    corrupt["state"]["farLit"] = json!(false);
    assert_eq!(
        State::from_checkpoint(&corrupt),
        Err(Rejection::InvalidCheckpoint)
    );
}

#[test]
fn instrument_transport_requires_arrival_and_placement_before_sharing() {
    let mut state = finish_lights_v2();
    for action in humming_actions("gathering").into_iter().take(9) {
        replay_step(&mut state, action);
    }
    assert_eq!(state.location(), Location::FarBank);
    assert_eq!(state.instrument_home(), Some(Location::BridgeBank));
    assert_eq!(state.quantity("wind_instrument"), 1);
    assert!(!state.has_story_flag("instrument_placed"));
    let before = state.clone();
    assert_eq!(
        state.apply(&command(
            &state,
            build("wind_instrument", "place_instrument")
        )),
        Err(Rejection::NotAvailable)
    );
    assert_eq!(state, before);
    replay_step(&mut state, travel("bridge_bank"));
    assert_eq!(
        state.apply(&command(&state, talk("pip", "share_discovery"))),
        Err(Rejection::NotAvailable)
    );
    replay_step(&mut state, build("wind_instrument", "place_instrument"));
    assert_eq!(state.quantity("wind_instrument"), 0);
    assert!(state.has_story_flag("instrument_placed"));
    replay_step(&mut state, talk("pip", "share_discovery"));
    assert_eq!(state.mission_status(), MissionStatus::Completed);
}

#[test]
fn neighbourhood_stories_cap_archive_resume_complete_and_replay() {
    let mut state = State::new();
    perform(&mut state, Action::AcceptMission);
    assert_eq!(
        state.apply(&command(&state, travel("cove"))),
        Err(Rejection::NotAvailable)
    );
    perform(&mut state, travel("workshop"));
    for p in &exploration::PLACES {
        perform(&mut state, travel(p.location.as_str()));
        let talk = Action::Talk {
            character_id: p.character.into(),
            topic_id: "start_neighbour_story".into(),
        };
        let cmd = command(&state, talk);
        let receipt = state.apply(&cmd).unwrap();
        assert_eq!(state.apply(&cmd).unwrap(), receipt);
        assert!(
            state
                .exploration
                .iter()
                .filter(|q| q.status == "open")
                .count()
                <= 3
        );
        assert_eq!(state.mission_status(), MissionStatus::Active);
        let premature = Action::Talk {
            character_id: p.character.into(),
            topic_id: "finish_neighbour_story".into(),
        };
        assert_eq!(
            state.apply(&command(&state, premature)),
            Err(Rejection::NotAvailable)
        );
        perform(
            &mut state,
            Action::Inspect {
                object_id: format!("neighbour_{}", p.character),
            },
        );
        assert_eq!(State::from_checkpoint(&state.checkpoint()).unwrap(), state);
        assert_eq!(
            State::from_durable_checkpoint(&state.durable_checkpoint()).unwrap(),
            state.compact()
        );
    }
    assert_eq!(state.exploration[0].status, "archived");
    for destination in ["owl_grove", "mossy_clearing", "fern_hollow", "cove"] {
        perform(&mut state, travel(destination));
    }
    perform(
        &mut state,
        Action::Talk {
            character_id: "marina".into(),
            topic_id: "start_neighbour_story".into(),
        },
    );
    let marina = state.exploration.last().unwrap();
    assert!(marina.inspected, "archiving preserves the task");
    assert_eq!(marina.status, "open");
    perform(
        &mut state,
        Action::Talk {
            character_id: "marina".into(),
            topic_id: "finish_neighbour_story".into(),
        },
    );
    assert_eq!(state.exploration.last().unwrap().status, "completed");
    assert!(!state.available_actions().iter().any(
        |a| matches!(a, Action::Talk { topic_id, .. } if topic_id == "start_neighbour_story")
    ));
    perform(&mut state, travel("workshop"));
    perform(&mut state, travel("bridge_bank"));
    assert!(!state.bridge_repaired());
    assert_eq!(state.mission_status(), MissionStatus::Active);
    assert_eq!(State::from_checkpoint(&state.checkpoint()).unwrap(), state);
}

#[test]
fn exploration_checkpoint_rejects_corruption_and_old_version_route_claims() {
    let mut state = State::new();
    perform(&mut state, travel("workshop"));
    perform(&mut state, travel("cove"));
    assert_eq!(
        State::from_checkpoint(&state.legacy_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
    assert_eq!(
        State::from_checkpoint(&state.v2_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
    perform(
        &mut state,
        Action::Talk {
            character_id: "marina".into(),
            topic_id: "start_neighbour_story".into(),
        },
    );
    let checkpoint = state.durable_checkpoint();
    for status in ["invented", "completed"] {
        let mut corrupt = checkpoint.clone();
        corrupt["state"]["exploration"][0]["status"] = json!(status);
        assert_eq!(
            State::from_durable_checkpoint(&corrupt),
            Err(Rejection::InvalidCheckpoint)
        );
    }
    let mut duplicate = checkpoint;
    let quest = duplicate["state"]["exploration"][0].clone();
    duplicate["state"]["exploration"]
        .as_array_mut()
        .unwrap()
        .push(quest);
    assert_eq!(
        State::from_durable_checkpoint(&duplicate),
        Err(Rejection::InvalidCheckpoint)
    );
}

#[test]
fn generated_stories_repeat_without_replacing_unfinished_work_and_replay_every_step() {
    use exploration::{GeneratedContent, GeneratedStep, GeneratedStory};
    let mut state = State::new();
    perform(&mut state, travel("workshop"));
    for generation in 1..=8 {
        let story = GeneratedStory {
            model_generated: true,
            content: GeneratedContent {
                complete_on_last_step: false,
                name: format!("Visitor {generation}"),
                personality: "A curious neighbour.".into(),
                sprite: "flutter".into(),
                greeting: "Shall we explore?".into(),
                title: format!("Discovery {generation}"),
                premise: "Listen and notice the woodland.".into(),
                payoff: "You shared a discovery.".into(),
                steps: vec![
                    GeneratedStep {
                        mechanic: super::exploration::Mechanic::Inspect,
                        location_id: None,
                        discovery: None,
                        learning_bridge: None,
                        label: "Listen to the hollow branch.".into(),
                        shape: "hollow".into(),
                    },
                    GeneratedStep {
                        mechanic: super::exploration::Mechanic::Inspect,
                        location_id: None,
                        discovery: None,
                        learning_bridge: None,
                        label: "Look at the rustling leaf.".into(),
                        shape: "leaf".into(),
                    },
                ],
            },
        };
        perform(
            &mut state,
            Action::GeneratedTravel {
                destination_id: "cove".into(),
                story: story.clone(),
            },
        );
        assert_eq!(state.current_neighbour().unwrap().generation, generation);
        assert_eq!(
            State::from_checkpoint(&state.legacy_checkpoint()),
            Err(Rejection::InvalidCheckpoint)
        );
        assert_eq!(
            State::from_checkpoint(&state.v2_checkpoint()),
            Err(Rejection::InvalidCheckpoint)
        );
        perform(
            &mut state,
            Action::Talk {
                character_id: "marina".into(),
                topic_id: "start_neighbour_story".into(),
            },
        );
        perform(
            &mut state,
            Action::Inspect {
                object_id: "neighbour_marina_0".into(),
            },
        );
        let before = state.checkpoint();
        assert_eq!(
            state.apply(&command(
                &state,
                Action::Inspect {
                    object_id: "neighbour_marina_0".into()
                }
            )),
            Err(Rejection::NotAvailable)
        );
        assert_eq!(state.checkpoint(), before);
        perform(&mut state, travel("workshop"));
        let before = state.checkpoint();
        assert_eq!(
            state.apply(&command(
                &state,
                Action::GeneratedTravel {
                    destination_id: "cove".into(),
                    story
                }
            )),
            Err(Rejection::NotAvailable)
        );
        assert_eq!(state.checkpoint(), before);
        perform(&mut state, travel("cove"));
        assert_eq!(state.current_neighbour().unwrap().step, 1);
        perform(
            &mut state,
            Action::Inspect {
                object_id: "neighbour_marina_1".into(),
            },
        );
        perform(
            &mut state,
            Action::Talk {
                character_id: "marina".into(),
                topic_id: "finish_neighbour_story".into(),
            },
        );
        assert_eq!(State::from_checkpoint(&state.checkpoint()).unwrap(), state);
        assert_eq!(
            State::from_durable_checkpoint(&state.durable_checkpoint()).unwrap(),
            state.compact()
        );
        assert!(state.durable_checkpoint().to_string().len() < 12000);
        perform(&mut state, travel("workshop"));
    }
}

#[test]
fn keepsakes_are_location_bound_once_only_and_replayable_without_a_mission() {
    let mut state = State::new();
    let pickup = Action::Collect {
        source_id: "find_clover".into(),
        item_id: "lucky_clover".into(),
        quantity: 1,
    };
    assert!(state.available_actions().contains(&pickup));
    for action in [
        Action::Collect {
            source_id: "find_shell".into(),
            item_id: "seashell".into(),
            quantity: 1,
        },
        Action::Collect {
            source_id: "find_clover".into(),
            item_id: "wood".into(),
            quantity: 1,
        },
        Action::Collect {
            source_id: "find_clover".into(),
            item_id: "lucky_clover".into(),
            quantity: 2,
        },
    ] {
        let before = state.clone();
        assert!(state.apply(&command(&state, action)).is_err());
        assert_eq!(state, before);
    }
    let cmd = command(&state, pickup.clone());
    let receipt = state.apply(&cmd).unwrap();
    assert_eq!(state.apply(&cmd), Ok(receipt));
    assert_eq!(state.inventory().get("lucky_clover"), Some(&1));
    assert!(!state.available_actions().contains(&pickup));
    assert!(state.apply(&command(&state, pickup)).is_err());
    assert_eq!(state, State::from_checkpoint(&state.checkpoint()).unwrap());
    assert_eq!(
        state.compact(),
        State::from_durable_checkpoint(&state.durable_checkpoint()).unwrap()
    );
    perform(&mut state, travel("workshop"));
    perform(
        &mut state,
        Action::Collect {
            source_id: "find_ribbon".into(),
            item_id: "ribbon".into(),
            quantity: 1,
        },
    );
    assert_eq!(state.quantity("ribbon"), 1);
    assert_eq!(state.quantity("wood"), 0);
    assert_eq!(state.mission_status(), MissionStatus::Offered);
    let mut corrupt = state.durable_checkpoint();
    corrupt["state"]["finds"]["ribbon"] = json!(999);
    assert_eq!(
        State::from_durable_checkpoint(&corrupt),
        Err(Rejection::InvalidCheckpoint)
    );
}

#[test]
fn every_area_has_a_finite_pickup_and_old_saves_omit_empty_finds() {
    let state = State::new();
    assert!(state.durable_checkpoint()["state"].get("finds").is_none());
    for find in items::FINDS {
        let mut state = State::new();
        state.location = find.location;
        let action = Action::Collect {
            source_id: find.source.into(),
            item_id: find.id.into(),
            quantity: 1,
        };
        assert!(state.available_actions().contains(&action));
        perform(&mut state, action);
        assert_eq!(state.quantity(find.id), 1);
        assert_eq!(state.inventory().len(), 1);
    }
}

#[test]
fn legacy_versions_cannot_claim_new_keepsake_commands() {
    let mut state = State::new();
    perform(
        &mut state,
        Action::Collect {
            source_id: "find_clover".into(),
            item_id: "lucky_clover".into(),
            quantity: 1,
        },
    );
    assert_eq!(
        State::from_checkpoint(&state.legacy_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
    assert_eq!(
        State::from_checkpoint(&state.v2_checkpoint()),
        Err(Rejection::InvalidCheckpoint)
    );
}

#[test]
fn generated_journey_requires_the_right_area_and_returns_to_its_original_friend() {
    use exploration::{GeneratedContent, GeneratedStep, GeneratedStory};
    let mut state = State::new();
    perform(&mut state, travel("workshop"));
    let story = GeneratedStory {
        model_generated: true,
        content: GeneratedContent {
            complete_on_last_step: false,
            name: "Wren".into(),
            personality: "A curious listener.".into(),
            sprite: "patter".into(),
            greeting: "Can you hear it?".into(),
            title: "The travelling tune".into(),
            premise: "Follow the tune into Fern Hollow, then return.".into(),
            payoff: "The breeze makes music!".into(),
            steps: vec![GeneratedStep {
                mechanic: super::exploration::Mechanic::Inspect,
                location_id: Some("fern_hollow".into()),
                discovery: Some("The wind hums through a stem.".into()),
                learning_bridge: None,
                label: "Listen to the stem in Fern Hollow".into(),
                shape: "instrument".into(),
            }],
        },
    };
    perform(
        &mut state,
        Action::GeneratedTravel {
            destination_id: "cove".into(),
            story,
        },
    );
    perform(
        &mut state,
        Action::Talk {
            character_id: "marina".into(),
            topic_id: "start_neighbour_story".into(),
        },
    );
    let inspect = Action::Inspect {
        object_id: "neighbour_marina_0".into(),
    };
    assert!(!state.available_actions().contains(&inspect));
    assert!(state.apply(&command(&state, inspect.clone())).is_err());
    perform(&mut state, travel("fern_hollow"));
    assert!(state.available_actions().contains(&inspect));
    assert_eq!(
        state
            .neighbour_for_object("neighbour_marina_0")
            .unwrap()
            .character,
        "marina"
    );
    let restored = State::from_checkpoint(&state.checkpoint()).unwrap();
    assert_eq!(restored.available_actions(), state.available_actions());
    perform(&mut state, inspect.clone());
    assert!(!state.available_actions().contains(&inspect));
    let finish = Action::Talk {
        character_id: "marina".into(),
        topic_id: "finish_neighbour_story".into(),
    };
    assert!(!state.available_actions().contains(&finish));
    perform(&mut state, travel("cove"));
    perform(&mut state, finish);
    assert_eq!(state.current_neighbour().unwrap().status, "completed");
}
