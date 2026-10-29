# Working in Wonderwilds

## Start here

- Read `README.md` and `docs/IMPLEMENTATION_STATUS.md` before changing behavior.
  Read the relevant feature README and `docs/adr/` decision records as needed.
  Some feature READMEs describe earlier milestones; verify current behavior in
  code and the root documentation rather than treating plans as shipped features.
- Use the existing checkout. Do not create a worktree unless requested.
- Inspect `git status` before editing, preserve unrelated changes, and keep the
  patch focused on the requested work.
- The project includes a playable Phaser guest adventure and a development
  Rust/PostgreSQL backend. It is not a completed production release.

## Repository map

- `frontend/`: strict TypeScript, Phaser 3, Vite, and accessible HTML activity UI.
  `src/world.ts` renders the world; `src/main.ts` handles guest UI;
  `src/progress.ts` validates guest saves and evidence; `src/navigation.ts` handles
  pathfinding. `src/backend-world.ts` integrates backend play, `src/api/` owns the
  typed client/import preparation, and `src/learning/` owns activity controls.
- `contracts/`: versioned JSON Schema and OpenAPI, generated Rust/TypeScript wire
  models, and shared positive/negative fixtures.
- `backend/crates/`: `domain` for deterministic gameplay, `learning` for assessment
  and mastery policy, `content` for authored content, `director` for proposals,
  and `api` for HTTP, identity, and PostgreSQL persistence.
- `backend/migrations/`: numbered SQL migrations and database checks.
- `content/`: draft curriculum, activities, story packs, and validators.
- `qa/e2e/`: Playwright browser checks. `qa/fixtures/legacy/`: guest-save fixtures.
- `scripts/`: contract generation, quality checks, and local database/demo setup.
- `docs/`: implementation status, design decisions, review gates, and privacy notes.

## Setup and local development

- Use the Rust toolchain pinned in `rust-toolchain.toml` and committed `Cargo.lock`.
  Use Node 24 for repository tooling and Python 3 for generators/validators.
- Install with `npm ci` separately in `frontend/`, `contracts/`, and `qa/e2e/`;
  run `cargo fetch --locked` at the root. npm is the established package manager
  despite the additional frontend pnpm lockfile. Avoid unrelated lockfile churn.
- Guest frontend: from `frontend/`, run `npm run dev -- --host 127.0.0.1`.
- macOS backend demo: install PostgreSQL 17 and run `sh scripts/local-demo.sh`
  from the root. See the root README for prerequisites and shutdown instructions.
  Ports 5173 and 3001 must be free. `WONDERWILDS_PG_BIN` overrides the PostgreSQL
  binary directory. `/?guest=1` selects the original guest adventure.
- `.local-demo/` contains persistent synthetic data and session secrets. Keep it
  ignored, preserve its data, and never expose the demo bridge through a tunnel.
- Several scripts assume the prepared Linux `/workspace` environment. Inspect
  them before running locally. In particular, `scripts/check.sh` defaults Cargo
  and rustup homes to `/workspace`; on macOS pass the actual local homes, e.g.
  `CARGO_HOME="$HOME/.cargo" RUSTUP_HOME="$HOME/.rustup" bash scripts/check.sh`
  for a standard installation. `scripts/check-postgres.sh` hardcodes Linux paths
  and requires that prepared environment or a deliberate portability change.

## Implementation boundaries

- Keep domain, learning, and content independent of HTTP, databases, browser
  types, and provider SDKs. API and director may depend on the pure crates.
- Rust authoritatively assesses backend answers and commits progress/rewards.
  Neither the browser nor director may invent correctness, rewards, or mastery.
- `contracts/schemas/v1.json` is the wire source of truth. Change the schema,
  relevant OpenAPI definitions, consumers, and fixtures together. Regenerate with
  `python3 scripts/generate-contracts.py`; do not hand-edit
  `contracts/src/models.rs` or `contracts/generated/contracts.ts`.
- Preserve version compatibility. Validate untrusted wire data using the shared
  schema before consuming it; generated types alone do not enforce bounds.
  Keep answer keys and internal task definitions out of public responses.
- Derive identity from verified server sessions and check profile ownership and
  CSRF/Origin. Preserve transaction atomicity, RLS, revision checks, and command
  idempotency. On uncertain writes, retry the exact prepared command/body/ID;
  do not silently create a new command or revision.
- Preserve guest saves and keep them separate from backend progress. Imported
  browser evidence remains `legacy_unverified` and cannot establish mastery.
  Do not enable import/account UI for endpoints that do not exist.
- Add a new numbered migration instead of editing an applied migration. Migration
  hashes detect drift; do not bypass them. Run destructive database fixtures only
  against disposable synthetic databases, never the persistent demo or user data.
- Keep content review gates honest: pending reviews and `releaseEnabled: false`
  remain until real approval. Story access must retain supported routes and must
  not require independent academic mastery. Preserve support/evidence distinctions.
  For content edits, follow the pack README and update integrity hashes only for
  intentional changes. Passing validators is not educator approval.
- Preserve keyboard/touch alternatives, accessible control names, and responsive
  activity layouts. Treat stored JSON as untrusted and render user text as text.
  Keep credentials and real child/parent data out of fixtures, logs, and commits.

## Verification

Run checks appropriate to the changed behavior. For broad changes, the root
`bash scripts/check.sh` is the aggregate check (subject to the environment note
above). Its main components can also be run separately:

```sh
# Repository root
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd contracts && npm test && npm run typecheck && npm run check:generated)
python3 content/curriculum/validate.py
python3 -m unittest discover -s content/curriculum -p 'test_*.py'
node content/curriculum/check-schema.mjs
python3 content/story/validate.py
python3 -m unittest discover -s content/story -p 'test_*.py'
(cd frontend && npm run check && node --import tsx src/api/api.test.ts)
node --import ./frontend/node_modules/tsx/dist/loader.mjs qa/fixtures/legacy/baseline.test.ts
```

- Frontend `npm run check` includes strict typechecking, Oxlint, Oxfmt checking,
  top-level unit tests, and the production build. The nested API tests require the
  separate command above. Follow existing formatting rather than mass-formatting
  unrelated files.
- For browser changes, start Vite, then run `(cd qa/e2e && npm test)` and, for
  activity controls, `node frontend/src/learning/activity.browser.mjs`.
  Set `CHROMIUM_PATH` when Chromium is not at `/usr/bin/chromium`.
- For backend-world changes, follow the root README's fresh QA database and
  separate-port setup for `qa/e2e/backend-demo.mjs`; do not run it on demo progress.
- Persistence changes need the real database checks in
  `sh scripts/check-postgres.sh` in its supported environment. The ordinary Rust
  suite deliberately ignores three database integration tests and is insufficient
  to establish database correctness alone.
- Add regression coverage for changed logic or bugs using the existing test
  structure. Documentation-only edits do not require the application suite.
- Before finishing, inspect the diff and run `git diff --check`. Report what
  changed, checks actually run, and any unverified behavior or blocked checks.
  Update relevant status documentation when integration scope or limitations change.
