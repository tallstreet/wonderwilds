# Wonderwilds implementation planning

Updated: 29 October 2026.

The earlier Starwhisker Island frontend-first plan has been superseded. Wonderwilds now has a Rust backend, versioned contracts, a server-backed Meadow adventure, twelve illustrated character profiles and 75 connected exploration destinations; do not rebuild those foundations or follow the old suggested frontend layout. Exploration destinations are not evidence of server-authoritative regional story integration.

- [Companion progression plan](COMPANION_PROGRESSION_PLAN.md) is the active gameplay sequence: individual levels, training, collection, teams, friendship, transformations and regional discovery.
- [Quest generation contract](QUEST_GENERATION_CONTRACT.md) defines authored JSON
  episodes now and the future model context/proposal boundary.
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

The next delivery is JSON-driven personal quest episodes with choices, supported
activities and continuation hooks. The game has no final ending: completing an
episode leaves new threads and a choice of another adventure. A fixed authored
bank proves the format before live generation or fine-tuning is added.

Then verify the Orchard-unlocked two-slot journey and implement optional companion
roles, automatic friendship participation and Orchard/Cove story integration.
Regional celebrations close local arcs and hand off to continuing adventures.
PostgreSQL storage now uses bounded working checkpoints with
durable duplicate prevention; its request-rate policy replaces the lifetime cap.
The [durable storage decision](adr/durable-game-storage.md) records the implemented
format and migration policy. The [original storage design](LONG_PLAY_STORAGE_PLAN.md)
retains the acceptance reference; operational retention and restore drills remain.

Public release separately requires reviewed parent identity and lifecycle flows,
save import, parent learning views, educator/content/art/accessibility approval,
security review and operational deployment/backup evidence. These are outstanding
gates, not features implied by the current local demo. Future model generation
uses validated quest documents and curated context; live AI remains deferred.
