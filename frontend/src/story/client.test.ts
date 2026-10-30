import assert from 'node:assert/strict';
import test from 'node:test';
import type { StorySnapshot } from '../../../contracts/generated/contracts.ts';
import input from '../../../backend/crates/director/src/story/fixtures/scene-workshop.json';
import { ApiFailure } from '../api/client.ts';
import { StoryClient, StorySession, type PendingStore } from './client.ts';
import { isStorySnapshot } from './validation.ts';
const snapshot = (revision = 3): StorySnapshot => ({
  schemaVersion: '1.0',
  contentVersion: 'pip-bridge-1',
  revision,
  locationId: 'workshop',
  commandBudgetExhausted: false,
  acknowledgeableEntryIds: ['greeting'],
  sceneInput: input as StorySnapshot['sceneInput'],
  scene: {
    sceneId: 'workshop-intro',
    presentation: [
      { type: 'dialogue', entryId: 'greeting', speakerId: 'woodcutter', text: 'Hello!' },
    ],
    choices: [{ label: 'Travel to the bank', actionId: 'story-3-1' }],
  },
});
function storage(): PendingStore & { values: Map<string, string> } {
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
const payload = { type: 'choose', actionId: 'story-3-1' } as const;
test('uncertain write survives reload with exact body and refreshes past old receipt', async () => {
  const store = storage(),
    bodies: string[] = [];
  const client = {
    snapshot: async () => snapshot(8),
    send: async (body: string) => {
      bodies.push(body);
      if (bodies.length === 1) throw new ApiFailure('UNAVAILABLE', 0);
      return snapshot(4);
    },
  };
  const original = new StorySession(client, 'profile-a', store, () => 'fixed-command');
  await original.refresh();
  await assert.rejects(original.choose(payload));
  assert.equal(store.values.size, 1);
  const resumed = new StorySession(client, 'profile-a', store);
  await resumed.refresh();
  assert.equal(resumed.hasPending, true);
  await assert.rejects(resumed.choose(payload));
  await resumed.retry();
  assert.equal(bodies[0], bodies[1]);
  assert.equal(resumed.needsRefresh, true);
  await assert.rejects(resumed.choose(payload));
  await resumed.refresh();
  assert.equal(resumed.snapshot?.revision, 8);
  assert.equal(store.values.size, 0);
});
test('conflict requires explicit refresh without automatic reissue', async () => {
  let sent = 0;
  const session = new StorySession(
    {
      snapshot: async () => snapshot(),
      send: async () => {
        sent++;
        throw new ApiFailure('REVISION_CONFLICT', 409);
      },
    },
    'a',
    storage(),
    () => 'one',
  );
  await session.refresh();
  await assert.rejects(session.choose(payload));
  assert.equal(session.hasPending, false);
  assert.equal(session.needsRefresh, true);
  await assert.rejects(session.choose(payload));
  await session.refresh();
  assert.equal(sent, 1);
});
test('blocked storage fails before sending and corrupt stored command fails closed', async () => {
  let sent = 0;
  const store = storage();
  store.setItem = () => {
    throw new Error('blocked');
  };
  const client = {
    snapshot: async () => snapshot(),
    send: async () => {
      sent++;
      return snapshot();
    },
  };
  const session = new StorySession(client, 'a', store, () => 'one');
  await session.refresh();
  await assert.rejects(session.choose(payload));
  assert.equal(sent, 0);
  assert.throws(
    () => new StorySession(client, 'a', { ...store, getItem: () => '{"progress":"invented"}' }),
  );
});
test('client rejects invalid or oversized wire data and late destroyed responses', async () => {
  assert.equal(isStorySnapshot(snapshot()), true);
  for (const body of [JSON.stringify({ ...snapshot(), unexpected: true }), ' '.repeat(65537)]) {
    await assert.rejects(new StoryClient('a', 'csrf', async () => new Response(body)).snapshot());
  }
  let respond!: (value: Response) => void;
  const client = new StoryClient(
    'a',
    'csrf',
    () =>
      new Promise((resolve) => {
        respond = resolve;
      }),
  );
  const pending = client.snapshot();
  client.destroy();
  respond(new Response(JSON.stringify(snapshot())));
  await assert.rejects(
    pending,
    (error: unknown) => error instanceof ApiFailure && error.code === 'CONTEXT_CHANGED',
  );
});
test('terminal snapshots forbid scenes and playable snapshots require scenes', () => {
  const terminal = {
    schemaVersion: '1.0',
    contentVersion: 'pip-bridge-1',
    revision: 256,
    locationId: 'bridge_bank',
    commandBudgetExhausted: true,
    acknowledgeableEntryIds: [],
  };
  assert.equal(isStorySnapshot(terminal), true);
  assert.equal(isStorySnapshot({ ...terminal, scene: snapshot().scene }), false);
  assert.equal(isStorySnapshot({ ...terminal, commandBudgetExhausted: false }), false);
});

test('failed GET after successful replay keeps actions blocked without reissuing command', async () => {
  let gets = 0;
  let sends = 0;
  const session = new StorySession(
    {
      snapshot: async () => {
        if (gets++ > 0) throw new ApiFailure('UNAVAILABLE', 0);
        return snapshot();
      },
      send: async () => {
        sends++;
        return snapshot(4);
      },
    },
    'a',
    storage(),
    () => 'one',
  );
  await session.refresh();
  await session.choose(payload);
  await assert.rejects(session.refresh());
  assert.equal(session.hasPending, false);
  assert.equal(session.needsRefresh, true);
  await assert.rejects(session.choose(payload));
  assert.equal(sends, 1);
});

test('auth failure on a retry cannot discard an earlier uncertain commit', async () => {
  for (const code of ['UNAUTHENTICATED', 'FORBIDDEN', 'NOT_FOUND', 'INVALID_INPUT'] as const) {
    const bodies: string[] = [];
    const store = storage();
    const session = new StorySession(
      {
        snapshot: async () => snapshot(),
        send: async (body) => {
          bodies.push(body);
          throw new ApiFailure(bodies.length === 1 ? 'UNAVAILABLE' : code, 401);
        },
      },
      'a',
      store,
      () => 'one',
    );
    await session.refresh();
    await assert.rejects(session.choose(payload));
    await assert.rejects(session.retry());
    assert.equal(session.hasPending, true);
    assert.equal(store.values.size, 1);
    assert.equal(bodies[0], bodies[1]);
    const resumed = new StorySession(
      {
        snapshot: async () => snapshot(4),
        send: async (body) => {
          bodies.push(body);
          return snapshot(4);
        },
      },
      'a',
      store,
    );
    await resumed.refresh();
    await resumed.retry();
    assert.equal(bodies[2], bodies[0]);
    assert.equal(resumed.hasPending, false);
  }
});
