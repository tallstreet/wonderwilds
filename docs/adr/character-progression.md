# Character progression API and development presentation

29 October 2026.

Expose the existing deterministic domain rules through `GameCommand@1`:

- `help_friend`, with `creatureId` and `completedQuestId`, records one distinct completed same-region story moment for an owned friend.
- `transform_companion`, with `creatureId` and boolean `enabled`, toggles the existing reversible cosmetic milestone. Enabling requires the authored friendship threshold and regional story flag.

The server derives eligibility from persisted domain state. Clients cannot supply points, rewards, story flags or learning evidence. Existing command IDs, revisions, ownership checks, transactional persistence and response receipts apply. These actions do not create learning evidence or Explorer XP. Repeating a helping moment with a new command is rejected; retrying an identical command returns its original receipt.

Snapshots add optional `friendship` entries (`creatureId`, `points`), `transformedCreatures` transformation IDs and `availableHelpingMoments` entries (`creatureId`, `completedQuestId`). Optional fields preserve old development fixtures and allow clients to hide controls when progression state is unavailable. Generated Rust/TypeScript models and shared positive/negative fixtures change together. Deploy the server before clients that send the new command variants; older clients that enforce closed response schemas require a compatible upgrade.

The local development journal uses server-owned state for memory actions, glow controls and authored dialogue selection. Rendering dialogue never changes progress. Unknown or incomplete state yields profile information without earned milestone claims. Glow is represented with text and styling while distinct transformation art remains unavailable. This is a development preview: pending content review, missing art and regional integration remain release gates. The sidecar's release activation and frozen gameplay pack hash do not change.

Existing stored aggregates load without a data migration: friendship and transformation state already belong to the domain. New receipts and returned snapshots contain the added fields. Rollback must retain a reader compatible with these receipts and command variants; do not remove persisted progress or receipts to force an older build to load them.
