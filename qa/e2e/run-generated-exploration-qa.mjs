// Real API and disposable database, synthetic loopback model. No paid provider calls.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
let requests = 0;
const server = createServer(async (req, res) => {
  assert.equal(req.headers.authorization, 'Bearer synthetic.id');
  let text = ''; for await (const chunk of req) text += chunk;
  const body = JSON.parse(text);
  assert.equal(body.response_format.json_schema.name, 'GeneratedNeighbourhood');
  const input = JSON.parse(body.messages[1].content);
  assert.equal(Object.hasOwn(input, 'profileId'), false);
  assert.equal(Object.hasOwn(input, 'sessionId'), false);
  requests++;
  const content = requests === 5 ? { invalid: true } : {
    name: `Visitor ${requests}`, personality: 'A curious neighbour who enjoys quiet discoveries.',
    sprite: 'glimmer', greeting: `Hello! Let us listen for little sounds in ${input.area}.`,
    title: `The little sounds of ${input.area}`, premise: 'Investigate two places and return to share your discoveries.',
    steps: [{ label: 'Listen beside the hollow branch.', shape: 'hollow', learningBridge: 'Help organise our discovery markers so we can keep track of the sounds.' }, { label: 'Look closely at the rustling leaves.', shape: 'leaf' }],
    payoff: 'You shared the sounds of the woodland with your new friend.'
  };
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ choices: [{ finish_reason: 'stop', message: { content: JSON.stringify(content) } }] }));
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
try {
  const child = spawn(process.execPath, ['qa/e2e/run-training-qa.mjs'], {
    stdio: 'inherit', env: { ...process.env, WW_QA_BROWSER_TEST: 'qa/e2e/story-exploration.mjs', WW_GENERATED_STORIES: '1',
      WONDERWILDS_MODAL_STORY: '0', WONDERWILDS_MODAL_PLANNER: '0', WONDERWILDS_MODAL_EXPLORATION: '1',
      MODAL_PROXY_TOKEN_ID: 'synthetic', MODAL_PROXY_TOKEN_SECRET: 'id',
      WONDERWILDS_MODAL_EXPLORATION_ENDPOINT: `http://127.0.0.1:${server.address().port}/v1/chat/completions` }
  });
  const [code] = await once(child, 'exit');
  assert.equal(code, 0);
  assert.equal(requests, 6, 'unfinished stories and exact retries never regenerate content');
  console.log('Generated exploration: six arrival requests, stable saved content, repeat generation and malformed-model fallback passed.');
} finally { await new Promise(resolve => server.close(resolve)); }
