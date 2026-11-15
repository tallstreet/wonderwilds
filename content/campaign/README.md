# The Missing Moonlight

Authored campaign development content, revised 14 November 2026. Narrative,
educator, artwork and accessibility reviews remain pending. Release activation is
not approved.

The executable development pack is [runtime.v1.json](runtime.v1.json), validated
against [runtime.v1.schema.json](runtime.v1.schema.json). It contains all 26 main
missions across sixteen locations, 260 finite actions and 22 tactical boards:
three free starter practices, eight keeper trials, three Rowan contests and eight
post-game variants. Seven opening missions form the first playtest checkpoint.
All twelve friends have invitations, four distinct recipient-pinned growth scenes,
a personal keepsake and an optional habitat cache. Fossils, copied clues, the early
island and the sanctuary are executable finite actions. The Rust campaign loader
interprets these rules; see root implementation status for actual API, persistence,
UI and end-to-end evidence. Authored content coverage is not a claim that every
system in the build plan has been implemented or playtested.

The [full story treatment](TREATMENT.md) follows all 26 main-story beats and resolves
the collectors' continuing orders, local restoration versus the central reserve,
eight grounded beacon lines, lens recovery before Illuminate and Rowan's distinct
partners Moss and Reed. Its main-story and personal-conflict material now has finite
runtime definitions. Narrative, educator and audience review remain pending.

[missing-moonlight.v2.json](missing-moonlight.v2.json), with
[schema.v2.json](schema.v2.json), remains the complete **design specification**;
it is preserved unchanged as design history and is not loaded as executable rules.
The v1 Moonlight Trail files remain superseded history. Legacy story content and
integrity hashes are unchanged. Generated campaign extensions remain future work.

## The experience we are building

The player is an apprentice Wayfinder preparing for the Moonlight Gathering.
Hazel introduces three companions. During the welcome, the first beacon goes dark.
Your chosen friend notices a brass cart leaving the bridge. Rowan, another new
Wayfinder, thinks following it is a chance to prove who has the strongest partner.

The mystery leads through three regions. Warden Vell's Stillworks collectors are
moving moonlight, water and supplies into the Observatory. Vell remembers the Great
Storm and believes a central reserve will keep everyone safe. Its protection comes
at the expense of the habitats below. He dismisses the early clues, refuses to stop
the pumps and finally locks the Observatory. The player must restore the old local
beacons, build relationships and demonstrate that sharing the load can work.

Rowan's story develops alongside this conflict. Early encounters favour one strong
partner. At the river, that partner is exhausted and Rowan needs rescuing. Later
Rowan uses two complementary friends, shares a route clue and helps keep the final
escape route open. The rivalry changes because of events the player participates in.

The ending returns to the first bridge. Light is flowing through eight restored
beacons; people and companions from each region arrive. Vell brings a machine that
shares light. Rowan proposes exploring together. A newly visible constellation
points to the sanctuary where Glimmer has sheltered displaced creatures.

The player always has overlapping goals:

- **Now:** reach a landmark, solve an encounter, help a particular friend.
- **Next:** earn a seal, try a team combination, develop a companion, investigate a silhouette.
- **Eventually:** restore eight beacons, resolve the Observatory crisis, complete the twelve-friend guide.

## How the twelve mechanics connect

