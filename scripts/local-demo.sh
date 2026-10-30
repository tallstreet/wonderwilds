#!/bin/sh
# Isolated synthetic database; no production identity issuer or public login route.
set -eu
umask 077
# Fail before rotating the session if an existing demo occupies either port.
python3 - <<'CHECK_PORTS'
import socket
for port in (3001, 5173):
    with socket.socket() as probe:
        probe.settimeout(0.5)
        if probe.connect_ex(('127.0.0.1', port)) == 0:
            raise SystemExit(f'Port {port} is busy. Stop the existing server before starting the backend demo.')
CHECK_PORTS
task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$task_root"
task_dir="$task_root/.local-demo"
export WONDERWILDS_PG_BIN=${WONDERWILDS_PG_BIN:-/opt/homebrew/opt/postgresql@17/bin}
task_bin=$WONDERWILDS_PG_BIN
mkdir -p "$task_dir/socket"
chmod 700 "$task_dir" "$task_dir/socket"
if [ ! -f "$task_dir/data/PG_VERSION" ]; then
  "$task_bin/initdb" -D "$task_dir/data" --auth-local=trust --auth-host=reject --encoding=UTF8 --no-locale >/dev/null
fi
if ! "$task_bin/pg_ctl" -D "$task_dir/data" status >/dev/null 2>&1; then
  "$task_bin/pg_ctl" -D "$task_dir/data" -l "$task_dir/postgres.log" -o "-k '$task_dir/socket' -h '' -p 55433" start
fi
export WONDERWILDS_DATABASE_URL="host='$task_dir/socket' port=55433 dbname=wonderwilds_demo"
if [ "$("$task_bin/psql" -h "$task_dir/socket" -p 55433 -d postgres -Atqc "SELECT count(*) FROM pg_database WHERE datname='wonderwilds_demo'")" = 0 ]; then
  "$task_bin/createdb" -h "$task_dir/socket" -p 55433 wonderwilds_demo
fi
sh scripts/migrate-postgres.sh > "$task_dir/migrations.log"
python3 scripts/seed-local-demo.py
export WONDERWILDS_DATABASE_URL="$WONDERWILDS_DATABASE_URL user=ww_demo_runtime"
export WONDERWILDS_ORIGIN=https://wonderwilds.local
export WW_LOCAL_DEMO_CONFIG="$task_dir/session.json"
cargo build -p wonderwilds-api --locked
./target/debug/wonderwilds-api > "$task_dir/api.log" 2>&1 &
task_api=$!
trap 'kill "$task_api" ${task_vite:-} 2>/dev/null || true' EXIT
trap 'exit 0' INT TERM
task_attempt=0
until curl -fsS http://127.0.0.1:3001/readyz >/dev/null; do
  task_attempt=$((task_attempt + 1))
  if [ "$task_attempt" -ge 30 ] || ! kill -0 "$task_api" 2>/dev/null; then cat "$task_dir/api.log"; exit 1; fi
  sleep 1
done
node frontend/node_modules/vite/bin/vite.js frontend --host 127.0.0.1 --port 5173 --strictPort &
task_vite=$!
echo 'Play Wonderwilds: http://127.0.0.1:5173/'
wait "$task_vite"
