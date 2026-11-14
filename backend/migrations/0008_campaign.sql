BEGIN;
CREATE TABLE wonderwilds.campaign_progress (
 profile_id uuid PRIMARY KEY REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 revision bigint NOT NULL CHECK(revision BETWEEN 1 AND 1000000000),
 checkpoint jsonb NOT NULL CHECK(jsonb_typeof(checkpoint)='object' AND octet_length(checkpoint::text)<=262144)
);
CREATE TABLE wonderwilds.campaign_command_receipts (
 profile_id uuid NOT NULL REFERENCES wonderwilds.profiles(id) ON DELETE CASCADE,
 command_id text NOT NULL CHECK(length(command_id) BETWEEN 1 AND 120),
 revision bigint NOT NULL CHECK(revision BETWEEN 1 AND 1000000000),
 request jsonb NOT NULL CHECK(octet_length(request::text)<=32768),
 response jsonb NOT NULL CHECK(octet_length(response::text)<=65536),
 digest text NOT NULL CHECK(length(digest)=64),
 PRIMARY KEY(profile_id,command_id),UNIQUE(profile_id,revision)
);
DO $$ DECLARE name text; BEGIN
 FOREACH name IN ARRAY ARRAY['campaign_progress','campaign_command_receipts'] LOOP
 EXECUTE format('ALTER TABLE wonderwilds.%I ENABLE ROW LEVEL SECURITY',name);
 EXECUTE format('ALTER TABLE wonderwilds.%I FORCE ROW LEVEL SECURITY',name);
 EXECUTE format('CREATE POLICY profile_owned ON wonderwilds.%I USING (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent())) WITH CHECK (EXISTS (SELECT 1 FROM wonderwilds.profiles p WHERE p.id=profile_id AND p.parent_id=wonderwilds.current_parent()))',name);
 EXECUTE format('REVOKE ALL ON wonderwilds.%I FROM PUBLIC',name);
 END LOOP;
 IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='ww_demo_runtime') THEN
 GRANT SELECT,INSERT,UPDATE ON wonderwilds.campaign_progress TO ww_demo_runtime;
 GRANT SELECT,INSERT ON wonderwilds.campaign_command_receipts TO ww_demo_runtime;
 END IF;
END $$;
ALTER FUNCTION wonderwilds.export_profile(uuid) RENAME TO export_profile_v7;
CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$
DECLARE result jsonb; BEGIN
 result:=wonderwilds.export_profile_v7(target);
 RETURN result || jsonb_build_object(
 'campaignProgress',coalesce((SELECT checkpoint FROM wonderwilds.campaign_progress WHERE profile_id=target),'{}'::jsonb),
 'campaignReceipts',coalesce((SELECT jsonb_agg(jsonb_build_object('commandId',command_id,'revision',revision,'request',request,'response',response) ORDER BY revision) FROM wonderwilds.campaign_command_receipts WHERE profile_id=target),'[]'::jsonb));
END $$;
COMMIT;
