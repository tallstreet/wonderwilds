# Authored story payload builder

This is a development projection over `wonderwilds_domain::story::State`.
The default path calls a local deterministic mock provider and reads no profile, account,
learning, browser or free-text user data. Existing `director::core` and Director v1 contracts are unchanged.

Public functions return schema-validated JSON:

- `build_plan_input(&State)`: small area, authored opportunity, state-dependent
  legal future bindings, and context.
- `authored_plan(&State)`: authored mission definition, not a generated plan or a
  new offer. It remains inspectable after completion; `opportunities` is then empty.
- `build_scene_input(&State)`: default preferences, actual inventory and facts,
  presented-introduction history, compact mission progress and currently legal actions.
- `authored_scene(&State)`: deterministic local mock scenes through the provider seam,
  validated against the current input with a canonical authored fallback.
- `scene_with_provider(&State, &impl SceneProvider)`: builds input and canonical
  presentation metadata, calls the provider, then validates or falls back.
- `MockSceneProvider`: local scene responses for the seven connected authored stories; no
  network, provider SDK, random output or model credentials.
- `resolve_action(&State, handle)`: resolves only an exact current revision/index
  handle to its bound domain action. Execute with the same expected revision and
  stable command ID; the domain rechecks preconditions and idempotency.
- `payload(&State, Mode::Plan | Mode::Scene)`: adds `story-proposal-1`, content version
  and state revision outside the model input. The headless harness needs no account or
  request correlation; the separate story API supplies authenticated scoping.

## Sources and limits

Locations, route descriptions, character descriptions and narrative are authored
constants here. Inventory, bridge state, supplies, Pip's presence, progress, events
and woodcutter introduction acknowledgement come from read-only runtime getters.
The runtime owns mechanics; descriptions explain its fixed bound recipes. Golden
fixtures and journey tests verify these projections against real state transitions.
No claim is made that the current small catalogue supports arbitrary mission planning.

The series has fixed default preferences and seven authored opportunities. Existing
planks and an already-repaired bridge reduce the authored milestone list; the neutral
goal is to help Pip cross rather than claiming the player repaired an existing bridge.
A paused mission retains its narrative/progress with explicit `status: abandoned`.
History is a bounded deterministic event window plus relevant persistent facts;
no model-generated summarization or psychological inference is used.

Implementation refinements to the design shapes: scene input includes mission
`status` so resumed/paused context is explicit; authored scene output includes
`sceneId` and presentation `entryId` to bind acknowledgements. Entries support
narration when no character is present. The runtime canonical presentation supplies revision-bound scene IDs and the
woodcutter introduction. The mock preserves that exact introduction until Continue
is acknowledged; other entries are selected from the actual input state. Rendering/building never acknowledges an entry automatically.
Available actions include accept/cancel lifecycle variants. Responses retain all eligible actions (up to 12), preserving access to every
needed action. Ordinary dismissal
requires no state change and is not a domain command.

The builder does not load the existing API snapshot, which mixes story and learning
fields. It does not publish endpoints, persist database state, create browser art,
verify physical walking or accept model-proposed missions. The `Story*` contracts are additive definitions in the shared schema;
the separate API consumes this module for its authenticated story routes. Validating a JSON shape is not validating arbitrary prose safety.

## Checks

```sh
cargo test -p wonderwilds-director -p wonderwilds-contracts --locked
cargo clippy -p wonderwilds-director --all-targets --locked -- -D warnings
(cd contracts && npm test && npm run typecheck && npm run check:generated)
```

`fixtures/plan-initial.json` and `fixtures/scene-workshop.json` are exact golden
projections from the authored initial state and accept/collect/travel journey.
Shared contract fixtures cover populated/null missions, version rejection, unknown
fields, educational leakage, free-text preference injection and invalid actions.

The developer runtime caps history at 256 commands (including acknowledgements).
At that bound the builder and action resolver return `CommandBudgetExhausted`
instead of emitting a supposedly playable payload. Checkpoints and historical plan
inspection remain available. PostgreSQL callers use a separate compact state/receipt boundary and can continue
past 256; see [long-play storage](../../../../../docs/LONG_PLAY_STORAGE_PLAN.md).

