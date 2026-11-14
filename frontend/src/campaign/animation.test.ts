import assert from 'node:assert/strict';
import test from 'node:test';
import { campaignAnimationFrame, campaignAnimationSheet } from './animation.ts';
import type { Facing } from '../types.ts';

test('Pip motion metadata covers twenty distinct native frames with grounded anchors', () => {
  const { frames, dimensions } = campaignAnimationSheet;
  assert.equal(frames.length, 20);
  assert.equal(new Set(frames.map((frame) => frame.id)).size, 20);
  for (const frame of frames) {
    const [x, y, w, h] = frame.rect;
    assert.ok(x >= 0 && y >= 0 && w > 0 && h > 0);
    assert.ok(x + w <= dimensions[0]! && y + h <= dimensions[1]!);
    assert.ok(frame.foot[0]! > 0 && frame.foot[0]! < w);
    assert.ok(frame.foot[1]! > 0 && frame.foot[1]! < h);
  }
});
test('Each heading has its own idle and two different actual stride frames', () => {
  for (const facing of ['down', 'right', 'up', 'left'] as Facing[]) {
    assert.equal(campaignAnimationFrame(facing, 0, false, false), `idle-${facing}`);
    assert.equal(campaignAnimationFrame(facing, 0.1, true, false), `walk-${facing}-a`);
    assert.equal(campaignAnimationFrame(facing, 0.6, true, false), `walk-${facing}-b`);
    assert.equal(campaignAnimationFrame(facing, 1.1, true, false), `walk-${facing}-a`);
    assert.equal(campaignAnimationFrame(facing, 0.6, true, true), `idle-${facing}`);
  }
});
test('Confirmed visual effects expire and reduced motion keeps a stable pose', () => {
  assert.equal(
    campaignAnimationFrame('down', 0, false, false, 'signature', 0),
    'signature-anticipate',
  );
  assert.equal(
    campaignAnimationFrame('down', 0, false, false, 'signature', 300),
    'signature-release',
  );
  assert.equal(
    campaignAnimationFrame('down', 0, false, false, 'signature', 600),
    'signature-settle',
  );
  assert.equal(campaignAnimationFrame('up', 0, false, false, 'signature', 720), 'idle-up');
  assert.equal(campaignAnimationFrame('down', 0, false, false, 'cheer', 400), 'cheer-lift');
  assert.equal(campaignAnimationFrame('down', 0, false, false, 'rest', 1200), 'rest-asleep');
  assert.equal(campaignAnimationFrame('left', 0, false, true, 'signature', 300), 'idle-left');
  assert.equal(campaignAnimationFrame('down', 0, false, true, 'rest', 0), 'rest-asleep');
});

import { EventEmitter } from 'node:events';
import type Phaser from 'phaser';
import type { Actor } from '../types.ts';
import registry from '../../../content/campaign/art/motion-registry.v1.json';
import {
  syncCampaignAnimations,
  campaignAnimationCache,
  applyCampaignAnimation,
  startCampaignPose,
  preloadCampaignAnimation,
} from './animation.ts';
class Textures extends EventEmitter {
  images = new Map<string, Set<string>>();
  removed: string[] = [];
  exists(key: string) {
    return this.images.has(key);
  }
  get(key: string) {
    const frames = this.images.get(key)!;
    return { has: (id: string) => frames.has(id), add: (id: string) => frames.add(id) };
  }
  addImage(key: string) {
    this.images.set(key, new Set());
    this.emit('addtexture', key);
  }
  remove(key: string) {
    this.removed.push(key);
    this.images.delete(key);
  }
}
class Loader extends EventEmitter {
  queue: string[] = [];
  requested: string[] = [];
  loading = false;
  constructor(readonly textures: Textures) {
    super();
  }
  image(key: string, _url: string) {
    this.queue.push(key);
    this.requested.push(key);
    return this;
  }
  isLoading() {
    return this.loading;
  }
  start() {
    this.loading = true;
  }
  complete() {
    const key = this.queue.shift()!;
    assert.ok(key);
    this.loading = false;
    this.textures.addImage(key);
    this.emit(`filecomplete-image-${key}`);
  }
  fail() {
    const key = this.queue.shift()!;
    this.loading = false;
    this.emit('loaderror', { key });
  }
}
function fakeScene(textures = new Textures()) {
  const load = new Loader(textures),
    events = new EventEmitter(),
    children: { list: unknown[] } = { list: [] };
  const scene = { textures, load, events, children } as unknown as Phaser.Scene;
  return { scene, textures, load, events, children };
}
const ids = registry.sheets.map((sheet) => sheet.character);
const keyFor = (id: string) => registry.sheets.find((sheet) => sheet.character === id)!.texture;
const flush = async () => {
  for (let i = 0; i < 5; i++) await Promise.resolve();
};
async function drain(fake: ReturnType<typeof fakeScene>) {
  for (let i = 0; i < 100; i++) {
    await flush();
    if (!fake.load.queue.length) return;
    fake.load.complete();
    assert.ok(fake.textures.images.size <= 4);
  }
  throw new Error('motion queue did not settle');
}

