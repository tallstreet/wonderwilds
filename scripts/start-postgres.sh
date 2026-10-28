#!/bin/sh
# Local synthetic development cluster only: private Unix socket, no TCP listener.
set -eu
umask 077
task_pg=/workspace/setup-tools/postgres
task_data=/workspace/setup-tools/pgdata
task_socket=/workspace/setup-tools/pgsocket
task_bin=$task_pg/usr/lib/postgresql/17/bin
export LD_LIBRARY_PATH=$task_pg/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
mkdir -p "$task_socket"
chmod 700 "$task_socket"
if [ ! -f "$task_data/PG_VERSION" ]; then
    "$task_bin/initdb" -D "$task_data" --auth-local=trust --auth-host=reject --no-locale --encoding=UTF8
fi
if ! "$task_bin/pg_ctl" -D "$task_data" status >/dev/null 2>&1; then
    "$task_bin/pg_ctl" -D "$task_data" -l "$task_data/server.log" \
        -o "-k $task_socket -h '' -p 55432" start
fi
task_encoding=$("$task_bin/psql" -h "$task_socket" -p 55432 -d postgres -Atqc 'SHOW server_encoding')
if [ "$task_encoding" != UTF8 ]; then
    echo 'Existing local cluster is not UTF8; preserve it and migrate before continuing.' >&2
    exit 1
fi
if [ "$("$task_bin/psql" -h "$task_socket" -p 55432 -d postgres -Atqc "SELECT count(*) FROM pg_database WHERE datname='identity_test'")" = 0 ]; then
    "$task_bin/createdb" -h "$task_socket" -p 55432 identity_test
fi
"$task_bin/psql" -h "$task_socket" -p 55432 -d identity_test -v ON_ERROR_STOP=1 -Atqc 'SELECT 1'
