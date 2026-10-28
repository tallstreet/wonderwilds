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
    fn connect(&self) -> Result<Client, Error> {
        Client::connect(&self.connection, NoTls).map_err(|_| Error::Unavailable)
    }
}
impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let row=client.query_one("SELECT to_regclass('wonderwilds.progress') IS NOT NULL AND to_regclass('wonderwilds.game_events') IS NOT NULL AND to_regclass('wonderwilds.game_outbox') IS NOT NULL AND to_regclass('wonderwilds.reward_grants') IS NOT NULL AND to_regclass('wonderwilds.sessions') IS NOT NULL",&[]).map_err(|_|Error::Unavailable)?;
        if row.get::<_, bool>(0) {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
    fn read(&self, identity: &Identity) -> Result<Aggregate, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
            .map_err(|_| Error::Unavailable)?;
        tx.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&identity.parent_id],
        )
        .map_err(|_| Error::Unavailable)?;
        let row=tx.query_opt("SELECT p.id FROM wonderwilds.profiles p WHERE p.id::text=$1 AND p.parent_id::text=$2",&[&identity.profile_id,&identity.parent_id]).map_err(|_|Error::Unavailable)?;
        if row.is_none() {
            return Err(Error::Forbidden);
        }
        let saved = tx
            .query_opt(
                "SELECT snapshot FROM wonderwilds.progress WHERE profile_id::text=$1",
                &[&identity.profile_id],
            )
            .map_err(|_| Error::Unavailable)?;
        let aggregate = match saved {
            Some(row) => serde_json::from_value(row.get(0)).map_err(|_| Error::Unavailable)?,
            None => Aggregate::new(&self.content),
        };
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(aggregate)
    }
    fn execute(&self, identity: &Identity, request: &Value, now: u64) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
            .map_err(|_| Error::Unavailable)?;
        tx.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&identity.parent_id],
        )
        .map_err(|_| Error::Unavailable)?;
        // All writers and deletion lock this same owner-scoped row. Never load
        // state, validate a revision, then save outside this transaction.
        let row=tx.query_opt("SELECT p.id FROM wonderwilds.profiles p WHERE p.id::text=$1 AND p.parent_id::text=$2 FOR UPDATE",&[&identity.profile_id,&identity.parent_id]).map_err(|_|Error::Unavailable)?;
        if row.is_none() {
            return Err(Error::Forbidden);
        }
        let saved = tx
            .query_opt(
                "SELECT snapshot FROM wonderwilds.progress WHERE profile_id::text=$1",
                &[&identity.profile_id],
            )
            .map_err(|_| Error::Unavailable)?;
        let mut aggregate = match saved {
            Some(row) => serde_json::from_value(row.get(0)).map_err(|_| Error::Unavailable)?,
            None => Aggregate::new(&self.content),
        };
        let old_evidence = aggregate.evidence.len();
        let old_events = aggregate.events.len();
        let response = aggregate.execute(request, &self.content, now, &identity.session_id)?;
        let value = serde_json::to_value(&aggregate).map_err(|_| Error::Unavailable)?;
        tx.execute("INSERT INTO wonderwilds.progress(profile_id,snapshot) VALUES($1::text::uuid,$2) ON CONFLICT(profile_id) DO UPDATE SET snapshot=EXCLUDED.snapshot",&[&identity.profile_id,&value]).map_err(|_|Error::Unavailable)?;
        let revision = i64::try_from(aggregate.revision).map_err(|_| Error::Unavailable)?;
        tx.execute(
            "UPDATE wonderwilds.profiles SET revision=$1 WHERE id::text=$2 AND parent_id::text=$3",
            &[&revision, &identity.profile_id, &identity.parent_id],
        )
        .map_err(|_| Error::Unavailable)?;
        for evidence in &aggregate.evidence[old_evidence..] {
            let sequence =
                i64::try_from(evidence.server_sequence).map_err(|_| Error::Unavailable)?;
            let value = serde_json::to_value(evidence).map_err(|_| Error::Unavailable)?;
            tx.execute("INSERT INTO wonderwilds.learning_events(profile_id,sequence,evidence) VALUES($1::text::uuid,$2,$3)",&[&identity.profile_id,&sequence,&value]).map_err(|_|Error::Unavailable)?;
        }
        for event in &aggregate.events[old_events..] {
            let sequence = i64::try_from(event.sequence).map_err(|_| Error::Unavailable)?;
            let value = serde_json::to_value(event).map_err(|_| Error::Unavailable)?;
            tx.execute("INSERT INTO wonderwilds.game_events(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3)",&[&identity.profile_id,&sequence,&value]).map_err(|_|Error::Unavailable)?;
            tx.execute("INSERT INTO wonderwilds.game_outbox(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3)",&[&identity.profile_id,&sequence,&value]).map_err(|_|Error::Unavailable)?;
            if let engine::Outcome::RewardGranted {
                reward_id,
                quest_id,
            } = &event.outcome
            {
                tx.execute("INSERT INTO wonderwilds.reward_grants(profile_id,reward_definition,quest_instance) VALUES($1::text::uuid,$2,$3)",&[&identity.profile_id,&reward_id,&quest_id]).map_err(|_|Error::Unavailable)?;
            }
        }
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
        let repo = PostgresRepository::new(restricted, content.clone());
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
        assert_eq!(restored.evidence.len(), 1);
        assert_eq!(
            restored.evidence[0].event_kind,
            wire::LearningEvidenceEventKind::Abandonment
        );
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
        // Deletion covers game effects through cascade; tombstone remains independently.
        admin.batch_execute(&format!("BEGIN; SELECT set_config('app.parent_id','{parent}',true); SELECT wonderwilds.delete_profile('{profile}'); COMMIT;")).unwrap();
        for table in [
            "learning_events",
            "game_events",
            "game_outbox",
            "reward_grants",
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
