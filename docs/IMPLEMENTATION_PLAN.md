# Wonderwilds implementation planning

Updated: 29 October 2026.

The earlier Starwhisker Island frontend-first plan has been superseded. Wonderwilds now has a Rust backend, versioned contracts, a server-backed Meadow adventure, twelve illustrated character profiles and 75 connected exploration destinations; do not rebuild those foundations or follow the old suggested frontend layout. Exploration destinations are not evidence of server-authoritative regional story integration.

- [Companion progression plan](COMPANION_PROGRESSION_PLAN.md) is the active gameplay sequence: individual levels, training, collection, teams, friendship, transformations and regional discovery.
- [Wonderwilds roadmap](WONDERWILDS_IMPLEMENTATION_PLAN.md) retains architecture, curriculum, privacy and broader release requirements.
- [Implementation status](IMPLEMENTATION_STATUS.md) records delivered systems, validation evidence and outstanding integration work.
- [First milestone review](MILESTONE_REVIEW.md) preserves acceptance evidence for the earlier guest adventure.

Use the companion plan for the next gameplay work. Preserve the guest save format and regression coverage while extending authoritative server progression. Historical review documents describe their original milestone; they are not the current implementation queue.

## Current delivery

The first persistent companion loop is implemented and verified: recruit Pip, select Pip,
complete a server-assessed training milestone, reach level two, see the new
reaction and reload with the same progress. The same rules and controls support
all four Meadow friends. Domain/content, contracts/API, frontend and isolated
browser QA were integrated and checked; see the current implementation evidence.

Next add richer Meadow milestones, verify the Orchard-unlocked two-slot journey,
and implement optional companion roles, automatic friendship participation,
Orchard/Cove story integration and the
supported ending. Before sustained training, implement bounded history storage
with durable duplicate prevention; do not remove the 2,000-command guard alone.
The [long-play storage design](LONG_PLAY_STORAGE_PLAN.md) specifies the required
receipt, checkpoint, migration and acceptance work; it is not yet implemented.

Public release separately requires reviewed parent identity and lifecycle flows,
save import, parent learning views, educator/content/art/accessibility approval,
security review and operational deployment/backup evidence. These are outstanding
gates, not features implied by the current local demo. Live AI remains optional.
