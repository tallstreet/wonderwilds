# Curriculum practice inside missions

Updated: 31 October 2026. First five-skill integration implemented; final checks and delivered
scope are recorded in `IMPLEMENTATION_STATUS.md`. This is development content,
not an educator-approved mastery instrument or released curriculum.

## Product loop

The player's curriculum state persists with their profile. Rust reads the
existing prerequisite graph and selects a skill the player is ready to practise.
An activity belongs to the current mission: preparing bridge materials, counting
picnic supplies, arranging lights or organising the gathering. The mission gives
the work a purpose; it does not choose an arbitrary harder skill to fit a prop.

The player reaches an activity by gathering, investigating, exchanging or
assembling within a mission. Generated investigation steps may author a short
`learningBridge` joining line around the server's discovery-marker interaction;
old content uses a mechanic-specific authored fallback. The same assessed task
shapes and fingerprints are reused with consistent prop labels. There is no
separate practice-launch button in the playable story world.

The player can try, ask for a concrete hint, learn with a worked example, complete
the step together with a server-authored solution, or put the task aside. Answers and support update server-owned practice evidence.
When the experimental readiness criteria are satisfied, the next supported
prerequisite-ready skill becomes available. Missions remain accessible through
supported routes without independent academic mastery.

## Existing graph and first activity coverage

Load the existing Foundation and Years 1–6 packs: 120 nodes (70 maths and 50
English). Keep their prerequisite edges, identifiers, pending reviews and
disabled release gates. A graph node is not an implemented activity. The initial
reachable maths path is:

| Node | Skill | Original mission activity |
| --- | --- | --- |
| M01 | One-to-one correspondence | Assign a supply to each recipient/place, with no missing or repeated recipient |
| M03 | Counting collections | Count mission supplies with objects and structured layouts |
| M04 | Comparing collections | Compare aligned or paired supply groups |
| M05 | Counting sequences | Fill a missing step in a route or preparation sequence |
| M06 | Composing and partitioning | Work out the missing part of a collection of mission materials |

M03 requires M01; M04, M05 and M06 require M03. Other nodes retain their actual
graph prerequisites and an explicit unavailable-activity status. Do not skip
prerequisites to reach the older M05/M06 assessment templates. English and later
years remain on the expansion roadmap, not silently removed or marked mastered.

## IXL references and authoring decisions

Research checked 31 October 2026:

- [Australian Foundation catalogue](https://au.ixl.com/maths/foundation):
  count pictures, ten frames, matching/comparing collections, number sequences
  and partitioning are relevant activity patterns for this slice.
- [Learn with an example](https://blog.ixl.com/2019/05/22/introducing-two-helpful-features-on-ixl/):
  a worked sample introduces the strategy before practice.
- [Helping students get unstuck](https://blog.ixl.com/2020/07/23/helping-students-get-themselves-unstuck/):
  explanations and prerequisite practice offer routes through difficulty.

Use these public references for teaching/interaction patterns. Author Wonderwilds
wording, examples, quantities, art and assessment rules. Do not copy question
banks, reverse-engineer SmartScore or claim IXL endorsement. Individual interactive
skill pages were not retrievable in this research; the catalogue and official
instructional descriptions establish patterns, not a completed item-by-item audit.
The existing coverage files treat IXL as a breadth comparison and ACARA alignment
as a separate unapproved review; preserve that distinction.

## State and authority contract

Use additive `/api/v1/mission-learning` snapshot and command routes. The
existing story, assessed-adventure and guest saves retain their formats and
semantics. Persist mission learning separately under the same owned profile.
Neither the browser nor LLM supplies skill readiness, answer keys or rewards.

- A begin command selects the skill and binds the activity to authoritative story
  context. The browser cannot nominate a higher skill, fake a mission or mark a
  prerequisite complete. Reads do not create activities.
- Persist the active item, its original mission context, policy/content version,
  first-check outcome and monotonic support. Reload resumes the same activity.
  New story actions do not reframe an already issued item or erase its support.
- Use the verified server session for evidence diversity. Reload, browser-supplied
  session identifiers or starting another item cannot manufacture a return visit.
- Commands use exact request IDs/bodies, revision checks, profile locking and
  atomic original-response receipts. Check old receipts before rejecting stale
  revisions. An uncertain write is retried unchanged after reload.
- Add a numbered migration with forced RLS and profile export/deletion coverage.
  Bound current state; retained receipts/evidence may grow and need operational
  retention planning. Do not modify applied migrations or persistent demo data
  during destructive QA.
- No learner history is added to the external story-provider prompt. `begin_step`
  resolves a legal story action and revision under the profile lock and freezes
  its mission/generation, location, label and joining text in the activity.
- Correct answers and explicit `finish_together` commands execute that bound
  story action in the same transaction as evidence, checkpoints and both receipt
  logs. Rust derives the actual story effects; learning does not invent resources
  or rewards. Together-completion records a worked example with `not_assessed`,
  not a successful answer. Exact retries return the original learning response
  without repeating the story action. Stale bindings fail atomically; abandonment
  can still put them aside. Direct story/inspector commands remain compatible.
- Browser completion and retry recovery refresh authoritative story state. A
  failed refresh retains the confirmed learning receipt and locks world actions
  until story recovery, rather than rendering a historical receipt as current.

## Progression and mastery distinction

Preserve `independent-transfer-v1` and its existing secure-mastery semantics.
The assessed adventure's compact `authored-no-transfer` projection is unchanged.
The mission slice uses a separate versioned experimental practice-readiness
policy. UI language must distinguish readiness to practise the next skill from
validated mastery or school-grade achievement.

Initial readiness target: eight distinct independent first checks, at least seven
correct, the latest three correct, at least three item variants, two contexts,
two permitted representations and two verified server sessions. Support cannot create qualifying evidence. A canonical item occupies at most one
slot: a later independent first check can replace its projection without adding
a duplicate qualifying item. Retain the original evidence for audit. This allows
recovery after mistakes and later-session review even after exhausting the finite
item bank; it is a versioned difference from the existing transfer policy. Showing hints or
worked examples requires committed support before the material is revealed.
Help never reduces access to the story. An incorrect first check followed by
correction cannot become an independent first-check success.

The policy intentionally does not claim reviewed novel transfer. Educator-reviewed
transfer tasks, difficulty calibration and accessibility research are required
before treating this readiness as the existing secure-mastery status. Preserve
evidence and version the policy so future changes can be audited rather than
rewriting prior outcomes.

## Acceptance and follow-on work

Verify graph loading and prerequisites; real assessment of each supported task;
no readiness from hints, retries, repeated items or one session; advancement after
the declared diverse evidence; and explicit unsupported-node coverage. Exercise
wrong answers, teaching, abandon/resume, restart, exact lost-response retry,
stale commands, ownership/CSRF/RLS, profile export/deletion and transaction rollback.
Browser checks cover mission discovery, keyboard/touch controls, phone layout,
progress feedback and continuing the story without mastery. Existing guest,
training and story journeys must remain compatible.

After this slice, expand activity coverage in prerequisite order, strengthen
misconception-specific feedback and review spacing, calibrate transfer evidence,
and evaluate natural placement within each mission with human playtesting.
Full graph coverage and educator/release approval remain explicit future gates.
The existing persistence CLI does not yet audit/recover mission-learning state;
this slice verifies full journal replay in integration tests. Add operator tooling
and retention design before production use.
