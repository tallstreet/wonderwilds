use super::*;
use crate::identity::postgres::{PostgresIdentity, VerifiedContext};
use serde_json::json;

fn command(id: &str, revision: u64, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
}
fn choose(snapshot: &Value, id: &str, kind: &str, destination: Option<&str>) -> Value {
    let action = snapshot["sceneInput"]["availableActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| {
            a["action"]["type"] == kind
                && destination.is_none_or(|d| a["action"]["destinationId"] == d)
        })
        .unwrap();
    command(
        id,
        snapshot["revision"].as_u64().unwrap(),
        json!({"type":"choose","actionId":action["id"]}),
    )
}
fn count(client: &mut Client, table: &str, profile: &str) -> i64 {
    client
        .query_one(
            &format!("SELECT count(*) FROM wonderwilds.{table} WHERE profile_id::text=$1"),
            &[&profile],
        )
        .unwrap()
        .get(0)
}

#[test]
#[ignore = "requires a fresh migrated PostgreSQL cluster in WW_STORY_TEST_DATABASE_URL"]
fn postgres_story_atomicity_retry_ownership_export_and_delete() {
    let connection = std::env::var("WW_STORY_TEST_DATABASE_URL")
        .expect("dedicated synthetic story database required");
    let mut admin = Client::connect(&connection, NoTls).unwrap();
    admin.batch_execute("CREATE ROLE ww_story_test LOGIN NOSUPERUSER NOBYPASSRLS; GRANT USAGE ON SCHEMA wonderwilds TO ww_story_test; GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_story_test; GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_story_test;").unwrap();
    let parent = "61000000-0000-0000-0000-000000000001";
    let other_parent = "61000000-0000-0000-0000-000000000002";
    let profile = "62000000-0000-0000-0000-000000000001";
    let other_profile = "62000000-0000-0000-0000-000000000002";
    for (p, c, subject) in [(parent, profile, "a"), (other_parent, other_profile, "b")] {
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-story-test',$2)", &[&p,&subject]).unwrap();
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')", &[&c,&p]).unwrap();
    }
    let mut config: postgres::Config = connection.parse().unwrap();
    config.user("ww_story_test");
    let host = match config.get_hosts().first().unwrap() {
        postgres::config::Host::Unix(path) => path.to_str().unwrap(),
        _ => panic!("private Unix socket required"),
    };
    let restricted = format!(
        "host={host} port={} user=ww_story_test dbname={}",
        config.get_ports()[0],
        config.get_dbname().unwrap()
    );
    let repo = PostgresRepository::new(restricted.clone());
    assert_eq!(repo.readiness(), Ok(()));
    let identity = Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "synthetic-story-session".into(),
    };
    let other = Identity {
        parent_id: other_parent.into(),
        profile_id: other_profile.into(),
        session_id: "synthetic-story-other".into(),
    };
    let initial = repo.read(&identity).unwrap();
    assert_eq!(initial["revision"], 0);
    assert_eq!(count(&mut admin, "story_progress", profile), 0);
    let accept = choose(&initial, "accept", "accept_mission", None);
    let planned_state = repo.planning_state(&identity, &accept).unwrap().unwrap();
    let mut plan = wonderwilds_director::story::authored_plan(&planned_state).unwrap();
    plan["goal"] = json!("A crossing planned for Pip");
    plan["milestones"][0]["label"] = json!("Find wood for our crossing.");
    let validated = std::sync::Arc::new(ValidatedPlan::new(&planned_state, plan.clone()).unwrap());
    let mut faulty = repo.clone();
    faulty.fail_before_commit = true;
    assert_eq!(
        faulty.execute_planned(&identity, &accept, Some(&validated)),
        Err(Error::Unavailable)
    );
    assert_eq!(repo.read(&identity).unwrap(), initial);
    assert_eq!(count(&mut admin, "story_progress", profile), 0);
    assert_eq!(count(&mut admin, "story_command_receipts", profile), 0);

    assert_eq!(count(&mut admin, "story_mission_plans", profile), 0);
    // Two same-ID writes race under the owned profile lock and publish one result.
    let mut threads = Vec::new();
    for _ in 0..2 {
        let repo = repo.clone();
        let identity = identity.clone();
        let accept = accept.clone();
        let validated = validated.clone();
        threads.push(std::thread::spawn(move || {
            repo.execute_planned(&identity, &accept, Some(&validated))
        }));
    }
    let first = threads.remove(0).join().unwrap().unwrap();
    assert_eq!(threads.remove(0).join().unwrap().unwrap(), first);
    assert_eq!(first["revision"], 1);
    assert_eq!(
        first["sceneInput"]["mission"]["goal"],
        "A crossing planned for Pip"
    );
    assert_eq!(repo.read(&identity).unwrap(), first);
    assert_eq!(count(&mut admin, "story_mission_plans", profile), 1);
    assert!(repo.planning_state(&identity, &accept).unwrap().is_none());
    assert_eq!(count(&mut admin, "story_command_receipts", profile), 1);
    let collect = choose(&first, "collect", "collect", None);
    assert_eq!(faulty.execute(&identity, &collect), Err(Error::Unavailable));
    assert_eq!(repo.read(&identity).unwrap(), first);
    let collected = repo.execute(&identity, &collect).unwrap();
    assert_eq!(
        collected["sceneInput"]["context"]["inventory"],
        json!([{"itemId":"wood","quantity":3}])
    );
    // Late exact retry returns its original response, not the now-current snapshot.
    assert_eq!(repo.execute(&identity, &accept).unwrap(), first);
    assert_eq!(repo.read(&identity).unwrap(), collected);
    let mut changed = accept.clone();
    changed["payload"]["actionId"] = json!("story-0-1");
    assert_eq!(
        repo.execute(&identity, &changed),
        Err(Error::RevisionConflict)
    );
    let mut stale = accept.clone();
    stale["commandId"] = json!("new-stale-command");
    assert_eq!(
        repo.execute(&identity, &stale),
        Err(Error::RevisionConflict)
    );
    let foreign = Identity {
        parent_id: other_parent.into(),
        ..identity.clone()
    };
    assert_eq!(repo.read(&foreign), Err(Error::Forbidden));
    assert!(matches!(
        repo.planning_state(&foreign, &accept),
        Err(Error::Forbidden)
    ));
    assert_eq!(repo.execute(&foreign, &accept), Err(Error::Forbidden));

    // Genuine restricted-role RLS checks, independent of repository owner checks.
    let mut runtime = Client::connect(&restricted, NoTls).unwrap();
    runtime
        .execute(
            "SELECT set_config('app.parent_id',$1,false)",
            &[&other_parent],
        )
        .unwrap();
    assert_eq!(count(&mut runtime, "story_mission_plans", profile), 0);
    assert!(runtime.execute("INSERT INTO wonderwilds.story_mission_plans(profile_id,content_version,mission_id,accepted_revision,plan) VALUES($1::text::uuid,'test','test',1,$2)", &[&profile,&plan]).is_err());
    assert_eq!(count(&mut runtime, "story_progress", profile), 0);
    assert_eq!(count(&mut runtime, "story_command_receipts", profile), 0);
    let checkpoint: Value = admin
        .query_one(
            "SELECT checkpoint FROM wonderwilds.story_progress WHERE profile_id::text=$1",
            &[&profile],
        )
        .unwrap()
        .get(0);
    assert!(runtime.execute("INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,2,$2)", &[&profile,&checkpoint]).is_err());
    for table in [
        "story_progress",
        "story_command_receipts",
        "story_mission_plans",
    ] {
        let row = admin.query_one("SELECT relrowsecurity,relforcerowsecurity FROM pg_class WHERE oid=$1::text::regclass", &[&format!("wonderwilds.{table}")]).unwrap();
        assert!(row.get::<_, bool>(0) && row.get::<_, bool>(1));
    }
    assert_eq!(repo.read(&other).unwrap()["revision"], 0);

    // Saved corruption fails closed and cannot be masked by an idempotent retry.
    let mut corrupt = checkpoint.clone();
    corrupt["state"]["wood"] = json!(999);
    admin
        .execute(
            "UPDATE wonderwilds.story_progress SET checkpoint=$2 WHERE profile_id::text=$1",
            &[&profile, &corrupt],
        )
        .unwrap();
    assert_eq!(repo.read(&identity), Err(Error::Unavailable));
    assert_eq!(repo.execute(&identity, &accept), Err(Error::Unavailable));
    admin
        .execute(
            "UPDATE wonderwilds.story_progress SET checkpoint=$2 WHERE profile_id::text=$1",
            &[&profile, &checkpoint],
        )
        .unwrap();
    admin
        .execute(
            "UPDATE wonderwilds.story_progress SET revision=3 WHERE profile_id::text=$1",
            &[&profile],
        )
        .unwrap();
    assert_eq!(repo.read(&identity), Err(Error::Unavailable));
    admin
        .execute(
            "UPDATE wonderwilds.story_progress SET revision=2 WHERE profile_id::text=$1",
            &[&profile],
        )
        .unwrap();

    // Finish through actual operations. Story never writes assessed-game data.
    let mut current = collected;
    for (id, kind, destination) in [
        ("workshop", "travel", Some("workshop")),
        ("exchange", "exchange", None),
        ("return", "travel", Some("bridge_bank")),
        ("build", "assemble", None),
        ("cross", "accompany", Some("far_bank")),
    ] {
        let request = choose(&current, id, kind, destination);
        current = repo.execute(&identity, &request).unwrap();
        assert_eq!(repo.read(&identity).unwrap(), current);
    }
    assert_eq!(current["sceneInput"]["mission"]["status"], "completed");
    assert!(current["sceneInput"]["mission"]["current"].is_null());
    assert_eq!(current["sceneInput"]["context"]["inventory"], json!([]));
    assert_eq!(repo.execute(&identity, &accept).unwrap(), first);
    for table in [
        "progress",
        "learning_events",
        "game_events",
        "game_outbox",
        "reward_grants",
    ] {
        assert_eq!(
            count(&mut admin, table, profile),
            0,
            "story touched {table}"
        );
    }
    assert_eq!(
        admin
            .query_one(
                "SELECT revision FROM wonderwilds.profiles WHERE id::text=$1",
                &[&profile]
            )
            .unwrap()
            .get::<_, i64>(0),
        0
    );

    let initial_size:i32=admin.query_one("SELECT octet_length(checkpoint::text) FROM wonderwilds.story_progress WHERE profile_id=$1::text::uuid", &[&profile]).unwrap().get(0);
    let timer = std::time::Instant::now();
    for n in 0..300 {
        let destination = if current["locationId"] == "bridge_bank" {
            "workshop"
        } else {
            "bridge_bank"
        };
        let request = choose(
            &current,
            &format!("long-travel-{n}"),
            "travel",
            Some(destination),
        );
        current = repo.execute(&identity, &request).unwrap();
        if n == 0 || n == 299 {
            assert_eq!(repo.execute(&identity, &accept).unwrap(), first);
            assert_eq!(
                PostgresRepository::new(restricted.clone())
                    .read(&identity)
                    .unwrap(),
                current
            );
        }
    }
    assert!(current["revision"].as_u64().unwrap() > 256);
    assert_eq!(current["commandBudgetExhausted"], false);
    let size:i32=admin.query_one("SELECT octet_length(checkpoint::text) FROM wonderwilds.story_progress WHERE profile_id=$1::text::uuid", &[&profile]).unwrap().get(0);
    assert!(
        size < initial_size + 1024,
        "Hot checkpoint grew with history"
    );
    assert_eq!(count(&mut admin, "story_checkpoints", profile), 2);
    eprintln!(
        "long-play story: {} commands; hot bytes {initial_size}->{size}; 300 writes {:?}",
        current["revision"],
        timer.elapsed()
    );

    assert_eq!(repo.verify_history(&identity).unwrap(), current);
    admin
        .execute(
            "UPDATE wonderwilds.story_progress SET checkpoint='{}' WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .unwrap();
    admin.execute("UPDATE wonderwilds.story_checkpoints SET digest=repeat('0',64) WHERE profile_id=$1::text::uuid", &[&profile]).unwrap();
    assert_eq!(repo.read(&identity), Err(Error::Unavailable));
    assert_eq!(repo.recover_current(&identity).unwrap(), current);
    assert_eq!(repo.read(&identity).unwrap(), current);
    assert_eq!(repo.execute(&identity, &accept).unwrap(), first);
    admin.execute("UPDATE wonderwilds.command_windows SET window_minute=floor(extract(epoch FROM clock_timestamp())/60)::bigint,commands=4096 WHERE profile_id=$1::text::uuid AND kind='story'", &[&profile]).unwrap();
    let limited = choose(&current, "limited", "travel", None);
    assert_eq!(repo.execute(&identity, &limited), Err(Error::RateLimited));
    assert_eq!(repo.execute(&identity, &accept).unwrap(), first);
    let archived:Value=admin.query_one("SELECT domain_command FROM wonderwilds.story_command_receipts WHERE profile_id=$1::text::uuid AND revision=200", &[&profile]).unwrap().get(0);
    admin.execute("UPDATE wonderwilds.story_command_receipts SET domain_command='{}' WHERE profile_id=$1::text::uuid AND revision=200", &[&profile]).unwrap();
    assert_eq!(repo.verify_history(&identity), Err(Error::Unavailable));
    assert_eq!(repo.recover_current(&identity), Err(Error::Unavailable));
    admin.execute("UPDATE wonderwilds.story_command_receipts SET domain_command=$2 WHERE profile_id=$1::text::uuid AND revision=200", &[&profile,&archived]).unwrap();
    // New tables enforce the same ownership predicate even for direct SQL.
    let mut outsider = Client::connect(&restricted, NoTls).unwrap();
    outsider
        .execute(
            "SELECT set_config('app.parent_id',$1,false)",
            &[&other_parent],
        )
        .unwrap();
    for table in [
        "command_receipts",
        "domain_command_receipts",
        "game_encounters",
        "game_checkpoints",
        "story_checkpoints",
        "persistence_origins",
        "command_windows",
    ] {
        assert_eq!(
            outsider
                .query_one(
                    &format!(
                        "SELECT count(*) FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"
                    ),
                    &[&profile]
                )
                .unwrap()
                .get::<_, i64>(0),
            0
        );
        assert_eq!(
            outsider
                .execute(
                    &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
                    &[&profile]
                )
                .unwrap(),
            0
        );
        assert!(admin.query_one("SELECT relrowsecurity AND relforcerowsecurity FROM pg_class WHERE oid=$1::text::regclass", &[&format!("wonderwilds.{table}")]).unwrap().get::<_,bool>(0));
    }
    legacy_series_upgrade(&mut admin, &repo, &other);
    v2_series_upgrade(&mut admin, &repo, &other);
    exhausted_upgrade(&mut admin, &repo, &other);
    generated_neighbourhood_replay(&repo, &other);

    let mut lifecycle = PostgresIdentity::new(Client::connect(&restricted, NoTls).unwrap());
    let verified = VerifiedContext {
        parent_id: parent.into(),
        profile_id: profile.into(),
        reauthenticated_at: 10,
    };
    let export = lifecycle.export_profile(&verified, 11).unwrap();
    assert_eq!(
        export["storyProgress"]["state"]["state"]["mission_status"],
        "completed"
    );
    assert_eq!(
        export["storyCommandReceipts"].as_array().unwrap().len(),
        current["revision"].as_u64().unwrap() as usize
    );
    assert_eq!(export["learningEvents"], json!([]));
    assert_eq!(export["story_checkpoints"].as_array().unwrap().len(), 2);
    assert_eq!(export["persistence_origins"].as_array().unwrap().len(), 1);
    assert_eq!(
        export["storyReplay"].as_array().unwrap().len(),
        current["revision"].as_u64().unwrap() as usize
    );
    assert_eq!(export["storyMissionPlans"][0]["plan"], plan);
    assert_eq!(
        current["sceneInput"]["context"]["memory"]["recentMissions"][0],
        "A crossing planned for Pip Completed."
    );
    let foreign_context = VerifiedContext {
        parent_id: other_parent.into(),
        ..verified.clone()
    };
    assert!(lifecycle.export_profile(&foreign_context, 11).is_err());
    lifecycle.delete_profile(&verified, 11).unwrap();
    assert_eq!(repo.read(&identity), Err(Error::Forbidden));
    assert_eq!(repo.execute(&identity, &accept), Err(Error::Forbidden));
    assert_eq!(count(&mut admin, "story_progress", profile), 0);
    assert_eq!(count(&mut admin, "story_command_receipts", profile), 0);
    for table in [
        "story_checkpoints",
        "persistence_origins",
        "command_windows",
    ] {
        assert_eq!(count(&mut admin, table, profile), 0);
    }
    assert_eq!(count(&mut admin, "deletion_tombstones", profile), 1);
    assert_eq!(count(&mut admin, "story_mission_plans", profile), 0);

    lifecycle
        .delete_profile(
            &VerifiedContext {
                parent_id: other_parent.into(),
                profile_id: other_profile.into(),
                reauthenticated_at: 10,
            },
            11,
        )
        .unwrap();
    admin.execute("DELETE FROM wonderwilds.deletion_tombstones WHERE profile_id::text=$1 OR profile_id::text=$2", &[&profile,&other_profile]).unwrap();
    admin
        .execute(
            "DELETE FROM wonderwilds.parents WHERE identity_issuer='synthetic-story-test'",
            &[],
        )
        .unwrap();
    admin
        .batch_execute("DROP OWNED BY ww_story_test; DROP ROLE ww_story_test;")
        .unwrap();
}

