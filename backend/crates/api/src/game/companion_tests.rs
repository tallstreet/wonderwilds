use super::*;

fn finish(game: &mut Aggregate, content: &Content, quest: &str) {
    let encounter = format!("test-{quest}");
    let mut facts = TrustedFacts::default();
    facts
        .issued_encounters
        .insert(encounter.clone(), quest.into());
    facts.reachable_quests.insert(quest.into());
    game.domain_apply(
        Action::BeginQuest {
            quest_id: quest.into(),
            encounter_id: encounter.clone(),
        },
        &format!("begin-{quest}"),
        content,
        facts,
        1,
    )
    .unwrap();
    let mut facts = TrustedFacts::default();
    facts.completed_encounters.insert(
        encounter.clone(),
        VerifiedCompletion {
            quest_id: quest.into(),
            supported: true,
        },
    );
    game.domain_apply(
        Action::CompleteQuest {
            quest_id: quest.into(),
            encounter_id: encounter,
        },
        &format!("finish-{quest}"),
        content,
        facts,
        1,
    )
    .unwrap();
}

fn request(game: &Aggregate, id: &str, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":game.revision,"payload":payload})
}

#[test]
fn companion_commands_preserve_authority_retries_and_persisted_progress() {
    let content = development_content().unwrap();
    let mut game = Aggregate::new(&content);
    let help = json!({"type":"help_friend","creatureId":"pip","completedQuestId":"meadow-bridge"});
    let transform = json!({"type":"transform_companion","creatureId":"pip","enabled":true});
    for payload in [help.clone(), transform.clone()] {
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.execute(&request(&game, "locked", payload), &content, 2, "session"),
            Err(Error::Forbidden)
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
    }
    for quest in [
        "meadow-welcome",
        "meadow-bridge",
        "meadow-picnic",
        "meadow-garden",
        "meadow-letter",
        "meadow-finale",
        "befriend-pip",
    ] {
        finish(&mut game, &content, quest);
    }
    let xp = game.domain.xp();
    let inventory = game.domain.inventory().clone();
    let snapshot = game.snapshot(&content).unwrap();
    assert_eq!(
        snapshot["friendship"],
        json!([{"creatureId":"pip","points":1}])
    );
    assert!(
        snapshot["availableHelpingMoments"]
            .as_array()
            .unwrap()
            .contains(&json!({"creatureId":"pip","completedQuestId":"meadow-bridge"}))
    );
    assert_eq!(
        game.execute(
            &request(&game, "too-early", transform.clone()),
            &content,
            2,
            "session"
        ),
        Err(Error::Forbidden)
    );

    let first = request(&game, "help-bridge", help.clone());
    let response = game.execute(&first, &content, 2, "session").unwrap();
    let events = game.events.len();
    assert_eq!(
        game.execute(&first, &content, 3, "session").unwrap(),
        response
    );
    assert_eq!(game.events.len(), events);
    assert_eq!(
        game.execute(
            &request(&game, "repeat-moment", help),
            &content,
            3,
            "session"
        ),
        Err(Error::Forbidden)
    );
    assert_eq!(
        game.execute(
            &request(
                &game,
                "invented-moment",
                json!({"type":"help_friend","creatureId":"pip","completedQuestId":"orchard-seeds"})
            ),
            &content,
            3,
            "session"
        ),
        Err(Error::Forbidden)
    );
    game.execute(
        &request(
            &game,
            "help-garden",
            json!({"type":"help_friend","creatureId":"pip","completedQuestId":"meadow-garden"}),
        ),
        &content,
        3,
        "session",
    )
    .unwrap();
    let enable = request(&game, "glow", transform);
    let response = game.execute(&enable, &content, 4, "session").unwrap();
    assert_eq!(response["transformedCreatures"], json!(["pip-glow"]));
    assert_eq!(response["friendship"][0]["points"], 3);
    // Repository serialization retains receipts and domain history across reload.
    game = serde_json::from_value(serde_json::to_value(&game).unwrap()).unwrap();
    assert_eq!(
        game.execute(&enable, &content, 5, "session").unwrap(),
        response
    );
    game.execute(
        &request(
            &game,
            "unglow",
            json!({"type":"transform_companion","creatureId":"pip","enabled":false}),
        ),
        &content,
        5,
        "session",
    )
    .unwrap();
    assert!(game.domain.transformed().is_empty());
    assert_eq!(game.domain.xp(), xp);
    assert_eq!(game.domain.inventory(), &inventory);
    assert!(
        game.evidence.is_empty(),
        "social/cosmetic commands must not fabricate learning evidence"
    );
}

