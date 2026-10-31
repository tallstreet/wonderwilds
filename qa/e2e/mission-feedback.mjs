import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, writeFile, readFile, rm, appendFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from 'playwright';

const temp = await mkdtemp(join(tmpdir(), 'wonderwilds-mission-feedback-'));
const log = join(temp, 'missions.jsonl'), feedback = join(temp, 'feedback.json');
const capture = {format:'mission-capture-1',id:'a'.repeat(64),kind:'neighbourhood',providerValidation:'passed',elapsedMs:40,
  request:{model:'synthetic',messages:[{role:'system',content:'No real provider'}]},
  output:{title:'<img src=x onerror=alert(1)>',premise:'Trace the travelling tune.',steps:[{label:'Listen to the leaves',discovery:'A breeze makes the tune.'}]}};
let server, browser;
async function start() {
  server = spawn('python3',['qa/story-eval/mission_review.py','--log',log,'--feedback',feedback,'--port','0']);
  return await new Promise((resolve,reject) => {
    let output = '';
    const timeout = setTimeout(() => reject(new Error(output)),15000);
    server.on('error',reject); server.stderr.on('data',chunk => {output += chunk;});
    server.stdout.on('data',chunk => {output += chunk; const match = output.match(/http:\/\/127\.0\.0\.1:\d+\//); if(match) {clearTimeout(timeout); resolve(match[0]);}});
    server.on('exit',code => {clearTimeout(timeout); reject(new Error(`Server exited ${code}: ${output}`));});
  });
}
async function stop() {
  if(server?.exitCode === null) {const exited = new Promise(resolve => server.once('exit',resolve)); server.kill(); await exited;}
}
try {
  await writeFile(log,JSON.stringify(capture)+'\n');
  let url = await start();
  browser = await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH});
  const page = await browser.newPage({viewport:{width:1280,height:1000}});
  const errors = []; page.on('pageerror',error => errors.push(error.message));
  await page.goto(url);
  await page.getByRole('heading',{name:capture.output.title,exact:true}).waitFor();
  assert.equal(await page.locator('img').count(),0);
  await page.getByLabel('Reviewer name').fill('Browser reviewer');
  await page.getByLabel('Reviewer name').press('Tab');
  for(const name of ['Story coherence and payoff','Clear next steps','Variety / originality','Learning invitation fits the story','Age suitability (6–8)']) {
    await page.getByLabel(name,{exact:true}).selectOption('4');
  }
  await page.getByLabel('Decision',{exact:true}).selectOption('revise');
  await page.getByLabel('Notes / suggested improvements').fill('Make the clue more concrete.');
  await page.getByRole('button',{name:'Save feedback',exact:true}).click();
  await page.locator('#saved').filter({hasText:/^Saved /}).waitFor();
  const saved = JSON.parse(await readFile(feedback,'utf8'));
  assert.equal(saved[0].scores.story,4);
  await stop(); url = await start(); await page.goto(url);
  await page.getByLabel('Reviewer name').fill('Browser reviewer');
  await page.getByLabel('Reviewer name').press('Tab');
  await page.getByRole('button',{name:'Refresh captures'}).click();
  await page.locator('#saved').filter({hasText:/^Saved /}).waitFor();
  assert.equal(await page.getByLabel('Notes / suggested improvements').inputValue(),'Make the clue more concrete.');
  await appendFile(log,JSON.stringify({...capture,id:'b'.repeat(64),providerValidation:'failed',output:'Malformed JSON from model'})+'\n');
  await page.getByRole('button',{name:'Refresh captures'}).click();
  await page.locator('#counts').filter({hasText:'2 captures'}).waitFor();
  await page.getByLabel('Show',{exact:true}).selectOption('failed');
  assert.equal(await page.locator('#queue button').count(),1);
  await page.locator('#queue button').click();
  await page.setViewportSize({width:390,height:844});
  await page.screenshot({path:join(temp,'phone.png'),fullPage:true});
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  const exported = await (await fetch(url+'api/export')).text();
  assert.equal(exported.trim().split('\n').length,2);
  assert.ok(exported.includes('Make the clue more concrete.'));
  assert.deepEqual(errors,[]);
  console.log(`Mission feedback passed: safe text, scoring, restart, live queue, export and phone layout. Screenshot: ${join(temp,'phone.png')}`);
} finally {await browser?.close(); await stop();}
