#!/bin/sh
# Real PostgreSQL checks in a unique disposable database, never the dev database.
set -eu
umask 077
task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ "${1:-}" != '--qa-child' ]; then
    exec python3 "$task_root/scripts/with-qa-postgres.py" sh "$task_root/scripts/check-postgres.sh" --qa-child
fi
# Internal invocation must match the wrapper's generated cluster, never a supplied URL.
[ -n "${WW_QA_CLUSTER_DIR:-}" ] && [ -f "$WW_QA_CLUSTER_DIR/data/postmaster.pid" ]
[ "$WW_QA_DATABASE_URL" = "host=$WW_QA_CLUSTER_DIR/socket port=55432 user=ww_qa_admin dbname=ww_qa" ]
task_bin=$WONDERWILDS_PG_BIN
# The wrapper owns this fresh cluster and removes it even when checks fail.
export WONDERWILDS_DATABASE_URL=$WW_QA_DATABASE_URL
"$task_root/scripts/migrate-postgres.sh"
# A second run must preserve state and succeed from its recorded hashes.
"$task_root/scripts/migrate-postgres.sh"
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --file "$task_root/backend/migrations/0001_parent_profiles.test.sql"
export WW_IDENTITY_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
export WW_GAME_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
export WW_STORY_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
export WW_MISSION_LEARNING_TEST_DATABASE_URL=$WONDERWILDS_DATABASE_URL
cd "$task_root"
cargo test -p wonderwilds-api --locked postgres_identity_ownership_export_revocation_and_delete -- --ignored --nocapture
cargo test -p wonderwilds-api --locked postgres_atomic_rollback_races_unique_grants_and_profile_ownership -- --ignored --nocapture
cargo test -p wonderwilds-api --locked postgres_story_atomicity_retry_ownership_export_and_delete -- --ignored --nocapture
cargo test -p wonderwilds-api --locked postgres_mission_learning_atomicity_retry_ownership_export_and_delete -- --ignored --nocapture
# Prove recorded-history drift fails closed without touching repository files.
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --command "UPDATE public.wonderwilds_schema_history SET sha256=repeat('0',64) WHERE version='0001_parent_profiles.sql'" >/dev/null
task_drift_log=$(mktemp /tmp/wonderwilds-drift.XXXXXX)
if "$task_root/scripts/migrate-postgres.sh" >"$task_drift_log" 2>&1; then
    rm -f "$task_drift_log"
    echo 'Migration history drift was incorrectly accepted' >&2
    exit 1
fi
if ! grep -Fq 'Migration hash drift: 0001_parent_profiles.sql' "$task_drift_log"; then
    cat "$task_drift_log" >&2
    rm -f "$task_drift_log"
    exit 1
fi
rm -f "$task_drift_log"
task_hash=$(python3 -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())' "$task_root/backend/migrations/0001_parent_profiles.sql")
"$task_bin/psql" --dbname="$WONDERWILDS_DATABASE_URL" --no-psqlrc --set ON_ERROR_STOP=1 \
    --command "UPDATE public.wonderwilds_schema_history SET sha256='$task_hash' WHERE version='0001_parent_profiles.sql'" >/dev/null
"$task_root/scripts/migrate-postgres.sh"
