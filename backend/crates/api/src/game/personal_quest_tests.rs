use super::*;

fn request(game: &Aggregate, id: &str, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":game.revision,"payload":payload})
}

fn send(game: &mut Aggregate, content: &Content, payload: Value) -> Value {
    let id = format!("personal-command-{}", game.revision);
    game.execute(&request(game, &id, payload), content, 100, "quest-session")
        .unwrap()
}

fn complete_quantity(game: &mut Aggregate, content: &Content, supported: bool) -> Value {
    let encounter = game.snapshot(content).unwrap()["pendingEncounters"][0].clone();
    let id = encounter["id"].clone();
    if supported {
        send(
            game,
            content,
            json!({"type":"hint","encounterId":id,"supportKind":"hint"}),
        );
    }
    let quantity = encounter["parameters"]["total"].as_u64().unwrap()
        - encounter["parameters"]["given"].as_u64().unwrap();
    send(
        game,
        content,
        json!({"type":"attempt","encounterId":id,"answer":{"kind":"quantity","value":quantity}}),
    )
}

fn ready_pip(content: &Content) -> Aggregate {
    let mut game = companion_tests::recruited_pip_via_api(content);
    for n in 1..=3 {
        send(
            &mut game,
            content,
            json!({"type":"start_encounter","beatId":format!("training-pip-{n}")}),
        );
        complete_quantity(&mut game, content, false);
    }
    assert_eq!(game.domain.character_level(content, "pip"), 4);
    game
}

fn start_path(game: &mut Aggregate, content: &Content, choice: &str) -> Value {
    send(
        game,
        content,
        json!({"type":"start_personal_quest","questId":"pip-lantern-trail"}),
    );
    send(
        game,
        content,
        json!({"type":"choose_personal_path","choiceId":choice}),
    )
}

