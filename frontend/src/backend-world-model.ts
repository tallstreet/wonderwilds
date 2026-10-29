import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import type { WorldEncounter } from './types.ts';
import story from '../../content/story/adventure.v1.json';
import friendships from '../../content/story/help-quests.v1.json';
import creatures from '../../content/creatures/friends.v1.json';
import { meadowPlacements, placeWorldEncounters } from './world-placements.ts';

export const meadowBeats = story.beats.filter((b) => b.chapterId === 'meadow');
const meadowCreatureIds = new Set(
  creatures.creatures.filter((c) => c.regionId === 'meadow').map((c) => c.id),
);
export const meadowFriends = friendships.quests.filter((q) => meadowCreatureIds.has(q.creatureId));
// Companion IDs resolve to their own artwork, independently of playable hero IDs.
export const friendSprites: Record<string, string> = Object.fromEntries(
  [...meadowCreatureIds].map((id) => [id, id]),
);
export const friendName = (id: string) => id.charAt(0).toUpperCase() + id.slice(1);
const placedEncounters = placeWorldEncounters(
  [
    ...meadowBeats.map((b) => ({
      id: b.id,
      region: meadowPlacements[b.id].region,
      short: b.title,
      character: b.id === 'meadow-welcome' ? 'Pip' : b.title,
      sprite: 'pip',
    })),
    ...meadowFriends.map((q) => ({
      id: q.id,
      region: meadowPlacements[q.id].region,
      short: q.title,
      character: friendName(q.creatureId),
      sprite: friendSprites[q.creatureId],
    })),
    {
      id: 'meadow-training-clearing',
      short: 'Train together',
      character: 'Training clearing',
      sprite: 'pip',
      region: meadowPlacements['meadow-training-clearing'].region,
    },
  ],
  meadowPlacements,
);
export const trainingClearing: WorldEncounter = placedEncounters.find(
  (q) => q.id === 'meadow-training-clearing',
)!;
export const worldQuests = placedEncounters.filter((q) => q !== trainingClearing);
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
