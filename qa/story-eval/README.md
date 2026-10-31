# Offline story evaluation

This is a synthetic **scene-generation** pilot. It prepares real runtime inputs,
imports candidate scenes, checks bindings and executable actions, prepares quality
judge requests, and exports human-approved supervised examples. It does not call a
model, upload data, train, change gameplay, or validate newly invented missions.
Use Python 3 and the repository's pinned Rust toolchain. No Python dependencies.

The separate [mission planning evaluator](planning/README.md) tests structured
mission plans against bounded completion searches in the real authored runtime.
It includes explicit inference against the current Modal model and keeps planning
results separate from this scene-writing dataset.

The [versioned pilot dataset](datasets/pilot-2026-10-30/README.md) preserves the
137 inputs, 411 generated candidates, independent quality review and 96/40
training/evaluation shortlists. It includes commands for verifying, reviewing,
revising and preparing later training experiments without relying on temporary files.

The user's subsequent [acceptance of the pilot recommendations](approvals/pilot-2026-10-30-user-acceptance/README.md)
approves the 386 keep candidates and leaves 25 revisions unapproved. That acceptance
is recorded separately from the original model judgments and frozen snapshot.

The current [broader input pack](datasets/broader-2026-10-30-inputs/README.md)
contains 607 reachable states, 1,821 generation requests, and 607 mechanically
checked authored references across seven missions and eight lasting-choice
combinations. It contains no new model candidates, quality grades or approvals.
Verify it with `python3 qa/story-eval/dataset.py verify-inputs --dataset
qa/story-eval/datasets/broader-2026-10-30-inputs`.

For the older content version, use `dataset.py review --archived --out PATH`.
This verifies the frozen snapshot and reuses its recorded mechanical evidence;
it does not claim compatibility with the current engine. Default review reruns
current-engine checks and rejects outdated inputs. `prepare --parent-dataset PATH`
records a verified parent snapshot without transferring candidates or approvals.

## Run the local workflow

From the repository root, choose new output directories (existing outputs are
never overwritten):

```sh
python3 qa/story-eval/run.py prepare \
  --out /tmp/story-eval-run \
  --brief 'Ages 6–8; short, warm sentences; concrete next steps; no frightening or manipulative content.'

# Smoke test using the existing authored mock as the candidate provider:
python3 qa/story-eval/run.py grade \
  --run /tmp/story-eval-run \
  --candidates /tmp/story-eval-run/reference-candidates.jsonl \
  --out /tmp/story-eval-graded
```

The age range above is an example editorial target, not an approved product age
policy. Supply the intended brief explicitly. `--candidates 1..5` controls how many
generation requests are prepared per input; the default is three.

Open `/tmp/story-eval-graded/review.html`. It displays inputs, candidate outputs,
hard checks and any imported quality judgments. All external text is escaped and
the page runs no scripts. The reference smoke test establishes pipeline operation,
not story quality or independent model performance.

## Model integration through files

`generation-requests.jsonl` contains one request per candidate with scenarioId,
candidateId, promptVersion and standard system/user messages. Send these messages
through an external model runner of your choice. The messages include the actual
compact scene input, scene ID, required immutable introduction entries, closed
output schema and editorial brief. They exclude the checkpoint and authored answer.

Wrap each returned scene in this **single-line JSONL record**:

```json
{"scenarioId":"COPY_FROM_REQUEST","candidateId":"COPY_FROM_REQUEST","generator":{"provider":"your-provider","model":"exact-model-version","promptVersion":"story-scene-eval-1"},"output":{"sceneId":"...","presentation":[],"choices":[]}}
```

The empty scene above illustrates the envelope only; it is not a valid answer.
Use parsed JSON as `output`. Record malformed provider text as a JSON string in
`output` so it receives a hard failure instead of silently disappearing. Record
the actual provider/model/prompt versions, including any prompt customization.
Re-run `grade` with your candidates and a new output directory. Partial batches
are supported; report candidate/scenario coverage when comparing models.

Hard-passing candidates produce `judge-requests.jsonl`. Run these through a capable
judge model. It receives the actual input and candidate, not the reference answer.
The response contains `criteria`; wrap it with the request's IDs, hash and rubric:

```json
{"scenarioId":"COPY_FROM_REQUEST","candidateId":"COPY_FROM_REQUEST","candidateSha256":"COPY_FROM_REQUEST","rubricVersion":"story-quality-1","judge":{"model":"exact-judge-version","promptVersion":"story-quality-1"},"criteria":{"continuity":{"verdict":"pass","evidence":"The scene acknowledges the three carried planks."}}}
```

Include **all seven** criteria: `continuity`, `choice_fidelity`, `clarity`,
`character_voice`, `age_suitability`, `education_separate`,
`unnecessary_repetition`. Each needs `pass`, `fail` or `uncertain`, and brief evidence.
The shortened example above is not a complete judgment. Import judgments by
running `grade` again with the same candidates plus `--judgments judgments.jsonl`
and another new output directory. Unknown IDs, duplicate records, missing criteria
and stale candidate hashes fail closed. Missing judgments remain visibly unjudged.

