// Full story journeys use the supported route; mission-learning.mjs separately
// checks independent answers, wrong answers, support and uncertain learning saves.
export async function finishInlineStep(page, type) {
  if (!['inspect', 'collect', 'exchange', 'assemble'].includes(type)) return;
  await page.locator('.ww-mission-activity').waitFor();
  await page.getByRole('button', { name: 'Let’s do it together', exact: true }).click();
  await page.getByRole('button', { name: 'Continue the story', exact: true }).click();
  await page.locator('dialog').waitFor({ state: 'hidden' });
}
