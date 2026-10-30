import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../../', import.meta.url));
const temp = await mkdtemp(join(tmpdir(), 'wonderwilds-review-qa-'));
const progress = join(temp, 'progress.json');
let server, browser, context, url;
async function start() {
  server = spawn('python3', ['qa/story-eval/review_server.py', '--port', '0', '--progress', progress], {cwd: root});
  let output = '';
  url = await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`Review server startup timeout: ${output}`)), 30000);
    server.on('error', error => {clearTimeout(timeout); reject(error);});
    server.on('exit', code => {clearTimeout(timeout); reject(new Error(`Server exited ${code}: ${output}`));});
    server.stderr.on('data', chunk => {output += chunk;});
    server.stdout.on('data', chunk => {
      output += chunk;
      const match = output.match(/http:\/\/127\.0\.0\.1:\d+\//);
      if (match) {clearTimeout(timeout); resolve(match[0]);}
    });
  });
}
async function stop() {
  if (server && server.exitCode === null) {
    const exit = new Promise(resolve => server.once('exit', resolve));
    server.kill('SIGTERM'); await exit;
  }
}
try {
  await start();
  browser = await chromium.launch({headless: true, executablePath: process.env.CHROMIUM_PATH});
  context = await browser.newContext({viewport: {width: 1360, height: 1000}});
  const page = await context.newPage();
  const errors = []; page.on('pageerror', error => errors.push(error.message));
  let gradeRequests = 0;
  page.on('request', request => {if (request.url().includes('/api/grade')) gradeRequests++;});
  await page.goto(url);
  await page.locator('#candidate-id').waitFor();
  assert.equal(gradeRequests, 0);
  assert.equal(await page.locator('#model-feedback').isVisible(), false);
  const workspace = await (await fetch(`${url}api/review`)).json();
  assert.ok(workspace.items.every(item => !('judgment' in item) && !('recommendation' in item)));
  const first = workspace.items[0];
  const params = new URLSearchParams({scenarioId: first.scenarioId, candidateId: first.candidateId});
  assert.equal((await fetch(`${url}api/grade?${params}`)).status, 403);
  assert.equal((await fetch(`${url}api/review`, {headers: {Origin: 'https://example.com'}})).status, 403);
  assert.equal((await fetch(`${url}api/decision`, {method: 'POST', headers: {'Content-Type': 'application/json'}, body: '{}'})).status, 403);

  await page.getByLabel('Reviewer name', {exact: true}).fill('Browser QA');
  await page.getByRole('button', {name: 'Approve unchanged', exact: true}).click();
  await page.locator('#decision-status').filter({hasText: 'Saved · Approved unchanged'}).waitFor();
  await page.locator('#model-feedback').waitFor();
  const approvedId = await page.locator('#candidate-id').textContent();
  await page.reload();
  await page.locator('#decision-status').filter({hasText: 'Saved · Approved unchanged'}).waitFor();
  assert.equal(await page.locator('#candidate-id').textContent(), approvedId);

  await page.getByRole('button', {name: 'Next unreviewed', exact: true}).click();
  assert.equal(await page.locator('#model-feedback').isVisible(), false);
  await page.getByLabel('Review notes', {exact: true}).fill('Use concrete language about the bridge.');
  await page.getByLabel('Suggested replacement text', {exact: false}).fill('The bridge is ready for our walk.');
  assert.equal(await page.getByRole('button', {name: 'Approve unchanged'}).isDisabled(), true);
  await page.getByRole('button', {name: 'Needs revision', exact: true}).click();
  await page.locator('#decision-status').filter({hasText: 'Saved · Needs revision'}).waitFor();
  await page.locator('#model-feedback').waitFor();

  await page.getByRole('button', {name: 'Next unreviewed', exact: true}).click();
  await page.getByLabel('Review notes', {exact: true}).fill('This test decision rejects the original wording.');
  await page.route('**/api/decision', route => route.fulfill({status: 500, contentType: 'application/json', body: JSON.stringify({error: 'Simulated disk failure'})}));
  await page.getByRole('button', {name: 'Reject', exact: true}).click();
  await page.locator('#save-error').filter({hasText: 'Simulated disk failure'}).waitFor();
  assert.equal(await page.locator('#model-feedback').isVisible(), false);
  assert.match(await page.getByLabel('Review notes', {exact: true}).inputValue(), /rejects/);
  await page.unroute('**/api/decision');
  await page.getByRole('button', {name: 'Reject', exact: true}).click();
  await page.locator('#decision-status').filter({hasText: 'Saved · Rejected'}).waitFor();
  await page.locator('#model-feedback').waitFor();
  const exported = (await (await fetch(`${url}api/export`)).text()).trim().split('\n').map(JSON.parse);
  assert.deepEqual(exported.map(row => row.decision).sort(), ['approve', 'reject', 'revise']);
  assert.ok(exported.every(row => row.firstDecision && row.candidateSha256 && row.reviewer === 'Browser QA'));
  await page.screenshot({path: '/tmp/wonderwilds-human-review-desktop.png', fullPage: true});
  await page.setViewportSize({width: 390, height: 844});
  await page.getByRole('button', {name: 'Next unreviewed', exact: true}).click();
  assert.equal(await page.locator('#model-feedback').isVisible(), false);
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
  await page.screenshot({path: '/tmp/wonderwilds-human-review-mobile.png', fullPage: true});

  await stop(); await start();
  await page.goto(url); await page.locator('#candidate-id').waitFor();
  await page.locator('#progress').filter({hasText: '3 of 96 decided'}).waitFor();
  assert.equal(await page.getByLabel('Reviewer name', {exact: true}).inputValue(), 'Browser QA');
  const persisted = JSON.parse(await readFile(progress, 'utf8'));
  assert.equal(persisted.reviews.length, 3);
  assert.deepEqual(errors, []);
  console.log('PASS: blind grades, approve/revise/reject, durable restart/reload, honest failed save, revision gating, export, origin/token checks, mobile layout');
} finally {
  await context?.close(); await browser?.close(); await stop();
  await rm(temp, {recursive: true, force: true});
}
