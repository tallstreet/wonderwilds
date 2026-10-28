# Repository checks in GitHub Actions

`.github/workflows/checks.yml` runs on pull requests, pushes to `main` and manual
dispatch. It grants only read access to repository contents and does not deploy,
publish artifacts or use application credentials.

The Rust job installs the toolchain pinned in `rust-toolchain.toml`, fetches
locked dependencies, checks formatting and Clippy, and runs workspace tests.
The web/content job uses Node 24, Python 3.13 and `npm ci` with the committed npm
lockfiles. It checks contracts and generated-code drift, authored content and
artwork, frontend types/lint/format/unit tests/build, nested API tests and legacy
guest-save fixtures. Together these mirror the components of `scripts/check.sh`
without relying on that script's `/workspace` Cargo-home defaults.

## Gates that remain separate

The normal Rust suite deliberately ignores the real PostgreSQL integration tests.
This workflow does **not** establish migration, RLS, rollback, race, deletion or
database persistence correctness. Continue running `sh scripts/check-postgres.sh`
against its disposable synthetic database in the prepared Linux environment for
persistence changes. That helper hardcodes `/workspace/setup-tools` and requires a
portability update before it can run directly on a hosted GitHub runner. Do not
point it at a persistent demo or user database.

Browser journeys also remain separate: run the guest smoke, activity controls and
server-backed training journey using their documented local server and isolated
QA database setup. This workflow makes no browser, accessibility, educator review,
deployment or public-release readiness claim.

The workflow must run on GitHub before its hosted-runner success is established.
Local command checks and YAML validation alone do not prove a hosted execution.
