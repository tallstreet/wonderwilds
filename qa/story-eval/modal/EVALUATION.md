# Baseline and adapter evaluation on Modal

Run commands from the repository root with the uv-created environment active:

```sh
source /tmp/wonderwilds-modal-env/bin/activate
```

If it needs recreating, follow [local setup](README.md#local-setup-and-checks).
Use the same Modal workspace and the two volumes created for training. No serving
endpoint is needed. Each evaluation requests one H100, has a one-hour timeout,
and incurs GPU cost. The baseline does not need a completed training job.

## Check locally, then measure the baseline

These checks use the prepared bundle and source export; they do not build images,
upload data or allocate a GPU:

```sh
python qa/story-eval/modal/evaluate.py --dry-run
python qa/story-eval/modal/evaluate.py --dry-run --split test
```

The default is the 76 validation scenarios. Test selects the 79 final held-out
scenarios. Selection verifies bundle and corpus hashes, exact prompt equality to
the original generation requests, and exclusion of training scenario IDs. Only
system/user messages are uploaded. Approved answers, checkpoints, reference
scenes and human review notes stay local. Training has already excluded exact
situation overlap; related authored journeys still limit generalization claims.
Each scene uses its recorded state independently; this does not simulate a live
journey driven by earlier generated scenes.

For a paid, small pipeline check, add `--limit 3` and use distinct output paths.
For the full validation baseline:

```sh
modal run --env main qa/story-eval/modal/evaluate.py \
  --out qa/story-eval/experiments/glimmer-base-validation
```

No adapter is loaded. The runner uses the pinned Glimmer revision with NF4 double
quantization and BF16 compute. Generation is greedy, seed 42, with a 2,048-output-
token limit and no input truncation. The native chat template is used unchanged.
The worker preserves raw Harmony output and extracts the final user-directed
response for JSON parsing; it does not strip fences, repair JSON or insert authored
fallbacks. Malformed JSON, generation exceptions and token-limit truncations are
recorded as failing candidates. Even parseable JSON is rejected if generation hit
the token limit without a stop token.

The downloaded result is automatically passed through the **current Rust story
grader locally**. This requires the repository's pinned Rust toolchain. Outputs:

- `suite.json`: exact input prompts, split, checkpoint and generation settings.
- `remote.json`: Modal evaluation ID and remote volume location.
- `result.json`: raw outputs, exceptions, truncation, output tokens, generation
  latency, GPU/package versions and adapter hashes when applicable.
- `candidates.jsonl`: standard story-grader candidate records.
- `graded/review.html`, `graded/hard-results.jsonl`, `graded/judge-requests.jsonl`:
  existing mechanical review and narrative-judge workflow.

Output directories are never overwritten. No actual baseline scores were measured
during implementation; local integration used explicitly synthetic fixture outputs.
If an earlier attempt failed while importing `train` in the container, update the
runner and use a new output directory (for example `baseline-check-2`). Image setup
and grading helpers now import only on the local client; the remote entrypoint
needs only Modal and the explicitly mounted worker files.

## Evaluate the trained adapter

Replace `YOUR_TRAINING_RUN_ID` with the training run ID printed by `train.py`, such
as the name of a completed `full-...` directory in `wonderwilds-glimmer-runs`:

```sh
modal run --env main qa/story-eval/modal/evaluate.py \
  --adapter-run YOUR_TRAINING_RUN_ID \
  --out qa/story-eval/experiments/glimmer-adapter-validation
```

The adapter must have completed training and reload verification. Its base
revision and training-bundle hash must match this evaluation. Saved adapter tensors
are checked against the loaded tensors. A completed smoke adapter is also accepted
for a pipeline check, but two optimizer steps do not establish useful adaptation.
Use a full-training adapter for the substantive comparison.

Use identical `--split`, `--limit`, `--max-new-tokens`, `--experiment` and `--run`
options for both runs. Defaults select the prepared `glimmer-modal-r1` experiment
and its accepted source corpus. No test data is used to tune generation settings.

## Compare

```sh
python qa/story-eval/modal/compare.py \
  --base qa/story-eval/experiments/glimmer-base-validation \
  --adapter qa/story-eval/experiments/glimmer-adapter-validation \
  --out qa/story-eval/experiments/glimmer-validation-comparison
```

`comparison.md` and `comparison.json` report mechanical pass counts/rates,
generation errors, truncation, generation p50/p95 latency, average output tokens,
and the IDs that improved or regressed. Every requested scenario stays in the
pass-rate denominator. Latency excludes loading and includes generation/CUDA
synchronization; the first-call warmup remains included. Per-request failures are
reported separately and excluded from latency/token averages.

Comparison fails if prompts/settings, coverage, package/GPU versions, evaluator
code, candidate hashes or grading integrity differ. If game validation code has
changed between runs, regrade both under the same checkout before comparing.
Mechanical validity is not narrative quality, child safety or educational efficacy.

Narrative quality uses the existing seven-criterion judge workflow. Process each
`graded/judge-requests.jsonl` with the same judge model, prompt and rubric, then
import its candidate-hash-bound judgments with the existing command, for example:

```sh
python qa/story-eval/run.py grade \
  --run qa/story-eval/experiments/broader-reference-quality-2026-10-30-r2/run \
  --candidates qa/story-eval/experiments/glimmer-base-validation/candidates.jsonl \
  --judgments /path/to/base-judgments.jsonl \
  --out qa/story-eval/experiments/glimmer-base-quality
```

Repeat for adapter candidates. Pass the resulting directories to `compare.py` as
`--base-graded ... --adapter-graded ...`. The report shows per-criterion
pass/fail/uncertain counts on jointly judged scenarios, alongside coverage. Missing
judgments are not silently counted as passes. Inspect a blinded human sample,
especially regressions and judge disagreements. This runner makes no judge API
calls and does not treat model judgments as human approvals.

Once the experiment and settings are frozen, repeat both model runs with
`--split test`, using new output directories. Reserve that comparison for the final
79-case test set; do not use it to repeatedly select checkpoints or prompts.

## Recover and regrade

The worker writes partial results after each response to
`wonderwilds-glimmer-runs/evaluations/EVALUATION_ID/`. Completion does not imply
all generations succeeded; examine each response and the hard grader. If a
container times out or loading fails, do not treat partial output as a full result.
The local grader rejects missing, duplicated or out-of-order responses.

Keep the local command attached so it can download results and run Rust grading.
If the local client disconnects after the remote job finishes, retrieve its
`suite.json` and `result.json` from the volume using the ID in `remote.json`, and run:

```sh
python qa/story-eval/modal/grade.py \
  --result /path/to/downloaded-evaluation \
  --out qa/story-eval/experiments/glimmer-recovered
```

This also regrades a previously downloaded result under the current engine into
a new directory without rerunning GPU inference. No job is retried automatically.

## Prompt-only speaker experiment

`--prompt-variant speakers-v1` adds an explicit `allowedSpeakerIds` list derived
only from `input.here.characters`, plus system instructions distinguishing present
speakers from remembered characters and mission participants. Empty lists require
narration only. Source provenance is verified before transformation; source files,
training data, required entries, generation settings and the Rust grader remain
unchanged. The transformed prompts and version are recorded in `suite.json` and
covered by its hash. The default `original` preserves the existing suite format.
This is a prompt constraint, not enforced constrained decoding.

```sh
python qa/story-eval/modal/evaluate.py --dry-run --prompt-variant speakers-v1

modal run --env main qa/story-eval/modal/evaluate.py \
  --prompt-variant speakers-v1 \
  --out qa/story-eval/experiments/glimmer-base-speakers-v1

modal run --env main qa/story-eval/modal/evaluate.py \
  --prompt-variant speakers-v1 \
  --adapter-run full-6584b280eaaf \
  --out qa/story-eval/experiments/glimmer-adapter-speakers-v1

python qa/story-eval/modal/compare.py \
  --base qa/story-eval/experiments/glimmer-base-speakers-v1 \
  --adapter qa/story-eval/experiments/glimmer-adapter-speakers-v1 \
  --out qa/story-eval/experiments/glimmer-speakers-v1-comparison
```

These select all 76 validation cases. Use fresh output directories. Keep the
original results for comparison of the two experiments; do not pair an original
baseline with a speaker-variant adapter. The paired comparison intentionally
rejects different suites. Inspect each experiment's review and pass rates to
assess prompt effects on each model. Do not use the final test set to select this
variant. No additional training is needed and no GPU run was launched during
implementation.

## Speaker and unique-entry experiment

`--prompt-variant speakers-unique-v1` retains the exact `speakers-v1` user payload
and speaker instructions, then adds system instructions for unique entry IDs and
non-repeated dialogue/narration. Required entries stay unchanged and appear once;
their IDs are reserved. Additional entries must use distinct IDs and contribute
new information rather than padding the array. No outputs are repaired or
silently deduplicated. This remains a prompt-only experiment, not a guarantee.
The original and `speakers-v1` variants remain available unchanged.

```sh
python qa/story-eval/modal/evaluate.py --dry-run --prompt-variant speakers-unique-v1

modal run --env main qa/story-eval/modal/evaluate.py \
  --prompt-variant speakers-unique-v1 \
  --out qa/story-eval/experiments/glimmer-base-speakers-unique-v1

modal run --env main qa/story-eval/modal/evaluate.py \
  --prompt-variant speakers-unique-v1 \
  --adapter-run full-6584b280eaaf \
  --out qa/story-eval/experiments/glimmer-adapter-speakers-unique-v1

python qa/story-eval/modal/compare.py \
  --base qa/story-eval/experiments/glimmer-base-speakers-unique-v1 \
  --adapter qa/story-eval/experiments/glimmer-adapter-speakers-unique-v1 \
  --out qa/story-eval/experiments/glimmer-speakers-unique-v1-comparison
```

Use all 76 validation cases for both models and fresh output directories. Compare
complete grader results and inspect repetition in the story reviews before
choosing a variant. Earlier partial results are not full-suite pass rates. Keep
the final test split reserved. Neither training nor GPU inference is triggered
by the local preflight.
