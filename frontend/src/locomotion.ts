import type Phaser from 'phaser';
interface WalkSprite {
  scene: {
    textures: { exists(key: string): boolean; get(key: string): { has(frame: string): boolean } };
  };
  setTexture(key: string, frame: string): unknown;
}
/**
 * Distance-driven visual gait. Logical positions/collisions must remain at the
 * ground point; apply lift/rotation/scale only to its child artwork.
 *
 * phase = advanceGaitPhase(phase, actualDistance, species);
 * const pose = sampleGait(species: string, phase: number, moving: boolean, { reducedMotion });
 * art.y = -pose.lift;
 * art.setScale(baseScaleX * pose.scaleX, baseScaleY * pose.scaleY);
 * art.setAngle(pose.rotation); // degrees, not radians
 *
 * Reset phase to 0 when stopping. Blocked input must pass distance=0 / moving=false.
 * Reduced-motion users receive a stable grounded pose, including during travel.
 */
const TAU = Math.PI * 2;
const CYCLE_DISTANCE: Record<string, number> = { kitten: 108, bird: 82, bunny: 98 };
const standing = () => ({ lift: 0, scaleX: 1, scaleY: 1, rotation: 0, stride: 0 });
const wrap = (phase: number) => (Number.isFinite(phase) ? ((phase % 1) + 1) % 1 : 0);

export function advanceGaitPhase(phase: number, distance: number, species = 'kitten') {
  const travelled = Number.isFinite(distance) ? Math.max(0, distance) : 0;
  return wrap(wrap(phase) + travelled / (CYCLE_DISTANCE[species] || CYCLE_DISTANCE.kitten));
}

export function sampleGait(
  species: string,
  phase: number,
  moving: boolean,
  { reducedMotion = false } = {},
) {
  if (!moving || reducedMotion) return standing();
  const p = wrap(phase),
    step = Math.sin(TAU * p),
    rise = step * step;
  if (species === 'bunny') {
    // Crouch -> take-off -> airborne arc -> landing. Both feet meet the same
    // ground baseline before/after each hop; there is never a negative lift.
    const airborne = p >= 0.22 && p <= 0.78 ? Math.sin((Math.PI * (p - 0.22)) / 0.56) ** 2 : 0;
    const compression =
      p < 0.22
        ? Math.sin((Math.PI * p) / 0.22) ** 2
        : p > 0.78
          ? Math.sin((Math.PI * (p - 0.78)) / 0.22) ** 2
          : 0;
    return {
      lift: 9 * airborne,
      scaleX: 1 + 0.035 * compression - 0.012 * airborne,
      scaleY: 1 - 0.065 * compression + 0.025 * airborne,
      rotation: 0.65 * step,
      stride: step,
    };
  }
  if (species === 'bird') {
    return {
      lift: 1.2 * rise,
      scaleX: 1 + 0.014 * rise,
      scaleY: 1 - 0.012 * rise,
      rotation: 3 * step,
      stride: step,
    };
  }
  return {
    lift: 1.7 * rise,
    scaleX: 1 + 0.006 * rise,
    scaleY: 1 - 0.01 * rise,
    rotation: 1.25 * step,
    stride: step,
  };
}

export const ROSIE_WALK_TEXTURE = 'rosie-walk-v1';
export const ROSIE_WALK_URL = '/assets/rosie-walk-v1.png';
export const ROSIE_WALK_FRAME_COUNT = 6;

/** Call during scene.preload. The original atlas remains the idle fallback. */
export function preloadWalkCycle(scene: Phaser.Scene) {
  scene.load.image(ROSIE_WALK_TEXTURE, ROSIE_WALK_URL);
}

/** Registers a 3-column, 2-row atlas without modifying the generated image. */
export function registerWalkCycle(scene: Phaser.Scene) {
  if (!scene.textures.exists(ROSIE_WALK_TEXTURE)) return false;
  const texture = scene.textures.get(ROSIE_WALK_TEXTURE);
  const { width, height } = texture.source[0];
  for (let i = 0; i < ROSIE_WALK_FRAME_COUNT; i++) {
    const col = i % 3,
      row = Math.floor(i / 3);
    const x = Math.round((col * width) / 3),
      y = Math.round((row * height) / 2);
    const right = Math.round(((col + 1) * width) / 3),
      bottom = Math.round(((row + 1) * height) / 2);
    if (!texture.has(String(i))) texture.add(String(i), 0, x, y, right - x, bottom - y);
  }
  return true;
}

export function walkFrameIndex(phase: number) {
  return Math.min(ROSIE_WALK_FRAME_COUNT - 1, Math.floor(wrap(phase) * ROSIE_WALK_FRAME_COUNT));
}

// Measured head centres and planted soles, in the generated 512px cells.
// Align on the face rather than alpha bounds (the fluffy tail is asymmetric).
const WALK_ANCHORS = [
  [302, 489],
  [269.5, 485],
  [236.5, 489],
  [301.5, 481],
  [269, 480],
  [237, 482],
];
export function walkFrameOrigin(phase: number) {
  const [x, y] = WALK_ANCHORS[walkFrameIndex(phase)];
  return { x: x / 512, y: y / 512 };
}

/**
 * Selects a real alternating-foot frame for Rosie facing down. Returns false
 * for every other hero/direction or idle; the caller then applies its normal
 * directional pose. Does not set origin, size, angle or flip (caller owns those).
 * On success setOrigin(walkFrameOrigin(phase: number).x, walkFrameOrigin(phase: number).y)
 * to compensate for generated frame alignment; restore the idle origin later.
 * Pass facing='down' explicitly; left/right/up must never show front-facing feet.
 */
export function applyWalkCycle(
  sprite: WalkSprite,
  heroId: string,
  phase: number,
  moving: boolean,
  { reducedMotion = false, facing = 'down' } = {},
) {
  if (
    heroId !== 'rose' ||
    facing !== 'down' ||
    !moving ||
    reducedMotion ||
    !sprite.scene.textures.exists(ROSIE_WALK_TEXTURE)
  )
    return false;
  const frame = String(walkFrameIndex(phase));
  if (!sprite.scene.textures.get(ROSIE_WALK_TEXTURE).has(frame)) return false;
  sprite.setTexture(ROSIE_WALK_TEXTURE, frame);
  return true;
}
