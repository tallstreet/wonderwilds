# WW-07: deterministic independent-evidence policy

Status: implemented experimental core; authored API evidence and durable replay
are integrated. Reviewed transfer items, calibration and general secure-mastery
integration remain pending. Mission practice uses the separate experimental
policy described below.

`backend/crates/learning/src/mastery.rs` implements `independent-transfer-v1`.
Recomputation uses immutable ordered server events. Imported evidence is ignored
for qualification; support, worked examples and read-aloud cannot qualify.
Abandonment introduces a skill without implying an incorrect answer. Reopening
an identical independently assessed item-family/parameter combination cannot
increase qualifying checks. Supported introductions do not consume an independent
qualification slot. Support state must stay monotonic across encounter events.
Duplicate event IDs/sequences, contradictory first-check flags and disordered
histories fail rather than partially updating a summary.

The latest eight eligible checks need seven correct, the latest three correct,
three variants, two server sessions, two contexts, a correct transfer item, and
two representations where valid. A reviewed node may permit one representation.
The trusted encounter factory supplies parameter fingerprints and transfer design;
these are not fields a browser may claim. Server provenance is an internal trust
boundary and must be assigned after authentication/assessment, never imported.

The frontier excludes secure skills, checks prerequisites, supports explicit
approved entry probes, prefers sparse independent evidence, and uses a recorded
seed for deterministic ties. Subject selection preserves the history. All-secure
packs return enrichment; an empty frontier returns prerequisite support. Two
unsuccessful checks offer a concrete hint; three offer a worked example or a
simpler prerequisite. Difficulty advice uses a bounded explicit rung index:
two distinct independent successes offer a harder probe; repeated difficulty
offers support before a simpler probe. This advice does not gate story rewards.

Validation: learning crate unit tests exercise diverse transfer qualification,
seven-of-eight/latest-three, representation limits, assisted and imported
records, repeat items, missing transfer/session diversity, corrupted order,
abandonment, subject switching, frontier, enrichment, deterministic replay and
bounded difficulty advice. Run from the repository root:

```sh
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH="$CARGO_HOME/bin:$PATH"
cargo test -p wonderwilds-learning --locked
```

Remaining WW-07 gates: educator calibration against independently reviewed
transfer items and accessibility/language subgroups; validated integration of
server-issued sessions, immutable parameter fingerprints and transfer metadata;
general transfer-policy replay beyond the delivered authored audit CLI; window-scoped item freshness (the current core
conservatively counts each independent parameter combination once per history); curriculum/context balance beyond sparse-evidence
ranking; optional refresher state following fresh composite-task difficulties.
No diagnostic, school-grade or learning-efficacy claim follows from these tests.
Rollback retains immutable events and recomputes under a previous version; never
rewrite evidence to force a policy result.

## Mission practice progression — 31 October 2026

The [curriculum mission plan](../CURRICULUM_MISSION_PLAN.md) introduces a separate
`provisional-mission-v1` practice-readiness policy over the same graph. It does
not set `mastery::Status::Secure`, change existing authored summaries or claim
reviewed transfer. Eight distinct canonical tasks, seven correct, the latest
three correct, diverse variants/contexts/representations and two server sessions
are the initial experimental readiness criteria.

A canonical item occupies at most one qualifying slot. Later independent first
checks can replace that item's projected outcome/session without increasing the
distinct-item count; immutable evidence remains available for audit. This differs
from `independent-transfer-v1` lifetime first-item semantics, so it has a separate
policy identifier and state. It permits later-session recovery with a finite
practice bank. Supported practice does not qualify, and the story remains
available without readiness. Educator review and transfer calibration are still
required before promoting this result to secure academic mastery.
