# Offline story evaluation

This is a synthetic **scene-generation** pilot. It prepares real runtime inputs,
imports candidate scenes, checks bindings and executable actions, prepares quality
judge requests, and exports human-approved supervised examples. It does not call a
model, upload data, train, change gameplay, or validate newly invented missions.
Use Python 3 and the repository's pinned Rust toolchain. No Python dependencies.

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

Copy `reviews-template.jsonl` to a separate file. Set selected records' decision
to `approve` or `reject`, add a reviewer identifier and explanatory notes. Leave
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
generation. Never train on `eval.jsonl`. All families still use the same three
authored missions. Training examples with exactly the same input and presentation
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
```

Keep generated runs outside the repository and use synthetic game state only.
The normal root check includes the workflow's Python tests. A small real CLI run
as above checks Rust/Python integration without PostgreSQL, a browser or API keys.
