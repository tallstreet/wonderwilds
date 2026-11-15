import assert from 'node:assert/strict';
import test from 'node:test';
import { projectBoard } from './projects.ts';
const move = (label: string, movement: string, disabled = false) =>
  `<button data-move="${movement}" ${disabled ? 'disabled' : ''}>${label}</button>`;
test('construction needs matching pieces and both braces before crossing', () => {
  const empty = projectBoard('trial_bridge', [], 'long', move)!;
  assert.match(empty, /bridge_fit:middle:long/);
  assert.match(empty, /data-move="bridge_cross" disabled/);
  assert.doesNotMatch(empty, /garden_route/);
  const built = ['near', 'middle', 'far', 'brace_near', 'brace_far'].map(
    (part) => `project:trial_bridge:${part}`,
  );
  assert.match(
    projectBoard('trial_bridge', built, 'short', move)!,
    /data-move="bridge_cross" >Help/,
  );
});
test('garden gates follow the saved channel and mulch follows watering', () => {
  const empty = projectBoard('trial_bloom', [], 'short', move)!;
  assert.match(empty, /data-move="garden_water:north" disabled/);
  assert.match(empty, /data-move="garden_mulch:north" disabled/);
  assert.doesNotMatch(empty, /bridge_fit/);
  const wet = projectBoard(
    'trial_bloom',
    ['project:trial_bloom:north:beds', 'project:trial_bloom:watered:north'],
    'short',
    move,
  )!;
  assert.match(wet, /data-move="garden_mulch:north" >Spread/);
  assert.match(wet, /data-move="garden_check" disabled/);
  assert.equal(projectBoard('trial_echo', [], 'short', move), undefined);
});
