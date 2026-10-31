// Walking onto an area exit may commit travel before an encounter dialog opens.
// Wait for either outcome in one poll so no losing dialog wait is left running.
export async function waitForStoryVisit(page, id) {
  await page.waitForFunction(target => {
    if (target.startsWith('route:') && document.querySelector('#game')?.dataset.storyLocation === target.slice(6)) return true;
    return [...document.querySelectorAll('dialog[open] [data-story-action], dialog[open] [data-story-continue]')]
      .some(button => button.checkVisibility() && !button.disabled);
  }, id, { timeout: 20000 });
}