pub(super) fn recruited_pip(content: &Content) -> Aggregate {
    let mut game = Aggregate::new(content);
    for quest in [
        "meadow-welcome",
        "meadow-bridge",
        "meadow-picnic",
        "meadow-garden",
        "meadow-letter",
        "meadow-finale",
        "befriend-pip",
    ] {
        finish(&mut game, content, quest);
    }
    game
}

#[test]
fn training_is_bound_persistent_supported_and_rewards_only_once() {
    let content = development_content().unwrap();
    let mut game = recruited_pip(&content);
    let explorer_xp = game.domain.xp();
    let start = request(
        &game,
        "training-start",
        json!({"type":"start_encounter","beatId":"training-pip-1"}),
    );
    let snapshot = game.execute(&start, &content, 2, "session").unwrap();
    let pending = &snapshot["pendingEncounters"][0];
    assert_eq!(pending["training"]["creatureId"], "pip");
    assert_eq!(pending["training"]["trainingId"], "training-pip-1");
    let encounter = pending["id"].as_str().unwrap().to_owned();
    let original = serde_json::to_value(&game).unwrap();
    game = serde_json::from_value(original).unwrap();
    assert_eq!(
        game.execute(&start, &content, 3, "session").unwrap(),
        snapshot
    );
    game.execute(
        &request(
            &game,
            "training-team-clear",
            json!({"type":"select_team","creatureIds":[]}),
        ),
        &content,
        3,
        "session",
    )
    .unwrap();
    game.execute(
        &request(
            &game,
            "training-hint",
            json!({"type":"hint","encounterId":encounter,"supportKind":"hint"}),
        ),
        &content,
        3,
        "session",
    )
    .unwrap();
    let pending = game.public_encounter(&encounter).unwrap();
    let answer = pending["parameters"]["total"].as_u64().unwrap()
        - pending["parameters"]["given"].as_u64().unwrap();
    let complete = request(
        &game,
        "training-answer",
        json!({"type":"attempt","encounterId":encounter,"answer":{"kind":"quantity","value":answer}}),
    );
    let result = game.execute(&complete, &content, 4, "session").unwrap();
    assert_eq!(result["characterProgression"][0]["level"], 2);
    assert!(result["characterProgression"][0]["xp"].as_u64().unwrap() > 0);
    assert_eq!(game.domain.xp(), explorer_xp);
    assert_eq!(game.evidence.last().unwrap().support.hint_count, 1);
    let xp = game.domain.character_xp("pip");
    let evidence = game.evidence.len();
    assert_eq!(
        game.execute(&complete, &content, 5, "session").unwrap(),
        result
    );
    assert_eq!(game.evidence.len(), evidence);
    game.execute(
        &request(
            &game,
            "training-practice",
            json!({"type":"start_encounter","beatId":"training-pip-1"}),
        ),
        &content,
        5,
        "session",
    )
    .unwrap();
    let pending = game.snapshot(&content).unwrap()["pendingEncounters"][0].clone();
    let answer = pending["parameters"]["total"].as_u64().unwrap()
        - pending["parameters"]["given"].as_u64().unwrap();
    game.execute(&request(&game, "practice-answer", json!({"type":"attempt","encounterId":pending["id"],"answer":{"kind":"quantity","value":answer}})), &content, 6, "session").unwrap();
    assert_eq!(game.domain.character_xp("pip"), xp);
    assert_eq!(game.domain.character_level(&content, "pip"), 2);
}

#[test]
fn training_rejects_unowned_locked_and_client_invented_rewards_without_mutation() {
    let content = development_content().unwrap();
    let mut game = Aggregate::new(&content);
    let before = serde_json::to_value(&game).unwrap();
    assert_eq!(
        game.execute(
            &request(
                &game,
                "unowned-training",
                json!({"type":"start_encounter","beatId":"training-pip-1"})
            ),
            &content,
            1,
            "session"
        ),
        Err(Error::Forbidden)
    );
    assert_eq!(serde_json::to_value(&game).unwrap(), before);
    let mut game = recruited_pip(&content);
    assert_eq!(
        game.execute(
            &request(
                &game,
                "locked-training",
                json!({"type":"start_encounter","beatId":"training-pip-4"})
            ),
            &content,
            1,
            "session"
        ),
        Err(Error::Forbidden)
    );
    assert_eq!(
        game.execute(
            &request(
                &game,
                "invented-xp",
                json!({"type":"start_encounter","beatId":"training-pip-1","xp":99999})
            ),
            &content,
            1,
            "session"
        ),
        Err(Error::InvalidInput)
    );
    let started = game
        .execute(
            &request(
                &game,
                "abandon-start",
                json!({"type":"start_encounter","beatId":"training-pip-1"}),
            ),
            &content,
            1,
            "session",
        )
        .unwrap();
    game.execute(
        &request(
            &game,
            "abandon-training",
            json!({"type":"abandon","encounterId":started["pendingEncounters"][0]["id"]}),
        ),
        &content,
        1,
        "session",
    )
    .unwrap();
    assert_eq!(game.domain.character_xp("pip"), 0);
    assert!(!game.domain.training_completed("training-pip-1"));
}

