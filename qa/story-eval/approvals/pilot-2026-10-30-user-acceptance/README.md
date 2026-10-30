# User acceptance of the pilot recommendations

The user explicitly accepted the model's recommendations while requesting the
human-review/dataset PR. All **386 keep** recommendations are approved; the **25
revise** recommendations remain unapproved. Seventeen existing user decisions
were preserved unchanged. The other 394 decisions were recorded through the local
review server, so the screen and downloaded decision file reflect the same result.

This is **user-authorized batch acceptance**, not 394 independent blind human
quality judgments. Do not use those batch decisions to claim human/model grader
agreement. The notes and `acceptance.json` preserve that distinction; the 17 prior
records are separately retained in `prior-decisions.jsonl`.

Files:

- `human-reviews.jsonl`: all 411 candidate-bound decisions, compatible with export.
- `acceptance.json`: scope, authorization, counts and raw-file integrity hashes.
- `prior-decisions.jsonl`: original decisions before the batch operation.
- `export-manifest.json`: verified export provenance and counts; model training has
  not run. The larger generated exports remain reproducible ignored artifacts.

From the repository root, with new output directories:

```sh
python3 qa/story-eval/dataset.py review \
  --out qa/story-eval/experiments/accepted-pilot-review
python3 qa/story-eval/run.py export \
  --run qa/story-eval/datasets/pilot-2026-10-30/run \
  --graded qa/story-eval/experiments/accepted-pilot-review \
  --reviews qa/story-eval/approvals/pilot-2026-10-30-user-acceptance/human-reviews.jsonl \
  --out qa/story-eval/experiments/accepted-pilot-export
```

The result is **269 training examples and 114 held-out evaluation examples**.
Three approved candidates whose input overlaps evaluation are excluded from
training. These counts include all approved alternatives, rather than only the
96/40 preferred shortlists. Never upload the evaluation file as training data.
Candidate changes require new validation and approval; this authorization is
bound to the archived hashes and does not approve future generated examples.
