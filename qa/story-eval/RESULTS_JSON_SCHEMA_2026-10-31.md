# JSON Schema serving evaluation — 31 October 2026

Both API providers now send `response_format` with `type: "json_schema"`, a named
`json_schema` containing `strict: true`, and the shared output contract. The
planner includes the transitive `StoryNarrative` definition. Prompt schemas and
local validators remain in place. No model weights, gameplay rules or persistence
behavior changed, and no request retries without schema constraints.

| Evaluation | Passed | Transport failures |
| --- | ---: | ---: |
| Mission planning, ordinary conditions | 18/18 | 0 |
| Mission planning, reversed conditions | 18/18 | 0 |
| Scene generation, current live prompt | 35/36 | 0 |

The planner scenarios, prompts, candidate IDs and generation settings match the
prior `dependencies-v1` ordinary/reversed runs. Removing only `response_format`
from each prepared request reproduces the prior request exactly. Both previous
prompt-only batches also passed 18/18. The Bramble ordering case `bank-series-062`
continues to pass. Successful planner grades include real-domain completion
witnesses; these are known authored families, not unseen missions.

Scene selection takes the first exported reachable state for each combination of
mission ID, mission status, location and required-introduction presence. The
resulting 36 cases cover all seven supported missions plus pre-mission states.
This is a mechanical endpoint integration sample, not the historical training
validation split or a prose-quality assessment. It uses the same prompt file,
allowed speakers, schema and editorial brief as the live scene provider.

## Remaining constraint violation

`bank-series-060` returned valid JSON but a 210-character narration where
`StoryScene.presentation.items.oneOf` limits text to 180 characters. The Rust
validator rejected it. Thus request acceptance and `strict: true` do **not** prove
that this server enforces every constraint in these schemas. The game retains its
local validation and authored fallback; the evaluator records the failure without
substitution. No prompt change or response repair was used to conceal it.

A separate synthetic request asked for `{"probe":"prompt_only"}` while its schema
required `const: "schema_enforced"`. It returned `{"probe":"schema_enforced"}`,
confirming that the endpoint enforces at least that constraint rather than simply
ignoring the parameter. Another simple schema limited `probe` to three characters
while the prompt requested `"overlong"`; it returned `"..."`. This does not diagnose
the cause of the nested scene violation or establish general schema compliance.

## Reproduction and evidence

Use `planning/run.py prepare` with its default `--response-mode json-schema` and
`--planner-prompt dependencies-v1`, then `generate` and `grade`; repeat preparation
with `--condition-order reversed`. Use `endpoint_scene.py --out NEW_DIR --env-file
/path/to/model.env` for the scene sample. Both runners use the existing Modal
endpoint, `wonderwilds` alias, temperature 0, max 2048 tokens, low reasoning,
60-second evaluation timeout and no automatic retries. The API retains its
six-second timeout. The endpoint does not expose an immutable weight revision.

Raw artifacts are preserved in ignored `qa/story-eval/experiments/`:

- `schema-2026-10-31-planning-{ordinary,reversed}`: scenarios, requests and manifests.
- Corresponding `-live` directories: candidates, raw responses and generation settings.
- Corresponding `-grades` directories: summaries, witnesses and review reports.
- `schema-2026-10-31-scenes`: selected/full corpus, requests, raw results and grades.
- `schema-2026-10-31-enforcement` and `schema-2026-10-31-length-probe`: probe requests and results.

Request/corpus hashes bind the local artifacts. Credentials are read as data from
the authorized local environment file and are excluded from evidence. Checks run:
API tests (31 passed, three DB tests ignored), all 53 Python evaluation tests,
Clippy for API/director, `cargo fmt --all --check`, and `git diff --check`.
