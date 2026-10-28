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
