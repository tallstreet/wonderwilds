use super::*;
use crate::{
    game::Identity,
    identity::postgres::{PostgresIdentity, VerifiedContext},
    story::repository::{PostgresRepository as StoryRepository, Repository as StoryApi},
};
use serde_json::{Value, json};

fn command(id: &str, revision: u64, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
}

fn story_action(snapshot: &Value, kind: &str, destination: Option<&str>) -> Value {
    let action = snapshot["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| {
            candidate["action"]["type"] == kind
                && destination.is_none_or(|value| candidate["action"]["destinationId"] == value)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing story action {kind} to {destination:?}: {}",
                snapshot["sceneInput"]["availableActions"]
            )
        });
    command(
        &format!("story-{}-{}", snapshot["revision"], kind),
        snapshot["revision"].as_u64().unwrap(),
        json!({"type":"choose","actionId":action["id"]}),
    )
}

fn travel(story: &StoryRepository, identity: &Identity, destination: &str) -> Value {
    let snapshot = story.read(identity).unwrap();
    if snapshot["locationId"] == destination {
        return snapshot;
    }
    let request = story_action(&snapshot, "travel", Some(destination));
    story.execute(identity, &request).unwrap()
}

fn answer_for(activity: &Value) -> Value {
    let task = &activity["task"];
    match task["kind"].as_str().unwrap() {
        "pairing" => {
            let objects = task["objects"].as_array().unwrap();
            let recipients = task["recipients"].as_array().unwrap();
            assert_eq!(objects.len(), recipients.len());
            json!({
                "kind":"pairing",
                "assignments":objects.iter().zip(recipients).map(|(object,recipient)|json!({"objectId":object["id"],"recipientId":recipient["id"]})).collect::<Vec<_>>()
            })
        }
        "count_collection" => {
            let count = task["objects"].as_array().unwrap().len() as i64;
            let choice = task["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|choice| choice["value"] == count)
                .unwrap();
            json!({"kind":"choice","choiceId":choice["id"]})
        }
        "compare_collections" => {
            let groups = task["collections"].as_array().unwrap();
            let mode = task["promptMode"].as_str().unwrap();
            let target = groups
                .iter()
                .min_by_key(|group| {
                    let count = group["items"].as_array().unwrap().len();
                    if mode == "fewer" {
                        count
                    } else {
                        usize::MAX - count
                    }
                })
                .unwrap();
            json!({"kind":"choice","choiceId":target["id"]})
        }
        "number_path" => {
            let shown = task["shown"].as_array().unwrap();
            let last = shown.last().unwrap().as_i64().unwrap();
            let delta = if task["direction"] == "forward" {
                1
            } else {
                -1
            };
            let value = last + delta;
            let choice = task["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|choice| choice["value"] == value)
                .unwrap();
            json!({"kind":"choice","choiceId":choice["id"]})
        }
        "part_whole" => {
            let value = task["whole"].as_i64().unwrap() - task["shownPart"].as_i64().unwrap();
            let choice = task["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|choice| choice["value"] == value)
                .unwrap();
            json!({"kind":"choice","choiceId":choice["id"]})
        }
        kind => panic!("unknown mission task kind {kind}"),
    }
}

fn begin(
    repo: &PostgresRepository,
    identity: &Identity,
    snapshot: &Value,
    id: &str,
) -> (Value, Value) {
    let request = command(
        id,
        snapshot["revision"].as_u64().unwrap(),
        json!({"type":"begin"}),
    );
    let response = repo.execute(identity, &request).unwrap();
    (request, response)
}

fn submit(
    repo: &PostgresRepository,
    identity: &Identity,
    snapshot: &Value,
    id: &str,
    answer: Value,
) -> Value {
    let active = &snapshot["activeActivity"];
    let request = command(
        id,
        snapshot["revision"].as_u64().unwrap(),
        json!({"type":"answer","encounterId":active["id"],"answer":answer}),
    );
    repo.execute(identity, &request).unwrap_or_else(|error| {
        panic!("mission submit {id} failed: {error:?}; request={request}; active={active}")
    })
}

fn row_count(client: &mut Client, table: &str, profile: &str) -> i64 {
    client
        .query_one(
            &format!("SELECT count(*) FROM wonderwilds.{table} WHERE profile_id::text=$1"),
            &[&profile],
        )
        .unwrap()
        .get(0)
}

#[test]
fn all_authored_mission_activities_match_closed_wire_contracts_without_answer_keys() {
    for skill in ["M01", "M03", "M04", "M05", "M06"] {
        let mut fingerprints = std::collections::BTreeSet::new();
        for seed in 0..12 {
            let activity = mission::authored_activity(skill, seed, "schema-check").unwrap();
            fingerprints.insert(activity.canonical_fingerprint().to_owned());
            let mut value = serde_json::to_value(&activity.view).unwrap();
            value["support"] = json!({"hintCount":0,"workedExample":false,"readAloud":false});
            validate("MissionActivity", &value)
                .unwrap_or_else(|error| panic!("{skill}/{seed}: {error:?}"));
            assert!(!value.to_string().contains("answerKey"));
            let themed = mission::discovery_activity(activity.clone());
            validate("MissionLearningFeedback", &json!({"result":"assisted","message":themed.discovery_solution(),"completed":true})).unwrap();
            validate("MissionLearningAnswer", &answer_for(&value)).unwrap();
        }
        assert!(
            fingerprints.len() >= 8,
            "{skill} has only {} canonical task shapes",
            fingerprints.len()
        );
    }
}

#[test]
#[ignore = "requires a fresh migrated PostgreSQL cluster in WW_MISSION_LEARNING_TEST_DATABASE_URL or WW_STORY_TEST_DATABASE_URL"]
fn postgres_mission_learning_atomicity_retry_ownership_export_and_delete() {
    let connection = std::env::var("WW_MISSION_LEARNING_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("WW_STORY_TEST_DATABASE_URL"))
        .expect("dedicated synthetic mission-learning database required");
    let mut admin = Client::connect(&connection, NoTls).unwrap();
    admin.batch_execute("CREATE ROLE ww_mission_test LOGIN NOSUPERUSER NOBYPASSRLS; GRANT USAGE ON SCHEMA wonderwilds TO ww_mission_test; GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_mission_test; GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_mission_test;").unwrap();
    let parent = "71000000-0000-0000-0000-000000000001";
    let other_parent = "71000000-0000-0000-0000-000000000002";
    let profile = "72000000-0000-0000-0000-000000000001";
    let other_profile = "72000000-0000-0000-0000-000000000002";
    for (parent_id, profile_id, subject) in [
        (parent, profile, "primary"),
        (other_parent, other_profile, "other"),
    ] {
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-mission-test',$2)", &[&parent_id,&subject]).unwrap();
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')", &[&profile_id,&parent_id]).unwrap();
    }
    let mut config: postgres::Config = connection.parse().unwrap();
    config.user("ww_mission_test");
    let host = match config.get_hosts().first().unwrap() {
        postgres::config::Host::Unix(path) => path.to_str().unwrap(),
        _ => panic!("private Unix socket required for disposable database tests"),
    };
    let restricted = format!(
        "host={host} port={} user=ww_mission_test dbname={}",
        config.get_ports()[0],
        config.get_dbname().unwrap()
    );
    let repo = PostgresRepository::new(restricted.clone());
    let story = StoryRepository::new(restricted.clone());
    assert_eq!(repo.readiness(), Ok(()));
    let identity_a = Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "synthetic-mission-session-a".into(),
    };
    let identity_b = Identity {
        session_id: "synthetic-mission-session-b".into(),
        ..identity_a.clone()
    };
    let foreign = Identity {
        parent_id: other_parent.into(),
        profile_id: profile.into(),
        session_id: "synthetic-mission-other".into(),
    };

    let story_initial = story.read(&identity_a).unwrap();
    let accept = story_action(&story_initial, "accept_mission", None);
    let story_active = story.execute(&identity_a, &accept).unwrap();
    assert_eq!(story_active["locationId"], "bridge_bank");
    let initial = repo.read(&identity_a).unwrap();
    assert_eq!(initial["revision"], 0);
    assert_eq!(initial["available"], true);
    assert!(initial["invitation"]["title"].as_str().is_some());
    assert_eq!(
        row_count(&mut admin, "mission_learning_progress", profile),
        0
    );
    assert_eq!(row_count(&mut admin, "progress", profile), 0);

    // First activity is a one-item pairing. It is introduced, gets support only after commit,
    // then a correct supported answer completes it without independent credit.
    let (begin_request, begun) = begin(&repo, &identity_a, &initial, "begin-supported");
    let activity = begun["activeActivity"].clone();
    assert!(activity["revealedSupport"].is_null());
    let support = command(
        "support-first",
        1,
        json!({"type":"support","encounterId":activity["id"],"kind":"hint"}),
    );
    let supported = repo.execute(&identity_a, &support).unwrap();
    assert_eq!(
        supported["activeActivity"]["revealedSupport"]["kind"],
        "hint"
    );
    let supported_answer = command(
        "supported-answer",
        2,
        json!({"type":"answer","encounterId":activity["id"],"answer":answer_for(&activity)}),
    );
    let completed = repo.execute(&identity_a, &supported_answer).unwrap();
    assert_eq!(completed["lastFeedback"]["result"], "correct");
    assert_eq!(completed["activeActivity"], Value::Null);
    let event_count_before_retry = row_count(&mut admin, "mission_learning_events", profile);
    assert_eq!(repo.execute(&identity_a, &begin_request).unwrap(), begun);
    assert_eq!(
        row_count(&mut admin, "mission_learning_events", profile),
        event_count_before_retry
    );
    let mut changed = begin_request.clone();
    changed["payload"] = json!({"type":"abandon","encounterId":"different"});
    assert_eq!(
        repo.execute(&identity_a, &changed),
        Err(Error::RevisionConflict)
    );

    // A malformed-but-well-shaped non-bijection is assessed incorrect, then support and a
    // corrected answer complete the activity. Only its first unsupported check is recorded.
    let (_, second_begun) = begin(&repo, &identity_a, &completed, "begin-wrong");
    let second_activity = second_begun["activeActivity"].clone();
    let objects = second_activity["task"]["objects"].as_array().unwrap();
    let recipients = second_activity["task"]["recipients"].as_array().unwrap();
    assert!(objects.len() >= 2);
    let wrong = json!({"kind":"pairing","assignments":objects.iter().map(|o|json!({"objectId":o["id"],"recipientId":recipients[0]["id"]})).collect::<Vec<_>>()});
    let incorrect_request = command(
        "wrong-first",
        second_begun["revision"].as_u64().unwrap(),
        json!({"type":"answer","encounterId":second_activity["id"],"answer":wrong}),
    );
    let incorrect = repo.execute(&identity_a, &incorrect_request).unwrap();
    assert_eq!(incorrect["lastFeedback"]["result"], "incorrect");
    assert_eq!(incorrect["activeActivity"]["id"], second_activity["id"]);
    let hint = command(
        "hint-second",
        incorrect["revision"].as_u64().unwrap(),
        json!({"type":"support","encounterId":second_activity["id"],"kind":"hint"}),
    );
    let hinted = repo.execute(&identity_a, &hint).unwrap();
    assert!(
        hinted["activeActivity"]["revealedSupport"]["text"]
            .as_str()
            .unwrap()
            .contains("object")
    );
    let retry_correct = command(
        "retry-correct",
        hinted["revision"].as_u64().unwrap(),
        json!({"type":"answer","encounterId":second_activity["id"],"answer":answer_for(&second_activity)}),
    );
    let second_completed = repo.execute(&identity_a, &retry_correct).unwrap();
    assert_eq!(second_completed["lastFeedback"]["result"], "correct");
    let m01 = second_completed["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|skill| skill["skillId"] == "M01")
        .unwrap();
    assert_eq!(m01["eligibleChecks"], 1);
    assert_eq!(m01["correctChecks"], 0);

    // Fail after the command has staged its receipt/checkpoint/event. Every write rolls back.
    let (_, rollback_active) = begin(&repo, &identity_a, &second_completed, "begin-rollback");
    let rollback_answer = command(
        "rollback-answer",
        rollback_active["revision"].as_u64().unwrap(),
        json!({"type":"answer","encounterId":rollback_active["activeActivity"]["id"],"answer":answer_for(&rollback_active["activeActivity"])}),
    );
    let before_event_count = row_count(&mut admin, "mission_learning_events", profile);
    let mut failing = repo.clone();
    failing.fail_before_commit = true;
    assert_eq!(
        failing.execute(&identity_a, &rollback_answer),
        Err(Error::Unavailable)
    );
    assert_eq!(repo.read(&identity_a).unwrap(), rollback_active);
    assert_eq!(
        row_count(&mut admin, "mission_learning_events", profile),
        before_event_count
    );
    let rollback_committed = repo.execute(&identity_a, &rollback_answer).unwrap();
    assert_eq!(rollback_committed["lastFeedback"]["result"], "correct");

    // Same-command concurrent begin publishes exactly one response/encounter.
    let race_request = command(
        "race-begin",
        rollback_committed["revision"].as_u64().unwrap(),
        json!({"type":"begin"}),
    );
    let mut workers = Vec::new();
    for _ in 0..2 {
        let repo = repo.clone();
        let identity = identity_a.clone();
        let request = race_request.clone();
        workers.push(std::thread::spawn(move || {
            repo.execute(&identity, &request)
        }));
    }
    let race_first = workers.remove(0).join().unwrap().unwrap();
    let race_second = workers.remove(0).join().unwrap().unwrap();
    assert_eq!(race_first, race_second);
    assert_eq!(
        race_first["activeActivity"]["id"],
        race_second["activeActivity"]["id"]
    );

    assert_eq!(repo.read(&foreign), Err(Error::Forbidden));
    assert_eq!(repo.execute(&foreign, &race_request), Err(Error::Forbidden));
    let mut runtime = Client::connect(&restricted, NoTls).unwrap();
    runtime
        .execute(
            "SELECT set_config('app.parent_id',$1,false)",
            &[&other_parent],
        )
        .unwrap();
    for table in [
        "mission_learning_progress",
        "mission_learning_command_receipts",
        "mission_learning_events",
    ] {
        assert_eq!(row_count(&mut runtime, table, profile), 0);
        let row = admin.query_one("SELECT relrowsecurity,relforcerowsecurity FROM pg_class WHERE oid=$1::text::regclass", &[&format!("wonderwilds.{table}")]).unwrap();
        assert!(row.get::<_, bool>(0) && row.get::<_, bool>(1));
    }
    assert!(runtime.execute(
        "INSERT INTO wonderwilds.mission_learning_progress(profile_id,revision,checkpoint,digest) VALUES($1::text::uuid,0,'{}',repeat('0',64))",
        &[&profile],
    ).is_err());
    assert_eq!(row_count(&mut admin, "progress", profile), 0);
    assert_eq!(row_count(&mut admin, "game_events", profile), 0);
    assert_eq!(row_count(&mut admin, "story_command_receipts", profile), 1);

    // Use real generated tasks in two authorized sessions and two story locations.
    // The API's active skill selection enforces M01 -> M03 -> M04/M05/M06.
    let mut next_id = 0_u64;
    let mut current_story = story_active;
    let mut practice_attempts = 0_u8;
    let before_journey_event_count = row_count(&mut admin, "mission_learning_events", profile);
    while practice_attempts < 64 {
        let current = repo.read(&identity_a).unwrap();
        let Some(skill) = current["frontierSkillId"].as_str().map(str::to_owned) else {
            break;
        };
        let mut session_identity = identity_a.clone();
        let context_revision;
        let started;
        if !current["activeActivity"].is_null() {
            // The racing begin above left one server-issued M01 activity active in context A.
            assert_eq!(current["activeActivity"]["skillId"], skill);
            context_revision = current_story["revision"].clone();
            started = current;
        } else {
            let progress = current["skills"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["skillId"] == skill)
                .unwrap();
            // Real frontier selection can interleave M04/M05/M06. Each skill starts in A,
            // then its later checks use session/context B.
            let second_context = progress["eligibleChecks"].as_u64().unwrap() >= 5;
            let (identity, location) = if second_context {
                (&identity_b, "workshop")
            } else {
                (&identity_a, "bridge_bank")
            };
            session_identity = identity.clone();
            current_story = travel(&story, identity, location);
            context_revision = current_story["revision"].clone();
            let snapshot = repo.read(identity).unwrap();
            assert_eq!(snapshot["frontierSkillId"], skill);
            started = begin(
                &repo,
                identity,
                &snapshot,
                &format!("journey-begin-{next_id}"),
            )
            .1;
        }
        assert_eq!(started["activeActivity"]["skillId"], skill);
        let answer = answer_for(&started["activeActivity"]);
        let response = submit(
            &repo,
            &session_identity,
            &started,
            &format!("journey-answer-{next_id}"),
            answer,
        );
        assert_eq!(
            response["lastFeedback"]["result"], "correct",
            "{skill} task {:?}",
            started["activeActivity"]["task"]["kind"]
        );
        assert_eq!(response["lastFeedback"]["completed"], true);
        assert_eq!(
            story.read(&session_identity).unwrap()["revision"],
            context_revision,
            "mission practice changed story state"
        );
        next_id += 1;
        practice_attempts += 1;
    }
    assert!(
        practice_attempts < 64,
        "frontier failed to exhaust the supported graph"
    );
    let current = repo.read(&identity_b).unwrap();
    let completed_skills: std::collections::BTreeSet<String> = current["skills"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["supportedByMission"] == true && item["status"] == "ready_for_next")
        .map(|item| item["skillId"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        completed_skills,
        ["M01", "M03", "M04", "M05", "M06"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    for skill in &completed_skills {
        let progress = current["skills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["skillId"].as_str() == Some(skill.as_str()))
            .unwrap();
        assert_eq!(progress["eligibleChecks"], 8, "{skill}");
        assert_eq!(progress["distinctSessions"], 2, "{skill}");
        assert_eq!(progress["distinctContexts"], 2, "{skill}");
    }
    assert_eq!(current["available"], false);
    assert_eq!(current["frontierSkillId"], Value::Null);
    assert!(
        current["skills"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["supportedByMission"] == true)
            .all(|s| s["status"] == "ready_for_next")
    );
    assert!(
        current["skills"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["supportedByMission"] == false)
            .all(|s| s["status"] == "unavailable")
    );
    assert_eq!(
        row_count(&mut admin, "story_command_receipts", profile),
        current_story["revision"].as_i64().unwrap()
    );
    assert_eq!(row_count(&mut admin, "progress", profile), 0);
    assert_eq!(row_count(&mut admin, "learning_events", profile), 0);
    assert_eq!(row_count(&mut admin, "game_events", profile), 0);

    // Durable full event replay agrees with the compact <=60-item checkpoint.
    let loaded = {
        let mut client = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        PostgresRepository::own_profile(&mut tx, &identity_b).unwrap();
        let loaded = PostgresRepository::load(&mut tx, profile).unwrap();
        PostgresRepository::verify_history(&mut tx, profile, &loaded).unwrap();
        tx.commit().unwrap();
        loaded
    };
    assert!(loaded.policy.observations.len() <= 60);
    let final_event_count = row_count(&mut admin, "mission_learning_events", profile);
    assert_eq!(
        final_event_count,
        before_journey_event_count + i64::from(practice_attempts)
    );
    let checkpoint_bytes: i32 = admin.query_one("SELECT octet_length(checkpoint::text) FROM wonderwilds.mission_learning_progress WHERE profile_id=$1::text::uuid", &[&profile]).unwrap().get(0);
    assert!(
        checkpoint_bytes < 65_536,
        "bounded policy checkpoint grew to {checkpoint_bytes} bytes"
    );
    assert_eq!(repo.execute(&identity_a, &begin_request).unwrap(), begun);

    exercise_inline_steps(&mut admin, &restricted, parent);

    // Export includes the mission checkpoint, immutable events, and command receipts; delete cascades them.
    let verified = VerifiedContext {
        parent_id: parent.into(),
        profile_id: profile.into(),
        reauthenticated_at: 10,
    };
    let mut lifecycle = PostgresIdentity::new(Client::connect(&restricted, NoTls).unwrap());
    let export = lifecycle.export_profile(&verified, 11).unwrap();
    assert!(export["missionLearningProgress"].is_object());
    assert_eq!(
        export["missionLearningEvents"].as_array().unwrap().len(),
        final_event_count as usize
    );
    assert_eq!(
        export["missionLearningReceipts"].as_array().unwrap().len(),
        loaded.revision as usize
    );
    lifecycle.delete_profile(&verified, 11).unwrap();
    for table in [
        "mission_learning_progress",
        "mission_learning_command_receipts",
        "mission_learning_events",
        "story_progress",
        "story_command_receipts",
    ] {
        assert_eq!(
            row_count(&mut admin, table, profile),
            0,
            "profile delete left rows in {table}"
        );
    }
}

fn exercise_inline_steps(admin: &mut Client, restricted: &str, parent: &str) {
    let profile = "72000000-0000-0000-0000-000000000003";
    admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')", &[&profile,&parent]).unwrap();
    let identity = Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "inline-synthetic-session".into(),
    };
    let repo = PostgresRepository::new(restricted.into());
    let story = StoryRepository::new(restricted.into());
    travel(&story, &identity, "workshop");
    let at_workshop = story.read(&identity).unwrap();
    let arrival = story_action(&at_workshop, "travel", Some("cove"));
    let candidate = story
        .generation_state(&identity, &arrival)
        .unwrap()
        .unwrap();
    let content = json!({"name":"Lumi","personality":"A curious listener.","sprite":"flutter","greeting":"Shall we listen?","title":"The quiet hollow","premise":"Find what makes the hollow hum.","steps":[
        {"label":"Listen at the hollow.","shape":"hollow","learningBridge":"Help organise our discovery markers so we can keep track of the sounds."},
        {"label":"Look beneath the leaves.","shape":"leaf"},
        {"label":"Notice the smooth stones.","shape":"stones"}
    ],"payoff":"Lumi and you shared the hollow's song."});
    let plan = crate::story::planning::ValidatedPlan::generated(&candidate, content, true).unwrap();
    let offered = story
        .execute_planned(&identity, &arrival, Some(&plan))
        .unwrap();
    let start = offered["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["topicId"] == "start_neighbour_story")
        .unwrap();
    let accepted = story
        .execute(
            &identity,
            &command(
                "inline-accept",
                offered["revision"].as_u64().unwrap(),
                json!({"type":"choose","actionId":start["id"]}),
            ),
        )
        .unwrap();
    let begin_step = |story_snapshot: &Value, learning: &Value, id: &str| {
        let action = story_snapshot["sceneInput"]["availableActions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"]["type"] == "inspect")
            .unwrap();
        command(
            id,
            learning["revision"].as_u64().unwrap(),
            json!({"type":"begin_step","actionId":action["id"],"storyRevision":story_snapshot["revision"]}),
        )
    };
    let initial = repo.read(&identity).unwrap();
    let travel_action = accepted["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["type"] == "travel")
        .unwrap();
    assert_eq!(repo.execute(&identity, &command("inline-invalid-travel", 0, json!({"type":"begin_step","actionId":travel_action["id"],"storyRevision":accepted["revision"]}))), Err(Error::InvalidInput));
    let begin_request = begin_step(&accepted, &initial, "inline-begin-1");
    let mut stale_begin = begin_request.clone();
    stale_begin["payload"]["storyRevision"] = json!(0);
    assert_eq!(
        repo.execute(&identity, &stale_begin),
        Err(Error::RevisionConflict)
    );
    let first = repo.execute(&identity, &begin_request).unwrap();
    assert_eq!(repo.execute(&identity, &begin_request).unwrap(), first);
    assert_eq!(
        repo.read(&identity).unwrap()["activeActivity"],
        first["activeActivity"]
    );
    assert_eq!(
        first["activeActivity"]["storyStep"]["introduction"],
        "Help organise our discovery markers so we can keep track of the sounds."
    );
    assert_eq!(
        first["activeActivity"]["task"]["objects"][0]["label"],
        "Discovery marker 1"
    );
    let answer = command(
        "inline-answer-1",
        first["revision"].as_u64().unwrap(),
        json!({"type":"answer","encounterId":first["activeActivity"]["id"],"answer":answer_for(&first["activeActivity"])}),
    );
    let failing = PostgresRepository {
        connection: restricted.into(),
        fail_before_commit: true,
    };
    assert_eq!(failing.execute(&identity, &answer), Err(Error::Unavailable));
    assert_eq!(
        story.read(&identity).unwrap(),
        accepted,
        "story advancement rolled back with learning"
    );
    assert_eq!(repo.read(&identity).unwrap(), first);
    let correct = repo.execute(&identity, &answer).unwrap();
    assert_eq!(correct["lastFeedback"]["result"], "correct");
    let second_story = story.read(&identity).unwrap();
    assert_eq!(
        second_story["revision"].as_u64().unwrap(),
        accepted["revision"].as_u64().unwrap() + 1
    );
    assert_eq!(repo.execute(&identity, &answer).unwrap(), correct);
    assert_eq!(story.read(&identity).unwrap(), second_story);
    story.verify_history(&identity).unwrap();

    let second = repo
        .execute(
            &identity,
            &begin_step(&second_story, &correct, "inline-begin-2"),
        )
        .unwrap();
    assert!(
        second["activeActivity"]["storyStep"]["introduction"]
            .as_str()
            .unwrap()
            .contains("organise"),
        "old generated steps use authored joining copy"
    );
    let mut wrong = answer_for(&second["activeActivity"]);
    wrong["assignments"][1]["recipientId"] = wrong["assignments"][0]["recipientId"].clone();
    let incorrect = submit(&repo, &identity, &second, "inline-wrong", wrong);
    assert_eq!(incorrect["lastFeedback"]["result"], "incorrect");
    assert_eq!(story.read(&identity).unwrap(), second_story);
    let together = command(
        "inline-together",
        incorrect["revision"].as_u64().unwrap(),
        json!({"type":"finish_together","encounterId":incorrect["activeActivity"]["id"]}),
    );
    let assisted = repo.execute(&identity, &together).unwrap();
    assert_eq!(assisted["lastFeedback"]["result"], "assisted");
    let event: Value = admin.query_one("SELECT record FROM wonderwilds.mission_learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence DESC LIMIT 1", &[&profile]).unwrap().get(0);
    assert_eq!(
        event["observation"]["evidence"]["eventKind"],
        "worked_example"
    );
    assert_eq!(event["observation"]["evidence"]["result"], "not_assessed");
    assert_eq!(
        event["observation"]["evidence"]["support"]["workedExample"],
        true
    );
    assert_eq!(
        assisted["skills"][0]["correctChecks"],
        correct["skills"][0]["correctChecks"]
    );
    let third_story = story.read(&identity).unwrap();
    assert_eq!(
        third_story["revision"].as_u64().unwrap(),
        second_story["revision"].as_u64().unwrap() + 1
    );
    assert_eq!(repo.execute(&identity, &together).unwrap(), assisted);
    assert_eq!(story.read(&identity).unwrap(), third_story);
    story.verify_history(&identity).unwrap();

    let third = repo
        .execute(
            &identity,
            &begin_step(&third_story, &assisted, "inline-begin-3"),
        )
        .unwrap();
    let moved = travel(&story, &identity, "workshop");
    let stale_finish = command(
        "inline-stale-finish",
        third["revision"].as_u64().unwrap(),
        json!({"type":"finish_together","encounterId":third["activeActivity"]["id"]}),
    );
    assert_eq!(
        repo.execute(&identity, &stale_finish),
        Err(Error::RevisionConflict)
    );
    assert_eq!(story.read(&identity).unwrap(), moved);
    assert_eq!(
        repo.read(&identity).unwrap()["activeActivity"],
        third["activeActivity"]
    );
    let abandoned = repo
        .execute(
            &identity,
            &command(
                "inline-put-aside",
                third["revision"].as_u64().unwrap(),
                json!({"type":"abandon","encounterId":third["activeActivity"]["id"]}),
            ),
        )
        .unwrap();
    assert!(abandoned.get("activeActivity").is_none());
    assert_eq!(story.read(&identity).unwrap(), moved);
    let mut client = Client::connect(restricted, NoTls).unwrap();
    let mut tx = client.transaction().unwrap();
    PostgresRepository::own_profile(&mut tx, &identity).unwrap();
    let loaded = PostgresRepository::load(&mut tx, profile).unwrap();
    PostgresRepository::verify_history(&mut tx, profile, &loaded).unwrap();
    tx.commit().unwrap();
}
