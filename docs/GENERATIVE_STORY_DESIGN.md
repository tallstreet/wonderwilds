# Mission-led generative storytelling

Status: authored runtime, payload builder, persistence and playable mock-driven world slice implemented; broader missions/model integration pending. Revised: 30 October 2026.

Implementation update: the [story implementation plan](STORY_IMPLEMENTATION_PLAN.md)
now delivers authored Rust mechanics, deterministic planning/scene payloads,
authenticated PostgreSQL story saves, an opt-in `/?story=1` Phaser adventure and a
`/story.html` scene inspector. A deterministic mock provider supplies varied scene
responses across three connected missions—Pip’s bridge, Pip’s picnic and Welcome
lights—with binding validation and authored fallback. Completed history and world
changes persist between missions; each next mission is explicitly accepted.
The broader design below remains a proposal for integration. The
[builder notes](../backend/crates/director/src/story/README.md) record implemented
refinements and limits. This separate slice does not integrate
the existing assessed bridge, player preferences or model generation; source-map
entries below still identify work needed for that broader live-game design.

## Purpose

**Given where the player is, what has happened, what they like and what they are
trying to achieve, what should we show them next?**

The model plans a mission and then writes its scenes. Code owns world state,
inventory, action execution and completion. Input is a compact state projection;
output is a proposal, never a replacement save.

Normally the player has an active mission and a selected focus. A mission should
usually involve five to ten meaningful actions, but action count is a pacing target,
not a completion requirement. Completion provides a satisfying stopping point and
an optional next mission. Declining or abandoning leaves an accessible offer without
forcing acceptance or implying that a character suffers when the player leaves.

Education is outside this protocol. Curriculum, activity selection, instructions,
answers, assessment, support and mastery stay in separate systems. Building Pip's
bridge is a world interaction, not an arithmetic assessment. No academic success
is required to finish it.

This updates the future story direction in section 7 of the
[roadmap](WONDERWILDS_IMPLEMENTATION_PLAN.md). Existing `DirectorInput@1` and
`DirectorProposal@1` remain unchanged: their authored implementation rejects new
model dialogue. The headless implementation adds separate Story contracts and
mechanics without changing those existing contracts; inference remains absent.

## Four building blocks

| Building block | Owner | Purpose |
| --- | --- | --- |
| Opportunity | Code/content | A useful, currently possible world outcome, such as repairing a bridge and crossing with Pip |
| Capability | Code | A reusable action with legal bindings and deterministic effects, such as collect, travel, exchange or assemble |
| Mission | Model proposes; code accepts | A goal, narrative brief and ordered milestones leading to a supported outcome |
| Scene | Model proposes; code accepts | Short narration/dialogue and choices among actions currently executable |

The model chooses a supported outcome, connects it to characters and interests,
composes milestones and writes scenes. It does not invent mechanics or arbitrary
world effects. The catalogue must offer genuine alternatives: one preassembled
chain of seven mission-specific operations only tests paraphrasing.

Reusable actions use bounded arguments from the start. For example, the same
`collect` action can gather wood from an allowed source or picnic supplies from
another. The `assemble` action can repair an available structure using a registered
recipe. Changing a noun does not create a new recipe or renderable object.

Opportunities prevent pointless but executable errands. Capabilities prevent
unplayable promises. The model supplies narrative purpose and composition within
those boundaries. More capable and varied world content increases creative scope;
fine-tuning cannot compensate for a catalogue with only one possible journey.

## Runtime flow

1. Build relevant context from committed state and player-known memory.
2. When a new goal is needed, request a mission using nearby opportunities and
   capabilities, including those that become available after earlier actions.
3. Validate the outcome, narrative brief and milestone feasibility. Save the plan
   as an offer; acceptance activates it and binds its resources and versions.
4. Request a scene for the current situation. Supply the narrative brief, compact
   progress and currently executable actions, not the complete dependency graph.
5. Validate and save the scene before presentation. Record what was actually shown.
6. Execute the selected action through normal server commands. Commit its effects
   atomically, then evaluate milestone and outcome conditions.
7. Continue from that state. On completion, acknowledge the payoff and optionally
   offer a new mission. Resume saved valid scenes after reload without inference.

Walking and routine animations do not require model calls. Leaving, navigation and
stopping remain available in the ordinary UI. Invalid or unavailable model output
uses action-derived fallback text and controls for the same mission. Keep an authored
mission offer available when planning fails; no bespoke fallback is required for
every possible wording of a generated mission.

## Missions are milestones, not rigid action scripts

A mission has an explicit supported outcome and a short ordered list of milestones.
A milestone references a registered condition, not a required button click. “Have
suitable wood” can be satisfied by compatible existing inventory or collecting it
from any permitted source. The player need not perform a prescribed collection
operation if the condition is already true.

