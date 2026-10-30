# Muse Glimmer QLoRA on Modal

This opt-in developer job trains scene-writing adapters from approved synthetic
Wonderwilds scenes. It does not connect a model to gameplay. Default execution is
a smoke run: eight short training examples, two optimizer steps on one H100,
checkpoint save, fresh-process adapter reload, tensor equality checks and bounded
generation. `--full` explicitly selects one epoch with validation. Both modes
have a one-hour function timeout and no automatic retries. GPU jobs incur cost.

## Prepared on 30 October 2026

`../experiments/glimmer-modal-r1/bundle/` contains 221 training, 76 validation,
79 final test and 8 smoke examples. All 607 approved source scenes passed the
current Rust mechanical grader. Export excludes 110 cases overlapping the original
held-out set, producing 418/79. Reserving the complete `bank-series` journey for
validation excludes another 121 overlapping/duplicate training records.

The pinned tokenizer passes loss-mask and 4,096-token bounds checks for every row;
maximum length is 1,764 tokens (smoke maximum 1,034). The training targets include
the complete assistant turn, including its routing header and terminator; all
system/user tokens are masked. No targets are truncated. These authored scenes
remain repetitive; the split does not establish unseen-mission generalization.
Approval was user-authorized batch acceptance after model review, not independent
blind human labeling. No GPU run, remote image build or deployment has been tested.

## Local setup and checks

Run from the repository root. The development session already installed the local
requirements in `/tmp/wonderwilds-modal-env`; recreate it with `uv` when needed
(tested with uv 0.13.1):

```sh
uv venv /tmp/wonderwilds-modal-env
source /tmp/wonderwilds-modal-env/bin/activate
uv pip install -r qa/story-eval/modal/requirements-local.txt
uv pip check
python qa/story-eval/modal/train.py --dry-run
```

That Python dry run validates hashes and messages without authentication, image
builds, uploads or GPU allocation. Do not use `modal run --dry-run` for a guaranteed
local-only check: Modal may initialize the App before its entrypoint runs.

Optional real-tokenizer preflight downloads tokenizer assets only, not weights:

```sh
HF_HOME=/tmp/wonderwilds-glimmer-hf python qa/story-eval/modal/preflight.py \
  --bundle qa/story-eval/experiments/glimmer-modal-r1/bundle \
  --out qa/story-eval/experiments/glimmer-modal-r1/tokenization.json
```

Output paths must be new. The current prepared preflight report already exists.
The remote worker independently repeats masking and length checks with its own
installed tokenizer version before downloading weights.

## Launch the smoke run

Authenticate to the intended `garytafe` workspace (the profile name need not match
the workspace name), then create the two volumes in its `main` environment:

```sh
source /tmp/wonderwilds-modal-env/bin/activate
modal setup
modal profile list
modal volume create wonderwilds-glimmer-data --version=2 --env main
modal volume create wonderwilds-glimmer-runs --version=2 --env main
modal run --env main qa/story-eval/modal/train.py
```

If volumes already exist, reuse them. The first launch builds the CUDA image and
caches the approximately 60 GB base checkpoint. No Hugging Face secret is needed
for these ungated public weights. Local credentials are not copied into the image.
The runner sends only the manifest and train/validation/smoke message files;
held-out test targets and human review notes stay local. Source code is added by
an explicit file allowlist, not a repository upload.

Watch logs under the `wonderwilds-glimmer-qlora` App in Modal. Each invocation gets
a fresh run ID; a supplied `--run-id` must not already exist. The output volume
contains `/RUN_ID/status.json`, `config.yaml`, `environment.json`, `train.log`,
`tokenization.json`, `adapter/`, checkpoints and `reload-check.json`. `completed`
is written only after adapter tensors reload correctly and generation succeeds.
Generation uses a training prompt and is a loading check, not a quality score.
The run commits output on normal completion and exceptions; Modal's background
volume commits also persist intermediate writes. A forced timeout may leave
`running` status and incomplete artifacts: inspect the Modal failure, do not
interpret that directory as a successful run. The job never resumes automatically.

For a later full experiment, first measure the untuned baseline and reserve the
final test split. Then explicitly launch:

```sh
modal run --detach --env main qa/story-eval/modal/train.py --full
```

The full run uses 221 training examples and only the 76 validation examples for
validation loss. It does not use the 79 final test examples. Retrieve the adapter
and evaluate base versus tuned generated scenes with the existing story harness
before considering any game integration. There is no serving endpoint or merge
into base weights in this workflow.

## Rebuild the bundle

