import { isMapId } from './maps.ts';
import type { CommandPayload, GameSnapshot } from '../../contracts/generated/contracts.ts';
import type { Progress, WorldEncounter } from './types.ts';
import { isRecord } from './types.ts';
import { GameClient, ApiFailure, type PreparedCommand } from './api/client.ts';
import { mountActivity } from './learning/activity.ts';
import { authoredPresentation } from './learning/presentation.ts';
import { heroes, heroPortrait } from './art.ts';
import { companionDetails } from './companions.ts';
import { trainingDetails, trainingLevelUp } from './companion-training.ts';
import {
  activePersonalQuest,
  personalQuestDetails,
  personalQuestLibrary,
  personalQuestScene,
} from './personal-quests.ts';
import './companion-training.css';
import './personal-quests.css';
import './companions.css';
import { characterPortrait } from './character-art.ts';
import './character-art.css';
import { isBlocked } from './navigation.ts';
import {
  completedWorldQuests,
  friendName,
  friendSprites,
  meadowBeats,
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
type Hooks = {
  open(html: string): void;
  close(): void;
  apply(snapshot: GameSnapshot): void;
  visit(quest: WorldEncounter): void;
  toast(message: string): void;
};
export class BackendWorld {
  private activity?: ReturnType<typeof mountActivity>;
  private pending?: { payload: CommandPayload; command: PreparedCommand };
  private busy = false;
  private activityOpen = false;
  private viewKey: string;
  constructor(
    private client: GameClient,
    public snapshot: GameSnapshot,
    profileId: string,
    private hooks: Hooks,
  ) {
    this.viewKey = `wonderwilds.world-view.v1:${profileId}`;
  }
  get ownsActivity() {
    return this.activityOpen;
  }
  get working() {
    return this.busy;
  }
  get markers() {
    return [...worldQuests, trainingClearing];
  }
  companions() {
    return (this.snapshot.selectedCompanions ?? [])
      .filter((id) => this.snapshot.ownedCreatures.includes(id))
      .slice(0, this.snapshot.teamCapacity ?? 1)
      .map((id) => ({ sprite: friendSprites[id] ?? 'pip', name: friendName(id) }));
  }
  companion() {
    return this.companions()[0];
  }
  restoreView(state: Progress) {
    try {
      const saved: unknown = JSON.parse(localStorage.getItem(this.viewKey) ?? 'null');
      if (!isRecord(saved)) return;
      if (typeof saved.hero === 'string' && Object.hasOwn(heroes, saved.hero))
        state.hero = saved.hero;
      if (isMapId(saved.region)) state.region = saved.region;
      const p = saved.position;
      if (
        isRecord(p) &&
        typeof p.x === 'number' &&
        typeof p.y === 'number' &&
        Number.isFinite(p.x) &&
        Number.isFinite(p.y) &&
        !isBlocked(p.x, p.y, questCompleted(this.snapshot, 'meadow-bridge'), state.region)
      )
        state.position = { x: p.x, y: p.y };
    } catch {
      /* Server progress remains available when local storage is blocked. */
    }
  }
  saveView(state: Progress) {
    try {
      localStorage.setItem(
        this.viewKey,
        JSON.stringify({ hero: state.hero, region: state.region, position: state.position }),
      );
    } catch {
      /* Cosmetic preference only. */
    }
  }
  updateHud() {
    const count = meadowBeats.filter((b) => questCompleted(this.snapshot, b.id)).length;
    document.querySelector('#star-count')!.textContent = `${count} / 6`;
    document.querySelector('#mission-status')!.textContent = this.snapshot.pendingEncounters.length
      ? 'Continue your helping adventure'
      : this.snapshot.personalQuest && this.snapshot.personalQuest.node.type !== 'handoff'
        ? `Continue ${this.snapshot.personalQuest.title}`
        : count === 6
          ? 'Find your Meadow friends!'
          : `${6 - count} story moments await`;
    document.querySelector('#save-status')!.textContent =
      `Local server · progress saved · revision ${this.snapshot.revision}`;
  }
  private apply() {
    this.hooks.apply(this.snapshot);
  }
  private fail(error: unknown) {
    const conflict = error instanceof ApiFailure && error.code === 'REVISION_CONFLICT';
    this.hooks.toast(
      conflict
        ? 'Progress changed in another tab. Refresh server progress to continue.'
        : 'Could not save that action. Your progress is safe; retry or refresh to reconnect.',
    );
    const host = document.querySelector('#server-error');
    if (host) {
      host.replaceChildren();
      const message = document.createElement('p');
      message.textContent = conflict
        ? 'Another tab changed this adventure.'
        : 'The server could not confirm that action.';
      host.append(message);
      if (this.pending) {
        const retry = document.createElement('button');
        retry.textContent = 'Retry saving';
        retry.className = 'secondary';
        retry.onclick = () => {
          if (!this.busy) void this.retry().catch((e) => this.fail(e));
        };
        host.append(retry);
      }
      const refresh = document.createElement('button');
      refresh.textContent = 'Refresh server progress';
      refresh.className = 'secondary';
      refresh.onclick = () => {
        if (!this.busy) void this.refresh().catch((e) => this.fail(e));
      };
      host.append(refresh);
    }
  }
  private async send(payload: CommandPayload) {
    if (this.busy) throw new Error('An action is still saving');
    if (this.pending && JSON.stringify(this.pending.payload) !== JSON.stringify(payload))
      throw new Error('Retry the pending action first');
    this.pending ??= {
      payload,
      command: this.client.prepare({
        schemaVersion: '1.0',
        commandId: crypto.randomUUID(),
        expectedRevision: this.snapshot.revision,
        payload,
      }),
    };
    this.busy = true;
    try {
      this.snapshot = await this.client.send(this.pending.command);
      this.pending = undefined;
      this.apply();
      document.querySelector('#server-error')?.replaceChildren();
    } catch (error) {
      if (error instanceof ApiFailure && !['UNAVAILABLE', 'RATE_LIMITED'].includes(error.code))
        this.pending = undefined;
      this.fail(error);
      throw error;
    } finally {
      this.busy = false;
    }
  }
  private async retry() {
    if (!this.pending) return;
    await this.send(this.pending.payload);
    this.reconcile();
  }
  async refresh() {
    if (this.busy) return;
    this.busy = true;
    try {
      this.snapshot = await this.client.snapshot();
      this.pending = undefined;
      this.apply();
    } finally {
      this.busy = false;
    }
    this.reconcile();
  }
  private reconcile() {
    if (this.snapshot.pendingEncounters.length) this.mount();
    else {
      this.disposeActivity();
      if (activePersonalQuest(this.snapshot)) this.personalQuest();
      else this.missions();
    }
  }
  private open(html: string) {
    this.hooks.open(`${html}<div id="server-error" role="status"></div>`);
  }
  private disposeActivity() {
    this.activity?.destroy();
    this.activity = undefined;
    this.activityOpen = false;
  }
  async leave() {
    if (this.busy) {
      this.hooks.toast('Wait for your adventure to finish saving.');
      return;
    }
    if (this.pending) {
      this.fail(new Error('Resolve the uncertain action first'));
      return;
    }
    const encounter = this.snapshot.pendingEncounters[0];
    try {
      if (encounter) await this.send({ type: 'abandon', encounterId: encounter.id });
      this.disposeActivity();
      this.hooks.close();
    } catch {
      /* Keep the dialog and exact pending command available for retry. */
    }
  }
  encounter(quest: WorldEncounter) {
    if (this.busy) return;
    if (quest.id === trainingClearing.id) {
      this.training();
      return;
    }
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    if (questCompleted(this.snapshot, quest.id)) {
      this.hooks.toast('You already helped here! Open Missions to find your next adventure.');
      return;
    }
    if (!questReady(this.snapshot, quest.id)) {
      this.hooks.toast('There is more to do first. Open Missions to find your next story moment.');
      return;
    }
    this.open(
      `<span class="pill">A HELPING MOMENT</span><h2 id="dialog-title">${escape(quest.short)}</h2><p>Let’s see how we can help.</p>`,
    );
    // Keep movement paused while the server issues the encounter.
    this.activityOpen = true;
    void this.send({ type: 'start_encounter', beatId: quest.id })
      .then(() => this.mount())
      .catch(() => {});
  }
  private startTraining(trainingId: string) {
    if (this.busy || this.pending) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    this.open(
      '<span class="pill">TRAIN TOGETHER</span><h2 id="dialog-title">Getting ready for training</h2>',
    );
    this.activityOpen = true;
    void this.send({ type: 'start_encounter', beatId: trainingId })
      .then(() => this.mount())
      .catch(() => {
        this.activityOpen = false;
      });
  }
  private bindTrainingActions() {
    document.querySelectorAll<HTMLButtonElement>('[data-server-training]').forEach((button) => {
      button.onclick = () => this.startTraining(button.dataset.serverTraining!);
    });
  }
  training() {
    if (this.busy || this.activityOpen) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    const friends = meadowFriends.filter((q) =>
      this.snapshot.ownedCreatures.includes(q.creatureId),
    );
    this.open(
      `<span class="pill">MEADOW TRAINING CLEARING</span><h2 id="dialog-title">Grow together</h2><p>Choose a friend for a little practice. You can ask for help at every step.</p><div class="journal-entries">${friends.map((q) => `<div class="journal-entry earned">${characterPortrait(q.creatureId, 110)}<strong>${escape(friendName(q.creatureId))}</strong>${trainingDetails(q.creatureId, this.snapshot)}</div>`).join('') || '<p>Befriend a Meadow character to start training together.</p>'}</div>`,
    );
    this.bindTrainingActions();
  }
  personalAdventures() {
    if (this.busy || this.activityOpen) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    this.open(personalQuestLibrary(this.snapshot));
    this.bindPersonalActions();
  }
  personalQuest() {
    if (this.busy || this.activityOpen) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    const html = personalQuestScene(this.snapshot);
    if (!html) {
      this.personalAdventures();
      return;
    }
    this.open(html);
    this.bindPersonalActions();
    const heading = document.querySelector<HTMLElement>('#dialog-title');
    if (heading) {
      heading.tabIndex = -1;
      heading.focus();
    }
  }
  private personalAction(payload: CommandPayload) {
    if (this.busy || this.pending || this.activityOpen) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    void this.send(payload)
      .then(() => {
        if (this.snapshot.pendingEncounters.length) this.mount();
        else if (payload.type === 'abandon_personal_quest') this.personalAdventures();
        else this.personalQuest();
      })
      .catch(() => {});
  }
  private bindPersonalActions() {
    document
      .querySelectorAll<HTMLButtonElement>('[data-server-personal-quest]')
      .forEach((button) => {
        button.onclick = () =>
          this.personalAction({
            type: 'start_personal_quest',
            questId: button.dataset.serverPersonalQuest!,
          });
      });
    document
      .querySelectorAll<HTMLButtonElement>('[data-server-personal-choice]')
      .forEach((button) => {
        button.onclick = () =>
          this.personalAction({
            type: 'choose_personal_path',
            choiceId: button.dataset.serverPersonalChoice!,
          });
      });
    document
      .querySelectorAll<HTMLButtonElement>('[data-server-personal-resume], #server-personal-resume')
      .forEach((button) => {
        button.onclick = () => this.personalQuest();
      });
    document.querySelector('#server-personal-activity')?.addEventListener('click', () => {
      this.personalAction({ type: 'start_personal_activity' });
    });
    document.querySelector('#server-personal-abandon')?.addEventListener('click', () => {
      this.personalAction({ type: 'abandon_personal_quest' });
    });
    document.querySelector('#server-personal-browse')?.addEventListener('click', () => {
      this.personalAdventures();
    });
  }
  private mount() {
    this.disposeActivity();
    const encounter = this.snapshot.pendingEncounters[0];
    if (!encounter) return;
    const presentation = authoredPresentation(encounter);
    const personal = encounter.personalQuest ? activePersonalQuest(this.snapshot) : undefined;
    this.activityOpen = true;
    this.open(
      `<span class="pill">${personal ? 'OUR PERSONAL ADVENTURE' : encounter.training ? 'TRAIN TOGETHER' : 'YOUR MEADOW ADVENTURE'}</span><h2 id="dialog-title"${personal ? ' class="personal-quest-heading"' : encounter.training ? ' class="training-heading"' : ''}>${personal ? escape(personal.title) : encounter.training ? `Training with ${escape(friendName(encounter.training.creatureId))}` : 'A little help, a big difference'}</h2>${personal ? `<p class="personal-quest-activity-context">${escape(personal.node.text)}</p>` : ''}<div id="server-activity"></div>`,
    );
    if (!presentation) {
      document.querySelector('#server-activity')!.textContent =
        'This activity needs an updated presentation. Close it to return safely to the world.';
      return;
    }
    this.activity = mountActivity(
      document.querySelector<HTMLElement>('#server-activity')!,
      encounter,
      {
        onAttempt: async (answer) => {
          const before = this.snapshot;
          await this.send({ type: 'attempt', encounterId: encounter.id, answer });
          if (!this.snapshot.pendingEncounters.some((e) => e.id === encounter.id)) {
            this.disposeActivity();
            if (encounter.personalQuest) {
              this.personalQuest();
              return {
                message: 'We helped together. Let’s see where our adventure goes next!',
                result: 'correct',
              };
            }
            const levelUp = encounter.training
              ? trainingLevelUp(before, this.snapshot, encounter.training.creatureId)
              : undefined;
            const friend = encounter.training?.creatureId;
            this.open(
              `<div class="success ${levelUp ? 'training-celebration' : ''}"><span class="pill">${levelUp ? 'LEVEL UP!' : encounter.training ? 'TRAINING COMPLETE!' : 'YOU HELPED!'}</span><h2 id="dialog-title">${levelUp ? `${escape(friendName(levelUp.creatureId))} reached level ${levelUp.level}!` : encounter.training ? 'Growing together' : 'A brighter Meadow'}</h2><p>${levelUp ? 'A happy wiggle for a new adventure milestone! Your character progress is saved.' : encounter.training ? 'Your practice and character progress are saved. Help is always welcome.' : 'Your adventure and learning progress are saved. Head back outside to see what changed!'}</p>${friend ? characterPortrait(friend, 110) : heroPortrait('pip', 110)}<button class="primary" id="server-return">Back to exploring →</button></div>`,
            );
            document.querySelector<HTMLButtonElement>('#server-return')!.onclick = () =>
              this.hooks.close();
            return { message: 'You did it!', result: 'correct' };
          }
          return {
            message: 'Not quite yet. Try another idea, or ask for a little help.',
            result: 'incorrect',
          };
        },
        onSupport: async (kind) => {
          await this.send({ type: 'hint', encounterId: encounter.id, supportKind: kind });
          return {
            message: `${presentation.prompt} Take it one step at a time with a helper. This help is recorded as support.`,
          };
        },
        onAbandon: () => this.leave(),
      },
      presentation,
    );
    this.activity.focus();
  }
  missions() {
    if (this.activityOpen || this.busy) return;
    const rows = worldQuests
      .map((q) => {
        const done = questCompleted(this.snapshot, q.id),
          ready = questReady(this.snapshot, q.id);
        return `<button class="quest ${done ? 'complete' : ''}" data-server-visit="${q.id}" ${done || !ready ? 'disabled' : ''}><span class="quest-icon">${done ? '✓' : ready ? '✦' : '◇'}</span><span><strong>${escape(q.short)}</strong><small>${done ? 'A lovely memory' : ready ? 'Walk to this adventure' : 'Complete earlier story moments first'}</small></span><span>→</span></button>`;
      })
      .join('');
    this.open(
      `<span class="pill">CLOVER MEADOW</span><h2 id="dialog-title">Our next adventure</h2><p>Choose a story moment or a friend. We’ll walk there together.</p>${this.snapshot.pendingEncounters.length ? '<button class="primary" id="server-resume">Continue your activity</button>' : ''}${this.snapshot.personalQuest ? '<button class="primary" id="server-personal-resume">Continue your personal adventure</button>' : ''}${this.snapshot.personalQuestOffers ? '<button class="secondary" id="server-personal-adventures">Personal adventures with friends</button>' : ''}<button class="secondary" id="server-training-clearing">Visit the training clearing</button><div class="modal-quests">${rows}</div><button class="text-button" id="server-refresh">Refresh server progress</button><p class="tiny-note">This local playtest uses draft activities. More islands are still being connected.</p>`,
    );
    this.bindPersonalActions();
    document
      .querySelector('#server-personal-adventures')
      ?.addEventListener('click', () => this.personalAdventures());
    document.querySelector('#server-training-clearing')?.addEventListener('click', () => {
      this.hooks.close();
      this.hooks.visit(trainingClearing);
    });
    document.querySelector('#server-resume')?.addEventListener('click', () => this.mount());
    document.querySelector('#server-refresh')?.addEventListener('click', () => {
      void this.refresh().catch((e) => this.fail(e));
    });
    document.querySelectorAll<HTMLButtonElement>('[data-server-visit]').forEach((button) => {
      button.onclick = () => {
        const quest = worldQuests.find((q) => q.id === button.dataset.serverVisit);
        if (!quest || this.busy) return;
        if (this.snapshot.pendingEncounters.length) {
          this.mount();
          return;
        }
        this.hooks.close();
        this.hooks.visit(quest);
      };
    });
  }
  journal(state: Progress) {
    if (this.activityOpen || this.busy) return;
    const cards = meadowFriends
      .map((q) => {
        const owned = this.snapshot.ownedCreatures.includes(q.creatureId),
          selected = this.snapshot.selectedCompanions?.includes(q.creatureId);
        return `<div class="journal-entry ${owned ? 'earned' : ''}">${characterPortrait(q.creatureId, 110)}<strong>${friendName(q.creatureId)}</strong><small>${owned ? (selected ? 'Walking with you' : 'Your friend') : 'A friendship waiting to grow'}</small>${owned ? `<button class="secondary" data-server-companion="${q.creatureId}" ${!selected && (this.snapshot.selectedCompanions?.length ?? 0) >= (this.snapshot.teamCapacity ?? 1) ? 'disabled' : ''} aria-pressed="${Boolean(selected)}">${selected ? `Rest ${friendName(q.creatureId)}` : `Walk with ${friendName(q.creatureId)}`}</button>` : ''}${trainingDetails(q.creatureId, this.snapshot)}${personalQuestDetails(q.creatureId, this.snapshot)}${companionDetails(q.creatureId, owned, this.snapshot)}</div>`;
      })
      .join('');
    this.open(
      `<span class="pill">OUR LITTLE MEMORIES</span><h2 id="dialog-title">My adventure journal</h2><h3>Choose your hero</h3><div class="hero-choices">${Object.entries(
        heroes,
      )
        .filter(([id]) => id !== 'pip')
        .map(
          ([id, h]) =>
            `<button data-hero="${id}" class="hero-choice ${state.hero === id ? 'selected' : ''}" aria-label="Play as ${h.name}">${heroPortrait(id, 60)}<strong>${h.name}</strong></button>`,
        )
        .join(
          '',
        )}</div>${this.snapshot.explorerProgression ? `<p class="explorer-level">Explorer level ${this.snapshot.explorerProgression.level} · ${this.snapshot.explorerProgression.xp} adventure XP</p>` : ''}<h3>Your Meadow friends</h3><p>Team: ${this.snapshot.selectedCompanions?.length ?? 0} / ${this.snapshot.teamCapacity ?? 1} companions. Let a friend rest to make room.</p><p><a href="/characters.html">Meet all twelve Wonderwilds friends</a></p><div class="journal-entries">${cards}</div><h3>Collected treasures</h3><p>${this.snapshot.inventory.map((i) => `${escape(i.itemId)} × ${i.quantity}`).join(' · ') || 'Help your neighbours to collect rewards.'}</p><p class="tiny-note">Your companion, character progress and shared memories are saved on the server.</p>`,
    );
    this.bindFriendshipActions(state);
    this.bindTrainingActions();
    this.bindPersonalActions();
    document.querySelectorAll<HTMLButtonElement>('[data-server-companion]').forEach((button) => {
      button.onclick = () => {
        if (this.busy) return;
        const id = button.dataset.serverCompanion!;
        const team = this.snapshot.selectedCompanions ?? [];
        const creatureIds = team.includes(id)
          ? team.filter((member) => member !== id)
          : [...team, id];
        void this.send({ type: 'select_team', creatureIds })
          .then(() => {
            this.journal(state);
            document.querySelector<HTMLButtonElement>(`[data-server-companion="${id}"]`)?.focus();
            this.hooks.toast('Your walking team is saved.');
          })
          .catch(() => {});
      };
    });
  }
  private refreshFriendshipJournal(state: Progress, source: HTMLButtonElement) {
    const expanded = new Set(
      Array.from(document.querySelectorAll<HTMLDetailsElement>('.companion-details[open]')).map(
        (details) => details.querySelector('summary')?.textContent,
      ),
    );
    const sourceSummary = source.closest('details')?.querySelector('summary')?.textContent;
    this.journal(state);
    let fallback: HTMLElement | undefined;
    document.querySelectorAll<HTMLDetailsElement>('.companion-details').forEach((details) => {
      const summary = details.querySelector('summary');
      details.open = expanded.has(summary?.textContent);
      if (summary && summary.textContent === sourceSummary) fallback = summary;
    });
    const action = Array.from(
      document.querySelectorAll<HTMLButtonElement>(
        '[data-server-help-friend], [data-server-transform]',
      ),
    ).find((button) =>
      source.dataset.serverTransform
        ? button.dataset.serverTransform === source.dataset.serverTransform
        : button.dataset.serverHelpFriend === source.dataset.serverHelpFriend &&
          button.dataset.completedQuest === source.dataset.completedQuest,
    );
    // A remembered moment disappears after acknowledgement; its summary remains a stable target.
    (action ?? fallback)?.focus();
  }
  private bindFriendshipActions(state: Progress) {
    document.querySelectorAll<HTMLButtonElement>('[data-server-help-friend]').forEach((button) => {
      button.onclick = () => {
        if (this.busy) return;
        void this.send({
          type: 'help_friend',
          creatureId: button.dataset.serverHelpFriend!,
          completedQuestId: button.dataset.completedQuest!,
        })
          .then(() => {
            this.refreshFriendshipJournal(state, button);
            this.hooks.toast('A shared memory, saved together!');
          })
          .catch(() => {});
      };
    });
    document.querySelectorAll<HTMLButtonElement>('[data-server-transform]').forEach((button) => {
      button.onclick = () => {
        if (this.busy) return;
        void this.send({
          type: 'transform_companion',
          creatureId: button.dataset.serverTransform!,
          enabled: button.dataset.transformEnabled === 'true',
        })
          .then(() => {
            this.refreshFriendshipJournal(state, button);
            this.hooks.toast('Your friend’s glow is saved.');
          })
          .catch(() => {});
      };
    });
  }
  grownup() {
    if (this.activityOpen || this.busy) return;
    this.open(
      `<span class="pill">A WINDOW INTO THEIR LEARNING</span><h2 id="dialog-title">Little steps, saved discoveries</h2><p>Answers, support, story rewards and friendships are saved by the local server. Movement and hero appearance stay on this device. Guest adventure saves are separate.</p><div class="skill-list">${this.snapshot.learnerSummaries.map((s) => `<div><strong>${escape(s.skillId)} · ${escape(s.status)}</strong><p>${escape(s.explanation)}</p></div>`).join('') || '<p>Try a story activity to begin your learning record.</p>'}</div><p>These experimental observations are not a validated mastery judgement. Help is recorded as support; repeating fixed tasks does not establish transfer. Hints currently repeat the instruction; narrated and worked examples are still pending.</p><details><summary>Server progress and playtest scope</summary><pre id="server-snapshot"></pre><p>Meadow story and friendships are connected. Regional travel, director integration, accounts and save import remain pending. Friendship memories and reversible glow use server progress in this local preview.</p></details><button class="secondary" id="server-download">Download progress snapshot</button><button class="text-button" id="server-refresh">Refresh server progress</button>`,
    );
    document.querySelector('#server-snapshot')!.textContent = JSON.stringify(
      this.snapshot,
      null,
      2,
    );
    document.querySelector('#server-refresh')!.addEventListener('click', () => {
      void this.refresh().catch((e) => this.fail(e));
    });
    document.querySelector('#server-download')!.addEventListener('click', () => {
      const url = URL.createObjectURL(
        new Blob([JSON.stringify(this.snapshot, null, 2)], { type: 'application/json' }),
      );
      const a = document.createElement('a');
      a.href = url;
      a.download = 'wonderwilds-server-progress.json';
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    });
  }
}
export async function connectBackend(hooks: Hooks): Promise<BackendWorld | undefined> {
  const response = await fetch('/__demo/bootstrap');
  if (!response.ok || !response.headers.get('content-type')?.includes('application/json')) return;
  const session: unknown = await response.json();
  if (
    !isRecord(session) ||
    typeof session.csrf !== 'string' ||
    typeof session.profileId !== 'string'
  )
    throw new Error('Invalid local session');
  const csrf = session.csrf;
  const client = new GameClient(() => csrf);
  client.selectProfile(session.profileId);
  return new BackendWorld(client, await client.snapshot(), session.profileId, hooks);
}
export { completedWorldQuests };
