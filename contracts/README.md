# Wonderwilds wire contracts v1

Source: `schemas/v1.json` (JSON Schema 2020-12). API: `openapi.yaml` (OpenAPI 3.1,
JSON syntax is valid YAML). Generated serde models and TypeScript types are checked
in, with positive/negative shared cases in `fixtures/cases.json`.

From the repository root run `python3 scripts/generate-contracts.py --check` and
`cargo test -p wonderwilds-contracts --locked`. From this directory run `npm ci`,
`npm run typecheck`, and `npm test`. Use `npm run generate` to regenerate.

Validate wire data before deserializing into generated models. Schema validation
checks representation, versions and bounds; the deterministic engine separately
checks semantic constraints and request authorization. `SkillNode` carries version
through its content pack. Pending reviews are not release approval. No child data,
credentials, real account IDs or executable generated content belong in fixtures.
