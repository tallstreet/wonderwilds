use super::*;
fn command(id: &str, revision: u64, payload: Value) -> Value {
    json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
}
fn perform(repo: &Repository, actor: &Identity, id: &str, payload: Value) -> Value {
    let revision = repo.run(actor, None).unwrap()["revision"].as_u64().unwrap();
    repo.run(actor, Some(&command(id, revision, payload)))
        .unwrap()
}
fn seed_campaign(admin: &mut Client, profile: &str, friend: &str, item: &str) {
    let def = wonderwilds_content::campaign().unwrap();
    let mut state = wonderwilds_domain::campaign::adopt(&def);
    state.starter = Some(friend.into());
    state.team = vec![friend.into()];
    state.flags.insert("starter_chosen".into());
    state.items.insert(item.into(), 1);
    let checkpoint = crate::persistence::envelope(
        "campaign-storage-1",
        json!({"contentFingerprint":crate::persistence::digest(&json!(def)),"campaign":state}),
    );
    let request = command(
        "fixture-earned",
        0,
        json!({"type":"action","action_id":"fixture-earned"}),
    );
    let response = json!({"campaign":{"state":state}});
    admin.execute("INSERT INTO wonderwilds.campaign_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,1,$2)",&[&profile,&checkpoint]).unwrap();
    admin.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,'fixture-earned',1,$2,$3,$4)",&[&profile,&request,&response,&crate::persistence::receipt(&request,&response,&json!({}))]).unwrap();
}
#[test]
#[ignore = "requires disposable WW_STORY_TEST_DATABASE_URL"]
fn postgres_paired_roles_attribution_copy_consent_retry_and_lifecycle() {
    let connection = std::env::var("WW_STORY_TEST_DATABASE_URL").unwrap();
    let mut admin = Client::connect(&connection, NoTls).unwrap();
    admin.batch_execute("CREATE ROLE ww_pair_test LOGIN NOSUPERUSER NOBYPASSRLS;GRANT USAGE ON SCHEMA wonderwilds TO ww_pair_test;GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_pair_test;GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_pair_test;").unwrap();
    let parent = "91000000-0000-0000-0000-000000000001";
    let foreign_parent = "91000000-0000-0000-0000-000000000002";
    let a = "92000000-0000-0000-0000-000000000001";
    let b = "92000000-0000-0000-0000-000000000002";
    let foreign = "92000000-0000-0000-0000-000000000003";
    for (p, subject) in [(parent, "pair-a"), (foreign_parent, "pair-b")] {
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-pair',$2)",&[&p,&subject]).unwrap();
    }
    for (profile, p) in [(a, parent), (b, parent), (foreign, foreign_parent)] {
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[&profile,&p]).unwrap();
    }
    let config: postgres::Config = connection.parse().unwrap();
    let host = match &config.get_hosts()[0] {
        postgres::config::Host::Unix(path) => path.to_str().unwrap(),
        _ => panic!(),
    };
    let restricted = format!(
        "host={host} port={} user=ww_pair_test dbname={}",
        config.get_ports()[0],
        config.get_dbname().unwrap()
    );
    let repo = Repository::new(Some(restricted.clone()));
    let actor = |profile: &str| Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "synthetic-pair-session".into(),
    };
    let ia = actor(a);
    let ib = actor(b);
    let _app = crate::game::http::AppState::development(
        Some(restricted.clone()),
        "https://wonderwilds.local".into(),
    )
    .unwrap();
    assert_eq!(
        repo.profiles(&ia).unwrap()["profiles"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        repo.run(
            &ia,
            Some(&command(
                "foreign",
                0,
                json!({"type":"start","partnerProfileId":foreign})
            ))
        ),
        Err(Error::Forbidden)
    );
    for (identity, friend, item) in [
        (&ia, "pip", "pip_keepsake"),
        (&ib, "tansy", "tansy_keepsake"),
    ] {
        let mut c = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = c.transaction().unwrap();
        context(&mut tx, identity).unwrap();
        tx.query_one(
            "SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid FOR UPDATE",
            &[&identity.profile_id],
        )
        .unwrap();
        crate::game::campaign_grant(
            &mut tx,
            identity,
            friend,
            &format!("campaign:recruit:{friend}"),
            0,
        )
        .unwrap();
        tx.commit().unwrap();
        seed_campaign(&mut admin, &identity.profile_id, friend, item);
    }
    let start = command("start", 0, json!({"type":"start","partnerProfileId":b}));
    let initial = repo.run(&ia, Some(&start)).unwrap();
    assert_eq!(initial["navigatorProfileId"], a);
    assert_eq!(repo.run(&ib, Some(&start)), Err(Error::RevisionConflict));
    assert_eq!(
        repo.run(
            &ia,
            Some(&command(
                "wrong-friend",
                1,
                json!({"type":"select_companion","companionId":"tansy"})
            ))
        ),
        Err(Error::Forbidden)
    );
    perform(
        &repo,
        &ia,
        "a-friend",
        json!({"type":"select_companion","companionId":"pip"}),
    );
    perform(
        &repo,
        &ib,
        "b-friend",
        json!({"type":"select_companion","companionId":"tansy"}),
    );
    let rev = repo.run(&ib, None).unwrap()["revision"].as_u64().unwrap();
    assert_eq!(
        repo.run(
            &ib,
            Some(&command(
                "keeper-start",
                rev,
                json!({"type":"start_practice"})
            ))
        ),
        Err(Error::Forbidden)
    );
    perform(&repo, &ia, "practice", json!({"type":"start_practice"}));
    let rev = repo.run(&ib, None).unwrap()["revision"].as_u64().unwrap();
    assert_eq!(
        repo.run(
            &ib,
            Some(&command(
                "wrong-turn",
                rev,
                json!({"type":"practice_turn","movement":"brace"})
            ))
        ),
        Err(Error::Forbidden)
    );
    for (index, (who, movement)) in [
        (&ia, "scout"),
        (&ib, "brace"),
        (&ia, "scout"),
        (&ib, "brace"),
    ]
    .iter()
    .enumerate()
    {
        perform(
            &repo,
            who,
            &format!("turn-{index}"),
            json!({"type":"practice_turn","movement":movement}),
        );
    }
    let won = repo.run(&ia, None).unwrap();
    assert_eq!(won["practice"]["outcome"], "won");
    assert_eq!(won["navigatorProfileId"], b);
    assert_eq!(won["completedPractices"], 1);
    assert_eq!(won["practice"]["history"][0]["profileId"], a);
    assert_eq!(won["practice"]["history"][1]["profileId"], b);
    perform(
        &repo,
        &ia,
        "offer-a",
        json!({"type":"offer_keepsake","keepsakeId":"pip_keepsake"}),
    );
    perform(
        &repo,
        &ib,
        "offer-b",
        json!({"type":"offer_keepsake","keepsakeId":"tansy_keepsake"}),
    );
    perform(&repo, &ia, "confirm-a", json!({"type":"confirm_exchange"}));
    assert_eq!(
        repo.run(&ia, None).unwrap()["keepsakes"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let last = command(
        "confirm-b",
        repo.run(&ib, None).unwrap()["revision"].as_u64().unwrap(),
        json!({"type":"confirm_exchange"}),
    );
    let mut faulty = repo.clone();
    faulty.fail_before_commit = true;
    assert_eq!(faulty.run(&ib, Some(&last)), Err(Error::Unavailable));
    assert_eq!(
        repo.run(&ia, None).unwrap()["keepsakes"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let threads = (0..2)
        .map(|_| {
            let r = repo.clone();
            let i = ib.clone();
            let c = last.clone();
            std::thread::spawn(move || r.run(&i, Some(&c)))
        })
        .collect::<Vec<_>>();
    for thread in threads {
        assert_eq!(thread.join().unwrap().unwrap()["exchangeComplete"], true)
    }
    assert_eq!(
        repo.run(&ia, None).unwrap()["keepsakes"][0]["keepsakeId"],
        "tansy_keepsake"
    );
    assert_eq!(
        repo.run(&ib, None).unwrap()["keepsakes"][0]["keepsakeId"],
        "pip_keepsake"
    );
    assert_eq!(repo.run(&ia, Some(&start)).unwrap(), initial);
    for id in [&ia, &ib] {
        let mut c = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = c.transaction().unwrap();
        context(&mut tx, id).unwrap();
        let owned = crate::game::campaign_companions(&mut tx, id).unwrap();
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].level, 1);
        assert_eq!(
            crate::campaign::repository::owned_keepsakes(&mut tx, id)
                .unwrap()
                .len(),
            1
        );
    }
    assert_eq!(admin.query_one("SELECT count(*) FROM wonderwilds.mission_learning_events WHERE profile_id=$1::text::uuid OR profile_id=$2::text::uuid",&[&a,&b]).unwrap().get::<_,i64>(0),0);
    let solo = perform(&repo, &ia, "solo", json!({"type":"continue_solo"}));
    assert_eq!(solo["mode"], "solo");
    assert_eq!(solo["keepsakes"].as_array().unwrap().len(), 1);
    admin
        .execute("SELECT set_config('app.parent_id',$1,false)", &[&parent])
        .unwrap();
    let exported: Value = admin
        .query_one("SELECT wonderwilds.export_profile($1::text::uuid)", &[&b])
        .unwrap()
        .get(0);
    assert_eq!(
        exported["sharedKeepsakeCopies"].as_array().unwrap().len(),
        1
    );
    let mut restricted_client = Client::connect(&restricted, NoTls).unwrap();
    restricted_client
        .execute(
            "SELECT set_config('app.parent_id',$1,false)",
            &[&foreign_parent],
        )
        .unwrap();
    assert_eq!(
        restricted_client
            .query_one("SELECT count(*) FROM wonderwilds.paired_progress", &[])
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    restricted_client
        .execute("SELECT set_config('app.parent_id',$1,false)", &[&parent])
        .unwrap();
    restricted_client
        .query_one("SELECT wonderwilds.delete_profile($1::text::uuid)", &[&a])
        .unwrap();
    let remaining = repo.run(&ib, None).unwrap();
    assert_eq!(remaining["mode"], "solo");
    assert_eq!(remaining["keepsakes"].as_array().unwrap().len(), 1);
    assert!(remaining["keepsakes"][0].get("sourceProfileId").is_none());
    assert_eq!(
        remaining["availableCompanions"].as_array().unwrap().len(),
        1
    );
    restricted_client
        .query_one("SELECT wonderwilds.delete_profile($1::text::uuid)", &[&b])
        .unwrap();
    assert_eq!(
        admin
            .query_one("SELECT count(*) FROM wonderwilds.paired_keepsakes", &[])
            .unwrap()
            .get::<_, i64>(0),
        0
    );
}

#[test]
#[ignore = "requires disposable WW_STORY_TEST_DATABASE_URL"]
fn postgres_paired_clock_survives_solo_and_active_participant_deletion() {
    let connection = std::env::var("WW_STORY_TEST_DATABASE_URL").unwrap();
    let mut admin = Client::connect(&connection, NoTls).unwrap();
    admin.batch_execute("CREATE ROLE ww_pair_clock_test LOGIN NOSUPERUSER NOBYPASSRLS;GRANT USAGE ON SCHEMA wonderwilds TO ww_pair_clock_test;GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_pair_clock_test;GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_pair_clock_test;").unwrap();
    let parent = "95000000-0000-0000-0000-000000000001";
    admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-pair-clock','clock')",&[&parent]).unwrap();
    let profiles = [
        "96000000-0000-0000-0000-000000000001",
        "96000000-0000-0000-0000-000000000002",
        "96000000-0000-0000-0000-000000000003",
        "96000000-0000-0000-0000-000000000004",
        "96000000-0000-0000-0000-000000000005",
        "96000000-0000-0000-0000-000000000006",
    ];
    for id in &profiles {
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[id,&parent]).unwrap();
    }
    let config: postgres::Config = connection.parse().unwrap();
    let host = match &config.get_hosts()[0] {
        postgres::config::Host::Unix(path) => path.to_str().unwrap(),
        _ => panic!(),
    };
    let restricted = format!(
        "host={host} port={} user=ww_pair_clock_test dbname={}",
        config.get_ports()[0],
        config.get_dbname().unwrap()
    );
    let repo = Repository::new(Some(restricted.clone()));
    let actor = |profile: &str| Identity {
        parent_id: parent.into(),
        profile_id: profile.into(),
        session_id: "synthetic-clock".into(),
    };
    let a = actor(profiles[0]);
    let mut runtime = Client::connect(&restricted, NoTls).unwrap();
    runtime
        .execute("SELECT set_config('app.parent_id',$1,false)", &[&parent])
        .unwrap();
    perform(
        &repo,
        &a,
        "clock-ab",
        json!({"type":"start","partnerProfileId":profiles[1]}),
    );
    let solo = perform(&repo, &a, "clock-solo-ab", json!({"type":"continue_solo"}));
    let solo_revision = solo["revision"].as_u64().unwrap();
    runtime
        .query_one(
            "SELECT wonderwilds.delete_profile($1::text::uuid)",
            &[&profiles[1]],
        )
        .unwrap();
    let recovered = repo.run(&a, None).unwrap();
    assert_eq!(recovered["mode"], "solo");
    assert_eq!(recovered["revision"], solo_revision);
    let ac = perform(
        &repo,
        &a,
        "clock-ac",
        json!({"type":"start","partnerProfileId":profiles[2]}),
    );
    assert_eq!(ac["revision"], solo_revision + 1);
    perform(&repo, &a, "clock-solo-ac", json!({"type":"continue_solo"}));
    let ad = perform(
        &repo,
        &a,
        "clock-ad",
        json!({"type":"start","partnerProfileId":profiles[3]}),
    );
    let active_revision = ad["revision"].as_u64().unwrap();
    runtime
        .query_one(
            "SELECT wonderwilds.delete_profile($1::text::uuid)",
            &[&profiles[3]],
        )
        .unwrap();
    let after = repo.run(&a, None).unwrap();
    assert_eq!(after["mode"], "solo");
    assert_eq!(after["revision"], active_revision);
    let resumed = perform(
        &repo,
        &a,
        "clock-ac-again",
        json!({"type":"start","partnerProfileId":profiles[2]}),
    );
    assert_eq!(resumed["revision"], active_revision + 1);
    assert_eq!(
        repo.run(
            &a,
            Some(&command(
                "stale-pre-delete",
                solo_revision,
                json!({"type":"swap_roles"})
            ))
        ),
        Err(Error::RevisionConflict)
    );
    let final_revision = resumed["revision"].as_u64().unwrap();
    for id in [profiles[0], profiles[2]] {
        runtime
            .query_one("SELECT wonderwilds.delete_profile($1::text::uuid)", &[&id])
            .unwrap();
    }
    assert_eq!(runtime.query_one("SELECT revision FROM wonderwilds.paired_household_clock WHERE parent_id=$1::text::uuid",&[&parent]).unwrap().get::<_,i64>(0) as u64,final_revision);
    let e = actor(profiles[4]);
    let fresh = repo.run(&e, None).unwrap();
    assert_eq!(fresh["mode"], "solo");
    assert_eq!(fresh["revision"], final_revision);
    let new_pair = perform(
        &repo,
        &e,
        "clock-new-ef",
        json!({"type":"start","partnerProfileId":profiles[5]}),
    );
    assert_eq!(new_pair["revision"], final_revision + 1);
}
