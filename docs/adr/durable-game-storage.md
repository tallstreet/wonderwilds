# Durable game storage

Status: implemented and validated with disposable PostgreSQL and browser checks.
This supersedes the proposed
storage layout in `../LONG_PLAY_STORAGE_PLAN.md` where the implementation differs.

The PostgreSQL repository owns command idempotency. It locks the authenticated
parent's profile, migrates a legacy aggregate when necessary, and checks the
durable API receipt before revision and rate checks. Equal requests receive the
exact original JSON response; a changed body cannot reuse a command ID. All new
state, receipts, evidence, grants, outbox events and item novelty commit together.

Migration `0003_game_storage.sql` adds profile-owned receipt, encounter,
checkpoint and mastery-item tables. Each forces RLS and cascades on profile
deletion. Parent exports include the new records and replay metadata. Existing
migrations remain immutable.

The current save is a versioned compact checkpoint: gameplay facts, explicit
domain/event/evidence high-water marks, at most one active encounter, and bounded
mastery projections. Complete receipts, completed encounters and histories remain
durable without being copied into every subsequent save. Each skill projection
keeps at most eight eligible observations; a separate unique item ledger prevents
old practice from becoming novel when it leaves that window. Full recomputation
remains the reference policy and is used to verify migration/reconstruction.

The repository checks the save digest, independent profile revision, indexed
durable event/evidence high-water marks and active encounter agreement. Corrupt,
inconsistent or unsupported state fails closed instead of becoming a fresh save.
Legacy migration validates receipt coverage and exact historical copies before
writing the new format. The compact wire envelope deliberately cannot be read as
an old aggregate; rolling back a binary must not silently lose retry history.

Two periodic recovery checkpoints are retained, at intervals of 256 API revisions.
Durable receipts, novelty, evidence and event records are not rotated. New domain
actions record trusted replay context; historical legacy actions did not record
that context and it is not invented during migration. This is not a completed
backup/restore or automatic corruption-recovery drill.

The durable path replaces the lifetime command cap with a persisted limit of
120 new commands per profile per 60-second window. Exact retries bypass this
limit. Existing request-size, concurrency and SQL-timeout limits remain. The
standalone aggregate executor retains its embedded receipt guard and refuses
compact checkpoints; it is not the long-running PostgreSQL path.

Old saves are upgraded lazily under the profile lock. Development tests use only
fresh disposable databases. Deployments need an operator backup and a compatible
binary; no script in this change migrates the user's persistent demo. A validated
legacy reconstruction helper supports controlled export/recovery, rather than
restoring a stale pre-upgrade snapshot over newer progress.

The bounded claim concerns working-save size and indexed per-command access.
Total durable history grows with use. Operational storage budgets, approved
retention, deployed cleanup workers, restore drills and privacy review remain
separate requirements. See `../IMPLEMENTATION_STATUS.md` for actual test evidence.