| Borrowed mechanic | Concrete Wonderwilds system | Why another system benefits |
| --- | --- | --- |
| Catching and collecting | Habitat clues → observe preferences → help encounter → invitation | Each recruit adds an affinity, field ability, personality and personal quest |
| Training and evolution | Four unique milestones per friend, five levels, technique specialisation and a visible Wayfinder form | New techniques change encounter plans; personal quests reveal more of the world |
| Team building | Twelve collectable friends, two travelling slots, two equipped techniques plus universal actions | A specialist and support partner cover different hazards and create combinations |
| Interconnected exploration | Three regions, fifteen named locations, sixteen bidirectional route links, camps and visible locked landmarks | New seals make old locations useful again |
| World abilities | Brace, bloom, resonate, ferry and illuminate | Open routes, reveal caches and shorten return journeys |
| Turn-based battles | Friendly rival contests, Stillworks machines and environmental challenges | Preparation, affinity, energy, partner timing and supplies affect the solution |
| Gym-style milestones | Eight keeper trials, each with investigation, project and tactical finale | Earn a seal, change the world and advance the central mystery together |
| Care and resources | Expedition energy, free camp recovery, a three-item pouch and personal care preferences | Decide who goes, what to carry, when to rest and whether to explore farther |
| Recurring rivals | Rowan's evolving strategy and Vell's escalating interventions | Game systems express character development and create anticipation |
| Side quests and puzzles | Flow routing, chime ordering, valve sequencing, rescue, delivery, fossils and optional designs | Find useful clues, alternate routes and distinctive keepsakes |
| Secrets and post-game | Early unreachable island, hidden habitat caches, fossil trails and sanctuary | Ability unlocks and collection goals keep earlier places relevant |
| Trading and social discovery | NPC clue/keepsake exchanges in the core; specified paired human expedition mode | Share discoveries, complete alternative collections and use complementary roles |

These are linked systems. A badge that changes no route, an evolution that changes
no options, or a recruit who contributes only another portrait does not meet this
specification. Each major reward must explain what the player can now try.

## Campaign structure

There are **26 main missions**: the welcome, three missions for each of eight
trials, and the constellation ending. Each trial arc contains a causal discovery,
a project that changes the situation, and a challenge using the prepared world.
There are also five named side quests and twelve companion personal-quest briefs.
The JSON contains IDs, prerequisites, locations, objectives, rewards and choices.

| Trial / keeper | Story and player action | Consequence |
| --- | --- | --- |
| Bridge / Hazel | Repair the crossing; distinguish storm damage from removed moonstone mounts; hold the braces while a traveller crosses | Bridge opens, brace kit, second team slot; brass filings implicate a machine |
| Bloom / Fern | Compare two garden beds; redirect irrigation; protect seedlings from the collector's draught | Bloom paths, garden design persists, another starter's help quest; the emergency-reserve label is found |
| Echo / Orla | Follow Flutter's clues; reconstruct a warning with chimes; keep the tower shutters open | Resonate locks, Orchard road, Flutter; the collectors were meant to stop after the storm |
| Harvest / Cleo | Investigate missing deliveries; recover a gear by canal or ridge; redirect the mill automaton | Mill crafting hub, Bramble and Cleo quests; the ledger reveals Vell's order |
| Current / Patter | Trace reversed water; rescue Rowan's partner; open the bypass and restore the landing | Ferry routes, Cove access, Rowan exchanges; the cost of his strategy becomes clear |
| Tide / Marina | Investigate a supposedly dangerous glow; compare tracks and pipes; stabilise a refuge | Restored habitat; Glimmer is revealed as a rescuer, not the cause |
| Beacon / Sail | Recover the lens seen in the Meadow tower; choose tunnel or ridge; align the beacon | Illuminate routes, Observatory approach; the old map reveals the shared-beacon solution |
| Accord / Vell | Present accumulated evidence; reconnect beacon lines; stabilise intake, shutters and light return | Restore all regions, change Vell's machines into wayposts, open the finale and sanctuary |

Harvest and Current can be tackled in either order after Echo. Tide requires both.
Optional habitats, puzzles, rival meetings and return visits sit alongside the main
path. The entire campaign is not a single list of conversations to click through.
Travel follows the authored route graph; no objective teleports the player.

The existing seven stories supply useful building blocks: bridge repair, gathering,
crafting, lighting, listening, building a home and escorting. They need adaptation
into this plot, not a claim that this larger campaign already runs on those scripts.
The new regions and sublocations need explicit world bindings and assets.

## Companion ownership, growth and choice

