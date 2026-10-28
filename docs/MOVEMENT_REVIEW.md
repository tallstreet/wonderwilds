# Grounded character movement review

## Requested work

1. Species-specific movement: kitten steps, bunny hops, bird waddles.
2. Ground contact: foot anchors, responsive shadows, movement tied to actual travel.
3. Directional facing and stable idle poses.

## Review criteria

- A walk cycle includes visible changes in foot position, not only translation/rotation of a still sprite.
- Gait advances with actual distance; walking into a collider does not run in place.
- Sprite feet and shadows share a stable logical ground point.
- Camera tracks ground position, not vertical hop offset.
- Bunny lands and character returns to idle when stopped or opening a modal.
- Front, rear, left and right facing track movement; stopped heroes retain their last heading.
- Companion follows a traversable trail instead of cutting across water.
- Existing hero selection, saves, reset and activity approaches continue to work.
- Reduced-motion preference suppresses optional bounce/sway.
- All imagery is stored in the repo with source prompts documented.

## Implementation and verification

Three subagents implemented locomotion, ground contact, and directional poses. Integration was reviewed for measured foot origins, matching pose sizes, actual-distance gait, and reachable NPC approaches.

- `npm test`: 24/24 passing, including collision, follower trail, direction persistence, geometry, gait, save and learning evidence checks.
- `npm run build`: passed; existing large Phaser bundle warning remains.
- Browser smoke check: Rosie moves with keyboard and changes to side/rear poses; Clover switches correctly and returns to grounded side-facing idle. Companion follows behind. No browser console errors in the fresh preview tab.
- Preview: `movement-preview.png`.

## Current limits

Rosie has a six-frame forward/down walk with visible alternating feet. Other heroes and directions currently combine static directional artwork with procedural kitten weight shifts, bunny hops, or bird waddles. Full limb cycles for every hero/direction remain future animation work. Automated tests verify gait and collision behavior; the browser smoke check is not exhaustive animation QA.

Generated asset prompts and geometry notes are in `frontend/docs/LOCOMOTION.md` and `frontend/docs/DIRECTIONAL_POSES.md`.

## Bridge collision alignment follow-up

The actor container already represents planted feet. The mismatch came from a
separately hard-coded bridge corridor (`342..374`) that reached into the rails,
a plank strip that stopped 13 world pixels before the far river boundary, and
both rail halves rendering underneath characters.

`frontend/src/mapGeometry.js` now shares the bridge placement between scenery
and navigation. The clear deck measured from the trimmed bridge art is world
`y=346..372`; a small ground footprint (24 by 8) keeps the planted-foot midpoint
within `350..368`. Visual height, hop offset, cape and tail do not affect collision.
Planks cover the full bridge span, and cropped rear/front rails render behind/in
front of crossing characters respectively. Original image files are unchanged.

27 tests pass and the production build passes. New regressions cover crossings
in both directions, stopping before each rail, continuous tap routes across both
banks, and using feet even when the torso or animated artwork lies above the deck.
The production build retains the existing Phaser bundle-size warning. Parent browser
QA confirmed Clover stops before the near rail when walking down on the deck,
crosses to the east bank, and returns to the west bank with the companion following.
The split rail artwork rendered correctly during these checks.
