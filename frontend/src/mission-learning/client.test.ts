import assert from 'node:assert/strict';
import test from 'node:test';
import type { MissionLearningSnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import {
  MissionLearningClient,
  MissionLearningSession,
  type MissionLearningPendingStore,
} from './client.ts';
import { isMissionLearningSnapshot } from './validation.ts';

const snapshot = (revision = 2): MissionLearningSnapshot => ({
  schemaVersion: '1.0',
  contentVersion: 'meadow-curriculum-1',
  policyVersion: 'frontier-v1',
  policyExplanation: 'Try several different examples over another visit.',
  revision,
  missionId: 'pip-bridge',
  locationId: 'bridge-bank',
  available: true,
  invitation: {
    title: 'Count the bridge planks',
    relation: 'Pip is getting the bridge ready.',
    message: 'Want to practise counting before you build together?',
  },
  frontierSkillId: 'M03',
  skills: [
    {
      skillId: 'M03',
      objective: 'Count a small collection',
      prerequisites: [],
      status: 'available',
      supportedByMission: true,
      blockers: [],
      eligibleChecks: 0,
      correctChecks: 0,
      distinctVariants: 0,
      distinctContexts: 0,
      distinctRepresentations: 0,
      distinctSessions: 0,
      nextStep: 'Try a counting activity.',
    },
  ],
});

function storage(): MissionLearningPendingStore & { values: Map<string, string> } {
  const values = new Map<string, string>();
  return {
    values,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value);
    },
    removeItem: (key) => {
      values.delete(key);
    },
  };
}

test('mission-learning responses validate the closed public contract', () => {
  const valid = snapshot();
  assert.equal(isMissionLearningSnapshot(valid), true);
  assert.equal(isMissionLearningSnapshot({ ...valid, answerKey: ['secret'] }), false);
});

test('client uses profile and CSRF headers on same-origin mission-learning routes', async () => {
  const requests: { url: string; init?: RequestInit }[] = [];
  const client = new MissionLearningClient('profile-1', 'csrf-1', async (url, init) => {
    requests.push({ url: String(url), init });
    return Response.json(snapshot());
  });
  await client.snapshot();
  await client.send(
    JSON.stringify({
      schemaVersion: '1.0',
      commandId: 'begin-1',
      expectedRevision: 2,
      payload: { type: 'begin' },
    }),
  );
  assert.ok(requests[0]);
  assert.ok(requests[1]);
  const firstHeaders = requests[0].init?.headers as Record<string, string>;
  const secondHeaders = requests[1].init?.headers as Record<string, string>;
  assert.equal(requests[0]?.url, '/api/v1/mission-learning');
  assert.equal(requests[0]?.init?.method, 'GET');
  assert.equal(firstHeaders['X-Wonderwilds-Profile'], 'profile-1');
  assert.equal(requests[1]?.url, '/api/v1/mission-learning/commands');
  assert.equal(secondHeaders['X-CSRF-Token'], 'csrf-1');
});

test('uncertain mission-learning write reloads and retries the exact command bytes', async () => {
  const store = storage();
  const bodies: string[] = [];
  let current = snapshot(9);
  const client = {
    snapshot: async () => current,
    send: async (body: string) => {
      bodies.push(body);
      if (bodies.length === 1) throw new ApiFailure('UNAVAILABLE', 0);
      return snapshot(3);
    },
  };
  const original = new MissionLearningSession(client, 'profile-1', store, () => 'fixed-command');
  await original.refresh();
  await assert.rejects(original.choose({ type: 'begin' }));
  assert.equal(store.values.size, 1);

  const resumed = new MissionLearningSession(client, 'profile-1', store);
  await resumed.refresh();
  assert.equal(resumed.hasPending, true);
  await resumed.retry();
  assert.equal(bodies[0], bodies[1]);
  assert.equal(resumed.needsRefresh, true);
  await resumed.refresh();
  assert.equal(resumed.snapshot?.revision, 9);
  assert.equal(store.values.size, 0);
});

test('revision conflict clears uncertain command and requires an explicit refresh', async () => {
  const store = storage();
  let sent = 0;
  const session = new MissionLearningSession(
    {
      snapshot: async () => snapshot(),
      send: async () => {
        sent++;
        throw new ApiFailure('REVISION_CONFLICT', 409);
      },
    },
    'profile-1',
    store,
    () => 'conflict-command',
  );
  await session.refresh();
  await assert.rejects(session.choose({ type: 'begin' }));
  assert.equal(session.hasPending, false);
  assert.equal(session.needsRefresh, true);
  await assert.rejects(session.choose({ type: 'begin' }));
  await session.refresh();
  assert.equal(sent, 1);
});

test('blocked storage fails before send and corrupt pending command fails closed', async () => {
  let sent = 0;
  const blocked = storage();
  blocked.setItem = () => {
    throw new Error('blocked');
  };
  const client = {
    snapshot: async () => snapshot(),
    send: async () => {
      sent++;
      return snapshot();
    },
  };
  const session = new MissionLearningSession(client, 'profile-1', blocked, () => 'x');
  await session.refresh();
  await assert.rejects(session.choose({ type: 'begin' }));
  assert.equal(sent, 0);
  const corrupted = storage();
  corrupted.values.set('wonderwilds.mission-learning.pending.v1:profile-1', '{bad');
  assert.throws(() => new MissionLearningSession(client, 'profile-1', corrupted));
});
