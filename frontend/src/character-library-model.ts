import roster from '../../content/creatures/friends.v1.json';
import development from '../../content/creatures/development.v1.json';
import { characterPortrait } from './character-art.ts';

export const libraryRegions = [
  { id: 'all', name: 'All friends' },
  { id: 'meadow', name: 'Clover Meadow' },
  { id: 'orchard', name: 'Peachy Orchard' },
  { id: 'cove', name: 'Starlight Cove' },
] as const;
export type LibraryRegion = (typeof libraryRegions)[number]['id'];
export function charactersForRegion(region: LibraryRegion) {
  return roster.creatures.filter((friend) => region === 'all' || friend.regionId === region);
}
export function escapeCharacterText(value: string): string {
  return value.replace(
    /[&<>"']/g,
    (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[char]!,
  );
}
export function characterCard(id: string): string {
  const friend = roster.creatures.find((entry) => entry.id === id);
  const profile = development.characters.find((entry) => entry.creatureId === id);
  if (!friend || !profile) return '';
  const esc = escapeCharacterText;
  const region = libraryRegions.find((entry) => entry.id === friend.regionId)!;
  return `<article class="friend-card" aria-labelledby="friend-${esc(id)}"><div class="friend-card-art">${characterPortrait(id, 260)}</div><div class="friend-card-body"><p class="friend-region">${esc(region.name)}</p><h2 id="friend-${esc(id)}">${esc(friend.name)}</h2><p class="friend-species">${esc(friend.species)}</p><p class="friend-personality">${esc(friend.personality)}</p><p>${esc(profile.motivation)}</p><details><summary>Get to know ${esc(friend.name)}</summary><dl><dt>Favourite place</dt><dd>${esc(friend.preferredHabitat)}</dd><dt>Look for</dt><dd>${esc(friend.habitatClue)}</dd><dt>A little growing moment</dt><dd>${esc(profile.growthOpportunity)}</dd><dt>How they speak</dt><dd>${esc(profile.voice)}</dd></dl></details></div></article>`;
}
