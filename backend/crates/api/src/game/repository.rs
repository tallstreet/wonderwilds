//! Sync PostgreSQL adapter. HTTP always calls this boundary through spawn_blocking.
use super::*;
use postgres::{Client, NoTls};

pub trait Repository: Send + Sync {
    fn readiness(&self) -> Result<(), Error> {
        Err(Error::Unavailable)
    }
    fn read(&self, identity: &Identity) -> Result<Aggregate, Error>;
    fn execute(&self, identity: &Identity, request: &Value, now: u64) -> Result<Value, Error>;
}
#[derive(Clone)]
pub struct PostgresRepository {
    connection: String,
    content: Content,
    #[cfg(test)]
    pub fail_before_commit: bool,
}
impl PostgresRepository {
    pub fn new(connection: String, content: Content) -> Self {
        Self {
            connection,
            content,
            #[cfg(test)]
            fail_before_commit: false,
        }
    }
    /// Operator audit: full durable replay and independent mastery recomputation.
    /// No HTTP route exposes this maintenance operation.
    pub fn verify_history(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own(&mut tx, identity)?;
        let rebuilt = super::storage::rebuild(&mut tx, &identity.profile_id, &self.content, false)?;
        let current = super::storage::load(&mut tx, &identity.profile_id, &self.content)?;
        let snapshot = rebuilt.snapshot(&self.content)?;
        if super::storage::encode(&rebuilt, &self.content)?
            != super::storage::encode(&current, &self.content)?
        {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(snapshot)
    }
    /// Compatible recovery rebuilds the latest revision, never a stale backup.
    pub fn recover_current(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own(&mut tx, identity)?;
        let snapshot = super::storage::recover(&mut tx, &identity.profile_id, &self.content)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(snapshot)
    }
    fn connect(&self) -> Result<Client, Error> {
        Client::connect(&self.connection, NoTls).map_err(|_| Error::Unavailable)
    }
}
fn own(tx: &mut postgres::Transaction<'_>, identity: &Identity) -> Result<(), Error> {
    tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
        .map_err(|_| Error::Unavailable)?;
    tx.query_one(
        "SELECT set_config('app.parent_id',$1,true)",
        &[&identity.parent_id],
    )
    .map_err(|_| Error::Unavailable)?;
    if tx.query_opt("SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE", &[&identity.profile_id,&identity.parent_id]).map_err(|_|Error::Unavailable)?.is_none() { return Err(Error::Forbidden); }
    Ok(())
}
impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let ready: bool=client.query_one("SELECT to_regclass('wonderwilds.command_receipts') IS NOT NULL AND to_regclass('wonderwilds.game_checkpoints') IS NOT NULL AND to_regclass('wonderwilds.domain_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.game_encounters') IS NOT NULL AND to_regclass('wonderwilds.persistence_origins') IS NOT NULL AND to_regclass('wonderwilds.command_windows') IS NOT NULL",&[]).map_err(|_|Error::Unavailable)?.get(0);
        if ready {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
    fn read(&self, identity: &Identity) -> Result<Aggregate, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own(&mut tx, identity)?;
        let state = super::storage::load(&mut tx, &identity.profile_id, &self.content)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(state)
    }
    fn execute(&self, identity: &Identity, request: &Value, now: u64) -> Result<Value, Error> {
        wire::validate("GameCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        own(&mut tx, identity)?;
        let mut aggregate = super::storage::load(&mut tx, &identity.profile_id, &self.content)?;
        // Validate current checkpoint integrity, then lookup original receipts
        // before any submitted-revision check. Migration is committed on retries.
        if let Some(row)=tx.query_opt("SELECT request,response,replay,digest FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2", &[&identity.profile_id,&id]).map_err(|_|Error::Unavailable)? {
            if row.get::<_,Value>(0)!=*request {return Err(Error::RevisionConflict);}
            let response:Value=row.get(1);
            if row.get::<_,String>(3)!=crate::persistence::receipt(request,&response,&row.get(2)) {return Err(Error::Unavailable);}
            wire::validate("GameSnapshot",&response).map_err(|_|Error::Unavailable)?;
            tx.commit().map_err(|_|Error::Unavailable)?;
            return Ok(response);
        }
        crate::persistence::take_budget(&mut tx, &identity.profile_id, "game")?;
        let response = aggregate.execute(request, &self.content, now, &identity.session_id)?;
        super::storage::commit(
            &mut tx,
            identity,
            &aggregate,
            &self.content,
            request,
            &response,
            now,
        )?;
        #[cfg(test)]
        if self.fail_before_commit {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
}

/// Deterministic transaction reference used by HTTP tests, never selected by the
/// production binary. Clone/apply/swap models the same all-or-nothing boundary.
#[cfg(test)]
pub struct MemoryRepository {
    pub content: Content,
    pub profiles: std::sync::Mutex<BTreeMap<String, (String, Aggregate)>>,
}
#[cfg(test)]
impl Repository for MemoryRepository {
    fn read(&self, id: &Identity) -> Result<Aggregate, Error> {
        let profiles = self.profiles.lock().map_err(|_| Error::Unavailable)?;
        let (parent, state) = profiles.get(&id.profile_id).ok_or(Error::Forbidden)?;
        if parent != &id.parent_id {
            return Err(Error::Forbidden);
        }
        Ok(state.clone())
    }
    fn execute(&self, id: &Identity, request: &Value, now: u64) -> Result<Value, Error> {
        let mut profiles = self.profiles.lock().map_err(|_| Error::Unavailable)?;
        let (parent, state) = profiles.get_mut(&id.profile_id).ok_or(Error::Forbidden)?;
        if parent != &id.parent_id {
            return Err(Error::Forbidden);
        }
        let mut candidate = state.clone();
        let response = candidate.execute(request, &self.content, now, &id.session_id)?;
        *state = candidate;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command(id: &str, revision: u64, payload: Value) -> Value {
        json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":revision,"payload":payload})
    }
    #[test]
    #[ignore = "requires dedicated migrated UTF8 PostgreSQL database in WW_GAME_TEST_DATABASE_URL"]
    fn postgres_atomic_rollback_races_unique_grants_and_profile_ownership() {
        let connection = std::env::var("WW_GAME_TEST_DATABASE_URL")
            .expect("dedicated synthetic database required");
        let mut admin = Client::connect(&connection, NoTls).unwrap();
        // Fixtures use a restricted login over the private synthetic socket, never the
        // database owner. They are removed at the end of this test.
        admin.batch_execute("CREATE ROLE ww_game_test LOGIN NOSUPERUSER NOBYPASSRLS; GRANT USAGE ON SCHEMA wonderwilds TO ww_game_test; GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_game_test; GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_game_test;").unwrap();
        let parent = "51000000-0000-0000-0000-000000000001";
        let profile = "52000000-0000-0000-0000-000000000001";
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-game-test','parent-a')",&[&parent]).unwrap();
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[&profile,&parent]).unwrap();
        let mut config: postgres::Config = connection.parse().unwrap();
        config.user("ww_game_test");
        let host = config.get_hosts().first().unwrap();
        let host = match host {
            postgres::config::Host::Unix(path) => path.to_str().unwrap(),
            _ => panic!("private Unix socket required"),
        };
        let restricted = format!(
            "host={host} port={} user=ww_game_test dbname={}",
            config.get_ports()[0],
            config.get_dbname().unwrap()
        );
        let content = development_content().unwrap();
        let repo = PostgresRepository::new(restricted.clone(), content.clone());
        let identity = Identity {
            parent_id: parent.into(),
            profile_id: profile.into(),
            session_id: "session-synthetic".into(),
        };
        let start = command(
            "db-start",
            0,
            json!({"type":"start_encounter","beatId":"meadow-welcome"}),
        );
        let mut faulty = repo.clone();
        faulty.fail_before_commit = true;
        assert_eq!(
            faulty.execute(&identity, &start, 10),
            Err(Error::Unavailable)
        );
        assert_eq!(repo.read(&identity).unwrap().revision, 0);
        let count: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.game_events WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
        let snapshot = repo.execute(&identity, &start, 10).unwrap();
        let old_eid = snapshot["pendingEncounters"][0]["id"].as_str().unwrap();
        let abandon = command(
            "db-abandon",
            1,
            json!({"type":"abandon","encounterId":old_eid}),
        );
        assert_eq!(
            faulty.execute(&identity, &abandon, 10),
            Err(Error::Unavailable)
        );
        assert_eq!(
            repo.read(&identity)
                .unwrap()
                .public_encounter(old_eid)
                .unwrap()["id"],
            old_eid
        );
        let ended = repo.execute(&identity, &abandon, 10).unwrap();
        assert_eq!(ended["pendingEncounters"], json!([]));
        let restored = repo.read(&identity).unwrap();
        assert!(
            restored.evidence.is_empty(),
            "Hot state contains no historical evidence"
        );
        let evidence: Value=admin.query_one("SELECT evidence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND sequence=1", &[&profile]).unwrap().get(0);
        assert_eq!(evidence["eventKind"], "abandonment");
        assert_eq!(repo.execute(&identity, &abandon, 10).unwrap(), ended);
        assert!(restored.domain.completed().is_empty());
        let restart = command(
            "db-restart",
            2,
            json!({"type":"start_encounter","beatId":"meadow-welcome"}),
        );
        let snapshot = repo.execute(&identity, &restart, 10).unwrap();
        let eid = snapshot["pendingEncounters"][0]["id"].as_str().unwrap();
        assert_ne!(eid, old_eid);
        let answer = command(
            "db-answer",
            3,
            json!({"type":"attempt","encounterId":eid,"answer":{"kind":"choices","choiceIds":["message-0"]}}),
        );
        let r1 = repo.clone();
        let i1 = identity.clone();
        let a1 = answer.clone();
        let t1 = std::thread::spawn(move || r1.execute(&i1, &a1, 11));
        let r2 = repo.clone();
        let i2 = identity.clone();
        let a2 = answer.clone();
        let t2 = std::thread::spawn(move || r2.execute(&i2, &a2, 11));
        let result = t1.join().unwrap().unwrap();
        assert_eq!(t2.join().unwrap().unwrap(), result);
        let evidence: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.learning_events WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(evidence, 3);
        let events: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.game_events WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap()
            .get(0);
        let outbox: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.game_outbox WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(events, outbox);
        let grants: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert!(grants > 0);
        // Different bodies with the same ID and stale distinct commands have no effect.
        let mut changed = answer;
        changed["payload"]["answer"]["choiceIds"] = json!(["message-1"]);
        assert_eq!(
            repo.execute(&identity, &changed, 12),
            Err(Error::RevisionConflict)
        );
        let foreign = Identity {
            parent_id: "51000000-0000-0000-0000-000000000002".into(),
            ..identity.clone()
        };
        assert!(matches!(repo.read(&foreign), Err(Error::Forbidden)));
        // Exercise actual training persistence with a separate synthetic recruited
        // profile: failed commits publish neither XP, evidence nor reward grants.
        let training_profile = "52000000-0000-0000-0000-000000000003";
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')", &[&training_profile, &parent]).unwrap();
        let mut recruited = super::super::companion_tests::recruited_pip(&content);
        let old_start = command(
            "db-training-start",
            0,
            json!({"type":"start_encounter","beatId":"training-pip-1"}),
        );
        let old_response = recruited
            .execute(&old_start, &content, 18, "original-session")
            .unwrap();
        let old_hint = command(
            "old-hint",
            1,
            json!({"type":"hint","encounterId":old_response["pendingEncounters"][0]["id"],"supportKind":"hint"}),
        );
        let old_supported = recruited
            .execute(&old_hint, &content, 19, "original-session")
            .unwrap();
        for evidence in &recruited.evidence {
            admin.execute("INSERT INTO wonderwilds.learning_events(profile_id,sequence,evidence) VALUES($1::text::uuid,$2,$3)", &[&training_profile,&(evidence.server_sequence as i64),&serde_json::to_value(evidence).unwrap()]).unwrap();
        }
        for record in recruited
            .domain
            .archived_receipts()
            .as_object()
            .unwrap()
            .values()
        {
            for event in record["events"].as_array().unwrap() {
                admin.execute("INSERT INTO wonderwilds.game_events(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3)", &[&training_profile,&event["sequence"].as_i64().unwrap(),event]).unwrap();
            }
        }
        let seeded = serde_json::to_value(recruited).unwrap();
        admin
            .execute(
                "UPDATE wonderwilds.profiles SET revision=2 WHERE id=$1::text::uuid",
                &[&training_profile],
            )
            .unwrap();
        admin
            .execute(
                "INSERT INTO wonderwilds.progress(profile_id,snapshot) VALUES($1::text::uuid,$2)",
                &[&training_profile, &seeded],
            )
            .unwrap();
        let training_identity = Identity {
            profile_id: training_profile.into(),
            ..identity.clone()
        };
        let change_team = command(
            "returning-team",
            2,
            json!({"type":"select_team","creatureIds":[]}),
        );
        assert_eq!(
            faulty.execute(&training_identity, &change_team, 20),
            Err(Error::Unavailable)
        );
        assert_eq!(
            admin
                .query_one(
                    "SELECT snapshot FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",
                    &[&training_profile]
                )
                .unwrap()
                .get::<_, Value>(0),
            seeded,
            "Failed lazy migration leaves original save intact"
        );
        assert_eq!(
            repo.read(&training_identity)
                .unwrap()
                .snapshot(&content)
                .unwrap(),
            old_supported
        );
        assert_eq!(
            repo.execute(&training_identity, &old_start, 20).unwrap(),
            old_response
        );
        assert_eq!(
            repo.execute(&training_identity, &old_hint, 20).unwrap(),
            old_supported
        );
        let started = repo.execute(&training_identity, &change_team, 20).unwrap();
        assert_eq!(started["pendingEncounters"][0]["support"]["hintCount"], 1);
        assert_eq!(
            started["pendingEncounters"][0]["training"]["creatureId"],
            "pip"
        );
        let encounter = &started["pendingEncounters"][0];
        let answer = encounter["parameters"]["total"].as_u64().unwrap()
            - encounter["parameters"]["given"].as_u64().unwrap();
        let attempt = command(
            "db-training-answer",
            3,
            json!({"type":"attempt","encounterId":encounter["id"],"answer":{"kind":"quantity","value":answer}}),
        );
        assert_eq!(
            faulty.execute(&training_identity, &attempt, 21),
            Err(Error::Unavailable)
        );
        assert_eq!(
            repo.read(&training_identity)
                .unwrap()
                .domain
                .character_xp("pip"),
            0
        );
        let count: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id::text=$1",
                &[&training_profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
        let r1 = repo.clone();
        let i1 = training_identity.clone();
        let a1 = attempt.clone();
        let t1 = std::thread::spawn(move || r1.execute(&i1, &a1, 22));
        let r2 = repo.clone();
        let i2 = training_identity.clone();
        let a2 = attempt.clone();
        let t2 = std::thread::spawn(move || r2.execute(&i2, &a2, 22));
        let earned = t1.join().unwrap().unwrap();
        assert_eq!(t2.join().unwrap().unwrap(), earned);
        assert_eq!(earned["characterProgression"][0]["level"], 2);
        assert_eq!(
            repo.execute(&training_identity, &attempt, 23).unwrap(),
            earned
        );
        let count: i64 = admin.query_one("SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id::text=$1 AND reward_definition='character-xp:training-pip-1'", &[&training_profile]).unwrap().get(0);
        assert_eq!(count, 1);
        let count: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.learning_events WHERE profile_id::text=$1",
                &[&training_profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 3);
        // More than the old lifetime cap, using real persisted encounters and
        // evidence. Every command reconnects, and checkpoints retain no history.
        let initial_size: i32=admin.query_one("SELECT octet_length(snapshot::text) FROM wonderwilds.progress WHERE profile_id=$1::text::uuid", &[&training_profile]).unwrap().get(0);
        let timer = std::time::Instant::now();
        let mut latest = earned.clone();
        for n in 0..1050 {
            let start = command(
                &format!("long-start-{n}"),
                latest["revision"].as_u64().unwrap(),
                json!({"type":"start_encounter","beatId":"training-pip-1"}),
            );
            latest = repo
                .execute(&training_identity, &start, 100 + n * 2)
                .unwrap();
            let abandon = command(
                &format!("long-abandon-{n}"),
                latest["revision"].as_u64().unwrap(),
                json!({"type":"abandon","encounterId":latest["pendingEncounters"][0]["id"]}),
            );
            latest = repo
                .execute(&training_identity, &abandon, 101 + n * 2)
                .unwrap();
            if n == 0 || n == 1049 {
                assert_eq!(
                    repo.execute(&training_identity, &attempt, 2300).unwrap(),
                    earned
                );
                assert_eq!(
                    repo.read(&training_identity)
                        .unwrap()
                        .snapshot(&content)
                        .unwrap(),
                    latest
                );
            }
        }
        assert!(latest["revision"].as_u64().unwrap() > 2000);
        assert_eq!(
            latest["characterProgression"],
            earned["characterProgression"]
        );
        assert_eq!(latest["learnerSummaries"], earned["learnerSummaries"]);
        let size:i32=admin.query_one("SELECT octet_length(snapshot::text) FROM wonderwilds.progress WHERE profile_id=$1::text::uuid", &[&training_profile]).unwrap().get(0);
        assert!(
            size < initial_size + 256,
            "Hot snapshot grew with history: {initial_size} -> {size}"
        );
        assert_eq!(admin.query_one("SELECT count(*) FROM wonderwilds.game_checkpoints WHERE profile_id=$1::text::uuid", &[&training_profile]).unwrap().get::<_,i64>(0),2);
        eprintln!(
            "long-play adventure: 2104 commands; hot bytes {initial_size}->{size}; 2100 writes {:?}",
            timer.elapsed()
        );
        assert_eq!(repo.verify_history(&training_identity).unwrap(), latest);
        // The compatible recovery path replays to the current revision. Both
        // hot copies may be damaged; the origin and journals remain authoritative.
        admin
            .execute(
                "UPDATE wonderwilds.progress SET snapshot='{}' WHERE profile_id=$1::text::uuid",
                &[&training_profile],
            )
            .unwrap();
        admin.execute("UPDATE wonderwilds.game_checkpoints SET digest=repeat('0',64) WHERE profile_id=$1::text::uuid", &[&training_profile]).unwrap();
        assert!(matches!(
            repo.read(&training_identity),
            Err(Error::Unavailable)
        ));
        assert_eq!(repo.recover_current(&training_identity).unwrap(), latest);
        assert_eq!(
            repo.read(&training_identity)
                .unwrap()
                .snapshot(&content)
                .unwrap(),
            latest
        );
        assert_eq!(
            repo.execute(&training_identity, &attempt, 2400).unwrap(),
            earned
        );
        admin.execute("UPDATE wonderwilds.command_windows SET window_minute=floor(extract(epoch FROM clock_timestamp())/60)::bigint,commands=4096 WHERE profile_id=$1::text::uuid AND kind='game'", &[&training_profile]).unwrap();
        let limited = command(
            "limited",
            latest["revision"].as_u64().unwrap(),
            json!({"type":"start_encounter","beatId":"training-pip-1"}),
        );
        assert_eq!(
            repo.execute(&training_identity, &limited, 2401),
            Err(Error::RateLimited)
        );
        assert_eq!(
            repo.execute(&training_identity, &attempt, 2401).unwrap(),
            earned
        );
        admin
            .query_one("SELECT set_config('app.parent_id',$1,false)", &[&parent])
            .unwrap();
        let exported: Value = admin
            .query_one(
                "SELECT wonderwilds.export_profile($1::text::uuid)",
                &[&training_profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(
            exported["command_receipts"].as_array().unwrap().len(),
            latest["revision"].as_u64().unwrap() as usize
        );
        assert!(exported["game_encounters"].as_array().unwrap().len() > 1000);
        assert_eq!(exported["game_checkpoints"].as_array().unwrap().len(), 2);
        assert!(
            exported["learningReconstruction"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["reconstruction"]["novelTransfer"] == false)
        );
        let mut outsider = Client::connect(&restricted, NoTls).unwrap();
        outsider
            .query_one(
                "SELECT set_config('app.parent_id',$1,false)",
                &[&foreign.parent_id],
            )
            .unwrap();
        for table in [
            "command_receipts",
            "domain_command_receipts",
            "game_encounters",
            "game_checkpoints",
            "persistence_origins",
            "command_windows",
        ] {
            assert_eq!(outsider.query_one(&format!("SELECT count(*) FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"), &[&training_profile]).unwrap().get::<_,i64>(0),0);
            assert_eq!(
                outsider
                    .execute(
                        &format!("DELETE FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),
                        &[&training_profile]
                    )
                    .unwrap(),
                0
            );
        }
        // Lost history must not roll recovery back to an older revision.
        admin.execute("DELETE FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND revision=2000", &[&training_profile]).unwrap();
        assert_eq!(
            repo.recover_current(&training_identity),
            Err(Error::Unavailable)
        );
        admin
            .execute(
                "DELETE FROM wonderwilds.profiles WHERE id::text=$1",
                &[&training_profile],
            )
            .unwrap();
        // Deletion covers game effects through cascade; tombstone remains independently.
        admin.batch_execute(&format!("BEGIN; SELECT set_config('app.parent_id','{parent}',true); SELECT wonderwilds.delete_profile('{profile}'); COMMIT;")).unwrap();
        for table in [
            "learning_events",
            "game_events",
            "game_outbox",
            "reward_grants",
            "command_receipts",
            "domain_command_receipts",
            "game_encounters",
            "game_checkpoints",
            "persistence_origins",
            "command_windows",
        ] {
            let count: i64 = admin
                .query_one(
                    &format!("SELECT count(*) FROM wonderwilds.{table} WHERE profile_id::text=$1"),
                    &[&profile],
                )
                .unwrap()
                .get(0);
            assert_eq!(count, 0);
        }
        admin
            .execute(
                "DELETE FROM wonderwilds.deletion_tombstones WHERE profile_id::text=$1",
                &[&profile],
            )
            .unwrap();
        admin
            .execute(
                "DELETE FROM wonderwilds.parents WHERE id::text=$1",
                &[&parent],
            )
            .unwrap();
        admin
            .batch_execute("DROP OWNED BY ww_game_test; DROP ROLE ww_game_test;")
            .unwrap();
    }
}
