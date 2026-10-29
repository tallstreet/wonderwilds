import type { GameSnapshot, PersonalQuestOffer } from '../../contracts/generated/contracts.ts';
import { friendName } from './backend-world-model.ts';
import { characterPortrait } from './character-art.ts';

type PersonalSnapshot = Pick<
  GameSnapshot,
  | 'ownedCreatures'
  | 'personalQuestOffers'
  | 'personalQuest'
  | 'questInterests'
  | 'questInterestOptions'
>;
const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );

export function activePersonalQuest(snapshot: PersonalSnapshot) {
  const quest = snapshot.personalQuest;
  return quest && snapshot.ownedCreatures.includes(quest.companionId) ? quest : undefined;
}

export function personalQuestOffers(snapshot: PersonalSnapshot, companionId?: string) {
  return (snapshot.personalQuestOffers ?? []).filter(
    (offer) =>
      snapshot.ownedCreatures.includes(offer.companionId) &&
      (companionId === undefined || offer.companionId === companionId),
  );
}

function offerButton(offer: PersonalQuestOffer): string {
  return `<button class="secondary" data-server-personal-quest="${escape(offer.questId)}" ${offer.available ? '' : 'disabled'}>${escape(offer.available ? `${offer.completed ? 'Revisit' : 'Start'} ${offer.title}` : (offer.lockedReason ?? `Unlock at character level ${offer.minimumCharacterLevel}`))}</button>${offer.recommendation && offer.available ? `<small class="quest-recommendation">${escape(offer.recommendation)}</small>` : ''}`;
}

/** Presentation reads confirmed server availability; it never derives a quest unlock or reward. */
export function personalQuestDetails(companionId: string, snapshot: PersonalSnapshot): string {
  const offers = personalQuestOffers(snapshot, companionId);
  const active = activePersonalQuest(snapshot);
  if (!offers.length && active?.companionId !== companionId) return '';
  const inProgress = active && active.node.type !== 'handoff';
  return `<section class="personal-quest-details" aria-label="${escape(friendName(companionId))} personal adventures"><h4>Personal adventures</h4>${active?.companionId === companionId ? `<button class="secondary" data-server-personal-resume="${escape(active.questId)}">${active.node.type === 'handoff' ? 'Choose our next adventure' : `Continue ${escape(active.title)}`}</button>` : ''}${inProgress ? '<p>Continue your current episode, or leave it before choosing a new one.</p>' : offers.map(offerButton).join('')}<small>Your choices become part of the adventure. Help is welcome at every step.</small></section>`;
}

export function interestControls(snapshot: PersonalSnapshot): string {
  const selected = snapshot.questInterests ?? [];
  return snapshot.questInterestOptions?.length
    ? `<section class="quest-interests" aria-label="Adventure interests"><h3>What would you like to explore?</h3><p>Choose up to five interests. Every available adventure stays your choice.</p>${snapshot.questInterestOptions.map((tag) => `<button class="secondary" data-server-quest-interest="${escape(tag)}" aria-pressed="${selected.includes(tag)}" ${!selected.includes(tag) && selected.length >= 5 ? 'disabled' : ''}>${escape(tag.replaceAll('-', ' '))}</button>`).join('')}</section>`
    : '';
}
export function personalQuestLibrary(snapshot: PersonalSnapshot): string {
  const active = activePersonalQuest(snapshot);
  const offers = personalQuestOffers(snapshot);
  const inProgress = active && active.node.type !== 'handoff';
  return `<span class="pill">ADVENTURES WITH FRIENDS</span><h2 id="dialog-title">What shall we explore next?</h2><p>Choose a personal adventure. Each little episode opens a way to explore again.</p>${interestControls(snapshot)}${active ? `<button class="primary" id="server-personal-resume">${inProgress ? `Continue ${escape(active.title)}` : 'Our latest shared memory'}</button>` : ''}${inProgress ? '<p>Finish your current episode, or leave it before beginning another.</p>' : `<div class="personal-quest-offers">${offers.map((offer) => `<article>${characterPortrait(offer.companionId, 72)}<h3>${escape(offer.title)}</h3><p>With ${escape(friendName(offer.companionId))}</p>${offerButton(offer)}</article>`).join('') || '<p>Befriend a Meadow companion, then train together to unlock a personal adventure at character level four.</p>'}</div>`}`;
}

export function personalQuestScene(snapshot: PersonalSnapshot): string | undefined {
  const quest = activePersonalQuest(snapshot);
  if (!quest) return undefined;
  const node = quest.node;
  const heading = node.type === 'handoff' ? 'What shall we explore next?' : quest.title;
  const choices =
    node.type === 'choice'
      ? `<div class="personal-quest-choices">${node.choices.map((choice) => `<button class="primary" data-server-personal-choice="${escape(choice.id)}">${escape(choice.label)}</button>`).join('')}</div>`
      : node.type === 'activity'
        ? `<button class="primary" id="server-personal-activity">Help ${escape(friendName(quest.companionId))} together</button><p class="tiny-note">You can try again, ask for help, or return to this moment later.</p>`
        : `<p class="personal-quest-memory">This episode joins your shared memories. Revisiting an episode keeps your choices without another keepsake.</p><p>Ideas for another day: ${node.continuationTags.map((tag) => escape(tag.replaceAll('-', ' '))).join(' · ')}.</p><div class="personal-quest-offers">${personalQuestOffers(
            snapshot,
          )
            .filter((offer) => offer.available)
            .map(
              (offer) => `<article><h3>${escape(offer.title)}</h3>${offerButton(offer)}</article>`,
            )
            .join(
              '',
            )}</div><button class="secondary" id="server-personal-browse">See all personal adventures</button>`;
  return `<section class="personal-quest-scene" data-personal-node="${escape(node.id)}" data-personal-node-type="${node.type}"><span class="pill">${node.type === 'handoff' ? 'MORE ADVENTURES AWAIT' : `WITH ${escape(friendName(quest.companionId).toUpperCase())}`}</span><h2 id="dialog-title">${escape(heading)}</h2>${characterPortrait(quest.companionId, 110)}${quest.acknowledgement ? `<p class="quest-acknowledgement" role="note">${escape(quest.acknowledgement)}</p>` : ''}<p class="personal-quest-story">${escape(node.text)}</p>${choices}${node.type === 'handoff' ? '' : '<button class="text-button" id="server-personal-abandon">Leave this episode</button><p class="tiny-note">Closing this window keeps your place for next time.</p>'}</section>`;
}
