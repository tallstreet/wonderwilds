# Encounter abandonment and completed director candidates

Independent review identified two executable-contract defects in the development
foundation: encounters advertised abandonment without any server command capable
of closing them, and the director accepted completed beats as playable candidates.

The existing `GameCommand@1` union now accepts
`{"type":"abandon","encounterId":"..."}` through `POST /api/v1/commands`.
This aligns the command surface with the already-advertised `abandon` permission.
Clients must generate a new command ID, supply the current revision and preserve
that exact command for uncertain retries, just as for attempts and hints. The
shared closed schema, both generated languages and positive/negative fixtures
change together; existing command shapes and schema versions remain supported.

The domain removes only the matching pending quest/encounter and emits
`QuestAbandoned`. The API records server-issued `abandonment` evidence with
`not_assessed`, archives the encounter without completing it, and commits state,
evidence and outbox events in the existing transaction. It grants no story reward
or independent learning credit. A new encounter can then start; attempts on the
closed encounter conflict. Retrying the cancellation cannot close a later task.
Existing JSON snapshots load with `abandoned: false` through a serde default.

This adds an internal event/action variant: older binaries cannot be assumed to
read new domain receipts. Rollback must retain a compatible reader or use an
explicit, reviewed state migration; do not delete receipts or learning evidence
to make an older binary load. No database table migration or destructive reset
is needed for forward loading of existing development snapshots.

The director excludes completed beats before validating eligible candidates.
A frontier with no unfinished candidates returns `NoPlayableCandidate`; it must
not propose a transition the domain would reject as already completed.

Regression coverage includes API cancellation/restart, exact retry, foreign
profile ownership, rejection of late answers, old snapshot loading, domain
replay/no rewards, PostgreSQL rollback/reload/idempotency, and completed-only and
mixed director frontiers. Guest-mode behavior and pending release gates remain
unchanged.
