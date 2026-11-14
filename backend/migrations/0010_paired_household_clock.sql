BEGIN;
-- Household sequence survives participant deletion. No child profile FK belongs
-- on this counter; parent deletion still removes it.
CREATE TABLE wonderwilds.paired_household_clock(
 parent_id uuid PRIMARY KEY REFERENCES wonderwilds.parents(id) ON DELETE CASCADE,
 revision bigint NOT NULL CHECK(revision BETWEEN 0 AND 1000000000)
);
INSERT INTO wonderwilds.paired_household_clock(parent_id,revision)
 SELECT p.id,greatest(coalesce(s.revision,0),coalesce((SELECT max(r.revision) FROM wonderwilds.paired_command_receipts r WHERE r.parent_id=p.id),0))
 FROM wonderwilds.parents p LEFT JOIN wonderwilds.paired_progress s ON s.parent_id=p.id;
ALTER TABLE wonderwilds.paired_household_clock ENABLE ROW LEVEL SECURITY;
ALTER TABLE wonderwilds.paired_household_clock FORCE ROW LEVEL SECURITY;
CREATE POLICY household_owned ON wonderwilds.paired_household_clock
 USING(parent_id=wonderwilds.current_parent()) WITH CHECK(parent_id=wonderwilds.current_parent());
REVOKE ALL ON wonderwilds.paired_household_clock FROM PUBLIC;
DO $$ BEGIN IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='ww_demo_runtime') THEN
 GRANT SELECT,INSERT,UPDATE ON wonderwilds.paired_household_clock TO ww_demo_runtime;
END IF; END $$;
ALTER FUNCTION wonderwilds.export_profile(uuid) RENAME TO export_profile_v9;
CREATE FUNCTION wonderwilds.export_profile(target uuid) RETURNS jsonb LANGUAGE plpgsql AS $$ DECLARE result jsonb; BEGIN
 result:=wonderwilds.export_profile_v9(target);
 RETURN result||jsonb_build_object('pairedHouseholdRevision',coalesce((SELECT c.revision FROM wonderwilds.paired_household_clock c JOIN wonderwilds.profiles p ON p.parent_id=c.parent_id WHERE p.id=target),0));
END $$;
COMMIT;
