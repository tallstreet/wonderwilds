BEGIN;
-- Immutable accepted model briefs, separate from deterministic gameplay checkpoints.
CREATE TABLE wonderwilds.story_mission_plans (
    profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
    content_version text NOT NULL CHECK (length(content_version) BETWEEN 1 AND 80),
    mission_id text NOT NULL CHECK (length(mission_id) BETWEEN 1 AND 80),
    accepted_revision bigint NOT NULL CHECK (accepted_revision BETWEEN 1 AND 256),
    plan jsonb NOT NULL CHECK (jsonb_typeof(plan) = 'object' AND octet_length(plan::text) <= 24576),
    PRIMARY KEY (profile_id, content_version, mission_id)
);
ALTER TABLE wonderwilds.story_mission_plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE wonderwilds.story_mission_plans FORCE ROW LEVEL SECURITY;
CREATE POLICY profile_owned ON wonderwilds.story_mission_plans
    USING (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent()))
    WITH CHECK (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id = profile_id AND p.parent_id = wonderwilds.current_parent()));
REVOKE ALL ON wonderwilds.story_mission_plans FROM PUBLIC;

CREATE OR REPLACE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb;
BEGIN
    SELECT jsonb_build_object(
        'schemaVersion', '1.0',
        'profile', jsonb_build_object('id', p.id, 'abilityBand', p.ability_band, 'revision', p.revision),
        'progress', coalesce((SELECT snapshot FROM wonderwilds.progress WHERE profile_id = p.id), '{}'::jsonb),
        'learningEvents', coalesce((SELECT jsonb_agg(jsonb_build_object('id', e.id, 'sequence', e.sequence, 'evidence', e.evidence, 'recordedAt', e.recorded_at) ORDER BY e.sequence) FROM wonderwilds.learning_events e WHERE e.profile_id = p.id), '[]'::jsonb),
        'consent', coalesce((SELECT jsonb_agg(jsonb_build_object('noticeVersion', c.notice_version, 'purpose', c.purpose, 'recordedAt', c.recorded_at, 'revokedAt', c.revoked_at) ORDER BY c.recorded_at, c.id) FROM wonderwilds.consent_records c WHERE c.profile_id = p.id), '[]'::jsonb),
        'storyMissionPlans', coalesce((SELECT jsonb_agg(jsonb_build_object('contentVersion', m.content_version, 'missionId', m.mission_id, 'acceptedRevision', m.accepted_revision, 'plan', m.plan) ORDER BY m.accepted_revision) FROM wonderwilds.story_mission_plans m WHERE m.profile_id = p.id), '[]'::jsonb),
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
