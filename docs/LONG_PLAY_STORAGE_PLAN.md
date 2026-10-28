# Long-running companion profiles

Status: proposed engineering design; not implemented or validated. The existing
2,000-command limit remains in force. This design is a prerequisite for sustained
training, rather than a reason to remove the guard now.

## Current constraint

`backend/crates/api/src/game/mod.rs` stores every API receipt, encounter,
learning observation and domain event inside one serialized aggregate. Each
receipt includes its original response, so repeated snapshots amplify growth.
`backend/crates/domain/src/engine.rs` independently retains domain-command
receipts. `repository.rs` clones/deserializes the aggregate, writes its whole
JSON document and also appends evidence/events to PostgreSQL tables.

The API derives evidence sequence numbers from the aggregate vector length.
Snapshot mastery recomputation resolves historical encounter parameters through
the aggregate encounter map. Removing old vectors or encounters would therefore
change sequencing or break mastery. Dropping either receipt map would allow old
command IDs to lose their original result. Increasing the cap alone leaves these
costs and retry hazards unresolved.

## Storage boundary

Add a new numbered migration after the migrations present at implementation
time; do not edit applied migrations. The migration and adapter must introduce:

| Record | Key and required contents | Lifetime |
| --- | --- | --- |
| `command_receipts` | `(profile_id, command_id)`; validated original request JSON, exact response JSON, API revision, receipt-format version | Until profile deletion; never evict a receipt merely because the hot cache is full |
| `domain_command_receipts` | `(profile_id, command_id)`; original versioned command and returned event sequence range | Until profile deletion; reconstruct original returned events from durable `game_events` |
| `game_encounters` | `(profile_id, encounter_id)`; quest/training ID, recipient, skill, seed, rules/content versions, support/check count, lifecycle state and issuing session | Pending records retained; completed records retained while evidence/replay requires them |
| `game_checkpoints` | `(profile_id, api_revision)`; checkpoint-format version, domain state, domain/event/evidence high-water marks, rules/content versions and integrity digest | Latest verified checkpoint plus previous recovery checkpoint; older checkpoints removable only after durable replay verification |
| `progress` | Existing owned profile row and versioned compact current snapshot with explicit sequence counters and checkpoint reference | Current state; no embedded full receipt/history copies |

All new profile-owned tables require foreign keys with `ON DELETE CASCADE`,
enabled and forced RLS using the existing ownership predicate, no public grants,
and explicit runtime-role grants in provisioning. Runtime SQL uses the same
verified parent setting and profile lock as existing commands. No request may
choose the parent identity. Extend readiness checks for the new required schema.

Preserve existing `learning_events`, `game_events`, unique `reward_grants` and
`game_outbox`. These are durable records, not a second copy in every snapshot.
Add versioned assessment-reconstruction metadata alongside learning events or
their encounter references, including canonical parameter fingerprints. Never
label repeated draft practice as novel-transfer evidence.

The bounded part is the loaded working state: one active encounter, explicit
sequence counters, a finite recent receipt cache (initial target 128), and a
checkpoint plus bounded event suffix (initial target 256 domain events). Durable
receipts and learning history still grow with use. Their storage budget,
monitoring and approved retention policy remain separate operational work;
this design does not claim constant total database size.

## Atomic execution and retry order

1. Validate the closed command schema, verify session/profile ownership, set
   transaction-local RLS identity and lock the same owned profile row used by
   deletion and all other writers.
2. Look up the durable API receipt **before checking the submitted revision**.
   Equal parsed JSON returns the exact stored response without side effects;
   the same ID with any changed body returns revision conflict. A hash may speed
   lookup but must not replace equality of stored JSON.
3. Load a compatible checkpoint/current state and the bounded required suffix.
   Validate format, counters, pending encounter bindings and versions. Corruption
   or missing necessary history fails closed; never reset a profile.
4. For a new command, enforce current revision, eligibility and bounded request
   rate. Apply the pure domain transition. Persist domain receipts outside the
   checkpoint through a repository-supported receipt boundary; do not clear a
   domain receipt map while retaining the old standalone apply semantics.
5. Commit the new current state, encounter changes, monotonic evidence/events,
   unique rewards, outbox and both receipt records together. Derive sequences
   from explicit persisted high-water counters, never retained-vector lengths.