fn legacy_series_upgrade(admin: &mut Client, repo: &PostgresRepository, identity: &Identity) {
    let legacy: Value = serde_json::from_str(include_str!(
        "../../../../domain/src/story/fixtures/bridge-completed-v1.json"
    ))
    .unwrap();
    let profile = &identity.profile_id;
    for table in ["story_checkpoints", "persistence_origins"] {
        admin
            .execute(
                &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
                &[profile],
            )
            .unwrap();
    }
    admin.execute("INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,7,$2)", &[profile,&legacy]).unwrap();
    // Synthetic v1 public response receipts accompany the literal legacy checkpoint.
    // Save exact old-format responses, including the ambiguous old completion handle.
    let mut state = State::new();
    let mut originals = Vec::new();
    for saved in legacy["commands"].as_array().unwrap() {
        let command: wonderwilds_domain::story::Command =
            serde_json::from_value(saved.clone()).unwrap();
        let index = state
            .available_actions()
            .iter()
            .position(|a| *a == command.action)
            .unwrap();
        let request = json!({"schemaVersion":"1.0","commandId":command.command_id,"expectedRevision":command.expected_revision,"payload":{"type":"choose","actionId":format!("story-{}-{index}",state.revision())}});
        state.apply(&command).unwrap();
        let mut response = super::super::snapshot(&state).unwrap();
        response["contentVersion"] = json!("pip-bridge-1");
        response["sceneInput"]["mission"]
            .as_object_mut()
            .unwrap()
            .remove("missionId");
        // v1 completion had no successor acceptance. Keep only old legal choices.
        let actions = response["sceneInput"]["availableActions"]
            .as_array_mut()
            .unwrap();
        if state.mission_status() == wonderwilds_domain::story::MissionStatus::Completed {
            actions.retain(|a| a["action"]["type"] != "accept_mission");
        }
        for (index, action) in actions.iter_mut().enumerate() {
            action["id"] = json!(format!("story-{}-{index}", state.revision()));
        }
        response["scene"]["choices"] = json!(
            actions
                .iter()
                .map(|a| json!({"label":a["description"],"actionId":a["id"]}))
                .collect::<Vec<_>>()
        );
        validate("StorySnapshot", &response).unwrap();
        let revision = state.revision() as i64;
        admin.execute("INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,request,response,revision) VALUES($1::text::uuid,$2,$3,$4,$5)", &[profile,&command.command_id,&request,&response,&revision]).unwrap();
        originals.push((request, response));
    }
    let mut current = repo.read(identity).unwrap();
    assert_eq!(current["contentVersion"], "pip-stories-3");
    assert_eq!(current["sceneInput"]["mission"]["status"], "completed");
    let stale = command(
        "old-handle-new-command",
        7,
        json!({"type":"choose","actionId":"story-7-0"}),
    );
    assert_eq!(repo.execute(identity, &stale), Err(Error::RevisionConflict));
    for (request, response) in &originals {
        assert_eq!(repo.execute(identity, request).unwrap(), *response);
    }
    for (kind, destination) in [
        ("accept_mission", None),
        ("collect", None),
        ("travel", Some("bridge_bank")),
        ("travel", Some("workshop")),
        ("exchange", None),
        ("travel", Some("bridge_bank")),
        ("assemble", None),
        ("travel", Some("far_bank")),
        ("accompany", Some("bridge_bank")),
        ("accept_mission", None),
        ("travel", Some("far_bank")),
        ("collect", None),
        ("travel", Some("bridge_bank")),
        ("travel", Some("workshop")),
        ("exchange", None),
        ("travel", Some("bridge_bank")),
        ("assemble", None),
        ("travel", Some("far_bank")),
        ("assemble", None),
    ] {
        let request = choose(
            &current,
            &format!("series-{}", current["revision"]),
            kind,
            destination,
        );
        current = repo.execute(identity, &request).unwrap();
        assert_eq!(repo.read(identity).unwrap(), current);
    }
    assert_eq!(
        current["sceneInput"]["mission"]["missionId"],
        "welcome_lights"
    );
    assert_eq!(current["sceneInput"]["mission"]["status"], "completed");
    assert_eq!(
        current["sceneInput"]["context"]["memory"]["recentMissions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for (request, response) in originals {
        assert_eq!(repo.execute(identity, &request).unwrap(), response);
    }
}

fn v2_series_upgrade(admin: &mut Client, repo: &PostgresRepository, identity: &Identity) {
    // Reuse only this disposable test profile after the separate v1 assertions.
    let profile = &identity.profile_id;
    for table in ["story_checkpoints", "persistence_origins"] {
        admin
            .execute(
                &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
                &[profile],
            )
            .unwrap();
    }
    admin
        .execute(
            "DELETE FROM wonderwilds.story_command_receipts WHERE profile_id::text=$1",
            &[profile],
        )
        .unwrap();
    admin
        .execute(
            "DELETE FROM wonderwilds.story_progress WHERE profile_id::text=$1",
            &[profile],
        )
        .unwrap();
    let legacy: Value = serde_json::from_str(include_str!(
        "../../../../domain/src/story/fixtures/lights-completed-v2.json"
    ))
    .unwrap();
    let mut state = State::new();
    let mut originals = Vec::new();
    let mut skip_reaccept = false;
    for saved in legacy["commands"].as_array().unwrap() {
        let mut command: wonderwilds_domain::story::Command =
            serde_json::from_value(saved.clone()).unwrap();
        use wonderwilds_domain::story::Action;
        // The domain fixture exercises partial collections and cancellation. Build
        // a publicly playable v2 history instead, with the advertised full collect
        // actions and a receipt for every committed revision.
        match &command.action {
            Action::Collect { quantity: 1, .. } => continue,
            Action::CancelMission => {
                skip_reaccept = true;
                continue;
            }
            Action::AcceptMission if skip_reaccept => {
                skip_reaccept = false;
                continue;
            }
            Action::Collect { source_id, .. } => {
                command.action = state.available_actions().into_iter().find(|action| {
                    matches!(action, Action::Collect { source_id: offered, .. } if offered == source_id)
                }).unwrap();
            }
            Action::Acknowledge { entry_id, .. } => {
                command.action = Action::Acknowledge {
                    scene_id: format!("pip-bridge-scene-{}", state.revision()),
                    entry_id: entry_id.clone(),
                };
            }
            _ => {}
        }
        command.expected_revision = state.revision();
        let payload = if let wonderwilds_domain::story::Action::Acknowledge { scene_id, entry_id } =
            &command.action
        {
            json!({"type":"acknowledge","sceneId":scene_id,"entryId":entry_id})
        } else {
            let index = state
                .available_actions()
                .iter()
                .position(|a| a == &command.action)
                .unwrap();
            json!({"type":"choose","actionId":format!("story-v2-{}-{index}",state.revision())})
        };
        let request = self::command(&command.command_id, command.expected_revision, payload);
        state.apply(&command).unwrap();
        // Synthetic old-format receipts accompany this derived public v2 history.
        let mut response = super::super::snapshot(&state).unwrap();
        response["contentVersion"] = json!("pip-stories-2");
        let actions = response["sceneInput"]["availableActions"]
            .as_array_mut()
            .unwrap();
        if state.mission_id() == "welcome_lights"
            && state.mission_status() == wonderwilds_domain::story::MissionStatus::Completed
        {
            actions.retain(|a| a["action"]["type"] != "accept_mission");
        }
        for (index, a) in actions.iter_mut().enumerate() {
            a["id"] = json!(format!("story-v2-{}-{index}", state.revision()));
        }
        response["scene"]["choices"] = json!(
            actions
                .iter()
                .map(|a| json!({"label":a["description"],"actionId":a["id"]}))
                .collect::<Vec<_>>()
        );
        validate("StorySnapshot", &response).unwrap();
        let revision = state.revision() as i64;
        admin.execute("INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,request,response,revision) VALUES($1::text::uuid,$2,$3,$4,$5)", &[profile,&command.command_id,&request,&response,&revision]).unwrap();
        originals.push((request, response));
    }
    let mut checkpoint = state.checkpoint();
    checkpoint["version"] = json!("pip-stories-2");
    checkpoint["state"]
        .as_object_mut()
        .unwrap()
        .remove("broader");
    assert!(State::from_checkpoint(&checkpoint).is_ok());
    let revision = state.revision() as i64;
    assert_eq!(originals.len() as i64, revision);
    admin.execute("INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,$2,$3)", &[profile,&revision,&checkpoint]).unwrap();
    let mut current = repo.read(identity).unwrap();
    assert_eq!(current["contentVersion"], "pip-stories-3");
    assert_eq!(current["revision"], revision);
    let stale = command(
        "v2-handle-after-upgrade",
        revision as u64,
        json!({"type":"choose","actionId":format!("story-v2-{revision}-0")}),
    );
    assert_eq!(repo.execute(identity, &stale), Err(Error::RevisionConflict));
    for (request, response) in &originals {
        assert_eq!(repo.execute(identity, request).unwrap(), *response);
    }
    for (kind, destination) in [
        ("travel", Some("bridge_bank")),
        ("accept_mission", None),
        ("talk", None),
        ("travel", Some("far_bank")),
        ("inspect", None),
    ] {
        let request = choose(
            &current,
            &format!("broader-{}", current["revision"]),
            kind,
            destination,
        );
        let response = repo.execute(identity, &request).unwrap();
        assert_eq!(repo.execute(identity, &request).unwrap(), response);
        current = repo.read(identity).unwrap();
        assert_eq!(current, response);
    }
    assert_eq!(
        current["sceneInput"]["mission"]["missionId"],
        "humming_hollow"
    );
    assert!(
        current["sceneInput"]["context"]["memory"]["facts"]
            .as_array()
            .unwrap()
            .contains(&json!("The player listened at the humming hollow."))
    );
    for (request, response) in originals {
        assert_eq!(repo.execute(identity, &request).unwrap(), response);
    }
}

fn exhausted_upgrade(admin: &mut Client, repo: &PostgresRepository, identity: &Identity) {
    for table in [
        "story_progress",
        "story_command_receipts",
        "story_mission_plans",
        "story_checkpoints",
        "persistence_origins",
    ] {
        admin
            .execute(
                &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
                &[&identity.profile_id],
            )
            .unwrap();
    }
    let mut state = State::new();
    let mut last = None;
    for n in 0..256 {
        let request = choose(
            &super::super::snapshot(&state).unwrap(),
            &format!("old-cap-{n}"),
            "travel",
            None,
        );
        let (next, response) = super::super::execute(&state, &request).unwrap();
        admin.execute("INSERT INTO wonderwilds.story_command_receipts(profile_id,command_id,revision,request,response) VALUES($1::text::uuid,$2,$3,$4,$5)", &[&identity.profile_id,&request["commandId"].as_str().unwrap(),&((n+1) as i64),&request,&response]).unwrap();
        state = next;
        last = Some((request, response));
    }
    let (request, terminal) = last.unwrap();
    assert_eq!(terminal["commandBudgetExhausted"], true);
    admin.execute("INSERT INTO wonderwilds.story_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,256,$2)", &[&identity.profile_id,&state.checkpoint()]).unwrap();
    let resumed = repo.read(identity).unwrap();
    assert_eq!(resumed["revision"], 256);
    assert_eq!(resumed["commandBudgetExhausted"], false);
    assert_eq!(repo.execute(identity, &request).unwrap(), terminal);
    let next = choose(&resumed, "beyond-old-cap", "travel", None);
    let current = repo.execute(identity, &next).unwrap();
    assert_eq!(current["revision"], 257);
    assert_eq!(repo.verify_history(identity).unwrap(), current);
}

fn generated_neighbourhood_replay(repo: &PostgresRepository, identity: &Identity) {
    let mut current = repo.read(identity).unwrap();
    if current["locationId"] == "far_bank" {
        current = repo
            .execute(
                identity,
                &choose(&current, "gen-bank", "travel", Some("bridge_bank")),
            )
            .unwrap();
    }
    if current["locationId"] != "workshop" {
        current = repo
            .execute(
                identity,
                &choose(&current, "gen-workshop", "travel", Some("workshop")),
            )
            .unwrap();
    }
    for cycle in 0..8 {
        let travel = choose(
            &current,
            &format!("gen-travel-{cycle}"),
            "travel",
            Some("cove"),
        );
        let candidate = repo.generation_state(identity, &travel).unwrap().unwrap();
        let mut content = super::super::generation::fallback(&candidate).unwrap();
        content["name"] = json!(format!("Visitor {cycle}"));
        content["steps"] = json!([{"label":"Listen to the shell.","shape":"stones"},{"label":"Look at the winding trail.","shape":"trail"}]);
        let plan = ValidatedPlan::generated(&candidate, content.clone(), true).unwrap();
        if cycle == 0 {
            let mut faulty = repo.clone();
            faulty.fail_before_commit = true;
            assert_eq!(
                faulty.execute_planned(identity, &travel, Some(&plan)),
                Err(Error::Unavailable)
            );
            assert_eq!(repo.read(identity).unwrap(), current);
        }
        current = repo
            .execute_planned(identity, &travel, Some(&plan))
            .unwrap();
        assert_eq!(repo.execute(identity, &travel).unwrap(), current);
        assert!(repo.generation_state(identity, &travel).unwrap().is_none());
        assert_eq!(
            current["sceneInput"]["neighbourMissions"][0]["generatedStory"],
            content
        );
        assert_eq!(
            current["sceneInput"]["neighbourMissions"][0]["generation"],
            cycle + 1
        );
        for (step, kind) in ["talk", "inspect", "inspect", "talk"].iter().enumerate() {
            current = repo
                .execute(
                    identity,
                    &choose(&current, &format!("gen-{cycle}-{step}"), kind, None),
                )
                .unwrap();
        }
        assert_eq!(
            current["sceneInput"]["neighbourMissions"][0]["status"],
            "completed"
        );
        assert_eq!(repo.verify_history(identity).unwrap(), current);
        assert_eq!(
            PostgresRepository::new(repo.connection.clone())
                .read(identity)
                .unwrap(),
            current
        );
        current = repo
            .execute(
                identity,
                &choose(
                    &current,
                    &format!("gen-return-{cycle}"),
                    "travel",
                    Some("workshop"),
                ),
            )
            .unwrap();
    }
}
