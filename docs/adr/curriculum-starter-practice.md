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

Story activity completion now advances the practice journey immediately. A correct
answer or explicit together-completion records a bounded `completedSkills` set;
completed activities are excluded from selection rather than repeated until a
multi-session readiness threshold is reached. This completion record is separate
from academic readiness and mastery. Completion can unlock prerequisite-related
practice for every skill. Original qualifying correct observations can establish
legacy activity completion; original checkpoints and evidence are not rewritten.
Viewing a worked example still records starter exposure but does not itself finish
an activity. Hints and abandonment do not complete an activity. Once all activities
are complete, the story continues through its existing non-academic routes.

Incorrect answers freeze a different canonical question of the same skill in the
same transaction, using a persisted seed and the existing encounter/story binding.
The encounter identity and cumulative checks/support are retained, preventing a
corrected answer from becoming an independent first check. Authored alternate
items cover previously single-example card tasks. Receipts replay the exact new
question; reload does not choose a further variant. A default-omitted internal
`activityCompleted` observation flag lets completion be replayed without changing
historical event projections. A default-omitted `modernTasks` active-task flag
preserves old frozen factories while new tasks use draft-3 props and variants.
Confirmed completion closes the panel automatically. New panels use the activity
title and illustrated props instead of repeating the world objective and generic
discovery-marker copy. The optional public `activityCompleted` skill flag drives
journal completion labels without inventing academic mastery.

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
