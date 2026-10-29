BEGIN;
-- Authored story progress is isolated from assessed-game progress/revision.
CREATE TABLE wonderwilds.story_progress (
    profile_id uuid PRIMARY KEY REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    revision bigint NOT NULL CHECK (revision BETWEEN 1 AND 256),
    checkpoint jsonb NOT NULL CHECK (
        jsonb_typeof(checkpoint) = 'object' AND octet_length(checkpoint::text) <= 262144
    )
);
CREATE TABLE wonderwilds.story_command_receipts (
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    command_id text NOT NULL CHECK (length(command_id) BETWEEN 1 AND 80),
    revision bigint NOT NULL CHECK (revision BETWEEN 1 AND 256),
    request jsonb NOT NULL CHECK (
        jsonb_typeof(request) = 'object' AND octet_length(request::text) <= 32768
    ),
    response jsonb NOT NULL CHECK (
        jsonb_typeof(response) = 'object' AND octet_length(response::text) <= 32768
    ),
    PRIMARY KEY (profile_id, command_id),
    UNIQUE (profile_id, revision)
);
DO $$ DECLARE table_name text; BEGIN
    FOREACH table_name IN ARRAY ARRAY['story_progress', 'story_command_receipts'] LOOP
        EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent())) WITH CHECK (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent()))', table_name);
    END LOOP;
END $$;
REVOKE ALL ON wonderwilds.story_progress, wonderwilds.story_command_receipts FROM PUBLIC;

-- The existing lifecycle function owns the profile check and existing export
-- fields. Extend it in a new migration; deletion already cascades through FKs.
CREATE OR REPLACE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb;
BEGIN
    SELECT jsonb_build_object(
        'schemaVersion', '1.0',
        'profile', jsonb_build_object('id', p.id, 'abilityBand', p.ability_band, 'revision', p.revision),
        'progress', coalesce((SELECT snapshot FROM wonderwilds.progress WHERE profile_id = p.id), '{}'::jsonb),
        'learningEvents', coalesce((SELECT jsonb_agg(jsonb_build_object('id', e.id, 'sequence', e.sequence, 'evidence', e.evidence, 'recordedAt', e.recorded_at) ORDER BY e.sequence) FROM wonderwilds.learning_events e WHERE e.profile_id = p.id), '[]'::jsonb),
        'consent', coalesce((SELECT jsonb_agg(jsonb_build_object('noticeVersion', c.notice_version, 'purpose', c.purpose, 'recordedAt', c.recorded_at, 'revokedAt', c.revoked_at) ORDER BY c.recorded_at, c.id) FROM wonderwilds.consent_records c WHERE c.profile_id = p.id), '[]'::jsonb),
        'storyProgress', coalesce((SELECT checkpoint FROM wonderwilds.story_progress WHERE profile_id = p.id), '{}'::jsonb),
        'storyCommandReceipts', coalesce((SELECT jsonb_agg(jsonb_build_object('commandId', r.command_id, 'revision', r.revision, 'request', r.request, 'response', r.response) ORDER BY r.revision) FROM wonderwilds.story_command_receipts r WHERE r.profile_id = p.id), '[]'::jsonb)
    ) INTO result
    FROM wonderwilds.profiles p WHERE p.id = target AND p.parent_id = wonderwilds.current_parent();
    IF result IS NULL THEN
        RAISE EXCEPTION 'profile unavailable' USING ERRCODE = '42501';
    END IF;
    RETURN result;
END $$;
COMMIT;
