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