## Mock scene provider

The scene path is `State -> build_scene_input -> MockSceneProvider -> validate_scene
-> StoryScene`. The provider receives JSON input and canonical fallback metadata,
not mutable runtime state. It selects bridge offer, gathering, workshop introduction,
exchange, return with planks, repair, shared crossing, completion, pause or exploration
responses using location, inventory, mission status and currently legal actions.
The authored successors are a shared picnic and welcome lights on both banks. There is no
model generation or educational input/output.

The validator checks the scene schema, exact scene ID, unique entry IDs, speakers
present in the input, and a complete unique set of current action handles. A pending
woodcutter introduction must retain its exact canonical entry. Unknown grant/state
fields fail the closed schema. Failed or invalid proposals use the canonical authored
scene with all legal actions. This does not validate the truth or safety of arbitrary
model prose; replacing the trusted local mock with a network model requires further
content controls. Actions still go through the domain and authenticated API.

Responses are deterministic for a given state and mock implementation. Persisted
command receipts remain the API's exact-retry authority even after future prose edits.
Tests cover each mission stage, pause/resume, absent characters, immutable input state,
invalid bindings/grants/introduction edits, and canonical fallback.

## Connected stories and upgrades

The current registry is `pips_bridge -> pips_picnic -> welcome_lights -> humming_hollow -> cosy_corner -> brambles_visit -> moonlight_gathering`. Completion
keeps the current mission visible until the child explicitly accepts its successor
beside Pip. Scene `missionId` is optional in the wire schema for old responses and
always emitted by the new builder. Goal, narrative and milestones describe the
current mission; plan opportunities and the accept choice describe the next story.
Completed goals persist in `context.memory.recentMissions`. Inventory and object facts
come only from runtime state; the bridge, shared picnic and lit banks survive later
stories. Each source capability uses its remaining quantity, including partial picks.

New payloads use `pip-stories-3`; contracts still accept `pip-bridge-1` and `pip-stories-2` receipts.
New action handles have the `story-v3-` prefix. Old index handles cannot be reused
as new commands because adding a successor could otherwise reinterpret an old
travel handle as mission acceptance. Exact stored command retries still return their
original response through the repository. Tests cover the legacy checkpoint upgrade,
full successor sequence and reload after every step. The exhaustive bridge exploration
remains bounded to the first mission; targeted sequel coverage avoids a state-space
explosion while testing every new action and shared outcome.


## Broader authored mechanics

The four successors add inspection/clues, a lasting instrument-home choice, two
cosy-corner designs, a distinct Bramble escort, and a gathering with independent
invitations and preparations. Inspect/talk/choose/invite are bounded wire action
variants; all mechanics remain authoritative domain actions. The instrument choice
selects its home; the player carries it when moving it, then places it explicitly.
The gathering uses a humming circle so it does not silently move that instrument.

`broader.rs` projects known flags, character locations, options and outcomes into
compact facts and local objects. Old investigation details are dropped from later
mission context while lasting outcomes remain. Completed goals are bounded to the
seven authored stories. Explicit facts explain flexible lantern/gathering ordering,
so scene writers do not have to infer future rules from a current action list.

Bramble is distinct from the woodcutter and appears only when the broader series
starts. Every present character has dialogue. The mock records a child's confirmed
inspection/conversation result in the next response before showing a next-step
hint; optional stump exploration never overrides a remaining required action.
Conversation results use a fixed map of authored runtime events, not arbitrary
user text. Mandatory woodcutter introductions remain intact. Returned presentation
still does not acknowledge an introduction automatically.

The broader journey regression checks both instrument homes, both corner designs,
both gathering sites, different invitation/preparation orders, all seven completed
goals, reply visibility, physical instrument placement and conversations after
leaving a finished scene. Historical v2 checkpoints upgrade through strict domain
replay; database tests preserve exact v2 public receipts while new writes use v3.
Frozen reviewed v2 evaluation artifacts are not silently relabeled or rewritten;
new evaluation runs use current v3 projections and separate review provenance.

