#!/bin/sh
# Real PostgreSQL checks in a unique disposable database, never the dev database.
set -eu
umask 077
task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_bin=/workspace/setup-tools/postgres/usr/lib/postgresql/17/bin
export LD_LIBRARY_PATH=/workspace/setup-tools/postgres/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH=/workspace/.cargo/bin:$PATH
"$task_root/scripts/start-postgres.sh" >/dev/null
# Test roles are cluster-scoped, so serialize this complete check helper locally.
exec 9>/workspace/setup-tools/pg-check.lock
flock 9
task_name=ww_check_$(date +%s)_$$
"$task_bin/createdb" -h /workspace/setup-tools/pgsocket -p 55432 "$task_name"
cleanup() {
    task_exit=$?
    trap - EXIT HUP INT TERM
    "$task_bin/dropdb" -h /workspace/setup-tools/pgsocket -p 55432 "$task_name" || task_exit=1
    exit "$task_exit"
}
trap cleanup EXIT HUP INT TERM
export WONDERWILDS_DATABASE_URL="host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=$task_name"
"$task_root/scripts/migrate-postgres.sh"
# A second run must preserve state and succeed from its recorded hashes.
"$task_root/scripts/migrate-postgres.sh"
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --file "$task_root/backend/migrations/0001_parent_profiles.test.sql"
export WW_IDENTITY_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
export WW_GAME_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
cd "$task_root"
cargo test -p wonderwilds-api --locked postgres_identity_ownership_export_revocation_and_delete -- --ignored --nocapture
cargo test -p wonderwilds-api --locked postgres_atomic_rollback_races_unique_grants_and_profile_ownership -- --ignored --nocapture
cargo test -p wonderwilds-api --locked postgres_durable_storage_migration_checkpoints_and_long_profile -- --ignored --nocapture
# Prove recorded-history drift fails closed without touching repository files.
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --command "UPDATE public.wonderwilds_schema_history SET sha256=repeat('0',64) WHERE version='0001_parent_profiles.sql'" >/dev/null
task_drift_log=$(mktemp /tmp/wonderwilds-drift.XXXXXX)
if "$task_root/scripts/migrate-postgres.sh" >"$task_drift_log" 2>&1; then
    rm -f "$task_drift_log"
    echo 'Migration history drift was incorrectly accepted' >&2
    exit 1
fi
if ! rg -q 'Migration hash drift: 0001_parent_profiles.sql' "$task_drift_log"; then
    cat "$task_drift_log" >&2
    rm -f "$task_drift_log"
    exit 1
fi
rm -f "$task_drift_log"
task_hash=$(sha256sum "$task_root/backend/migrations/0001_parent_profiles.sql" | cut -d ' ' -f 1)
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --command "UPDATE public.wonderwilds_schema_history SET sha256='$task_hash' WHERE version='0001_parent_profiles.sql'" >/dev/null
"$task_root/scripts/migrate-postgres.sh"