6. When the suffix reaches the checkpoint threshold, write and verify a new
   checkpoint in this same transaction. Do not delete receipts, grant keys,
   pending encounters or durable events at the boundary. Publish the response
   only after commit. A lost response remains recoverable by the original command.

The pure engine needs an explicit checkpoint executor API whose caller guarantees
durable receipt lookup/insertion. Preserve the current receipt-retaining API for
standalone replay callers until they migrate. Domain idempotency must not become
an undocumented assumption in the HTTP layer.

## Learning summaries and replay

First move mastery queries to durable learning/encounter records and persist
explicit evidence counters; retain full evidence until a separately tested,
versioned summary is proven sufficient. Current snapshot recomputation depends
on canonical parameter identity, support history, session context and evidence
ordering. A simple latest-score or rolling average is insufficient.

Only introduce incremental mastery checkpoints after golden comparisons against
the existing complete recomputation, including delayed review and context/session
boundaries. Policy upgrades must support rebuild from retained evidence. Fetching
all evidence per request can be a safe intermediate migration but does not meet
the final bounded-work acceptance gate; benchmark and report it honestly.

Store the versioned command and trusted facts needed for domain replay alongside
event history. Events alone currently do not supply every `ReplayEntry` input.
Pin original content/rules versions and retain their readers; replay must not
silently substitute newly authored training or change a pending recipient.

## Existing-save migration and rollback

Migrate lazily under the profile lock or with an owner-controlled offline job.
Detect the old format explicitly. Validate the full old aggregate, insert all API
and domain receipts, encounter bindings and reconstruction metadata, and compare
existing durable event/evidence rows with embedded copies. Duplicate equal data
is allowed; a mismatch aborts. Preserve revisions, counters, grants, support,
friendship, appearances, pending sessions and exact old responses.

Write the first new checkpoint and format marker only after every copied record
is consistent, in the same transaction. Restarting migration must be harmless.
Do not run migration against the persistent demo while developing it.

Deploy compatible readers before writing the new format. Once compact snapshots
are written, the old binary cannot safely load them. Rollback must use a compatible
binary or an explicit transactional reconstruction of the old aggregate from
durable records; it must never restore a stale pre-migration snapshot over newer
progress. Exercise this reconstruction in a disposable database before enabling
the format for persistent profiles.

Extend parent export to include encounter/evidence context and durable replay
records as appropriate; avoid exporting authentication secrets. Profile deletion
must cascade through new tables and retain existing tombstone/outbox behavior.
Do not infer approved retention periods from this engineering proposal.

## Delivery and acceptance

1. Add schema, RLS/ownership/deletion/export fixtures and backward-compatible
   record readers; keep the command cap.
2. Externalize API/domain receipts and encounters, introduce explicit counters
   and migrate synthetic old profiles. Preserve every old retry result.
3. Add verified checkpoints, durable learning reads and sufficient versioned
   mastery summaries; benchmark bounded hot state and per-command work.
4. Only then replace the lifetime cap with independent request-rate and storage
   limits that do not permanently strand a healthy active profile.

Required disposable-PostgreSQL tests:

- Execute more than 2,000 distinct valid commands across several checkpoints;
  restart the process and verify progress and monotonic evidence/event sequences.
- Retry commands from before and after every checkpoint and migration boundary;
  verify exact old response, no new evidence/reward/outbox entry, and rejection of
  changed bodies under the same ID.
- Race equal and conflicting commands through separate database connections;
  verify one commit and existing revision-conflict behavior.
- Inject failures before each durable insert/checkpoint commit and verify complete
  rollback, including both receipt layers and unique training grants.
- Resume a supported pending training encounter through migration/checkpoint and
  restart; changing team must not change its bound recipient or reward.
- Compare replay and complete mastery recomputation before/after compaction and
  across rules/content versions; corrupt or missing records must fail closed.
- Verify non-owner RLS denial, deletion cascades, export completeness and compatible
  rollback/reconstruction. Exercise browser uncertain-write retry after restart.

Until these checks pass and measured hot-state bounds are recorded, the current
cap is an explicit development limitation. Passing the first companion training
journey does not complete this storage work.
