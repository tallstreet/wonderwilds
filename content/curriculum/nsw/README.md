# NSW K–6 replacement curriculum

This is the active mission-learning graph: **736 skills and 8,125 practice items**,
with at least ten distinct item fingerprints per skill. `source-v1.json` preserves
the supplied authoring document, including templates, prerequisite edges, resource
mappings, visual suggestions and scoring specifications. Its text is content data,
not application instructions. No educator approvals are inferred from the import.

`graph-v1.json` and `exercises-v1.json` are deterministic private compilations.
The browser receives only the issued prompt, permitted stimuli and answer controls.
Answers, scoring rules, audio scripts, producer labels and explanations remain
server-private; explanations are released through the existing feedback/support flow.
All release flags remain disabled and reviews pending. Original curriculum packs
and factories remain diagnostic fixtures; active selection uses this replacement.
No legacy-to-new skill equivalence or evidence migration is asserted.

## Progression

Practice stays on a skill across mission activities. Ten distinct first-check
items, with at least eight correct independently, establish **provisional practice
readiness**, not secure mastery. Ten distinct finished items, including explicit
supported completion, can also open related practice; this is labelled exploration
and never independent evidence. A single finished story activity no longer finishes
a skill. Existing non-academic story routes remain available.

Two unsuccessful first checks schedule direct prerequisite revision on subsequent
activity selection. Each prerequisite receives a revision activity before returning
to the struggling skill. Earned progress is retained; no backwards edges are added
to the DAG. Root skills continue with new examples and support. The policy stores
at most ten independent checks and ten explored fingerprints per skill. It remains
a draft product policy needing educator calibration; it does not claim the source's
complete assessment-policy specification, taught-grapheme tracking, spaced review,
all-facet coverage or secure mastery.

## Delivery coverage and limitations

The initial adapter automatically marks **6,439 items**. Another **1,686 items** use
an explicit supported workshop because they require reviewed recordings, authentic
performance observation, or a structured renderer/geometry interaction not yet
implemented. `delivery-report-v1.json` enumerates every affected item and reason.
Every supplied exercise is retained; ten stored items does **not** mean ten automatic
assessments are available for every skill. No placeholder is automatically correct.
Missing-media workshops are exploration prompts, not faithful on-screen versions
of those source tasks. Their full authoring specifications remain in the source.

Supplied inline SVGs are compiled to content-addressed local images after an element
and attribute allowlist rejects scripts, external references and active markup.
Implemented structured diagrams include object sets, polygons/circles, clocks,
ten frames, arrays, hundred grids and fraction strips. Brief-exposure stimuli
hide after the authored interval; response time is not scored. Font Awesome Free 7.3.1
supplies verified optional context imagery; unavailable suggestions are omitted.
Other suggestions remain metadata. See the bundled Font Awesome attribution/licence
in `frontend/public/assets/learning/nsw/FONT-AWESOME-LICENSE.txt`. Icons do not replace
exact measurement or geometry diagrams. Neutral image descriptions and the supported
route do not establish equivalent independent screen-reader assessment.

Skill progress exposes all 1,418 suggested resource links through expandable
“Learning resources” sections, distinguishing learner resources from adult guidance.
These are source-provided topic/course links, not newly verified skill-specific
lessons. Opening them is explicit and does not send profile or answer data.

English transcription is disabled for the replacement skill IDs. Missing audio is
never substituted with a visible transcript for independent listening assessment.
Text marking honours declared case, whitespace, Unicode and apostrophe normalization;
numeric marking uses exact rational comparison and explicit decimal-place constraints.

## Rebuild and verify

```sh
node scripts/export-nsw-icons.mjs
python3 scripts/import-nsw-curriculum.py
node scripts/export-nsw-icons.mjs --check
python3 scripts/import-nsw-curriculum.py --check
python3 -m unittest discover -s content/curriculum/nsw -p 'test_*.py'
cargo test -p wonderwilds-learning --locked
```

After starting Vite, run `node qa/e2e/nsw-curriculum.mjs` with
`WW_ACTIVITY_TEST_URL` and `CHROMIUM_PATH` as needed. The default URL is port 5189.
Persistence verification uses `sh scripts/check-postgres.sh` and its disposable
synthetic cluster. Migration 0007 expands the bounded snapshot/receipt and checkpoint
limits for this larger graph. Restart the local demo to apply migrations and load
the new backend. No demo reset or production deployment is part of this import.
