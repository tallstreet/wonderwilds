#!/usr/bin/env python3
"""Explicit operator creation of one actual profile in the isolated synthetic demo."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import uuid

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--create-player', action='store_true', required=True)
parser.add_argument('--ability-band', choices=['beginning', 'growing', 'stretching'], default='beginning')
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
demo = root / '.local-demo'
if not (demo / 'data/PG_VERSION').is_file() or not (demo / 'session.json').is_file():
    parser.error('Start the isolated local demo first with sh scripts/local-demo.sh.')
config = json.loads((demo / 'session.json').read_text())
selected = str(uuid.UUID(config['profileId']))
profile = str(uuid.uuid4())
pg_bin = os.environ.get('WONDERWILDS_PG_BIN', '/opt/homebrew/opt/postgresql@17/bin')
psql = str(Path(pg_bin) / 'psql')
if not Path(psql).is_file():
    psql = shutil.which('psql') or ''
if not psql:
    parser.error('Set WONDERWILDS_PG_BIN to the PostgreSQL binaries used by the local demo.')
# The destination is fixed; no environment database URL or remote host is accepted.
sql = f"""BEGIN;
DO $$ DECLARE household uuid; BEGIN
 SELECT p.parent_id INTO household FROM wonderwilds.profiles p
 JOIN wonderwilds.parents a ON a.id=p.parent_id
 WHERE p.id='{selected}' AND a.identity_issuer='local-synthetic-demo' AND a.identity_subject='local-demo';
 IF household IS NULL THEN RAISE EXCEPTION 'expected isolated synthetic household unavailable'; END IF;
 PERFORM set_config('app.parent_id',household::text,true);
 INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES('{profile}',household,'{args.ability_band}');
END $$;
COMMIT;
"""
subprocess.run([psql, '-h', str(demo / 'socket'), '-p', '55433', '-d', 'wonderwilds_demo', '-X', '-q', '-v', 'ON_ERROR_STOP=1'], input=sql, text=True, check=True)
print(f'Created independent synthetic player {profile}. Reload the app and select that player; progress starts fresh.')
