# Durable game storage integration review

Status: source review complete; integrated validation is recorded separately.
This records the review of the storage branch, not release approval or a claim
that proposed acceptance tests have passed.

The review focuses on observable invariants across the legacy aggregate and the
compact PostgreSQL-backed representation:

- An owned-profile lock precedes durable receipt lookup. Equal old requests return
  their exact recorded responses before revision or request-rate checks; changed
  bodies cannot reuse a command ID.
- Legacy embedded event/evidence histories must match durable rows by count,
  sequence and JSON contents before the new format is committed. An existing
  conflicting row must never be hidden by an unchecked conflict clause.
- API, domain-event and learning-evidence counters remain distinct. Hints and
  incorrect answers advance API state without necessarily issuing a domain action.
- Pending encounter support, first-check state, issuing session, bound training
  recipient and policy fingerprint survive migration and process restart.
- Mastery retains seen/attempted state and the eligible eight-item window. Durable
  novelty is consumed only by qualifying independent first checks: supported or
  unassessed checks do not consume it, while both independent correct and incorrect
  first checks do. Repeated items across checkpoints remain repeated.
- New profile-owned records force RLS, cascade on profile deletion and appear in
  the authenticated parent export. Their durable growth is separate from the
  bounded current save document.

## Findings addressed during implementation

Review identified these concrete failure modes and the corresponding owners
changed their code:

1. Domain receipt validation originally checked sets/counts without requiring
   sequence ranges to follow command revision order. Swapped ranges now fail
   legacy-checkpoint validation, and each receipt keeps consistent versions.
2. Legacy API receipt extraction originally checked individual schemas without
   requiring complete revision coverage. Migration now requires one receipt per
   API revision, unique contiguous expected revisions and matching next revisions.
3. Legacy event vectors could differ from the domain receipt stream. Migration
   now compares the flattened sequence-ordered domain events exactly.
4. Pending encounter check counts/support/session could disagree with historical
   evidence. Migration now checks these fields against the full legacy history.
5. The first repository draft allocated replay storage but did not populate it.
   New domain actions now return recorded trusted contexts for durable storage;
   legacy contexts are not invented.
6. A conflicting preexisting outbox row could survive a new event insertion.
   New-write outbox conflicts now fail the transaction; migration only accepts
   equal historical contents.
7. Compact saves originally retained the legacy reader's required empty history
   fields. An old binary could ignore the new marker and treat that lost history
   as valid. The tagged compact envelope now omits `receipts`, `evidence` and
   `events`; the old reader rejects it. Current readers explicitly distinguish
   strict legacy and versioned compact shapes and reject missing/unknown markers.
8. API encounter records and the domain pending map could disagree. Both legacy
   migration and compact validation now require the exact active quest-to-instance
   mapping to match the domain pending state.
9. Profile revision was initially checked only for compact loads. It is now
   compared before every load branch, including legacy migration and missing-save
   initialization. A nonzero-revision profile with missing progress fails closed.
10. Legacy migration could retain orphan records in newly introduced external
    tables. It now rejects unmatched receipt/domain/encounter/novelty/reward rows
    and rejects existing recovery checkpoints or invented legacy replay contexts.

The repository now exposes an explicit `reconstruct_legacy` recovery/export
method. Under the owned-profile lock it reads complete durable receipt,
encounter, evidence and event histories, invokes the aggregate's reconstruction
and complete migration validators, and returns a legacy-format candidate.
It does not overwrite current progress. Missing history, inconsistent counters,
invalid receipt/event correspondence or changed public learning summaries fail
reconstruction. This source-review confirmation does not claim a production
recovery drill has been run.

All concrete defects raised in this review have source changes addressing them.
These are source-review observations. The integrated test results belong in the
implementation status record after the actual checks complete.

## Remaining verification boundaries

The current-save digest and independently stored high-water checks protect the
loaded working checkpoint; they are not a full scan of all archived history on
every request. Durable receipts, events, encounters and novelty keys still grow
with profile use. The bounded claim concerns loaded state and request work, not
total database bytes.

Recovery checkpoints retain two periodic snapshots. Persisting new trusted replay
contexts is a prerequisite for replay recovery, not evidence that a full
backup/restore or corruption-recovery drill has passed. Preexisting legacy
commands did not store those contexts, so reconstruction must retain this
historical limitation rather than creating guessed facts.
