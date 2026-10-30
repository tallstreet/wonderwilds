BEGIN;
-- Original responses and replay inputs live outside bounded working snapshots.
CREATE TABLE wonderwilds.command_receipts (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 command_id text NOT NULL, revision bigint NOT NULL CHECK(revision > 0),
 request jsonb NOT NULL, response jsonb NOT NULL, replay jsonb NOT NULL,
 digest text NOT NULL CHECK(length(digest)=64),
 PRIMARY KEY(profile_id,command_id), UNIQUE(profile_id,revision)
);
CREATE TABLE wonderwilds.domain_command_receipts (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 command_id text NOT NULL, record jsonb NOT NULL,
 PRIMARY KEY(profile_id,command_id)
);
CREATE TABLE wonderwilds.game_encounters (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 encounter_id text NOT NULL, encounter jsonb NOT NULL,
 PRIMARY KEY(profile_id,encounter_id)
);
CREATE TABLE wonderwilds.game_checkpoints (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 revision bigint NOT NULL CHECK(revision >= 0), snapshot jsonb NOT NULL,
 digest text NOT NULL CHECK(length(digest)=64),
 PRIMARY KEY(profile_id,revision)
);
CREATE TABLE wonderwilds.story_checkpoints (LIKE wonderwilds.game_checkpoints INCLUDING ALL);
ALTER TABLE wonderwilds.story_checkpoints ADD FOREIGN KEY(profile_id) REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE;
CREATE TABLE wonderwilds.persistence_origins (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 kind text NOT NULL CHECK(kind IN ('game','story')), snapshot jsonb NOT NULL,
 digest text NOT NULL CHECK(length(digest)=64),
 PRIMARY KEY(profile_id,kind)
);
CREATE TABLE wonderwilds.command_windows (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 kind text NOT NULL CHECK(kind IN ('game','story')),
 window_minute bigint NOT NULL, commands integer NOT NULL CHECK(commands BETWEEN 1 AND 4096),
 PRIMARY KEY(profile_id,kind)
);
ALTER TABLE wonderwilds.story_command_receipts ADD COLUMN digest text CHECK(length(digest)=64);
ALTER TABLE wonderwilds.learning_events ADD COLUMN reconstruction jsonb;
ALTER TABLE wonderwilds.story_command_receipts ADD COLUMN domain_command jsonb;
ALTER TABLE wonderwilds.story_progress DROP CONSTRAINT story_progress_revision_check;
ALTER TABLE wonderwilds.story_progress ADD CHECK(revision BETWEEN 1 AND 1000000000);
ALTER TABLE wonderwilds.story_command_receipts DROP CONSTRAINT story_command_receipts_revision_check;
ALTER TABLE wonderwilds.story_command_receipts ADD CHECK(revision BETWEEN 1 AND 1000000000);
ALTER TABLE wonderwilds.story_mission_plans DROP CONSTRAINT story_mission_plans_accepted_revision_check;
ALTER TABLE wonderwilds.story_mission_plans ADD CHECK(accepted_revision BETWEEN 1 AND 1000000000);

DO $$ DECLARE name text; BEGIN
 FOREACH name IN ARRAY ARRAY['command_receipts','domain_command_receipts','game_encounters','game_checkpoints','story_checkpoints','persistence_origins','command_windows'] LOOP
  EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY',name);
  EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY',name);
  EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent()))',name);
  EXECUTE format('REVOKE ALL ON wonderwilds.%I FROM PUBLIC',name);
  IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='ww_demo_runtime') THEN
   EXECUTE format('GRANT SELECT,INSERT,UPDATE,DELETE ON wonderwilds.%I TO ww_demo_runtime',name);
  END IF;
 END LOOP;
END $$;
-- Preserve previous fields while exporting the durable records needed to rebuild
-- either state system. Every subquery still runs with the caller's forced RLS.
ALTER FUNCTION wonderwilds.export_profile(uuid) RENAME TO export_profile_v4;
CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb; name text; records jsonb;
BEGIN
 result := wonderwilds.export_profile_v4(target);
 FOREACH name IN ARRAY ARRAY['command_receipts','domain_command_receipts','game_encounters','game_checkpoints','story_checkpoints','persistence_origins','game_events','reward_grants'] LOOP
  EXECUTE format('SELECT coalesce(jsonb_agg(to_jsonb(t)),''[]''::jsonb) FROM wonderwilds.%I t WHERE profile_id=$1', name) INTO records USING target;
  result := result || jsonb_build_object(name,records);
 END LOOP;
 SELECT coalesce(jsonb_agg(jsonb_build_object('sequence',sequence,'reconstruction',reconstruction) ORDER BY sequence),'[]'::jsonb) INTO records FROM wonderwilds.learning_events WHERE profile_id=target;
 result := result || jsonb_build_object('learningReconstruction',records);
 SELECT coalesce(jsonb_agg(jsonb_build_object('revision',revision,'command',domain_command) ORDER BY revision),'[]'::jsonb) INTO records FROM wonderwilds.story_command_receipts WHERE profile_id=target;
 RETURN result || jsonb_build_object('storyReplay',records);
END $$;
COMMIT;
