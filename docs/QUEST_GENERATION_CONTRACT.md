# Quest documents and the future generation boundary

Status: authored development episodes and a pure proposal-validation adapter.
There is no model provider call, fine-tuning job, conversation capture or generated
quest activation in this change. Documents and the catalog remain pending review.

The adventure continues through episodes. A quest can resolve a small problem,
leave a memory and hand off to another thread; that resolution is not a global
game ending. The initial four Meadow documents can be revisited with different
choices. This authored bank is finite and does not promise unlimited unique prose.

## The artifact we iterate on

`QuestDocument@1` is both the authored JSON shape and the proposed future model
output. The playable examples are under `content/quests/`:

- `pip-lantern-trail.v1.json`
- `tansy-welcome-garden.v1.json`
- `nib-sharing-picnic.v1.json`
- `flutter-pattern-parade.v1.json`

Each document declares its identity/version, companion, region, title, interest
tags, entry node and a bounded node graph. Nodes use three capabilities:

| Node | Data | What the engine does |
| --- | --- | --- |
| `choice` | Scene text and two or three labelled choices with destination IDs | Records the selected path and moves to its permitted destination |
| `activity` | Scene text, an approved `activitySlotId` and the next node ID | Issues a server-assessed activity using that trusted slot; verified completion advances the scene |
| `handoff` | A local resolution and continuation tags | Resolves this episode and presents further adventures/revisits |

The graph has at most sixteen nodes. All nodes must be reachable, references must
exist, cycles are rejected and every handoff path must include an activity. A
single episode is bounded; continuing play starts another episode rather than
looping forever inside one graph. The examples use distinct final choices and
handoff text/tags so the next thread follows the chosen path.

A document has no executable code, answer keys, correctness claims, reward
quantities, state-write instructions, consent flags or release approvals. The
separate trusted catalog defines level-four personal-quest eligibility and the
one-time cosmetic memory item. The server owns those rules. Supported activity
completion remains a valid route, and character level is separate from academic
ability.

An activity slot refers to existing gameplay capability. For example, `pip-count`
binds the approved M06 skill and `missing_quantity` renderer/assessor. A future
model can propose a scene using that slot; it cannot redefine the assessment or
invent an unsupported interaction. New mechanics need a separately implemented,
versioned capability before a document may reference them.

An active episode pins its document, activity slots and trusted reward policy.
Editing the authored bank affects subsequent episodes; reload resumes the pinned
one. Once a profile has personal-quest history, its compact envelope uses version
two so older runtimes fail closed instead of dropping that history. The current
reader remains compatible with pre-quest legacy and compact version-one saves.

## What we pass to the model later

`QuestGenerationContext@1` is assembled by the server after authorization,
redaction and permission decisions. It is not a browser claim of progress. The
complete synthetic example is
[`generation-context.example.json`](../content/quests/generation-context.example.json).

| Field | Meaning and authority |
| --- | --- |
| `requestId`, `revision`, `contextDigest` | Bind one proposal to one exact server context and state revision |
| `contentVersion`, `rulesVersion` | Select compatible catalog/runtime rules |
| `world` | Canonical world facts, current region and confirmed owned companion IDs |
| `progression` | Explorer level plus the target companion ID and its character level |
| `learning` | Server-selected eligible skill IDs and whether support is available, encouraged or together |
| `interests` | Bounded preference tags with `player_selected` or `curated_conversation` provenance |
| `conversationMemory` | Optional curated summary with explicit `not_shared` or `shared_summary` permission |
| `recentChoices` | A bounded selection of actual prior quest/node/choice IDs |
| `openThreads` | Bounded narrative hooks carried forward by the server/editor |
| `allowed` | Exact permitted actors, regions, activity-slot bindings, interest tags and continuation tags |

Explorer progress, character growth and learning needs remain separate inputs.
For example, a level-four Pip can appear in an accessible, supported number
activity. Interest changes can influence the setting or next hook without
changing correctness, rewards or mastery requirements.

Facts and preferences have different authority. A curated summary such as “enjoys
lantern stories” is a subjective hint; it cannot overwrite a repaired-bridge fact,
claim an earned memory, change a level or supply instructions to the engine. Raw
conversation transcripts, parent identity, contact information and authentication
secrets are absent from this contract. The current game does not collect the
conversation summary. A future capture/curation flow needs its own reviewed
permission, privacy and retention implementation.

`not_shared` requires an empty summary and forbids conversation-derived interest
tags. `shared_summary` can carry at most 500 characters; a conversation-derived
tag also requires a nonempty curated summary. The adapter enforces these structural
permission rules, but a string schema cannot prove that a future curator removed
all sensitive details. That is a separate data-handling obligation.

The digest is lowercase SHA-256 over compact UTF-8 JSON containing every context
field except `contextDigest`, with object keys sorted recursively and array order
preserved. `quest_generation::bind_context` supplies the Rust reference behavior.
Changing the selected companion, level, skill, memory or fact changes the binding.

## What we expect back

The schema-valid synthetic
[`generation-proposal.example.json`](../content/quests/generation-proposal.example.json)
wraps a complete `QuestDocument` with `schemaVersion`, the exact `requestId`,
`expectedRevision` and `contextDigest` from its request. It contains the actual
typed nodes and a digest verified by the Rust regression suite, so it can serve
as an editable example without placeholder fields. Canonical definitions live in
`contracts/schemas/v1.json`, with generated Rust/TypeScript models and shared
positive/negative fixtures.

The pure `director::quest_generation` adapter validates:

1. Context schema, byte bounds, compatible content/rules versions, permissions and
   the recomputed context digest.
2. Proposal byte/JSON/schema bounds and exact request/revision/digest echoes. The
   caller supplies the current revision and must recheck it under the commit lock.
3. Target companion identity/ownership, current region, catalog activity bindings,
   eligible learning skills, approved thematic tags and trusted level eligibility.
4. Shared domain graph validation, including reachable destinations, meaningful
   branches, no cycles, no activity bypass and valid handoffs.

The adapter can return a deterministic authored fallback in the same proposal
shape when a supplied proposal is absent or rejected. It records whether the
source was an authored fallback or a generated proposal and keeps
`content_review_required: true`. Neither result commits state or activates text.
The current playable runtime still loads authored documents from the catalog.

These checks prove structural compatibility and permitted references. They do
not prove that arbitrary prose is child-appropriate, accurate, coherent with the
world or consistent with a prior conversation. Natural-language contradictions,
reading burden, character voice and narrative quality require future moderation,
evaluation and content review. The first contract carries context to support
coherence; it does not claim to solve semantic continuity automatically.

## The next episode and eventual fine-tuning

After an actual player choice or verified activity, the server records the new
canonical outcome and the selected handoff. A later context can carry that
choice, its continuation tags and a bounded curated open-thread summary. The
model sees authoritative outcomes alongside preference hints, and proposes a new
episode. An episode resolution never sets a global “game over” flag.

For now, edit the authored JSON, run schema/graph validators, and play both paths.
Keep approved examples separate from rejected or edited proposals. Later training
records can pair a versioned context with the accepted document and explicit
review/evaluation labels; held-out examples should include changed interests,
supported learning, repeated visits and conflicting conversation hints. Model
training, provider selection, consented data collection, budgets and deployment
remain separate future work. No synthetic example here represents a real child.
