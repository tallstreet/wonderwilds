# Accessible activity components (WW-11)

`mountActivity(container, encounter, callbacks, presentation)` renders all six contract families using semantic buttons, fieldsets, labelled counts and a polite status region. It returns `focus()` and `destroy()`. Call `focus()` after mounting and restore the opener's focus after destroying. `destroy()` stops narration and suppresses late feedback.

Resolve `authoredPresentation(encounter)` first. It loads the public approved prompt registry, selects single/multiple/ordered controls, and returns `null` for unknown instruction/prompt IDs. The host must decline unknown content. The registry contains no answers. Quantity and sharing instructions derive from server-approved parameters. Additional reviewed content can supply `ActivityPresentation` explicitly; raw model prose must not be passed as an approved prompt.

The callbacks are asynchronous:

- `onAttempt(answer)` submits the contract `Answer` to Rust and returns `{message, result?}`. The component never decides correctness, mastery, rewards or completion. Quantity uses a number; sharing uses recipient counts; other families use object/card IDs. Letter tiles use stringified indices so duplicate letters retain separate identities.
- `onSupport(kind)` records `hint`, `worked_example` or `read_aloud` before returning approved support text. Rejection displays a retry notice and reveals no new help or audio. Only the authoritative encounter's permitted actions enable operations.
- Optional `narrate(text)` plays reviewed local narration after support is recorded; `stopNarration()` cancels playback. No browser speech, microphone or external audio endpoint is created here. Narration is absent by default until the host supplies a reviewed adapter.
- Optional `onAbandon()` returns to exploration. The host owns endpoint/revision handling, encounter refresh, cancellation policy and navigation.

Every edit has touch and keyboard buttons; no dragging is required. Ordered choices have undo/clear controls. Displayed pattern prefixes cannot be selected as answers. Measurement labels include lengths in units. Pressed states, textual counts and visible outlines avoid colour-only information. Targets are at least 44 CSS pixels; reduced motion disables animation. Adult-reviewed E13/E14/E15 responses explicitly avoid automatic writing/reasoning certification.

Run from `frontend/`:

```sh
npm run typecheck
npm run lint
npm run build
npm run dev -- --host 127.0.0.1 --port 5173 --strictPort
# Separate terminal; requires: npm ci --prefix ../qa/e2e
node src/learning/activity.browser.mjs
```

Set `WW_ACTIVITY_TEST_URL` to another running Vite origin if needed. The browser suite serves an isolated document, imports the real components through Vite, and checks all six families, multiple sorting, actual keyboard edits, focus, answer payloads, support-before-audio, failed support, duplicate-request guards, literal malicious text, unknown prompts, display-only prefixes, permissions, destroyed pending callbacks and tablet-sized targets. It is outside the existing `src/*.test.ts` test glob. Playwright resolves from the locked `qa/e2e` package. `CHROMIUM_PATH` can override `/usr/bin/chromium`. Browser tooling is separate from application dependencies.

The opt-in local `demo.html` entrypoint mounts these components for the Meadow API slice. They do not replace `main.ts` or enable unfinished maps. Current automated checks use Chromium, not a full assistive-technology audit. Human screen-reader, iPad Safari, educator language, reviewed audio and real API journey checks remain integration/release gates. Relative-position choices now include simple authored diagrams; no accessibility or educational efficacy claim follows from these tests.

## Visual play controls

Activities now use the same local hero/prop atlases as the world: loose planks
move onto a bridge, berries move between picnic plates, and number/letter/picture
cards move into answer trays. Authored message and seed-story cards have local
illustrations with short captions. Pointer gestures support mouse, pen and touch;
tap/click and native keyboard buttons remain alternative controls. Ordered cards
can be inserted, rearranged and returned. Invalid drops leave the answer alone.
Dragging never submits an answer or grants rewards; Check still uses the existing
server callback. Controls freeze during submissions, and destroy cancels gestures.

`qa/e2e/visual-activities.mjs` tests real mouse and touch drag events, tap placement,
plate transfers, card reordering, exact answer payloads and narrow-screen layout.
Run it with the local Vite server and `CHROMIUM_PATH` as for the other browser tests.
The component suite still covers all six families and support/lifecycle boundaries.

## Optional spoken and typed answers

`ActivityCallbacks.recorder` supplies `start`, `finish` (transcript) and `cancel`.
The backend-world host supplies the recorder only when the local bootstrap reports
voice enabled and the browser supports microphone recording. The component
provides typing for missing quantity and non-phonics single-choice activities;
spoken transcripts fill that same editable field. Explicit **Use this answer**
performs bounded numeric or unique exact-label matching, never assessment. Unknown
or ambiguous text preserves the existing selection. **Check my idea** still calls
`onAttempt` independently. Ordering, sharing and phonics retain their existing
controls. No model-generated text is treated as an approved activity prompt.

Recording freezes answer controls and offers Finish/Cancel. Destruction and
cancellation stop microphone tracks and invalidate late transcripts; denial,
empty transcription and provider errors preserve the previous answer. Browser
recordings are decoded/downmixed/resampled to at most 20 seconds of 16-kHz PCM WAV
before upload. Narration behavior is unchanged; this feature is input only.

The development-only `POST /__demo/transcribe` protocol takes `audio/wav` binary
RIFF PCM16 mono, 16 kHz, up to 640044 bytes, and returns at most 300 characters of
`text/plain` (not HTML or a gameplay JSON contract). The existing bridge requires
Host/Origin/CSRF/profile checks, then verifies the backend session, with one
active request, a two-second start interval and a 30-second deadline. Errors are
empty HTTP 400/401/403/405/413/415/429/502/503/504 responses; no provider errors,
keys or recordings are logged. No new gameplay schema or persistence is involved.