Pip, Tansy and Nib are the starters. The demo prefers one short practice with the
chosen friend; the other practices are optional. The domain departure rule accepts
one completed introduction even for retained packs that list all three.
The choice changes efficient early solutions and personality responses. The player
avatar is separate. The two unchosen starters remain recruitable. Rowan's companion
is a distinct individual; his choice never removes a species from your collection.

| Affinity | Friends | Signature and trade-off | Field role |
| --- | --- | --- | --- |
| Gleam | Pip, Luna, Glimmer | Reveal: 1 energy, 2 progress; see an extra upcoming hazard | Illuminate markings and dark passages |
| Growth | Tansy, Cleo | Weave: 2 energy, 2 progress; reduce this turn's pressure by 1 | Grow crossings and wake flowers |
| Craft | Nib, Bramble | Brace: 1 energy, 1 progress; reduce this turn's pressure by 1 | Support obstacles and open supply alcoves |
| Rhythm | Flutter, Twizzle, Sail | Chime: 1 energy, 1 progress; next partner progress move gains 1 | Open chime locks and follow echoes |
| Tide | Patter, Pebble | Surge: 2 energy, 3 progress; strong burst with no shield | Cross marked water routes |

Matching the current obstacle affinity adds 2 progress. Affinities describe a
useful approach to the situation, not a child's academic strength. Later encounters
change affinity mid-challenge, making a second partner useful. Two travelling slots
are deliberate: the player can understand both roles. All owned friends rest at
camp and can be selected there; collection capacity is twelve.

Each friend has a specific habitat, personality, private concern and personal
quest. For example, Bramble must learn to ask for help, not just carry a heavier
crate; Luna guides a short night walk despite fear; Flutter gets to lead the group
that initially ignored the sound. Those events become memories and later dialogue.
Their complete scene scripts still need authoring; the JSON records the story briefs.

Recruitment has no random catch failure. Observation reveals a preference; a
mismatched approach adds a clue rather than removing trust. A help encounter leads
to an invitation. Declining preserves it. Missing eligible friends are offered
within three eligible visits, with a saved sequence that reload cannot reroll.
Glimmer is a story-earned rare discovery after the finale, not an unlimited lottery.

Growth uses the existing five-level scale, with new versioned grant definitions:

| Level / total XP | What changes |
| --- | --- |
| 1 / 0 | Joins; signature technique and universal actions |
| 2 / 10 | Rally: spend 1 energy to give the next partner progress move +2 |
| 3 / 20 | Choose a free signature use per expedition or improve rally to +3; change at camp; optional habitat cache interaction |
| 4 / 30 | Personal quest payoff, keepsake and visible bond detail |
| 5 / 40 | Distinct Wayfinder silhouette and animation, with reversible appearance selection |

First help, field partnership, personal project and Wayfinder expedition each grant
10 XP once per friend. These are distinct, contextual learning activities joined to
adventure outcomes, not repeatable XP buttons. Story unlocks are welcome, Echo,
Current and Beacon respectively. Late recruits can complete all unlocked milestones
in order. All twelve have a path to level five after the ending. Existing glow
transformations keep their old rules; the new form does not remove old rewards.

Supported learning grants the same game reward. Character level, friendship and
academic evidence remain separate. No mandatory route requires an academic mastery
label, a particular starter or a high-level companion. A new seal grants the
relevant field tool as well as licensing the companion ability. Companion use
adds personality, optional discoveries and convenient routes; it cannot strand a
player who chose another starter.

## Opening exploration and hands-on projects

The development client now exposes the Bridge and Bloom finales as distinct
construction and irrigation projects. The domain supplies saved branch discoveries
in Picnic Clearing, opens its near-bank path after the bridge clue, and joins the
third discovery to the existing gather reward. Retained content definitions still
contain the original gather and tactical actions; the primary client flow uses the
world finds and project controls. `campaign/exploration.rs` defines these explicit
opening rules; every placement and care dependency is server-checked. See the
implementation status for actual verification and current scope.

