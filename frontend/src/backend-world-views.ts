import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import type { Progress, WorldEncounter } from './types.ts';
import { maps } from './maps.ts';
import { heroes, heroPortrait } from './art.ts';
import { companionDetails } from './companions.ts';
import { trainingDetails } from './companion-training.ts';
import { characterPortrait } from './character-art.ts';
import {
  friendName,
  meadowFriends,
  questCompleted,
  questReady,
  worldQuests,
  trainingClearing,
} from './backend-world-model.ts';

const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );

export function trainingView(snapshot: GameSnapshot): string {
  const friends = meadowFriends.filter((q) => snapshot.ownedCreatures.includes(q.creatureId));
  return `<span class="pill">MEADOW TRAINING CLEARING</span><h2 id="dialog-title">Grow together</h2><p>Choose a friend for a little practice. You can ask for help at every step.</p><div class="journal-entries">${friends.map((q) => `<div class="journal-entry earned">${characterPortrait(q.creatureId, 110)}<strong>${escape(friendName(q.creatureId))}</strong>${trainingDetails(q.creatureId, snapshot)}</div>`).join('') || '<p>Befriend a Meadow character to start training together.</p>'}</div>`;
}

export function missionsView(snapshot: GameSnapshot, trail = ''): string {
  const rows = worldQuests
    .map((q) => {
      const done = questCompleted(snapshot, q.id),
        ready = questReady(snapshot, q.id);
      return `<button class="quest ${done ? 'complete' : ''}" data-server-visit="${q.id}" ${done || !ready ? 'disabled' : ''}><span class="quest-icon">${done ? '✓' : ready ? '✦' : '◇'}</span><span><strong>${escape(q.short)}</strong><small>${escape(maps[q.region].name)} · ${done ? 'A lovely memory' : ready ? 'Track this adventure' : 'Complete earlier story moments first'}</small></span><span>→</span></button>`;
    })
    .join('');
  return `<span class="pill">CLOVER MEADOW</span><h2 id="dialog-title">Our next adventure</h2><p>Track a story moment or a friend, then follow the paths to meet them.</p>${trail}${snapshot.pendingEncounters.length ? '<button class="primary" id="server-resume">Continue your activity</button>' : ''}<button class="secondary" id="server-training-clearing">Track the training clearing · ${escape(maps[trainingClearing.region].name)}</button><div class="modal-quests">${rows}</div><button class="text-button" id="server-refresh">Refresh server progress</button><p class="tiny-note">This local playtest uses draft activities. More islands are still being connected.</p>`;
}

export function journalView(snapshot: GameSnapshot, state: Pick<Progress, 'hero'>): string {
  const cards = meadowFriends
    .map((q) => {
      const owned = snapshot.ownedCreatures.includes(q.creatureId),
        selected = snapshot.selectedCompanions?.includes(q.creatureId);
      return `<div class="journal-entry ${owned ? 'earned' : ''}">${characterPortrait(q.creatureId, 110)}<strong>${friendName(q.creatureId)}</strong><small>${owned ? (selected ? 'Walking with you' : 'Your friend') : 'A friendship waiting to grow'}</small>${owned ? `<button class="secondary" data-server-companion="${q.creatureId}" ${!selected && (snapshot.selectedCompanions?.length ?? 0) >= (snapshot.teamCapacity ?? 1) ? 'disabled' : ''} aria-pressed="${Boolean(selected)}">${selected ? `Rest ${friendName(q.creatureId)}` : `Walk with ${friendName(q.creatureId)}`}</button>` : ''}${trainingDetails(q.creatureId, snapshot)}${companionDetails(q.creatureId, owned, snapshot)}</div>`;
    })
    .join('');
  return `<span class="pill">OUR LITTLE MEMORIES</span><h2 id="dialog-title">My adventure journal</h2><h3>Choose your hero</h3><div class="hero-choices">${Object.entries(
    heroes,
  )
    .filter(([id]) => id !== 'pip')
    .map(
      ([id, h]) =>
        `<button data-hero="${id}" class="hero-choice ${state.hero === id ? 'selected' : ''}" aria-label="Play as ${h.name}">${heroPortrait(id, 60)}<strong>${h.name}</strong></button>`,
    )
    .join(
      '',
    )}</div>${snapshot.explorerProgression ? `<p class="explorer-level">Explorer level ${snapshot.explorerProgression.level} · ${snapshot.explorerProgression.xp} adventure XP</p>` : ''}<h3>Your Meadow friends</h3><p>Team: ${snapshot.selectedCompanions?.length ?? 0} / ${snapshot.teamCapacity ?? 1} companions. Let a friend rest to make room.</p><p><a href="/characters.html">Meet all twelve Wonderwilds friends</a></p><div class="journal-entries">${cards}</div><h3>Collected treasures</h3><p>${snapshot.inventory.map((i) => `${escape(i.itemId)} × ${i.quantity}`).join(' · ') || 'Help your neighbours to collect rewards.'}</p><p class="tiny-note">Your companion, character progress and shared memories are saved on the server.</p>`;
}

