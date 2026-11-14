// Current supported entry to the preserved story reader. Redirect URL coverage
// remains in story-source-status.mjs; these journeys exercise legacy gameplay.
export async function gotoLegacy(page, destination) {
  const url = new URL(destination);
  if (url.pathname === '/story.html' || url.pathname === '/demo.html') url.pathname = '/';
  url.searchParams.set('adventure', 'earlier');
  await page.goto(url.href);
}
