# Grounded character locomotion

`src/locomotion.ts` separates distance-driven gait from world position. Collision,
sorting, and shadows use the logical ground point. Only child artwork lifts,
scales, or rotates. Phases advance from actual travelled distance; held input at
an obstacle must not animate. Idle, paused activities, and reduced motion reset
to the neutral pose.

## Integration

1. Call `preloadWalkCycle(scene)` during preload and `registerWalkCycle(scene)` during create.
2. Advance `phase = advanceGaitPhase(phase, actualDistance, species)`.
3. Apply the directional base frame, then call
   `applyWalkCycle(art, heroId, phase, moving, { reducedMotion, facing })`.
4. If true, set origin using `walkFrameOrigin(phase)`. These measured anchors align
   the head and planted sole across generated cells. Reset the standard origin
   when returning to a directional or idle frame.
5. Use `sampleGait` for species-specific transforms. Rotation is degrees,
   scales are multipliers, and lift is world pixels. Avoid a second large bob
   on the actual walk frames.

## Art and coverage

`public/assets/rosie-walk-v1.png` is a 1536 × 1024 transparent RGBA image with six
512 × 512 frames (three columns, two rows), generated using the built-in imagegen
tool and existing `hero-atlas-v2.png` as identity/style reference. Colored RGB
values in fully transparent pixels are invisible in Phaser. Alpha and each
frame's planted-foot alignment were inspected. The visible height matches the
original atlas within one percent, so the same display size is appropriate.

Actual alternating-foot frames currently cover **Rosie facing down**. They must
never replace left/right/rear poses. Other heroes/directions use procedural
kitten steps, bird waddles, or bunny crouch/hop/landing motion as a fallback.
Full directional walk sheets for those heroes remain a further art pass.

## Generation prompt

> Use case: stylized-concept. Asset: six-frame WALK CYCLE sprite sheet for the original Rosie kitten (TOP LEFT character of the reference image). Create a NEW sheet using that character as an identity/style reference. Exact same rose-pink and cream fur, purple fitted suit, coral boots/gloves/cape, gold star chest badge, big teal eyes, fluffy tail and glossy premium preschool 3D animated rendering.
> Landscape sheet 1536x1024, THREE equal columns and TWO equal rows. Exactly SIX full-body frames, chronological reading order left-to-right, top row then bottom row. All are FRONT FACING walking towards camera, camera fixed slightly overhead, NO turning, NO other characters, NO text, NO panels, NO ground, NO shadows. True transparent alpha background.
> Keep the head size, body proportions, outfit, face, lighting, center alignment and total sprite scale identical in every frame. Each cell is 512x512. Entire body within center 76% cell width and 85% height. Lowest planted foot always at 90% cell height, horizontal center 50%. Keep both ears inside frame, generous gutters.
> Actually animate alternating leg stepping and opposing arm swings, NOT six duplicate standing poses. Frame1: LEFT foot forward/planted, RIGHT foot back, right hand forward. Frame2: body settles over LEFT planted foot, RIGHT knee bends and foot lifts. Frame3: RIGHT foot passes forwards under hips, left hand starts forward, planted left sole still at baseline. Frame4: RIGHT foot forward/planted, LEFT foot back, left hand forward. Frame5: body settles over RIGHT foot, LEFT knee lifts. Frame6: LEFT foot passes forwards under hips returning to frame1. Gentle cape/tail follow-through. Feet clearly separated and alternating, low natural walking stride rather than running or jumping. Head barely bobs; consistent centered camera and feet baseline. Clean game-ready atlas, warm light upper left, identical matching colors/materials.

## Verification

`cd frontend && node --import tsx --test src/locomotion.test.ts` checks idle/reduced-motion stability,
distance-based phase progression, bounded continuous species gaits and nonnegative
bunny lift, and six-frame selection with direction/hero/idle guards.
