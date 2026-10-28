import type { Point } from './types.ts';

// Edge path centres measured in the 1120 × 690 world, shared by art,
// clickable exits, crossing detection and destination spawn positions.
const originalMaps = {
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
export type MapId = string;
export interface MapDefinition {
  name: string;
  west: MapId;
  east: MapId;
  pathY: { west: number; east: number };
  biome?: number;
  variant?: number;
}
export const biomes = [
  {
    name: 'Whispering Woods',
    ground: 0x9dc984,
    canopy: 0x3f7856,
    accent: 0xffd781,
    feature: 'tree',
    places: [
      'Fern Hollow',
      'Mossy Clearing',
      'Owl Grove',
      'Acorn Trail',
      'Willow Retreat',
      'Firefly Glade',
    ],
  },
  {
    name: 'Sunflower Fields',
    ground: 0xc2d98a,
    canopy: 0x7ca451,
    accent: 0xffcd46,
    feature: 'flower',
    places: [
      'Honeybee Haven',
      'Daisy Downs',
      'Buttercup Hill',
      'Poppy Pasture',
      'Sunflower Walk',
      'Clover Crossing',
    ],
  },
  {
    name: 'Coral Coast',
    ground: 0xf1d8a4,
    canopy: 0x4aa69b,
    accent: 0xf29d99,
    feature: 'palm',
    places: [
      'Shell Beach',
      'Coral Lagoon',
      'Palm Peninsula',
      'Seaglass Shore',
      'Turtle Bay',
      'Pearl Sands',
    ],
  },
  {
    name: 'Frosted Peaks',
    ground: 0xdcecf1,
    canopy: 0x7298ae,
    accent: 0xb2d9ed,
    feature: 'pine',
    places: [
      'Snowdrop Valley',
      'Icicle Pass',
      'Polar Lookout',
      'Frostpine Camp',
      'Glacier Garden',
      'Aurora Ridge',
    ],
  },
  {
    name: 'Amber Desert',
    ground: 0xe9bb83,
    canopy: 0x759f7a,
    accent: 0xe88358,
    feature: 'cactus',
    places: [
      'Cactus Oasis',
      'Golden Dunes',
      'Sandstone Terrace',
      'Mirage Rest',
      'Copper Canyon',
      'Moonstone Mesa',
    ],
  },
  {
    name: 'Crystal Caverns',
    ground: 0xada1cb,
    canopy: 0x746b9d,
    accent: 0x8de5ed,
    feature: 'crystal',
    places: [
      'Amethyst Grotto',
      'Quartz Gallery',
      'Opal Hollow',
      'Sapphire Chamber',
      'Prism Passage',
      'Glowstone Refuge',
    ],
  },
  {
    name: 'Autumn Country',
    ground: 0xd9b17d,
    canopy: 0xc27348,
    accent: 0xf6c44e,
    feature: 'tree',
    places: [
      'Maple Lane',
      'Pumpkin Patch',
      'Chestnut Grove',
      'Harvest Hollow',
      'Cider Clearing',
      'Russet Ridge',
    ],
  },
  {
    name: 'Cloud Kingdom',
    ground: 0xd5d9f3,
    canopy: 0xa49cce,
    accent: 0xffd8ec,
    feature: 'cloud',
    places: [
      'Rainbow Rest',
      'Cottoncloud Court',
      'Skyflower Terrace',
      'Breezy Balcony',
      'Silver Lining',
      'Stardust Summit',
    ],
  },
  {
    name: 'Mushroom Marsh',
    ground: 0x9cbaaa,
    canopy: 0x607f7b,
    accent: 0xe3a5c7,
    feature: 'mushroom',
    places: [
      'Toadstool Dell',
      'Lilypond Landing',
      'Reed Retreat',
      'Dragonfly Nook',
      'Sporelight Trail',
      'Dewdrop Fen',
    ],
  },
  {
    name: 'Blossom Gardens',
    ground: 0xc1d6a6,
    canopy: 0xdba5b8,
    accent: 0xffd7e1,
    feature: 'tree',
    places: [
      'Cherry Courtyard',
      'Lotus Lawn',
      'Rose Retreat',
      'Wisteria Walk',
      'Petal Pavilion',
      'Magnolia Meadow',
    ],
  },
  {
    name: 'Moonlit Wilds',
    ground: 0x898eb6,
    canopy: 0x555f8b,
    accent: 0xf6e9a3,
    feature: 'crystal',
    places: [
      'Crescent Clearing',
      'Comet Camp',
      'Lunar Lookout',
      'Twilight Terrace',
      'Meteor Meadow',
      'Constellation Court',
    ],
  },
  {
    name: 'Adventure Highlands',
    ground: 0xa9c59a,
    canopy: 0x668878,
    accent: 0xcab59a,
    feature: 'rock',
    places: [
      'Pebble Plateau',
      'Windmill Rise',
      'Boulder Basin',
      'Heather Heath',
      'Explorer Camp',
      'Echo Valley',
    ],
  },
] as const;
export const maps: Record<MapId, MapDefinition> = { ...originalMaps };
biomes.forEach((biome, b) =>
  biome.places.forEach((name, variant) => {
    maps[`explore-${b}-${variant}`] = {
      name,
      biome: b,
      variant,
      west: '',
      east: '',
      pathY: { west: 330 + (variant % 3) * 45, east: 330 + ((variant + 1) % 3) * 45 },
    };
  }),
);
// Keep the original meadow/orchard/cove route and extend it into one reversible loop.
const itinerary = [
  'meadow',
  'cove',
  'orchard',
  ...Object.keys(maps).filter((id) => id.startsWith('explore-')),
];
itinerary.forEach((id, i) => {
  maps[id].west = itinerary[(i + itinerary.length - 1) % itinerary.length];
  maps[id].east = itinerary[(i + 1) % itinerary.length];
});
export function mapObstacles(region: string) {
  const map = maps[region];
  if (map?.biome === undefined) return [];
  const v = map.variant ?? 0;
  return [
    { x: 370 + v * 48, y: 245, radius: 30 },
    { x: 740 - v * 38, y: 535, radius: 34 },
    { x: 890 - v * 65, y: 170, radius: 28 },
  ];
}
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