The accepted source snapshot is an existing **ignored local artifact** at
`qa/story-eval/experiments/broader-reference-quality-2026-10-30-r2`. Back it up with
the prepared bundle; a fresh checkout alone cannot recreate that snapshot.
Committed approval hashes do not substitute for its actual data. The preparation
command fails if the snapshot or decisions no longer match the recorded acceptance.
It rechecks current engine validity and exports approved unchanged targets before
splitting whole journeys and removing exact situation overlap. It never overwrites
an existing experiment:

```sh
python3 qa/story-eval/modal/prepare.py \
  --out qa/story-eval/experiments/glimmer-modal-r2
python qa/story-eval/modal/train.py --dry-run \
  --bundle qa/story-eval/experiments/glimmer-modal-r2/bundle
# When ready to launch, pass that same --bundle path to modal run.
```

Hashes, split counts, excluded counts and training-row lineage live in each bundle.
All generated data, reviews and remote run artifacts belong under ignored
`experiments/`; never add model weights, credentials or child data to Git.

## Dependencies and limits

The image installs dependencies through Modal's `uv_pip_install`, pinned to uv
0.13.1, including the CUDA extensions with build isolation disabled. Build tools
(`setuptools`, `wheel`, `packaging`, `ninja`) are installed before FlashAttention;
its build imports `wheel` even though it does not declare that dependency. It checks
the resulting environment with `uv pip check --system`.
The image follows Meta's Axolotl 0.19.0 recipe: Python 3.12, CUDA 13.0, PyTorch
2.12.1, torchvision 0.27.1, FlashAttention 2.8.3 and a pinned Cut Cross Entropy
commit. The model/tokenizer revision is
`a4e59da52a7bc87ae7251dd5545c0dd437c44b68`. Language-layer LoRA targets keep the
vision components frozen. Rank/alpha follow the upstream example (32/16).
Transitive package versions are resolved during image build and recorded at run
time; this is not a fully locked transitive environment. Build/GPU compatibility,
checkpoint reload and memory use must still be established by the remote smoke.

References:
- [Meta Glimmer training recipe](https://dev.meta.ai/docs/muse-glimmer/post-training)
- [Axolotl Glimmer config](https://github.com/axolotl-ai-cloud/axolotl/blob/v0.19.0/examples/muse-glimmer/qlora.yaml)
- [Axolotl pretokenized labels](https://docs.axolotl.ai/docs/dataset-formats/tokenized.html)
- [Modal volumes](https://modal.com/docs/guide/volumes)

Local regression checks:

```sh
python3 -m unittest discover -s qa/story-eval -p 'test_*.py'
python3 -m compileall -q qa/story-eval/modal
```

## Measure baseline and adapter quality

Use the [Modal evaluation runner](EVALUATION.md) to generate the 76 validation
scenes or final 79 test scenes with either the pinned base or a completed adapter.
It automatically runs the local story grader and supports matched comparisons,
including optional narrative judgments. This is separate from the training smoke
run's single-prompt loading check.

## Export a standalone BF16 checkpoint

The exporter loads the pinned original base without bitsandbytes, verifies the
completed adapter, merges with PEFT safe merging, casts floating weights to BF16,
and saves sharded safetensors plus processor/tokenizer assets. A fresh process
checks file hashes, tensor dtypes, configuration and local checkpoint generation.
This is not a story-quality evaluation or proof that a particular serving recipe
accepts the export. BF16 behavior may differ from the evaluated NF4 model.

```sh
modal run --env main qa/story-eval/modal/export_bf16.py --dry-run
modal run --env main qa/story-eval/modal/export_bf16.py \
  --adapter-run full-6584b280eaaf \
  --export-id full-6584b280eaaf-bf16
```

The actual export requests one H100 with 192 GiB host memory and a two-hour
maximum runtime. Expect roughly 60 GB of model weights plus cached base files.
Existing destinations are refused; retries need a new export ID. Original adapter
files are never overwritten. Do not deploy a partial/failed export: wait for
`status: completed` and `verification.json` in the export directory.

Deployment values after successful completion:

- Environment: `main`
- Volume: `wonderwilds-glimmer-runs`
- Volume-relative model path: `exports/full-6584b280eaaf-bf16/model`
- With the Volume mounted at `/runs`: `/runs/exports/full-6584b280eaaf-bf16/model`
- Precision: BF16, unquantized (not NVFP4 or bitsandbytes)

The checkpoint does not bundle the evaluation system prompt. Configure the chosen
speaker/unique-entry prompt separately in the serving integration.
