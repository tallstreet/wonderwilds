use super::content::Reward;
use super::*;
use std::collections::BTreeSet;
const STORY: &str = include_str!("../../../../../content/story/adventure.v1.json");
const HELP: &str = include_str!("../../../../../content/story/help-quests.v1.json");
const FRIENDS: &str = include_str!("../../../../../content/creatures/friends.v1.json");
const PROGRESSION: &str = include_str!("../../../../../content/story/progression.v1.json");
fn content() -> Content {
    Content::development(STORY, HELP, FRIENDS, PROGRESSION).unwrap()
}
fn context() -> Context {
    Context {
        unix_millis: 1793145600000,
        recorded_random_choice: 73,
        facts: TrustedFacts::default(),
    }
}
fn command(state: &State, action: Action) -> Command {
    Command {
        command_id: format!("command-{}", state.revision),
        expected_revision: state.revision,
        action,
    }
}
fn execute(
    state: &mut State,
    action: Action,
    content: &Content,
    ctx: Context,
    log: &mut Vec<ReplayEntry>,
) {
    let cmd = command(state, action);
    let applied = apply(state, &cmd, content, &ctx).unwrap();
    assert!(!applied.duplicate);
    assert_eq!(applied.state.revision, state.revision + 1);
    assert!(
        applied
            .events
            .windows(2)
            .all(|w| w[1].sequence == w[0].sequence + 1)
    );
    log.push(ReplayEntry {
        command: cmd,
        context: ctx,
        events: applied.events,
    });
    *state = applied.state;
}
fn finish(
    state: &mut State,
    id: &str,
    content: &Content,
    supported: bool,
    log: &mut Vec<ReplayEntry>,
) {
    let encounter = format!("encounter-{id}");
    let mut ctx = context();
    ctx.facts
        .issued_encounters
        .insert(encounter.clone(), id.into());
    ctx.facts.reachable_quests.insert(id.into());
    execute(
        state,
        Action::BeginQuest {
            quest_id: id.into(),
            encounter_id: encounter.clone(),
        },
        content,
        ctx,
        log,
    );
    let mut ctx = context();
    ctx.facts.completed_encounters.insert(
        encounter.clone(),
        VerifiedCompletion {
            quest_id: id.into(),
            supported,
        },
    );
    execute(
        state,
        Action::CompleteQuest {
            quest_id: id.into(),
            encounter_id: encounter,
        },
        content,
        ctx,
        log,
    );
}
fn chapter(
    state: &mut State,
    region: &str,
    c: &Content,
    supported: bool,
    log: &mut Vec<ReplayEntry>,
) {
    if state.region() != region {
        execute(
            state,
            Action::Travel {
                region_id: region.into(),
            },
            c,
            context(),
            log,
        );
    }
    let ids = match region {
        "meadow" => vec![
            "meadow-welcome",
            "meadow-bridge",
            "meadow-picnic",
            "meadow-garden",
            "meadow-letter",
            "meadow-finale",
        ],
        "orchard" => vec![
            "orchard-welcome",
            "orchard-seeds",
            "orchard-ropes",
            "orchard-delivery",
            "orchard-instructions",
            "orchard-finale",
        ],
        _ => vec![
            "cove-welcome",
            "cove-observatory",
            "cove-shells",
            "cove-message",
            "cove-storybook",
            "cove-finale",
        ],
    };
    for id in ids {
        finish(state, id, c, supported, log);
    }
}
#[test]
fn malformed_training_policies_are_rejected_before_play() {
    let cases = [
        ("/characterProgression/levels/1", serde_json::json!(0)),
        ("/characterProgression/training/0/xp", serde_json::json!(1)),
        (
            "/characterProgression/training/0/creatureId",
            serde_json::json!("unknown"),
        ),
        (
            "/characterProgression/training/0/skillId",
            serde_json::json!("M03"),
        ),
        (
            "/characterProgression/training/0/reactionId",
            serde_json::json!(""),
        ),
        (
            "/characterProgression/training/0/requiredLevel",
            serde_json::json!(4),
        ),
        (
            "/characterProgression/supportedCompletionMultiplier",
            serde_json::json!(0),
        ),
        (
            "/characterProgression/repeatMilestoneXp",
            serde_json::json!(10),
        ),
        (
            "/characterProgression/rulesVersion",
            serde_json::json!("unknown"),
        ),
    ];
    for (pointer, replacement) in cases {
        let mut policy: serde_json::Value = serde_json::from_str(PROGRESSION).unwrap();
        *policy.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            matches!(
                Content::development(STORY, HELP, FRIENDS, &policy.to_string()),
                Err(Rejection::InvalidContent(_))
            ),
            "{pointer}"
        );
    }
}

