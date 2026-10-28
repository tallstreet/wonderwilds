# ADR: WW-02 contract-first modular Rust backend

Status: accepted engineering baseline; implementation gates remain separate.

The existing Phaser/TypeScript guest game remains intact. The backend uses a Rust
workspace, pinned Rust 1.99.1 and committed Cargo.lock. npm is retained for the
frontend and a separate locked contracts package. Shared wire models live in the
`wonderwilds-contracts` crate. Domain, learning and content depend only on contracts;
API and director may depend on those pure crates. Domain must not depend on API,
web frameworks, browser types, databases or provider SDKs. Rust, not the browser or
director, will authoritatively assess and commit cloud progress.

`contracts/schemas/v1.json` is the wire source of truth. The generator emits serde
models and TypeScript types. Models alone do not enforce numeric or string bounds:
call `wonderwilds_contracts::validate` at Rust trust boundaries and Ajv 2020 on the
TypeScript boundary before consuming data. Both execute the same positive and
negative fixtures. Closed objects reject body identity, fabricated answers on
encounters, extra director fields and unsupported versions. Typed parameters cover
six families; semantic matching of template/parameters, allowed IDs, arithmetic,
revision/request correspondence and dialogue safety remain deterministic service
validation, not JSON Schema claims. Director proposals cannot represent rewards,
inventory patches, correctness or arbitrary commands.

Version `1.0` is frozen for this initial implementation baseline. Exact version
acceptance is intentional; a minor version requires tested readers and fixtures.
SkillNode is versioned by its enclosing pack, not by adding fields to every node.
Unknown curriculum mappings stay null and content remains pending human review.

The OpenAPI 3.1 document describes the gameplay route contract only, not a running
server. Parent identity/lifecycle endpoints will be specified with WW-08/09 once
session and persistence integration is concrete. Attempt and hint handlers must
verify that the GameCommand variant and encounter ID match the route. Commands
receive identity from the authenticated session and use command ID/revision
idempotency. Public wire types omit answer keys; internal task definitions hold
solutions. The API must validate typed payloads before authentication-sensitive
loading and transactional state changes, never trust client support flags.

Run from repository root with the Rust cache activated:

```sh
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH=/workspace/.cargo/bin:$PATH
python3 scripts/generate-contracts.py --check
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cd contracts
npm ci --ignore-scripts --no-audit --no-fund
npm run typecheck
npm test
```

`python3 scripts/generate-contracts.py` refreshes both generated languages; it uses
rustfmt from the pinned toolchain. Review schema/consumer fixture changes together.
Rollback is removal of the isolated workspace/contracts additions; no guest-save
or frontend runtime migration is introduced by WW-02. External crate checksums and
TLS verification remain enabled. Crate shells establish boundaries and are not
claims that all later feature tasks are implemented.
