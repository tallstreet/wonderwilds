# Draft Foundation curriculum pack (WW-03)

This pack authors the implementation plan's 33 original seed nodes: M01–M18 and E01–E15. It is not a complete Foundation curriculum, a validated mastery instrument, or a released content pack. Every node and alignment reference is pending educator review. Nothing here activates additional maps or changes the existing guest save.

`seed-v1.json` uses the frozen `SkillNode@1` contract in `contracts/schemas/v1.json`. Nodes have explicit prerequisites, permitted representations, draft parameter ranges, a three-step hint ladder, misconception codes and an evidence policy. M16 is a Year 1 extension; M17, M18 and E15 are extension candidates. The original objectives and edges follow the project plan; contexts, hints, draft bounds and review tooling are original Wonderwilds authoring.

`authoring-v1.json` supplies two distinct contexts and three ordered parameter/representation/context variants for every node. These are authoring specifications, not calibrated encounter items. A numeric upper bound is not sufficient to validate an item: WW-06 must also enforce constraints such as divisible sharing totals, taught graphemes and available language alternatives. Existing rungs have supported routes. Story access must never depend on independently demonstrating academic mastery.

`evidence-policies-v1.json` records the initial section 5 hypotheses. Server-issued evidence, unique variants, multiple sessions and novel transfer remain necessary. Thresholds require calibration. Read-aloud of assessment text is recorded as support for reading; listening comprehension remains distinct. E13/E14/E15 require adult review and cannot be certified by tile selections. No microphone is required. Motor access, dialect and multilingual experience must not be treated as low ability.

`coverage.csv` separates IXL breadth comparison from Australian Curriculum Version 9 alignment. It accounts for all 33 seed nodes and discloses two unmapped scope areas; it is not an exhaustive source audit. Descriptor codes, reviewer identities and review dates stay null until actually verified. `reviews-v1.json` records separate prerequisite, alignment, item and accessibility gates. No approvals are invented.

`manifest-v1.json` follows `ContentManifest@1` and hashes `integrity-index-v1.json`, whose SHA-256 entries identify the exact content files. The index explicitly disables release. These hashes detect drift; they are not signatures or educator approvals. A later release pipeline must check signatures, verified reviews, representative assessed fixtures and actual activation gates.

Run from the repository root:

```sh
python3 content/curriculum/validate.py
python3 -m unittest discover -s content/curriculum -p 'test_*.py' -v
# Requires locked dependencies installed with: (cd contracts && npm ci)
node content/curriculum/check-schema.mjs
```

Validation checks the seed inventory, duplicate/dangling IDs, cycles, parameter bounds, context and representation references, support availability, coverage promotion, review gates, evidence policies and content hashes. Fourteen tests include deliberate invalid/cyclic/out-of-range/unreviewed/tampered inputs. The schema check accepts all 33 nodes and the manifest using the shared contract and rejects two malformed nodes. These checks do not prove educational fit, accessibility or learning gains.

Changes require reviewing affected nodes, retaining versioned copies for any released pack, updating file hashes in the integrity index, then updating the manifest hash of that index. Do not change a hash to conceal an unexpected modification. Rollback is selecting the previous immutable pack version; this draft has no active users or migration side effects.

Outstanding review: a qualified educator must verify exact ACARA descriptors, prerequisite rationale, difficulty calibration, item variants, meaningful representation alternatives and released scope. The assessment owner must build and validate actual tasks. Full source coverage, language/accessibility review and transfer calibration remain future gates.