Code evaluates conditions against authoritative state. The planner cannot write
predicates, scripts or arbitrary JSON patches. Opportunity definitions list the
conditions suitable for that outcome. A plan may omit unnecessary already-satisfied
work, but its remaining milestones must lead to the outcome. Completing the listed
milestones alone never substitutes for checking the outcome itself.

Milestones are an ordered presentation of progress, not locks on ordinary world
actions. A legitimate early action can count when its condition is evaluated.
Conditions specify whether they describe current state or a durable event. When a
milestone is satisfied, its receipt persists: consuming wood later does not undo
“obtain wood.” However, later actions still check current resources. Outcome
conditions are checked at completion and cannot be bypassed using old receipts.

For the first version, use one focused mission and an ordered milestone list.
Defer arbitrary branching graphs, optional steps inside a mission and shared scarce
resources between concurrent missions. Additional missions can later use the same
format once resource ownership and focus switching are defined.

Do not silently rewrite an accepted goal or erase progress. Recompute available
ways to reach the remaining milestones after detours. If world changes invalidate
the goal, provide an explicit recovery or replacement offer, preserving completed
consequences. Cancellation cannot undo an already repaired bridge.

## Full proposed input and output structures

These TypeScript shapes specify logical JSON. They are not an implemented schema.
Fields are required unless marked `?`; objects are closed; unions are discriminated.
Before implementation, add a new versioned JSON Schema, fixtures and generated
models rather than changing existing v1 meanings.

### Server envelope

```ts
type Request = {
  protocol: "story-proposal-1";
  requestId: string;
  stateRevision: number;
  contentVersion: string;
  policyVersion: string;
  input: PlanInput | SceneInput;
};

type Response = {
  requestId: string;
  stateRevision: number;
  output: MissionPlan | Scene;
};
```

Only `input` goes to the model. The caller selects the output schema and attaches
response metadata; the model does not echo IDs or versions. Code rechecks the
retained revision at commit. Store prompt/model/capability versions and accepted
outputs for replay; replay never calls the model.

### Common context and reusable actions

```ts
type Context = {
  trigger: string;
  player: {
    language: "short_simple" | "simple" | "standard";
    interests: string[];
    avoid: string[];
  };
  locationId: string;
  inventory: { itemId: string; quantity: number }[];
  memory: {
    facts: string[];
    recentEvents: string[];
    recentMissions: string[];
  };
};

type Action =
  | { type: "collect"; sourceId: string; itemId: string; quantity: number }
  | { type: "travel"; destinationId: string }
  | { type: "exchange"; characterId: string; recipeId: string }
  | { type: "assemble"; objectId: string; recipeId: string }
  | { type: "accompany"; characterId: string; destinationId: string }
  | { type: "talk"; characterId: string; topicId: string };

type Narrative = {
  premise: string;
  participantIds: string[];
  payoff: string;
};
```

`trigger` is a short server-written event description, not an instruction from
user text. Inventory and facts are relevant projections, not the full save.
`recentMissions` preserves themes, action patterns and outcomes after individual
walking/conversation events leave the recent window. Preferences are explicit
choices or modest observations; one decline is not a permanent dislike.

The action vocabulary is deliberately small and extensible. Argument tuples are
validated against the registry, not independently combined allowlists. A legal
recipe/character pair does not imply that every character can perform every recipe.
Recipes own quantities and effects; the model cannot change them. Travel and
accompany complete on verified arrival, not on starting navigation.

### Planning input

```ts
type PlanInput = {
  mode: "plan_mission";
  context: Context;
  area: {
    locations: { id: string; description: string }[];
    routes: { from: string; to: string; requirement: string | null }[];
    characters: {
      id: string;
      locationId: string;
      description: string;
      relationship: string;
    }[];
  };
  opportunities: {
    id: string;
    description: string;
    outcomeId: string;
    outcome: string;
    conditions: { id: string; meaning: string }[];
  }[];
  capabilities: {
    action: Action;
    at: string;
    requires: string[];
    effects: string[];
  }[];
};
```

Planning sees a small relevant neighbourhood, including remote participants and
routes. Routes are directed; include both directions where travel is reversible.
Capability entries are legal bound instances of reusable actions. They can include
future actions whose requirements are not yet satisfied. This initial representation
avoids asking the model to infer a combinatorial parameter grammar. Larger catalogues
can later expose typed domains without changing the action semantics.

Readable requirements, effects and condition meanings are explanations generated
from registry definitions. Code validates with the structured definitions, never
by interpreting those strings. Outcome/condition IDs reference existing definitions;
they are not executable model-authored expressions.

### Planning output

```ts
type MissionPlan = {
  opportunityId: string;
  goal: string;
  narrative: Narrative;
  milestones: { label: string; conditionId: string }[];
};
```

The opportunity fixes the outcome; the model need not echo `outcomeId`. `goal` is
the journal title. The narrative brief carries motivation, participants and the
promised resolution across every scene. It replaces separate duplicated `why`
and unstructured mission-thread fields. Code assigns mission identity.