There is deliberately no automatic judge-score threshold for training inclusion.
First compare judge verdicts with human reviews, investigate disagreement, and
improve the rubric. No model judgment establishes child safety or actual enjoyment.

## Human review and export

Use the local review screen on macOS or Linux:

```sh
python3 qa/story-eval/review_server.py
```

Open [http://127.0.0.1:5182/](http://127.0.0.1:5182/). Enter your reviewer name,
read the state and scene, then choose **Approve unchanged**, **Needs revision** or
**Reject**. Revision/rejection require a reason. Approve records that you reviewed
the unchanged scene; an optional note can explain your choice. Model grades and
recommendations are fetched only after a decision has successfully saved. Returning
to a decided scene reveals them again. The original decision/time is retained if
you later change your mind after seeing feedback.

The default queue contains the 96 training-shortlist candidates. Separate queues
cover the 40 held-out examples or all 411 candidates; filters show pending/decided
items. Last position/filters are remembered when browser storage is available.
Decisions themselves are saved on disk, independent of browser storage, and survive
server restarts. Notes still being edited are drafts: navigation warns before
discarding them. Failed saves remain visible and do not unlock grades.

Progress defaults to
`qa/story-eval/experiments/human-review/pilot-2026-10-30/progress.json` (ignored by Git).
Back it up or use **Download decisions** to obtain `human-reviews.jsonl` for the
existing export command. The internal progress JSON is not the JSONL export.
Use separate progress files for independent reviewers:

```sh
python3 qa/story-eval/review_server.py --port 5183 \
  --progress qa/story-eval/experiments/second-reviewer/progress.json
```

`--dataset PATH` selects another verified snapshot. A saved progress file cannot
silently move to changed candidate data. Only one server may write a progress file.
This is a loopback-only developer tool, not a hosted multi-user review service.

Suggested replacement text is saved with a **Needs revision** decision. It does
not modify the frozen candidate. To use an edit, create a new dataset candidate,
validate it, and review its new hash. The screen disables approval while replacement
text is present, and the server/exporter also reject such an approval. Revised and
rejected examples do not enter training exports.

To use JSONL directly instead of the screen:

Copy `reviews-template.jsonl` to a separate file. Set selected records' decision
to `approve`, `revise` or `reject`, add a reviewer identifier and explanatory notes. Leave
unreviewed records `pending`. Approvals require both reviewer and notes; document
any disagreement with the judge. Do not approve an entire generated batch solely
because its JSON passed. Hard failures cannot be approved for export.

```sh
python3 qa/story-eval/run.py export \
  --run /tmp/story-eval-run \
  --graded /tmp/story-eval-graded \
  --reviews /tmp/story-eval-reviewed.jsonl \
  --out /tmp/story-eval-approved
```

Outputs are provider-independent SFT-style `train.jsonl` and `eval.jsonl` messages,
plus provenance and manifests. The assistant target is the approved scene; grader
verdicts, checkpoints and human notes are not supplied to the model as answers.
The exact generation messages are retained. Candidate hashes prevent a review
from approving a subsequently changed response. These are integrity checks for
local artifacts, not an authenticity/security boundary against malicious editors.

Whole scenario journeys are assigned to stable training/evaluation groups before
generation. Never train on `eval.jsonl`. All current families use the same seven
authored missions (the historical pilot used three). Training examples with exactly the same input and presentation
metadata as any held-out scenario are excluded at export and flagged in review.
Near-duplicate semantic situations still exist: this pilot cannot
establish generalization to unseen mission structures. Build a separate held-out
mission catalogue before making that claim. Avoid large volumes of paraphrases
dominating a later training dataset. Export does not deduplicate intentional variants.

## What the hard grader proves

The Rust corpus follows real domain actions across the bridge, picnic and lights,
including partial collection, pause/resume, alternate sources, different lantern
orders and initial supplies/repaired bridge variants. Checkpoints are replayed and
the input/reference/presentation are rebuilt and compared before grading.

The grader invokes the existing scene schema and binding validator. It checks
the scene ID, every legal action exactly once, present speakers, unique entry IDs,
and mandatory introduction text. Each offered action is resolved and applied to
an independent copy of the state. This checks immediate executability without
letting model output change inventory or completion. It does not prove arbitrary
mission solvability, truthful prose, appropriate tone or absence of repetition.
There is no fallback substitution during evaluation: an invalid candidate remains
a recorded failure rather than receiving credit for the authored fallback.

The Rust tests deliberately mutate examples to test unknown handles, missing
choices, absent speakers and extra state-grant fields. They also show that a
structurally valid false completion claim can pass the hard checks: semantic
judging and human review remain necessary. Existing journey tests cover the
authored missions; this harness is not a generic mission planner or solver.

## Verification

```sh
cargo test -p wonderwilds-director --locked
python3 -m unittest discover -s qa/story-eval -p 'test_*.py'
CHROMIUM_PATH=/path/to/chromium node qa/e2e/story-human-review.mjs
```

Use synthetic game state only. Keep working runs outside the repository or in
ignored `qa/story-eval/experiments/`. Preserve selected versioned datasets under
`datasets/`, including their provenance and checksums; avoid committing disposable
HTML/build outputs or provider credentials.
The normal root check includes the workflow's Python tests. A small real CLI run
as above checks Rust/Python integration without PostgreSQL, a browser or API keys.

## Modal QLoRA experiment

The [Modal runner](modal/README.md) prepares the separately accepted revised
authored references, verifies current mechanical validity and splits journeys for
training/validation/test. Its default GPU job performs two optimizer steps, saves
an adapter and verifies a fresh reload. Preparation and the Python dry run are
local; `modal run` explicitly launches paid cloud work. No model is connected to
gameplay by these tools.


## Current serving endpoint with JSON Schema

The separate explicit inference runner below evaluates the production scene prompt
and `response_format` payload against the current Modal endpoint. It selects the
first reachable state per mission/status/location/required-introduction combination
from the current Rust corpus (36 cases at this revision). It uses synthetic states,
not the historical training validation split, and does not replace that evaluation.

```sh
python3 qa/story-eval/endpoint_scene.py \
  --out qa/story-eval/experiments/NEW-scene-schema-run \
  --env-file /path/to/local/model.env
```

It preserves selected corpus, exact requests, raw responses, request/corpus hashes,
transport failures and real Rust mechanical grades. Only compact game inputs and
required presentation metadata are sent; checkpoints and reference scenes remain
local. It performs no response repair, authored substitution, retries, training or
release approval. The server is asked for strict `StoryScene` JSON Schema output;
local validation remains authoritative. See
[measured results and the observed length violation](RESULTS_JSON_SCHEMA_2026-10-31.md).

## Live mission capture and feedback

`sh scripts/local-demo.sh` now sets `WONDERWILDS_MISSION_LOG` to
`.local-demo/missions.jsonl` unless a different nonempty path is supplied.
A separately launched API captures only when this variable is set. Use synthetic
local play only. Parent directories must already exist. New files use owner-only
permissions. Captures and feedback stay under ignored `.local-demo/` by default;
keep alternate paths outside tracked source and back them up as needed.

Each `mission-capture-1` JSONL record contains a unique attempt ID, timestamp,
kind (`mission_plan` or `neighbourhood`), exact request body (model, prompts,
schema and sampling settings), final output, elapsed time and provider-validation
result. Malformed final text is retained as a string. Busy, transport, HTTP,
oversized and malformed-envelope failures may have no final output. Credentials,
authorization headers, endpoints, provider error bodies, session/profile IDs and
learning records are not included. Ordinary scene dialogue is not captured.
The model alias is recorded, not an assertion about its deployed weight version.

`providerValidation: passed` means the provider's structural checks passed.
The subsequent state-bound/domain completion checks, authentication and commit
can still reject it or save an authored fallback. This is an attempt log, not
proof that a mission was played or committed. Exact retries and reloads do not
regenerate saved missions; existing saved content is not backfilled. Logging is
best effort: append failures emit a fixed diagnostic and never change gameplay.
One API process should write each capture file; there is no automatic rotation.

Start the separate loopback review tool from the repository root:

```sh
python3 qa/story-eval/mission_review.py
# Optional independent capture/feedback files or another loopback port:
python3 qa/story-eval/mission_review.py --log /tmp/missions.jsonl \
  --feedback /tmp/mission-feedback.json --port 5185
```

The default address is [http://127.0.0.1:5184/](http://127.0.0.1:5184/).
Enter a reviewer name, select a capture and grade all five criteria from 1–5:
story coherence/payoff, clear next steps, variety/originality, learning invitation
fit, and suitability for ages 6–8. Choose **Accept as written**, **Needs revision**
or **Reject**; revision and rejection require notes. All attempts, including
invalid output and failures, can receive feedback. A high score or acceptance
is a human opinion, not mechanical validation or release/training approval.

Reviews are keyed by attempt ID, the full capture fingerprint and reviewer name.
Different reviewers retain independent decisions; saving again replaces that
reviewer's current decision for that attempt. This is not an immutable review
history. The default `.local-demo/mission-feedback.json` is replaced atomically,
with an exclusive writer lock. Failed saves preserve prior decisions. Changed
captures cannot inherit grades. The pending filter applies to the entered reviewer.
New captures appear on **Refresh captures**; an incomplete trailing append is
withheld until complete. Invalid complete records produce a visible read error.
**Download captures + feedback** exports one JSONL row per captured attempt with
matching current reviews. This format is separate from the scene-training export;
it does not automatically approve or train on any data.

Verification (synthetic data, no paid model requests or persistent demo writes):

```sh
cargo test -p wonderwilds-api --locked
python3 -m unittest discover -s qa/story-eval -p 'test_mission_review.py'
CHROMIUM_PATH=/path/to/chromium node qa/e2e/mission-feedback.mjs
```
