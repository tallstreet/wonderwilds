BEGIN;
CREATE TABLE wonderwilds.game_events (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 sequence bigint NOT NULL CHECK(sequence > 0), event jsonb NOT NULL CHECK(jsonb_typeof(event)='object'),
 PRIMARY KEY(profile_id,sequence)
);
CREATE TABLE wonderwilds.reward_grants (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 reward_definition text NOT NULL, quest_instance text NOT NULL,
 PRIMARY KEY(profile_id,reward_definition,quest_instance)
);
CREATE TABLE wonderwilds.game_outbox (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 sequence bigint NOT NULL, event jsonb NOT NULL CHECK(jsonb_typeof(event)='object'),
 processed_at timestamptz, PRIMARY KEY(profile_id,sequence)
);
DO $$ DECLARE table_name text; BEGIN
 FOREACH table_name IN ARRAY ARRAY['game_events','reward_grants','game_outbox'] LOOP
  EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY',table_name);
  EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY',table_name);
  EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK (EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent()))',table_name);
 END LOOP;
END $$;
REVOKE ALL ON wonderwilds.game_events,wonderwilds.reward_grants,wonderwilds.game_outbox FROM PUBLIC;
COMMIT;
