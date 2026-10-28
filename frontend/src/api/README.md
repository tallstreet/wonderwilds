# Cloud client and legacy import preparation

`GameClient` uses generated `@1` contracts and same-origin session cookies. Supply the current CSRF token through its callback and call `selectProfile` with a parent-owned profile ID. The server must check that selector's ownership. Profile changes and `clearContext` abort pending requests and invalidate prepared commands, preventing late responses from updating another child's view. Call `clearContext` before logout and discard rendered cloud state.

Prepare a `GameCommand` once, then reuse that prepared value after uncertain transport failures. The client preserves its exact body/ID/revision and rejects ID reuse with changed content. It surfaces revision conflicts for an explicit refresh/reconciliation flow; it never retries a stale action with a new revision automatically. GET game and POST commands return a `GameSnapshot`; GET encounter returns an `EncounterInstance`. Error codes and safe correlation IDs are preserved. The current application entrypoint remains in guest mode until integration explicitly mounts this client.

`inspectLegacy` reads the original Starwhisker save without writing it. It preserves malformed input for download, uses the existing restore validator for a valid v1 envelope, and retains original bytes plus SHA-256. The proposed import retains old IDs/source version and labels all evidence `legacy_unverified`; it cannot establish server mastery. `originalSaveBlob` supports a user-initiated backup download.

There is currently **no backend import endpoint**. `importLegacy` therefore accepts an explicit transport rather than inventing an API route. Do not enable the migration UI until the server validates ownership, deduplicates `(profile, source digest)`, commits cosmetics/story import atomically and returns the defined receipt. It must retain original IDs/version and never accept browser evidence as verified mastery. A receipt is saved only after server acknowledgement of the exact profile/digest/provenance. On local receipt-write failure, the returned server receipt enables recovery; retry must use server idempotency. Original saves are never erased. Existing corrupt receipts block further import pending recovery.

Run from `frontend/`:

```sh
node --import tsx src/api/api.test.ts
npm run typecheck
npm run lint
npm run format:check
```

These nine tests exercise same-origin API paths, exact-byte retries and caller mutation, conflicts, cross-profile late responses during fetch and body parsing, corrupt/blocked storage, acknowledged imports and receipt recovery. They use an explicit fake transport for migration; they are not evidence of a functioning server import or cloud account flow. No package or legacy storage-format changes are required to remove this module.
