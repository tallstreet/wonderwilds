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

test('legacy begin conflicts stay strict and require an explicit refresh', async () => {
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
  corrupted.values.set('wonderwilds.mission-learning.pending.v2:profile-1', '{bad');
  assert.throws(() => new MissionLearningSession(client, 'profile-1', corrupted));
});

test('overlapping progress reads share one request and an answer waits for its latest revision', async () => {
  let resolveRead!: (value: MissionLearningSnapshot) => void;
  let reads = 0;
  const bodies: string[] = [];
  const session = new MissionLearningSession(
    {
      snapshot: () => {
        reads++;
        return new Promise((resolve) => {
          resolveRead = resolve;
        });
      },
      send: async (body) => {
        bodies.push(body);
        return snapshot(10);
      },
    },
    'profile-read-race',
    storage(),
    () => 'after-read',
  );
  const first = session.refresh();
  const second = session.refresh();
  const answer = session.choose({
    type: 'answer',
    encounterId: 'active-task',
    answer: { kind: 'choice', choiceId: 'number-2' },
  });
  await Promise.resolve();
  assert.equal(reads, 1);
  assert.equal(bodies.length, 0);
  resolveRead(snapshot(9));
  await Promise.all([first, second, answer]);
  assert.equal(JSON.parse(bodies[0]!).expectedRevision, 9);
  assert.equal(session.snapshot?.revision, 10);
});

test('a progress read cannot race a submitted command or replace its confirmed response', async () => {
  let finish!: (value: MissionLearningSnapshot) => void;
  const session = new MissionLearningSession(
    {
      snapshot: async () => snapshot(1),
      send: () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    },
    'profile-write-race',
    storage(),
    () => 'in-flight',
  );
  await session.refresh();
  const write = session.choose({ type: 'begin' });
  await assert.rejects(session.refresh(), /still running/);
  finish(snapshot(2));
  await write;
  assert.equal(session.snapshot?.revision, 2);
});

function issuedSnapshot(
  revision: number,
  variantId = 'M03-practice-v3-00',
): MissionLearningSnapshot {
  return {
    ...snapshot(revision),
    activeActivity: {
      schemaVersion: '1.0',
      id: 'issued-question',
      skillId: 'M03',
      contentVersion: 'mission-practice-draft-3',
      variantId,
      contextId: 'counting',
      representation: 'collection',
      presentation: {
        representation: 'collection',
        title: 'Count',
        objective: 'Count objects',
        lessonText: 'Count each once.',
        prompt: 'How many?',
        context: 'A picnic',
      },
      task: {
        kind: 'count_collection',
        layout: 'scattered',
        objects: [{ id: 'one', label: 'apple' }],
        choices: [
          { id: 'a', label: '1', value: 1 },
          { id: 'b', label: '2', value: 2 },
        ],
      },
      support: { hintCount: 0, workedExample: false, readAloud: false },
    },
  };
}

test('a definite conflict automatically saves the original answer and variant at the latest revision', async () => {
  const bodies: string[] = [];
  let reads = 0;
  let ids = 0;
  const store = storage();
  const session = new MissionLearningSession(
    {
      snapshot: async () =>
        ++reads === 1 ? issuedSnapshot(3) : issuedSnapshot(5, 'M03-practice-v3-01'),
      send: async (body) => {
        bodies.push(body);
        if (bodies.length === 1) throw new ApiFailure('REVISION_CONFLICT', 409);
        return {
          ...snapshot(6),
          lastFeedback: { result: 'correct', message: 'That works!', completed: true },
        };
      },
    },
    'profile-auto',
    store,
    () => `auto-${++ids}`,
  );
  await session.refresh();
  const payload = {
    type: 'answer',
    encounterId: 'issued-question',
    variantId: 'M03-practice-v3-00',
    answer: { kind: 'choice', choiceId: 'a' },
  } as const;
  const result = await session.choose(payload);
  assert.equal(result.lastFeedback?.completed, true);
  const first = JSON.parse(bodies[0]!);
  const second = JSON.parse(bodies[1]!);
  assert.notEqual(first.commandId, second.commandId);
  assert.equal(second.expectedRevision, 5);
  assert.deepEqual(first.payload, second.payload);
  assert.equal(store.values.size, 0);
});

