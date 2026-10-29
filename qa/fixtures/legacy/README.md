# Legacy guest baseline

Synthetic fixtures captured from the save-v1 contract at commit
`9e484b43eb349b9aa78342e781a3cb2f2236d524`. They contain no learner data.
These remain legacy browser observations, never server-verified mastery.

- `fresh-v1.json`: default guest state.
- `completed-v1.json`: all three Meadow rewards, a selected bunny avatar, custom
  companion name, and independent/hinted/assisted practice records.
- `invalid-v1.json`: duplicate/unknown reward, unknown map, prototype-named hero,
  contradictory support record and blocked position. Existing restoration keeps
  the known reward and replaces invalid fields; it does not reject the entire save.
- `malformed-save.txt`: syntactically invalid JSON; restore returns fresh state.
- `storage-blocked.json`: synthetic throwing-storage behavior, not serialized
  browser storage. The fixture test injects those failures into the actual adapter.
- `asset-inventory.json`: hashes, dimensions and provenance gaps for all ten PNGs.
  A documented generation prompt is evidence of provenance, not a licence opinion.

Run from the repository root after `cd frontend && npm ci`:

```sh
node --import ./frontend/node_modules/tsx/dist/loader.mjs qa/fixtures/legacy/baseline.test.ts
```

Migration must retain the original downloadable bytes and a one-time receipt.
The baseline restore helper sanitizes invalid data into memory; it does not itself
provide the new migration recovery/export UX.

Artwork inventory update (29 October 2026): `baselineCommit` now identifies the
ImgBot optimization commit `cd852f7`; `originalBaselineCommit` retains the original
capture reference. All ten optimized PNGs were decoded to RGBA and compared with
the original commit: dimensions and every pixel are identical. Only encoded byte
counts and SHA-256 values changed. The save fixtures still describe the original
save-v1 contract. Exact-byte integrity checks remain enabled.
