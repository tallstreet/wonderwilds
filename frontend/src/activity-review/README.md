# Activity control review

Start Vite with `npm run dev -- --host 127.0.0.1` from `frontend/`, then open
<http://127.0.0.1:5173/activity-review.html>. The page also has its own production
build entry. No API, database or provider credentials are needed.

Seven interactive examples demonstrate direct single/multiple choices, number
entry, word entry, ordered cards, object assignment and a writing workshop.
Tap/keyboard alternatives accompany dragging. Answers remain in page memory when
switching examples; Start again resets the selected example. Reload clears all.

These are UI samples for review, separate from the live mission controls and wire
contracts. Check displays the collected sample answer without deciding correctness
or saving evidence/progress. The workshop uses Ready to share and human-review
guidance. `controls.ts` deliberately owns no assessment rules or answer keys.

With Vite running, run `node qa/e2e/activity-review.mjs` from the repository root.
Set `CHROMIUM_PATH` to your Chromium executable when needed. The browser regression
covers all seven examples at desktop, phone and 320px widths, keyboard/touch,
dragging, undo, replacement, draft retention/reset, field validity and no writes.
