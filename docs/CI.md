# Repository checks in GitHub Actions

`.github/workflows/checks.yml` runs on pull requests, pushes to `main` and manual
dispatch. It grants only read access to repository contents and does not deploy,
publish artifacts or use application credentials.

The Rust job installs the repository-pinned toolchain and runs formatting, Clippy,
and workspace tests. The web/content job uses Node 24, Python 3.13 and locked npm
installs. It checks contracts, generated-code drift, content/art validators,
frontend types/lint/format/unit tests/build and legacy guest-save fixtures.
`npm test` in `frontend/` discovers nested tests, including the API client tests.

The integration matrix adds separate database and browser jobs. Both install
PostgreSQL 17; the browser job also installs Playwright Chromium. Database checks
exercise migrations, RLS/ownership/deletion, rollback, races and migration hash
drift. Browser checks cover the guest world, activity controls and server-backed
training journey. These jobs are configured; hosted execution must pass before
claiming hosted-runner validation.

## Disposable local integration checks

Run `sh scripts/check-postgres.sh` or `node qa/e2e/run-training-qa.mjs` from the
repository root. Both use `scripts/with-qa-postgres.py` to create a new private
Unix-socket cluster under `/tmp`, initialize a synthetic database, and stop/remove
the cluster after success or failure. Cluster-scoped test roles cannot collide
with other runs. The wrapper overrides an existing `WONDERWILDS_DATABASE_URL`;
it never connects to persistent demo progress. `WW_QA_DATABASE_URL` is an internal
wrapper variable and must not be set by operators.

Install PostgreSQL 17 and put Cargo/Python (and Node 24 for browsers) on PATH.
`WONDERWILDS_PG_BIN` selects the PostgreSQL bin directory. Without an override,
the wrapper checks the prepared Linux location, Homebrew PostgreSQL 17, then
`pg_config --bindir`. Existing `CARGO_HOME` and `RUSTUP_HOME` are respected.
Set `CHROMIUM_PATH` to an installed Chromium binary for local browser runs.

These checks do not establish human accessibility/content approval, educational
efficacy, production deployment readiness, or backup recovery. The normal Rust
suite still intentionally skips database integration tests; use the dedicated
command when validating persistence locally.
