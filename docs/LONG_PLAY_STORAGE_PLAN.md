# Long-running companion and story profiles

Status: implemented for the development PostgreSQL API, 31 October 2026.
Verification results and operational limitations are recorded in
[implementation status](IMPLEMENTATION_STATUS.md). Migration `0005_long_play.sql`
replaces the server's 2,000-command adventure and 256-command story lifetime
budgets with durable journals and bounded current state. The standalone, replay-log
CLI and in-memory test executors keep their original limits.

## Storage boundary

| Record | Contents | Retention |
| --- | --- | --- |
| `command_receipts` | Original adventure request/response, revision, replay clock/session, integrity digest | Until profile deletion |
| `domain_command_receipts` | Versioned domain command, trusted context and resulting events; legacy receipts preserved | Until profile deletion |
| `game_encounters` | Issued skill/seed/session, support/check count, lifecycle and bound training recipient | Until profile deletion |
| `learning_events` | Original ordered evidence plus versioned canonical-parameter fingerprint, content fingerprint and no-transfer policy metadata | Until profile deletion |
| `game_events`, `reward_grants`, `game_outbox` | Original events, unique reward keys and delivery records | Existing lifecycle unchanged |
| `game_checkpoints`, `story_checkpoints` | Format/version, current state and SHA-256 digest | Current and previous revision only |
| `persistence_origins` | Original validated pre-conversion save and digest; adventure content fingerprint | Until profile deletion |
| `story_command_receipts` | Original wire request/response and revision, resolved domain command, integrity digest | Until profile deletion |
| `story_mission_plans` | Original versioned accepted mission briefs | Existing lifecycle unchanged |
| `command_windows` | Per-profile/per-mode minute and committed command count | One renewable row per mode |

Every new profile-owned table has cascading foreign keys, enabled **and forced**
RLS, the existing parent ownership predicate, and no public table grants. Existing
runtime roles receive explicit grants during migration; provisioning also grants
newly created runtime roles access. Readiness checks require the new tables.
Parent export includes origins, checkpoints, receipts, encounter context,
reconstruction metadata and domain events. Deletion cascades through all of them.
No authentication secrets are added to exports.

The bounded part is working state, not total database size. Adventure snapshots
retain finite gameplay state, the pending encounter, explicit evidence/event
high-water marks and a versioned learner projection. Story snapshots retain
current mission/world/choice state and the latest eight events, without full
command history or response maps. There is no hot receipt cache or replay suffix:
a checkpoint is written on **every** successful command. This trades a second
small indexed write for a simpler verified boundary. Each current snapshot is
limited to 64 KiB; current and previous checkpoints are retained.

Archived receipts, encounters, evidence and events continue to grow. Storage
monitoring, quotas, backup operations and an approved retention policy remain
separate operational work. No records are evicted to make an old retry disappear.
The wire revision ceiling is 1,000,000,000, shared with the existing adventure
contract; it is an integer compatibility boundary, not the old play budget.

## Transactions, verification and retries

1. Validate the command, set transaction-local verified parent identity and lock
   the owned profile row shared by game, story and deletion.
2. Migrate a legacy save atomically if needed; otherwise read the current and
   previous checkpoints through their profile/revision indexes. Verify digests,
   supported versions and content fingerprint. Replay the latest committed
   command from the previous checkpoint and compare the resulting current state.
   Adventure also compares the original response and indexed evidence/event
   high-water marks. A sole initial checkpoint must match the preserved origin.
3. Look up the durable command ID **before checking its submitted revision**.
   Equal parsed JSON returns the original response after digest/schema checks;
   changed JSON returns conflict. Current corruption still fails closed, including
   for retries. Old responses are never regenerated with a new model or plan.
4. New commands consume a renewable budget of 4,096 committed commands per minute
   per profile/mode. Retries are exempt. Rollback does not consume the budget.
   Existing HTTP concurrency/body limits remain in force.
5. Apply authoritative assessment/domain transitions, then atomically commit the
   response receipt, resolved domain input/context, encounter/evidence/events,
   unique rewards/outbox, new current state and checkpoint. Failure rolls all of
   them back. Sequence counters come from persisted high-water marks.
6. Retain the latest two verified checkpoints. No receipt, evidence, grant or
   pending binding is deleted at this boundary. Publish only after commit.

`engine::apply_checkpointed` is the explicit executor for callers that own durable
receipt lookup/insertion. The original `engine::apply` retains standalone
idempotency semantics. Story `State::compact` and `from_durable_checkpoint` form a
separate trusted repository boundary; the untrusted CLI `from_checkpoint` still
requires full replay and does not accept compact server state.