## Tactical encounters: actual decisions

These are game challenges, not a question shown before every attack. A trial can
have one meaningful preparation activity; its turns use the visible game rules.
An incorrect academic answer never damages a companion or spends a resource.

One turn is one travelling companion's move or one item use. Either partner may
act; choosing the other does not cost an extra turn. Energy is separate per friend.
Maximum energy by level is 3, 4, 4, 5, 5. Pressure represents lost footing, machine
instability or a rival gaining ground. There is no injury or death.

Universal actions are focus (0 energy, 1 progress), guard (1 energy, reduce this
turn's pressure by 2) and rest (recover 2 energy, no progress). Each trial displays
a target, turn limit, pressure limit, current affinity and next hazard. Signature
and partner effects are defined in JSON. Resting under a large hazard can be a poor
decision; guarding delays progress; a matching signature spends scarce energy to
make a larger gain. Rally costs a turn and energy now for a more effective partner
move later. The effect does not stack with itself.

Resolve a turn in this order: pay energy, apply progress and a consumed partner
bonus, check victory, otherwise apply the hazard minus the shield, then check
pressure and turn limits. Reaching target wins before the hazard. Affinity changes
apply on the following turn after reaching a phase threshold. Energy, pressure,
turns and limited-use equipment carry through phases. There are no hidden dice.

Every keeper lends a three-charge tool for the attempt: each use gives 2 progress
and reduces the current hazard by 1, costing a turn and no energy. It is available
before earning the seal; permanent field use is the reward afterward. This closes
the circular dependency where a trial would require its own reward to finish.
A guided plan can explain a working sequence without lowering the game reward.

The eight trials have different hazard cycles, targets from 6 to 12 and turn
budgets from 8 to 14. The mill changes from craft to growth; the tide challenge
changes from tide to craft; the final reserve moves through craft, rhythm and gleam.
Rowan's three authored contests use increasing targets and round limits to express
his changing pace and team strategy. Losing an optional contest does not halt the
story. Final values need playtesting; these are initial rules, not calibrated balance.

## Expedition preparation and care

At camp, choose the pair, techniques, level-three specialisations and up to three
single-use items. Key items and reserved quest materials do not take pouch slots.
Resting at camp is free and restores energy; it does not erase progress.

- Two berries make a snack: restore 2 energy to one friend.
- Two leaves make a shelter patch: reduce this turn's pressure by 2.
- One wood and one glow pebble make a trail marker: the next progress move gains 2.

Each costs a turn in an encounter. Camp forage supplies enough for one of each.
It refreshes after a previously uncompleted project, trial or personal quest, not
on reload or repeated menu actions. Supplies are useful insurance and route-planning
choices; every mandatory trial also has a supply-free solution with keeper equipment.
There is no shop economy or selling quest items in this first campaign.

After an encounter, remaining energy carries into the expedition. Push on with
another partner, use a snack, or take a route back to camp. Failure returns to the
last camp and restores energy, retaining clues, XP, ownership and quest materials.
Consumed optional items stay consumed. Keeper equipment resets. A permanent shortcut
therefore has practical value: it makes recovery and returning to a secret easier.

Care also includes play, grooming, favourite places and quiet rest. These give
personality and memories independently of item consumption. No offline hunger,
neglect penalty, dying companion or attendance streak. Repeated care cannot farm XP.

## A complete example of the interlocking loop

1. At the opening bridge, see an island with a broken beacon. You cannot reach it.
2. The first trial unlocks bracing and a second team slot. Choose whether to keep
   the starter in front or bring the newly befriended companion into encounters.
3. At the hollow, Flutter's rhythm technique offers a different solution. Recruiting
   Flutter gives you a partner for future relay combinations and a personal story.
4. In Orchard, decide whether to fix the mill first for its crafting hub or restore
   the river first to open ferry travel. Prepare your pouch for that expedition.
5. Rescue Rowan's exhausted partner. The event shows why distributing work matters.
   The Current seal grants ferry capability and a folding raft alternative.
6. Return to the opening island. Find a beacon blueprint and a unique charm. The
   blueprint explains the shared network, advancing the mystery ahead of Vell.
7. Use the new route to reach Cove; befriend Pebble after testing the tide markers.
   Their tide signature changes your options against the refuge-pool collector.
8. When that encounter changes to craft, choose a support combination, a tool charge
   or a different travelling partner for the next attempt. Care and preparation
   now affect how you solve a story problem.
9. Restore the refuge. Glimmer's tracks become evidence of rescue. The later
   sanctuary reward resolves a question you first encountered hours earlier.

That journey simultaneously advances story, collection, team strategy, learning,
resource planning and map knowledge. It does not rely on extra reward currencies.

## Puzzles, exchange and continued discovery

Three puzzle definitions specify initial state, goal, available actions, reset and
supported route: garden water flow, visible/sounded chime order, and pump valve
sequence. They are environmental game rules. If an embedded academic task is used,
its answer and evidence still come from the learning engine. Puzzles support
untimed exploration; sound always has a visual equivalent.

The fossil side quest offers a real choice: restore the leaf or shell fossil first
for a different clue and keepsake. Cleo gives two rubbings, allowing an exchange
for the other branch later without destroying the original. Rowan trades a copied
watchtower clue for a ridge clue. Exchanges show both sides and have one-time grant
keys. These are authored NPC interactions in the single-player core.

Human social play is a **specified extension**, not claimed multiplayer support:
a paired expedition with a known partner, first on a shared device. Navigator and
keeper roles alternate each encounter. Each player controls their own companion;
learning evidence belongs only to the learner whose work was assessed. Duplicate
keepsakes and authored clue cards can be exchanged after both sides confirm.
Companion ownership is never traded away. Networked play would require authenticated
sessions, atomic exchange and disconnection recovery; no free-text chat or random
matchmaking is assumed. Every friend, ending and collection reward has a solo/NPC
path. Social play is designed into the progression without making another person
necessary to finish it.

Post-finale objectives are the Glimmer sanctuary quest, all twelve friendships,
personal quests and Wayfinder forms, revisited habitat caches and optional remixed
trial boards. New boards use authored patterns and do not repeat milestone XP.
There are finite, visible completion goals, not a claim of endless generated content.

## Delivery and compatibility

Follow the [build plan](../../docs/CAMPAIGN_BUILD_PLAN.md) for implementation order,
asset batches, state integration and milestone acceptance.

The first playtest checkpoint must prove **interaction between systems**: starter
choice, a meaningful team/technique decision, the Bridge trial, a changed route,
a recruitable second friend, care recovery and a return-visit reward. A character
picker and XP bar alone do not meet it. Next deliver Bloom/Echo and the Meadow
mystery, then the branching Orchard arcs, then Cove and the ending. Human paired
play follows the solo campaign's authoritative grant and identity foundations.

Reuse the existing Rust domain, story actions, learning controls and transaction
boundary. Current story and older companion/training state are separate streams;
connect them explicitly instead of inventing frontend XP. Pin content and rules
versions for pending tasks and turns. Retry uncertain writes with the exact body
and ID. Existing saves need opt-in campaign adoption and an explicit old/new grant
mapping. Preserve current progress, inventory, ownership, appearances, learning
records and pending missions. No demo reset or historical reward inference.

The v2 JSON is a design contract. Runtime v1 replaces its objective prose with
finite executable main-story, friendship, growth and side-quest actions. Scene
blocking, exact world coordinates, production transformation art, full narrated
accessibility and audience playtesting still require their own evidence. The
treatment itself is never interpreted as executable code. Human paired expeditions
remain a separate implementation milestone.

An LLM may later propose new instances of this format from approved catalogues.
It cannot invent assessment, new executable mechanics, reward policies or existing
save mutations. Review includes exact encounter solutions, route and supply
reachability, supported paths and narrative quality, not just JSON validity.

## Verification

With the existing `contracts/` npm dependencies installed:

```sh
node content/campaign/check-runtime.mjs # executable first-slice schema/references/supplies
node content/campaign/check-v2.mjs
node content/campaign/check.mjs # superseded v1 design regression only
```

V2 checks the schema, IDs, mission prerequisite cycles, route/ability/seal
reachability, all twelve recruitment references, XP thresholds and initial optional
recipe supply. An offline search finds a supply-free, level-one solution for each
starter against all eight trial boards, with keeper equipment: 24 witnesses.
Eleven negative cases cover malformed or inconsistent content and an impossible
trial. The search checks the universal/signature/keeper subset; two-friend combos,
upgrades, puzzles, expedition persistence and social exchanges still require runtime
tests. It does not prove balance, accessible UI, full narrative playability or
educator approval. Existing story release and integrity gates remain unchanged.


Executable checks reject duplicate action/grant IDs, missing locations, routes,
flags, encounters and recruits; contradictory prerequisites; unfunded quest-material
consumption; unreachable puzzle goals; repeatable puzzle rewards; unmarked learning;
and circular campaign dependencies, incomplete main-mission coverage, and missing
recipient-specific growth gates. Twenty negative mutations exercise these
failures. Branch/plank quantities are conserved exactly. The closure check deliberately
overapproximates mutually exclusive choices and tactical victories: it is a content
sanity check, not a domain completion witness. Rust transition tests must establish
actual starter paths, supported joins, retry semantics and board outcomes.

Puzzle controls set two irrigation destinations or three ordered chime/valve
positions, check an explicit goal, or reset.
Wrong routing never consumes a resource. The check reveals the result; dialogue is
not parsed for effects. Garden designs are mutually exclusive persistent flags.
Starter choice and canonical ownership are Rust commands; friendship invitations do
not remove an owned companion, and the first post-Bloom recruitment locks further
Bloom recruitment. Nib's later invitation is not pulled forward from Echo.


The runtime content contract deliberately differs from the public wire schema.
`requires_flags` and `forbids_flags` are conjunctive predicates; `costs` and `rewards`
are integer supply deltas committed once. `grants_flags` persist story, seal, tool,
choice and prop consequences. `encounter` references a typed board; `puzzle` selects
a supported reversible control operation with an explicit goal. `learning: true`
requires a verified learning join; `recipient` and `milestone` pin a unique growth
reward, with the learning engine selecting the private task from the current
curriculum frontier. Player-facing `learning_context` contains only the short
concrete invitation. Story progress never reads an academic mastery label.

Canonical growth grant identity is companion plus milestone, independent of action
ID. The bridge preparation's unpinned first-help binds the selected team lead;
that starter's later named first-help scene cannot award it again. Late recruits
can complete all four story-unlocked milestones in order. The checked schema does
not substitute for transaction/idempotency tests of this binding.


All five licensed field abilities have explicit companion and free-tool methods.
Brace opens a return alcove; Bloom opens the hollow path; Resonate opens the Orchard
gate; Ferry reaches the opening island; Illuminate reveals the Observatory ascent.
Each pair shares one lasting outcome and completion predicate, so choosing the
other method cannot repeat a cache reward. Matching companion affinity and the
licensed seal are checked by Rust, while tool methods require the permanent free
kit. Opening projects borrow equipment before earning their own seals.


[field-guide.v1.json](field-guide.v1.json) is the separate public twelve-friend
catalogue: names, habitats, visible clues, observations and respectful preferences.
Every unowned friend has an observation and wait/help/play approaches. The stated
preference opens the existing contextual help project; another respectful approach
saves a further clue without spending supplies, losing trust or closing the right
approach. Canonical ownership hides redundant discovery/help/invitation actions.
The original story eligibility gates and supported help rewards remain intact.
