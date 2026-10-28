import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import creaturePack from '../../content/creatures/friends.v1.json';
import development from '../../content/creatures/development.v1.json';
import story from '../../content/story/adventure.v1.json';
import helping from '../../content/story/help-quests.v1.json';
import progression from '../../content/story/progression.v1.json';

export type CompanionProgress = Pick<
  GameSnapshot,
  | 'ownedCreatures'
  | 'storyFlags'
  | 'friendship'
  | 'transformedCreatures'
  | 'availableHelpingMoments'
>;

export function companionProfile(id: string) {
  return creaturePack.creatures.find((creature) => creature.id === id);
}

const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );

export function companionProgress(id: string, state: CompanionProgress) {
  const friend = companionProfile(id);
  if (!friend || !state.ownedCreatures.includes(id)) return undefined;
  const reportedPoints = state.friendship?.find((entry) => entry.creatureId === id)?.points;
  const points =
    typeof reportedPoints === 'number' &&
    Number.isSafeInteger(reportedPoints) &&
    reportedPoints >= 0 &&
    reportedPoints <= progression.friendship.maximum
      ? reportedPoints
      : undefined;
  const profile = development.characters.find((character) => character.creatureId === id);
  const dialogue =
    points === undefined
      ? undefined
      : profile?.friendshipMoments
          .filter(
            (moment) =>
              points >= moment.minimumFriendship &&
              moment.requiredStoryFlags.every((flag) => state.storyFlags.includes(flag)),
          )
          .sort((a, b) => b.minimumFriendship - a.minimumFriendship)[0]?.dialogue;
  const transformed = state.transformedCreatures?.includes(friend.transformation.id) ?? false;
  const canTransform =
    points !== undefined &&
    points >= friend.transformation.minimumFriendship &&
    state.storyFlags.includes(friend.transformation.requiredStoryFlag);
  // A server offer must still resolve to a known authored quest before it becomes a UI action.
  const moments = (state.availableHelpingMoments ?? [])
    .filter((moment) => moment.creatureId === id)
    .map((moment) => {
      const title =
        story.beats.find((beat) => beat.id === moment.completedQuestId)?.title ??
        helping.quests.find((quest) => quest.id === moment.completedQuestId)?.title;
      return title ? { ...moment, title } : undefined;
    })
    .filter((moment) => moment !== undefined);
  return { points, dialogue, transformed, canTransform, moments };
}

/** Presentation reads confirmed server state and never grants progress or learning evidence. */
export function companionDetails(id: string, owned: boolean, state?: CompanionProgress): string {
  const friend = companionProfile(id);
  if (!friend) return '';
  const profile = development.characters.find((character) => character.creatureId === id);
  const progress = owned && state ? companionProgress(id, state) : undefined;
  const points =
    progress?.points === undefined
      ? ''
      : `<p class="companion-friendship">Friendship: ${progress.points} / ${progression.friendship.maximum}</p>`;
  const dialogue = progress?.dialogue
    ? `<blockquote>${escape(progress.dialogue)}</blockquote>`
    : '';
  const glow = progress
    ? `${progress.transformed ? '<p class="companion-glow" role="status">Lantern glow is on</p>' : ''}${progress.canTransform || progress.transformed ? `<button class="secondary" data-server-transform="${escape(id)}" aria-pressed="${progress.transformed}" data-transform-enabled="${!progress.transformed}">${progress.transformed ? 'Turn glow off' : 'Turn glow on'}</button>` : '<p>Lantern glow grows from shared memories and restoring this region’s light.</p>'}`
    : '';
  const moments =
    progress?.moments
      .map(
        (moment) =>
          `<button class="secondary" data-server-help-friend="${escape(id)}" data-completed-quest="${escape(moment.completedQuestId)}">Remember together: ${escape(moment.title)}</button>`,
      )
      .join('') ?? '';
  return `<details class="companion-details"><summary>Meet ${escape(friend.name)}</summary><p>${escape(friend.name)} is ${escape(friend.personality)}.</p>${profile ? `<p><strong>Wants to</strong> ${escape(profile.motivation)}</p>` : ''}<p><strong>Favourite place</strong> ${escape(friend.preferredHabitat)}</p>${owned ? `<p><strong>A helping idea</strong> ${escape(friend.assistance.alternative)}</p><p>You can ask for help and still earn story rewards.</p>${points}${dialogue}${moments}${glow}` : `<p><strong>Habitat clue</strong> ${escape(friend.habitatClue)}</p>`}</details>`;
}
