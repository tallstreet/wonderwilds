# Story scene pilot — 30 October 2026

Frozen synthetic dataset for future evaluation and training experiments. No real
player data, account/session secrets, model API keys or approved training targets.

This describes the original snapshot. The user's later
[batch acceptance](../../approvals/pilot-2026-10-30-user-acceptance/README.md) is
preserved separately, with reproducible export instructions and decision provenance.

| Contents | Count |
| --- | ---: |
| Reachable game situations | 137 |
| Subagent-authored candidate scenes | 411 |
| Independently judged candidates | 411 |
| Keep / revise / reject recommendations | 386 / 25 / 0 |
| Preferred training shortlist | 96 |
| Preferred held-out evaluation shortlist | 40 |
| Preferred candidate excluded for input overlap | 1 |

## Files

- `run/`: original replayable corpus, exact generation messages, authored baseline
  candidates and split/prompt/schema manifest. The baseline is a comparison, not
  the generated answers or a set of human approvals.
- `candidates.jsonl`: all first-generation answers, including the 25 recommended
  revisions. Do not silently remove or replace those failures.
- `judgments.jsonl`: seven quality criteria per candidate, with evidence and exact
  candidate hashes. Separate judge context; not a claim of a different model family.
- `recommendations.jsonl`: keep/revise/reject decisions and preferred scene flags.
- `hard-results.jsonl`: archived mechanical results (411 passes); the review
  command reruns the current Rust grader instead of trusting this as current proof.
- `quality-review.md`: common problems, concrete examples and limitations.
- `assessments.tsv`: the judge's working notes, indexed in original scenario order.
  Final keyed judgments/recommendations are authoritative; the temporary serializer
  mentioned in the historical review is not required to use this snapshot.
- `rubric.json` and `generation-provenance.json`: review instructions and generation
  method. The subagents authored prose; scripts bound engine-owned IDs, introductions
  and legal choice labels. This is not a raw full-JSON model benchmark.
- `recommended-{train,eval}-candidates.jsonl`: reproducible model-recommended
  shortlists. Neither is human-approved; never train on the evaluation file.
- `snapshot.json`: file checksums, counts, parent/version metadata and archive base
  commit. Exact generation-time model IDs and commit were not recorded; no precision
  is invented retroactively.

## Verify and view

From the repository root:

```sh
python3 qa/story-eval/dataset.py verify
python3 qa/story-eval/dataset.py review --out qa/story-eval/experiments/pilot-review
```

The review command needs Rust and Python, but no model, database or network call
beyond locally available Cargo dependencies. It creates a new directory, verifies
the snapshot, regrades mechanics and displays recommendations and criterion evidence
in `review.html`. It also recreates the shortlists and pending human-review template.
Ignored `experiments/` outputs can be regenerated; this snapshot stays in Git.

## Iterate without losing the baseline

1. Copy this directory to a new named dataset directory. Remove **only the copied**
   `snapshot.json`; leave this original version unchanged.
2. Edit candidate text or import fresh generations, recording the actual generator,
   model and prompt versions. For new input situations use the main `run.py prepare`
   workflow. Preserve the held-out set for comparisons; if it changes, treat it as
   a new benchmark and check that earlier training data has not leaked into it.
3. Run `run.py grade` against the new candidate file and its `run/` directory, writing
   to an experiment directory. This produces fresh hard checks and judge requests.
   Supply independent quality judgments for changed candidates. Unchanged judgments
   can be reused only when their hashes match. Update every recommendation and
   preferred flag as needed; candidate edits invalidate earlier judgments/reviews.
4. Copy the new hard results into the new dataset. Update review notes, rubric,
   generation provenance and quality summary to accurately describe this iteration.
   Keep the full candidate population, not only successful examples.
5. Seal the new version. This refreshes its derived shortlists, validates all record
   bindings and writes a new checksum manifest; it refuses an existing snapshot:

```sh
python3 qa/story-eval/dataset.py snapshot \
  --dataset qa/story-eval/datasets/YOUR_NEW_VERSION \
  --parent pilot-2026-10-30 \
  --notes 'Describe changes, generation/judging methods and any benchmark change.'
python3 qa/story-eval/dataset.py verify --dataset qa/story-eval/datasets/YOUR_NEW_VERSION
```

Snapshots are local reproducibility checks, not signed authenticity proofs.
Regrading an old snapshot against a changed runtime may reject its projections;
retain the old snapshot and evaluate a new dataset under the updated contracts.

## Prepare a training experiment

Review the preferred training shortlist first. Edit a copy of the generated
`reviews-template.jsonl`, recording actual human decisions, reviewer and notes.
Leave unreviewed entries pending. Model keep recommendations never become human
approvals automatically.

The [human review screen](../../README.md#human-review-and-export) provides these
decisions without editing JSON: run `python3 qa/story-eval/review_server.py`, then
open `http://127.0.0.1:5182/`. Model grades remain hidden until your decision saves.
Use **Download decisions** as the `--reviews` file below. Revision suggestions
remain unapproved until they become new, revalidated candidates.

```sh
python3 qa/story-eval/run.py export \
  --run qa/story-eval/datasets/pilot-2026-10-30/run \
  --graded qa/story-eval/experiments/pilot-review \
  --reviews qa/story-eval/experiments/human-reviews.jsonl \
  --out qa/story-eval/experiments/approved-v1
```

The export contains provider-independent message examples and provenance. Only
approved, hard-passing examples are exported, and exact held-out overlap is removed.
Upload/train is a separate, provider-specific action; this repository does not run
training jobs or choose a training provider automatically.

Copy [`training-run.template.json`](../../training-run.template.json) for each
actual experiment and record dataset/export hashes, prompt, model, training settings,
job ID and held-out results. Keep credentials and access tokens out. Compare the
prompted baseline and trained model on the same untouched evaluation inputs, plus
full simulated journeys. These related three-story scenarios cannot establish
generalization to novel missions or real child engagement.
