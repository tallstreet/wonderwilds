import {
  trainingView,
  missionsView,
  journalView,
  bindTrainingActions,
  bindMissionsActions,
  bindJournalActions,
  captureJournalFocus,
  restoreJournalFocus,
} from './backend-world-views.ts';
import { isMapId } from './maps.ts';
import type { CommandPayload, GameSnapshot } from '../../contracts/generated/contracts.ts';
import type { Progress, WorldEncounter } from './types.ts';
import { isRecord } from './types.ts';
import { GameClient, ApiFailure } from './api/client.ts';
import { CommandSession } from './api/command-session.ts';
import { mountActivity } from './learning/activity.ts';
import { authoredPresentation } from './learning/presentation.ts';
import { heroes, heroPortrait } from './art.ts';
import { trainingLevelUp } from './companion-training.ts';
import './companion-training.css';
import './companions.css';
import { characterPortrait } from './character-art.ts';
import './character-art.css';
import { isBlocked } from './navigation.ts';
import {
  completedWorldQuests,
  friendName,
  friendSprites,
  meadowBeats,
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
  private commands: CommandSession;
  private activityOpen = false;
  private viewKey: string;
  constructor(
    client: GameClient,
    snapshot: GameSnapshot,
    profileId: string,
    private hooks: Hooks,
  ) {
    this.commands = new CommandSession(client, snapshot);
    this.viewKey = `wonderwilds.world-view.v1:${profileId}`;
  }
  get snapshot() {
    return this.commands.snapshot;
  }
  private get busy() {
    return this.commands.busy;
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
    document.querySelector('#star-count')!.textContent = `${count} / ${meadowBeats.length}`;
    document.querySelector('#mission-status')!.textContent = this.snapshot.pendingEncounters.length
      ? 'Continue your helping adventure'
      : count === meadowBeats.length
        ? 'Find your Meadow friends!'
        : `${meadowBeats.length - count} story moments await`;
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
      if (this.commands.hasPending) {
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
    try {
      await this.commands.send(payload);
      this.apply();
      document.querySelector('#server-error')?.replaceChildren();
    } catch (error) {
      this.fail(error);
      throw error;
    }
  }
  private async retry() {
    if (!(await this.commands.retry())) return;
    this.apply();
    this.reconcile();
  }
  async refresh() {
    if (this.busy) return;
    await this.commands.refresh();
    this.apply();
    this.reconcile();
  }
  private reconcile() {
    if (this.snapshot.pendingEncounters.length) this.mount();
    else {
      this.disposeActivity();
      this.missions();
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
    if (this.commands.hasPending) {
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
    if (this.busy || this.commands.hasPending) return;
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
  private get dialog() {
    return document.querySelector<HTMLDialogElement>('#dialog')!;
  }
  private bindTrainingActions() {
    bindTrainingActions(this.dialog, (id) => this.startTraining(id));
  }
  training() {
    if (this.busy || this.activityOpen) return;
    if (this.snapshot.pendingEncounters.length) {
      this.mount();
      return;
    }
    this.open(trainingView(this.snapshot));
    this.bindTrainingActions();
  }
  private mount() {
    this.disposeActivity();
    const encounter = this.snapshot.pendingEncounters[0];
    if (!encounter) return;
    const presentation = authoredPresentation(encounter);
    this.activityOpen = true;
    this.open(
      `<span class="pill">${encounter.training ? 'TRAIN TOGETHER' : 'YOUR MEADOW ADVENTURE'}</span><h2 id="dialog-title"${encounter.training ? ' class="training-heading"' : ''}>${encounter.training ? `Training with ${escape(friendName(encounter.training.creatureId))}` : 'A little help, a big difference'}</h2><div id="server-activity"></div>`,
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
    this.open(missionsView(this.snapshot));
    bindMissionsActions(this.dialog, {
      training: () => {
        this.hooks.close();
        this.hooks.visit(trainingClearing);
      },
      resume: () => this.mount(),
      refresh: () => {
        void this.refresh().catch((e) => this.fail(e));
      },
      visit: (quest) => {
        if (this.busy) return;
        if (this.snapshot.pendingEncounters.length) {
          this.mount();
          return;
        }
        this.hooks.close();
        this.hooks.visit(quest);
      },
    });
  }
  journal(state: Progress) {
    if (this.activityOpen || this.busy) return;
    this.open(journalView(this.snapshot, state));
    this.bindTrainingActions();
    bindJournalActions(this.dialog, {
      companion: (id) => {
        if (this.busy) return;
        const team = this.snapshot.selectedCompanions ?? [];
        const creatureIds = team.includes(id)
          ? team.filter((member) => member !== id)
          : [...team, id];
        void this.send({ type: 'select_team', creatureIds })
          .then(() => {
            this.journal(state);
            Array.from(this.dialog.querySelectorAll<HTMLButtonElement>('[data-server-companion]'))
              .find((button) => button.dataset.serverCompanion === id)
              ?.focus();
            this.hooks.toast('Your walking team is saved.');
          })
          .catch(() => {});
      },
      friendship: (payload, source) => {
        if (this.busy) return;
        void this.send(payload)
          .then(() => {
            const focus = captureJournalFocus(this.dialog, source);
            this.journal(state);
            restoreJournalFocus(this.dialog, focus);
            this.hooks.toast(
              payload.type === 'help_friend'
                ? 'A shared memory, saved together!'
                : 'Your friend’s glow is saved.',
            );
          })
          .catch(() => {});
      },
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
