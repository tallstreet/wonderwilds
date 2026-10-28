import type { Point } from './types.ts';
import { BRIDGE, FOOTPRINT, bridgeContainsFeet } from './mapGeometry.ts';
// Inputs are the ground midpoint between the actor's feet, never sprite centre.
export function riverCenter() {
  return BRIDGE.x;
}
export function isBlocked(x: number, y: number, bridgeFixed: boolean, region = 'meadow') {
  if (x < 40 || x > 1080 || y < 90 || y > 615) return true;
  if (x > 98 && x < 282 && y > 55 && y < 230) return true;
  if (region !== 'meadow') return false;
  if (
    Math.abs(x - riverCenter()) < 76 + FOOTPRINT.halfWidth &&
    !(bridgeFixed && bridgeContainsFeet(y))
  )
    return true;
  return false;
}
// Grid breadth-first search finds a short walkable route. Four-way steps avoid cutting corners.
export function findRoute(
  start: Point,
  target: Point,
  bridgeFixed: boolean,
  region = 'meadow',
): Point[] {
  const step = 16,
    cols = 71,
    rows = 44;
  const blocked = (x: number, y: number) => isBlocked(x, y, bridgeFixed, region);
  const nearest = (point: Point) => {
    let best = null,
      d = Infinity;
    for (let y = 0; y < rows; y++)
      for (let x = 0; x < cols; x++) {
        const px = x * step,
          py = y * step,
          dist = Math.hypot(px - point.x, py - point.y);
        if (dist < d && !blocked(px, py)) {
          best = { x, y };
          d = dist;
        }
      }
    return best;
  };
  const a = nearest(start),
    b = nearest(target);
  if (!a || !b) return [];
  const key = (p: Point) => p.y * cols + p.x,
    queue = [a],
    seen = new Map<number, number | null>([[key(a), null]]);
  let head = 0;
  while (head < queue.length) {
    const p = queue[head++];
    if (key(p) === key(b)) {
      const path = [];
      let k: number | null = key(p);
      while (k !== null) {
        path.push({ x: (k % cols) * step, y: Math.floor(k / cols) * step });
        k = seen.get(k) ?? null;
      }
      return path.reverse();
    }
    for (const [dx, dy] of [
      [1, 0],
      [-1, 0],
      [0, 1],
      [0, -1],
    ]) {
      const n = { x: p.x + dx, y: p.y + dy };
      if (
        n.x < 0 ||
        n.x >= cols ||
        n.y < 0 ||
        n.y >= rows ||
        seen.has(key(n)) ||
        blocked(n.x * step, n.y * step)
      )
        continue;
      seen.set(key(n), key(p));
      queue.push(n);
    }
  }
  return [];
}
