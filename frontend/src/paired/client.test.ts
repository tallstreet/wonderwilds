import assert from 'node:assert/strict';
import test from 'node:test';
import type { PairedSnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import { PairedClient, PairedSession, type PendingStore } from './client.ts';
import { allowedProfile, profileLabel, selectedProfileUrl } from './panel.ts';
import { isPairedSnapshot, isHouseholdProfilesSnapshot } from './validation.ts';
const actor = '00000000-0000-4000-8000-000000000001',
  partner = '00000000-0000-4000-8000-000000000002';
const snapshot = (revision = 1): PairedSnapshot => ({
  schemaVersion: '1.0',
  revision,
  actorProfileId: actor,
  mode: 'solo',
  offers: [],
  confirmedProfileIds: [],
  exchangeComplete: false,
  companions: [],
  availableCompanions: [],
  availableKeepsakeIds: [],
  completedPractices: 0,
  legalMovements: [],
  practiceTarget: 6,
  practicePressureLimit: 5,
  practiceTurnLimit: 8,
  keepsakes: [],
});
const storage = (): PendingStore => {
  const values = new Map<string, string>();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value);
    },
    removeItem: (key) => {
      values.delete(key);
    },
  };
};
test('paired uncertain offer survives reload with exact actor-scoped command and fresh receipt reconciliation', async () => {
  const bodies: string[] = [],
    store = storage();
  const client = {
    snapshot: async () => snapshot(8),
    send: async (body: string) => {
      bodies.push(body);
      if (bodies.length === 1) throw new ApiFailure('UNAVAILABLE', 0);
      return snapshot(4);
    },
  };
  const first = new PairedSession(
    client,
    actor,
    store,
    () => 'd65e137d-e686-4070-aa1c-bea1b5ff8f47',
  );
  await first.refresh();
  await assert.rejects(first.choose({ type: 'offer_keepsake', keepsakeId: 'ribbon' }));
  const other = new PairedSession(client, partner, store);
  assert.equal(other.hasPending, false);
  const resumed = new PairedSession(client, actor, store);
  await resumed.refresh();
  assert.equal(resumed.hasPending, true);
  await resumed.retry();
  assert.equal(bodies[0], bodies[1]);
  assert.equal(resumed.needsRefresh, true);
  await assert.rejects(resumed.choose({ type: 'continue_solo' }));
  await resumed.refresh();
  assert.equal(resumed.snapshot?.revision, 8);
});
test('paired identity comes only from selected profile header and mismatched actor response fails closed', async () => {
  const calls: { url: string; headers: Headers; method?: string }[] = [];
  let wrong = false;
  const client = new PairedClient(actor, 'synthetic-csrf', async (url, init) => {
    calls.push({ url: String(url), headers: new Headers(init?.headers), method: init?.method });
    return new Response(
      JSON.stringify({ ...snapshot(), actorProfileId: wrong ? partner : actor }),
      { status: 200 },
    );
  });
  await client.snapshot();
  assert.equal(calls[0]?.headers.get('X-Wonderwilds-Profile'), actor);
  wrong = true;
  await assert.rejects(client.snapshot(), ApiFailure);
  client.destroy();
  await assert.rejects(client.snapshot(), ApiFailure);
});
test('paired definitive rejection unblocks refresh while uncertain auth failure preserves original intent', async () => {
  for (const code of ['INVALID_INPUT', 'REVISION_CONFLICT', 'FORBIDDEN'] as const) {
    const session = new PairedSession(
      {
        snapshot: async () => snapshot(),
        send: async () => {
          throw new ApiFailure(code, 400);
        },
      },
      actor,
      storage(),
    );
    await session.refresh();
    await assert.rejects(session.choose({ type: 'start', partnerProfileId: partner }));
    assert.equal(session.hasPending, code === 'FORBIDDEN');
  }
});
test('household explicit selection rejects foreign or missing profile instead of assigning answers to default', () => {
  const profiles = [
    { id: actor, abilityBand: 'beginning' as const },
    { id: partner, abilityBand: 'growing' as const },
  ];
  assert.equal(allowedProfile(partner, actor, profiles), partner);
  assert.equal(allowedProfile(null, actor, []), actor);
  assert.throws(() => allowedProfile(partner, actor, []));
  assert.throws(() => allowedProfile('foreign', actor, profiles));
  assert.equal(profileLabel(profiles, partner), 'Player 2 · 00000002');
  assert.equal(profileLabel([...profiles].reverse(), partner), 'Player 1 · 00000002');
  assert.equal(profileLabel([profiles[1]!], partner), 'Player 1 · 00000002');
  assert.equal(profileLabel([profiles[0]!], partner), 'Player not listed · 00000002');
  assert.equal(
    new URL(
      selectedProfileUrl('http://127.0.0.1:5173/?adventure=earlier', partner),
    ).searchParams.get('adventure'),
    'earlier',
  );
});
test('public paired schemas reject unknown fields and do not accept client-owned learning evidence', () => {
  assert.equal(isPairedSnapshot(snapshot()), true);
  assert.equal(isPairedSnapshot({ ...snapshot(), learningXp: 10 }), false);
  assert.equal(
    isHouseholdProfilesSnapshot({
      schemaVersion: '1.0',
      profiles: [{ id: actor, abilityBand: 'beginning' }],
    }),
    true,
  );
  assert.equal(
    isHouseholdProfilesSnapshot({
      schemaVersion: '1.0',
      profiles: [{ id: actor, abilityBand: 'made-up' }],
    }),
    false,
  );
});
