# Campaign artwork audit

`manifest.v1.json` is the campaign inventory, separate from the frozen creature
and story packs. Run `python3 content/campaign/art/validate.py` from the root to
check source hashes, PNG dimensions, source bounds, provenance paths, twelve
companion base/form entries and the campaign's five affinities. Passing this
check is not visual, rights, narrative or educator approval.

The 2026-11-14 audit inspected the actual Meadow, Orchard and Cove companion
sheets, the prop atlas and the three regional landscapes. The twelve friends
have distinct, readable front silhouettes. The source sheets are retained
unchanged; the existing regional metadata retains original prompts and reference
provenance. The unequal Cove column boundary is intentional: use 660/594px
columns, not an assumed 627px grid. All frames are source pixels. Suggested
128px character display boxes and 32×24px footprints are planning targets;
footprints do not create navigation blockers. Per-frame foot alignment and
phone-size readability still need review in the campaign world.

All base sheets have only one front pose per friend. Portraits can reuse these
frames during development; walking bob, scale or flip effects do not establish
walk-cycle, signature, cheer or rest coverage. All twelve Wayfinder forms now have static development art. Each retains an
explicit gap for walk, signature, cheer and rest animation. The manifest keeps these gaps explicit rather than declaring a complete
24-set animation library.

The existing bridge frame has a transparent deck and one separate plank, allowing
software to show repair progress. Sleeping/blooming garden frames have closely
matched bounds and can represent persistent before/after state. The picnic can
support a provisional care spot but is not final camp art. Landscapes have baked
paths and decorative objects; do not bake campaign inventories, unlocked paths or
collected rewards into new landscape images. Keep changed beacons and other
mission state in separate layers. Region source prompts were not found as complete
independent provenance records; the shared art-direction document is the available
legacy provenance, not an invented generation history.

The first new production sheet, `cast-v1.png`, contains Rowan (fox explorer),
Hazel (owl guide), Moss (Rowan's distinct mossback partner), and an empty collector
cart. These are original designs rather than duplicates of the recruitable roster.
`cast-v1.provenance.json` retains the exact prompt, inspected style sources, raw
source path, dimensions, frame rectangles and hash. The raw sheet is bundled under
`sources/`; the public asset is a byte-for-byte copy. Generation requested 1536px
but returned 1254px; runtime uses actual 627px cells. The upper Rowan ear and
Moss's feet have tighter gutters than requested, so source inspection alone cannot
approve game clipping or scale. All new art remains pending human review.

The frontend accepted a static 2×2 atlas with bottom-centre anchor, NPC target 96px
height and Moss target 72px height. `character-sprites.ts` preloads and registers
`rowan`, `hazel`, `moss` and `collector-cart` with the existing Phaser integration.
The first cast sheet was inspected in the actual Phaser campaign renderer;
`docs/screenshots/campaign/cast-phaser-first-preview.png` preserves this technical
gate. Cutouts and silhouettes render clearly; the opening cluster has overlapping
labels/player art, which is a frontend presentation issue, not final art approval. The same front frame is the
reduced-motion still. No directional animation is implied.

After that preview, `props-v1.png` adds care camp, matching dim/restored beacon
variants and garden irrigation. Exact prompt, raw source, frames and hash are
retained in `props-v1.provenance.json`. Bridge and garden can reuse the same beacon
variants. New sprite IDs are `campaign-camp`, `beacon-dim`, `beacon-restored` and
`garden-irrigation`; collector cart remains on the cast sheet. Source inspection
passed, while actual phone-scale prop review remains pending.

All eight seals and the five affinity symbols now have static bitmap artwork.
Six static encounter feedback effect images are now bundled; actual confirmed-turn
integration and reduced-motion presentation must be verified separately. Ordinary accessible UI symbols can use the existing SVG/CSS system.
Keep generated scenery separate from runtime quantities and world-change state.
No final art approval is claimed.

`wayfinders-meadow-v1.png` provides Pip, Tansy and Nib static Wayfinder poses plus
one moonburst emblem. Each retains species/face identity with distinct
crescent/leaf/bridge motifs. These are advancement portraits/front stills;
these source sheets remain static; separate native motion atlases now provide
minimal walk/signature/cheer/rest coverage as recorded below. Actual Phaser form preview was recorded before the remaining nine forms were
generated: `docs/screenshots/campaign/wayfinders-first-phaser.png` and
`wayfinders-first-phone.png`. These are synthetic visual fixtures, not evidence
of earned level-five progression. Desktop shows all three intact. Phone shows
Tansy/Nib readable while Pip is outside the camera viewport; it is not a full
three-form phone acceptance check. Canonical server
level/appearance determines rendering; artwork must never grant a form.

`orchard-v1.png` provides mill, friendly mill automaton, ferry raft and Reed,
Rowan's distinct long-legged marsh partner. `finale-v1.png` provides Vell (badger
inventor), lighthouse lens, Observatory machine and sanctuary. These isolated
layers avoid baking paths, quantities or progress into scenery. Each sheet has
its own exact prompt, raw copy, source references, dimensions, frames, hash and
pending review. Real region/phone preview remains required; source inspection
does not establish finale scene completion.

The remaining forms are in `wayfinders-orchard-v1.png` (Flutter, Cleo, Bramble,
Patter), `wayfinders-cove-v1.png` (Twizzle, Luna, Pebble, Glimmer), and
`wayfinders-sail-v1.png` (Sail, Bridge/Bloom/Accord medals). Inspected frames use
a 590px row split in the first sheet to preserve Bramble's ear and a 600px column
split in the second to preserve Glimmer's wing. Source pixels stay intact.
Luna's advanced form adds fin-like ear details; identity/recognition remains for
human review. All twelve forms now also have technical inspection in the actual registered
Phaser renderer at 1280×800 and 390×844, three at a time. Evidence is preserved
as `docs/screenshots/campaign/forms-group1.png` through `forms-group4.png` and
matching `-phone.png` files. The synthetic avatar is hidden so no form is
occluded. No source-to-frame clipping was observed; Bramble ears and Glimmer
wings remain intact with inspected nonuniform bounds. This establishes static
rendering coverage, not recognition testing, earned transformation or final
human approval. Luna identity embellishments remain pending human design review.

