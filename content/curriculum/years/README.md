# Years 1–6 Adaptive Learning Curriculum Graph (NSW Curriculum Alignment)

This directory contains the expanded Years 1–6 adaptive curriculum pack covering both **Mathematics** and **English**, mapped to the New South Wales (NSW) Australian Curriculum Version 9 framework and structured after the IXL NSW Stage 1 (Years 1–2), Stage 2 (Years 3–4), and Stage 3 (Years 5–6) skill progressions.

## Architecture & Graph Structure

The curriculum is represented as a Directed Acyclic Graph (DAG) with explicit prerequisite edges:
- **Maths Strands**:
  1. *Number* (Counting, Place Value, Add/Subtract, Multiply/Divide, Fractions, Decimals, Percentages, Integers/Ratios)
  2. *Algebra* (Patterns, Number sentences, Cartesian plane, Algebraic expressions & equations)
  3. *Measurement* (Length, Mass, Capacity/Volume, Perimeter/Area, Time, Duration)
  4. *Space* (2D Shapes, 3D Objects, Symmetry, Angles, Grid coordinates & Bearings)
  5. *Statistics* (Tally charts, Picture & Bar graphs, Line graphs, Central tendencies - Mean/Median/Mode)
  6. *Probability* (Chance language, Scales 0-1, Fraction probabilities, Mutually exclusive & Complementary events)

- **English Strands**:
  1. *Literacy & Phonics* (Grapheme-phoneme correspondence, Decoding, Syllabification, Sight words, Spelling rules, Morphology - Prefixes/Suffixes/Roots)
  2. *Literature & Reading Comprehension* (Decodable texts, Main idea, Inferencing, Author's purpose, Literary analysis, Theme, Perspectives, Critical literacy)
  3. *Language & Grammar* (Parts of speech, Sentence punctuation, Complex sentences, Voice & Modality, Register & Etymology)
  4. *Writing & Communication* (Sentence building, Recounts, Informational reports, Persuasive writing, Extended multi-paragraph text craft, Oral presentations & Collaborative inquiry)

## Adaptive Progression Model

Learners enter the curriculum at entry point nodes (skills with no unmet prerequisites). Once a learner demonstrates mastery (as governed by server evidence policies, multi-session verification, and support tracking), the node status transitions to mastered, automatically unlocking adjacent downstream successor nodes in the graph.

Prerequisite chains connect seamlessly from Foundation nodes (M01–M18 and E01–E15) into Stage 1, Stage 2, and Stage 3:
- **Maths**: 70 total nodes (18 Foundation + 52 Years 1–6) across 15 learning waves.
- **English**: 50 total nodes (15 Foundation + 35 Years 1–6) across 15 learning waves.

## Files
- `seed-years1to6-v1.json`: SkillNode definitions conforming to the versioned schema, parameter bounds, misconceptions, and 3-step hint ladders.
- `authoring-years1to6-v1.json`: Wonderwilds contextualized encounter settings and difficulty ladders (with supported learning options).
- `reviews-years1to6-v1.json`: Review audit tracking records (pedagogical, alignment, item, and accessibility review gates).
- `validate.py`: Validation harness checking integrity, acyclicity, references, and parameter bounds.
- `graph.py`: Topological sorting and adaptive progression wave simulator.

## Running Verification

From the repository root:
```sh
python3 content/curriculum/years/validate.py
python3 content/curriculum/years/graph.py
```
