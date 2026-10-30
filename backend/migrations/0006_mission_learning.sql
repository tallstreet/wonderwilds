BEGIN;
-- Separate graph practice from the assessed adventure and story checkpoint.
CREATE TABLE wonderwilds.mission_learning_progress (
    profile_id uuid PRIMARY KEY REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    revision bigint NOT NULL CHECK (revision BETWEEN 0 AND 1000000000),
    checkpoint jsonb NOT NULL CHECK (
        jsonb_typeof(checkpoint) = 'object' AND octet_length(checkpoint::text) <= 262144
    ),
    digest text NOT NULL CHECK (length(digest) = 64)
);
CREATE TABLE wonderwilds.mission_learning_command_receipts (
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    command_id text NOT NULL CHECK (length(command_id) BETWEEN 1 AND 120),
    revision bigint NOT NULL CHECK (revision BETWEEN 1 AND 1000000000),
    request jsonb NOT NULL CHECK (jsonb_typeof(request) = 'object' AND octet_length(request::text) <= 32768),
    response jsonb NOT NULL CHECK (jsonb_typeof(response) = 'object' AND octet_length(response::text) <= 65536),
    digest text NOT NULL CHECK (length(digest) = 64),
    PRIMARY KEY (profile_id, command_id),
    UNIQUE (profile_id, revision)
);
CREATE TABLE wonderwilds.mission_learning_events (
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    sequence bigint NOT NULL CHECK (sequence BETWEEN 1 AND 1000000000),
    event_id text NOT NULL CHECK (length(event_id) BETWEEN 1 AND 160),
    command_id text NOT NULL CHECK (length(command_id) BETWEEN 1 AND 120),
    record jsonb NOT NULL CHECK (jsonb_typeof(record) = 'object' AND octet_length(record::text) <= 32768),
    previous_digest text NOT NULL CHECK (length(previous_digest) = 64),
    digest text NOT NULL CHECK (length(digest) = 64),
    PRIMARY KEY (profile_id, sequence),
    UNIQUE (profile_id, event_id),
    UNIQUE (profile_id, command_id),
    FOREIGN KEY (profile_id, command_id)
        REFERENCES wonderwilds.mission_learning_command_receipts(profile_id, command_id)
        ON DELETE CASCADE
);

ALTER TABLE wonderwilds.command_windows DROP CONSTRAINT command_windows_kind_check;
ALTER TABLE wonderwilds.command_windows ADD CHECK (kind IN ('game', 'story', 'mission_learning'));

DO $$ DECLARE name text; BEGIN
    FOREACH name IN ARRAY ARRAY[
        'mission_learning_progress',
        'mission_learning_command_receipts',
        'mission_learning_events'
    ] LOOP
        EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY', name);
        EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY', name);
        EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent())) WITH CHECK (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent()))', name);
        EXECUTE format('REVOKE ALL ON wonderwilds.%I FROM PUBLIC', name);
    END LOOP;
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ww_demo_runtime') THEN
        GRANT SELECT, INSERT, UPDATE ON wonderwilds.mission_learning_progress TO ww_demo_runtime;
        GRANT SELECT, INSERT ON wonderwilds.mission_learning_command_receipts TO ww_demo_runtime;
        GRANT SELECT, INSERT ON wonderwilds.mission_learning_events TO ww_demo_runtime;
        GRANT SELECT, INSERT, UPDATE ON wonderwilds.command_windows TO ww_demo_runtime;
    END IF;
END $$;

ALTER FUNCTION wonderwilds.export_profile(uuid) RENAME TO export_profile_v5;
CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb;
BEGIN
    result := wonderwilds.export_profile_v5(target);
    RETURN result || jsonb_build_object(
        'missionLearningProgress', coalesce((SELECT checkpoint FROM wonderwilds.mission_learning_progress WHERE profile_id = target), '{}'::jsonb),
        'missionLearningReceipts', coalesce((SELECT jsonb_agg(jsonb_build_object('commandId', r.command_id, 'revision', r.revision, 'request', r.request, 'response', r.response) ORDER BY r.revision) FROM wonderwilds.mission_learning_command_receipts r WHERE r.profile_id = target), '[]'::jsonb),
        'missionLearningEvents', coalesce((SELECT jsonb_agg(jsonb_build_object('sequence', e.sequence, 'eventId', e.event_id, 'commandId', e.command_id, 'record', e.record) ORDER BY e.sequence) FROM wonderwilds.mission_learning_events e WHERE e.profile_id = target), '[]'::jsonb)
    );
END $$;
COMMIT;
