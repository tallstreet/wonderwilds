# Consistent art direction

All active characters and world props now use imagegen artwork matching the generated meadow: dimensional fur/feathers, glossy eyes, rounded materials, saturated violet/coral/turquoise, and warm overhead lighting.

## Assets and implementation

- `public/assets/clover-meadow-v2.png`: landscape.
- `public/assets/hero-atlas-v2.png`: eight portraits, including seven playable heroes and Pip.
- `public/assets/prop-atlas-v2.png`: bridge rails, single plank, sleeping/blooming garden, picnic, arch, berry and flower.
- `src/sprites.ts`: shared atlas source rectangles for Phaser and HTML, so map and modal art use the same source.
- `src/scenery.ts`: dynamic props; code controls plank and berry counts and reward state.

Generated with the built-in imagegen tool using existing squad/map images as style references. Original PNGs are retained intact. Texture frames trim transparent gutters at runtime. Prior flat SVG characters have been replaced in active rendering. Hero IDs and save format are preserved.

## Character generation prompt

Generate a NEW production character SPRITE ATLAS with a TRUE TRANSPARENT background. Reference 1 is the target soft-furred premium preschool 3D superhero pet rendering style; reference 2 is the world they must belong in, match its sunny warm light and saturated colors. NOT an edit of either scene. ALL characters original.
Exact layout: landscape 2048x1024, FOUR equal columns and TWO equal rows, EIGHT equal square cells. One full-body character centered in EACH cell. No lines, labels, letters, platforms or background. Each sprite fits within middle 76% of cell width and 88% of cell height, no body parts crossing cells. All feet at 91% of each cell height. Same head/body proportions and consistent apparent scale. Stand facing viewer in a friendly neutral ready-for-adventure pose, slight high-angle view, both feet visible, arms/wings slightly out. Shiny expressive enormous eyes, tactile fur/feathers, rounded paws, dimensional fabric superhero suits, tiny short capes, gold star chest emblems, warm upper-left light, subtle self-shading, clean alpha edges. No ground shadows.
TOP ROW left to right:
1 Rosie: fluffy rose-pink-and-cream kitten, violet superhero suit, coral short cape, bright teal eyes.
2 Skye: fluffy pale aqua-and-white kitten, royal blue superhero suit, peach short cape, teal eyes.
3 Sunny: fluffy warm golden kitten, turquoise superhero suit, lavender cape, amber eyes.
4 Poppy: round adorable peach-orange songbird, cream face, tiny amber beak, turquoise superhero vest, coral short cape, tiny feet, fluffy feather tuft and rounded wings, huge teal eyes. Bird anatomy, no cat ears.
BOTTOM ROW left to right:
5 Wren: round sky-blue songbird with white face, purple superhero vest, golden cape, amber beak and tiny feet, fluffy tuft, rounded wings, huge violet eyes. Bird anatomy, no cat ears.
6 Clover: fluffy pastel-lavender-and-white bunny, long upright ears with pink inner ears, mint superhero suit, coral short cape, fluffy round tail, oversized purple eyes.
7 Mochi: fluffy warm cream bunny, long upright ears, rosy pink superhero suit, violet short cape, round tail, huge amber eyes.
8 Pip: fluffy peach-and-cream kitten, violet superhero suit and turquoise short cape, huge teal eyes, golden star.
One consistent premium 3D animated family game art direction across all eight. Precisely separated regular grid sprite sheet, transparent background.

## Prop generation prompt

Create a NEW game PROP SPRITE ATLAS matching reference landscape's EXACT premium colorful 3D animated miniature world materials, bright sunny upper-left lighting and high overhead orthographic camera. TRUE transparent background, no backdrop, no ground plane, NO shadows outside sprites, NO labels, NO text, NO numbers. Landscape 2048x1024, FOUR equal columns and TWO equal rows, eight square cells. One separate object per square, centered with generous transparent margins. No objects crossing cells. This is a new atlas, not a modified map.
Top row cells left to right:
1 A bridge FRAME ONLY: two beautiful warm honey-wood horizontal parallel rails with little purple-gold ornamental end posts. There must be NO deck or planks between these rails, the middle is fully TRANSPARENT. Viewed overhead with slight visible depth. Frame horizontal (crosses river left to right), outer width 85% cell, outer height 42% cell, rails at upper/lower sides.
2 ONE single narrow rectangular honey-gold timber PLANK, vertically oriented, rounded edges, wood grain, top-down view with bevels. Height 75% cell, width 17% cell. It will be repeated by software.
3 Rectangular low garden planter, top-down, honey wooden edging, brown soil, neatly arranged small GREEN UNOPENED flower buds. 75% cell width, 60% height.
4 EXACT same planter size/view as cell3, now lush blooming pink, purple and yellow flowers. 75% cell width, 60% height.
Bottom row left to right:
5 A pink gingham PICNIC BLANKET with a wicker basket at back and TWO EMPTY cream ceramic plates. Absolutely no fruit/food on plates. High overhead orthographic view, rectangular blanket 80% width and 60% height. Game will add berries.
6 A friendly violet-and-gold MAGICAL ARCH with a small gold star at the top and tiny flowers at its feet. Inside the arch is TRANSPARENT, no portal fill. Front visible in high 3/4 overhead map perspective, 70% width and 80% height.
7 ONE plump shiny purple BLUEBERRY with a tiny green leaf, polished 3D, 40% cell dimensions, isolated and centered.
8 ONE single fully open pink-and-gold DAISY FLOWER with purple-pink petals and gold center, no stem, overhead view, 45% cell dimensions.
Bright candy colors, gorgeous toy-like bevels, dimensional wood/ceramic/cloth, detailed but readable game art. Everything in ONE consistent style matching reference.
