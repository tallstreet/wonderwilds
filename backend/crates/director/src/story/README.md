# Authored story payload builder

This is a headless development projection over `wonderwilds_domain::story::State`.
It makes no provider calls and reads no profile, account, learning, browser or
free-text user data. Existing `director::core` and Director v1 contracts are unchanged.

Public functions return schema-validated JSON:

- `build_plan_input(&State)`: small area, authored opportunity, state-dependent
  legal future bindings, and context.
- `authored_plan(&State)`: authored mission definition, not a generated plan or a
  new offer. It remains inspectable after completion; `opportunities` is then empty.
- `build_scene_input(&State)`: default preferences, actual inventory and facts,
  presented-introduction history, compact mission progress and currently legal actions.
- `authored_scene(&State)`: exact domain-authored presentation and action-derived choices.
- `resolve_action(&State, handle)`: resolves only an exact current revision/index
  handle to its bound domain action. Execute with the same expected revision and
  stable command ID; the domain rechecks preconditions and idempotency.
- `payload(&State, Mode::Plan | Mode::Scene)`: adds `story-proposal-1`, content version
  and state revision outside the model input. The local harness needs no account or
  request correlation; a future HTTP adapter must add authenticated scoping.

## Sources and limits

Locations, route descriptions, character descriptions and narrative are authored
constants here. Inventory, bridge state, supplies, Pip's presence, progress, events
and woodcutter introduction acknowledgement come from read-only runtime getters.
The runtime owns mechanics; descriptions explain its fixed bound recipes. Golden
fixtures and journey tests verify these projections against real state transitions.
No claim is made that the current small catalogue supports arbitrary mission planning.

The first slice has fixed default preferences and one authored opportunity. Existing
planks and an already-repaired bridge reduce the authored milestone list; the neutral
goal is to help Pip cross rather than claiming the player repaired an existing bridge.
A paused mission retains its narrative/progress with explicit `status: abandoned`.
History is a bounded deterministic event window plus relevant persistent facts;
no model-generated summarization or psychological inference is used.

Implementation refinements to the design shapes: scene input includes mission
`status` so resumed/paused context is explicit; authored scene output includes
`sceneId` and presentation `entryId` to bind acknowledgements. Entries support
narration when no character is present. The runtime canonical presentation supplies
those IDs and text. Rendering/building never acknowledges an entry automatically.
Available actions include accept/cancel lifecycle variants. The headless harness
shows all eligible actions (up to 12); narrowing to four UI choices is later browser
integration and must preserve access to every needed action. Ordinary dismissal
requires no state change and is not a domain command.

The builder does not load the existing API snapshot, which mixes story and learning
fields. It does not publish endpoints, persist database state, create browser art,
verify physical walking or accept model-proposed missions. The new `Story*` contracts
are additive definitions in the schema source and OpenAPI components only; no HTTP
route is implied. Validating a JSON shape is not validating arbitrary prose safety.

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
inspection remain available. This is not a sustained-play storage implementation.
