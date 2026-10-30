# Mission activity controls

`activity.ts` mounts the server-issued mission task. It never decides correctness,
readiness, story effects or rewards. Every answer and help request uses the existing
revisioned command session; uncertain writes retry the exact prepared body.

Matching uses objects in a piece bank and recipient places. Drag an object, or
select it with a tap/Space and activate a place with a tap/Enter. A second placement
moves it, and activating a filled place without a selected piece clears it.
Counting objects can be dragged or tapped between uncounted and counted trays.
Answer cards can be dragged to a tray or selected through native radios. None of
these gestures submits. Controls freeze during saves, recording and pending writes.

`practice_cards` renders a bounded authored stimulus and cards. Guided workshops
instead provide a local writing/dictation pad and explicit human-review guidance.
They have no automatic Check button. The pad is not saved or submitted, and the
screen says that closing discards it. Supported inline completion uses the existing
Let's do it together command; it is never an assessed writing/oral success.

`voice.ts` reuses the existing opt-in `AnswerRecorder` and Muse/Baseten bridge.
Transcripts are editable and require Use this answer, then a separate Check.
Exact unique card labels and explicit spoken number/fraction/decimal/symbol forms
only select a card; Rust assesses it.
Ambiguous, cancelled and failed transcripts preserve the previous answer. There
is no speech scoring or TTS. The host supplies a recorder only when bootstrap
reports voice input and the browser supports recording. Closing a dialog, leaving
the page or destroying the component cancels the microphone and late responses.
Phonics, spelling and decoding retain their existing non-STT or human-review route.

Run with Vite on the specified origin:

```sh
CHROMIUM_PATH=/path/to/chromium WW_ACTIVITY_TEST_URL=http://127.0.0.1:5173 \
  node frontend/src/mission-learning/activity.browser.mjs
```

This uses real mouse/touch/keyboard events and a mocked recorder, never a paid
provider request. The integrated disposable-database story test is:

```sh
CHROMIUM_PATH=/path/to/chromium WW_QA_BROWSER_TEST=qa/e2e/mission-learning.mjs \
  node qa/e2e/run-training-qa.mjs
```

See [curriculum starter decisions](../../../docs/adr/curriculum-starter-practice.md)
and [activity inventory](../../../docs/ACTIVITY_COVERAGE.md) for the distinction
between starter coverage, observed exploration and full-skill assessment.

## Completion and wrong-answer flow

Confirmed correct and supported completions hide the task and return to the world
automatically; there is no Continue confirmation. The journal's optional
`activityCompleted` field marks finished activities separately from readiness.
They are not selected again for that profile. Wrong answers keep the encounter
and its story binding, but Rust issues a different authored question and the host
mounts fresh controls. Previous selections/transcripts are cleared. Support and
check history remain cumulative, so a corrected answer cannot become a fresh
independent first check. Exact lost-response retries return the already issued
variation instead of generating another. Legacy active tasks retain their frozen
factory; newly issued tasks use `mission-practice-draft-3`.
