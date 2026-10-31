#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export PATH="$CARGO_HOME/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd contracts && npm test && npm run typecheck && npm run check:generated)
python3 content/curriculum/validate.py
python3 -m unittest discover -s content/curriculum -p 'test_*.py'
node content/curriculum/check-schema.mjs
python3 content/activities/validate.py
python3 -m unittest discover -s content/activities -p 'test_*.py'
python3 content/story/validate.py
python3 content/creatures/art/validate.py
python3 -m unittest discover -s content/story -p 'test_*.py'
python3 -m unittest discover -s qa/story-eval -p 'test_*.py'
(cd frontend && npm run check)
node --import ./frontend/node_modules/tsx/dist/loader.mjs qa/fixtures/legacy/baseline.test.ts