Checksums detect accidental corruption, not hostile modification by a database
administrator who can rewrite both records and checksums. Normal reads verify the
latest transition, not the entire archive. Offline audit detects older missing or
changed journal/evidence records. Missing required history never resets a profile.

## Learning policy and compatible replay

The authored assessment factory always supplies `novel_transfer=false`. Under
`independent-transfer-v1`, its public projection therefore has only introduced or
practising states: seen verified evidence and seen answer checks are sufficient
statistics. `authored-no-transfer-independent-transfer-v1` explicitly pins this
restricted projection. It is **not** a general mastery checkpoint or an eligible
check counter. Repeated draft practice never becomes transfer evidence.

Golden tests compare every response against full recomputation across supported
and independent practice, wrong answers, worked examples, read-aloud, abandonment,
repeated items, delayed play and multiple server sessions. Offline adventure audit
also reconstructs all observations using durable encounters and recomputes the
complete public snapshot independently. Existing learning-policy tests cover
transfer, session/context diversity and corrupt evidence ordering.

New metadata records canonical parameter fingerprints and content/policy versions.
Current content/rules or summary-policy mismatches fail closed. Content changes
require an explicit compatible reader/rebuild; this implementation does not
silently evaluate old records with changed content or claim support for arbitrary
future mastery policies. Training recipient and issuing session remain bound to
the original pending encounter when teams or sessions change.

## Existing saves and recovery

Migration is lazy, owner-locked and transactional. Adventure compares embedded
learning/events against their durable rows, validates old receipts and compares
full versus compact public projections before changing the save. Story replays
legacy v1/v2/v3 checkpoints and checks their receipt revisions, IDs and wire
schemas. Both preserve the original save as an integrity-checked origin. A failed
conversion leaves the original save intact. Existing terminal-at-256 responses
remain valid receipts, while the migrated live story can continue beyond 256.
Guest saves and legacy-unverified imports are unchanged.

Offline maintenance uses the same profile lock and ownership predicate:

```sh
# Explicitly configure the intended database; this tool has no default database.
# audit performs no recovery; recover is an explicit transactional repair.
cargo run -p wonderwilds-api --bin persistence --locked -- audit game PARENT_UUID PROFILE_UUID
cargo run -p wonderwilds-api --bin persistence --locked -- audit story PARENT_UUID PROFILE_UUID
cargo run -p wonderwilds-api --bin persistence --locked -- recover game PARENT_UUID PROFILE_UUID
cargo run -p wonderwilds-api --bin persistence --locked -- recover story PARENT_UUID PROFILE_UUID
```

`WONDERWILDS_DATABASE_URL` supplies the operator's connection. Audit replays the
journal in pages of 256 from the preserved origin, verifies response/input digests
and reaches the latest committed revision. Adventure additionally compares domain
records, evidence/reconstruction metadata, events and full mastery recomputation.
Recovery uses that same verification before transactionally replacing the current
and previous checkpoints and current snapshot. It refuses gaps and cannot replace
newer progress with the origin or an older backup. Full learning recomputation is
intentionally an offline, history-sized operation. Large audits hold the profile
lock and should run during a maintenance window.

Rollback means deploying a **compatible** reader or using this verified current
state reconstruction. An old binary cannot read compact saves or play beyond its
old caps; downgrading to it is unsupported. No reset or repair endpoint is exposed
to the browser. Development verification uses disposable databases only; it does
not migrate `.local-demo` or establish production backup readiness.

## Reproducible acceptance checks

- `bash scripts/check.sh`: Rust/contract/content/frontend suites, including golden
  compact/full-history projection comparison and standalone cap compatibility.
- `sh scripts/check-postgres.sh`: >2,000 adventure and >256 story commands, exact
  old retries, supported pending-save migration/rollback, team binding, concurrency,
  unique rewards, bounded snapshot/checkpoint counts, full audit, corrupt-checkpoint
  recovery, missing-history refusal, rate budget/retry exemption, RLS/export/delete,
  and continuation of a migrated story previously exhausted at revision 256.
- `WW_LONG_PLAY=1 node qa/e2e/run-training-qa.mjs`: >2,000 HTTP commands followed
  by browser training, supported pending encounter across process restart and
  lost-response exact retry after another API restart.
- `WW_LONG_PLAY=1 WW_STORY_RETURNING=1 WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs
  node qa/e2e/run-training-qa.mjs`: >256 HTTP story commands followed by seven
  missions, branch decisions, lost-response retry and three actual API/browser
  restarts. Repeat with `WW_STORY_BRANCHES=alternate` for the other branch.

Set `CHROMIUM_PATH` and `WONDERWILDS_PG_BIN` as required by the local environment.
Storage QA uses authored story/plans; live model latency/quality is a separate gate.
