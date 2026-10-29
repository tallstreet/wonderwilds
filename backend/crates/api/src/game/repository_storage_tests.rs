use super::*;

fn command(id: &str, revision: u64, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
}
fn scalar(admin: &mut Client, sql: &str, profile: &str) -> i64 {
    admin.query_one(sql, &[&profile]).unwrap().get(0)
}

#[test]
#[ignore = "requires dedicated migrated UTF8 PostgreSQL database in WW_GAME_TEST_DATABASE_URL"]
fn postgres_durable_storage_migration_checkpoints_and_long_profile() {
    let connection = std::env::var("WW_GAME_TEST_DATABASE_URL").unwrap();
    let mut admin = Client::connect(&connection, NoTls).unwrap();
    let role = format!("ww_storage_test_{}", std::process::id());
    admin.batch_execute(&format!("CREATE ROLE {role} LOGIN NOSUPERUSER NOBYPASSRLS; GRANT USAGE ON SCHEMA wonderwilds TO {role}; GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO {role}; GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO {role};")).unwrap();
    let parent = "61000000-0000-0000-0000-000000000001";
    let profile = "62000000-0000-0000-0000-000000000001";
    admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'storage-test','parent')", &[&parent]).unwrap();
    let content = development_content().unwrap();
    // Real legacy API receipts/evidence, including a supported pending training task.
    let mut legacy = super::super::companion_tests::recruited_pip_via_api(&content);
    let old_request = command(
        "legacy-team",
        legacy.revision,
        json!({"type":"select_team","creatureIds":["pip"]}),
    );
    let old_response = legacy
        .execute(&old_request, &content, 1, "legacy-session")
        .unwrap();
    let started_request = command(
        "legacy-training",
        legacy.revision,
        json!({"type":"start_encounter","beatId":"training-pip-1"}),
    );
    let started = legacy
        .execute(&started_request, &content, 2, "legacy-session")
        .unwrap();
    let eid = started["pendingEncounters"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    legacy
        .execute(
            &command(
                "legacy-hint",
                legacy.revision,
                json!({"type":"hint","encounterId":eid,"supportKind":"hint"}),
            ),
            &content,
            3,
            "legacy-session",
        )
        .unwrap();
    let expected_before = legacy.snapshot(&content).unwrap();
    admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band,revision) VALUES($1::text::uuid,$2::text::uuid,'beginning',$3)", &[&profile,&parent,&sequence(legacy.revision).unwrap()]).unwrap();
    admin
        .execute(
            "INSERT INTO wonderwilds.progress(profile_id,snapshot) VALUES($1::text::uuid,$2)",
            &[&profile, &serde_json::to_value(&legacy).unwrap()],
        )
        .unwrap();
    // Seed the same durable event copies the legacy repository writes.
    for e in &legacy.evidence {
        admin.execute("INSERT INTO wonderwilds.learning_events(profile_id,sequence,evidence) VALUES($1::text::uuid,$2,$3)", &[&profile,&sequence(e.server_sequence).unwrap(),&serde_json::to_value(e).unwrap()]).unwrap();
    }
    for e in &legacy.events {
        admin.execute("INSERT INTO wonderwilds.game_events(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3)", &[&profile,&sequence(e.sequence).unwrap(),&serde_json::to_value(e).unwrap()]).unwrap();
    }
    let mut config: postgres::Config = connection.parse().unwrap();
    config.user(&role);
    let host = match config.get_hosts().first().unwrap() {
        postgres::config::Host::Unix(path) => path.to_str().unwrap(),
        _ => panic!("private synthetic socket required"),
    };
    let restricted = format!(
        "host={host} port={} user={role} dbname={}",
        config.get_ports()[0],
        config.get_dbname().unwrap()
    );
    let repo = PostgresRepository::new(restricted, content.clone());
    let identity = Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "new-session".into(),
    };
    // Failed migration cannot publish a reset or accept an orphan future retry.
    admin
        .execute(
            "UPDATE wonderwilds.profiles SET revision=revision+1 WHERE id=$1::text::uuid",
            &[&profile],
        )
        .unwrap();
    assert!(matches!(repo.read(&identity), Err(Error::Unavailable)));
    admin
        .execute(
            "UPDATE wonderwilds.profiles SET revision=revision-1 WHERE id=$1::text::uuid",
            &[&profile],
        )
        .unwrap();
    let legacy_json = serde_json::to_value(&legacy).unwrap();
    admin
        .execute(
            "DELETE FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .unwrap();
    assert!(matches!(repo.read(&identity), Err(Error::Unavailable)));
    admin
        .execute(
            "INSERT INTO wonderwilds.progress(profile_id,snapshot) VALUES($1::text::uuid,$2)",
            &[&profile, &legacy_json],
        )
        .unwrap();
    admin.execute("INSERT INTO wonderwilds.command_receipts(profile_id,command_id,request,response) VALUES($1::text::uuid,'orphan','{}','{}')", &[&profile]).unwrap();
    assert!(matches!(repo.read(&identity), Err(Error::Unavailable)));
    admin.execute("DELETE FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND command_id='orphan'", &[&profile]).unwrap();
    assert_eq!(
        repo.read(&identity).unwrap().snapshot(&content).unwrap(),
        expected_before
    );
    assert_eq!(
        repo.execute(&identity, &old_request, 1000).unwrap(),
        old_response,
        "legacy exact response survives migration before revision/rate checks"
    );
    let compact = repo.read(&identity).unwrap();
    assert!(compact.is_storage_checkpoint());
    assert!(
        compact.evidence.is_empty() && compact.events.is_empty() && compact.receipts.is_empty()
    );
    assert_eq!(
        compact.public_encounter(&eid).unwrap()["support"]["hintCount"],
        1
    );
    assert_eq!(
        compact.public_encounter(&eid).unwrap()["training"]["creatureId"],
        "pip"
    );
    let enc = compact.public_encounter(&eid).unwrap();
    let answer =
        enc["parameters"]["total"].as_u64().unwrap() - enc["parameters"]["given"].as_u64().unwrap();
    let completion = command(
        "migrated-completion",
        compact.revision,
        json!({"type":"attempt","encounterId":eid,"answer":{"kind":"quantity","value":answer}}),
    );
    let completed = repo.execute(&identity, &completion, 1001).unwrap();
    assert_eq!(completed["characterProgression"][0]["xp"], 10);
    let mut current = completed;
    // Exercise more than the old cap with actual encounter/evidence growth, not only team commands.
    for i in 0..1025 {
        let revision = current["revision"].as_u64().unwrap();
        let started = repo
            .execute(
                &identity,
                &command(
                    &format!("long-start-{i}"),
                    revision,
                    json!({"type":"start_encounter","beatId":"training-pip-1"}),
                ),
                2000 + i * 2,
            )
            .unwrap();
        let eid = &started["pendingEncounters"][0]["id"];
        current = repo
            .execute(
                &identity,
                &command(
                    &format!("long-abandon-{i}"),
                    revision + 1,
                    json!({"type":"abandon","encounterId":eid}),
                ),
                2001 + i * 2,
            )
            .unwrap();
    }
    // Saturating the new-command quota never prevents an exact old retry.
    admin.execute("UPDATE wonderwilds.progress SET rate_window_start=8000,rate_window_count=120 WHERE profile_id=$1::text::uuid", &[&profile]).unwrap();
    assert_eq!(
        repo.execute(&identity, &old_request, 8000).unwrap(),
        old_response
    );
    let after_quota = command(
        "after-quota",
        current["revision"].as_u64().unwrap(),
        json!({"type":"select_team","creatureIds":[]}),
    );
    assert_eq!(
        repo.execute(&identity, &after_quota, 8000),
        Err(Error::RateLimited)
    );
    current = repo.execute(&identity, &after_quota, 8060).unwrap();
    // Reopening the repository models process restart: no in-memory receipt cache is needed.
    let restarted = PostgresRepository::new(repo.connection.clone(), content.clone());
    let loaded = restarted.read(&identity).unwrap();
    assert_eq!(loaded.snapshot(&content).unwrap(), current);
    assert!(loaded.revision > 2000);
    assert_eq!(loaded.domain.character_xp("pip"), 10);
    let raw: Value = admin
        .query_one(
            "SELECT snapshot FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .unwrap()
        .get(0);
    assert!(
        raw.to_string().len() < 30_000,
        "hot checkpoint stays bounded: {} bytes",
        raw.to_string().len()
    );
    println!(
        "durable storage: {} commands, {} hot-save bytes, two recovery checkpoints",
        loaded.revision,
        raw.to_string().len()
    );
    let recovered: Aggregate =
        serde_json::from_value(restarted.reconstruct_legacy(&identity).unwrap()).unwrap();
    assert!(!recovered.is_storage_checkpoint());
    assert_eq!(recovered.snapshot(&content).unwrap(), current);
    let mut recovered = recovered;
    assert_eq!(
        recovered
            .execute(&old_request, &content, 9000, "recovery-session")
            .unwrap(),
        old_response
    );
    assert!(
        loaded.encounters.is_empty()
            && loaded.evidence.is_empty()
            && loaded.events.is_empty()
            && loaded.receipts.is_empty()
    );
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid",
            profile
        ),
        sequence(loaded.revision).unwrap()
    );
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.game_checkpoints WHERE profile_id=$1::text::uuid",
            profile
        ),
        2
    );
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT max(sequence) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid",
            profile
        ),
        sequence(loaded.evidence_sequence()).unwrap()
    );
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid",
            profile
        ),
        sequence(loaded.evidence_sequence()).unwrap()
    );
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid",
            profile
        ),
        sequence(loaded.domain.event_sequence()).unwrap()
    );
    assert_eq!(
        restarted.execute(&identity, &old_request, 5000).unwrap(),
        old_response
    );
    assert_eq!(
        restarted.execute(&identity, &completion, 5000).unwrap()["characterProgression"][0]["xp"],
        10
    );
    let mut changed = old_request.clone();
    changed["payload"]["creatureIds"] = json!([]);
    assert_eq!(
        restarted.execute(&identity, &changed, 5000),
        Err(Error::RevisionConflict)
    );
    // A boundary retry never adds rows/evidence/grants; foreign identity cannot read receipts.
    assert_eq!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id=$1::text::uuid AND reward_definition='character-xp:training-pip-1'",
            profile
        ),
        1
    );
    let foreign = Identity {
        parent_id: "61000000-0000-0000-0000-000000000002".into(),
        ..identity.clone()
    };
    assert!(matches!(repo.read(&foreign), Err(Error::Forbidden)));
    assert_eq!(
        repo.execute(&foreign, &old_request, 5000),
        Err(Error::Forbidden)
    );
    // New writes store original trusted replay facts; legacy rows honestly have no invented facts.
    assert!(
        scalar(
            &mut admin,
            "SELECT count(*) FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND replay_entry IS NOT NULL",
            profile
        ) > 2000
    );
    // Digest corruption and durable highwater mismatch fail closed, never reset.
    let digest: String = admin
        .query_one(
            "SELECT snapshot_digest FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .unwrap()
        .get(0);
    admin.execute("UPDATE wonderwilds.progress SET snapshot_digest=repeat('0',64) WHERE profile_id=$1::text::uuid",&[&profile]).unwrap();
    assert!(matches!(repo.read(&identity), Err(Error::Unavailable)));
    admin
        .execute(
            "UPDATE wonderwilds.progress SET snapshot_digest=$2 WHERE profile_id=$1::text::uuid",
            &[&profile, &digest],
        )
        .unwrap();
    admin.batch_execute(&format!("BEGIN; SELECT set_config('app.parent_id','{parent}',true); SELECT wonderwilds.export_profile('{profile}'); COMMIT;")).unwrap();
    // Check direct RLS, complete export and deletion cascade under the restricted runtime role.
    let mut runtime = Client::connect(&repo.connection, NoTls).unwrap();
    let mut tx = runtime.transaction().unwrap();
    tx.query_one("SELECT set_config('app.parent_id',$1,true)", &[&parent])
        .unwrap();
    let exported: Value = tx
        .query_one(
            "SELECT wonderwilds.export_profile($1::text::uuid)",
            &[&profile],
        )
        .unwrap()
        .get(0);
    assert!(exported["commandReceipts"].as_array().unwrap().len() > 2000);
    assert!(exported["encounters"].as_array().unwrap().len() > 1000);
    tx.query_one(
        "SELECT set_config('app.parent_id',$1,true)",
        &[&foreign.parent_id],
    )
    .unwrap();
    assert_eq!(
        tx.query_one(
            "SELECT count(*) FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid",
            &[&profile]
        )
        .unwrap()
        .get::<_, i64>(0),
        0
    );
    tx.query_one("SELECT set_config('app.parent_id',$1,true)", &[&parent])
        .unwrap();
    tx.query_one(
        "SELECT wonderwilds.delete_profile($1::text::uuid)",
        &[&profile],
    )
    .unwrap();
    tx.commit().unwrap();
    for table in [
        "progress",
        "command_receipts",
        "domain_command_receipts",
        "game_encounters",
        "game_checkpoints",
        "game_mastery_items",
        "learning_events",
        "game_events",
        "reward_grants",
    ] {
        assert_eq!(
            scalar(
                &mut admin,
                &format!(
                    "SELECT count(*) FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"
                ),
                profile
            ),
            0
        );
    }
    admin
        .execute(
            "DELETE FROM wonderwilds.deletion_tombstones WHERE profile_id=$1::text::uuid",
            &[&profile],
        )
        .unwrap();
    admin
        .execute(
            "DELETE FROM wonderwilds.parents WHERE id=$1::text::uuid",
            &[&parent],
        )
        .unwrap();
    admin
        .batch_execute(&format!("DROP OWNED BY {role}; DROP ROLE {role};"))
        .unwrap();
}
