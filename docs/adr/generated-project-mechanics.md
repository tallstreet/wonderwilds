# Repeatable neighbourhood project mechanics

Status: development implementation, 31 October 2026. Human content/release review
remains pending.

Generated neighbourhood missions previously supported inspections followed by a
return conversation. Narrative variety alone could not vary what the player did.
We add bounded project mechanics through existing collect, assemble and choose
commands, without allowing the model to invent state effects or recipes.

A saved step may specify a fixed mechanic. Missing values preserve inspection.
The server selects an executable sequence and route; generation supplies labels,
character motivation, post-action text and a payoff. Fresh proposals must follow
that exact plan. Rust also validates the full spec independently: recipes must
have ingredients in order, project locations must be connected, supplies must be
consumed and projects must have a concrete outcome. The schema bounds stories to
six steps; current project plans have three or four.

Fixed recipes gather three wood/berries/glow pebbles, make three planks/one picnic
basket/one lantern, and consume those supplies in a delivery or construction.
Project inventory is stored per mission and included in the bag's public totals.
It cannot consume Pip's finite sources or another mission's reserved supplies.
Archiving preserves resources; replacing a completed project spec cannot retain
farmable surplus. Targets bind character, generation and step. Repeating a stale
or altered action cannot gather again; exact command retries return receipts.

Leafy and starry designs are equally valid. They are ordinary choices and persist
to the final project's appearance. Other project steps use the existing inline
learning boundary, including supported completion. Rust assesses learning and
commits story effects atomically under the same profile lock and receipt rules.
An activity bound to an old generation cannot act on a replacement mission.

Optional completeOnLastStep finishes at the work site; omitted/false retains the
legacy return conversation. Completion does not invent academic mastery or new
rewards. Seats, picnics, lanterns and supply stacks persist independently of the
current mission. Each of five areas stores at most one of each project kind;
rebuilding the same kind updates that area's existing project. This bounds world
state without erasing a finished project when its mission slot is replaced.

New checkpoint fields are optional and omitted when empty. No SQL migration or
data reset is needed. Old saves and original receipts retain their readers and
meaning. Rolling back after new project saves requires a reader supporting these
fields and mechanic IDs; deleting progress is not a rollback strategy.

The existing map and artwork are reused. This does not add escorts, arbitrary
items/recipes, new maps, free-form construction or content approval. Novelty
checks compare bounded recent content, not complete lifetime history or semantic
quality. Live model adherence and child playtesting remain separate evaluation.
