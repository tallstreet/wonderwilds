#!/usr/bin/env python3
"""Run a command against a fresh disposable PostgreSQL cluster, never demo data."""
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile


def postgres_bin():
    configured = os.environ.get('WONDERWILDS_PG_BIN')
    if configured:
        return Path(configured)
    for candidate in (
        '/workspace/setup-tools/postgres/usr/lib/postgresql/17/bin',
        '/opt/homebrew/opt/postgresql@17/bin',
        '/usr/local/opt/postgresql@17/bin',
    ):
        if Path(candidate, 'initdb').is_file():
            return Path(candidate)
    if shutil.which('pg_config'):
        return Path(subprocess.check_output(['pg_config', '--bindir'], text=True).strip())
    raise SystemExit('Install PostgreSQL or set WONDERWILDS_PG_BIN to its bin directory')


def interrupted(signum, _frame):
    raise SystemExit(128 + signum)


def main():
    if len(sys.argv) < 2:
        raise SystemExit('Usage: python3 scripts/with-qa-postgres.py COMMAND [ARG ...]')
    binary = postgres_bin()
    for name in ('initdb', 'pg_ctl', 'createdb', 'psql'):
        if not (binary / name).is_file():
            raise SystemExit(f'Missing PostgreSQL executable: {binary / name}')
    # Short socket paths also work with macOS Unix socket path limits.
    temporary = Path(tempfile.mkdtemp(prefix='ww-qa-', dir='/tmp'))
    data, socket = temporary / 'data', temporary / 'socket'
    socket.mkdir(mode=0o700)
    env = dict(os.environ, WONDERWILDS_PG_BIN=str(binary), PGHOST=str(socket),
               PGPORT='55432', PGUSER='ww_qa_admin', PGDATABASE='ww_qa')
    bundled_lib = Path('/workspace/setup-tools/postgres/usr/lib/x86_64-linux-gnu')
    if bundled_lib.is_dir():
        env['LD_LIBRARY_PATH'] = str(bundled_lib) + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else '')
    database = f'host={socket} port=55432 user=ww_qa_admin dbname=ww_qa'
    env.update(WW_QA_DATABASE_URL=database, WONDERWILDS_DATABASE_URL=database,
               WW_QA_CLUSTER_DIR=str(temporary), TMPDIR=str(temporary))
    child = None
    started = False
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, interrupted)
    try:
        subprocess.run([str(binary / 'initdb'), '-D', str(data), '-U', 'ww_qa_admin',
                        '--auth-local=trust', '--auth-host=reject', '--no-locale', '--encoding=UTF8'],
                       env=env, check=True, stdout=subprocess.DEVNULL)
        subprocess.run([str(binary / 'pg_ctl'), '-D', str(data), '-l', str(temporary / 'postgres.log'),
                        '-o', f"-k {socket} -h '' -p 55432", '-w', 'start'], env=env, check=True)
        started = True
        subprocess.run([str(binary / 'createdb'), 'ww_qa'], env=env, check=True)
        child = subprocess.Popen(sys.argv[1:], env=env, start_new_session=True)
        return child.wait()
    finally:
        # Stop descendants (API/Vite/browser) before removing their database.
        if child is not None:
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        if started or (data / 'postmaster.pid').exists():
            stopped = subprocess.run([str(binary / 'pg_ctl'), '-D', str(data), '-m', 'immediate', '-w', 'stop'], env=env)
            if stopped.returncode:
                print(f'Could not stop QA cluster; preserved {temporary}', file=sys.stderr)
                raise SystemExit(stopped.returncode)
        shutil.rmtree(temporary)


if __name__ == '__main__':
    sys.exit(main())
