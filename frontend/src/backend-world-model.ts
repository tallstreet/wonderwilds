import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import type { WorldEncounter } from './types.ts';
import story from '../../content/story/adventure.v1.json';
import friendships from '../../content/story/help-quests.v1.json';

export const meadowBeats = story.beats.filter((b) => b.chapterId === 'meadow');
export const meadowFriends = friendships.quests.filter((q) =>
  ['pip', 'tansy', 'nib', 'flutter'].includes(q.creatureId),
);
// Companion IDs resolve to their own artwork, independently of playable hero IDs.
export const friendSprites: Record<string, string> = {
  pip: 'pip',
  tansy: 'tansy',
  nib: 'nib',
  flutter: 'flutter',
};
export const friendName = (id: string) => id.charAt(0).toUpperCase() + id.slice(1);
const positions: Record<string, [number, number]> = {
  'meadow-welcome': [205, 490],
  'meadow-bridge': [585, 438],
  'meadow-picnic': [335, 338],
  'meadow-garden': [920, 323],
  'meadow-letter': [455, 535],
  'meadow-finale': [1000, 475],
  'befriend-pip': [810, 430],
  'befriend-tansy': [450, 260],
  'befriend-nib': [340, 485],
  'befriend-flutter': [540, 550],
};
export const worldQuests: WorldEncounter[] = [
  ...meadowBeats.map((b) => ({
    id: b.id,
    short: b.title,
    character: b.id === 'meadow-welcome' ? 'Pip' : b.title,
    sprite: 'pip',
  })),
  ...meadowFriends.map((q) => ({
    id: q.id,
    short: q.title,
    character: friendName(q.creatureId),
    sprite: friendSprites[q.creatureId],
  })),
].map((q) => ({ ...q, region: 'meadow', x: positions[q.id][0], y: positions[q.id][1] }));
export function questCompleted(snapshot: GameSnapshot, id: string): boolean {
  const beat = meadowBeats.find((b) => b.id === id);
  if (beat)
    return (
      beat.completionFlags.length > 0 &&
      beat.completionFlags.every((flag) => snapshot.storyFlags.includes(flag))
    );
  const friend = meadowFriends.find((q) => q.id === id);
  return Boolean(friend && snapshot.ownedCreatures.includes(friend.creatureId));
}
export function questReady(snapshot: GameSnapshot, id: string): boolean {
  const beat = meadowBeats.find((b) => b.id === id);
  const friend = meadowFriends.find((q) => q.id === id);
  if (!beat && !friend) return false;
  return (
    !questCompleted(snapshot, id) &&
    [
      ...(beat?.requiresBeatIds ?? friend?.requiresBeatIds ?? []),
      ...(beat?.finaleSteps ?? []),
    ].every((key) => questCompleted(snapshot, key)) &&
    (beat?.requiredCreatureIds ?? []).every((key) => snapshot.ownedCreatures.includes(key))
  );
}
export function completedWorldQuests(snapshot: GameSnapshot): string[] {
  return worldQuests.filter((q) => questCompleted(snapshot, q.id)).map((q) => q.id);
}
