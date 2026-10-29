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
    let mut faulty = repo.clone();
    faulty.fail_before_commit = true;
    assert_eq!(faulty.execute(&identity, &accept), Err(Error::Unavailable));
    assert_eq!(repo.read(&identity).unwrap(), initial);
    assert_eq!(count(&mut admin, "story_progress", profile), 0);
    assert_eq!(count(&mut admin, "story_command_receipts", profile), 0);

    // Two same-ID writes race under the owned profile lock and publish one result.
    let mut threads = Vec::new();
    for _ in 0..2 {
        let repo = repo.clone();
        let identity = identity.clone();
        let accept = accept.clone();
        threads.push(std::thread::spawn(move || repo.execute(&identity, &accept)));
    }
    let first = threads.remove(0).join().unwrap().unwrap();
    assert_eq!(threads.remove(0).join().unwrap().unwrap(), first);
    assert_eq!(first["revision"], 1);
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
    assert_eq!(repo.execute(&foreign, &accept), Err(Error::Forbidden));

    // Genuine restricted-role RLS checks, independent of repository owner checks.
    let mut runtime = Client::connect(&restricted, NoTls).unwrap();
    runtime
        .execute(
            "SELECT set_config('app.parent_id',$1,false)",
            &[&other_parent],
        )
        .unwrap();
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
    for table in ["story_progress", "story_command_receipts"] {
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

    let mut lifecycle = PostgresIdentity::new(Client::connect(&restricted, NoTls).unwrap());
    let verified = VerifiedContext {
        parent_id: parent.into(),
        profile_id: profile.into(),
        reauthenticated_at: 10,
    };
    let export = lifecycle.export_profile(&verified, 11).unwrap();
    assert_eq!(
        export["storyProgress"]["state"]["missionStatus"],
        "completed"
    );
    assert_eq!(
        export["storyCommandReceipts"].as_array().unwrap().len(),
        current["revision"].as_u64().unwrap() as usize
    );
    assert_eq!(export["learningEvents"], json!([]));
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
    assert_eq!(count(&mut admin, "deletion_tombstones", profile), 1);

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
