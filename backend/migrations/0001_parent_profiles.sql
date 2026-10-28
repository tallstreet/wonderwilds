BEGIN;
CREATE SCHEMA IF NOT EXISTS wonderwilds;

CREATE TABLE wonderwilds.parents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_issuer text NOT NULL,
    identity_subject text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(identity_issuer, identity_subject)
);
-- Contact data is intentionally not duplicated from the identity provider.
CREATE TABLE wonderwilds.profiles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id uuid NOT NULL REFERENCES wonderwilds.parents(id) ON DELETE CASCADE,
    ability_band text NOT NULL CHECK (ability_band IN ('beginning', 'growing', 'stretching')),
    revision bigint NOT NULL DEFAULT 0 CHECK (revision >= 0),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX profiles_parent ON wonderwilds.profiles(parent_id);
CREATE TABLE wonderwilds.sessions (
    token_digest bytea PRIMARY KEY CHECK (octet_length(token_digest) = 32),
    csrf_digest bytea NOT NULL CHECK (octet_length(csrf_digest) = 32),
    parent_id uuid NOT NULL REFERENCES wonderwilds.parents(id) ON DELETE CASCADE,
    expires_at timestamptz NOT NULL,
    reauthenticated_at timestamptz NOT NULL,
    revoked_at timestamptz,
    CHECK (token_digest <> csrf_digest),
    CHECK (expires_at > reauthenticated_at)
);
CREATE TABLE wonderwilds.progress (
    profile_id uuid PRIMARY KEY REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    snapshot jsonb NOT NULL CHECK (jsonb_typeof(snapshot) = 'object')
);
CREATE TABLE wonderwilds.learning_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    sequence bigint NOT NULL CHECK (sequence > 0),
    evidence jsonb NOT NULL CHECK (jsonb_typeof(evidence) = 'object'),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(profile_id, sequence)
);
CREATE TABLE wonderwilds.consent_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    notice_version text NOT NULL CHECK (length(notice_version) BETWEEN 1 AND 100),
    purpose text NOT NULL CHECK (length(purpose) BETWEEN 1 AND 100),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz,
    CHECK (revoked_at IS NULL OR revoked_at >= recorded_at)
);
CREATE TABLE wonderwilds.export_jobs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    status text NOT NULL CHECK (status IN ('pending', 'ready', 'failed')),
    expires_at timestamptz NOT NULL,
    -- Private object keys, never public URLs. Object deletion is a durable outbox task.
    object_key text
);
CREATE TABLE wonderwilds.profile_cache (
    profile_id uuid PRIMARY KEY REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    value jsonb NOT NULL
);
-- Retain this journal outside ordinary backup rollback. No FK: deletion must
-- survive removal of the original parent/profile and backup restoration.
CREATE TABLE wonderwilds.deletion_tombstones (
    profile_id uuid PRIMARY KEY,
    deleted_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE wonderwilds.deletion_outbox (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id uuid NOT NULL,
    object_key text NOT NULL,
    completed_at timestamptz,
    UNIQUE(profile_id, object_key)
);

CREATE FUNCTION wonderwilds.current_parent() RETURNS uuid
LANGUAGE sql STABLE AS $$
    SELECT nullif(current_setting('app.parent_id', true), '')::uuid
$$;
ALTER TABLE wonderwilds.parents ENABLE ROW LEVEL SECURITY;
ALTER TABLE wonderwilds.parents FORCE ROW LEVEL SECURITY;
CREATE POLICY parent_owned ON wonderwilds.parents
USING (id = wonderwilds.current_parent())
WITH CHECK (id = wonderwilds.current_parent());
ALTER TABLE wonderwilds.profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE wonderwilds.profiles FORCE ROW LEVEL SECURITY;
CREATE POLICY profiles_owned ON wonderwilds.profiles
USING (parent_id = wonderwilds.current_parent())
WITH CHECK (parent_id = wonderwilds.current_parent());

DO $$ DECLARE table_name text; BEGIN
    FOREACH table_name IN ARRAY ARRAY['progress', 'learning_events', 'consent_records', 'export_jobs', 'profile_cache'] LOOP
        EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent())) WITH CHECK (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent()))', table_name);
    END LOOP;
END $$;

CREATE FUNCTION wonderwilds.reject_deleted_profile() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM wonderwilds.deletion_tombstones WHERE profile_id = NEW.id) THEN
        RAISE EXCEPTION 'profile deleted' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER prevent_resurrection BEFORE INSERT ON wonderwilds.profiles
FOR EACH ROW EXECUTE FUNCTION wonderwilds.reject_deleted_profile();

CREATE FUNCTION wonderwilds.delete_profile(target uuid) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    -- Row lock serialises deletion against owned-profile updates.
    PERFORM 1 FROM wonderwilds.profiles WHERE id = target AND parent_id = wonderwilds.current_parent() FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'profile unavailable' USING ERRCODE = '42501';
    END IF;
    INSERT INTO wonderwilds.deletion_outbox(profile_id, object_key)
        SELECT target, object_key FROM wonderwilds.export_jobs WHERE profile_id = target AND object_key IS NOT NULL
        ON CONFLICT DO NOTHING;
    INSERT INTO wonderwilds.deletion_tombstones(profile_id) VALUES(target) ON CONFLICT DO NOTHING;
    DELETE FROM wonderwilds.profiles WHERE id = target AND parent_id = wonderwilds.current_parent();
END $$;

CREATE FUNCTION wonderwilds.require_profile_deletion_first() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM wonderwilds.profiles WHERE parent_id = OLD.id) THEN
        RAISE EXCEPTION 'delete owned profiles through delete_profile first' USING ERRCODE = '23514';
    END IF;
    RETURN OLD;
END $$;
CREATE TRIGGER parents_lifecycle BEFORE DELETE ON wonderwilds.parents
FOR EACH ROW EXECUTE FUNCTION wonderwilds.require_profile_deletion_first();

CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb;
BEGIN
    SELECT jsonb_build_object(
        'schemaVersion', '1.0',
        'profile', jsonb_build_object('id', p.id, 'abilityBand', p.ability_band, 'revision', p.revision),
        'progress', coalesce((SELECT snapshot FROM wonderwilds.progress WHERE profile_id = p.id), '{}'::jsonb),
        'learningEvents', coalesce((SELECT jsonb_agg(jsonb_build_object('id', e.id, 'sequence', e.sequence, 'evidence', e.evidence, 'recordedAt', e.recorded_at) ORDER BY e.sequence) FROM wonderwilds.learning_events e WHERE e.profile_id = p.id), '[]'::jsonb),
        'consent', coalesce((SELECT jsonb_agg(jsonb_build_object('noticeVersion', c.notice_version, 'purpose', c.purpose, 'recordedAt', c.recorded_at, 'revokedAt', c.revoked_at) ORDER BY c.recorded_at, c.id) FROM wonderwilds.consent_records c WHERE c.profile_id = p.id), '[]'::jsonb)
    ) INTO result
    FROM wonderwilds.profiles p WHERE p.id = target AND p.parent_id = wonderwilds.current_parent();
    IF result IS NULL THEN
        RAISE EXCEPTION 'profile unavailable' USING ERRCODE = '42501';
    END IF;
    RETURN result;
END $$;

REVOKE ALL ON ALL TABLES IN SCHEMA wonderwilds FROM PUBLIC;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA wonderwilds FROM PUBLIC;
COMMIT;
