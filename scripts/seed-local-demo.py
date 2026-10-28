"""Issue a synthetic local session using an operator connection, never HTTP."""
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess

root = Path(__file__).resolve().parent.parent
session = secrets.token_hex(32)
csrf = secrets.token_hex(32)
parent = '10000000-0000-0000-0000-000000000001'
profile = '10000000-0000-0000-0000-000000000002'
sql = f"""
DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname='ww_demo_runtime') THEN
CREATE ROLE ww_demo_runtime LOGIN NOSUPERUSER NOBYPASSRLS; END IF; END $$;
GRANT USAGE ON SCHEMA wonderwilds TO ww_demo_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_demo_runtime;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_demo_runtime;
INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject)
VALUES('{parent}','local-synthetic-demo','local-demo') ON CONFLICT DO NOTHING;
INSERT INTO wonderwilds.profiles(id,parent_id,ability_band)
VALUES('{profile}','{parent}','beginning') ON CONFLICT DO NOTHING;
DELETE FROM wonderwilds.sessions WHERE parent_id='{parent}';
INSERT INTO wonderwilds.sessions(token_digest,csrf_digest,parent_id,expires_at,reauthenticated_at)
VALUES(decode('{hashlib.sha256(session.encode()).hexdigest()}','hex'),
decode('{hashlib.sha256(csrf.encode()).hexdigest()}','hex'),'{parent}',now()+interval '24 hours',now());
"""
subprocess.run([os.environ['WONDERWILDS_PG_BIN'] + '/psql',
                '--dbname=' + os.environ['WONDERWILDS_DATABASE_URL'], '-X', '-q', '-v', 'ON_ERROR_STOP=1'],
               input=sql, text=True, check=True)
path = root / '.local-demo/session.json'
path.write_text(json.dumps(dict(session=session, csrf=csrf, profileId=profile,
                               browserToken=secrets.token_hex(32))))
path.chmod(0o600)
