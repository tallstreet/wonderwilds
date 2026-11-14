import type Phaser from 'phaser';
import type { Actor, Facing } from '../types.ts';
import registry from '../../../content/campaign/art/motion-registry.v1.json';
import pilot from '../../../content/campaign/art/pip-base-motion-v1.frames.json';
interface Sheet {
  character: string;
  texture: string;
  url: string;
  nominalCell: number;
  frames: { id: string; rect: number[]; foot: number[] }[];
}
const sheets = registry.sheets as Sheet[];
const byActor = new Map(sheets.map((sheet) => [sheet.character, sheet]));
const byTexture = new Map(sheets.map((sheet) => [sheet.texture, sheet]));
export const campaignAnimationSheet = pilot;
export const maximumCampaignMotionTextures = 4;
export type CampaignPose = 'signature' | 'cheer' | 'rest';
const effects: Record<CampaignPose, { frames: string[]; duration: number }> = {
  signature: {
    frames: ['signature-anticipate', 'signature-release', 'signature-settle'],
    duration: 720,
  },
  cheer: { frames: ['cheer-prepare', 'cheer-lift', 'cheer-settle'], duration: 900 },
  rest: { frames: ['rest-settle', 'rest-asleep'], duration: 1800 },
};
const poses = new WeakMap<Actor, { pose: CampaignPose; started: number }>();
interface Owner {
  scene: Phaser.Scene;
  cache: Cache;
  desired: Set<string>;
  failed: Set<string>;
  disposed: boolean;
  waiters: (() => void)[];
}
interface Job {
  owner: Owner;
  cleanup(): void;
}
interface Cache {
  textures: Phaser.Textures.TextureManager;
  owners: Set<Owner>;
  pending: Map<string, Job>;
  preloads: Map<string, Owner>;
  cancelled: Set<string>;
  used: Map<string, number>;
  clock: number;
  scheduled: boolean;
}
const owners = new WeakMap<Phaser.Scene, Owner>();
const caches = new WeakMap<Phaser.Textures.TextureManager, Cache>();
function loaded(cache: Cache) {
  return [...byTexture.keys()].filter((key) => cache.textures.exists(key));
}
function wanted(cache: Cache, key: string) {
  return [...cache.owners].some((owner) => !owner.disposed && owner.desired.has(key));
}
/** A texture can still be used by an image while a replacement team is painted. */
function pinned(cache: Cache, key: string) {
  const scenes =
    cache.textures.game?.scene?.scenes ?? [...cache.owners].map((owner) => owner.scene);
  const visit = (objects: unknown[]): boolean =>
    objects.some((entry) => {
      if (!entry || typeof entry !== 'object') return false;
      const object = entry as { active?: boolean; texture?: { key?: string }; list?: unknown[] };
      if (object.active === false) return false;
      return object.texture?.key === key || (Array.isArray(object.list) && visit(object.list));
    });
  return scenes.some((scene) => visit(scene.children?.list ?? []));
}
function registerFrameArt(scene: Pick<Phaser.Scene, 'textures'>, sheet: Sheet) {
  if (!scene.textures.exists(sheet.texture)) return;
  const texture = scene.textures.get(sheet.texture);
  for (const frame of sheet.frames)
    if (!texture.has(frame.id)) {
      const [x, y, width, height] = frame.rect;
      texture.add(frame.id, 0, x!, y!, width!, height!);
    }
}
function evict(cache: Cache, reserve = 0) {
  const candidates = loaded(cache)
    .filter((key) => !wanted(cache, key) && !pinned(cache, key))
    .sort((a, b) => (cache.used.get(a) ?? 0) - (cache.used.get(b) ?? 0));
  while (loaded(cache).length + reserve > maximumCampaignMotionTextures && candidates.length) {
    const key = candidates.shift()!;
    cache.textures.remove(key);
    cache.used.delete(key);
  }
  return loaded(cache).length + reserve <= maximumCampaignMotionTextures;
}
function resolve(owner: Owner) {
  for (const done of owner.waiters.splice(0)) done();
}
function schedule(cache: Cache) {
  if (cache.scheduled) return;
  cache.scheduled = true;
  queueMicrotask(() => {
    cache.scheduled = false;
    pump(cache);
  });
}
function ownerFor(scene: Phaser.Scene): Owner {
  const previous = owners.get(scene);
  if (previous && !previous.disposed) return previous;
  let cache = caches.get(scene.textures);
  if (!cache) {
    cache = {
      textures: scene.textures,
      owners: new Set(),
      pending: new Map(),
      preloads: new Map(),
      cancelled: new Set(),
      used: new Map(),
      clock: 0,
      scheduled: false,
    };
    caches.set(scene.textures, cache);
    const shared = cache;
    scene.textures.on('addtexture', (key: string) => {
      const sheet = byTexture.get(key);
      if (!sheet) return;
      shared.preloads.delete(key);
      registerFrameArt({ textures: shared.textures }, sheet);
      if (shared.cancelled.has(key) && !wanted(shared, key) && !pinned(shared, key))
        shared.textures.remove(key);
      shared.cancelled.delete(key);
      evict(shared);
      // A cancelled network request may finish after another owner fills the budget.
      // The newly added texture has no live image yet; preserve existing live users.
      if (loaded(shared).length > maximumCampaignMotionTextures && !pinned(shared, key))
        shared.textures.remove(key);
      schedule(shared);
    });
  }
  const owner: Owner = {
    scene,
    cache,
    desired: new Set(),
    failed: new Set(),
    disposed: false,
    waiters: [],
  };
  owners.set(scene, owner);
  cache.owners.add(owner);
  const loadError = (file: { key: string }) => {
    if (cache.preloads.get(file.key) !== owner) return;
    cache.preloads.delete(file.key);
    owner.failed.add(file.key);
    schedule(cache);
  };
  scene.load.on('loaderror', loadError);
  const dispose = () => {
    if (owner.disposed) return;
    owner.disposed = true;
    owner.desired.clear();
    cache.owners.delete(owner);
    scene.load.off('loaderror', loadError);
    for (const [key, job] of cache.pending)
      if (job.owner === owner) {
        job.cleanup();
        cache.pending.delete(key);
        cache.cancelled.add(key);
      }
    for (const [key, source] of cache.preloads)
      if (source === owner) {
        cache.preloads.delete(key);
        cache.cancelled.add(key);
      }
    resolve(owner);
    evict(cache);
    schedule(cache);
  };
  scene.events.once('shutdown', dispose);
  scene.events.once('destroy', dispose);
  return owner;
}
function select(owner: Owner, ids: string[]) {
  const desired = new Set(
    ids
      .map((id) => byActor.get(id)?.texture)
      .filter((key): key is string => Boolean(key))
      .slice(0, maximumCampaignMotionTextures),
  );
  for (const key of owner.failed) if (!desired.has(key)) owner.failed.delete(key);
  owner.desired = desired;
  for (const key of desired) owner.cache.used.set(key, ++owner.cache.clock);
}
function pump(cache: Cache) {
  evict(cache);
  if (cache.pending.size || cache.preloads.size) return;
  for (const owner of cache.owners) {
    if (owner.disposed) continue;
    const key = [...owner.desired].find(
      (id) => !owner.scene.textures.exists(id) && !owner.failed.has(id),
    );
    if (!key) {
      resolve(owner);
      continue;
    }
    if (!evict(cache, 1)) {
      resolve(owner);
      continue;
    }
    const sheet = byTexture.get(key)!;
    const completeEvent = `filecomplete-image-${key}`;
    const cleanup = () => {
      owner.scene.load.off(completeEvent, complete);
      owner.scene.load.off('loaderror', error);
    };
    const finish = (failed: boolean) => {
      cleanup();
      cache.pending.delete(key);
      if (failed) owner.failed.add(key);
      else if (owner.scene.textures.exists(key)) registerFrameArt(owner.scene, sheet);
      else owner.failed.add(key);
      evict(cache);
      schedule(cache);
    };
    const complete = () => finish(false);
    const error = (file: { key: string }) => {
      if (file.key === key) finish(true);
    };
    cache.pending.set(key, { owner, cleanup });
    owner.scene.load.once(completeEvent, complete);
    owner.scene.load.on('loaderror', error);
    try {
      owner.scene.load.image(key, sheet.url);
      if (!owner.scene.load.isLoading()) owner.scene.load.start();
    } catch {
      finish(true);
    }
    return;
  }
}
/** Initial preload includes only the avatar and current travelling pair. */
export function preloadCampaignAnimation(scene: Phaser.Scene, activeIds: string[] = []) {
  const owner = ownerFor(scene);
  select(owner, activeIds);
  for (const key of owner.desired) {
    if (scene.textures.exists(key) || owner.cache.preloads.has(key) || owner.cache.pending.has(key))
      continue;
    if (!evict(owner.cache, owner.cache.preloads.size + 1)) break;
    owner.cache.preloads.set(key, owner);
    scene.load.image(key, byTexture.get(key)!.url);
  }
}
export function registerCampaignAnimation(scene: Phaser.Scene) {
  for (const sheet of sheets) registerFrameArt(scene, sheet);
}
/** Call after paint destroys old actors; obsolete completions never select frames or repaint. */
export function syncCampaignAnimations(scene: Phaser.Scene, activeIds: string[]): Promise<void> {
  const owner = ownerFor(scene);
  resolve(owner);
  select(owner, activeIds);
  const result = new Promise<void>((done) => owner.waiters.push(done));
  pump(owner.cache);
  return result;
}
export function campaignAnimationCache(scene: Phaser.Scene) {
  const owner = ownerFor(scene);
  return {
    keys: loaded(owner.cache),
    pending: [...owner.cache.pending.keys(), ...owner.cache.preloads.keys()],
  };
}
export function campaignAnimationFrame(
  facing: Facing,
  phase: number,
  moving: boolean,
  reducedMotion: boolean,
  pose?: CampaignPose,
  elapsed = 0,
) {
  if (pose && elapsed >= 0 && elapsed < effects[pose].duration) {
    const effect = effects[pose];
    return reducedMotion
      ? pose === 'rest'
        ? 'rest-asleep'
        : `idle-${facing}`
      : effect.frames[
          Math.min(
            effect.frames.length - 1,
            Math.floor((elapsed / effect.duration) * effect.frames.length),
          )
        ]!;
  }
  if (!moving || reducedMotion) return `idle-${facing}`;
  const cycle = Number.isFinite(phase) ? ((phase % 1) + 1) % 1 : 0;
  return `walk-${facing}-${cycle < 0.5 ? 'a' : 'b'}`;
}
export function startCampaignPose(actor: Actor, pose: CampaignPose, now: number) {
  if (byActor.has(actor.heroId) && Number.isFinite(now)) poses.set(actor, { pose, started: now });
}
export function applyCampaignAnimation(
  actor: Actor,
  options: { dx: number; dy: number; distance: number; now: number; reducedMotion: boolean },
) {
  const sheet = byActor.get(actor.heroId);
  if (!sheet || !actor.scene.textures.exists(sheet.texture)) return false;
  const { dx, dy, distance, now, reducedMotion } = options;
  const moving = Number.isFinite(distance) && distance > 0.01;
  if (moving) {
    actor.facing =
      Math.abs(dx) > Math.abs(dy) ? (dx < 0 ? 'left' : 'right') : dy < 0 ? 'up' : 'down';
    actor.phase = (((actor.phase + distance / 34) % 1) + 1) % 1;
    poses.delete(actor);
  } else actor.phase = 0;
  let effect = poses.get(actor);
  if (effect && (!Number.isFinite(now) || now - effect.started >= effects[effect.pose].duration)) {
    poses.delete(actor);
    effect = undefined;
  }
  const id = campaignAnimationFrame(
    actor.facing,
    actor.phase,
    moving,
    reducedMotion,
    effect?.pose,
    effect ? now - effect.started : 0,
  );
  const frame = sheet.frames.find((entry) => entry.id === id);
  if (!frame || !actor.scene.textures.get(sheet.texture).has(id)) return false;
  const [, , width, height] = frame.rect;
  const [footX, footY] = frame.foot;
  actor.art
    .setTexture(sheet.texture, id)
    .setFlipX(false)
    .setOrigin(footX! / width!, footY! / height!)
    .setDisplaySize(
      (actor.baseWidth * width!) / sheet.nominalCell,
      (actor.baseHeight * height!) / sheet.nominalCell,
    )
    .setPosition(0, 0)
    .setAngle(0);
  return true;
}