The validator must establish a feasible action path through the milestones to
the outcome, within a bounded local simulation of registered mechanics. Reject
unsupported or unprovable plans and use an authored offer. This is not a general
planner for arbitrary scripts or an unrestricted world. Accepting a mission pins
necessary finite resources or ensures a recovery source; otherwise an initially
valid plan could become impossible midway through play.

### Scene input and output

```ts
type SceneInput = {
  mode: "continue_story";
  context: Context;
  here: {
    description: string;
    characters: { id: string; description: string; relationship: string }[];
    objects: { id: string; state: string }[];
  };
  mission: {
    goal: string;
    narrative: Narrative;
    completed: string[];
    current: string | null;
    remaining: string[];
  } | null;
  availableActions: {
    id: string;
    description: string;
    action: Action | { type: "dismiss" };
  }[];
};

type Scene = {
  presentation: (
    | { type: "narration"; text: string }
    | { type: "dialogue"; speakerId: string; text: string }
  )[];
  choices: { label: string; actionId: string }[];
};
```

Only current legal actions appear in scene input. Server-issued action handles
bind exact arguments and expected state; the model returns handles, not rewritten
action bodies. Code already knows the mission and milestone, so no `advances` echo
is needed. Scene progress omits internal condition IDs, receipts and dependencies.
`remaining` excludes `current`; both are empty/null on completion. A completed
mission brief may remain for its celebration before being removed from context.

Routine visuals follow world state and action renderers; the model does not place
wood bundles or manipulate asset slots. Speakers must be present. Every scene has
at least one usable choice, including a supplied dismiss action when appropriate.
UI navigation and leaving remain possible without model-generated choices.

## Building the input payload

### Ownership: code assembles, the model proposes, the player acts

Most input is generated programmatically from saved state and authored content.
Previously accepted model output supplies the mission's goal and narrative brief;
it does not supply evidence of progress. Player input consists of ordinary choices
and optional preference settings, not a written account of where they are or what
happened. No second model is needed to assemble the initial payload.

There are four source classes:

1. **Authored content:** locations, character descriptions, routes, recipes,
   opportunities, conditions and action semantics.
2. **Authoritative runtime state:** inventory, object states, accepted location,
   committed events, milestone receipts and outcomes.
3. **Accepted narrative:** validated mission plans and saved scenes. A generated
   scene becomes presented history only after acknowledgement, not on generation.
4. **Explicit settings:** language and comfort preferences; interests may start
   empty. Later observations must be modest and distinguishable from explicit choices.

A proposed server-side context builder joins these sources into `PlanInput` or
`SceneInput`. The API loads an ownership-checked, consistent state snapshot; a pure
projection function builds the payload. Provider calls and database access remain
outside the pure domain/content logic. Do not build the payload by copying the
browser save or sending the existing public `GameSnapshot` wholesale: that snapshot
also includes learner summaries and educational encounters.

### Field provenance and current integration status

Status below is based on the repository on 30 October 2026. Existing foundations
are reusable data, not proof that the proposed story interface is implemented.

| Input field | Source and derivation | Existing foundation / work required |
| --- | --- | --- |
| Envelope `protocol`, `contentVersion`, `policyVersion` | Server configuration and selected content bundle | Content/rules versioning exists; new story protocol/policy configuration needed |
| Envelope `requestId`, `stateRevision` | Caller correlation ID and consistent authoritative revision | Revision/idempotency foundations exist; new generation-request binding needed |
| `context.trigger` | Short template over a committed event, arrival or session resume | Domain events exist; new story event types and trigger mapping needed |
| `player.language` | Explicit setting or `short_simple` default | Add story setting; do not derive from mastery or reading assessment |
| `player.interests` | Explicit topic selections; optional observations later | New preference storage/UI; use `[]` initially |
| `player.avoid` | Explicit comfort settings only | New preference storage/UI; use `[]` plus fixed content policy initially |
| `context.locationId` | Accepted coarse location in story runtime | Browser map/position and domain region exist; authoritative story-location integration needed |
| `context.inventory` | Select relevant item IDs/counts from server inventory | Inventory exists; wood/plank items, sources and recipe mechanics need implementation |
| `memory.facts` | Deterministic templates over known world consequences, meetings and relevant completed briefs | Story flags/friendship exist; fact projection, meeting records and narrative retrieval needed |
| `memory.recentEvents` | Ordered committed story events and presented-scene receipts | Domain events exist; story history projection and presentation acknowledgements needed |
| `memory.recentMissions` | Summaries of completed accepted missions, including outcome and action pattern | New mission storage and deterministic summary projection needed |
| `area.locations` | Relevant authored location IDs and short descriptions | Browser map catalogue exists; shared story catalogue/descriptions needed |
| `area.routes` | Shared directed route definitions joined with current gate state | Browser connections exist; server route eligibility and story gates need integration |
| `area.characters` | Authored descriptions plus current placement and confirmed relationship facts | Character personality content and browser placements exist; server placement/meeting projection needed |
| `opportunities` and nested outcome/condition fields | Authored opportunity definitions filtered by state and supported mechanics | New opportunity catalogue, condition predicates and feasibility checks |
| `capabilities.action` and `at` | Legal bound instances from registered mechanics and relevant locations | New reusable action registry and bound-instance builder |
| `capabilities.requires/effects` | Readable templates generated from the same registered definitions used for execution | New semantic projections; never separately maintained model claims |
| `here.description` | Authored current location description | Derive from shared catalogue once built |
| `here.characters` | Characters actually present, with authored descriptions and confirmed relationships | New presence projection over existing content and new location state |
| `here.objects` | Relevant registered objects joined with persistent states | Some story flags exist; new object catalogue/state projection for generic actions |
| `mission.goal/narrative` | Verbatim fields from an accepted plan, authored or model-proposed | New accepted mission persistence |
| `mission.completed/current/remaining` | Code evaluates conditions and projects persisted milestone labels | New milestone evaluation/receipts; never copied from a model's claim of success |
| `availableActions.id` | Server-issued handle scoped to saved scene/request and relevant state | New handle binding and validation |
| `availableActions.description/action` | Currently executable registered actions, including exact arguments, plus dismiss | New eligibility projection and action renderers; reuse existing command safeguards |