## Quality-review corrections

Authored dialogue now states carried quantities and remaining supplies, names the
remaining lantern bank, and acknowledges discoveries and the selected corner
design. Instrument pickup labels distinguish choosing/carrying from walking to
its home. Route hints name a reachable destination; pause labels refer to the
current mission. Workshop lines preserve the required first introduction without
repeating it after acknowledgement. Corpus regressions cover these distinctions.

Earlier sealed inputs and reviews remain historical snapshots. Prepare a fresh
run after changing authored prose; do not copy approvals onto revised candidates.


The API now has an optional development Modal provider in `api/src/story/modal.rs`.
It enriches authenticated snapshot reads after database locks are released, then
uses this module's validator. The pure director and default authored functions
remain offline. The API rechecks identity/state after generation and retains
canonical fallback and unchanged command receipts. See the root README for opt-in
configuration and scope; valid structure does not establish prose safety.


## Live mission plans

The API's optional planner uses the same `prompts/mission_baseline.txt` plus
`prompts/mission_dependencies.txt` bytes as the offline evaluator. The original
baseline request hashes remain valid. `planning_evaluation::assess` is the shared
pure completion validator: the offline wrapper checks scenario provenance, while
the API supplies an authorized post-acceptance state and a short search deadline.
Deadline exhaustion is unproven and falls back; the model never sets completion.

Accepted plans are stored by the API in a separate profile-owned table alongside
unchanged domain checkpoints. The API projects plan labels from actual milestone
receipts and supplies the saved brief to scene generation. This module still does
not accept arbitrary new mechanics or change domain mission definitions. See the
root README for activation and persistent-plan behavior.

## Neighbourhood exploration

Five additive neighbourhood locations and defined character missions are supplied
by the pure domain exploration catalogue. They use the existing travel/talk/inspect
actions, appended after old candidates so legacy action indexes stay stable.
`StorySceneInput.neighbourMissions` is an optional bounded journal projection;
old receipts omit it. Models receive these statuses and only currently legal actions.
The mock uses the canonical neighbourhood greeting; the optional scene provider can
supply prose through the existing validator. Pip's plan search excludes detours
outside its capability graph. Neighbourhood mission acceptance is deterministic
and does not request a generated mission plan.

## Saved generated neighbourhood content

Arrival-time generation now supplies a validated `GeneratedNeighbourhood` containing
character identity/display art, greeting, mission premise and one to three local
investigation steps. The API commits the spec inside an internal `GeneratedTravel`
domain receipt. This internal action is never advertised or accepted as a public
browser payload. Replaying it checks the same domain preconditions and spec bounds.
`neighbourMissions` optionally carries the saved content, generation counter,
current step and generation source. The scene builder resolves names and canonical
text from that saved content; model IDs never determine new commands or rewards.
Completed slots can be replaced on a later arrival, while unfinished work remains
stable. Existing pure authored projections and receipts stay compatible.

New neighbourhood steps may optionally save a `locationId`, a `discovery`,
`learningBridge` and a fixed `mechanic`. Missing mechanics mean inspection and
missing destinations mean the mission's home area, preserving older saves.
Supported project mechanics gather reserved wood/berries/glow pebbles, craft
planks/baskets/lanterns, deliver supplies, build a seat, place a lantern or choose
a leafy/starry design. Generated specs must conserve their mission supplies and
use connected destinations. Quantities and effects are authored domain rules.
Project steps use existing collect/assemble/choose wire actions with generation-
and step-bound targets. The director projects their actual saved labels and
permanent project objects. Both design options are creative choices, not assessed
answers. Bag quantities include supplies reserved to active/archived projects.

Optional `completeOnLastStep` allows projects to finish at their destination.
Old missing/false values retain the final return conversation. Up to six steps are
supported; current projects use three or four. The server selects supported plans
and the model writes their narrative. Fresh responses are checked against that
plan and recent-copy fingerprints; twelve discovery and six project fallbacks
remain available. Generation receives compact avoidance hints from the latest
story per area, not complete prior stories or lifetime history. Original authored
neighbourhood missions remain unchanged without generation credentials.
