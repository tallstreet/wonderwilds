import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { GameCommand, GameSnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure, type PreparedCommand } from './client.ts';
import { CommandSession } from './command-session.ts';

const snapshot = (revision = 4): GameSnapshot => ({
  schemaVersion: '1.0',
  revision,
  contentVersion: 'test',
  rulesVersion: 'test',
  storyFlags: [],
  ownedCreatures: [],
  inventory: [],
  pendingEncounters: [],
  learnerSummaries: [],
});
function harness() {
  const prepared: PreparedCommand[] = [];
  const sent: PreparedCommand[] = [];
  const outcomes = {
    send: async () => snapshot(5),
    refresh: async () => snapshot(7),
  };
  const client = {
    prepare(command: GameCommand) {
      const value = Object.freeze({
        body: JSON.stringify(command),
        commandId: command.commandId,
        context: 1,
      });
      prepared.push(value);
      return value;
    },
    send(command: PreparedCommand) {
      sent.push(command);
      return outcomes.send();
    },
    snapshot: () => outcomes.refresh(),
  };
  let sequence = 0;
  return {
    prepared,
    sent,
    outcomes,
    session: new CommandSession(client, snapshot(), () => `command-${++sequence}`),
  };
}

test('Uncertain retries retain the exact prepared object, bytes, ID and revision despite caller mutation', async () => {
  const { session, prepared, sent, outcomes } = harness();
  const payload = { type: 'select_team' as const, creatureIds: ['pip'] };
  outcomes.send = async () => {
    throw new ApiFailure('UNAVAILABLE', 503);
  };
  await assert.rejects(session.send(payload), ApiFailure);
  assert.equal(session.status, 'awaiting-retry');
  assert.equal(session.busy, false);
  assert.equal(session.hasPending, true);
  assert.equal(session.snapshot.revision, 4);
  payload.creatureIds.push('tansy');
  await assert.rejects(session.send(payload), /Retry the pending action first/);
  assert.equal(sent.length, 1);
  outcomes.send = async () => snapshot(5);
  await session.retry();
  assert.equal(prepared.length, 1);
  assert.equal(sent[0], sent[1]);
  assert.deepEqual(JSON.parse(sent[1].body), {
    schemaVersion: '1.0',
    commandId: 'command-1',
    expectedRevision: 4,
    payload: { type: 'select_team', creatureIds: ['pip'] },
  });
  assert.equal(session.snapshot.revision, 5);
  assert.equal(session.status, 'idle');
  assert.equal(session.hasPending, false);
  assert.equal(await session.retry(), undefined);
});

test('A repeated payload retries without preparing another command; unknown errors and rate limits remain pending', async () => {
  for (const error of [new Error('disconnected'), new ApiFailure('RATE_LIMITED', 429)]) {
    const { session, prepared, sent, outcomes } = harness();
    outcomes.send = async () => {
      throw error;
    };
    await assert.rejects(session.send({ type: 'abandon', encounterId: 'enc-1' }));
    outcomes.send = async () => snapshot(5);
    await session.send({ type: 'abandon', encounterId: 'enc-1' });
    assert.equal(prepared.length, 1);
    assert.equal(sent[0], sent[1]);
  }
});

test('In-flight sends reject overlapping sends, retries and refreshes', async () => {
  const { session, prepared, outcomes } = harness();
  let resolve!: (value: GameSnapshot) => void;
  outcomes.send = () =>
    new Promise((done) => {
      resolve = done;
    });
  const sending = session.send({ type: 'abandon', encounterId: 'enc-1' });
  assert.equal(session.status, 'sending');
  assert.equal(session.busy, true);
  await assert.rejects(session.send({ type: 'abandon', encounterId: 'enc-1' }), /still saving/);
  await assert.rejects(session.retry(), /still saving/);
  await assert.rejects(session.refresh(), /still saving/);
  assert.equal(prepared.length, 1);
  resolve(snapshot(5));
  await sending;
  assert.equal(session.status, 'idle');
});

test('Definitive conflicts discard the command without silently retrying at a new revision', async () => {
  const { session, prepared, sent, outcomes } = harness();
  outcomes.send = async () => {
    throw new ApiFailure('REVISION_CONFLICT', 409);
  };
  await assert.rejects(session.send({ type: 'abandon', encounterId: 'enc-1' }), ApiFailure);
  assert.equal(session.status, 'idle');
  assert.equal(session.hasPending, false);
  assert.equal(session.snapshot.revision, 4);
  assert.equal(await session.retry(), undefined);
  assert.equal(sent.length, 1);
  await session.refresh();
  outcomes.send = async () => snapshot(8);
  await session.send({ type: 'abandon', encounterId: 'enc-2' });
  assert.equal(JSON.parse(prepared[1].body).expectedRevision, 7);
  assert.equal(prepared[1].commandId, 'command-2');
});

test('Failed refresh preserves the exact uncertain command; successful explicit refresh reconciles it', async () => {
  const { session, prepared, outcomes } = harness();
  outcomes.send = async () => {
    throw new ApiFailure('UNAVAILABLE', 0);
  };
  await assert.rejects(session.send({ type: 'abandon', encounterId: 'enc-1' }));
  let reject!: (reason: Error) => void;
  outcomes.refresh = () =>
    new Promise((_done, fail) => {
      reject = fail;
    });
  const refreshing = session.refresh();
  assert.equal(session.status, 'refreshing');
  assert.equal(session.hasPending, true);
  await assert.rejects(session.send({ type: 'abandon', encounterId: 'enc-1' }), /still saving/);
  reject(new Error('offline'));
  await assert.rejects(refreshing, /offline/);
  assert.equal(session.status, 'awaiting-retry');
  assert.equal(session.snapshot.revision, 4);
  assert.equal(prepared.length, 1);
  outcomes.refresh = async () => snapshot(7);
  await session.refresh();
  assert.equal(session.snapshot.revision, 7);
  assert.equal(session.status, 'idle');
  assert.equal(session.hasPending, false);
  assert.equal(await session.retry(), undefined);
});
