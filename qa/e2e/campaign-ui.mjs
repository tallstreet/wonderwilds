// Shared UI navigation for disposable campaign checks.
export async function finishLearningSetup(page, age = '8') {
  await page.locator('#campaign-age, #campaign-missions').first().waitFor();
  if (await page.locator('#campaign-age').count()) {
    await page.locator('#campaign-age').selectOption(age);
    await page.locator('#campaign-save-age').click();
    await page.locator('#campaign-age').waitFor({ state: 'hidden' });
  }
}

export async function openCampaignPage(page, id) {
  await page.locator('#campaign-more').click();
  await page.locator(`#campaign-${id}`).click();
}