test('uncertain recovery preserves the replacement command exactly across reload', async () => {
  const store = storage();
  const bodies: string[] = [];
  let ids = 0;
  const client = {
    snapshot: async () => issuedSnapshot(7),
    send: async (body: string) => {
      bodies.push(body);
      if (bodies.length === 1) throw new ApiFailure('REVISION_CONFLICT', 409);
      if (bodies.length === 2) throw new ApiFailure('UNAVAILABLE', 0);
      return snapshot(8);
    },
  };
  const original = new MissionLearningSession(client, 'profile-lost', store, () => `lost-${++ids}`);
  await original.refresh();
  await assert.rejects(
    original.choose({
      type: 'answer',
      encounterId: 'issued-question',
      variantId: 'M03-practice-v3-00',
      answer: { kind: 'choice', choiceId: 'a' },
    }),
  );
  assert.equal(original.hasPending, true);
  const resumed = new MissionLearningSession(client, 'profile-lost', store);
  await resumed.refresh();
  await resumed.retry();
  assert.equal(bodies[1], bodies[2]);
  assert.equal(store.values.size, 0);
});

test('already completed activities reconcile without resending an answer or reopening another activity', async () => {
  const store = storage();
  let reads = 0;
  let sends = 0;
  const finished = {
    ...issuedSnapshot(6),
    activeActivity: { ...issuedSnapshot(6).activeActivity!, id: 'another-question' },
  };
  finished.skills[0]!.activityCompleted = true;
  const session = new MissionLearningSession(
    {
      snapshot: async () => (++reads === 1 ? issuedSnapshot(3) : finished),
      send: async () => {
        sends++;
        throw new ApiFailure('REVISION_CONFLICT', 409);
      },
    },
    'profile-finished',
    store,
  );
  await session.refresh();
  const result = await session.choose({
    type: 'answer',
    encounterId: 'issued-question',
    variantId: 'M03-practice-v3-00',
    answer: { kind: 'choice', choiceId: 'a' },
  });
  assert.equal(result, finished);
  assert.equal(sends, 1);
  assert.equal(store.values.size, 0);
});

test('a failed conflict reconciliation read preserves the answer across reload', async () => {
  const store = storage();
  const bodies: string[] = [];
  let reads = 0;
  const client = {
    snapshot: async () => {
      if (++reads === 2) throw new ApiFailure('UNAVAILABLE', 0);
      return issuedSnapshot(reads === 1 ? 3 : 5);
    },
    send: async (body: string) => {
      bodies.push(body);
      if (bodies.length <= 2) throw new ApiFailure('REVISION_CONFLICT', 409);
      return snapshot(6);
    },
  };
  const original = new MissionLearningSession(
    client,
    'profile-read-failed',
    store,
    () => 'original-answer',
  );
  await original.refresh();
  await assert.rejects(
    original.choose({
      type: 'answer',
      encounterId: 'issued-question',
      variantId: 'M03-practice-v3-00',
      answer: { kind: 'choice', choiceId: 'a' },
    }),
  );
  assert.equal(original.hasPending, true);
  assert.equal([...store.values.values()][0], bodies[0]);
  const resumed = new MissionLearningSession(
    client,
    'profile-read-failed',
    store,
    () => 'reconciled-answer',
  );
  await resumed.refresh();
  await resumed.retry();
  assert.equal(bodies[0], bodies[1]);
  assert.deepEqual(JSON.parse(bodies[0]!).payload, JSON.parse(bodies[2]!).payload);
  assert.equal(store.values.size, 0);
});