test('Preload is active-only and all registered metadata preserves its measured dimensions and anchors', () => {
  const fake = fakeScene();
  preloadCampaignAnimation(fake.scene, [ids[0]!, 'unknown']);
  assert.deepEqual(fake.load.requested, [keyFor(ids[0]!)]);
  for (const sheet of registry.sheets) {
    assert.ok(sheet.nominalCell > 0);
    assert.equal(new Set(sheet.frames.map((frame) => frame.id)).size, 20);
    for (const frame of sheet.frames) {
      const [x, y, w, h] = frame.rect;
      assert.ok(x! + w! <= sheet.dimensions[0]! && y! + h! <= sheet.dimensions[1]!);
      assert.ok(
        frame.foot[0]! >= 0 && frame.foot[0]! <= w! && frame.foot[1]! >= 0 && frame.foot[1]! <= h!,
      );
    }
  }
});
test('Deferred team switches skip superseded requests and resident motion textures stay bounded', async () => {
  assert.ok(ids.length >= 6);
  const fake = fakeScene();
  const first = syncCampaignAnimations(fake.scene, [ids[0]!]);
  const second = syncCampaignAnimations(fake.scene, [ids[1]!]);
  await first;
  assert.deepEqual(campaignAnimationCache(fake.scene).pending, [keyFor(ids[0]!)]);
  fake.load.complete();
  await flush();
  assert.deepEqual(fake.load.queue, [keyFor(ids[1]!)]);
  await drain(fake);
  await second;
  const old = syncCampaignAnimations(fake.scene, [ids[2]!]);
  void syncCampaignAnimations(fake.scene, [ids[3]!]);
  const latest = syncCampaignAnimations(fake.scene, [ids[4]!]);
  await old;
  await drain(fake);
  await latest;
  assert.ok(!fake.load.requested.includes(keyFor(ids[3]!)));
  const final = syncCampaignAnimations(fake.scene, [ids[5]!]);
  await drain(fake);
  await final;
  assert.ok(campaignAnimationCache(fake.scene).keys.length <= 4);
});
test('Live images pin old textures until paint removes actors; full pinned cache uses static fallback', async () => {
  const fake = fakeScene();
  for (const id of ids.slice(0, 4)) {
    const pending = syncCampaignAnimations(fake.scene, [id]);
    await drain(fake);
    await pending;
  }
  const keys = campaignAnimationCache(fake.scene).keys;
  fake.children.list = [{ list: keys.map((key) => ({ active: true, texture: { key } })) }];
  const before = [...fake.textures.images.keys()];
  await syncCampaignAnimations(fake.scene, [ids[4]!]);
  assert.deepEqual([...fake.textures.images.keys()], before);
  assert.equal(fake.load.queue.length, 0);
  fake.children.list = [];
  const pending = syncCampaignAnimations(fake.scene, [ids[4]!]);
  await drain(fake);
  await pending;
  assert.ok(fake.textures.exists(keyFor(ids[4]!)));
});
test('Shutdown resolves pending callers and stale image completion cannot resurrect an old scene', async () => {
  const old = fakeScene();
  const cancelled = syncCampaignAnimations(old.scene, [ids[0]!]);
  old.events.emit('shutdown');
  await cancelled;
  const current = fakeScene(old.textures);
  const pending = syncCampaignAnimations(current.scene, [ids[1]!]);
  old.load.complete();
  await flush();
  assert.equal(old.textures.exists(keyFor(ids[0]!)), false);
  await drain(current);
  await pending;
  assert.deepEqual(campaignAnimationCache(current.scene).pending, []);
});
test('A late cancelled texture wanted by a new owner cannot displace four live images or exceed resident budget', async () => {
  const old = fakeScene();
  const cancelled = syncCampaignAnimations(old.scene, [ids[0]!]);
  old.events.emit('shutdown');
  await cancelled;
  const current = fakeScene(old.textures);
  for (const id of ids.slice(1, 5)) {
    const pending = syncCampaignAnimations(current.scene, [id]);
    await drain(current);
    await pending;
  }
  const keys = campaignAnimationCache(current.scene).keys;
  current.children.list = keys.map((key) => ({ active: true, texture: { key } }));
  await syncCampaignAnimations(current.scene, [ids[0]!]);
  old.load.complete();
  await flush();
  assert.equal(current.textures.images.size, 4);
  assert.equal(current.textures.exists(keyFor(ids[0]!)), false);
  for (const key of keys) assert.ok(current.textures.exists(key));
});
test('Image errors produce a stable static fallback; selecting away permits a deliberate retry', async () => {
  const fake = fakeScene();
  const first = syncCampaignAnimations(fake.scene, [ids[0]!]);
  fake.load.fail();
  await flush();
  await first;
  await syncCampaignAnimations(fake.scene, [ids[0]!]);
  assert.equal(fake.load.requested.length, 1);
  await syncCampaignAnimations(fake.scene, []);
  const retry = syncCampaignAnimations(fake.scene, [ids[0]!]);
  await drain(fake);
  await retry;
  assert.equal(fake.load.requested.length, 2);
});
test('Native metadata anchors the child at its logical ground and reduced motion is read on every frame', async () => {
  const fake = fakeScene();
  const id = ids[1]!;
  const pending = syncCampaignAnimations(fake.scene, [id]);
  await drain(fake);
  await pending;
  const values = { texture: '', frame: '', origin: [0, 0], size: [0, 0], position: [0, 0] };
  const art = {
    setTexture(texture: string, frame: string) {
      values.texture = texture;
      values.frame = frame;
      return this;
    },
    setFlipX(_v: boolean) {
      return this;
    },
    setOrigin(x: number, y: number) {
      values.origin = [x, y];
      return this;
    },
    setDisplaySize(w: number, h: number) {
      values.size = [w, h];
      return this;
    },
    setPosition(x: number, y: number) {
      values.position = [x, y];
      return this;
    },
    setAngle(_v: number) {
      return this;
    },
  };
  const actor = {
    x: 72,
    y: 128,
    heroId: id,
    facing: 'down',
    phase: 0,
    baseWidth: 78,
    baseHeight: 87,
    art,
    scene: fake.scene,
  } as unknown as Actor;
  assert.equal(
    applyCampaignAnimation(actor, { dx: 1, dy: 0, distance: 2, now: 100, reducedMotion: false }),
    true,
  );
  const sheet = registry.sheets.find((sheet) => sheet.character === id)!;
  const frame = sheet.frames.find((frame) => frame.id === values.frame)!;
  assert.deepEqual(values.origin, [
    frame.foot[0]! / frame.rect[2]!,
    frame.foot[1]! / frame.rect[3]!,
  ]);
  assert.deepEqual(values.position, [0, 0]);
  assert.equal(actor.x, 72);
  assert.equal(actor.y, 128);
  startCampaignPose(actor, 'cheer', 200);
  applyCampaignAnimation(actor, { dx: 0, dy: 0, distance: 0, now: 250, reducedMotion: true });
  assert.equal(values.frame, 'idle-right');
  fake.textures.remove(keyFor(id));
  const before = JSON.stringify(values);
  assert.equal(
    applyCampaignAnimation(actor, { dx: 0, dy: 0, distance: 0, now: 260, reducedMotion: false }),
    false,
  );
  assert.equal(JSON.stringify(values), before);
});

test('Restarting the same scene drops old loader callbacks and resumes only its latest identity', async () => {
  const fake = fakeScene();
  const old = syncCampaignAnimations(fake.scene, [ids[0]!]);
  fake.events.emit('shutdown');
  await old;
  const current = syncCampaignAnimations(fake.scene, [ids[1]!]);
  fake.load.complete();
  await flush();
  assert.equal(fake.textures.exists(keyFor(ids[0]!)), false);
  await drain(fake);
  await current;
  assert.ok(fake.textures.exists(keyFor(ids[1]!)));
  assert.deepEqual(campaignAnimationCache(fake.scene).pending, []);
});
