# Authored personal quest documents

These disabled, pending-review development episodes are the first editable
examples of the document shape a future quest-generating model can propose.
They are short choose-your-own-adventure episodes in a continuing game. A
`handoff` closes the current episode with a new thread; it never ends the game.
The current authored bank has four documents and can be revisited. It does not
claim an unlimited supply of unique stories or run a model.

## Where to edit

| File | Personal episode | Main activity slot |
| --- | --- | --- |
| `pip-lantern-trail.v1.json` | Choose a lantern route, then a star or woodland clue | `pip-count` → M06 / missing quantity |
| `tansy-welcome-garden.v1.json` | Welcome bees or birds, then follow water or mystery seeds | `tansy-sequence` → M05 / number sequence |
| `nib-sharing-picnic.v1.json` | Pick a picnic place, then save a song or trail idea | `nib-share` → M08 / equal sharing |
| `flutter-pattern-parade.v1.json` | Lead with ribbons or petals, then choose rhythm or colour | `flutter-pattern` → M09 / repeating pattern |

Each first choice changes the next activity scene. Both supported routes return
to a reflection choice. Each reflection has its own handoff text and continuation
tag, so the selected thread can shape the next episode. Both choices remain
valid; a preference is not an academic assessment.

`catalog.v1.json` is trusted server policy, separate from generated documents.
It owns the level-four character unlock, activity slot bindings and distinct
one-time cosmetic memory item for each episode. Character level is independent
of learner ability. The model-facing document cannot declare reward amounts,
correct answers, skill mastery, unlocks, scripts or arbitrary server actions.
Supported completion receives the same game reward; revisiting an earned episode
does not manufacture another reward or novel learning evidence.

## Document shape

`QuestDocument` is defined in `contracts/schemas/v1.json`; generated Rust and
TypeScript models are not edited by hand. Its bounded graph contains:

- `choice`: plain text plus two or three labelled choices, each selecting a
  `nextNodeId`.
- `activity`: scene text, a trusted `activitySlotId` and its continuation node.
  The server resolves and assesses the actual activity. No answer key appears in
  a document or public quest view.
- `handoff`: scene text and one or more continuation tags for another adventure.

For example, a reflection may point to two different continuing hooks:

```json
{
  "id": "reflection",
  "type": "choice",
  "text": "The trail glows. What shall we look for next?",
  "choices": [
    { "id": "stars", "label": "Look for a star-shaped pebble", "nextNodeId": "handoff-stars" },
    { "id": "leaves", "label": "Listen for rustling leaves", "nextNodeId": "handoff-leaves" }
  ]
}
```

Documents declare their `questId`, `documentVersion`, companion, region, interest
tags and entry node. Node IDs are unique, every node is reachable, every path is
acyclic and reaches a handoff after at least one verified activity. Model output
uses the same document contract. Schema validation checks bounds and closed
fields; semantic validation also checks graph structure and trusted bindings.

The Rust API embeds these JSON files at build time. After editing a document,
rebuild and restart the local backend, then revisit or start a new episode.
Active episodes retain their pinned document, so a running episode continues
with the version it started rather than silently adopting your edits.

This is an iteration format: intentional structural changes require a schema
version and compatible reader. Intentional text/route changes require a new
`documentVersion`; active episodes must retain their pinned content or offer the
defined safe abandonment path. Do not rewrite the historical story manifest to
activate these additive quest documents.

## Future generation input and output

`generation-context.example.json` is a synthetic example of
`QuestGenerationContext`. The server supplies:

- A request ID, current revision, rules/content versions and a context digest.
- Canonical world facts, current region and owned companions.
- Explorer and target-companion levels separately from learning skill/support
  needs. `progression.companionId` identifies whose level is provided.
- Bounded interest tags with their source, recent choices and open threads.
- A curated conversation summary only when sharing has explicitly been enabled.
  `permission: "not_shared"` uses an empty summary. Raw conversations, parent or
  child identifiers, dates of birth, credentials and answer keys are not input.
- Allowlists of companions, regions, activity slots, interests and continuation
  tags the proposed episode may reference.

`generation-proposal.example.json` is the expected `QuestGenerationProposal`:
matching request ID, expected revision and context digest, plus a `document`.
The digest binds the complete context: omit `contextDigest`, recursively sort
object keys, encode compact UTF-8 JSON while preserving array order, then SHA-256.
The sample digest is computed from its sample context, rather than a placeholder.

The future adapter validates that binding, the graph, allowlists and learning
slots before proposing anything for a server transaction. A generated quest is
not a command, a reward, a completion or content approval. Prose coherence,
age-appropriate language, safety and reading level also need review/evaluation;
passing a JSON schema does not prove those qualities. Arbitrary generated prose
is not activated by this authoring workflow.

There is no provider call, live chat ingestion, fine-tuning dataset or training
run in this delivery. The existing authored fallback remains the playable source.
A later model can generate additional episode documents within the validated
contract while the server continues to own authoritative game progression.

## Validation

From the repository root:

```sh
python3 content/quests/validate.py
python3 -m unittest discover -s content/quests -p 'test_*.py'
node content/quests/check-schema.mjs
```

The Python checks validate the catalog, paths, reward uniqueness, graph,
activity coverage, supported bindings and pending-review status. The schema check
uses the shared closed JSON Schema for every document, activity slot and generation
example. Both Rust and TypeScript execute the shared positive/negative wire
fixtures through the normal contracts tests. Tests reject dangling targets,
cycles, unreachable nodes, activity bypasses, unknown bindings, invented reward
fields, answer keys, duplicate identifiers, unsafe document paths and self-approval.
These automated checks do not replace educator, narrative, accessibility or child
usability review. `releaseEnabled` stays `false` and `reviewStatus` stays `pending`.
