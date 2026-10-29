//! PostgreSQL storage: bounded working checkpoints and durable retry/history records.
use super::*;
use postgres::{Client, NoTls, Transaction};
use sha2::{Digest, Sha256};

const CHECKPOINT_INTERVAL: u64 = 256;
const COMMANDS_PER_MINUTE: i32 = 120;

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
fn snapshot_digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
fn sequence(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Unavailable)
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
    fn lock_profile(tx: &mut Transaction<'_>, identity: &Identity) -> Result<(), Error> {
        tx.batch_execute("SET LOCAL statement_timeout='2000ms'; SET LOCAL lock_timeout='1000ms';")
            .map_err(|_| Error::Unavailable)?;
        tx.query_one(
            "SELECT set_config('app.parent_id',$1,true)",
            &[&identity.parent_id],
        )
        .map_err(|_| Error::Unavailable)?;
        if tx.query_opt("SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE", &[&identity.profile_id,&identity.parent_id])
            .map_err(|_| Error::Unavailable)?.is_none() { return Err(Error::Forbidden); }
        Ok(())
    }
    fn save_checkpoint(
        tx: &mut Transaction<'_>,
        profile: &str,
        aggregate: &Aggregate,
        force: bool,
    ) -> Result<(), Error> {
        let value = serde_json::to_value(aggregate).map_err(|_| Error::Unavailable)?;
        let digest = snapshot_digest(&value);
        tx.execute("INSERT INTO wonderwilds.progress(profile_id,snapshot,snapshot_digest) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id) DO UPDATE SET snapshot=EXCLUDED.snapshot,snapshot_digest=EXCLUDED.snapshot_digest", &[&profile,&value,&digest]).map_err(|_|Error::Unavailable)?;
        if force || aggregate.revision.is_multiple_of(CHECKPOINT_INTERVAL) {
            let revision = sequence(aggregate.revision)?;
            tx.execute("INSERT INTO wonderwilds.game_checkpoints(profile_id,revision,snapshot,snapshot_digest) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT(profile_id,revision) DO UPDATE SET snapshot=EXCLUDED.snapshot,snapshot_digest=EXCLUDED.snapshot_digest", &[&profile,&revision,&value,&digest]).map_err(|_|Error::Unavailable)?;
            // Only recovery checkpoints rotate. Retry, evidence and event records never do.
            tx.execute("DELETE FROM wonderwilds.game_checkpoints WHERE profile_id=$1::text::uuid AND revision NOT IN (SELECT revision FROM wonderwilds.game_checkpoints WHERE profile_id=$1::text::uuid ORDER BY revision DESC LIMIT 2)", &[&profile]).map_err(|_|Error::Unavailable)?;
        }
        Ok(())
    }
    fn store_domain_receipts(
        tx: &mut Transaction<'_>,
        profile: &str,
        receipts: &[engine::DomainReceipt],
    ) -> Result<(), Error> {
        for receipt in receipts {
            let value = serde_json::to_value(receipt).map_err(|_| Error::Unavailable)?;
            let id = &receipt.command.command_id;
            let inserted=tx.execute("INSERT INTO wonderwilds.domain_command_receipts(profile_id,command_id,receipt) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,id,&value]).map_err(|_|Error::Unavailable)?;
            if inserted == 0 {
                let old:Value=tx.query_one("SELECT receipt FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",&[&profile,id]).map_err(|_|Error::Unavailable)?.get(0);
                if old != value {
                    return Err(Error::Unavailable);
                }
            }
        }
        Ok(())
    }
    fn store_encounters(
        tx: &mut Transaction<'_>,
        profile: &str,
        encounters: &[SavedEncounter],
        migrating: bool,
    ) -> Result<(), Error> {
        for encounter in encounters {
            let value = serde_json::to_value(encounter).map_err(|_| Error::Unavailable)?;
            if migrating {
                let inserted=tx.execute("INSERT INTO wonderwilds.game_encounters(profile_id,encounter_id,encounter) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,&encounter.id,&value]).map_err(|_|Error::Unavailable)?;
                if inserted == 0 {
                    let old:Value=tx.query_one("SELECT encounter FROM wonderwilds.game_encounters WHERE profile_id=$1::text::uuid AND encounter_id=$2",&[&profile,&encounter.id]).map_err(|_|Error::Unavailable)?.get(0);
                    if old != value {
                        return Err(Error::Unavailable);
                    }
                }
            } else {
                tx.execute("INSERT INTO wonderwilds.game_encounters(profile_id,encounter_id,encounter) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id,encounter_id) DO UPDATE SET encounter=EXCLUDED.encounter",&[&profile,&encounter.id,&value]).map_err(|_|Error::Unavailable)?;
            }
        }
        Ok(())
    }
    fn store_effects(
        tx: &mut Transaction<'_>,
        profile: &str,
        evidence: &[wire::LearningEvidence],
        events: &[engine::Event],
        migrating: bool,
    ) -> Result<(), Error> {
        for e in evidence {
            let n = sequence(e.server_sequence)?;
            let value = serde_json::to_value(e).map_err(|_| Error::Unavailable)?;
            let inserted=tx.execute("INSERT INTO wonderwilds.learning_events(profile_id,sequence,evidence) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,&n,&value]).map_err(|_|Error::Unavailable)?;
            if inserted == 0 {
                if !migrating {
                    return Err(Error::Unavailable);
                }
                let old:Value=tx.query_one("SELECT evidence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND sequence=$2",&[&profile,&n]).map_err(|_|Error::Unavailable)?.get(0);
                if old != value {
                    return Err(Error::Unavailable);
                }
            }
        }
        for e in events {
            let n = sequence(e.sequence)?;
            let value = serde_json::to_value(e).map_err(|_| Error::Unavailable)?;
            let inserted=tx.execute("INSERT INTO wonderwilds.game_events(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,&n,&value]).map_err(|_|Error::Unavailable)?;
            if inserted == 0 {
                if !migrating {
                    return Err(Error::Unavailable);
                }
                let old:Value=tx.query_one("SELECT event FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid AND sequence=$2",&[&profile,&n]).map_err(|_|Error::Unavailable)?.get(0);
                if old != value {
                    return Err(Error::Unavailable);
                }
            }
            let outbox=tx.execute("INSERT INTO wonderwilds.game_outbox(profile_id,sequence,event) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,&n,&value]).map_err(|_|Error::Unavailable)?;
            if outbox == 0 && !migrating {
                return Err(Error::Unavailable);
            }
            if outbox == 0 && migrating {
                let old:Value=tx.query_one("SELECT event FROM wonderwilds.game_outbox WHERE profile_id=$1::text::uuid AND sequence=$2",&[&profile,&n]).map_err(|_|Error::Unavailable)?.get(0);
                if old != value {
                    return Err(Error::Unavailable);
                }
            }
            if let engine::Outcome::RewardGranted {
                reward_id,
                quest_id,
            } = &e.outcome
            {
                let granted=tx.execute("INSERT INTO wonderwilds.reward_grants(profile_id,reward_definition,quest_instance) VALUES($1::text::uuid,$2,$3) ON CONFLICT DO NOTHING",&[&profile,&reward_id,&quest_id]).map_err(|_|Error::Unavailable)?;
                if granted == 0 && !migrating {
                    return Err(Error::Unavailable);
                }
            }
        }
        Ok(())
    }
    fn load(
        tx: &mut Transaction<'_>,
        identity: &Identity,
        content: &Content,
    ) -> Result<Aggregate, Error> {
        let saved=tx.query_opt("SELECT snapshot,snapshot_digest FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?;
        let (aggregate, digest) = match saved {
            Some(row) => {
                let value: Value = row.get(0);
                let digest: Option<String> = row.get(1);
                if digest
                    .as_ref()
                    .is_some_and(|d| *d != snapshot_digest(&value))
                {
                    return Err(Error::Unavailable);
                }
                (
                    serde_json::from_value::<Aggregate>(value).map_err(|_| Error::Unavailable)?,
                    digest,
                )
            }
            None => (Aggregate::new(content), None),
        };
        let profile_revision: i64 = tx
            .query_one(
                "SELECT revision FROM wonderwilds.profiles WHERE id=$1::text::uuid",
                &[&identity.profile_id],
            )
            .map_err(|_| Error::Unavailable)?
            .get(0);
        // This also protects legacy migration and missing-save initialization.
        // A missing save for an advanced profile must never appear reset.
        if profile_revision != sequence(aggregate.revision)? {
            return Err(Error::Unavailable);
        }
        if aggregate.is_storage_checkpoint() {
            if digest.is_none() {
                return Err(Error::Unavailable);
            }
            aggregate.validate_storage(content)?;
            let event_highwater:i64=tx.query_one("SELECT coalesce((SELECT sequence FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid ORDER BY sequence DESC LIMIT 1),0)",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?.get(0);
            let evidence_highwater:i64=tx.query_one("SELECT coalesce((SELECT sequence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence DESC LIMIT 1),0)",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?.get(0);
            if event_highwater != sequence(aggregate.domain.event_sequence())?
                || evidence_highwater != sequence(aggregate.evidence_sequence())?
            {
                return Err(Error::Unavailable);
            }
            // Pending support/recipient state must match its durable encounter record.
            for encounter in aggregate.encounters.values() {
                let row=tx.query_opt("SELECT encounter FROM wonderwilds.game_encounters WHERE profile_id=$1::text::uuid AND encounter_id=$2",&[&identity.profile_id,&encounter.id]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
                let stored: Value = row.get(0);
                if stored != serde_json::to_value(encounter).map_err(|_| Error::Unavailable)? {
                    return Err(Error::Unavailable);
                }
            }
            return Ok(aggregate);
        }
        if digest.is_some() {
            return Err(Error::Unavailable);
        }
        let migration = aggregate.into_storage(content)?;
        Self::validate_migration_records(tx, &identity.profile_id, &migration)?;
        // Reject extra/orphan history before backfilling legitimate legacy copies.
        for (table, field, expected) in [
            (
                "learning_events",
                "evidence",
                &migration
                    .evidence
                    .iter()
                    .map(|e| serde_json::to_value(e).unwrap())
                    .collect::<Vec<_>>(),
            ),
            (
                "game_events",
                "event",
                &migration
                    .events
                    .iter()
                    .map(|e| serde_json::to_value(e).unwrap())
                    .collect::<Vec<_>>(),
            ),
            (
                "game_outbox",
                "event",
                &migration
                    .events
                    .iter()
                    .map(|e| serde_json::to_value(e).unwrap())
                    .collect::<Vec<_>>(),
            ),
        ] {
            for row in tx.query(&format!("SELECT sequence,{field} FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid ORDER BY sequence"),&[&identity.profile_id]).map_err(|_|Error::Unavailable)?{
                let n:i64=row.get(0);let value:Value=row.get(1);
                if n<=0 || expected.get(n as usize-1)!=Some(&value){return Err(Error::Unavailable)}
            }
        }
        Self::store_effects(
            tx,
            &identity.profile_id,
            &migration.evidence,
            &migration.events,
            true,
        )?;
        Self::store_domain_receipts(tx, &identity.profile_id, &migration.domain_receipts)?;
        Self::store_encounters(tx, &identity.profile_id, &migration.encounters, true)?;
        for (id, request, response) in &migration.api_receipts {
            let inserted=tx.execute("INSERT INTO wonderwilds.command_receipts(profile_id,command_id,request,response) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT DO NOTHING",&[&identity.profile_id,id,request,response]).map_err(|_|Error::Unavailable)?;
            if inserted == 0 {
                let row=tx.query_one("SELECT request,response FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",&[&identity.profile_id,id]).map_err(|_|Error::Unavailable)?;
                if row.get::<_, Value>(0) != *request || row.get::<_, Value>(1) != *response {
                    return Err(Error::Unavailable);
                }
            }
        }
        for item in &migration.mastery_items {
            Self::novel_item(tx, &identity.profile_id, item)?;
        }
        Self::save_checkpoint(tx, &identity.profile_id, &migration.aggregate, true)?;
        Ok(migration.aggregate)
    }
    fn novel_item(
        tx: &mut Transaction<'_>,
        profile: &str,
        item: &MasteryItem,
    ) -> Result<bool, Error> {
        tx.execute("INSERT INTO wonderwilds.game_mastery_items(profile_id,skill_id,item_family,parameter_fingerprint) VALUES($1::text::uuid,$2,$3,$4) ON CONFLICT DO NOTHING",&[&profile,&item.skill_id,&item.item_family,&item.parameter_fingerprint]).map(|n|n==1).map_err(|_|Error::Unavailable)
    }

    fn validate_migration_records(
        tx: &mut Transaction<'_>,
        profile: &str,
        migration: &storage::StorageMigration,
    ) -> Result<(), Error> {
        let mut expected = BTreeMap::new();
        for (id, request, response) in &migration.api_receipts {
            expected.insert(id.as_str(), (request, response));
        }
        for row in tx.query("SELECT command_id,request,response FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_| Error::Unavailable)? {
            let id: String = row.get(0);
            let request: Value = row.get(1);
            let response: Value = row.get(2);
            if expected.get(id.as_str()) != Some(&(&request, &response)) {
                return Err(Error::Unavailable);
            }
        }
        for (table, key, field, expected) in [
            (
                "domain_command_receipts",
                "command_id",
                "receipt",
                migration
                    .domain_receipts
                    .iter()
                    .map(|r| {
                        Ok((
                            r.command.command_id.clone(),
                            serde_json::to_value(r).map_err(|_| Error::Unavailable)?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, Error>>()?,
            ),
            (
                "game_encounters",
                "encounter_id",
                "encounter",
                migration
                    .encounters
                    .iter()
                    .map(|e| {
                        Ok((
                            e.id.clone(),
                            serde_json::to_value(e).map_err(|_| Error::Unavailable)?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, Error>>()?,
            ),
        ] {
            for row in tx.query(&format!("SELECT {key},{field} FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"), &[&profile]).map_err(|_| Error::Unavailable)? {
                let id: String = row.get(0);
                let value: Value = row.get(1);
                if expected.get(&id) != Some(&value) {
                    return Err(Error::Unavailable);
                }
            }
        }
        let items: std::collections::BTreeSet<_> =
            migration.mastery_items.iter().cloned().collect();
        for row in tx.query("SELECT skill_id,item_family,parameter_fingerprint FROM wonderwilds.game_mastery_items WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_| Error::Unavailable)? {
            let item = MasteryItem { skill_id: row.get(0), item_family: row.get(1), parameter_fingerprint: row.get(2) };
            if !items.contains(&item) {
                return Err(Error::Unavailable);
            }
        }
        let has_checkpoint: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM wonderwilds.game_checkpoints WHERE profile_id=$1::text::uuid) OR EXISTS(SELECT 1 FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND replay_entry IS NOT NULL)", &[&profile]).map_err(|_| Error::Unavailable)?.get(0);
        if has_checkpoint {
            return Err(Error::Unavailable);
        }
        let grants: std::collections::BTreeSet<_> = migration
            .events
            .iter()
            .filter_map(|event| match &event.outcome {
                engine::Outcome::RewardGranted {
                    reward_id,
                    quest_id,
                } => Some((reward_id.clone(), quest_id.clone())),
                _ => None,
            })
            .collect();
        for row in tx.query("SELECT reward_definition,quest_instance FROM wonderwilds.reward_grants WHERE profile_id=$1::text::uuid", &[&profile]).map_err(|_| Error::Unavailable)? {
            if !grants.contains(&(row.get::<_, String>(0), row.get::<_, String>(1))) {
                return Err(Error::Unavailable);
            }
        }
        Ok(())
    }

    /// Rebuild a validated legacy-format recovery/export candidate under the
    /// owned-profile lock. This never overwrites current progress with a backup.
    /// Loading all history is intentionally an operator recovery operation,
    /// separate from bounded ordinary game reads and commands.
    pub fn reconstruct_legacy(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        Self::lock_profile(&mut tx, identity)?;
        let aggregate = Self::load(&mut tx, identity, &self.content)?;
        let api_receipts = tx.query("SELECT command_id,request,response FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid ORDER BY command_id", &[&identity.profile_id]).map_err(|_| Error::Unavailable)?.iter().map(|row| (row.get(0),row.get(1),row.get(2))).collect();
        fn records<T: serde::de::DeserializeOwned>(
            tx: &mut Transaction<'_>,
            profile: &str,
            table: &str,
            field: &str,
            order: &str,
        ) -> Result<Vec<T>, Error> {
            tx.query(&format!("SELECT {field} FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid ORDER BY {order}"), &[&profile]).map_err(|_| Error::Unavailable)?.iter().map(|row| serde_json::from_value(row.get(0)).map_err(|_| Error::Unavailable)).collect()
        }
        let history = storage::StorageHistory {
            api_receipts,
            domain_receipts: records(
                &mut tx,
                &identity.profile_id,
                "domain_command_receipts",
                "receipt",
                "command_id",
            )?,
            encounters: records(
                &mut tx,
                &identity.profile_id,
                "game_encounters",
                "encounter",
                "encounter_id",
            )?,
            evidence: records(
                &mut tx,
                &identity.profile_id,
                "learning_events",
                "evidence",
                "sequence",
            )?,
            events: records(
                &mut tx,
                &identity.profile_id,
                "game_events",
                "event",
                "sequence",
            )?,
        };
        let restored = aggregate.restore_legacy_history(&self.content, history)?;
        let value = serde_json::to_value(restored).map_err(|_| Error::Unavailable)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(value)
    }
}
impl Repository for PostgresRepository {
    fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let row=client.query_one("SELECT to_regclass('wonderwilds.command_receipts') IS NOT NULL AND to_regclass('wonderwilds.domain_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.game_encounters') IS NOT NULL AND to_regclass('wonderwilds.game_checkpoints') IS NOT NULL AND to_regclass('wonderwilds.game_mastery_items') IS NOT NULL AND EXISTS(SELECT 1 FROM information_schema.columns WHERE table_schema='wonderwilds' AND table_name='progress' AND column_name='snapshot_digest')",&[]).map_err(|_|Error::Unavailable)?;
        if row.get::<_, bool>(0) {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
    fn read(&self, identity: &Identity) -> Result<Aggregate, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        Self::lock_profile(&mut tx, identity)?;
        let aggregate = Self::load(&mut tx, identity, &self.content)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(aggregate)
    }
    fn execute(&self, identity: &Identity, request: &Value, now: u64) -> Result<Value, Error> {
        wire::validate("GameCommand", request).map_err(|_| Error::InvalidInput)?;
        let id = request["commandId"].as_str().ok_or(Error::InvalidInput)?;
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        Self::lock_profile(&mut tx, identity)?;
        // Migrate legacy receipts before lookup, then exact retries precede revision/rate checks.
        let mut aggregate = Self::load(&mut tx, identity, &self.content)?;
        if let Some(row)=tx.query_opt("SELECT request,response FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",&[&identity.profile_id,&id]).map_err(|_|Error::Unavailable)?{
            if row.get::<_,Value>(0)!=*request{return Err(Error::RevisionConflict)}
            let response=row.get(1);tx.commit().map_err(|_|Error::Unavailable)?;return Ok(response)
        }
        if request["expectedRevision"].as_u64() != Some(aggregate.revision) {
            return Err(Error::RevisionConflict);
        }
        let row=tx.query_one("SELECT rate_window_start,rate_window_count FROM wonderwilds.progress WHERE profile_id=$1::text::uuid",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?;
        let start: i64 = row.get(0);
        let used: i32 = row.get(1);
        let now = sequence(now)?;
        let (start, used) = if now.saturating_sub(start) >= 60 {
            (now, 0)
        } else {
            (start, used)
        };
        if used >= COMMANDS_PER_MINUTE {
            return Err(Error::RateLimited);
        }
        let domain_id = format!("api-{}", digest_id(id));
        let domain_receipt=tx.query_opt("SELECT receipt FROM wonderwilds.domain_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",&[&identity.profile_id,&domain_id]).map_err(|_|Error::Unavailable)?.map(|row|serde_json::from_value::<engine::DomainReceipt>(row.get(0)).map_err(|_|Error::Unavailable)).transpose()?;
        let delta = aggregate.execute_persisted(
            request,
            &self.content,
            now as u64,
            &identity.session_id,
            domain_receipt.as_ref(),
            &mut |item| Self::novel_item(&mut tx, &identity.profile_id, item),
        )?;
        Self::store_effects(
            &mut tx,
            &identity.profile_id,
            &delta.evidence,
            &delta.events,
            false,
        )?;
        Self::store_domain_receipts(&mut tx, &identity.profile_id, &delta.domain_receipts)?;
        Self::store_encounters(&mut tx, &identity.profile_id, &delta.encounters, false)?;
        for replay in &delta.domain_replays {
            let replay_id = replay["command"]["commandId"]
                .as_str()
                .ok_or(Error::Unavailable)?;
            let changed=tx.execute("UPDATE wonderwilds.domain_command_receipts SET replay_entry=$1 WHERE profile_id=$2::text::uuid AND command_id=$3 AND replay_entry IS NULL",&[replay,&identity.profile_id,&replay_id]).map_err(|_|Error::Unavailable)?;
            if changed != 1 {
                return Err(Error::Unavailable);
            }
        }
        Self::save_checkpoint(&mut tx, &identity.profile_id, &aggregate, false)?;
        tx.execute("INSERT INTO wonderwilds.command_receipts(profile_id,command_id,request,response) VALUES($1::text::uuid,$2,$3,$4)",&[&identity.profile_id,&id,request,&delta.response]).map_err(|_|Error::Unavailable)?;
        tx.execute("UPDATE wonderwilds.progress SET rate_window_start=$1,rate_window_count=$2 WHERE profile_id=$3::text::uuid",&[&start,&(used+1),&identity.profile_id]).map_err(|_|Error::Unavailable)?;
        tx.execute("UPDATE wonderwilds.profiles SET revision=$1 WHERE id=$2::text::uuid AND parent_id=$3::text::uuid",&[&sequence(aggregate.revision)?,&identity.profile_id,&identity.parent_id]).map_err(|_|Error::Unavailable)?;
        #[cfg(test)]
        if self.fail_before_commit {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(delta.response)
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
                "SELECT count(*) FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid",
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
            "checkpoint keeps evidence in durable history"
        );
        let abandoned_evidence: Value = admin.query_one("SELECT evidence FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid ORDER BY sequence DESC LIMIT 1", &[&profile]).unwrap().get(0);
        assert_eq!(abandoned_evidence["eventKind"], "abandonment");
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
                "SELECT count(*) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(evidence, 3);
        let events: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.game_events WHERE profile_id=$1::text::uuid",
                &[&profile],
            )
            .unwrap()
            .get(0);
        let outbox: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.game_outbox WHERE profile_id=$1::text::uuid",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(events, outbox);
        let grants: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id=$1::text::uuid",
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
        let recruited = super::super::companion_tests::recruited_pip_via_api(&content);
        let training_revision = recruited.revision;
        admin
            .execute(
                "UPDATE wonderwilds.profiles SET revision=$2 WHERE id=$1::text::uuid",
                &[&training_profile, &sequence(training_revision).unwrap()],
            )
            .unwrap();
        let seeded = serde_json::to_value(recruited).unwrap();
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
        let started = repo
            .execute(
                &training_identity,
                &command(
                    "db-training-start",
                    training_revision,
                    json!({"type":"start_encounter","beatId":"training-pip-1"}),
                ),
                20,
            )
            .unwrap();
        let encounter = &started["pendingEncounters"][0];
        let answer = encounter["parameters"]["total"].as_u64().unwrap()
            - encounter["parameters"]["given"].as_u64().unwrap();
        let attempt = command(
            "db-training-answer",
            training_revision + 1,
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
                "SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id=$1::text::uuid",
                &[&training_profile],
            )
            .unwrap()
            .get(0);
        assert!(
            count > 0,
            "legacy story reward grants were preserved by migration"
        );
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
        let count: i64 = admin.query_one("SELECT count(*) FROM wonderwilds.reward_grants WHERE profile_id=$1::text::uuid AND reward_definition='character-xp:training-pip-1'", &[&training_profile]).unwrap().get(0);
        assert_eq!(count, 1);
        let count: i64 = admin
            .query_one(
                "SELECT count(*) FROM wonderwilds.learning_events WHERE profile_id=$1::text::uuid AND evidence->>'contextId'='training-pip-1'",
                &[&training_profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 2);
        admin
            .execute(
                "DELETE FROM wonderwilds.profiles WHERE id=$1::text::uuid",
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
            "game_mastery_items",
        ] {
            let count: i64 = admin
                .query_one(
                    &format!(
                        "SELECT count(*) FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"
                    ),
                    &[&profile],
                )
                .unwrap()
                .get(0);
            assert_eq!(count, 0);
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
            .batch_execute("DROP OWNED BY ww_game_test; DROP ROLE ww_game_test;")
            .unwrap();
    }
}

#[cfg(test)]
#[path = "repository_storage_tests.rs"]
mod storage_tests;
