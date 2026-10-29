import assert from 'node:assert/strict';
import { test } from 'node:test';
import { GameClient, ApiFailure } from './client.ts';
import { importLegacy, inspectLegacy, originalSaveBlob } from './migration.ts';
import { freshState, SAVE_KEY } from '../progress.ts';
import type { GameCommand } from '../../../contracts/generated/contracts.ts';
const snapshot = {
  schemaVersion: '1.0',
  revision: 1,
  contentVersion: 'test',
  rulesVersion: '1',
  storyFlags: [],
  ownedCreatures: [],
  inventory: [],
  pendingEncounters: [],
  learnerSummaries: [],
};
const command = (): GameCommand => ({
  schemaVersion: '1.0',
  commandId: 'retry-same-id',
  expectedRevision: 0,
  payload: { type: 'start_encounter', beatId: 'meadow-bridge' },
});
const storage = (raw: string | null) => {
  const values = new Map<string, string>();
  if (raw !== null) values.set(SAVE_KEY, raw);
  return {
    values,
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
  };
};
test('API base rejects browser-normalized cross-origin paths and control characters', () => {
  const fetcher: typeof fetch = async () => Response.json(snapshot);
  for (const base of [
    '/\\evil.example',
    '//evil.example',
    '/\t/evil.example',
    '/\n/evil.example',
    'https://evil.example/api',
    '/api?redirect=evil',
    '/api#fragment',
  ]) {
    assert.throws(() => new GameClient(() => 'csrf', fetcher, base), /same-origin path/);
  }
  assert.doesNotThrow(() => new GameClient(() => 'csrf', fetcher, '/api/v1'));
});
test('uncertain writes retry exact command bytes, include cookie/CSRF/profile context', async () => {
  const requests: RequestInit[] = [];
  const fetcher: typeof fetch = async (_input, init) => {
    requests.push(init ?? {});
    if (requests.length === 1) throw new Error('connection lost after commit');
    return Response.json(snapshot);
  };
  const client = new GameClient(() => 'csrf', fetcher);
  client.selectProfile('owned-profile');
  const prepared = client.prepare(command());
  await assert.rejects(
    client.send(prepared),
    (e: unknown) => e instanceof ApiFailure && e.code === 'UNAVAILABLE',
  );
  assert.equal((await client.send(prepared)).revision, 1);
  assert.equal(requests[0]?.body, requests[1]?.body);
  assert.equal(requests[1]?.credentials, 'same-origin');
  const sentHeaders = requests[1]?.headers;
  assert.ok(sentHeaders);
  assert.equal((sentHeaders as Record<string, string>)['X-CSRF-Token'], 'csrf');
  assert.equal((sentHeaders as Record<string, string>)['X-Wonderwilds-Profile'], 'owned-profile');
  const altered = command();
  altered.expectedRevision = 1;
  assert.throws(
    () => client.prepare(altered),
    (e: unknown) => e instanceof ApiFailure && e.code === 'REVISION_CONFLICT',
  );
});
test('revision conflict is surfaced without automatic overwrite or retry', async () => {
  let calls = 0;
  const client = new GameClient(
    () => 'csrf',
    async () => {
      calls++;
      return Response.json(
        { schemaVersion: '1.0', code: 'REVISION_CONFLICT', correlationId: 'safe-id' },
        { status: 409 },
      );
    },
  );
  await assert.rejects(
    client.send(client.prepare(command())),
    (e: unknown) =>
      e instanceof ApiFailure && e.code === 'REVISION_CONFLICT' && e.correlationId === 'safe-id',
  );
  assert.equal(calls, 1);
});
test('profile switch aborts requests and rejects their late results and prepared writes', async () => {
  let finish: (response: Response) => void = () => {};
  const client = new GameClient(
    () => 'csrf',
    async () =>
      new Promise<Response>((resolve) => {
        finish = resolve;
      }),
  );
  const prepared = client.prepare(command());
  const pending = client.snapshot();
  client.selectProfile('other-profile');
  finish(Response.json(snapshot));
  await assert.rejects(
    pending,
    (e: unknown) => e instanceof ApiFailure && e.code === 'CONTEXT_CHANGED',
  );
  await assert.rejects(
    client.send(prepared),
    (e: unknown) => e instanceof ApiFailure && e.code === 'CONTEXT_CHANGED',
  );
});
test('profile switches and logout discard deferred response bodies, including error bodies', async () => {
  for (const action of ['profile', 'logout'] as const) {
    for (const ok of [true, false]) {
      let finishBody: (value: unknown) => void = () => {};
      let bodyStarted: () => void = () => {};
      const started = new Promise<void>((resolve) => {
        bodyStarted = resolve;
      });
      const body = new Promise<unknown>((resolve) => {
        finishBody = resolve;
      });
      const response = new Response(null, { status: ok ? 200 : 409 });
      response.json = () => {
        bodyStarted();
        return body;
      };
      const client = new GameClient(
        () => 'csrf',
        async () => response,
      );
      client.selectProfile('first-profile');
      const pending = client.snapshot();
      await started;
      if (action === 'profile') client.selectProfile('second-profile');
      else client.clearContext();
      finishBody(ok ? snapshot : { code: 'REVISION_CONFLICT', correlationId: 'old-profile' });
      await assert.rejects(
        pending,
        (error: unknown) => error instanceof ApiFailure && error.code === 'CONTEXT_CHANGED',
        `${action} must discard ${ok ? 'successful' : 'error'} body from the old profile`,
      );
    }
  }
});
test('caller mutation after preparation cannot change exact command bytes', async () => {
  const bodies: (BodyInit | null | undefined)[] = [];
  const client = new GameClient(
    () => 'csrf',
    async (_input, init) => {
      bodies.push(init?.body);
      return Response.json(snapshot);
    },
  );
  const original = command();
  const prepared = client.prepare(original);
  const originalBytes = prepared.body;
  original.expectedRevision = 99;
  original.payload = { type: 'start_encounter', beatId: 'different-beat' };
  assert.ok(Object.isFrozen(prepared));
  await client.send(prepared);
  await client.send(prepared);
  assert.deepEqual(bodies, [originalBytes, originalBytes]);
  assert.throws(
    () => client.prepare(original),
    (error: unknown) => error instanceof ApiFailure && error.code === 'REVISION_CONFLICT',
  );
});
test('malformed saves remain untouched and downloadable; blocked storage is explicit', async () => {
  const target = storage('{broken');
  const result = await inspectLegacy(target);
  assert.equal(result.status, 'corrupt');
  assert.equal(target.getItem(SAVE_KEY), '{broken');
  if (result.status === 'corrupt') assert.equal(await originalSaveBlob(result).text(), '{broken');
  assert.equal(
    (
      await inspectLegacy({
        getItem: () => {
          throw new Error('blocked');
        },
        setItem: () => {},
      })
    ).status,
    'storage_unavailable',
  );
});
test('migration keeps original evidence, unverified provenance and one acknowledged receipt', async () => {
  const original = JSON.stringify(freshState());
  const target = storage(original);
  const result = await inspectLegacy(target);
  assert.equal(result.status, 'available');
  if (result.status !== 'available') return;
  assert.equal(result.data.evidenceProvenance, 'legacy_unverified');
  assert.equal(result.data.skillMapping['missing-10'], 'M06');
  let calls = 0;
  const transport = async () => {
    calls++;
    return {
      receiptId: 'server-issued',
      profileId: 'parent-owned',
      sourceDigest: result.data.digest,
      evidenceProvenance: 'legacy_unverified' as const,
    };
  };
  const first = await importLegacy(result.data, 'parent-owned', target, transport);
  assert.equal(first.persisted, true);
  assert.equal(
    (await importLegacy(result.data, 'parent-owned', target, transport)).alreadyImported,
    true,
  );
  assert.equal(calls, 1);
  assert.equal(target.getItem(SAVE_KEY), original);
});
test('failed or mismatched acknowledgement never creates a receipt or erases source', async () => {
  const original = JSON.stringify(freshState());
  const target = storage(original);
  const result = await inspectLegacy(target);
  if (result.status !== 'available') throw new Error('fixture unavailable');
  await assert.rejects(
    importLegacy(result.data, 'owner', target, async () => ({
      receiptId: 'id',
      profileId: 'other',
      sourceDigest: result.data.digest,
      evidenceProvenance: 'legacy_unverified',
    })),
  );
  assert.equal(target.values.size, 1);
  assert.equal(target.getItem(SAVE_KEY), original);
  const blocked = {
    getItem: target.getItem,
    setItem: () => {
      throw new Error('quota');
    },
  };
  const outcome = await importLegacy(result.data, 'owner', blocked, async () => ({
    receiptId: 'recoverable',
    profileId: 'owner',
    sourceDigest: result.data.digest,
    evidenceProvenance: 'legacy_unverified',
  }));
  assert.equal(outcome.persisted, false);
  assert.equal(outcome.receipt.receiptId, 'recoverable');
});

