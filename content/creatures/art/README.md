# Character artwork library

All twelve original friends now have generated full-body transparent PNG artwork.
The three source sheets are retained unchanged under
`frontend/public/assets/characters/`. Regional metadata records the generation
prompt, reference asset, source and exact hash; `library.v1.json` records the
current inspected frames used for development presentation.

| Region | Row-major character order | Sheet |
| --- | --- | --- |
| Meadow | Pip, Tansy, Nib, Flutter | `meadow.png` |
| Orchard | Cleo, Bramble, Patter, Twizzle | `orchard.png` |
| Cove | Luna, Pebble, Glimmer, Sail | `cove.png` |

All sheets are 1254×1254 RGBA. Meadow and Orchard use 627×627 frames.
Cove uses a 660px left column and 594px right column, preserving Luna's cape
and Glimmer's wings. HTML and Phaser share the inspected boundary helpers;
the original PNG pixels are not edited. Metadata describing a requested equal
grid is generation provenance; the inspected inventory is the framing authority.

Start the frontend with `npm run dev` and open `/characters.html` to browse all
friends, filter regions and read their profiles. Both guest and server journals
link to the library. Library browsing does not recruit friends or change saves.
Server Meadow friendship NPCs and followers use their own artwork rather than
stand-in hero sprites. The seven selectable hero IDs remain intact.

These are front poses with existing procedural movement effects, not new
walk cycles, directional sets or separate glow transformation artwork. Generation
and automated checks do not constitute human content or rights review. The frozen
story pack and its pending release activation are unchanged; this development
art inventory does not silently sign or activate a new release pack.

Run `python3 content/creatures/art/validate.py` for file/hash/roster/frame validation,
`bash scripts/check.sh` for repository checks and
`node qa/e2e/character-library-smoke.mjs` for browser artwork/filter/keyboard checks
against a running frontend. Browser checks use Chromium, default port5174; set
`WONDERWILDS_TEST_URL` to change it.