The manifest inventories production assets and companion coverage. It is not yet
a comprehensive object list for every scene; character animation coverage remains
a separate production requirement.


Reproduce a three-form technical review with standalone Vite on port 5185:

```sh
node qa/e2e/campaign-art-preview.mjs flutter,cleo,bramble /tmp/campaign-forms-group2
```

The script uses `createWorld` and registered sprite IDs, captures desktop and phone,
and grants no progress. Groups used for the complete inspection are Pip/Tansy/Nib,
Flutter/Cleo/Bramble, Patter/Twizzle/Luna, and Pebble/Glimmer/Sail.

## Final static symbol and landmark batch

`seals-v1.png` adds Echo, Harvest, Current and Tide medals;
`affinities-a-v1.png` adds Beacon medal and Gleam/Growth/Craft symbols;
`affinities-b-v1.png` adds Rhythm/Tide symbols plus fossil/rubbing item icons.
Together with the existing Bridge/Bloom/Accord artwork, all eight medals and
five affinity symbols are represented. `landmarks-v1.png` adds chime tower,
tide refuge and matching closed/open habitat caches. Empty cache artwork leaves
runtime quantities and grants authoritative.

Each sheet is unchanged 1254×1254 RGBA with four inspected 627px cells. Source
inspection confirmed transparent alpha (0–255) and no cross-frame clipping.
`docs/screenshots/campaign/final-static-phone.png` shows the exact frame draws in
a 390px browser viewport: icons at32/48px and props80px, on a checkerboard for
alpha inspection. Shapes remain readable at those sizes. This is a technical
source-frame review, not actual Phaser scene acceptance or human art approval.
Frontend registration and meaningful scene integration must be verified separately.

Static source sheets never establish animation coverage. Separate native motion
atlases and their actual renderer evidence establish the current minimal coverage;
a procedural bob or mirrored pose alone does not satisfy it.

## Encounter board and feedback

The trial board uses existing semantic HTML/CSS/SVG controls for progress,
pressure, turns, phases, hazard preview, actor selection and moves. It does not
need a baked bitmap board that would hide changing values from accessibility
controls. `effects-v1.png` provides six original bitmap feedback stills: Gleam,
Growth, Craft, Rhythm, Tide signatures and celebration. This sheet is 1536×1024,
three columns by two rows of512px cells; other sheets'1254px assumptions do not
apply. Broad glows are transparent overlays, verified source alpha0–254.

Frontend's agreed integration is visible matching-affinity art plus explanatory
text after a confirmed signature, and celebration art after a confirmed cleared
encounter. Reduced motion retains a static image. These stills are concrete
feedback assets, not a native effect animation sequence or reward authority.
Actual authoritative trial previews are preserved in `trial-signature-desktop.png`,
`trial-signature-phone.png` and `trial-signature-reduced.png` under the campaign
screenshots directory: Pip’s confirmed Gleam effect has a saved-light explanation
and remains static with reduced motion. `trial-effect-cleared.png` shows the next
confirmed tool turn clears it. `trial-celebration-desktop.png` and `-phone.png` show
completed-trial feedback. All are visually inspected with readable cutouts/text.
The other four affinity frames have source inspection and implemented selection,
but no individual actual-turn screenshot in this record.

Native motion atlases are registered deliberately after their producer freezes the
raw/public files and per-frame metadata:

```sh
python3 content/campaign/art/register_motion.py content/campaign/art/ASSET.provenance.json
python3 content/campaign/art/validate.py
```

Registration verifies raw/public equality, the declared hash/dimensions and every
frame/foot-anchor bound. It retains pending review and does not substitute for
actual renderer inspection. Incoming sets are separate animation entries, so
static source-sheet poses never acquire invented movement coverage.

## Native motion delivery

`motion-registry.v1.json` now contains24 separate original motion atlases: twelve
base companions and twelve Wayfinder forms. Each has20 native articulated poses:
four directional idle stills, two walking strides for each heading, three
signature poses, three cheer poses and two seated rest poses. Raw generation and
imagegen alpha-cleanup sources, exact prompts, hashes, measured irregular rows
and foot anchors are retained per atlas. No raster pixel processing substitutes
for artwork generation. These are minimal two-stride cycles, not polished
four/eight-frame locomotion or breathing/blinking loops.

The nine nonstarter base sets have actual `createWorld` renderer assertions and
visual inspection across desktop and all four phone groups; all180 poses were
checked. `nonstarter-base-technical-preview.v1.json` records per-character source
and frame hashes, reproduction commands, assertions and five representative
screenshots. No clipping, detached opaque haze, species/costume identity or foot
anchor defect was observed in this technical review. The world contains ambient
butterflies; those are separate decorations. Starter and Wayfinder review is recorded in individual provenance and
`native-motion-content-review.v1.json`. All24 sets passed actual native frame/
anchor, timed signature and reduced-motion selection assertions. All480 native
poses have desktop and all-four-phone-group technical visual inspection across
the team. `starter-motion-phone-supplement-review.v1.json` completes the five
starter sets beyond the original domain-agent phone subsets;
`nonstarter-wayfinder-supplement-review.v1.json` records the four shared reviews.
No final human art approval is claimed.
