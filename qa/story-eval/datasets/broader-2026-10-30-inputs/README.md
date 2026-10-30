# Broader story input pack — 30 October 2026

An input-only expansion for `pip-stories-3`: **607 reachable situations**, **1,821
generation requests**, and **607 authored reference scenes passing mechanical
checks**. There are no model-generated candidates, story-quality judgments or
human approvals in this pack.

Coverage includes all seven missions, optional clues, pause/resume situations,
parallel preparation orders, completed outcomes and all eight combinations of
instrument home, corner design and gathering location. States come from replayed
authored journeys rather than fabricated snapshots. Education remains separate.

The verified parent is the frozen [reviewed pilot](../pilot-2026-10-30/README.md).
Its candidates, judgments and separate user approvals remain unchanged and are
not copied into this pack. `input-pack.json` records file hashes and coverage;
this is an input pack, not a reviewed-dataset snapshot.

From the repository root:

```sh
python3 qa/story-eval/dataset.py verify-inputs \
  --dataset qa/story-eval/datasets/broader-2026-10-30-inputs

# After generating candidate JSONL using run/generation-requests.jsonl:
python3 qa/story-eval/run.py grade \
  --run qa/story-eval/datasets/broader-2026-10-30-inputs/run \
  --candidates /tmp/broader-candidates.jsonl \
  --out /tmp/broader-graded
```

Generate responses using each request’s messages and preserve its scenario and
candidate IDs. Follow the [evaluation workflow](../../README.md) for candidate
format, independent quality judging, human review and approved training export.
The grader creates quality-judge requests; mechanical passes alone do not make
training examples. Keep all new outputs outside this immutable input pack.

`reference-hard-results.jsonl` records the authored-reference check. To repeat it,
use `run/reference-candidates.jsonl` as the candidates argument with a fresh output
directory. `run/manifest.json` retains the editorial brief and parent lineage.

Eight journey families share the same authored catalogue, so the held-out split
measures related story situations, not generalization to unseen mission types.
