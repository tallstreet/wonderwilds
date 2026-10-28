# Starwhisker visual refresh

Original superhero-kitten adventure: violet, coral, turquoise and warm gold; expressive big-eyed characters; toy-like buttons; a lush, readable meadow. The generated squad is original promotional character art, while editable SVG characters are used in Phaser and activity panels.

## Assets

- `public/assets/hero-squad.png`: generated using the built-in image generation tool, with transparency requested. Copied into this project so the app has no generated-assets directory dependency.
- `src/art.ts`: original scalable in-game kitten art, with colour-specific gradient IDs.
- `src/redesign.css`: responsive visual layer over the existing UI.
- `src/world.ts`: shaded trees, hideout details, garden fence, butterflies and sparkles. Decorative animations respect reduced-motion preferences.

## Generation prompt

Use case: stylized-concept. Asset: polished character illustration for a children's superhero kitten maths adventure game, transparent background. Create THREE entirely ORIGINAL cute superhero kitten characters together in a dynamic friendly team pose, full bodies fully visible with generous clean margin, wide landscape composition. Centre leader: fluffy peach-and-cream kitten, enormous expressive teal eyes, violet fitted superhero suit with bright coral gloves/boots, tiny golden four-point star chest badge, coral short cape, happy confident smile and one paw lifted. Left: small lavender-and-white kitten, mint suit and purple goggles resting on forehead, sitting with curious happy expression. Right: fluffy caramel kitten with cream muzzle, turquoise suit and indigo short cape, excited wave. Premium preschool 3D animated-series quality, beautifully groomed soft fur, oversized heads and paws, dimensional glossy eyes with rich iris details, rounded silhouettes, appealing asymmetry, tactile fabric, warm cinematic rim lighting, saturated candy colors. Similar upbeat superhero-kitten genre appeal to SuperKitties, but DO NOT reproduce any existing characters, costumes, insignia or branding. No text, no letters, no background scenery, no pedestal, no ground plane, true transparent background. Not flat vector, not pixel art. All three faces large and readable.

## Verification

Existing curriculum, deterministic checks, evidence, local saves, and collision geometry were preserved. Automated tests and production build pass. Desktop and phone browser visual checks accompany this refresh; child playtesting remains future work.

## Exploration library imagery

The 72 exploration destinations use individual WebP landscapes in
`public/assets/places/explore-{biome}-{variant}.webp`. The three original island
maps retain their existing artwork. New artwork uses the original Clover Meadow
landscape as a visual reference: overhead views, saturated foliage, rounded
storybook landmarks, soft dimensional shading, and clear central walking space.
Each biome was generated as six coordinated scenes, then separated into assets.

`src/mapImages.ts` is the shared image URL mapping for the Explore cards and
Phaser backgrounds. Cards use native lazy loading; the world loads artwork only
when a destination is visited. A failed image request leaves the procedural
landscape playable and retries on a subsequent visit. Late image responses must
not replace the current destination. Exploration exits use the central path;
unmatched procedural circular blockers have been removed from these maps.
