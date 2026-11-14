import assert from 'node:assert/strict';
import test from 'node:test';
import type { CampaignSnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import { CampaignSession, type PendingStore } from './client.ts';
import { isCampaignSnapshot } from './validation.ts';
const snapshot = (revision = 3): CampaignSnapshot => ({
  schemaVersion: '1.0',
  revision,
  adopted: false,
  adoptionBlocked: false,
  locations: [],
  starters: [],
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
test('uncertain campaign adoption survives reload and retries the exact body before refreshing', async () => {
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
  const session = new CampaignSession(
    client,
    'a',
    store,
    () => 'd65e137d-e686-4070-aa1c-bea1b5ff8f47',
  );
  await session.refresh();
  await assert.rejects(session.choose({ type: 'adopt' }));
  const resumed = new CampaignSession(client, 'a', store);
  await resumed.refresh();
  assert.equal(resumed.hasPending, true);
  await assert.rejects(resumed.choose({ type: 'adopt' }));
  await resumed.retry();
  assert.equal(bodies[0], bodies[1]);
  assert.equal(resumed.needsRefresh, true);
  await assert.rejects(resumed.choose({ type: 'adopt' }));
  await resumed.refresh();
  assert.equal(resumed.snapshot?.revision, 8);
  assert.equal(store.values.size, 0);
});
test('definitive invalid move does not trap the player in retry; uncertain and auth failures retain intent', async () => {
  for (const code of ['INVALID_INPUT', 'REVISION_CONFLICT', 'UNAVAILABLE', 'FORBIDDEN'] as const) {
    const store = storage();
    const session = new CampaignSession(
      {
        snapshot: async () => snapshot(),
        send: async () => {
          throw new ApiFailure(code, 400);
        },
      },
      'a',
      store,
      () => 'd65e137d-e686-4070-aa1c-bea1b5ff8f47',
    );
    await session.refresh();
    await assert.rejects(session.choose({ type: 'adopt' }));
    assert.equal(session.hasPending, code === 'UNAVAILABLE' || code === 'FORBIDDEN');
  }
});
test('blocked storage prevents campaign write and unknown pending JSON fails closed', async () => {
  const store = storage();
  let writes = 0;
  store.setItem = () => {
    throw Error('blocked');
  };
  const client = {
    snapshot: async () => snapshot(),
    send: async () => {
      writes++;
      return snapshot();
    },
  };
  const session = new CampaignSession(client, 'a', store);
  await session.refresh();
  await assert.rejects(session.choose({ type: 'adopt' }));
  assert.equal(writes, 0);
  store.values.set(
    session.storageKey,
    '{"schemaVersion":"1.0","payload":{"type":"invent_reward"}}',
  );
  assert.throws(() => new CampaignSession(client, 'a', store));
});
test('campaign wire schema rejects untrusted projection properties and invalid version', () => {
  assert.equal(isCampaignSnapshot(snapshot()), true);
  assert.equal(isCampaignSnapshot({ ...snapshot(), revision: -1 }), false);
  assert.equal(isCampaignSnapshot({ ...snapshot(), schemaVersion: 'future' }), false);
  assert.equal(isCampaignSnapshot({ ...snapshot(), answerKey: 42 }), false);
});

test('confirmed write followed by failed read stays blocked until explicit refresh', async () => {
  let reads = 0,
    sends = 0;
  const session = new CampaignSession(
    {
      snapshot: async () => {
        reads++;
        if (reads === 2) throw new ApiFailure('UNAVAILABLE', 0);
        return snapshot(reads === 1 ? 3 : 9);
      },
      send: async () => {
        sends++;
        return snapshot(4);
      },
    },
    'refresh-profile',
    storage(),
    () => 'd65e137d-e686-4070-aa1c-bea1b5ff8f47',
  );
  await session.refresh();
  await session.choose({ type: 'adopt' });
  await assert.rejects(session.refresh());
  assert.equal(session.hasPending, false);
  assert.equal(session.needsRefresh, true);
  await assert.rejects(session.choose({ type: 'adopt' }));
  await session.refresh();
  assert.equal(session.needsRefresh, false);
  assert.equal(session.snapshot?.revision, 9);
  assert.equal(sends, 1);
});