test('character state survives reload and malformed optional progression fails closed', async () => {
  const character = {
    friendship: [{ creatureId: 'pip', points: 3 }],
    transformedCreatures: ['pip-glow'],
    availableHelpingMoments: [{ creatureId: 'pip', completedQuestId: 'meadow-bridge' }],
  };
  const read = async (fields: object) => {
    const client = new GameClient(
      () => 'csrf',
      async () => Response.json({ ...snapshot, ...fields }),
    );
    client.selectProfile('owned-profile');
    return client.snapshot();
  };
  assert.deepEqual((await read(character)).friendship, character.friendship);
  assert.equal((await read({})).friendship, undefined);
  for (const fields of [
    { friendship: [{ creatureId: 'pip', points: -1 }] },
    { friendship: [{ creatureId: 'pip', points: 7 }] },
    { friendship: [{ creatureId: 'pip', points: 2.5 }] },
    {
      friendship: [
        { creatureId: 'pip', points: 1 },
        { creatureId: 'pip', points: 2 },
      ],
    },
    { transformedCreatures: [true] },
    { transformedCreatures: ['pip-glow', 'pip-glow'] },
    { availableHelpingMoments: [{ creatureId: 'pip' }] },
    { availableHelpingMoments: null },
  ]) {
    await assert.rejects(
      read(fields),
      (error: unknown) => error instanceof ApiFailure && error.code === 'UNAVAILABLE',
    );
  }
});

