# Connected map artwork

Generated with the built-in imagegen tool, using `frontend/public/assets/clover-meadow-v2.png` as the style and camera reference. Originals remain in the generated-images directory. Game assets:

- `frontend/public/assets/peachy-orchard-v2.png`
- `frontend/public/assets/starlight-cove-v2.png`

Both are rendered full bleed at 1120 × 690 world units, exactly like Clover Meadow. Path centres are measured from the resulting images in `frontend/src/maps.ts`; the same definitions drive click targets, transitions and opposite-path arrival positions.

## Orchard prompt

Create a new Peachy Orchard game map using the reference Clover Meadow as strict art style and camera reference: same richly detailed soft shaded 3D toy-like fantasy landscape, vivid lime grass, lush flowers, pink purple foliage, overhead oblique view, seamless full bleed 1598x984 landscape composition. This is an adjacent playable map, no characters, no text, no UI, no border. Broad uncluttered grassy center, peach trees and blossoming fruit trees concentrated at top and bottom outer margins; tiny charming orchard cottage in upper left only. No river or water, no obstacles in central walking lawn. Critical connecting path geometry: a golden sandy footpath enters LEFT edge at exactly 50% image height and exits RIGHT edge at exactly 69% image height, width 5% image height. Curve gently through open central lawn, with a short branch toward the cottage. Leave both edge openings clear of all foliage. Match the reference rendering detail, scale, lighting, colors and texture closely; not flat vector art.

## Cove prompt

Generate Starlight Cove, an adjacent game map to the reference Clover Meadow. Strictly match its richly detailed bright soft shaded 3D fantasy toy landscape, overhead oblique camera, scale, grass textures, pink purple blossoming trees and lush flowers, full-bleed 1598x984 landscape. No characters, text, UI or border. A coastal grassy clearing with sandy golden path crossing from LEFT edge at 50% image height to RIGHT edge at 50% image height, 5% image-height path width. Both path openings must reach the edges with no foliage blocking them. Open unobstructed lawn across middle and lower-middle, no central obstacles. Turquoise sparkling sea only at the very top 8% strip, ornamental shells and starfish on shore above the playable lawn. Dense flowers and shrubs around bottom border and corners, tiny pearl-roof seaside cottage in upper LEFT at same footprint as reference clubhouse. Keep daytime warm lighting and same level of rendering polish as reference; subtle star motifs supply Starlight theme, not a dark night scene. No river through center. The path gently meanders but meets both outer edges horizontally exactly halfway down.