#[test]
fn training_is_bound_supported_idempotent_and_replayable() {
    let c = content();
    let mut initial = State::new(&c);
    initial.creatures.insert("pip".into());
    initial.creatures.insert("tansy".into());
    let mut state = initial.clone();
    let mut log = Vec::new();
    finish(&mut state, "training-pip-1", &c, true, &mut log);
    assert_eq!(state.character_xp("pip"), 10);
    assert_eq!(state.character_level(&c, "pip"), 2);
    assert_eq!(state.character_xp("tansy"), 0);
    assert_eq!(state.xp(), 0);
    assert_eq!(state.friendship("pip"), 0);
    assert!(state.completed().is_empty());
    let receipt = log.last().unwrap();
    let duplicate = apply(&state, &receipt.command, &c, &receipt.context).unwrap();
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.state, state);
    finish(&mut state, "training-pip-1", &c, false, &mut log);
    assert_eq!(state.character_xp("pip"), 10);
    assert!(
        log.last()
            .unwrap()
            .events
            .iter()
            .all(|e| !matches!(e.outcome, Outcome::RewardGranted { .. }))
    );
    assert_eq!(replay(&initial, &c, &log).unwrap(), state);
    let reloaded: State = serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
    assert_eq!(reloaded, state);
    for n in 2..=4 {
        finish(&mut state, &format!("training-pip-{n}"), &c, true, &mut log);
    }
    assert_eq!(state.character_level(&c, "pip"), 5);
    assert_eq!(state.character_xp("pip"), 40);
}

#[test]
fn training_rejects_locked_unowned_and_cross_bound_completions() {
    let c = content();
    let mut state = State::new(&c);
    state.creatures.insert("pip".into());
    let mut ctx = context();
    for id in ["training-tansy-1", "training-pip-2"] {
        ctx.facts
            .issued_encounters
            .insert("encounter".into(), id.into());
        ctx.facts.reachable_quests.insert(id.into());
        let cmd = command(
            &state,
            Action::BeginQuest {
                quest_id: id.into(),
                encounter_id: "encounter".into(),
            },
        );
        assert_eq!(apply(&state, &cmd, &c, &ctx), Err(Rejection::Locked));
    }
    ctx.facts
        .issued_encounters
        .insert("encounter".into(), "training-pip-1".into());
    ctx.facts.reachable_quests.insert("training-pip-1".into());
    let cmd = command(
        &state,
        Action::BeginQuest {
            quest_id: "training-pip-1".into(),
            encounter_id: "encounter".into(),
        },
    );
    state = apply(&state, &cmd, &c, &ctx).unwrap().state;
    ctx.facts.completed_encounters.insert(
        "encounter".into(),
        VerifiedCompletion {
            quest_id: "training-tansy-1".into(),
            supported: false,
        },
    );
    let cmd = command(
        &state,
        Action::CompleteQuest {
            quest_id: "training-pip-1".into(),
            encounter_id: "encounter".into(),
        },
    );
    assert_eq!(
        apply(&state, &cmd, &c, &ctx),
        Err(Rejection::UnverifiedEncounter)
    );
    let cmd = command(
        &state,
        Action::AbandonQuest {
            quest_id: "training-pip-1".into(),
            encounter_id: "encounter".into(),
        },
    );
    state = apply(&state, &cmd, &c, &ctx).unwrap().state;
    assert_eq!(state.character_xp("pip"), 0);
}

#[test]
fn old_state_loads_without_speculative_character_xp() {
    let c = content();
    let mut old = State::new(&c);
    old.creatures.insert("pip".into());
    old.friendship.insert("pip".into(), 3);
    old.transformed.insert("pip".into());
    let mut value = serde_json::to_value(&old).unwrap();
    value.as_object_mut().unwrap().remove("character_xp");
    value.as_object_mut().unwrap().remove("training_grants");
    let loaded: State = serde_json::from_value(value).unwrap();
    assert_eq!(loaded.character_xp("pip"), 0);
    assert_eq!(loaded.character_level(&c, "pip"), 1);
    assert_eq!(loaded.friendship("pip"), 3);
    assert_eq!(loaded.transformed(), old.transformed());
}