Relevant current sources:

- [Backend snapshot projection](../backend/crates/api/src/game/snapshot.rs) exposes
  story flags, inventory, companions and friendship alongside learning fields.
- [Domain state](../backend/crates/domain/src/engine/state.rs) contains region,
  inventory, flags, progress and receipts; the
  [API aggregate](../backend/crates/api/src/game/mod.rs) retains domain events.
- [Character content](../content/creatures/friends.v1.json) supplies authored
  personalities and habitats, subject to its existing review gates.
- [Map definitions](../frontend/src/maps.ts) and
  [encounter placements](../frontend/src/world-placements.ts) provide browser
  geography. Placements explicitly do not establish server eligibility.
- [Backend world view persistence](../frontend/src/backend-world.ts) saves map,
  position and trail preferences locally. The existing
  [exploration trail](../frontend/src/exploration-quest.ts) combines local acceptance/
  return state with server activity completion; it is not this new mission runtime.

### Location and player-known history need explicit rules

Presentation location is where the browser reports the player is. Gameplay location
is a transition accepted under the story runtime's rules. Browser reports may help
choose cosmetic dialogue, but alone cannot prove arrival for an exchange, delivery
or milestone. A domain region value is also insufficient to identify the workshop.

Start with coarse location IDs and registered interaction points rather than
server simulation of every walking frame. Define accepted transition commands,
route/gate checks and interaction availability. If a mission depends on traversing
a particular path, add the necessary verification explicitly; a coarse arrival
record cannot prove a continuous route. Do not send coordinates to the model.

A presentation acknowledgement means the client presented an entry, not that the
child read, heard or understood it. Bind acknowledgements to saved scene/entry IDs,
deduplicate them and track partial presentation. A proposed or unshown introduction
must not turn into “you have met before.” Presentation records cannot grant items
or prove gameplay completion. Facts based on presentation should describe what was
shown; mechanical facts come from committed game events.

### Payload assembly algorithm

1. Load the authorized state, accepted mission, relevant content versions and
   settings from one consistent revision. Keep internal source/event IDs for
   provenance; do not send account identifiers to the model.
2. Normalize the trigger from a committed event or valid resume request. Do not
   turn arbitrary browser text into an authoritative event description.
3. Select scope: a small reachable neighbourhood for planning; the current location
   and focused mission for a scene. Include future gated routes only with their
   requirements and a feasible way to satisfy them.
4. Join content with state: object states, character presence, relationships,
   relevant inventory, opportunities and bound capabilities. For scenes, filter
   actions by current eligibility; for planning, include feasible future actions.
5. Retrieve memory by referenced characters, locations and objects. Preserve
   mission commitments and older consequential facts before recent incidental
   movement. Include a few recent mission patterns to avoid repeating errands.
6. Copy the accepted brief verbatim and derive progress from conditions/receipts.
   Project events/facts using short templates. Do not ask a model to reconstruct
   inventory, completion or history from prose.
7. Apply size limits and validate the input schema and references. Exclude learning
   records by construction using an explicit field allowlist, not by stripping
   selected keys from a broad snapshot.
8. Bind the request to the revision, invoke the provider outside the transaction,
   then validate against current state before saving the proposal. If state changed,
   discard/rebuild or use the existing fallback; never commit against stale state.

Use deterministic ordering and selection rules so a fixed state/content/settings
snapshot yields the same model input. Do not hold a database transaction open while
waiting for generation. Provenance is internal diagnostic data, not a reason to
log full prompts or child data by default.

### Missing data and honest defaults

Empty interests, empty recent history and `mission: null` are valid. An unknown
relationship should be described as “no recorded prior meeting,” not “has never
met.” Use authored description defaults only where they are true of the location.
Never fabricate facts to fill arrays or treat missing inventory as an available item.

