# Native companion motion pipeline

The first technical gate was Pip's base form: one source atlas, actual Phaser
registration, measured frame/foot anchors, desktop and phone inspection, and a
real timed signature sequence. Only after that gate did production expand to
other identities. Human art, child/audience and accessibility review remain pending.

## Existing source audit

The three regional companion sheets contain one front pose per friend. Existing
hero side/back atlases contain static Pip views, while `rosie-walk-v1.png` contains
actual movement for a different pink kitten. Rosie's frames cannot stand in for
Pip, Tansy, Nib or another recruit. Software bob/flip/scale of these stills is not
counted as native animation coverage. New motion uses each existing base/form
identity as its reference; existing static portraits, artwork and hashes are retained.

## Twenty-pose contract

Every motion atlas follows the same logical frame order. Actual returned dimensions
and gutter positions vary, so runtime registration reads the per-sheet metadata
rather than assuming the requested 1536×1920 grid was delivered exactly.

| Logical cells | Native frame art |
| --- | --- |
| 1–3 | Front/down idle and two alternating strides |
| 4–6 | Right idle and two alternating strides |
| 7–9 | Back/up idle and two alternating strides |
| 10–12 | Left idle and two alternating strides |
| 13–15 | Signature anticipation, release and settle |
| 16–18 | Cheer preparation, lift and settle |
| 19–20 | Rest settle and eyes-closed rest |

Each direction has its own drawn art, including rear faces/costumes. Runtime never
flips one heading to claim another heading. A two-stride cycle is a minimal walk
animation; four/eight-stride refinement and richer transitions are not implied.
Directional idle currently uses one still per heading, without a breathing/blink
cycle. Signature, cheer and rest use distinct articulated frame art. Reduced motion
uses a stable directional idle, or stable resting pose, rather than repeated frames.

## Sources, metadata and review

`<identity>-motion-v1.request.json` records the identity description, reference,
exact generation prompt, original output and optional cleanup output. The source
PNG and any background-cleanup edit are retained separately under `sources/`.
The public atlas is a byte-for-byte copy of the selected generated source; these
production steps do not programmatically paint/key/reshape raster pixels.

Some first generations contain translucent background washes despite a transparent
request. Those require an explicit image-generation cleanup edit, with its prompt
and original source retained. A true alpha-zero gutter is checked before metadata
is published. Existing world butterflies are ambient decorations, not stray atlas
fragments; source inspection and actual-background comparison distinguish them.

`measure_motion.py` reads alpha geometry without altering pixels. It identifies
row gutters, records exact source rectangles and measures support anchors near the
lowest opaque part of each pose. These measurements are provisional until actual
rendering is inspected, particularly for quills, long tails and transparent wings.
A matching `<identity>.frames.json` is the source of frame/anchor truth. All current
metadata is published atomically through `motion-registry.v1.json`; the helper uses
a shared file lock to avoid competing atlas batches overwriting registry entries.

The asset manifest owner deliberately registers each frozen atlas through
`register_motion.py`. Provenance preserves the source hash, dimensions, reference,
exact prompt, available poses, limitations, review status and inspected preview
paths. Registration/validation never changes pending human review into approval.

## Runtime and technical preview

`frontend/src/campaign/animation.ts` resolves the selected base/form identity to its
own atlas, uses distance-based strides, applies measured foot anchors and plays
short visual effects. `world.ts` skips the older static bob/flip path when native
art applies. Campaign signature/cheer/rest effects follow confirmed server events;
the motion module cannot grant a companion, game XP or a form.

The runtime loads active identities through a bounded texture cache instead of
preloading all native atlases. Loader races, disposal, live-image pinning and cache
limits require their own tests; the existence of small atlas files is not a mobile
memory-budget result.

Run a renderer-only fixture for one registered identity:

```sh
node qa/e2e/campaign-motion-preview.mjs /tmp/tansy-motion tansy
node qa/e2e/campaign-motion-preview.mjs /tmp/tansy-form-motion tansy-wayfinder
```

It renders all twenty registered native frames through the actual Phaser world,
checks frame names/no flip, captures a desktop view and four phone groups, and
verifies a timed signature traverses three frames then returns to idle through the
world event hook. The fixture is explicitly synthetic: it does not establish
canonical ownership, an earned transformation, supported learning, server rewards,
human approval or production scene blocking. Those require the separate campaign
journeys and review evidence.
