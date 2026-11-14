# Shared-device pairing and attribution

The existing verified parent session can select only profiles owned by that
parent. The profile header remains the answering learner. Navigator/keeper roles
are tactical permissions and never substitute for that identity or redirect
issued activities, companion rewards, mastery or personal campaign commands.
The browser explicitly reloads a selected learner's own client state.

A separate household coordinator owns two participant IDs, owned companion
selections, finite practice turns, exact keepsake offers and confirmations. Its
fixed `paired-1` rules are pinned in the storage envelope. Only the navigator
starts practice; turns alternate between the actors whose roles allow those
moves. Every turn records the actor and their own companion. Roles swap at a
practice boundary and may be swapped explicitly outside active practice. This
practice creates no academic evidence, XP or campaign rewards.

Transactions lock the household parent row, then the participant profile rows in
lexical ID order. Personal campaign and learning writers lock only profile rows;
they do not subsequently lock a parent row. Every new paired command checks
revision and ownership under these locks. Exact receipts are bound to the acting
profile and original request, and contain the original response and resulting
paired state. The checkpoint, revision and latest receipt are checked together.
Unknown rules or malformed state fail closed. Existing receipt formats stay intact.
A durable parent-owned high-water revision survives participant and receipt
deletion. Missing or invalidated coordinators resume solo at that revision; new
commands advance it atomically, so stale prepared commands cannot attach to a new
pair and historical receipt revisions are never reused.

Keepsakes are reciprocal **copies** of actually owned authored, whitelisted
nonquest collectibles. No arbitrary item can be minted. Each selected learner
offers an owned original; changing either offer clears both confirmations. Both
originals are rechecked when confirming. Only two matching confirmations record
the two copies in one transaction. Originals remain in their independent owners'
campaigns. These copies confer no XP, academic evidence or quest resources and
cannot be offered onward as originals. A per-owner/source/item uniqueness key
prevents repeated pairing from duplicating the same copy.

Forced RLS protects household state/receipts and profile-owned copies. Profile
export includes their pair participation and copies. Deleting a participant
removes their coordinator and receipts; a surviving owner's received copy remains
with its source cleared. Continuing solo preserves each learner's canonical
companions, campaign, pending learning and keepsakes.

This implements shared-device play under one verified household. It does not
implement network invitations, independently authenticated paired sessions,
network disconnection recovery or transfers of canonical assets. An explicit
operator CLI can create another actual profile only in the isolated synthetic
local demo; the app never invents another learner or silently provisions one.
