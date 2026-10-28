import roster from '../../content/creatures/friends.v1.json';

export const characterSheets = {
  meadow: { path: '/assets/characters/meadow.png', ids: ['pip', 'tansy', 'nib', 'flutter'] },
  orchard: {
    path: '/assets/characters/orchard.png',
    ids: ['cleo', 'bramble', 'patter', 'twizzle'],
  },
  cove: { path: '/assets/characters/cove.png', ids: ['luna', 'pebble', 'glimmer', 'sail'] },
} as const;

// All delivered sheets use a clean horizontal midpoint.
export function characterSheetRowSplit(_region: string): number {
  return 0.5;
}

// Cove's broad wings need a wider left column than its right column.
export function characterSheetColumnSplit(region: string): number {
  return region === 'cove' ? 660 / 1254 : 0.5;
}

export function characterArtwork(id: string) {
  const creature = roster.creatures.find((friend) => friend.id === id);
  if (!creature) return undefined;
  const sheet = characterSheets[creature.regionId as keyof typeof characterSheets];
  const index = (sheet.ids as readonly string[]).indexOf(id);
  if (index < 0) return undefined;
  const column = index % 2;
  const row = Math.floor(index / 2);
  const sheetSize = 1254;
  const split = Math.round(sheetSize * characterSheetRowSplit(creature.regionId));
  const y = row === 0 ? 0 : split;
  const columnSplit = Math.round(sheetSize * characterSheetColumnSplit(creature.regionId));
  const x = column === 0 ? 0 : columnSplit;
  return {
    path: sheet.path,
    column,
    row,
    x,
    y,
    width: column === 0 ? columnSplit : sheetSize - columnSplit,
    height: row === 0 ? split : sheetSize - split,
    sheetWidth: sheetSize,
    sheetHeight: sheetSize,
  };
}

/** Decorative artwork: callers supply the character name as visible text. */
export function characterPortrait(id: string, size = 160): string {
  const art = characterArtwork(id);
  if (!art) return '';
  const safeSize = Number.isFinite(size) ? Math.max(24, Math.min(600, size)) : 160;
  return `<span class="friend-portrait" aria-hidden="true" style="--portrait-size:${safeSize}px;background-image:url('${art.path}');background-position:${art.column * 100}% ${art.row * 100}%;background-size:${(art.sheetWidth / art.width) * 100}% ${(art.sheetHeight / art.height) * 100}%;aspect-ratio:${art.width}/${art.height}"></span>`;
}
