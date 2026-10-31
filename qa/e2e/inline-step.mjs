// Full story journeys use the supported route; mission-learning.mjs separately
// checks independent answers, wrong answers, support and uncertain learning saves.
export async function finishInlineStep(page, type) {
  if (!['inspect', 'collect', 'exchange', 'assemble'].includes(type)) return;
  await page.locator('.ww-mission-activity').waitFor();
  await page.getByRole('button', { name: /^(Let’s do it together|Finish activity)$/ }).click();
  await page.locator('.ww-mission-activity').waitFor({ state: 'hidden' });
  // Completion can return either speech or the next actionable encounter.
  await page.waitForFunction(() => {
    const dialog = document.querySelector('dialog[open]');
    return !dialog || Boolean(dialog.querySelector('[data-story-handle], [data-story-continue]'));
  });
}
