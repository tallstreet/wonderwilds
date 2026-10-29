import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import { friendName, meadowBeats, meadowFriends, questCompleted } from './backend-world-model.ts';
import { maps, type MapId } from './maps.ts';
import { meadowPlacements } from './world-placements.ts';
import { isRecord } from './types.ts';

/** Local conversation preferences only. Activity progress always comes from the server. */
export interface QuestTrailState {
  accepted: boolean;
  returned: boolean;
}
export interface TrailStep {
  id: string;
  label: string;
  region: MapId;
  complete: boolean;
}

const activities = [
  { id: 'meadow-welcome', label: 'Listen to Pip and help with a message' },
  { id: 'meadow-bridge', label: 'Count the missing bridge planks' },
  { id: 'meadow-picnic', label: 'Share berries at the seaside picnic' },
  { id: 'befriend-tansy', label: 'Meet Tansy and arrange the seed pots' },
  { id: 'meadow-garden', label: 'Help the garden grow' },
] as const;

export function parseQuestTrailState(value: unknown): QuestTrailState {
  if (
    !isRecord(value) ||
    typeof value.accepted !== 'boolean' ||
    typeof value.returned !== 'boolean'
  )
    return { accepted: false, returned: false };
  return { accepted: value.accepted, returned: value.accepted && value.returned };
}

export function trailReadyToReturn(snapshot: GameSnapshot): boolean {
  return activities.every(({ id }) => questCompleted(snapshot, id));
}

export function trailSteps(snapshot: GameSnapshot, state: QuestTrailState): TrailStep[] {
  return [
    {
      id: 'trail-start',
      label: 'Talk to Pip and accept the trail',
      region: 'meadow',
      complete: state.accepted,
    },
    ...activities.map(({ id, label }) => ({
      id,
      label,
      region: meadowPlacements[id].region,
      complete: questCompleted(snapshot, id),
    })),
    {
      id: 'trail-return',
      label: 'Return to Pip and share your adventure',
      region: 'meadow',
      complete: state.accepted && state.returned && trailReadyToReturn(snapshot),
    },
  ];
}

export function nextTrailEncounter(
  snapshot: GameSnapshot,
  state: QuestTrailState,
): string | undefined {
  const next = trailSteps(snapshot, state).find((step) => !step.complete);
  if (!next) return undefined;
  return next.id === 'trail-start' || next.id === 'trail-return' ? 'meadow-welcome' : next.id;
}

const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );

export function trailChecklistView(snapshot: GameSnapshot, state: QuestTrailState): string {
  const steps = trailSteps(snapshot, state);
  const current = steps.find((step) => !step.complete)?.id;
  const complete = steps.filter((step) => step.complete).length;
  return `<section class="quest-trail" aria-label="A trail of little kindnesses"><span class="pill">YOUR EXPLORATION QUEST</span><h3>A trail of little kindnesses</h3><p>Follow Pip’s trail from the meadow to the seaside picnic and the orchard. Meet Tansy, help the garden grow, then come back to tell Pip.</p><p class="quest-trail-progress">${complete} of ${steps.length} steps complete${current ? '' : ' · Trail complete!'}</p><ol>${steps
    .map((step) => {
      const status = step.complete ? 'complete' : step.id === current ? 'current' : 'locked';
      const statusText = step.complete
        ? 'Complete'
        : step.id === current
          ? 'Current step'
          : 'Locked · follow the earlier steps';
      return `<li class="quest-trail-step ${status}"${status === 'current' ? ' aria-current="step"' : ''}><span class="quest-trail-symbol" aria-hidden="true">${step.complete ? '✓' : status === 'current' ? '→' : '○'}</span><span><strong>${escape(step.label)}</strong><small>${escape(maps[step.region].name)} · ${statusText}</small></span></li>`;
    })
    .join(
      '',
    )}</ol><p class="quest-trail-support">You can ask for help with every activity. Helping together counts too.</p></section>`;
}

export function encounterDialogue(
  questId: string,
  snapshot: GameSnapshot,
  state: QuestTrailState,
): { speaker: string; text: string; actionLabel: string } {
  if (questId === 'meadow-welcome') {
    if (!state.accepted)
      return {
        speaker: 'Pip',
        text: 'Will you follow a trail of little kindnesses with me? We’ll fix the bridge, share a picnic, meet Tansy and help the garden grow. Come back and tell me about your adventure!',
        actionLabel: 'Accept the trail',
      };
    if (trailReadyToReturn(snapshot))
      return {
        speaker: 'Pip',
        text: state.returned
          ? 'Look at all the kindness you shared! The bridge is mended, the picnic is shared and the garden is growing. There are more friends waiting along the paths.'
          : 'You came back! The bridge is mended, the picnic is shared, and you helped Tansy and the garden. Thank you for bringing everyone a little closer together.',
        actionLabel: state.returned ? 'Keep exploring' : 'Tell Pip about the adventure',
      };
  }
  const friend = meadowFriends.find((quest) => quest.id === questId);
  if (friend) {
    const name = friendName(friend.creatureId);
    return {
      speaker: name,
      text: questCompleted(snapshot, questId)
        ? `It’s lovely to see you again! We became friends by helping together. You can choose me in your journal when you want company.`
        : `${friend.prompt} Can we try it together? You can ask for help; we’ll become friends when we finish.`,
      actionLabel: questCompleted(snapshot, questId) ? 'Open my journal' : `Help ${name}`,
    };
  }
  const beat = meadowBeats.find((item) => item.id === questId);
  if (beat)
    return {
      speaker: 'Pip',
      text: questCompleted(snapshot, questId)
        ? 'Thank you for helping here! Follow the next step in your trail, or explore the paths to find another friend.'
        : beat.dialogue,
      actionLabel: questCompleted(snapshot, questId) ? 'Follow the trail' : 'Let’s help',
    };
  return {
    speaker: 'Training clearing',
    text: 'A quiet place to practise together. Choose a friend and try a little activity; help is always welcome.',
    actionLabel: 'Train together',
  };
}