Missing location authority, content definitions, recipes, outcome predicates or
required resources are blocking for dependent gameplay proposals. Omit unsupported
opportunities; if none remain, use an authored supported offer. A new profile needs
an explicit starting location and authored entry opportunity, not invented history.
A failed state read is not an empty world: do not generate until a valid snapshot
or supported recovery path is available.

### Example: build the next input after exchanging wood

The bound recipe succeeds and commits: consume 3 wood, grant 3 planks. Code records
its receipt, satisfies `planks_ready` and preserves the earlier `wood_ready` receipt.
The builder emits:

```json
{
  "trigger": "The woodcutter made three planks.",
  "inventory": [{ "itemId": "planks", "quantity": 3 }],
  "progress": {
    "completed": ["Find suitable wood.", "Get the planks ready."],
    "current": "Bring the planks to the bridge.",
    "remaining": ["Repair the bridge.", "Cross with Pip."]
  }
}
```

This is an explanatory projection fragment, not an additional input schema:
`trigger`/`inventory` populate `context`, and `progress` populates the scene's
`mission` progress fields. The accepted goal and narrative remain unchanged.
Code offers an eligible travel action back to the bank. Only the next dialogue
and choice wording require generation. If the player merely saw the exchange offer,
none of these state changes occur.

## Integration sequence and completion gates

| Stage | Work | Evidence needed before moving on |
| --- | --- | --- |
| 1. Shared story catalogue | Define the small bridge/workshop/far-bank area, characters, routes, items, recipes, opportunities and conditions; map them to renderers | Content/reference checks pass; every capability has supported visuals/controls and declared effects |
| 2. Authored mission runtime | Add authoritative coarse travel/interaction rules, collect/exchange/assemble/accompany commands, milestone/outcome evaluation, recovery and accepted mission storage | Complete Pip's journey without a model, including existing inventory, both sources, cancellation and retry; preserve educational/guest behavior |
| 3. Context builder | Implement the source mapping and deterministic projections above; add minimal settings and presentation records | Golden payload fixtures match real states, exclude learning/account data and handle missing/stale state honestly |
| 4. Payload review | Capture synthetic test payloads at every milestone and revisit; inspect size, continuity, sufficient choices and unsupported claims | Full journey supplies enough context without fabricated facts; contradictory/missing definitions fail closed |
| 5. Generated scenes | Keep an authored mission fixed; add gated provider adapter, narrative evaluation and saved scene/action bindings | Generation cannot change mechanics; outage, reload, late response and misleading labels have usable fallbacks |
| 6. Generated missions | Supply several supported opportunities and reusable action bindings; validate proposed milestones/outcomes | Distinct mission purposes and action patterns remain playable; promises match outcomes; no dependency on academic success |
| 7. Optional fine-tuning | Use reviewed input/output pairs only after measuring the constrained baseline | Frozen journey evaluation demonstrates a useful improvement; provider/budget approval remains separate |

Stages 1–3 require new game integration, not simply prompt formatting. Existing
inventory, events, content and transaction code provide foundations, but the bridge
recipes, generic actions, authoritative arrivals and mission persistence must be
built. Use new migrations/contracts where needed and retain compatibility; do not
repurpose the current assessed bridge encounter or local exploration trail silently.

Verification should include source-to-payload tests: an exchange updates inventory
and progress, a displayed offer does not; an unseen introduction is not recalled;
old bridge repairs survive recent-history trimming; absent preferences use defaults;
a rejected action produces no success trigger; stale generation cannot overwrite
new progress. These tests complement the mission journey cases below.

## Worked example: Pip's bridge

All mechanics, condition IDs and locations below are illustrative proposed content,
not claims about the current game. The objective is **repair the bridge and cross
together with Pip**. Visiting an unavailable unnamed friend is not promised.

### Conditions and mechanics owned by code

| Condition | Exact intended meaning |
| --- | --- |
| `wood_ready` | Player currently has at least 3 compatible wood; pre-existing wood counts |
| `planks_ready` | Player currently has at least 3 bridge planks; pre-existing compatible planks count |
| `planks_at_bridge` | Player is at the bridge bank carrying at least 3 bridge planks |
| `bridge_repaired` | Bridge object's persistent state is repaired |
| `crossed_with_pip` | A verified shared crossing event for this mission has committed |
| Outcome `bridge_and_crossing` | Bridge is repaired AND this mission's shared crossing has committed |

Satisfied milestone receipts persist even after their inventory predicates cease
to hold. The final outcome remains authoritative. Existing world state is respected:
if the bridge is already repaired, offer a crossing/exploration opportunity rather
than making the player repair it again.

`wood_to_planks` consumes 3 wood and grants 3 planks at the woodcutter.
`repair_bridge` consumes 3 planks and repairs the bridge at the bank. Sources have
finite quantities; collection decrements them atomically. Both wood sources below
can satisfy the first milestone. No step requires one particular source.

