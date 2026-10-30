# Offline mission planning evaluation

This tests the current model on `StoryMissionPlan`, independently of scene writing.
It never accepts generated plans into game saves, trains weights, or changes a deployment.
Python uses the standard library; Rust uses the pinned workspace dependencies.

The [31 October baseline results](RESULTS_2026-10-31.md) record 18/18 ordinary
mechanical passes and 17/18 ordering-stress passes from the current model. The
[dependency prompt comparison](RESULTS_DEPENDENCIES_2026-10-31.md) improves the
stress test to 18/18 while retaining 18/18 ordinary passes, without fine-tuning.

## Run

From the repository root, choose fresh output directories (existing files are not overwritten):

```sh
python3 qa/story-eval/planning/run.py prepare --out /tmp/planning-inputs
python3 qa/story-eval/planning/run.py grade --run /tmp/planning-inputs \
  --candidates /tmp/planning-inputs/references.jsonl --out /tmp/planning-reference-check

# Explicit inference against the same deployed wonderwilds alias used by the game.
# Reads only MODAL_PROXY_TOKEN_ID and MODAL_PROXY_TOKEN_SECRET; never executes the file.
python3 qa/story-eval/planning/run.py generate --run /tmp/planning-inputs \
  --env-file /path/to/local/model.env --out /tmp/planning-model
python3 qa/story-eval/planning/run.py grade --run /tmp/planning-inputs \
  --candidates /tmp/planning-model/candidates.jsonl --out /tmp/planning-results
```

New runs default to `--planner-prompt dependencies-v1`. This adds a dependency
check based on capability requirements and effects, including recursive prerequisites,
starting facts and consumed resources. It does not supply a mission-specific answer.
Use `--planner-prompt baseline` to reproduce the original prompt. Prompt variants
have distinct versions and hashes; manifests without `plannerPrompt` still resolve
to the original baseline. Neither saved requests nor previous results are rewritten.

To compare both condition orders with the revised prompt:

```sh
python3 qa/story-eval/planning/run.py prepare --planner-prompt dependencies-v1 \
  --out /tmp/planning-dependencies-ordinary
python3 qa/story-eval/planning/run.py prepare --planner-prompt dependencies-v1 \
  --condition-order reversed --out /tmp/planning-dependencies-reversed
```

Generate and grade each prepared run separately with the commands above. Compare
identical scenario hashes, condition order, user messages and inference settings;
only the system message should differ from its corresponding baseline.

For an ordering stress test, prepare a separate run with
`--condition-order reversed`. This reverses only the supplied condition vocabulary,
leaving state, capabilities and grader semantics unchanged. It has a distinct
prompt version and request hash. Generate and grade it with the same commands;
compare it as a paired robustness check, not extra independent mission coverage.

Credentials can instead be exported in the shell. The endpoint is the existing
Modal development endpoint in `run.py`. Only synthetic planning input, output
schema and editorial instructions are sent. Checkpoints, reference plans and
credentials are excluded from model messages and saved reports. Redirects are
refused. Requests are sequential, temperature zero, low reasoning, 2,048 output
tokens and a 60-second per-request timeout; there are no automatic retries.
An interrupted batch retains flushed records. Grade it to report missing coverage;
use a fresh directory for a deliberate rerun. An endpoint alias is recorded honestly
as an alias, not an immutable weight revision.

`review.html` contains escaped input/output, failure details and executable
completion witnesses. `summary.json` includes coverage and per-mission results.
`responses.jsonl` preserves provider responses, finish reasons, timing and errors;
`generation.json` records settings. Generation failures count as unsuccessful
requests but are separately identified: an unavailable endpoint says nothing
about model planning quality. Malformed/truncated outputs receive no repaired
JSON or authored-fallback credit. Imported candidates use the same envelope as
the scene evaluator; grade one candidate per scenario in a comparison run.

## Corpus and mechanical evidence

The corpus contains 18 distinct prompts derived from replay-checked synthetic
states: seven mission starts, seven partially completed missions, supplied-plank
and repaired-bridge cases, plus two later missions with contrasting lasting history.
They are taken immediately after acceptance or during progress, so this evaluates
planning/replanning an available authored opportunity, not selecting a new mission
from several competing opportunities. Exact duplicate prompts are excluded.

The Rust grader verifies the shared schema, opportunity, known/unique participants,
unique available conditions and explicit coverage of the terminal outcome. It
searches the actual domain action menu for a path that first achieves the requested
remaining milestones in order and reaches the domain's completed mission state.
Already recorded conditions count as satisfied; simultaneous conditions are allowed.
Alternative legal orders (for example either lantern bank first) are supported.
Successful witnesses are replayed through checkpoint validation. Searches stop at
10,000 expanded states or 40 actions; hitting a bound is **unproven**, not proof
that a mission is impossible. Search excludes cancelling/accepting another mission.

Conditions omitted by the initialized runtime (such as wood acquisition when
planks already exist) are unavailable milestones even if listed in the broader
opportunity vocabulary. References are pipeline checks, not independent model
outputs or approved training targets. Prompt, scenario and candidate hashes bind
saved artifacts; replay rejects tampered or stale runtime inputs.

## What this does not establish

- Novel mission invention, multiple-opportunity selection, or generalization beyond
  the existing seven authored mechanics. These families overlap scene-training work.
- Prose truth: a valid condition ID can still have a misleading label, premise or
  payoff. Review clarity, continuity, motivation, age suitability, meaningful variety
  and education separation separately. The report supplies a review checklist;
  there is no automatic prose score or human approval.
- Recovery after arbitrary future actions, arbitrary branching plans, persistence
  integration, live gameplay readiness, or a fine-tuning improvement.

Start with this baseline, inspect failures, and improve prompts/context before
considering planner training. A broader catalogue and independent mission holdout
are needed before drawing conclusions about creative mission generation.

## Checks

```sh
cargo test -p wonderwilds-director --locked
cargo clippy -p wonderwilds-director --all-targets --locked -- -D warnings
python3 -m unittest discover -s qa/story-eval -p 'test_*.py'
```

Rust regressions cover real reference completion, impossible ordering, both lantern
orders, invented bindings, duplicate conditions, missing outcomes and input drift.
Python checks cover strict parsing, prompt/reference separation, env-file parsing,
manifest drift, preserved outputs, transport failures, partial coverage and HTML
escaping. Prompt-version tests preserve legacy manifests and reject mismatched
variant metadata. No database or browser setup is required.


## Schema-constrained endpoint requests

New `prepare` runs default to `--response-mode json-schema`. Requests include
`response_format: {"type":"json_schema","json_schema":{"name":"StoryMissionPlan",
"strict":true,"schema":...}}`, using the same contract and referenced
`StoryNarrative` definition as the live API. The schema remains in the prompt too.
Use `--response-mode prompt-only` for a controlled baseline. Old manifests without
`responseMode` retain their original prompt-only requests and hashes. Generation
forwards the recorded format exactly; it does not retry without constraints.

The [31 October structured-output results](../RESULTS_JSON_SCHEMA_2026-10-31.md)
record both 18-case planner reruns and a separate scene sample. Runtime validation
and completion searches remain required even with constrained generation.