test('training projections validate bounds while older snapshots remain readable', async () => {
  const progression = {
    creatureId: 'pip',
    xp: 10,
    level: 2,
    nextLevelXp: 30,
    reactionId: 'pip-lantern-cheer',
    completedTrainingIds: ['training-pip-1'],
    availableTraining: [
      { trainingId: 'training-pip-1', requiredLevel: 1, xp: 10, reactionId: 'pip-lantern-cheer' },
    ],
  };
  const read = async (fields: object) => {
    const client = new GameClient(
      () => 'csrf',
      async () => Response.json({ ...snapshot, ...fields }),
    );
    client.selectProfile('owned-profile');
    return client.snapshot();
  };
  assert.deepEqual(
    (
      await read({
        characterProgression: [progression],
        explorerProgression: { xp: 20, level: 2 },
        teamCapacity: 2,
      })
    ).characterProgression,
    [progression],
  );
  for (const fields of [
    { characterProgression: [{ ...progression, xp: -1 }] },
    { characterProgression: [{ ...progression, level: 6 }] },
    { characterProgression: [progression, progression] },
    { characterProgression: [{ ...progression, completedTrainingIds: undefined }] },
    {
      characterProgression: [
        { ...progression, completedTrainingIds: ['training-pip-1', 'training-pip-1'] },
      ],
    },
    {
      characterProgression: [
        { ...progression, availableTraining: [{ trainingId: 'training-pip-1', xp: 10 }] },
      ],
    },
    { explorerProgression: { xp: -1, level: 1 } },
    { teamCapacity: 3 },
  ])
    await assert.rejects(
      read(fields),
      (error: unknown) => error instanceof ApiFailure && error.code === 'UNAVAILABLE',
    );
});

test('personal quest projections validate bounded scenes and pinned encounter bindings', async () => {
  const offer = {
    questId: 'pip-lantern-trail',
    documentVersion: '1',
    title: 'A trail of little lights',
    companionId: 'pip',
    minimumCharacterLevel: 4,
    available: true,
    completed: false,
  };
  const personal = {
    questId: offer.questId,
    documentVersion: '1',
    episodeId: 'episode-1',
    title: offer.title,
    companionId: 'pip',
    node: {
      id: 'start',
      type: 'choice',
      text: 'Which path shall we follow?',
      choices: [
        { id: 'pond', label: 'The pond', nextNodeId: 'path-one' },
        { id: 'hill', label: 'The hill', nextNodeId: 'path-two' },
      ],
    },
  };
  const read = async (fields: object) => {
    const client = new GameClient(
      () => 'csrf',
      async () => Response.json({ ...snapshot, ...fields }),
    );
    client.selectProfile('owned-profile');
    return client.snapshot();
  };
  assert.deepEqual(
    (await read({ personalQuestOffers: [offer], personalQuest: personal })).personalQuest,
    personal,
  );
  assert.equal((await read({})).personalQuest, undefined);
  for (const fields of [
    { personalQuestOffers: [offer, offer] },
    { personalQuestOffers: [{ ...offer, minimumCharacterLevel: 0 }] },
    { personalQuestOffers: [{ ...offer, available: 'yes' }] },
    { personalQuestOffers: [{ ...offer, rewardXp: 100 }] },
    { personalQuest: { ...personal, node: { ...personal.node, text: 'x'.repeat(601) } } },
    {
      personalQuest: {
        ...personal,
        node: { ...personal.node, choices: [personal.node.choices[0]] },
      },
    },
    {
      personalQuest: {
        ...personal,
        node: { ...personal.node, choices: [personal.node.choices[0], personal.node.choices[0]] },
      },
    },
    {
      personalQuest: {
        ...personal,
        node: { id: 'done', type: 'ending', text: 'The game is over' },
      },
    },
    {
      personalQuest: {
        ...personal,
        node: {
          id: 'next',
          type: 'handoff',
          text: 'More adventures await',
          continuationTags: ['stars', 'stars'],
        },
      },
    },
    {
      personalQuest: personal,
      pendingEncounters: [
        {
          personalQuest: {
            questId: offer.questId,
            documentVersion: '1',
            episodeId: 'episode-other',
            nodeId: 'start',
          },
        },
      ],
    },
  ])
    await assert.rejects(
      read(fields),
      (error: unknown) => error instanceof ApiFailure && error.code === 'UNAVAILABLE',
    );
});