### Complete planning input

```json
{
  "mode": "plan_mission",
  "context": {
    "trigger": "The player met Pip beside the broken bridge.",
    "player": { "language": "short_simple", "interests": ["helping friends", "exploring"], "avoid": ["spooky scenes"] },
    "locationId": "bridge_bank",
    "inventory": [],
    "memory": {
      "facts": ["Pip wants to explore the far bank with the player."],
      "recentEvents": ["The player followed Pip to the stream."],
      "recentMissions": []
    }
  },
  "area": {
    "locations": [
      { "id": "bridge_bank", "description": "A broken bridge and fallen wood beside a stream." },
      { "id": "workshop", "description": "A nearby workshop with spare wood." },
      { "id": "far_bank", "description": "A quiet clearing across the stream." }
    ],
    "routes": [
      { "from": "bridge_bank", "to": "workshop", "requirement": null },
      { "from": "workshop", "to": "bridge_bank", "requirement": null },
      { "from": "bridge_bank", "to": "far_bank", "requirement": "Bridge repaired" },
      { "from": "far_bank", "to": "bridge_bank", "requirement": "Bridge repaired" }
    ],
    "characters": [
      { "id": "pip", "locationId": "bridge_bank", "description": "A cheerful, curious friend.", "relationship": "Already knows the player." },
      { "id": "woodcutter", "locationId": "workshop", "description": "A patient neighbour who shapes wood.", "relationship": "Has not met the player." }
    ]
  },
  "opportunities": [
    {
      "id": "pips_bridge",
      "description": "Help Pip make a way across the stream, then explore the far bank together.",
      "outcomeId": "bridge_and_crossing",
      "outcome": "The bridge is repaired and the player has crossed it with Pip.",
      "conditions": [
        { "id": "wood_ready", "meaning": "Have 3 compatible wood, including existing inventory." },
        { "id": "planks_ready", "meaning": "Have 3 compatible bridge planks." },
        { "id": "planks_at_bridge", "meaning": "Be at the bank carrying 3 bridge planks." },
        { "id": "bridge_repaired", "meaning": "The bridge is repaired." },
        { "id": "crossed_with_pip", "meaning": "A shared crossing has completed for this mission." }
      ]
    }
  ],
  "capabilities": [
    { "action": { "type": "collect", "sourceId": "fallen_wood", "itemId": "wood", "quantity": 3 }, "at": "bridge_bank", "requires": ["Source has 3 wood"], "effects": ["Source loses 3 wood", "Player gains 3 wood"] },
    { "action": { "type": "collect", "sourceId": "spare_wood", "itemId": "wood", "quantity": 3 }, "at": "workshop", "requires": ["Source has 3 wood"], "effects": ["Source loses 3 wood", "Player gains 3 wood"] },
    { "action": { "type": "travel", "destinationId": "workshop" }, "at": "bridge_bank", "requires": [], "effects": ["Player arrives at workshop; inventory unchanged"] },
    { "action": { "type": "exchange", "characterId": "woodcutter", "recipeId": "wood_to_planks" }, "at": "workshop", "requires": ["Player has 3 wood", "Woodcutter present"], "effects": ["Consume 3 wood", "Grant 3 planks"] },
    { "action": { "type": "travel", "destinationId": "bridge_bank" }, "at": "workshop", "requires": [], "effects": ["Player arrives at bridge bank; inventory unchanged"] },
    { "action": { "type": "assemble", "objectId": "bridge", "recipeId": "repair_bridge" }, "at": "bridge_bank", "requires": ["Player has 3 planks", "Bridge broken"], "effects": ["Consume 3 planks", "Bridge becomes repaired"] },
    { "action": { "type": "accompany", "characterId": "pip", "destinationId": "far_bank" }, "at": "bridge_bank", "requires": ["Bridge repaired", "Pip present"], "effects": ["Player and Pip arrive at far bank", "Record shared crossing for this mission"] }
  ]
}
```

Both sources initially contain three wood. Those quantities and character presence
are verified in the server registry/state; prose requirements do not establish them.
This focused example demonstrates one opportunity and alternative ways to pursue
it. A diversity evaluation must supply multiple genuinely different opportunities,
not claim that a single bridge catalogue demonstrates varied mission generation.

### Complete planning output

```json
{
  "opportunityId": "pips_bridge",
  "goal": "Build a bridge and explore with Pip.",
  "narrative": {
    "premise": "Pip would love to explore the clearing across the stream. Together you can repair the bridge.",
    "participantIds": ["pip", "woodcutter"],
    "payoff": "Cross the repaired bridge together and celebrate reaching the far bank."
  },
  "milestones": [
    { "label": "Find suitable wood.", "conditionId": "wood_ready" },
    { "label": "Get the planks ready.", "conditionId": "planks_ready" },
    { "label": "Bring the planks to the bridge.", "conditionId": "planks_at_bridge" },
    { "label": "Repair the bridge.", "conditionId": "bridge_repaired" },
    { "label": "Cross with Pip.", "conditionId": "crossed_with_pip" }
  ]
}
```

