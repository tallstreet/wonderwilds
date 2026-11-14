BEGIN;
ALTER TABLE wonderwilds.command_windows DROP CONSTRAINT command_windows_kind_check;
ALTER TABLE wonderwilds.command_windows ADD CHECK(kind IN ('game','story','mission_learning','paired'));
CREATE TABLE wonderwilds.paired_progress(
 parent_id uuid PRIMARY KEY REFERENCES wonderwilds.parents(id) ON DELETE CASCADE,
 revision bigint NOT NULL CHECK(revision BETWEEN 1 AND 1000000000),
 navigator_profile_id uuid REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 keeper_profile_id uuid REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 checkpoint jsonb NOT NULL CHECK(octet_length(checkpoint::text)<=32768),
 CHECK((navigator_profile_id IS NULL)=(keeper_profile_id IS NULL)),
 CHECK(navigator_profile_id IS DISTINCT FROM keeper_profile_id OR navigator_profile_id IS NULL)
);
CREATE TABLE wonderwilds.paired_command_receipts(
 parent_id uuid NOT NULL REFERENCES wonderwilds.parents(id) ON DELETE CASCADE,
 command_id text NOT NULL CHECK(length(command_id) BETWEEN 1 AND 120),
 revision bigint NOT NULL CHECK(revision BETWEEN 1 AND 1000000000),
 actor_profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 navigator_profile_id uuid REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 keeper_profile_id uuid REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 domain_state jsonb NOT NULL CHECK(octet_length(domain_state::text)<=32768),
 request jsonb NOT NULL CHECK(octet_length(request::text)<=32768),
 response jsonb NOT NULL CHECK(octet_length(response::text)<=65536),
 digest text NOT NULL CHECK(length(digest)=64),
 PRIMARY KEY(parent_id,command_id),UNIQUE(parent_id,revision)
);
CREATE TABLE wonderwilds.paired_keepsakes(
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 source_profile_id uuid REFERENCES wonderwilds.profiles(id) ON DELETE SET NULL,
 recorded_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 keepsake_id text NOT NULL CHECK(length(keepsake_id) BETWEEN 1 AND 120),
 UNIQUE(profile_id,source_profile_id,keepsake_id)
);
DO $$ DECLARE name text; BEGIN
 FOREACH name IN ARRAY ARRAY['paired_progress','paired_command_receipts'] LOOP
 EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY',name);
 EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY',name);
 EXECUTE format('CREATE POLICY household_owned ON wonderwilds.%I USING (parent_id=wonderwilds.current_parent()) WITH CHECK (parent_id=wonderwilds.current_parent() AND (navigator_profile_id IS NULL OR EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=navigator_profile_id AND p.parent_id=wonderwilds.current_parent())) AND (keeper_profile_id IS NULL OR EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=keeper_profile_id AND p.parent_id=wonderwilds.current_parent())))',name);
 EXECUTE format('REVOKE ALL ON wonderwilds.%I FROM PUBLIC',name);
 END LOOP;
 CREATE POLICY actor_owned ON wonderwilds.paired_command_receipts AS RESTRICTIVE USING(EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=actor_profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK(EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=actor_profile_id AND p.parent_id=wonderwilds.current_parent()));
 ALTER TABLE wonderwilds.paired_keepsakes ENABLE ROW LEVEL SECURITY;
 ALTER TABLE wonderwilds.paired_keepsakes FORCE ROW LEVEL SECURITY;
 CREATE POLICY profile_owned ON wonderwilds.paired_keepsakes USING(EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK(EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent()) AND EXISTS(SELECT 1 FROM wonderwilds.profiles p WHERE p.id=source_profile_id AND p.parent_id=wonderwilds.current_parent()));
 REVOKE ALL ON wonderwilds.paired_keepsakes FROM PUBLIC;
 IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='ww_demo_runtime') THEN
 GRANT SELECT,INSERT,UPDATE ON wonderwilds.paired_progress TO ww_demo_runtime;
 GRANT SELECT,INSERT ON wonderwilds.paired_command_receipts,wonderwilds.paired_keepsakes TO ww_demo_runtime;
 END IF;
END $$;
ALTER FUNCTION wonderwilds.export_profile(uuid) RENAME TO export_profile_v8;
CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$ DECLARE result jsonb; BEGIN
 result:=wonderwilds.export_profile_v8(target);
 RETURN result||jsonb_build_object(
 'pairedProgress',coalesce((SELECT checkpoint FROM wonderwilds.paired_progress WHERE navigator_profile_id=target OR keeper_profile_id=target),'{}'::jsonb),
 'pairedReceipts',coalesce((SELECT jsonb_agg(jsonb_build_object('commandId',command_id,'revision',revision,'actorProfileId',actor_profile_id,'request',request,'response',response) ORDER BY revision) FROM wonderwilds.paired_command_receipts WHERE actor_profile_id=target OR navigator_profile_id=target OR keeper_profile_id=target),'[]'::jsonb),
 'sharedKeepsakeCopies',coalesce((SELECT jsonb_agg(jsonb_build_object('keepsakeId',keepsake_id,'sourceProfileId',source_profile_id)) FROM wonderwilds.paired_keepsakes WHERE profile_id=target),'[]'::jsonb));
END $$;
COMMIT;
