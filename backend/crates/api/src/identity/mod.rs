//! Parent-only lifecycle primitives. HTTP adapters must verify an external identity
//! before issuing sessions; this module does not provide a development login.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub mod postgres;

pub const SESSION_COOKIE_ATTRIBUTES: &str = "Secure; HttpOnly; SameSite=Strict; Path=/";
pub const REAUTH_WINDOW_SECONDS: u64 = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unauthenticated,
    Forbidden,
    InvalidInput,
    Deleted,
}

/// Digests of cryptographically random bearer and CSRF secrets, calculated by
/// the authentication adapter. Never pass raw tokens into logs or persistence.
#[derive(Clone)]
pub struct Session {
    parent_id: String,
    token_digest: [u8; 32],
    csrf_digest: [u8; 32],
    expires_at: u64,
    reauthenticated_at: u64,
    revoked: bool,
}

impl Session {
    /// Call only after verification of the selected identity provider's assertion.
    /// This constructor deliberately performs no authentication or token generation.
    pub fn after_verified_identity(
        parent_id: String,
        token_digest: [u8; 32],
        csrf_digest: [u8; 32],
        now: u64,
        expires_at: u64,
    ) -> Result<Self, Error> {
        if parent_id.is_empty() || expires_at <= now || token_digest == csrf_digest {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            parent_id,
            token_digest,
            csrf_digest,
            expires_at,
            reauthenticated_at: now,
            revoked: false,
        })
    }

    pub fn authenticate(&self, token_digest: &[u8; 32], now: u64) -> Result<&str, Error> {
        if self.revoked
            || now >= self.expires_at
            || !constant_time_eq(&self.token_digest, token_digest)
        {
            return Err(Error::Unauthenticated);
        }
        Ok(&self.parent_id)
    }

    pub fn authorize_write(
        &self,
        token_digest: &[u8; 32],
        csrf_digest: &[u8; 32],
        origin: &str,
        configured_origin: &str,
        now: u64,
    ) -> Result<&str, Error> {
        let parent = self.authenticate(token_digest, now)?;
        if !configured_origin.starts_with("https://")
            || origin != configured_origin
            || !constant_time_eq(&self.csrf_digest, csrf_digest)
        {
            return Err(Error::Forbidden);
        }
        Ok(parent)
    }

    pub fn require_recent_reauthentication(&self, now: u64) -> Result<(), Error> {
        if self.revoked || now >= self.expires_at || now < self.reauthenticated_at {
            return Err(Error::Unauthenticated);
        }
        if now - self.reauthenticated_at > REAUTH_WINDOW_SECONDS {
            return Err(Error::Forbidden);
        }
        Ok(())
    }

    /// Revoke before issuing a fresh session on rotation, reauthentication or logout.
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub parent_id: String,
    pub ability_band: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConsentRecord {
    pub notice_version: String,
    pub purpose: String,
    pub recorded_at: u64,
    pub revoked_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProfileExport {
    pub schema_version: String,
    pub profile: Profile,
    pub progress: BTreeMap<String, String>,
    pub learning_events: Vec<String>,
    pub consent: Vec<ConsentRecord>,
}

/// Synthetic test/reference adapter. Production must use ownership-scoped SQL
/// transactions and a durable independent tombstone journal (see migrations).
#[derive(Default)]
pub struct MemoryRepository {
    profiles: BTreeMap<String, ProfileExport>,
    tombstones: BTreeSet<String>,
    cached_profiles: BTreeSet<String>,
    export_jobs: BTreeSet<String>,
}

impl MemoryRepository {
    pub fn create(&mut self, profile: Profile) -> Result<(), Error> {
        if profile.id.is_empty() || profile.parent_id.is_empty() || profile.ability_band.is_empty()
        {
            return Err(Error::InvalidInput);
        }
        if self.tombstones.contains(&profile.id) {
            return Err(Error::Deleted);
        }
        if self.profiles.contains_key(&profile.id) {
            return Err(Error::InvalidInput);
        }
        self.profiles.insert(
            profile.id.clone(),
            ProfileExport {
                schema_version: "1.0".into(),
                profile,
                progress: BTreeMap::new(),
                learning_events: Vec::new(),
                consent: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn export(&self, parent_id: &str, profile_id: &str) -> Result<ProfileExport, Error> {
        // Deliberately identical response for absent and foreign profiles.
        self.profiles
            .get(profile_id)
            .filter(|record| record.profile.parent_id == parent_id)
            .cloned()
            .ok_or(Error::Forbidden)
    }

    pub fn record_consent(
        &mut self,
        parent_id: &str,
        profile_id: &str,
        consent: ConsentRecord,
    ) -> Result<(), Error> {
        if consent.notice_version.is_empty() || consent.purpose.is_empty() {
            return Err(Error::InvalidInput);
        }
        self.export(parent_id, profile_id)?;
        self.profiles
            .get_mut(profile_id)
            .ok_or(Error::Forbidden)?
            .consent
            .push(consent);
        Ok(())
    }

    pub fn delete(&mut self, parent_id: &str, profile_id: &str) -> Result<(), Error> {
        self.export(parent_id, profile_id)?;
        self.tombstones.insert(profile_id.to_owned());
        self.profiles.remove(profile_id);
        self.cached_profiles.remove(profile_id);
        self.export_jobs.remove(profile_id);
        Ok(())
    }

    /// Restore only after loading the latest separately retained deletion journal.
    pub fn restore(&mut self, records: Vec<ProfileExport>) {
        for record in records {
            if !self.tombstones.contains(&record.profile.id) {
                self.profiles.insert(record.profile.id.clone(), record);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> MemoryRepository {
        let mut repository = MemoryRepository::default();
        for (profile, parent) in [("child-a", "parent-a"), ("child-b", "parent-b")] {
            repository
                .create(Profile {
                    id: profile.into(),
                    parent_id: parent.into(),
                    ability_band: "beginning".into(),
                })
                .unwrap();
        }
        repository
    }

    #[test]
    fn parent_isolation_covers_reads_consent_and_deletion() {
        let mut repository = repository();
        assert_eq!(
            repository.export("parent-a", "child-b"),
            Err(Error::Forbidden)
        );
        assert_eq!(
            repository.delete("parent-a", "child-b"),
            Err(Error::Forbidden)
        );
        assert_eq!(
            repository.record_consent(
                "parent-a",
                "child-b",
                ConsentRecord {
                    notice_version: "1".into(),
                    purpose: "play".into(),
                    recorded_at: 1,
                    revoked_at: None
                }
            ),
            Err(Error::Forbidden)
        );
        assert!(repository.export("parent-b", "child-b").is_ok());
    }

    #[test]
    fn deletion_clears_events_cache_exports_and_prevents_backup_resurrection() {
        let mut repository = repository();
        repository.cached_profiles.insert("child-a".into());
        repository.export_jobs.insert("child-a".into());
        repository
            .profiles
            .get_mut("child-a")
            .unwrap()
            .learning_events
            .push("synthetic-event".into());
        let backup = repository.export("parent-a", "child-a").unwrap();
        repository.delete("parent-a", "child-a").unwrap();
        repository.restore(vec![backup.clone()]);
        assert_eq!(
            repository.export("parent-a", "child-a"),
            Err(Error::Forbidden)
        );
        assert_eq!(repository.create(backup.profile), Err(Error::Deleted));
        assert!(repository.cached_profiles.is_empty());
        assert!(repository.export_jobs.is_empty());
        assert!(repository.export("parent-b", "child-b").is_ok());
    }

    #[test]
    fn export_includes_consent_events_and_progress_without_child_contact_fields() {
        let mut repository = repository();
        let record = repository.profiles.get_mut("child-a").unwrap();
        record.progress.insert("bridge".into(), "repaired".into());
        record.learning_events.push("synthetic-event".into());
        repository
            .record_consent(
                "parent-a",
                "child-a",
                ConsentRecord {
                    notice_version: "notice-1".into(),
                    purpose: "save-progress".into(),
                    recorded_at: 7,
                    revoked_at: Some(8),
                },
            )
            .unwrap();
        let export = repository.export("parent-a", "child-a").unwrap();
        let json = serde_json::to_string(&export).unwrap();
        assert_eq!(
            serde_json::from_str::<ProfileExport>(&json).unwrap(),
            export
        );
        assert_eq!(export.consent[0].revoked_at, Some(8));
        assert_eq!(export.learning_events.len(), 1);
        assert_eq!(export.progress["bridge"], "repaired");
        assert!(!json.contains("email"));
    }

    #[test]
    fn sessions_require_valid_token_csrf_origin_expiry_and_recent_reauth() {
        let mut session =
            Session::after_verified_identity("parent-a".into(), [1; 32], [2; 32], 10, 1000)
                .unwrap();
        assert_eq!(
            session.authenticate(&[3; 32], 11),
            Err(Error::Unauthenticated)
        );
        assert_eq!(
            session.authorize_write(
                &[1; 32],
                &[2; 32],
                "https://evil.example",
                "https://game.example",
                11
            ),
            Err(Error::Forbidden)
        );
        assert_eq!(
            session.authorize_write(
                &[1; 32],
                &[3; 32],
                "https://game.example",
                "https://game.example",
                11
            ),
            Err(Error::Forbidden)
        );
        assert_eq!(
            session.authorize_write(
                &[1; 32],
                &[2; 32],
                "https://game.example",
                "https://game.example",
                11
            ),
            Ok("parent-a")
        );
        assert_eq!(
            session.require_recent_reauthentication(311),
            Err(Error::Forbidden)
        );
        assert_eq!(
            session.authenticate(&[1; 32], 1000),
            Err(Error::Unauthenticated)
        );
        session.revoke();
        assert_eq!(
            session.authenticate(&[1; 32], 11),
            Err(Error::Unauthenticated)
        );
    }
}