The model chooses a narrative framing and milestones; it does not prescribe which
wood source to use. Code checks feasibility, assigns mission identity and saves the
brief. On acceptance, compatible inventory may immediately satisfy milestones. If
planning starts with planks already carried, omit wood acquisition rather than
forcing unnecessary work. The accepted outcome stays the same.

### Complete scene input at the workshop

The player collected fallen wood and walked to the workshop. Code has recorded
`wood_ready` and now offers executable actions. Receipts and predicate definitions
stay on the server.

```json
{
  "mode": "continue_story",
  "context": {
    "trigger": "The player arrived at the workshop carrying 3 wood.",
    "player": { "language": "short_simple", "interests": ["helping friends", "exploring"], "avoid": ["spooky scenes"] },
    "locationId": "workshop",
    "inventory": [ { "itemId": "wood", "quantity": 3 } ],
    "memory": {
      "facts": ["Pip is waiting at the broken bridge."],
      "recentEvents": ["The player gathered fallen wood and followed the workshop path."],
      "recentMissions": []
    }
  },
  "here": {
    "description": "An open-air workshop beside the stream path.",
    "characters": [
      { "id": "woodcutter", "description": "A patient neighbour who shapes wood.", "relationship": "Meeting the player for the first time." }
    ],
    "objects": [ { "id": "workbench", "state": "ready" } ]
  },
  "mission": {
    "goal": "Build a bridge and explore with Pip.",
    "narrative": {
      "premise": "Pip would love to explore the clearing across the stream. Together you can repair the bridge.",
      "participantIds": ["pip", "woodcutter"],
      "payoff": "Cross the repaired bridge together and celebrate reaching the far bank."
    },
    "completed": ["Find suitable wood."],
    "current": "Get the planks ready.",
    "remaining": ["Bring the planks to the bridge.", "Repair the bridge.", "Cross with Pip."]
  },
  "availableActions": [
    { "id": "a1", "description": "Give the woodcutter 3 wood and receive 3 planks.", "action": { "type": "exchange", "characterId": "woodcutter", "recipeId": "wood_to_planks" } },
    { "id": "a2", "description": "Leave the conversation without changing items or progress.", "action": { "type": "dismiss" } }
  ]
}
```

### Complete scene output

```json
{
  "presentation": [
    { "type": "dialogue", "speakerId": "woodcutter", "text": "Hello! I can turn that wood into sturdy planks for your bridge. Shall we get started?" }
  ],
  "choices": [
    { "label": "Make the planks", "actionId": "a1" },
    { "label": "Come back later", "actionId": "a2" }
  ]
}
```

Displaying this dialogue grants nothing. Choosing `a1` executes the bound exchange,
consuming wood and granting planks in one transaction, then satisfies `planks_ready`.
Choosing `a2` leaves the mission and inventory unchanged. “Find suitable wood” stays
complete after consumption. Reloading or retrying the exchange cannot grant more
planks. Speaker introductions enter player-known memory only after presentation.

### Journey, alternatives and payoff

A typical journey includes accepting Pip's offer, collecting wood, walking to the
workshop, exchanging it, returning, assembling and crossing together. Five
milestones support roughly seven meaningful actions without requiring seven
artificially locked buttons.

| Situation | Expected behavior |
| --- | --- |
| Player already carries suitable wood | First condition is satisfied; offer a route to the workshop |
| Player prefers workshop spare wood | Travel there first and collect from that allowed source |
| Player already carries suitable planks at planning time | Omit wood/exchange milestones; proceed toward bridge assembly |
| Player explores elsewhere midway | Preserve progress; offer a valid route back when they resume |
| Player consumes materials elsewhere | Keep historical milestone receipts but reacquire needed resources through a supported recovery source |
| Player crosses alone first | Bridge remains repaired; shared crossing still requires Pip and a verified return/escort route |
| Mission is cancelled after assembly | Keep the repaired bridge; do not restore consumed materials as a duplicate grant |
| Model is unavailable | Show the current milestone and eligible actions using registry text |

The shared crossing completes only after both arrive. If companions can move away,
the registry must supply a way to reunite; this example keeps Pip at the bank until
accompanied. At the far bank, show a celebration grounded in the stored payoff:
“We made it! Thank you for building this with me.” Preserve the repaired bridge,
the shared experience and the woodcutter introduction for future stories.

A new mission may use supported opportunities on the far bank. The scene must not
promise a new character, gift or destination absent from the available world.

## Narrative freedom and continuity

The accepted brief is the persistent narrative commitment. The model may vary
voice, humour, imagery and reactions while respecting the premise and payoff.
Scenes cannot add durable promises, hidden motives, new relatives or new world
facts that are absent from the accepted brief or supplied context. This is a
specific first-version limit, not a claim of unrestricted story generation.

