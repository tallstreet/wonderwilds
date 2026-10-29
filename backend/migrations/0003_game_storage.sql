BEGIN;
-- Nullable only for legacy aggregates. New compact saves require a checked digest.
ALTER TABLE wonderwilds.progress ADD COLUMN snapshot_digest text
 CHECK (snapshot_digest IS NULL OR snapshot_digest ~ '^[0-9a-f]{64}$');
ALTER TABLE wonderwilds.progress ADD COLUMN rate_window_start bigint NOT NULL DEFAULT 0 CHECK (rate_window_start >= 0);
ALTER TABLE wonderwilds.progress ADD COLUMN rate_window_count integer NOT NULL DEFAULT 0 CHECK (rate_window_count BETWEEN 0 AND 120);

CREATE TABLE wonderwilds.command_receipts (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 command_id text NOT NULL CHECK (length(command_id) BETWEEN 1 AND 120),
 request jsonb NOT NULL CHECK (jsonb_typeof(request) = 'object'),
 response jsonb NOT NULL CHECK (jsonb_typeof(response) = 'object'),
 PRIMARY KEY (profile_id, command_id)
);
CREATE TABLE wonderwilds.domain_command_receipts (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 command_id text NOT NULL CHECK (length(command_id) BETWEEN 1 AND 120),
 receipt jsonb NOT NULL CHECK (jsonb_typeof(receipt) = 'object'),
 replay_entry jsonb CHECK (replay_entry IS NULL OR jsonb_typeof(replay_entry) = 'object'),
 PRIMARY KEY (profile_id, command_id)
);
CREATE TABLE wonderwilds.game_encounters (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 encounter_id text NOT NULL CHECK (length(encounter_id) BETWEEN 1 AND 120),
 encounter jsonb NOT NULL CHECK (jsonb_typeof(encounter) = 'object'),
 PRIMARY KEY (profile_id, encounter_id)
);
CREATE TABLE wonderwilds.game_checkpoints (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 revision bigint NOT NULL CHECK (revision >= 0),
 snapshot jsonb NOT NULL CHECK (jsonb_typeof(snapshot) = 'object'),
 snapshot_digest text NOT NULL CHECK (snapshot_digest ~ '^[0-9a-f]{64}$'),
 PRIMARY KEY (profile_id, revision)
);
CREATE TABLE wonderwilds.game_mastery_items (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 skill_id text NOT NULL CHECK (length(skill_id) BETWEEN 1 AND 120),
 item_family text NOT NULL CHECK (length(item_family) BETWEEN 1 AND 120),
 parameter_fingerprint text NOT NULL CHECK (length(parameter_fingerprint) BETWEEN 1 AND 120),
 PRIMARY KEY (profile_id, skill_id, item_family, parameter_fingerprint)
);

DO $$ DECLARE table_name text; BEGIN
 FOREACH table_name IN ARRAY ARRAY['command_receipts','domain_command_receipts','game_encounters','game_checkpoints','game_mastery_items'] LOOP
  EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY',table_name);
  EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY',table_name);
  EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent()))',table_name);
 END LOOP;
END $$;
REVOKE ALL ON wonderwilds.command_receipts,wonderwilds.domain_command_receipts,wonderwilds.game_encounters,wonderwilds.game_checkpoints,wonderwilds.game_mastery_items FROM PUBLIC;

-- Parent exports retain durable replay and encounter context after compaction.
-- Secrets are not present in these game records; session tokens stay in sessions.
CREATE OR REPLACE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb;
BEGIN
    SELECT jsonb_build_object(
        'schemaVersion', '1.0',
        'profile', jsonb_build_object('id', p.id, 'abilityBand', p.ability_band, 'revision', p.revision),
        'progress', coalesce((SELECT snapshot FROM wonderwilds.progress WHERE profile_id = p.id), '{}'::jsonb),
        'learningEvents', coalesce((SELECT jsonb_agg(jsonb_build_object('id', e.id, 'sequence', e.sequence, 'evidence', e.evidence, 'recordedAt', e.recorded_at) ORDER BY e.sequence) FROM wonderwilds.learning_events e WHERE e.profile_id = p.id), '[]'::jsonb),
        'consent', coalesce((SELECT jsonb_agg(jsonb_build_object('noticeVersion', c.notice_version, 'purpose', c.purpose, 'recordedAt', c.recorded_at, 'revokedAt', c.revoked_at) ORDER BY c.recorded_at, c.id) FROM wonderwilds.consent_records c WHERE c.profile_id = p.id), '[]'::jsonb),
        'commandReceipts', coalesce((SELECT jsonb_agg(jsonb_build_object('commandId', c.command_id, 'request', c.request, 'response', c.response) ORDER BY c.command_id) FROM wonderwilds.command_receipts c WHERE c.profile_id=p.id), '[]'::jsonb),
        'domainCommandReceipts', coalesce((SELECT jsonb_agg(jsonb_build_object('receipt',c.receipt,'replayEntry',c.replay_entry) ORDER BY c.command_id) FROM wonderwilds.domain_command_receipts c WHERE c.profile_id=p.id), '[]'::jsonb),
        'encounters', coalesce((SELECT jsonb_agg(e.encounter ORDER BY e.encounter_id) FROM wonderwilds.game_encounters e WHERE e.profile_id=p.id), '[]'::jsonb),
        'gameEvents', coalesce((SELECT jsonb_agg(e.event ORDER BY e.sequence) FROM wonderwilds.game_events e WHERE e.profile_id=p.id), '[]'::jsonb),
        'rewardGrants', coalesce((SELECT jsonb_agg(jsonb_build_object('rewardDefinition', r.reward_definition, 'questInstance', r.quest_instance) ORDER BY r.reward_definition, r.quest_instance) FROM wonderwilds.reward_grants r WHERE r.profile_id=p.id), '[]'::jsonb),
        'checkpoints', coalesce((SELECT jsonb_agg(jsonb_build_object('revision', c.revision, 'snapshot', c.snapshot, 'digest', c.snapshot_digest) ORDER BY c.revision) FROM wonderwilds.game_checkpoints c WHERE c.profile_id=p.id), '[]'::jsonb),
        'masteryItems', coalesce((SELECT jsonb_agg(jsonb_build_object('skillId', m.skill_id, 'itemFamily', m.item_family, 'parameterFingerprint', m.parameter_fingerprint) ORDER BY m.skill_id,m.item_family,m.parameter_fingerprint) FROM wonderwilds.game_mastery_items m WHERE m.profile_id=p.id), '[]'::jsonb)
    ) INTO result
    FROM wonderwilds.profiles p WHERE p.id = target AND p.parent_id = wonderwilds.current_parent();
    IF result IS NULL THEN
        RAISE EXCEPTION 'profile unavailable' USING ERRCODE = '42501';
    END IF;
    RETURN result;
END $$;
REVOKE ALL ON FUNCTION wonderwilds.export_profile(uuid) FROM PUBLIC;
COMMIT;
