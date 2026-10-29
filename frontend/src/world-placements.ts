import { isMapId, type MapId } from './maps.ts';
import type { WorldEncounter } from './types.ts';

type Placement = { region: MapId; x: number; y: number };
export type WorldPlacements = Readonly<Record<string, Placement>>;

/** Browser scenery positions only; these do not establish server eligibility. */
export const meadowPlacements = {
  'meadow-welcome': { region: 'meadow', x: 205, y: 490 },
  'meadow-bridge': { region: 'meadow', x: 585, y: 438 },
  'meadow-picnic': { region: 'meadow', x: 335, y: 338 },
  'meadow-garden': { region: 'meadow', x: 920, y: 323 },
  'meadow-letter': { region: 'meadow', x: 455, y: 535 },
  'meadow-finale': { region: 'meadow', x: 1000, y: 475 },
  'befriend-pip': { region: 'meadow', x: 810, y: 430 },
  'befriend-tansy': { region: 'meadow', x: 450, y: 260 },
  'befriend-nib': { region: 'meadow', x: 340, y: 485 },
  'befriend-flutter': { region: 'meadow', x: 540, y: 550 },
  'meadow-training-clearing': { region: 'meadow', x: 850, y: 520 },
} satisfies WorldPlacements;

/** Fail with an actionable content error instead of indexing a missing coordinate. */
export function placeWorldEncounters(
  encounters: readonly Omit<WorldEncounter, 'x' | 'y'>[],
  placements: WorldPlacements,
): WorldEncounter[] {
  const ids = new Set(encounters.map((encounter) => encounter.id));
  if (ids.size !== encounters.length) throw new Error('Duplicate world encounter ID');
  for (const id of Object.keys(placements)) {
    if (!ids.has(id)) throw new Error(`Obsolete world placement: ${id}`);
  }
  return encounters.map((encounter) => {
    if (!Object.hasOwn(placements, encounter.id))
      throw new Error(`Missing world placement: ${encounter.id}`);
    const placement = placements[encounter.id];
    if (!isMapId(placement.region) || placement.region !== encounter.region)
      throw new Error(`Invalid map for world placement: ${encounter.id}`);
    if (
      !Number.isFinite(placement.x) ||
      !Number.isFinite(placement.y) ||
      placement.x < 0 ||
      placement.x > 1120 ||
      placement.y < 0 ||
      placement.y > 690
    )
      throw new Error(`Invalid coordinates for world placement: ${encounter.id}`);
    return { ...encounter, ...placement };
  });
}