#[test]
fn abandonment_is_replayable_without_rewards_and_can_restart() {
    let c = content();
    let initial = State::new(&c);
    let mut state = initial.clone();
    let mut log = Vec::new();
    let mut ctx = context();
    ctx.facts
        .issued_encounters
        .insert("enc-1".into(), "meadow-welcome".into());
    ctx.facts.reachable_quests.insert("meadow-welcome".into());
    execute(
        &mut state,
        Action::BeginQuest {
            quest_id: "meadow-welcome".into(),
            encounter_id: "enc-1".into(),
        },
        &c,
        ctx.clone(),
        &mut log,
    );
    let abandon = command(
        &state,
        Action::AbandonQuest {
            quest_id: "meadow-welcome".into(),
            encounter_id: "enc-1".into(),
        },
    );
    let applied = apply(&state, &abandon, &c, &context()).unwrap();
    assert!(matches!(
        &applied.events[0].outcome,
        Outcome::QuestAbandoned { .. }
    ));
    assert!(applied.state.grants.is_empty());
    assert!(applied.state.completed.is_empty());
    assert_eq!(applied.state.xp, 0);
    assert!(applied.state.pending.is_empty());
    assert!(
        apply(&applied.state, &abandon, &c, &context())
            .unwrap()
            .duplicate
    );
    execute(&mut state, abandon.action, &c, context(), &mut log);
    ctx.facts
        .issued_encounters
        .insert("enc-2".into(), "meadow-welcome".into());
    execute(
        &mut state,
        Action::BeginQuest {
            quest_id: "meadow-welcome".into(),
            encounter_id: "enc-2".into(),
        },
        &c,
        ctx,
        &mut log,
    );
    assert_eq!(replay(&initial, &c, &log).unwrap(), state);
}
#[test]
fn golden_welcome_events_keep_rules_content_and_replay_compatible() {
    let c = content();
    let initial = State::new(&c);
    let mut state = initial.clone();
    let mut log = Vec::new();
    finish(&mut state, "meadow-welcome", &c, true, &mut log);
    let events: Vec<_> = log.iter().flat_map(|entry| entry.events.clone()).collect();
    let golden: Vec<Event> = serde_json::from_str(include_str!(
        "../../tests/fixtures/meadow-welcome-events.v1.json"
    ))
    .unwrap();
    assert_eq!(events, golden);
    assert_eq!(replay(&initial, &c, &log).unwrap(), state);
}
#[test]
fn small_domain_properties_hold_for_many_recorded_discovery_choices() {
    let c = content();
    let mut base = State::new(&c);
    chapter(&mut base, "meadow", &c, true, &mut Vec::new());
    for seed in 0..64 {
        let mut state = base.clone();
        let mut log = Vec::new();
        let mut choices = BTreeSet::new();
        for n in 0..4 {
            let mut ctx = context();
            ctx.recorded_random_choice = seed * 17 + n;
            let old_revision = state.revision();
            execute(&mut state, Action::Discover, &c, ctx, &mut log);
            assert_eq!(state.revision(), old_revision + 1);
            let creature = state.offered_creature().unwrap().to_string();
            assert!(choices.insert(creature.clone()));
            finish(
                &mut state,
                &format!("befriend-{creature}"),
                &c,
                n % 2 == 0,
                &mut log,
            );
            assert!(
                state
                    .friendship
                    .values()
                    .all(|&points| points <= c.max_friendship)
            );
            assert_eq!(state.grants.len(), 9 + (n as usize + 1) * 2);
        }
        assert_eq!(state.creatures.len(), 4);
        assert_eq!(state.xp(), 65);
        assert_eq!(replay(&base, &c, &log).unwrap(), state);
    }
}
#[test]
fn authored_packs_load_but_unreviewed_content_cannot_activate() {
    assert_eq!(content().version(), "wonderwilds-story-1.0.0");
    assert_eq!(content().quest_skill_id("meadow-welcome"), Some("E01"));
    assert_eq!(
        content().quest_template_id("meadow-welcome"),
        Some("story_message")
    );
    assert_eq!(content().quest_skill_id("befriend-pip"), Some("M06"));
    assert_eq!(content().quest_skill_id("invented"), None);
    assert!(matches!(
        Content::released(STORY, HELP, FRIENDS, PROGRESSION),
        Err(Rejection::InvalidContent(_))
    ));
    let invalid = STORY.replace("\"schemaVersion\": \"1.0\"", "\"schemaVersion\": \"2.0\"");
    assert!(Content::development(&invalid, HELP, FRIENDS, PROGRESSION).is_err());
    let mut story: serde_json::Value = serde_json::from_str(STORY).unwrap();
    story["beats"][0]["requiresBeatIds"] = serde_json::json!(["cove-finale"]);
    assert!(Content::development(&story.to_string(), HELP, FRIENDS, PROGRESSION).is_err());
}
#[test]
fn supported_journey_completes_every_finale_and_replays_exactly() {
    let c = content();
    let initial = State::new(&c);
    let mut state = initial.clone();
    let mut log = Vec::new();
    for region in ["meadow", "orchard", "cove"] {
        chapter(&mut state, region, &c, true, &mut log);
        assert!(state.flags().contains(&format!("{region}_light")));
    }
    assert_eq!(state.completed().len(), 18);
    assert_eq!(state.xp(), 135);
    assert_eq!(state.level(&c), 4);
    assert!(state.features.contains("ending"));
    assert_eq!(replay(&initial, &c, &log).unwrap(), state);
    assert!(
        log.iter()
            .flat_map(|e| &e.events)
            .filter_map(
                |e| if let Outcome::QuestCompleted { supported, .. } = &e.outcome {
                    Some(supported)
                } else {
                    None
                }
            )
            .all(|s| *s)
    );
    let mut bad = log.clone();
    bad[1].events[0].unix_millis += 1;
    assert_eq!(replay(&initial, &c, &bad), Err(Rejection::ReplayMismatch));
}
#[test]
fn idempotent_retries_return_original_events_and_reused_id_rejects_different_body() {
    let c = content();
    let mut s = State::new(&c);
    let mut log = Vec::new();
    finish(&mut s, "meadow-welcome", &c, false, &mut log);
    let receipt = log.last().unwrap();
    let xp = s.xp();
    let revision = s.revision();
    let retry = apply(&s, &receipt.command, &c, &context()).unwrap();
    assert!(retry.duplicate);
    assert_eq!(retry.events, receipt.events);
    assert_eq!(retry.state.xp(), xp);
    assert_eq!(retry.state.revision(), revision);
    let mut reused = receipt.command.clone();
    reused.action = Action::Discover;
    assert_eq!(
        apply(&s, &reused, &c, &context()),
        Err(Rejection::CommandIdReused)
    );
    let cmd = command(
        &s,
        Action::CompleteQuest {
            quest_id: "meadow-welcome".into(),
            encounter_id: "encounter-meadow-welcome".into(),
        },
    );
    assert_eq!(
        apply(&s, &cmd, &c, &context()),
        Err(Rejection::AlreadyCompleted)
    );
}
#[test]
fn client_claims_and_cross_quest_assessments_cannot_grant_rewards() {
    let c = content();
    let mut s = State::new(&c);
    let mut log = Vec::new();
    let id = "meadow-welcome";
    let begin = command(
        &s,
        Action::BeginQuest {
            quest_id: id.into(),
            encounter_id: "fake".into(),
        },
    );
    assert_eq!(
        apply(&s, &begin, &c, &context()),
        Err(Rejection::UnverifiedEncounter)
    );
    let mut ctx = context();
    ctx.facts.issued_encounters.insert("real".into(), id.into());
    ctx.facts.reachable_quests.insert(id.into());
    execute(
        &mut s,
        Action::BeginQuest {
            quest_id: id.into(),
            encounter_id: "real".into(),
        },
        &c,
        ctx,
        &mut log,
    );
    let complete = command(
        &s,
        Action::CompleteQuest {
            quest_id: id.into(),
            encounter_id: "real".into(),
        },
    );
    assert_eq!(
        apply(&s, &complete, &c, &context()),
        Err(Rejection::UnverifiedEncounter)
    );
    let mut ctx = context();
    ctx.facts.completed_encounters.insert(
        "real".into(),
        VerifiedCompletion {
            quest_id: "meadow-bridge".into(),
            supported: false,
        },
    );
    assert_eq!(
        apply(&s, &complete, &c, &ctx),
        Err(Rejection::UnverifiedEncounter)
    );
    assert_eq!(s.xp(), 0);
    assert!(s.completed.is_empty());
    let injected = serde_json::json!({"commandId":"forged","expectedRevision":0,"action":{"type":"complete_quest","quest_id":id,"encounter_id":"real","correct":true,"supported":false}});
    assert!(serde_json::from_value::<Command>(injected).is_err());
}
#[test]
fn stale_revision_unknown_ids_and_unreachable_interactions_are_rejected_atomically() {
    let c = content();
    let state = State::new(&c);
    for (action, expected) in [
        (
            Action::Travel {
                region_id: "orchard".into(),
            },
            Rejection::Locked,
        ),
        (
            Action::Travel {
                region_id: "invented".into(),
            },
            Rejection::NotFound,
        ),
        (
            Action::BeginQuest {
                quest_id: "meadow-finale".into(),
                encounter_id: "enc".into(),
            },
            Rejection::Locked,
        ),
        (
            Action::SelectTeam {
                creature_ids: vec!["pip".into()],
            },
            Rejection::Locked,
        ),
    ] {
        assert_eq!(
            apply(&state, &command(&state, action), &c, &context()),
            Err(expected)
        );
    }
    let mut stale = command(&state, Action::Discover);
    stale.expected_revision = 1;
    assert_eq!(
        apply(&state, &stale, &c, &context()),
        Err(Rejection::RevisionConflict)
    );
    assert_eq!(state, State::new(&c));
}
#[test]
fn independent_and_supported_completions_have_identical_story_rewards() {
    let c = content();
    let mut a = State::new(&c);
    let mut b = a.clone();
    chapter(&mut a, "meadow", &c, false, &mut Vec::new());
    chapter(&mut b, "meadow", &c, true, &mut Vec::new());
    assert_eq!(a.xp(), b.xp());
    assert_eq!(a.flags(), b.flags());
    assert_eq!(a.completed(), b.completed());
    assert_eq!(a.grants, b.grants);
}
#[test]
fn every_friend_can_reach_and_reverse_transformation_on_supported_story() {
    let c = content();
    let initial = State::new(&c);
    let mut s = initial.clone();
    let mut log = Vec::new();
    let mut transformed = 0;
    for region in ["meadow", "orchard", "cove"] {
        chapter(&mut s, region, &c, true, &mut log);
        for friend in c.creatures.values().filter(|f| f.region_id == region) {
            finish(&mut s, &friend.help_quest_id, &c, true, &mut log);
            let xp = s.xp();
            for beat in c.beats.values().filter(|b| b.chapter_id == region) {
                if s.friendship(&friend.id) >= friend.transformation.minimum_friendship {
                    break;
                }
                execute(
                    &mut s,
                    Action::HelpFriend {
                        creature_id: friend.id.clone(),
                        completed_quest_id: beat.id.clone(),
                    },
                    &c,
                    context(),
                    &mut log,
                );
            }
            assert!(s.friendship(&friend.id) >= friend.transformation.minimum_friendship);
            for enabled in [true, false] {
                execute(
                    &mut s,
                    Action::Transform {
                        creature_id: friend.id.clone(),
                        enabled,
                    },
                    &c,
                    context(),
                    &mut log,
                );
                assert_eq!(s.transformed().contains(&friend.transformation.id), enabled);
            }
            assert_eq!(s.xp(), xp, "friendship/cosmetics must not farm explorer XP");
            transformed += 1;
        }
    }
    assert_eq!(transformed, 12);
    assert_eq!(s.creatures().len(), 12);
    assert_eq!(replay(&initial, &c, &log).unwrap(), s);
}
#[test]
fn collection_friendship_cosmetics_and_team_have_independent_gates() {
    let c = content();
    let mut s = State::new(&c);
    let mut log = Vec::new();
    chapter(&mut s, "meadow", &c, true, &mut log);
    finish(&mut s, "befriend-pip", &c, true, &mut log);
    assert!(s.creatures().contains("pip"));
    assert_eq!(s.friendship("pip"), 1);
    let transform = Action::Transform {
        creature_id: "pip".into(),
        enabled: true,
    };
    assert_eq!(
        apply(&s, &command(&s, transform.clone()), &c, &context()),
        Err(Rejection::Locked)
    );
    for quest in ["meadow-bridge", "meadow-garden"] {
        execute(
            &mut s,
            Action::HelpFriend {
                creature_id: "pip".into(),
                completed_quest_id: quest.into(),
            },
            &c,
            context(),
            &mut log,
        );
    }
    assert_eq!(s.friendship("pip"), 3);
    execute(&mut s, transform, &c, context(), &mut log);
    assert!(s.transformed().contains("pip-glow"));
    let repeated = command(
        &s,
        Action::HelpFriend {
            creature_id: "pip".into(),
            completed_quest_id: "meadow-bridge".into(),
        },
    );
    assert_eq!(apply(&s, &repeated, &c, &context()), Err(Rejection::Locked));
    execute(
        &mut s,
        Action::SelectTeam {
            creature_ids: vec!["pip".into()],
        },
        &c,
        context(),
        &mut log,
    );
    assert_eq!(s.team(), ["pip"]);
    let xp = s.xp();
    execute(
        &mut s,
        Action::Transform {
            creature_id: "pip".into(),
            enabled: false,
        },
        &c,
        context(),
        &mut log,
    );
    assert!(s.transformed().is_empty());
    assert_eq!(s.xp(), xp);
}
#[test]
fn discovery_is_stable_when_declined_and_missing_friends_are_bounded() {
    let c = content();
    let mut s = State::new(&c);
    let mut log = Vec::new();
    chapter(&mut s, "meadow", &c, true, &mut log);
    let mut seen = BTreeSet::new();
    for index in 0..4 {
        execute(&mut s, Action::Discover, &c, context(), &mut log);
        let id = s.offered_creature().unwrap().to_string();
        if index == 0 {
            assert_eq!(id, "pip");
        }
        let mut changed = context();
        changed.recorded_random_choice = 999;
        execute(&mut s, Action::Discover, &c, changed, &mut log);
        assert_eq!(s.offered_creature(), Some(id.as_str()));
        assert!(seen.insert(id.clone()));
        finish(&mut s, &format!("befriend-{id}"), &c, true, &mut log);
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(s.creatures().len(), 4);
    execute(&mut s, Action::Discover, &c, context(), &mut log);
}
#[test]
fn inventory_subtraction_is_checked_and_failed_reward_batches_rollback() {
    let mut c = content();
    c.beats
        .get_mut("meadow-welcome")
        .unwrap()
        .rewards
        .push(Reward::InventoryItem {
            id: "planks".into(),
            value: "wood".into(),
            amount: 3,
        });
    let mut s = State::new(&c);
    finish(&mut s, "meadow-welcome", &c, false, &mut Vec::new());
    for amount in [0, 4, u32::MAX] {
        let before = s.clone();
        assert!(
            apply(
                &s,
                &command(
                    &s,
                    Action::UseItem {
                        item_id: "wood".into(),
                        amount
                    }
                ),
                &c,
                &context()
            )
            .is_err()
        );
        assert_eq!(s, before);
    }
    execute(
        &mut s,
        Action::UseItem {
            item_id: "wood".into(),
            amount: 3,
        },
        &c,
        context(),
        &mut Vec::new(),
    );
    assert_eq!(s.inventory()["wood"], 0);
    let mut overflow = State::new(&c);
    overflow.xp = u32::MAX;
    let mut ctx = context();
    ctx.facts
        .issued_encounters
        .insert("overflow".into(), "meadow-welcome".into());
    ctx.facts.reachable_quests.insert("meadow-welcome".into());
    execute(
        &mut overflow,
        Action::BeginQuest {
            quest_id: "meadow-welcome".into(),
            encounter_id: "overflow".into(),
        },
        &c,
        ctx,
        &mut Vec::new(),
    );
    let before = overflow.clone();
    let mut ctx = context();
    ctx.facts.completed_encounters.insert(
        "overflow".into(),
        VerifiedCompletion {
            quest_id: "meadow-welcome".into(),
            supported: false,
        },
    );
    assert_eq!(
        apply(
            &overflow,
            &command(
                &overflow,
                Action::CompleteQuest {
                    quest_id: "meadow-welcome".into(),
                    encounter_id: "overflow".into()
                }
            ),
            &c,
            &ctx
        ),
        Err(Rejection::Overflow)
    );
    assert_eq!(overflow, before);
}