export function bindTrainingActions(host: HTMLElement, start: (id: string) => void) {
  host.querySelectorAll<HTMLButtonElement>('[data-server-training]').forEach((button) => {
    button.onclick = () => start(button.dataset.serverTraining!);
  });
}
export function bindMissionsActions(
  host: HTMLElement,
  actions: {
    training(): void;
    resume(): void;
    refresh(): void;
    visit(quest: WorldEncounter): void;
  },
) {
  host.querySelector('#server-training-clearing')?.addEventListener('click', actions.training);
  host.querySelector('#server-resume')?.addEventListener('click', actions.resume);
  host.querySelector('#server-refresh')?.addEventListener('click', actions.refresh);
  host.querySelectorAll<HTMLButtonElement>('[data-server-visit]').forEach((button) => {
    button.onclick = () => {
      const quest = worldQuests.find((q) => q.id === button.dataset.serverVisit);
      if (quest) actions.visit(quest);
    };
  });
}
type FriendshipCommand = Extract<
  import('../../contracts/generated/contracts.ts').CommandPayload,
  { type: 'help_friend' | 'transform_companion' }
>;
export function bindJournalActions(
  host: HTMLElement,
  actions: {
    companion(id: string): void;
    friendship(payload: FriendshipCommand, source: HTMLButtonElement): void;
  },
) {
  host.querySelectorAll<HTMLButtonElement>('[data-server-companion]').forEach((button) => {
    button.onclick = () => actions.companion(button.dataset.serverCompanion!);
  });
  host.querySelectorAll<HTMLButtonElement>('[data-server-help-friend]').forEach((button) => {
    button.onclick = () =>
      actions.friendship(
        {
          type: 'help_friend',
          creatureId: button.dataset.serverHelpFriend!,
          completedQuestId: button.dataset.completedQuest!,
        },
        button,
      );
  });
  host.querySelectorAll<HTMLButtonElement>('[data-server-transform]').forEach((button) => {
    button.onclick = () =>
      actions.friendship(
        {
          type: 'transform_companion',
          creatureId: button.dataset.serverTransform!,
          enabled: button.dataset.transformEnabled === 'true',
        },
        button,
      );
  });
}
export function captureJournalFocus(host: HTMLElement, source: HTMLButtonElement) {
  return {
    expanded: new Set(
      Array.from(host.querySelectorAll<HTMLDetailsElement>('.companion-details[open]')).map(
        (details) => details.dataset.companionId,
      ),
    ),
    creatureId: source.closest<HTMLElement>('[data-companion-id]')?.dataset.companionId,
    transform: source.dataset.serverTransform,
    helping: source.dataset.serverHelpFriend,
    quest: source.dataset.completedQuest,
  };
}
export function restoreJournalFocus(
  host: HTMLElement,
  focus: ReturnType<typeof captureJournalFocus>,
) {
  let fallback: HTMLElement | null = null;
  host.querySelectorAll<HTMLDetailsElement>('.companion-details').forEach((details) => {
    details.open = focus.expanded.has(details.dataset.companionId);
    if (details.dataset.companionId === focus.creatureId)
      fallback = details.querySelector('summary');
  });
  const action = Array.from(
    host.querySelectorAll<HTMLButtonElement>('[data-server-help-friend], [data-server-transform]'),
  ).find((button) =>
    focus.transform
      ? button.dataset.serverTransform === focus.transform
      : button.dataset.serverHelpFriend === focus.helping &&
        button.dataset.completedQuest === focus.quest,
  );
  // A remembered action disappears after acknowledgement; its creature's summary persists.
  (action ?? fallback)?.focus();
}
