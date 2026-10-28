-- Run only in an isolated synthetic database after 0001_parent_profiles.sql.
-- Requires a migration/test operator able to create/set a non-BYPASSRLS role.
-- Every fixture and role is rolled back. psql --set ON_ERROR_STOP=1 --file ...
BEGIN;
CREATE ROLE wonderwilds_identity_test NOLOGIN NOSUPERUSER NOBYPASSRLS;
GRANT USAGE ON SCHEMA wonderwilds TO wonderwilds_identity_test;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA wonderwilds TO wonderwilds_identity_test;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO wonderwilds_identity_test;
INSERT INTO wonderwilds.parents(id, identity_issuer, identity_subject) VALUES
 ('10000000-0000-0000-0000-000000000001', 'synthetic-test', 'parent-a'),
 ('10000000-0000-0000-0000-000000000002', 'synthetic-test', 'parent-b');
SET LOCAL ROLE wonderwilds_identity_test;
SELECT set_config('app.parent_id', '10000000-0000-0000-0000-000000000001', true);
INSERT INTO wonderwilds.profiles(id, parent_id, ability_band) VALUES
 ('20000000-0000-0000-0000-000000000001', wonderwilds.current_parent(), 'beginning');
INSERT INTO wonderwilds.progress VALUES ('20000000-0000-0000-0000-000000000001', '{"bridge":"repaired","syntheticUnicode":"Pip’s café 🌼"}');
INSERT INTO wonderwilds.learning_events(profile_id, sequence, evidence) VALUES
 ('20000000-0000-0000-0000-000000000001', 1, '{"synthetic":true}');
INSERT INTO wonderwilds.consent_records(profile_id, notice_version, purpose) VALUES
 ('20000000-0000-0000-0000-000000000001', 'notice-1', 'save-progress');
INSERT INTO wonderwilds.export_jobs(profile_id, status, expires_at, object_key) VALUES
 ('20000000-0000-0000-0000-000000000001', 'ready', now() + interval '1 hour', 'synthetic-export');
INSERT INTO wonderwilds.profile_cache VALUES ('20000000-0000-0000-0000-000000000001', '{}');
DO $$ DECLARE exported jsonb; BEGIN
 exported := wonderwilds.export_profile('20000000-0000-0000-0000-000000000001');
 IF exported #>> '{progress,bridge}' <> 'repaired'
    OR exported #>> '{progress,syntheticUnicode}' <> 'Pip’s café 🌼'
    OR jsonb_array_length(exported->'learningEvents') <> 1
    OR jsonb_array_length(exported->'consent') <> 1 THEN
   RAISE EXCEPTION 'export incomplete';
 END IF;
END $$;
SELECT set_config('app.parent_id', '10000000-0000-0000-0000-000000000002', true);
DO $$ BEGIN
 IF EXISTS(SELECT 1 FROM wonderwilds.profiles) OR EXISTS(SELECT 1 FROM wonderwilds.learning_events) THEN
   RAISE EXCEPTION 'foreign profile leaked';
 END IF;
 DELETE FROM wonderwilds.parents WHERE id = '10000000-0000-0000-0000-000000000001';
 IF FOUND THEN
   RAISE EXCEPTION 'foreign parent deleted';
 END IF;
 BEGIN
   INSERT INTO wonderwilds.profiles(parent_id, ability_band) VALUES
    ('10000000-0000-0000-0000-000000000001', 'beginning');
   RAISE EXCEPTION 'foreign profile inserted';
 EXCEPTION WHEN insufficient_privilege THEN NULL;
 END;
 BEGIN
   PERFORM wonderwilds.export_profile('20000000-0000-0000-0000-000000000001');
   RAISE EXCEPTION 'foreign export accepted';
 EXCEPTION WHEN insufficient_privilege THEN NULL;
 END;
 BEGIN
   PERFORM wonderwilds.delete_profile('20000000-0000-0000-0000-000000000001');
   RAISE EXCEPTION 'foreign deletion accepted';
 EXCEPTION WHEN insufficient_privilege THEN NULL;
 END;
END $$;
SELECT set_config('app.parent_id', '10000000-0000-0000-0000-000000000001', true);
SELECT wonderwilds.delete_profile('20000000-0000-0000-0000-000000000001');
DO $$ BEGIN
 IF EXISTS(SELECT 1 FROM wonderwilds.profiles)
    OR EXISTS(SELECT 1 FROM wonderwilds.learning_events)
    OR EXISTS(SELECT 1 FROM wonderwilds.progress)
    OR EXISTS(SELECT 1 FROM wonderwilds.consent_records)
    OR EXISTS(SELECT 1 FROM wonderwilds.profile_cache)
    OR EXISTS(SELECT 1 FROM wonderwilds.export_jobs) THEN
   RAISE EXCEPTION 'profile deletion incomplete';
 END IF;
 IF NOT EXISTS(SELECT 1 FROM wonderwilds.deletion_outbox WHERE object_key = 'synthetic-export') THEN
   RAISE EXCEPTION 'object cleanup lost';
 END IF;
 BEGIN
   INSERT INTO wonderwilds.profiles(id, parent_id, ability_band) VALUES
     ('20000000-0000-0000-0000-000000000001', wonderwilds.current_parent(), 'beginning');
   RAISE EXCEPTION 'deleted profile resurrected';
 EXCEPTION WHEN check_violation THEN NULL;
 END;
END $$;
RESET ROLE;
ROLLBACK;
