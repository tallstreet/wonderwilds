import type { Point } from './types.ts';

// Edge path centres measured in the 1120 × 690 world, shared by art,
// clickable exits, crossing detection and destination spawn positions.
export const maps = {
  meadow: { name: 'Clover Meadow', west: 'orchard', east: 'cove', pathY: { west: 475, east: 345 } },
  orchard: {
    name: 'Peachy Orchard',
    west: 'cove',
    east: 'meadow',
    pathY: { west: 337, east: 453 },
  },
  cove: {
    name: 'Starlight Cove',
    west: 'meadow',
    east: 'orchard',
    pathY: { west: 330, east: 330 },
  },
} as const;
export type MapId = keyof typeof maps;
export type MapSide = 'west' | 'east';
export function isMapId(value: unknown): value is MapId {
  return typeof value === 'string' && Object.hasOwn(maps, value);
}
export function exitPoint(region: MapId, side: MapSide): Point {
  return { x: side === 'west' ? 48 : 1072, y: maps[region].pathY[side] };
}
export function mapExit(region: string, point: Point) {
  if (!isMapId(region)) return null;
  const side = point.x <= 64 ? 'west' : point.x >= 1056 ? 'east' : null;
  if (!side || Math.abs(point.y - maps[region].pathY[side]) > 16) return null;
  const destination = maps[region][side];
  const arrivalSide = side === 'west' ? 'east' : 'west';
  return {
    region: destination,
    // Preserve lateral position on the joining path, clear of its trigger.
    position: {
      x: arrivalSide === 'west' ? 96 : 1024,
      y: maps[destination].pathY[arrivalSide] + point.y - maps[region].pathY[side],
    },
  };
}
