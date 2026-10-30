# Combined story generation and mission planning — 31 October 2026

## Scope and method

The opt-in seven-mission Phaser journey uses both live Modal providers, real
Rust commands and a fresh disposable PostgreSQL cluster for each branch. No
persistent demo or guest progress is used. The default branch places the instrument
in the woods, builds a leafy nook and gathers on the near bank; the alternate
branch places the instrument at the gathering, builds a star canopy and gathers
on the far bank, with the opposite preparation order.

Each journey requires seven generated saved plans, matching Missions titles and
briefs, and accepted scene responses in four sessions. It closes the browser
context and restarts the API after the trilogy, during a paused corner mission,
and during gathering preparations. The entire canonical scene input, revision
and saved plans must match across each restart. This clears the scene cache and
uses a fresh browser context but reuses the synthetic authentication credential;
it does not establish production reauthentication or session renewal.

The test loses a committed exchange response, reloads and retries the exact
command body, checking unchanged revision and plans after the retry. Both runs
retain seven completed missions, lasting scenery, phone history, correct present
characters and unchanged educational progress. All chosen actions use real
revision-bound browser handles and Rust validation.

## Evidence conventions

The API uses its unchanged six-second provider timeout. Accepted/fallback counts
are HTTP scene-response counts, including cache hits and QA diagnostic reads;
they are **not unique generations or first-pass model-quality rates**. Existing
logs combine unavailable, busy and invalid output into one fallback category, so
these runs cannot attribute each fallback to a specific cause. Authored fallback
never counts as an accepted model result. Timing distributions are HTTP read and
UI action-through-refresh measurements, including cache, UI work and injected recovery; they are not isolated model inference latency.

The model is the deployed `wonderwilds` alias; immutable weights are not exposed.
The saved JSON summaries retain source/prompt/schema hashes and these limits.
Mechanical state continuity does not establish prose suitability, contradiction
rates, unseen-mission generalisation, cost budgets or release approval.

## Successful live journeys

| Measure | Default branch | Alternate branch (diagnostic rerun) |
| --- | ---: | ---: |
| Completed missions / persisted generated plans | 7 / 7 | 7 / 7 |
| Successful API restart + fresh browser resumptions | 3 | 3 |
| Accepted scene responses | 216 | 223 |
| Authored scene fallbacks | 10 | 9 |
| Planner fallbacks / stale scene responses | 0 / 0 | 0 / 0 |
| Accepted scenes by session | 83, 58, 64, 11 | 83, 61, 68, 11 |
| Story HTTP read median / p95 | 33 / 72 ms | 34 / 50 ms |
| Action-through-refresh median / p95 | 1,925 / 4,372 ms | 1,871 / 4,004 ms |
| Maximum action-through-refresh (includes injected recovery) | 31,266 ms | 31,276 ms |

Both browser processes exited successfully and their disposable clusters were
stopped and removed. Exact retry recovery, all branch-specific outcomes, full
canonical context equality across restarts, plan stability and unchanged education
passed. Maximum diagnostic reads were 6,058/6,052 ms, consistent with the bounded
provider wait. This does not establish a percentile budget for uncached inference.

Saved summaries: [default](combined-2026-10-31/default.json) and
[alternate](combined-2026-10-31/alternate.json). Raw local run logs remain at
`/tmp/ww-combined-default-live.log` and `/tmp/ww-combined-alternate-live-r2.log`;
these temporary paths are not durable repository artifacts. Final desktop/mobile
screenshots were inspected. These successful runs establish the requested
combined-provider branch/resumption path, but the earlier intermittent timeout
below remains an open reliability finding; M1 is not unconditionally closed.

## Local regression checks

- Both authored branch rehearsals passed the same restart/retry assertions.
  The first rehearsal exposed cleanup waiting on already signal-terminated API
  children; the runner now skips those children. Its completed test wrapper was
  stopped to clean its cluster; subsequent rehearsal/live runs cleaned up normally.
- `cargo test -p wonderwilds-api --locked`: 32 passed, three database tests ignored.
  The new combined-provider test checks saved-brief inputs, scene-outage fallback
  and unchanged original acceptance receipts without replanning.
- API all-target Clippy with warnings denied, Rust formatting, JavaScript syntax
  checks and `git diff --check` passed. Desktop/tablet guest smoke passed.
- Removed the duplicate scene request `response_format` key and reconciled a
  conflicting regression assertion, retaining the effective `story_scene` schema
  name and request semantics. No persistence/schema/gameplay changes were made.
- Node 26.12.0 was available, not Node 24. No full aggregate suite or separate
  PostgreSQL integration suite was rerun; browser journeys used real disposable
  PostgreSQL. No new mastery, educator, accessibility or production approval is
  implied.

## Failed attempts retained

- [Unavailable endpoint](combined-2026-10-31/unavailable.json): before operator
  recovery, zero accepted plans/scenes, five scene fallbacks and one planner
  fallback. A separate HTTP status probe returned 503. The strict journey failed
  at the first missing saved plan; this is availability evidence.
- [Alternate travel timeout](combined-2026-10-31/alternate-timeout.json): after
  recovery, the first alternate attempt saved its first plan but timed out waiting
  20 seconds for the first workshop travel revision. No cause was established in
  that attempt. Subsequent diagnostics retain command-send/status timestamps and
  UI state on failure. A later successful run does not erase this intermittent
  failure or establish its cause.

## Reproduction

See [browser QA instructions](../../qa/e2e/README.md). Use both provider flags,
`WW_STORY_COMBINED=1`, `WW_STORY_EXPECT_PLANS=1`, the seven-story browser test and
an output report path; repeat with `WW_STORY_BRANCHES=alternate`. Credentials
remain in the local environment, outside evidence. The runner always owns and
removes a new synthetic cluster. Default ports are 3002/5180.
