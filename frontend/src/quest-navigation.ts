import { maps, isMapId, exitPoint, type MapSide } from './maps.ts';

/** First exit on the shortest connected walking route; never changes map state. */
export function nextQuestExit(from: string, to: string): MapSide | undefined {
  if (!isMapId(from) || !isMapId(to) || from === to) return undefined;
  const queue: { map: string; first?: MapSide }[] = [{ map: from }];
  const seen = new Set([from]);
  for (let i = 0; i < queue.length; i++) {
    const current = queue[i];
    for (const side of ['east', 'west'] as const) {
      const next = maps[current.map][side];
      if (seen.has(next)) continue;
      const first = current.first ?? side;
      if (next === to) return first;
      seen.add(next);
      queue.push({ map: next, first });
    }
  }
  return undefined;
}
export function questDirection(from: string, to: string): string {
  const side = nextQuestExit(from, to);
  if (side)
    return `${side === 'east' ? '→' : '←'} Follow the ${side} path to ${maps[maps[from][side]].name}`;
  return from === to ? 'You are here — walk over and say hello' : 'Choose an adventure to follow';
}
export function questWaypoint(from: string, target: { region: string; x: number; y: number }) {
  const side = nextQuestExit(from, target.region);
  return side
    ? { point: exitPoint(from, side), arrived: false }
    : from === target.region
      ? { point: { x: target.x - 30, y: target.y + 25 }, arrived: true }
      : undefined;
}
