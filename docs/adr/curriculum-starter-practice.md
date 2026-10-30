# Authored starter practice across the curriculum

Status: development, 31 October 2026; educator and release gates remain pending.

All 120 graph nodes now have an activity factory. The original five maths factories
and their evidence/readiness policy stay unchanged. Another 115 factories provide
focused card exercises or guided workshops. Their short texts, sample sizes and
single representations do not cover the complete objectives or the longer-text
assessment ranges in the curriculum authoring packs. They must not be advertised
as full-skill assessments. See [coverage](../ACTIVITY_COVERAGE.md).

Rust selects tasks, constructs distractors, rotates card positions and assesses
card answers. Seeded number and word variations are deterministic. LLM story prose
cannot choose activities, answers, progression or rewards. Workshop responses are
not automatically assessed: writing, oral fluency, phoneme production, research
and collaboration need human observation. Speech-to-text is neither decoding nor
pronunciation assessment. Adult review is an instruction to review the work, not
an invented approval record or a server-certified review workflow.

The original five-skill readiness path is selected first. Thereafter, new starters
follow prerequisite exposure: a correct practice answer or viewing a committed
worked example records that starter as explored. Hints and abandonment alone do
not record exploration. Its successors may then be offered as practice; this does
not mark the prerequisite ready, mastered or educator-approved. Original skills
still require their original readiness policy. Previously recorded original
correct-answer observations can establish exposure for a new starter's prerequisite.
Within the available new starters, unexplored skills and earlier year bands take
priority. When all reachable starters are explored, selection offers review.

The optional `exploredSkills` checkpoint set is bounded by the finite graph. It is
omitted while empty, so historical five-skill checkpoints and event replay retain
their original projection. New starter events remain in the immutable journal but
do not occupy the original <=60 qualifying-observation window or manufacture
independent full-skill readiness. No applied migration or old content factory is
changed. Exact receipts, transaction atomicity, RLS, guest separation and inline
story bindings continue to use the existing repository path. Older binaries may
not read new practice-card activities or nonempty exploration sets; rollback must
retain a compatible reader and must not reset progress.

The additive `practice_cards` wire task exposes only stimulus, card IDs/labels and
an adult-review flag. Its schema requires at least two choices for automatic
practice and no choices for workshops. Answers keep the existing choice command;
review-only tasks reject answer commands. Server-issued hints/examples reveal
teaching material only after support commits. Together-completion remains
supported/not-assessed and atomically executes its bound story action.

Pointer dragging, tapping and keyboard buttons produce the same answer. Matching
pieces cannot occupy two places; moving or clearing a piece is reversible.
Check is explicit. Transcription fills editable input and requires Use this answer
before Check. Unknown or ambiguous transcripts preserve the current answer.
Phonics, spelling and decoding do not use transcription as a skill assessment.
Workshop dictation is an optional local draft for human review, never a correctness
claim; it is discarded on closing. The existing authenticated local Muse/Baseten
bridge is reused with its provider key kept server-side. Cancellation, closing and
destruction stop recording and discard late transcripts. There is no TTS adapter.

Remaining work includes richer manipulatives, meaningful multi-representation
variants, full-length and transfer tasks, educator calibration, child playtesting,
reviewed audio, assistive-technology testing and production child-data approval.
All original pending review and disabled release gates remain unchanged.
