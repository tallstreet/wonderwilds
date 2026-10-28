//! Synchronous PostgreSQL identity/lifecycle adapter. Run in a blocking executor.
//! The caller supplies a Client configured with verified TLS in production, or a
//! private Unix socket for local synthetic tests. No connection URI is logged.
use super::{ConsentRecord, Error, REAUTH_WINDOW_SECONDS, constant_time_eq};
use ::postgres::Client;
use serde_json::Value;

#[derive(Debug)]
pub enum RepositoryError {
    Access(Error),
    Storage(::postgres::Error),
}

impl From<::postgres::Error> for RepositoryError {
    fn from(error: ::postgres::Error) -> Self {
        Self::Storage(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedContext {
    pub parent_id: String,
    pub profile_id: String,
    pub reauthenticated_at: u64,
}

impl VerifiedContext {
    pub fn require_recent_reauthentication(&self, now: u64) -> Result<(), Error> {
        if now < self.reauthenticated_at || now - self.reauthenticated_at > REAUTH_WINDOW_SECONDS {
            return Err(Error::Forbidden);
        }
        Ok(())
    }
}

pub struct PostgresIdentity {
    client: Client,
}

impl PostgresIdentity {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn authenticate_context(
        &mut self,
        token_digest: &[u8; 32],
        csrf_digest: Option<&[u8; 32]>,
        origin: Option<&str>,
        configured_origin: &str,
        profile_id: &str,
        now: u64,
    ) -> Result<VerifiedContext, RepositoryError> {
        let mut transaction = self.client.transaction()?;
        let row = transaction.query_opt(
            "SELECT parent_id::text, csrf_digest, extract(epoch FROM expires_at)::bigint, extract(epoch FROM reauthenticated_at)::bigint, revoked_at IS NOT NULL FROM wonderwilds.sessions WHERE token_digest = $1",
            &[&&token_digest[..]],
        )?.ok_or(RepositoryError::Access(Error::Unauthenticated))?;
        let parent_id: String = row.get(0);
        let stored_csrf: Vec<u8> = row.get(1);
        let expires_at: i64 = row.get(2);
        let reauthenticated_at: i64 = row.get(3);
        let revoked: bool = row.get(4);
        if revoked
            || expires_at < 0
            || now >= expires_at as u64
            || reauthenticated_at < 0
            || now < reauthenticated_at as u64
        {
            return Err(RepositoryError::Access(Error::Unauthenticated));
        }
        if let Some(submitted_csrf) = csrf_digest {
            let stored: [u8; 32] = stored_csrf
                .try_into()
                .map_err(|_| RepositoryError::Access(Error::Unauthenticated))?;
            if !configured_origin.starts_with("https://")
                || origin != Some(configured_origin)
                || !constant_time_eq(&stored, submitted_csrf)
            {
                return Err(RepositoryError::Access(Error::Forbidden));
            }
        }
        transaction.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&parent_id],
        )?;
        let owns_profile = transaction
            .query_opt(
                "SELECT id FROM wonderwilds.profiles WHERE id::text = $1 AND parent_id::text = $2",
                &[&profile_id, &parent_id],
            )?
            .is_some();
        if !owns_profile {
            return Err(RepositoryError::Access(Error::Forbidden));
        }
        transaction.commit()?;
        Ok(VerifiedContext {
            parent_id,
            profile_id: profile_id.into(),
            reauthenticated_at: reauthenticated_at as u64,
        })
    }

    pub fn create_profile(
        &mut self,
        verified_parent_id: &str,
        ability_band: &str,
    ) -> Result<String, RepositoryError> {
        if !matches!(ability_band, "beginning" | "growing" | "stretching") {
            return Err(RepositoryError::Access(Error::InvalidInput));
        }
        let mut transaction = self.client.transaction()?;
        transaction.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&verified_parent_id],
        )?;
        let profile_id = transaction.query_one(
            "INSERT INTO wonderwilds.profiles(parent_id, ability_band) VALUES($1::text::uuid, $2) RETURNING id::text",
            &[&verified_parent_id, &ability_band],
        )?.get(0);
        transaction.commit()?;
        Ok(profile_id)
    }

    pub fn export_profile(
        &mut self,
        context: &VerifiedContext,
        now: u64,
    ) -> Result<Value, RepositoryError> {
        context
            .require_recent_reauthentication(now)
            .map_err(RepositoryError::Access)?;
        let mut transaction = self
            .client
            .build_transaction()
            .isolation_level(::postgres::IsolationLevel::RepeatableRead)
            .start()?;
        transaction.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&context.parent_id],
        )?;
        let result = transaction
            .query_one(
                "SELECT wonderwilds.export_profile($1::text::uuid)",
                &[&context.profile_id],
            )?
            .get(0);
        transaction.commit()?;
        Ok(result)
    }

    pub fn record_consent(
        &mut self,
        context: &VerifiedContext,
        consent: &ConsentRecord,
    ) -> Result<(), RepositoryError> {
        let recorded_at = consent.recorded_at as f64;
        let revoked_at = consent.revoked_at.map(|time| time as f64);
        let mut transaction = self.client.transaction()?;
        transaction.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&context.parent_id],
        )?;
        transaction.execute("INSERT INTO wonderwilds.consent_records(profile_id, notice_version, purpose, recorded_at, revoked_at) VALUES($1::text::uuid, $2, $3, to_timestamp($4::double precision), to_timestamp($5::double precision))", &[&context.profile_id, &consent.notice_version, &consent.purpose, &recorded_at, &revoked_at])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn delete_profile(
        &mut self,
        context: &VerifiedContext,
        now: u64,
    ) -> Result<(), RepositoryError> {
        context
            .require_recent_reauthentication(now)
            .map_err(RepositoryError::Access)?;
        let mut transaction = self.client.transaction()?;
        transaction.query_one(
            "SELECT set_config('app.parent_id', $1, true)",
            &[&context.parent_id],
        )?;
        transaction.query_one(
            "SELECT wonderwilds.delete_profile($1::text::uuid)",
            &[&context.profile_id],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn revoke_session(&mut self, token_digest: &[u8; 32]) -> Result<(), RepositoryError> {
        self.client.execute("UPDATE wonderwilds.sessions SET revoked_at = now() WHERE token_digest = $1 AND revoked_at IS NULL", &[&&token_digest[..]])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::postgres::NoTls;

    #[test]
    #[ignore = "requires a dedicated migrated synthetic PostgreSQL database in WW_IDENTITY_TEST_DATABASE_URL"]
    fn postgres_identity_ownership_export_revocation_and_delete() {
        let url = std::env::var("WW_IDENTITY_TEST_DATABASE_URL")
            .expect("dedicated synthetic database required");
        let mut operator = Client::connect(&url, NoTls).unwrap();
        operator.batch_execute("CREATE ROLE ww_identity_adapter_test NOLOGIN NOSUPERUSER NOBYPASSRLS;
            GRANT USAGE ON SCHEMA wonderwilds TO ww_identity_adapter_test;
            GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_identity_adapter_test;
            GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_identity_adapter_test;
            INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES
            ('30000000-0000-0000-0000-000000000001','synthetic-adapter','a'),
            ('30000000-0000-0000-0000-000000000002','synthetic-adapter','b');").unwrap();
        for (token, csrf, parent) in [
            ([1u8; 32], [2u8; 32], "30000000-0000-0000-0000-000000000001"),
            ([3u8; 32], [4u8; 32], "30000000-0000-0000-0000-000000000002"),
        ] {
            operator.execute("INSERT INTO wonderwilds.sessions(token_digest,csrf_digest,parent_id,expires_at,reauthenticated_at) VALUES($1,$2,$3::text::uuid,to_timestamp(1000),to_timestamp(10))", &[&&token[..], &&csrf[..], &parent]).unwrap();
        }
        let mut runtime = Client::connect(&url, NoTls).unwrap();
        runtime
            .batch_execute("SET ROLE ww_identity_adapter_test")
            .unwrap();
        let mut repository = PostgresIdentity::new(runtime);
        let parent_a = "30000000-0000-0000-0000-000000000001";
        let profile = repository.create_profile(parent_a, "beginning").unwrap();
        assert!(
            repository
                .authenticate_context(&[3; 32], None, None, "https://game.example", &profile, 11)
                .is_err()
        );
        assert!(
            repository
                .authenticate_context(
                    &[1; 32],
                    Some(&[2; 32]),
                    Some("https://evil.example"),
                    "https://game.example",
                    &profile,
                    11
                )
                .is_err()
        );
        let context = repository
            .authenticate_context(
                &[1; 32],
                Some(&[2; 32]),
                Some("https://game.example"),
                "https://game.example",
                &profile,
                11,
            )
            .unwrap();
        repository
            .record_consent(
                &context,
                &ConsentRecord {
                    notice_version: "notice-1".into(),
                    purpose: "save-progress".into(),
                    recorded_at: 11,
                    revoked_at: Some(12),
                },
            )
            .unwrap();
        let export = repository.export_profile(&context, 12).unwrap();
        assert_eq!(export["profile"]["abilityBand"], "beginning");
        assert_eq!(export["consent"].as_array().unwrap().len(), 1);
        assert!(repository.export_profile(&context, 400).is_err());
        repository.delete_profile(&context, 12).unwrap();
        assert!(repository.export_profile(&context, 12).is_err());
        repository.revoke_session(&[1; 32]).unwrap();
        assert!(
            repository
                .authenticate_context(&[1; 32], None, None, "https://game.example", &profile, 13)
                .is_err()
        );
        assert_eq!(operator.query_one("SELECT count(*) FROM wonderwilds.deletion_tombstones WHERE profile_id::text=$1", &[&profile]).unwrap().get::<_,i64>(0),1);
        operator.batch_execute("DELETE FROM wonderwilds.parents WHERE identity_issuer='synthetic-adapter'; DROP OWNED BY ww_identity_adapter_test; DROP ROLE ww_identity_adapter_test;").unwrap();
    }
}
