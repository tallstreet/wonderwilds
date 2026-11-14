# Missing Moonlight audience playtest

Status: prepared protocol; no participant sessions or approvals recorded.
Automated technical evidence is in [implementation status](IMPLEMENTATION_STATUS.md).
Use the project's approved recruitment and consent process for observed sessions.
This protocol does not supply that approval or establish learning efficacy.

## Setup

Run the local demo and provision separate synthetic profiles using the documented
local-demo profile command. Select the intended saved player explicitly; do not
reset another player's save. Record the
campaign content version, device, input method, support used and starting mission.
Use anonymous session labels in findings; recordings are not required.

With the isolated local demo running, an operator can explicitly add one fresh
synthetic player with:

```sh
python3 scripts/add-local-demo-player.py --create-player --ability-band beginning
```

Reload and select that saved player. This command is restricted to the local
synthetic household; it is not a production account-creation flow.

Invite the player to explore without a timer. Aim for a short session with a natural
camp stopping point. An adult may read instructions or choose supported practice;
record that support so independent understanding is not inferred from assisted play.
Observe younger and older learners separately rather than assuming one group's
results apply across the audience.

## Scenarios and observations

| Scenario | Observe | Evidence of a problem |
| --- | --- | --- |
| Choose any starter, then find the bridge task | Can the player explain the next destination and interact by their preferred input? | Repeated wandering with no usable cue, unreadable instructions, or inaccessible controls |
| Gather branches, make planks and restore the bridge | Does the changed world make the result clear? | The player cannot identify the repaired crossing or thinks progress was lost |
| Try a wrong irrigation arrangement, then change it | Is the feedback useful and non-punitive? | The player thinks the puzzle is permanently failed or cannot find reset/check |
| Enter a trial, use support and recover after a loss | Can the player explain the changing approach, pressure, partner and free recovery? | Hidden rules, repeated identical failed turns, or a perceived resource dead end |
| Recruit a friend, change the team, care and return | Does the player understand what the new friend offers and which friend is travelling? | Unclear ownership, care response or team state |
| Choose either Orchard order and use a field tool | Is the destination choice understandable, and can any starter continue? | A route appears mandatory despite a supported alternative, or a free tool is undiscoverable |
| Resume a saved game in a later session | Are the objective and prior choices recognisable? | The player cannot recover context or sees apparently reset progress |
| Reach the ending and inspect collection goals | Does completion feel clear, and are optional goals distinguishable? | The game promises unavailable invitations or the ending is mistaken for an error |
| Try shared-device roles, swap, then continue solo | Can each player identify whose companion and activity are active? | Profile confusion, one learner's response attributed to another, or inability to leave the pair |

Ask neutral prompts: “What would you like to try?”, “What changed?”, “What do you
think this friend can do?” and “Where could you stop and come back later?” Avoid
explaining the intended answer before noting what the player understood.

## Accessibility and session length

Include keyboard-only and touch sessions, zoomed text, reduced motion, and read-aloud
unavailable as well as available. A non-reader needs observed usable adult or audio
support; the presence of text alone is not evidence of independent access. Include
assistive-technology review with an appropriate reviewer rather than treating an
automated DOM check as screen-reader approval.

Record time to a natural stopping point, requests for breaks, repeated travel,
confusing dialogue and optional activities skipped. Tune pacing, turn budgets and
resource frequency from these observations. Solver completion does not establish
enjoyable balance or an age-appropriate session length.

## Findings record

For each finding, record: anonymous session label; content version; device/input;
scenario; what happened; support offered; impact; reproducible steps; proposed fix;
owner; and retest result. Separate observations from interpretations. Keep an empty
record empty until an actual session occurs.

A blocker includes lost or misattributed progress, an inaccessible required action,
or inability to continue with support. Resolve and retest blockers before requesting
the plan's narrative, curriculum, artwork, accessibility and release reviews. Each
review needs its actual reviewer, date, scope and decision; technical tests do not
fill in those fields.
