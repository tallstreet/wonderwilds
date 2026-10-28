import { friendName } from './backend-world-model.ts';

import type { CharacterProgression } from '../../contracts/generated/contracts.ts';

type TrainingSnapshot = {
  ownedCreatures: string[];
  characterProgression?: CharacterProgression[];
};
const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );

/** Only server-confirmed owned characters expose progression or training actions. */
export function trainingProgress(id: string, snapshot: TrainingSnapshot) {
  if (!snapshot.ownedCreatures.includes(id)) return undefined;
  return snapshot.characterProgression?.find((progress) => progress.creatureId === id);
}

export function trainingDetails(id: string, snapshot: TrainingSnapshot): string {
  const progress = trainingProgress(id, snapshot);
  if (!progress) return '';
  const next =
    progress.nextLevelXp === undefined
      ? '<p>All five character levels reached!</p>'
      : `<p>Next level at ${progress.nextLevelXp} XP. Train together to grow.</p>`;
  const training = progress.availableTraining
    .map((offer) => {
      const completed = progress.completedTrainingIds?.includes(offer.trainingId) ?? false;
      return `<button class="secondary" data-server-training="${escape(offer.trainingId)}">${completed ? 'Practise' : 'Train'} with ${escape(friendName(id))} · ${completed ? 'no extra XP' : `${offer.xp} XP`}</button>`;
    })
    .join('');
  return `<section class="companion-training" aria-label="${escape(friendName(id))} training"><p class="character-level" data-character-level="${escape(id)}">Level ${progress.level} · ${progress.xp} character XP</p>${next}${progress.level >= 2 ? `<p class="training-reaction">✦ ${escape(friendName(id))} celebrates with a happy wiggle!</p>` : '<p>At level two: a happy celebration wiggle.</p>'}${training || '<p>More training will become available as your adventure grows.</p>'}<small>Help is always welcome. Character levels celebrate adventures, not academic ability.</small></section>`;
}

export function trainingLevelUp(
  before: TrainingSnapshot,
  after: TrainingSnapshot,
  creatureId: string,
) {
  const previous = trainingProgress(creatureId, before);
  const current = trainingProgress(creatureId, after);
  return previous && current && current.level > previous.level ? current : undefined;
}
