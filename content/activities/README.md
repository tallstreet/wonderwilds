# Draft deterministic activities

`draft-v1.json` contains six original activity-family packs, each with three solution fixtures, two contexts and a three-step hint ladder. These are server-internal authoring fixtures: never serve the `solution`, `accepted`, target relation or canonical event order as encounter parameters. Approved instructions are resolved from authored templates; visible manipulatives alone must not disclose internal acceptance lists.

The Rust implementation is `backend/crates/learning/src/assessment/`. `generate(family, recorded_seed)` creates bounded reproducible tasks; `Task::validate_skill` binds a task to the current draft node bounds; `Task::validate_representation` rejects unsupported representations. Call both before persisting an encounter. `assess` consumes a typed answer and server-loaded support state. Clients cannot assert independent evidence or supply the support history. The API integration must enforce this boundary.

`assessment::wire::authored_encounter(skill_id, seed, encounter_id)` returns a private task and a schema-validated public `EncounterInstance@1`. Store the skill, seed, encounter ID and server support history; regenerate the task on replay. `assess_wire` maps only the original encounter's allowed object/card IDs or tile indices into private typed answers, then assesses them. Never deserialize a client-supplied task or replace the original encounter parameters with request data. `authored_parameters` supplies the deterministic director candidate parameters. `prompts-v1.json` is the public instruction registry; unknown prompt IDs are rejected. It carries no acceptance lists or solution keys.

Hints and worked examples classify all assessed responses as supported. Assessment narration also classifies independent-reading tasks as supported; narration of maths instructions and motor access accommodations alone do not change the assessed skill. Free composition, oral explanations and subjective character reasoning return `Unassessed`, never automatic success. The result is an answer observation, not a mastery decision or story reward.

Run from repository root with the configured Rust toolchain:

```sh
cargo test -p wonderwilds-learning --locked
cargo clippy -p wonderwilds-learning --all-targets --locked -- -D warnings
```

The assessment tests check 3,072 generated tasks, exhaustive small quantity and sharing domains, equal-total unequal groups, malformed inputs, explicit spelling alternatives, support distinctions, curriculum bounds and every JSON solution fixture against the actual Rust assessor.

All packs remain `reviewStatus: pending` and `releaseEnabled: false`. Hints, decoding progression, pronunciation/dialect alternatives, accessibility representations, asset instruction binding and educator approval remain review gates. The wire adapter is available for API/browser integration, which has its own acceptance checks. The current fixed word and sentence tasks are narrow practice observations, not proof of general decoding or comprehension. This module does not certify broad language outcomes or claim curriculum alignment. Existing guest play and browser saves are unchanged; removing these draft files/module rolls back this addition without a save migration.
