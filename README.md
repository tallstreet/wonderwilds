# Wonderwilds development

The existing Phaser guest adventure remains playable in `frontend/`. The Rust
workspace adds versioned contracts, pure gameplay and assessment, an experimental
mastery policy, an authored director, and a PostgreSQL-backed development API.
The expansion is **not a completed release**. See
[implementation status](docs/IMPLEMENTATION_STATUS.md) for integration and review gates.

Use this existing isolated checkout; do not create a worktree unless requested.
Node 24 is available in the prepared environment. Rust is pinned by
`rust-toolchain.toml`; activate it in this cloud workspace with:

```sh
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH="$CARGO_HOME/bin:$PATH"
```

Install locked JavaScript dependencies with `npm ci` separately in `frontend/`,
`contracts/` and `qa/e2e/`; use `cargo fetch --locked` at the root.
For a new cloud machine, install local PostgreSQL binaries with
`sh scripts/install-postgres.sh` if they are absent. Start and migrate the local
synthetic development database using `sh scripts/migrate-postgres.sh`.
This starts a private Unix-socket cluster without a TCP listener; it is not
production database provisioning. Migrations record hashes and reject drift.

Run the guest game from `frontend/` with `npm run dev -- --host 127.0.0.1`.
In a separate terminal, run the authored API from the repository root:

```sh
WONDERWILDS_DATABASE_URL='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=wonderwilds_dev' \
  cargo run -p wonderwilds-api --locked
```

The API listens on local port 3001. `/healthz` reports the development process;
`/readyz` probes database/schema readiness and explicitly reports that content is
not release-ready. Gameplay requests without a valid parent session return 401.
There is no test-login bypass or configured parent identity provider. The
browser remains in guest mode; new cloud client and activity components are
exported modules awaiting the account/import integration gate.

From the root, `bash scripts/check.sh` runs Rust format/Clippy/tests, shared
schema and generated-code checks, content validators, frontend checks/build,
client tests and legacy fixtures. `sh scripts/check-postgres.sh` separately runs
real migration, RLS, ownership, deletion, race and rollback tests in a disposable
database. Its two database tests are deliberately ignored in the default Rust
suite and explicitly executed by this helper. With Vite running, run:

```sh
(cd qa/e2e && npm test)
node frontend/src/learning/activity.browser.mjs
```

The prepared machine has Chromium at `/usr/bin/chromium`. Browser and network
operations may need the executor's supported sandbox escalation. No public
preview or production deployment is created. See the feature READMEs under
`frontend/src/api/`, `frontend/src/learning/`, `content/`, and
`backend/crates/api/` for trust boundaries and limitations.