#[test]
fn personal_quests_enforce_unlocks_and_server_activity_authority() {
    let content = development_content().unwrap();
    for mut game in [
        Aggregate::new(&content),
        companion_tests::recruited_pip_via_api(&content),
    ] {
        let old = serde_json::to_value(&game).unwrap();
        let command = request(
            &game,
            "locked-personal",
            json!({"type":"start_personal_quest","questId":"pip-lantern-trail"}),
        );
        assert_eq!(
            game.execute(&command, &content, 100, "session"),
            Err(Error::Forbidden)
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), old);
    }
    let mut game = ready_pip(&content);
    let narrative = send(
        &mut game,
        &content,
        json!({"type":"start_personal_quest","questId":"pip-lantern-trail"}),
    );
    assert_eq!(narrative["personalQuest"]["node"]["type"], "choice");
    let before = serde_json::to_value(&game).unwrap();
    for payload in [
        json!({"type":"start_personal_activity"}),
        json!({"type":"choose_personal_path","choiceId":"invented-branch"}),
        json!({"type":"choose_personal_path","choiceId":"pond","xp":100}),
    ] {
        assert!(
            game.execute(
                &request(&game, "invalid-personal", payload),
                &content,
                101,
                "session"
            )
            .is_err()
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
    }
    let evidence_count = game.evidence.len();
    let choice = send(
        &mut game,
        &content,
        json!({"type":"choose_personal_path","choiceId":"pond"}),
    );
    assert_eq!(choice["personalQuest"]["node"]["id"], "path-one");
    assert_eq!(
        game.evidence.len(),
        evidence_count,
        "Narrative choices are not assessed evidence"
    );
    let started = send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    let encounter = &started["pendingEncounters"][0];
    assert_eq!(encounter["skillId"], "M06");
    assert_eq!(encounter["personalQuest"]["nodeId"], "path-one");
    assert_eq!(
        encounter["personalQuest"]["episodeId"],
        choice["personalQuest"]["episodeId"]
    );
    let id = encounter["id"].clone();
    let quantity = encounter["parameters"]["total"].as_u64().unwrap()
        - encounter["parameters"]["given"].as_u64().unwrap();
    let wrong = send(
        &mut game,
        &content,
        json!({"type":"attempt","encounterId":id,"answer":{"kind":"quantity","value":quantity+1}}),
    );
    assert_eq!(wrong["personalQuest"]["node"]["id"], "path-one");
    assert_eq!(wrong["pendingEncounters"].as_array().unwrap().len(), 1);
    assert_eq!(game.domain.inventory().get("pip-personal-memory"), None);
    let completed = complete_quantity(&mut game, &content, true);
    assert_eq!(completed["personalQuest"]["node"]["id"], "reflection");
    assert!(
        completed["pendingEncounters"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        game.evidence
            .iter()
            .rev()
            .take(2)
            .all(|e| e.support.hint_count == 1)
    );
}

#[test]
fn personal_handoffs_continue_into_new_episodes_without_repeat_rewards() {
    let content = development_content().unwrap();
    let mut game = ready_pip(&content);
    let original_xp = game.domain.character_xp("pip");
    let first = start_path(&mut game, &content, "pond");
    let first_episode = first["personalQuest"]["episodeId"].clone();
    send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    complete_quantity(&mut game, &content, true);
    let handoff_request = request(
        &game,
        "remember-stars",
        json!({"type":"choose_personal_path","choiceId":"stars"}),
    );
    let handoff = game
        .execute(&handoff_request, &content, 100, "quest-session")
        .unwrap();
    assert_eq!(handoff["personalQuest"]["node"]["type"], "handoff");
    assert_eq!(
        handoff["personalQuest"]["node"]["continuationTags"],
        json!(["night-sky"])
    );
    assert_eq!(game.domain.inventory().get("pip-personal-memory"), Some(&1));
    let original = serde_json::to_value(&game).unwrap();
    assert_eq!(
        game.execute(&handoff_request, &content, 101, "another-session")
            .unwrap(),
        handoff
    );
    assert_eq!(serde_json::to_value(&game).unwrap(), original);
    assert!(
        handoff["personalQuestOffers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|offer| offer["questId"] == "pip-lantern-trail"
                && offer["available"] == true
                && offer["completed"] == true)
    );
    let second = start_path(&mut game, &content, "hill");
    assert_ne!(second["personalQuest"]["episodeId"], first_episode);
    assert_eq!(second["personalQuest"]["node"]["id"], "path-two");
    send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    complete_quantity(&mut game, &content, false);
    let handoff = send(
        &mut game,
        &content,
        json!({"type":"choose_personal_path","choiceId":"leaves"}),
    );
    assert_eq!(
        handoff["personalQuest"]["node"]["continuationTags"],
        json!(["woodland-trails"])
    );
    assert_eq!(game.domain.inventory().get("pip-personal-memory"), Some(&1));
    assert_eq!(game.domain.character_xp("pip"), original_xp);
}

#[test]
fn personal_pending_encounters_reload_abandon_and_keep_their_bound_recipient() {
    let content = development_content().unwrap();
    let mut game = ready_pip(&content);
    start_path(&mut game, &content, "pond");
    let started = send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    let id = started["pendingEncounters"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    send(
        &mut game,
        &content,
        json!({"type":"hint","encounterId":id,"supportKind":"hint"}),
    );
    send(
        &mut game,
        &content,
        json!({"type":"select_team","creatureIds":[]}),
    );
    game = serde_json::from_value(serde_json::to_value(&game).unwrap()).unwrap();
    let public = game.public_encounter(&id).unwrap();
    assert_eq!(public["support"]["hintCount"], 1);
    assert_eq!(public["personalQuest"]["questId"], "pip-lantern-trail");
    let mut wrong_binding = game.clone();
    wrong_binding
        .encounters
        .get_mut(&id)
        .unwrap()
        .personal_quest
        .as_mut()
        .unwrap()
        .episode_id = "other-episode".into();
    assert_eq!(wrong_binding.public_encounter(&id), Err(Error::Unavailable));
    assert!(wrong_binding.clone().into_storage(&content).is_err());
    for kind in ["abandon_personal_quest", "start_personal_quest"] {
        let payload = if kind == "start_personal_quest" {
            json!({"type":kind,"questId":"pip-lantern-trail"})
        } else {
            json!({"type":kind})
        };
        assert_eq!(
            game.execute(
                &request(&game, "pending-conflict", payload),
                &content,
                100,
                "session"
            ),
            Err(Error::RevisionConflict)
        );
    }
    let abandoned = send(
        &mut game,
        &content,
        json!({"type":"abandon","encounterId":id}),
    );
    assert_eq!(abandoned["personalQuest"]["node"]["id"], "path-one");
    assert!(
        abandoned["pendingEncounters"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(game.domain.inventory().get("pip-personal-memory"), None);
    send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    let advanced = complete_quantity(&mut game, &content, false);
    assert_eq!(advanced["personalQuest"]["node"]["id"], "reflection");
    let abandoned_episode = send(
        &mut game,
        &content,
        json!({"type":"abandon_personal_quest"}),
    );
    assert!(abandoned_episode.get("personalQuest").is_none());
    assert_eq!(game.domain.inventory().get("pip-personal-memory"), None);
}

#[test]
fn personal_pending_state_survives_compaction_with_identical_server_assessment() {
    let content = development_content().unwrap();
    let mut game = ready_pip(&content);
    start_path(&mut game, &content, "hill");
    let started = send(
        &mut game,
        &content,
        json!({"type":"start_personal_activity"}),
    );
    let id = started["pendingEncounters"][0]["id"].clone();
    send(
        &mut game,
        &content,
        json!({"type":"hint","encounterId":id,"supportKind":"hint"}),
    );
    let snapshot = game.snapshot(&content).unwrap();
    let migration = game.clone().into_storage(&content).unwrap();
    let mut compact = migration.aggregate;
    let serialized = serde_json::to_value(&compact).unwrap();
    assert_eq!(serialized["storageFormat"], 2);
    // The previous runtime explicitly accepts only envelope 1, so it must reject
    // this profile instead of silently dropping personal episode progress.
    let mut stripped_version = serialized.clone();
    stripped_version["storageFormat"] = json!(1);
    assert!(serde_json::from_value::<Aggregate>(stripped_version).is_err());
    compact = serde_json::from_value(serialized).unwrap();
    assert_eq!(compact.snapshot(&content).unwrap(), snapshot);
    let encounter = &snapshot["pendingEncounters"][0];
    let quantity = encounter["parameters"]["total"].as_u64().unwrap()
        - encounter["parameters"]["given"].as_u64().unwrap();
    let completion = request(
        &game,
        "personal-compacted-completion",
        json!({"type":"attempt","encounterId":id,"answer":{"kind":"quantity","value":quantity}}),
    );
    let expected = game
        .execute(&completion, &content, 101, "quest-session")
        .unwrap();
    let delta = compact
        .execute_persisted(
            &completion,
            &content,
            101,
            "quest-session",
            None,
            &mut |_| Ok(true),
        )
        .unwrap();
    assert_eq!(delta.response, expected);
    assert_eq!(delta.evidence.len(), 2);
    assert_eq!(delta.domain_receipts.len(), 1);
    assert_eq!(delta.response["personalQuest"]["node"]["id"], "reflection");
    compact.validate_storage(&content).unwrap();
    assert!(compact.encounters.is_empty());
}
