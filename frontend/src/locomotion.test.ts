import test from 'node:test';
import assert from 'node:assert/strict';
import { advanceGaitPhase, sampleGait, walkFrameIndex, applyWalkCycle } from './locomotion.ts';
const idle = { lift: 0, scaleX: 1, scaleY: 1, rotation: 0, stride: 0 };

test('Idle and reduced motion produce a grounded stable pose at every phase', () => {
  for (const species of ['kitten', 'bunny', 'bird'])
    for (const phase of [0, 0.13, 0.5, 0.97, NaN]) {
      assert.deepEqual(sampleGait(species, phase, false), idle);
      assert.deepEqual(sampleGait(species, phase, true, { reducedMotion: true }), idle);
    }
});
test('Distance-driven cycles do not advance at obstacles and are frame-rate independent', () => {
  for (const species of ['kitten', 'bird', 'bunny']) {
    assert.equal(advanceGaitPhase(0.5, 0, species), 0.5);
    let smallSteps = 0;
    for (let i = 0; i < 20; i++) smallSteps = advanceGaitPhase(smallSteps, 3, species);
    assert.ok(Math.abs(smallSteps - advanceGaitPhase(0, 60, species)) < 1e-12);
    assert.equal(advanceGaitPhase(0, NaN, species), 0);
  }
});
test('Species gait stays bounded, has ground contact, and is continuous across cycle boundary', () => {
  for (const species of ['kitten', 'bird', 'bunny']) {
    let maxLift = 0;
    for (let i = 0; i <= 1000; i++) {
      const p = sampleGait(species, i / 1000, true);
      maxLift = Math.max(maxLift, p.lift);
      assert.ok(p.lift >= 0 && p.lift <= 9.001);
      assert.ok(p.scaleX > 0.96 && p.scaleX < 1.04);
      assert.ok(p.scaleY > 0.93 && p.scaleY < 1.03);
      assert.ok(Math.abs(p.rotation) <= 3.001);
    }
    assert.ok(maxLift > 0);
    const a = sampleGait(species, 1 - 1e-7, true),
      b = sampleGait(species, 1e-7, true);
    for (const key of Object.keys(a) as (keyof typeof a)[])
      assert.ok(Math.abs(a[key] - b[key]) < 0.00001, `${species} ${key} discontinuous`);
    assert.equal(sampleGait(species, 0, true).lift, 0);
  }
  assert.equal(sampleGait('bunny', 0.1, true).lift, 0);
  assert.ok(sampleGait('bunny', 0.1, true).scaleY < 1);
  assert.equal(sampleGait('bunny', 0.5, true).lift, 9);
});
test('Real walk sheet selects six phases and cannot override another direction or idle', () => {
  assert.deepEqual(
    [0, 1, 2, 3, 4, 5].map((i) => walkFrameIndex(i / 6 + 0.001)),
    [0, 1, 2, 3, 4, 5],
  );
  assert.equal(walkFrameIndex(1), 0);
  const changed: string[][] = [];
  const sprite = {
    scene: { textures: { exists: () => true, get: () => ({ has: () => true }) } },
    setTexture: (...args: string[]) => changed.push(args),
  };
  assert.equal(applyWalkCycle(sprite, 'rose', 0.4, true, { facing: 'up' }), false);
  assert.equal(applyWalkCycle(sprite, 'rose', 0.4, false), false);
  assert.equal(applyWalkCycle(sprite, 'poppy', 0.4, true), false);
  assert.equal(applyWalkCycle(sprite, 'rose', 0.4, true, { reducedMotion: true }), false);
  assert.equal(changed.length, 0);
  assert.equal(applyWalkCycle(sprite, 'rose', 0.4, true, { facing: 'down' }), true);
  assert.equal(changed[0][1], '2');
});