New mission premises may connect existing characters and opportunities in fresh
ways, but must not contradict established relationships or promise unsupported
consequences. Narrative review decides whether these statements are acceptable;
structural checks alone cannot establish that. Once accepted, retain the brief
verbatim through the mission and retrieve it with relevant completed missions.

Keep a source event history plus durable mission briefs and consequences. Retrieve
facts relevant to returning characters, objects and places; do not repeatedly
summarize only the previous summary. Distinguish presented text from proposed text,
and known facts from unshown material. The first version sends no hidden secrets
and supports no arbitrary model-written memory patches. Defer a separate thread
system until stories require commitments spanning several missions.

Preferences influence themes without overriding continuity or forcing repetition.
Avoid inferring personality, emotion or ability from inactivity. Recent mission
summaries should expose repeated patterns so the model can choose a different
supported outcome when one exists. Rephrasing the same delivery chain is not
meaningful diversity.

## Validation and implementation boundaries

### Deterministic checks

Validate schema/size, IDs and bound argument tuples, route feasibility, resources,
registered conditions/outcomes, milestone ordering, scene speakers and action
eligibility. Simulate a bounded path to the outcome; a list of individually legal
actions is insufficient. Recheck current state at execution and mission completion.

Retain ownership, CSRF, transaction atomicity, revision checks and idempotency.
Retry uncertain writes with the exact prepared command/body/ID. Pin operation and
recipe versions for active missions or migrate them explicitly. Stored scenes
must be revalidated on resume if relevant state changed; reusing a stale scene is
not required merely because it was saved.

Resource recovery must be real before a plan is accepted. For this example reserve
sufficient compatible material for the active goal or provide a replenishable
recovery source. An initially feasible mission with no recovery after permitted
item use is not playable. Deferring competing concurrent missions keeps this small.

### Narrative evaluation

Assess whether the goal and payoff match the supported outcome, prose contradicts
facts, choices describe their effects honestly, and tone/language are suitable.
These are fallible semantic evaluations, not guarantees from JSON validation.
Use reviewed examples and complete journeys to establish quality. Until live text
checks have evidence, retain authored text for player-facing operation. Provider,
privacy, budget and deployment review remain separate gates.

Model input excludes account IDs, exact birth dates, raw answers, audio and personal
names supplied by the child. Derived preferences/progress still require appropriate
provider review. Treat descriptions and retrieved text as data, not instructions;
do not log raw prompts by default.

### Keep the first implementation small

- One focused mission; ordered milestones; no general dependency-graph editor.
- Reusable typed actions, legal bound instances and registered outcome predicates.
- One persistent brief, with no independent thread/memory editing language.
- Compact progress for scenes; full receipts and predicates stay on the server.
- Renderer-owned routine visuals and action-derived fallback controls.
- Bounded input/output and short scenes. Start with at most 10 milestones, 6
  presentation entries and 4 choices; tune context budgets using real examples.

The model does not need versions, revision numbers, repeated mission IDs, execution
receipts, curriculum state or the full world. It does need nearby opportunities,
relevant history, persistent motivation, progress and honest executable choices.

## Education remains separate

Neither planning nor scene contracts include academic skill IDs, mastery states,
activity parameters or correctness. Story operations never assess learning.
If an independent educational system later presents an activity alongside a scene,
its invitation and minimal story-level return event belong in a separate adapter.
An incorrect answer cannot block this mission. Existing supported routes and
support/evidence distinctions remain intact.

## Verification and training direction

Start with this loop using an authored mission and registered mechanics. Then add
new contract schemas, validator, persistence/resume and scene rendering. A gated
model adapter comes after those foundations. No existing contracts, saves or
learning behavior change with this document.

The two potential training pairs remain `PlanInput → MissionPlan` and
`SceneInput → Scene`. Use synthetic/adult-authored data and a constrained baseline
before deciding whether fine-tuning addresses measured deficits. Split examples
by scenario family before variants; keep rejected outputs out of approved targets.

Before training, test multiple supported opportunities in one small world and
measure differences in purpose, action pattern, relationships and payoff. Include
construction, delivery and exploration opportunities when their mechanics exist.
The same action vocabulary should serve different goals. Do not count renamed
items or different prose as sufficient mission diversity.

Required complete-journey cases include:

- Pip's normal journey and both wood sources, with exact inventory conservation.
- Existing wood/planks, a repaired bridge and early actions satisfying conditions.
- Refusal, detours, cancellation, reload, uncertain retries and model outage.
- Recovery after material use; travel interruption; crossing alone before escort.
- A valid action sequence that fails its promised outcome: reject the plan.
- Invalid recipes, unsupported promises, missing characters and inaccessible routes.
- Returning to Pip/woodcutter later without forgetting established experiences.
- A different next mission and a natural stop, without academic dependencies.

Application limits and narrative evaluation need empirical calibration. This is a
proposed core flow, not evidence that generated stories are already consistent,
playable, approved or ready for release.