#[test]
fn changed_training_policy_blocks_awards_but_allows_abandonment() {
    let content = development_content().unwrap();
    let mut game = recruited_pip(&content);
    let snapshot = game
        .execute(
            &request(
                &game,
                "policy-start",
                json!({"type":"start_encounter","beatId":"training-pip-1"}),
            ),
            &content,
            1,
            "session",
        )
        .unwrap();
    let encounter = snapshot["pendingEncounters"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    game.encounters.get_mut(&encounter).unwrap().training_policy = Some("old-policy".into());
    let before = serde_json::to_value(&game).unwrap();
    assert_eq!(
        game.execute(
            &request(
                &game,
                "policy-answer",
                json!({"type":"hint","encounterId":encounter,"supportKind":"hint"})
            ),
            &content,
            2,
            "session"
        ),
        Err(Error::Unavailable)
    );
    assert_eq!(serde_json::to_value(&game).unwrap(), before);
    game.execute(
        &request(
            &game,
            "policy-abandon",
            json!({"type":"abandon","encounterId":encounter}),
        ),
        &content,
        2,
        "session",
    )
    .unwrap();
    assert_eq!(game.domain.character_xp("pip"), 0);
    assert!(
        game.snapshot(&content).unwrap()["pendingEncounters"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

/// A production-shaped legacy aggregate: every domain action, observation and
/// API receipt was produced through the same assessed application boundary.
pub(super) fn recruited_pip_via_api(content: &Content) -> Aggregate {
    let mut game = Aggregate::new(content);
    for quest in [
        "meadow-welcome",
        "meadow-bridge",
        "meadow-picnic",
        "meadow-garden",
        "meadow-letter",
        "meadow-finale",
        "befriend-pip",
    ] {
        let started = game
            .execute(
                &request(
                    &game,
                    &format!("real-start-{quest}"),
                    json!({"type":"start_encounter","beatId":quest}),
                ),
                content,
                1,
                "session",
            )
            .unwrap();
        let encounter = &started["pendingEncounters"][0];
        let p = &encounter["parameters"];
        let answer = match encounter["templateId"].as_str().unwrap() {
            "missing_quantity" => {
                json!({"kind":"quantity","value":p["total"].as_u64().unwrap()-p["given"].as_u64().unwrap()})
            }
            "equal_sharing" => {
                json!({"kind":"groups","groups":vec![p["total"].as_u64().unwrap()/p["recipients"].as_u64().unwrap();p["recipients"].as_u64().unwrap() as usize]})
            }
            _ => {
                let ids: Vec<String> = match p["instructionId"]
                    .as_str()
                    .or_else(|| p["promptId"].as_str())
                    .unwrap()
                {
                    "count-next" => {
                        let objects = p["objects"].as_array().unwrap();
                        let expected =
                            objects[p["prefixLength"].as_u64().unwrap() as usize - 1]["label"]
                                .as_str()
                                .unwrap()
                                .parse::<u64>()
                                .unwrap()
                                + 1;
                        vec![
                            objects
                                .iter()
                                .find(|o| {
                                    o["attribute"] == "choice"
                                        && o["label"].as_str().unwrap().parse::<u64>().unwrap()
                                            == expected
                                })
                                .unwrap()["id"]
                                .as_str()
                                .unwrap()
                                .into(),
                        ]
                    }
                    "sound-s" => vec![
                        p["tiles"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .position(|t| t == "s")
                            .unwrap()
                            .to_string(),
                    ],
                    "seed-order" => {
                        vec!["find_seed".into(), "plant_seed".into(), "water_seed".into()]
                    }
                    "listen-direction" => vec![
                        p["cards"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|c| c["text"] == "Put the cup beside the tree.")
                            .unwrap()["id"]
                            .as_str()
                            .unwrap()
                            .into(),
                    ],
                    other => panic!("Unexpected Meadow task {other}"),
                };
                json!({"kind":"choices","choiceIds":ids})
            }
        };
        game.execute(
            &request(
                &game,
                &format!("real-finish-{quest}"),
                json!({"type":"attempt","encounterId":encounter["id"],"answer":answer}),
            ),
            content,
            2,
            "session",
        )
        .unwrap();
    }
    game
}
