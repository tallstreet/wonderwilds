# Dependency prompt comparison — 31 October 2026

The `dependencies-v1` prompt fixed the observed Bramble ordering failure without
fine-tuning. All ordinary and reversed-condition cases now have replay-verified
completion paths, with no mechanical regressions against the saved baseline.

| Evaluation | Original prompt | Dependency prompt | Improvements | Regressions |
| --- | ---: | ---: | ---: | ---: |
| Ordinary condition vocabulary | 18/18 | 18/18 | 0 | 0 |
| Reversed condition vocabulary | 17/18 | 18/18 | 1 | 0 |

Both new batches completed all 18 requests with no transport, parsing or schema
failures. These are two views of the same 18 situations, not 36 independent
missions. The old results remain unchanged in [the baseline report](RESULTS_2026-10-31.md).

## Change and comparison controls

The new system prompt says to derive order from capability requirements and
effects, follow prerequisites recursively, and check the sequence against the
starting location, inventory and completed conditions. It explicitly treats
condition/capability arrays as vocabularies rather than suggested sequences.
It supplies no Bramble-specific answer, reference plan or extra world data.

The prepared scenario records and user messages match the corresponding baselines
exactly. Only the system prompt and its version differ. Both use the same endpoint
and `wonderwilds` alias, temperature 0, low reasoning and 2,048 output tokens.
The Rust grader and domain rules are unchanged. Comparison checks matched all
scenario IDs, request/candidate hashes, coverage and recorded inference settings.
The endpoint does not expose an immutable model revision; unchanged weights
cannot be independently verified from its alias alone. No training or deployment
changes were made by this work.

## Bramble result

For `bank-series-062`, the revised reversed-condition plan now says:

1. Check the workshop path (`workshop_path_checked`).
2. Clear loose branches (`workshop_path_clear`).
3. Ask whether Bramble is ready (`bramble_ready`).
4. Walk with Bramble (`bramble_arrived`).
5. Welcome Bramble (`bramble_visit_shared`).

This restores the omitted path-check milestone and places clearing before
readiness. The engine found and replayed a legal completion sequence. The original
stress-test plan put readiness before clearing and had no valid ordered path.

## Quality and limits

Assistant inspection of the 36 outputs found that the ordering improvement does
not resolve all editorial issues: technical wording such as “compatible wood,”
internal `bridge_bank` text and stale gathering instructions in partial-progress
premises remain. The supplied-planks case now correctly omits the unnecessary
woodcutter participant in both batches. This inspection is not a calibrated
narrative score or independent content approval.

Observed median request durations were 1.622 seconds ordinary and 1.585 seconds
reversed (ranges 1.376–2.743 and 1.355–1.914 seconds). These small samples do not
establish a latency change. Two earlier attempts at each batch returned only
HTTP 503 responses; those 72 failed requests remain separate availability evidence,
not model-quality scores. The successful runs use fresh `live-r3` directories.

Retain the dependency prompt for further evaluation. This is a regression fix on
a known failure in scene-training-related authored families, not an unseen-mission
holdout. Broader scenarios and narrative review remain the next useful tests;
this result does not provide a reason to fine-tune yet.

## Evidence

Under ignored `qa/story-eval/experiments/`, each
`planning-2026-10-31-dependencies-{ordinary,reversed}` directory contains prepared
inputs, requests and a manifest. Its `-live-r3` sibling contains candidate and raw
response JSONL plus inference settings; its `-r3-grades` sibling contains grades,
completion witnesses, summary and HTML review. The paired comparison is saved as
`planning-2026-10-31-dependencies-comparison.json`.

| Artifact | Canonical JSON SHA-256 |
| --- | --- |
| Shared scenario records | `839d70ef26d8f6fdf41ba2556ae53b917c297cdbecce5947152c53315fbdd959` |
| Ordinary requests | `40d318d4bf1451baf968a2f41a07365be0c8a230353ce2dec9df82ab3c6d7cbb` |
| Ordinary candidates | `28e2574b81da1fddc749502944098eecc246bc5e19285119583c0dcb9724dab6` |
| Reversed requests | `cd38fe848196f724b014bced8e2e64f05a64b8a6a7557d10383fa8a4e664f411` |
| Reversed candidates | `20014cdc3670190c2e826bebddacd7d9fd7c7e49d24b468daf59c7bd8a7e8659` |

Validation: all 51 Python evaluation tests passed, including legacy prompt
compatibility and prompt-version drift rejection. Both live batches were graded
through the real Rust executable. Diff checks passed. No Rust, browser or database
behavior was changed, so their broader suites were not rerun for this prompt edit.
