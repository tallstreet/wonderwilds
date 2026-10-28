import type { Point, Blocked } from './types.ts';
// Physical ground motion is separate from sprite animation and camera motion.
export function moveGround(
  point: Point,
  dx: number,
  dy: number,
  distance: number,
  blocked: Blocked,
) {
  const length = Math.hypot(dx, dy),
    path: Point[] = [];
  let x = point.x,
    y = point.y;
  if (!length || distance <= 0) return { x, y, path, distance: 0 };
  const nx = x + (dx / length) * distance,
    ny = y + (dy / length) * distance;
  if (!blocked(nx, y)) {
    x = nx;
    if (x !== point.x) path.push({ x, y });
  }
  if (!blocked(x, ny)) {
    y = ny;
    if (y !== point.y) path.push({ x, y });
  }
  return { x, y, path, distance: Math.hypot(x - point.x, y - point.y) };
}

// A follower consumes the same ordered path as its leader. Following a straight
// offset can cut corners into the river; a trail preserves each safe turn.
export function advanceFollower(
  point: Point,
  trail: Point[],
  budget: number,
  gap: number,
  blocked: Blocked,
) {
  let remaining = 0,
    previous = point;
  for (const p of trail) {
    remaining += Math.hypot(p.x - previous.x, p.y - previous.y);
    previous = p;
  }
  let allowance = Math.min(budget, Math.max(0, remaining - gap)),
    x = point.x,
    y = point.y,
    moved = 0;
  while (trail.length && allowance > 0) {
    const target = trail[0],
      dx = target.x - x,
      dy = target.y - y,
      length = Math.hypot(dx, dy);
    if (length < 0.01) {
      trail.shift();
      continue;
    }
    const step = Math.min(allowance, length),
      nx = x + (dx / length) * step,
      ny = y + (dy / length) * step;
    if (blocked(nx, ny)) break;
    x = nx;
    y = ny;
    moved += step;
    allowance -= step;
    if (step >= length - 0.01) trail.shift();
  }
  return { x, y, distance: moved };
}
