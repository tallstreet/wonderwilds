# Starwhisker Island implementation plan

## Product direction

Build a bright, friendly, Pokémon-inspired maths adventure for children aged 5–8, with original superhero kittens inspired by the energy of SuperKitties. Maths changes the world rather than interrupting it with unrelated quizzes. Use Phaser and keep the implementation in `frontend/`. Build and test the frontend before adding a backend or model fine-tuning.

Initial scaffolding includes world rendering and encounter data; it is not a finished prototype.

## 1. First playable adventure

Working title: **Starwhisker Island**. First map: **Clover Meadow**.

- One playable kitten and one recruitable companion, Pip.
- Three story activities: repair a bridge, share picnic berries, wake a flower garden.
- Walkable map, visible world changes, collection journal, and local saving.
- Mouse, touch, and keyboard controls.

Acceptance: a satisfying 10–15 minute adventure that a child can complete without an adult explaining controls. This usability claim requires child playtesting; implementation alone does not establish it.

## 2. Frontend architecture

Use Phaser for the world, HTML/CSS for menus and activities, and Vite for development/builds.

Suggested structure:

```
frontend/
  src/
    game/{scenes,entities,movement}/
    learning/{curriculum,activities,assessment}/
    content/{maps,encounters,companions}/
    ui/{journal,grown-up-view}/
    storage/
    main.js
  public/assets/
  tests/
  docs/
```

Keep world movement, activity interactions, deterministic assessment, story rewards, and content provision separate. Equivalent simpler module boundaries are acceptable for the first milestone. New encounter data should not require movement or assessment changes.

## 3. Movement and exploration

- Tap-to-walk with obstacle-aware navigation.
- Arrow keys and WASD.
- Large interaction targets and clear next-action prompts.
- Collision boundaries for water, trees, and buildings.
- Responsive scaling and appropriate camera behaviour.
- Accessible menu controls and optional spoken instructions.

Acceptance: players can reach characters, start activities, and return to exploring on desktop and tablet.

## 4. Curriculum progression

Use ability bands rather than age alone. Begin with a small Foundation–Year 2 subset; expand toward Year 3 after mechanics are tested.

| Band | Concepts | Interaction |
| --- | --- | --- |
| Beginning | Counting, quantities to 10, missing parts | Add bridge planks |
| Growing | Addition/subtraction to 20, equal sharing, counting in twos | Share fruit |
| Stretching | Place value, larger missing quantities, equal groups, counting in fives | Group lighthouse crystals |

Each skill needs prerequisites, objective, allowed number ranges, representations, common mistakes, hints, and evidence criteria for increasing difficulty. This is a prototype curriculum slice, not a complete curriculum.

Reference direction:
- https://www.australiancurriculum.edu.au/resources/work-samples/mathematics/foundation/ws01-knowing-numbers
- https://www.australiancurriculum.edu.au/resources/work-samples/mathematics/year-1/ws03-place-value

Acceptance: every activity maps explicitly to a skill and an appropriate interaction.

## 5. Three reusable activities

### Missing quantities: repair the bridge

Add/remove planks; count existing planks, see empty spaces, and check the construction.

### Equal sharing: prepare the picnic

Distribute objects between plates. Feedback highlights unequal groups and supports one-at-a-time sharing.

### Number sequences: wake the garden

Complete a flower/stepping-stone sequence, with countable objects alongside numerals at the beginning level.

All activities include a short prompt, touchable objects, clear feedback, unlimited retries, progressive hints, a supported “let’s do it together” route, and a visible world consequence.

Acceptance: maths actions change the scene; they are not unrelated quiz questions.

## 6. Rewards, attachment, progression

- Companion selection/naming.
- Friendship rewards for helping.
- Persistent repairs and blooming gardens.
- Journal of friends, discoveries, completed adventures.
- Story completion unlocks future areas.
- Local saves and a grown-up reset option.

Story progress and learning evidence are separate. Supported completion earns story rewards but is not independent understanding.

## 7. Local adaptation and grown-up view

Begin with transparent rules: offer concrete support after difficulty, reduce quantities when appropriate, revisit concepts in different contexts, and increase complexity only after repeated independent evidence. A single answer does not establish mastery.

Record locally: skill, encounter/variant, attempts, hints, independent versus supported completion, and representation. Show skills practised and support used in the grown-up view, with JSON export for prototype review. Explain why activities are selected.

## 8. AI gameplay contract

Models generate bounded encounter data, never executable code or unrestricted game rules.

```json
{
  "schemaVersion": "1",
  "template": "missing_quantity",
  "skillId": "missing-parts-to-10",
  "difficulty": { "total": 10, "given": 6 },
  "setting": "meadow_bridge",
  "characterId": "pip",
  "story": { "prompt": "Pip needs a bridge with 10 planks. Six are ready!" },
  "representation": "countable_planks",
  "rewardId": "bridge_repaired"
}
```

Engine sequence: validate schema; check maths/curriculum constraints; resolve approved assets/rewards; independently compute the answer; reject invalid output and use authored fallback. Authored and simulated model content share this interface.

## 9. Fine-tuning discovery

Establish a template/constrained-prompt baseline first. Fine-tune only if evaluation identifies recurring problems it could address.

Training examples:
- Input: target skill, permitted difficulty, recent evidence, world state, available templates.
- Output: valid encounter matching the contract.
- Review labels: maths correctness, curriculum fit, reading level, playability, narrative quality.

Use a held-out evaluation set with unseen quantities/scenarios. Measure schema validity, correctness, alignment, difficulty, consistency, understandable variety, rendering, and completion. Begin with authored/educator-reviewed data; separately decide whether consented playtest data is appropriate.

Acceptance: documented dataset format, evaluation suite, and evidence-based fine-tuning decision.

## 10. Playtest and expand

1. Adult functional tests.
2. Parent-assisted child sessions.
3. Iterate on movement, reading burden, hints, feedback.
4. Add Peachy Orchard and Starlight Cove.
5. Add skills and activity types.
6. Build backend after content contract stabilises.

Observe navigation problems, comprehension of the maths interaction, and transfer to similar problems with different objects.

## Current milestone: one polished map

Implement Clover Meadow only as the active adventure. Future content may remain as data, but do not present unfinished areas as playable.

Required deliverables:
- Polished responsive Phaser map with original kitten art and reliable navigation/collisions.
- Three complete maths activities with manipulatives, retries, hints, supported completion, and visible world changes.
- Companion/journal reward and a clear adventure completion state.
- Versioned, validated local saves, graceful storage failure handling, reload restoration, and intentional reset.
- Learning evidence separate from rewards; a grown-up view and JSON export.
- Useful automated tests for assessment, save restoration, progression, and evidence integrity.
- Passing production build, browser QA where tools permit, and documented run instructions and known limitations.

Out of scope for this milestone: backend, authentication, live AI, actual fine-tuning, additional playable maps, claims of validated mastery or a complete curriculum.
