#!/bin/sh
# Serialize migrations, detect edited history, and commit each file with its hash.
set -eu
umask 077
task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_bin=${WONDERWILDS_PG_BIN:-/workspace/setup-tools/postgres/usr/lib/postgresql/17/bin}
export LD_LIBRARY_PATH=/workspace/setup-tools/postgres/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
if [ -n "${WONDERWILDS_DATABASE_URL:-}" ]; then
    task_database=$WONDERWILDS_DATABASE_URL
else
    "$task_root/scripts/start-postgres.sh" >/dev/null
    if [ "$("$task_bin/psql" -h /workspace/setup-tools/pgsocket -p 55432 -d postgres -Atqc "SELECT count(*) FROM pg_database WHERE datname='wonderwilds_dev'")" = 0 ]; then
        "$task_bin/createdb" -h /workspace/setup-tools/pgsocket -p 55432 wonderwilds_dev
    fi
    task_database='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=wonderwilds_dev'
fi
task_sql=$(mktemp /tmp/wonderwilds-migrations.XXXXXX)
trap 'rm -f "$task_sql"' EXIT HUP INT TERM
python3 - "$task_root/backend/migrations" > "$task_sql" <<'PY'
import hashlib
import pathlib
import re
import sys

directory = pathlib.Path(sys.argv[1])
migrations = sorted(path for path in directory.glob('*.sql') if not path.name.endswith('.test.sql'))
if not migrations:
    raise SystemExit('No PostgreSQL migrations found')
print(r'\set ON_ERROR_STOP on')
print('SELECT pg_advisory_lock(812743091);')
print('CREATE TABLE IF NOT EXISTS public.wonderwilds_schema_history (version text PRIMARY KEY, sha256 text NOT NULL CHECK (length(sha256)=64), applied_at timestamptz NOT NULL DEFAULT now());')
print("DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM public.wonderwilds_schema_history) AND to_regclass('wonderwilds.parents') IS NOT NULL THEN RAISE EXCEPTION 'Unmanaged existing Wonderwilds schema: use a fresh development database or explicitly audited bootstrap, never rerun migrations blindly'; END IF; END $$;")
for path in migrations:
    if not re.fullmatch(r'[0-9]{4}_[a-z0-9_]+\.sql', path.name):
        raise SystemExit(f'Unsupported migration filename: {path.name}')
    raw = path.read_bytes()
    text = raw.decode('utf-8')
    # Existing migration files are independently executable transaction scripts.
    # Move only the outer transaction into this runner so history is atomic too.
    match = re.fullmatch(r'\s*BEGIN;\s*(.*?)\s*COMMIT;\s*', text, re.S | re.I)
    if not match:
        raise SystemExit(f'Migration must have one outer BEGIN/COMMIT: {path.name}')
    digest = hashlib.sha256(raw).hexdigest()
    name = path.name
    print(f"DO $$ BEGIN IF EXISTS (SELECT 1 FROM public.wonderwilds_schema_history WHERE version='{name}' AND sha256<>'{digest}') THEN RAISE EXCEPTION 'Migration hash drift: {name}'; END IF; END $$;")
    print(f"SELECT EXISTS (SELECT 1 FROM public.wonderwilds_schema_history WHERE version='{name}') AS applied \\gset")
    print(r'\if :applied')
    print(r'\else')
    print('BEGIN;')
    print(match.group(1))
    print(f"INSERT INTO public.wonderwilds_schema_history(version,sha256) VALUES('{name}','{digest}');")
    print('COMMIT;')
    print(r'\endif')
print('SELECT pg_advisory_unlock(812743091);')
print("SELECT version FROM public.wonderwilds_schema_history ORDER BY version;")
PY
"$task_bin/psql" --dbname="$task_database" --no-psqlrc --set ON_ERROR_STOP=1 --file "$task_sql"
