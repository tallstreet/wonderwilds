import assert from 'node:assert/strict';
import test from 'node:test';
import { sceneSourceLabel } from './source-status.ts';

test('scene status distinguishes generated prose, authored mode and provider fallback', () => {
  const base = { commandBudgetExhausted: false };
  assert.equal(sceneSourceLabel({ ...base, sceneSource: 'llm' }), 'LLM-generated dialogue');
  assert.equal(sceneSourceLabel({ ...base, sceneSource: 'authored' }), 'Preauthored story');
  assert.equal(
    sceneSourceLabel({ ...base, sceneSource: 'authored_fallback' }),
    'Preauthored story · LLM fallback',
  );
  assert.equal(sceneSourceLabel(base), 'Story source unavailable · refresh to check');
  assert.equal(
    sceneSourceLabel({ commandBudgetExhausted: true, sceneSource: 'llm' }),
    'Story dialogue unavailable',
  );
});
