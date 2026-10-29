# Cloud client and legacy import preparation

`GameClient` uses generated `@1` contracts and same-origin session cookies. Supply the current CSRF token through its callback and call `selectProfile` with a parent-owned profile ID. The server must check that selector's ownership. Profile changes and `clearContext` abort pending requests and invalidate prepared commands, preventing late responses from updating another child's view. Call `clearContext` before logout and discard rendered cloud state.

Prepare a `GameCommand` once, then reuse that prepared value after uncertain transport failures. The client preserves its exact body/ID/revision and rejects ID reuse with changed content. It surfaces revision conflicts for an explicit refresh/reconciliation flow; it never retries a stale action with a new revision automatically. GET game and POST commands return a `GameSnapshot`; GET encounter returns an `EncounterInstance`. Error codes and safe correlation IDs are preserved. The main game remains in guest mode. The opt-in local `demo.html` entrypoint now mounts this client against the real development API; see the root README for setup.

`inspectLegacy` reads the original Starwhisker save without writing it. It preserves malformed input for download, uses the existing restore validator for a valid v1 envelope, and retains original bytes plus SHA-256. The proposed import retains old IDs/source version and labels all evidence `legacy_unverified`; it cannot establish server mastery. `originalSaveBlob` supports a user-initiated backup download.

There is currently **no backend import endpoint**. `importLegacy` therefore accepts an explicit transport rather than inventing an API route. Do not enable the migration UI until the server validates ownership, deduplicates `(profile, source digest)`, commits cosmetics/story import atomically and returns the defined receipt. It must retain original IDs/version and never accept browser evidence as verified mastery. A receipt is saved only after server acknowledgement of the exact profile/digest/provenance. On local receipt-write failure, the returned server receipt enables recovery; retry must use server idempotency. Original saves are never erased. Existing corrupt receipts block further import pending recovery.

`CommandSession` owns the shared command lifecycle for the world and diagnostic
screen. Its explicit idle/sending/awaiting-retry/refreshing states guard concurrent
writes; uncertain failures retain the original prepared command. A failed refresh
keeps that command available, while a successful explicit refresh adopts the
server snapshot. It has no DOM or local-storage dependency. Create one session per
selected profile and discard it with that profile's rendered state.

World journal, missions and training markup and container-scoped event bindings
live in `backend-world-views.ts`; `BackendWorld` owns the activity lifecycle and
command effects. Expanded companion details and focus restore by creature ID,
independently of display wording. `world-placements.ts` checks authored encounter
coverage, maps and coordinate bounds; route tests cover approachable placements.
These browser positions do not establish server eligibility.

Run from `frontend/`:

```sh
npm test
npm run typecheck
npm run lint
npm run format:check
```

The API and command-session tests exercise same-origin API paths, exact-byte retries and caller mutation, conflicts, cross-profile late responses during fetch and body parsing, corrupt/blocked storage, acknowledged imports and receipt recovery, concurrent command guards and explicit refresh reconciliation. They use an explicit fake transport for migration; they are not evidence of a functioning server import or cloud account flow. No package or legacy storage-format changes are required to remove this module.
