import type Phaser from 'phaser';
import type { StorySnapshot } from '../../../contracts/generated/contracts.ts';
import type { WorldEncounter } from '../types.ts';
import { isRecord } from '../types.ts';
import { freshState } from '../progress.ts';
import { createWorld } from '../world.ts';
import { StoryClient, StorySession, type StoryPayload } from './client.ts';
import {
  storyPlaces,
  storyBounds,
  storyMarkers,
  bridgeRepaired,
  actionTarget,
  storyDecorations,
  storySceneryFlags,
  crossingRoute,
  nextStoryLabel,
} from './world-model.ts';
const escape = (value: string) =>
  value.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );
type Hooks = { open(html: string): void; close(): void; toast(text: string): void };
export class StoryWorld {
  private state = freshState();
  private game!: Phaser.Game;
  private commands: StorySession;
  private crossing = false;
  private escort?: { id: string; sprite: string; name: string; offsetX: number };
  private location?: StorySnapshot['locationId'];
  private disposed = false;
  private applied?: StorySnapshot;
  constructor(
    private client: StoryClient,
    profile: string,
    private hooks: Hooks,
  ) {
    this.commands = new StorySession(client, profile, sessionStorage);
    this.state.hero = 'rose';
    document.querySelector<HTMLElement>('.hud-stars')!.style.display = 'none';
    window.addEventListener('pagehide', () => {
      this.disposed = true;
      this.client.destroy();
    });
    window.addEventListener('pageshow', (event) => {
      if (event.persisted) location.reload();
    });
  }
  get snapshot() {
    return this.applied!;
  }
  get markers() {
    return storyMarkers(this.snapshot).filter(
      (marker) => !this.crossing || marker.id !== `npc:${this.escort?.id}`,
    );
  }
  async start() {
    await this.commands.refresh();
    if (this.disposed) throw new Error('Story page is no longer active');
    this.apply();
    if (this.snapshot.commandBudgetExhausted) {
      this.missions();
      return undefined;
    }
    this.game = createWorld(
      'game',
      () => this.state,
      (marker) => this.encounter(marker),
      () => this.missions(),
      (point) => {
        this.state.position = point;
        this.positionMetadata(point.x, point.y);
      },
      undefined,
      {
        encounters: () => this.markers,
        companions: () => (this.crossing && this.escort ? [this.escort] : []),
        decorations: () => storyDecorations(this.snapshot),
        allowExits: false,
        blocked: (x, y) => storyBounds(this.snapshot.locationId, this.crossing)(x, y),
        inputLocked: () => this.locked(),
        observe: (point, followers) => {
          this.positionMetadata(point.x, point.y);
          document.querySelector<HTMLElement>('#game')!.dataset.storyFollowers =
            JSON.stringify(followers);
        },
      },
    );
    this.hud();
    return this.game;
  }
  private positionMetadata(x: number, y: number) {
    const surface = document.querySelector<HTMLElement>('#game')!;
    surface.dataset.storyPlayerX = String(x);
    surface.dataset.storyPlayerY = String(y);
  }
  private apply() {
    this.applied = this.commands.snapshot!;
    const snapshot = this.snapshot;
    if (snapshot.commandBudgetExhausted) {
      this.game?.scene.stop('island');
      document.querySelector('#game')!.textContent =
        'This development story has reached its save limit. Your progress is preserved.';
      this.hud();
      return;
    }
    const changed = this.location !== snapshot.locationId;
    this.location = snapshot.locationId;
    this.state.region = storyPlaces[snapshot.locationId].region;
    this.state.done = storySceneryFlags(snapshot);
    if (changed) this.state.position = { ...storyPlaces[snapshot.locationId].spawn };
    this.game?.events.emit(changed ? 'authoritative-position' : 'refresh');
    this.positionMetadata(this.state.position.x, this.state.position.y);
    this.hud();
  }
  private hud() {
    const snapshot = this.snapshot;
    document.querySelector('.location-badge h1')!.textContent =
      storyPlaces[snapshot.locationId].name;
    document.querySelector('#star-count')!.textContent = 'Story';
    document.querySelector('#mission-status')!.textContent =
      snapshot.sceneInput?.mission?.current ??
      (snapshot.sceneInput?.mission?.status === 'completed'
        ? `Completed: ${snapshot.sceneInput.mission.goal}`
        : 'Find Pip to begin your story');
    document.querySelector('#save-status')!.textContent = this.commands.hasPending
      ? 'A save needs confirmation · open Missions'
      : this.commands.needsRefresh
        ? 'Refresh story progress · open Missions'
        : `Story saved · revision ${snapshot.revision}`;
    const surface = document.querySelector<HTMLElement>('#game')!;
    surface.dataset.storyLocation = snapshot.locationId;
    surface.dataset.storyRevision = String(snapshot.revision);
    surface.dataset.storyBridge = snapshot.commandBudgetExhausted
      ? 'unavailable'
      : bridgeRepaired(snapshot)
        ? 'repaired'
        : 'broken';
    surface.dataset.storyMarkers = JSON.stringify(this.markers);
    surface.dataset.storyCrossing = String(this.crossing);
    surface.dataset.storyMission = snapshot.sceneInput?.mission?.missionId ?? '';
    surface.dataset.storyVisuals = JSON.stringify(storyDecorations(snapshot));
    surface.setAttribute(
      'aria-label',
      `${storyPlaces[snapshot.locationId].name}. Tap to walk or use arrow keys. Space talks to a nearby friend. Open Missions to follow a path.`,
    );
  }
  private locked() {
    return (
      this.commands.busy ||
      this.commands.hasPending ||
      this.commands.needsRefresh ||
      this.crossing ||
      this.snapshot.commandBudgetExhausted
    );
  }
  handle(button: HTMLButtonElement) {
    if (
      button.id === 'missions' ||
      button.id === 'journal' ||
      button.id === 'explore' ||
      button.id === 'grownup'
    ) {
      this.missions();
      return;
    }
    if (button.dataset.storyVisit) {
      const marker = this.markers.find((marker) => marker.id === button.dataset.storyVisit);
      if (marker && !this.locked()) {
        this.hooks.close();
        this.game.events.emit('visit', marker);
      }
    }
  }
  missions() {
    if (this.crossing) return;
    if (this.snapshot.commandBudgetExhausted) {
      this.hooks.open(
        '<h2 id="dialog-title">This development story has reached its save limit</h2><p>Your saved progress is preserved. No further story actions are available in this run.</p><p><a href="/">Return to the adventure</a></p>',
      );
      return;
    }
    const input = this.snapshot.sceneInput;
    const mission = input?.mission;
    const missionControls = this.markers.some((marker) => marker.id === 'npc:pip')
      ? []
      : (input?.availableActions ?? []).filter((item) =>
          ['accept_mission', 'cancel_mission'].includes(item.action.type),
        );
    this.hooks.open(
      `<span class="pill">PIP’S STORY ADVENTURE</span><h2 id="dialog-title">${escape(mission?.goal ?? 'A way across, together')}</h2><p>${escape(mission?.narrative.premise ?? 'Find Pip by the bridge to begin.')}</p>${mission ? `<p data-story-mission-status="${mission.status}">${mission.status === 'completed' ? `Mission complete · ${escape(mission.narrative.payoff)}` : mission.status === 'abandoned' ? 'Mission paused · your progress is saved.' : escape(mission.current ?? '')}</p><ul>${mission.completed.map((text) => `<li>✓ ${escape(text)}</li>`).join('')}${mission.remaining.map((text) => `<li>${escape(text)}</li>`).join('')}</ul>` : ''}${nextStoryLabel(this.snapshot) ? `<p data-story-next>${escape(nextStoryLabel(this.snapshot)!)}</p>` : ''}${input?.context.memory.recentMissions.length ? `<section data-story-history><h3>Our adventures so far</h3><ul>${input.context.memory.recentMissions.map((text) => `<li>${escape(text)}</li>`).join('')}</ul></section>` : ''}<p id="story-world-inventory">Carrying: ${input?.context.inventory.map((item) => `${item.quantity} ${escape(item.itemId)}`).join(', ') || 'an empty bag'}</p><div class="modal-quests">${this.markers.map((marker) => `<button class="quest" data-story-visit="${escape(marker.id)}" ${this.locked() ? 'disabled' : ''}><span><strong>${escape(marker.character)}</strong><small>Walk there →</small></span></button>`).join('')}</div>${missionControls.map((item) => `<button class="secondary" data-story-action="${item.action.type}" data-story-mission-handle="${escape(item.id)}" ${this.locked() ? 'disabled' : ''}>${escape(this.snapshot.scene?.choices.find((choice) => choice.actionId === item.id)?.label ?? item.description)}</button>`).join('')}<p id="story-world-status" role="status">${this.commands.hasPending ? 'A previous save needs confirmation. Retry the same action.' : this.commands.needsRefresh ? 'Refresh before choosing another action.' : this.snapshot.commandBudgetExhausted ? 'This development story reached its save limit.' : 'Your adventure and activities are saved separately.'}</p><button class="secondary" data-story-refresh>Refresh progress</button>${this.commands.hasPending ? '<button class="primary" data-story-retry>Retry the same action</button>' : ''}<p><a href="/">Return to the main adventure</a> · <a href="/story.html">Story inspector</a></p>`,
    );
    document.querySelector<HTMLButtonElement>('[data-story-refresh]')!.onclick = () =>
      void this.run(async () => {
        await this.commands.refresh();
      });
    document
      .querySelectorAll<HTMLButtonElement>('[data-story-mission-handle]')
      .forEach((button) => {
        button.onclick = () =>
          void this.choose({ type: 'choose', actionId: button.dataset.storyMissionHandle! });
      });
    const retry = document.querySelector<HTMLButtonElement>('[data-story-retry]');
    if (retry)
      retry.onclick = () =>
        void this.run(async () => {
          await this.commands.retry();
          await this.commands.refresh();
        });
  }
  encounter(marker: WorldEncounter) {
    if (this.disposed || this.crossing || !this.markers.some((item) => item.id === marker.id))
      return;
    if (this.locked()) {
      this.missions();
      return;
    }
    const scene = this.snapshot.scene!;
    const characterId = marker.id.startsWith('npc:') ? marker.id.slice(4) : undefined;
    const entries = scene.presentation.filter(
      (entry) => entry.type === 'narration' || (characterId && entry.speakerId === characterId),
    );
    const actions = this.snapshot
      .sceneInput!.availableActions.filter((item) => actionTarget(item.action) === marker.id)
      .sort(
        (a, b) =>
          Number(a.action.type === 'cancel_mission') - Number(b.action.type === 'cancel_mission'),
      );
    const ack = characterId === 'woodcutter' ? this.snapshot.acknowledgeableEntryIds[0] : undefined;
    this.hooks.open(
      `<span class="pill">${escape(storyPlaces[this.snapshot.locationId].name)}</span><h2 id="dialog-title">${escape(marker.character)}</h2><div data-story-dialogue>${entries.map((entry) => `<p>${escape(entry.text)}</p>`).join('')}</div><div class="grownup-actions">${ack ? '<button class="primary" data-story-continue>Continue</button>' : actions.map((item) => `<button class="${item.action.type === 'cancel_mission' ? 'secondary' : 'primary'}" data-story-action="${item.action.type}" data-story-handle="${escape(item.id)}">${escape(scene.choices.find((choice) => choice.actionId === item.id)?.label ?? item.description)}</button>`).join('')}</div><p id="story-world-status" role="status"></p>`,
    );
    const next = document.querySelector<HTMLButtonElement>('[data-story-continue]');
    if (next && ack)
      next.onclick = () =>
        void this.choose({ type: 'acknowledge', sceneId: scene.sceneId, entryId: ack }, marker);
    document.querySelectorAll<HTMLButtonElement>('[data-story-handle]').forEach((button) => {
      button.onclick = () =>
        void this.choose({ type: 'choose', actionId: button.dataset.storyHandle! }, marker);
    });
  }
  private outcome(marker: WorldEncounter) {
    if (this.snapshot.commandBudgetExhausted) return this.missions();
    if (this.markers.some((item) => item.id === marker.id)) return this.encounter(marker);
    const present = new Set(
      this.snapshot.sceneInput!.here.characters.map((character) => character.id),
    );
    const entries = this.snapshot.scene!.presentation.filter(
      (entry) => entry.type === 'narration' || present.has(entry.speakerId),
    );
    const texts = [...new Set(entries.map((entry) => entry.text))];
    this.hooks.open(
      `<span class="pill">${escape(storyPlaces[this.snapshot.locationId].name)}</span><h2 id="dialog-title">${escape(marker.character)}</h2><div data-story-dialogue data-story-outcome>${texts.map((text) => `<p>${escape(text)}</p>`).join('')}</div><button class="primary" data-story-outcome-close>Continue exploring</button>`,
    );
    document.querySelector<HTMLButtonElement>('[data-story-outcome-close]')!.onclick = () =>
      this.hooks.close();
  }
  private async choose(payload: StoryPayload, marker?: WorldEncounter) {
    const action =
      payload.type === 'choose'
        ? this.snapshot.sceneInput?.availableActions.find((item) => item.id === payload.actionId)
            ?.action
        : undefined;
    const previousLocation = this.snapshot.locationId;
    const wasCompleted = this.snapshot.sceneInput?.mission?.status === 'completed';
    await this.run(
      async () => {
        await this.commands.choose(payload);
        await this.commands.refresh();
        const next = this.commands.snapshot!;
        const route =
          action?.type === 'accompany'
            ? crossingRoute(previousLocation, next.locationId)
            : undefined;
        const completion =
          !wasCompleted && next.sceneInput?.mission?.status === 'completed'
            ? `Completed: ${next.sceneInput.mission.goal}`
            : undefined;
        if (route && !next.commandBudgetExhausted) {
          this.applied = next;
          this.crossing = true;
          this.escort = {
            id: action!.type === 'accompany' ? action.characterId : 'pip',
            sprite:
              action!.type === 'accompany' && action.characterId === 'bramble' ? 'bramble' : 'pip',
            name:
              action!.type === 'accompany' && action.characterId === 'bramble' ? 'Bramble' : 'Pip',
            offsetX: route.followerOffset,
          };
          this.location = next.locationId;
          this.state.region = storyPlaces[next.locationId].region;
          this.state.done = storySceneryFlags(next);
          this.state.position = { ...route.start };
          this.hooks.close();
          this.game.events.emit('authoritative-position');
          this.hud();
          this.game.events.emit('scripted-walk', route.end, () => {
            const name = this.escort?.name ?? 'Pip';
            this.crossing = false;
            this.escort = undefined;
            this.state.position = { ...route.end };
            this.apply();
            this.hooks.toast(completion ?? `You and ${name} arrived together.`);
          });
        } else if (completion) this.hooks.toast(completion);
      },
      payload.type === 'acknowledge' ||
        (action && ['talk', 'inspect', 'choose'].includes(action.type))
        ? marker
        : undefined,
    );
  }
  private async run(work: () => Promise<void>, reopen?: WorldEncounter) {
    if (this.disposed || this.commands.busy || this.crossing) return;
    document.querySelectorAll<HTMLButtonElement>('#dialog button:not(.close)').forEach((button) => {
      button.disabled = true;
    });
    try {
      await work();
      if (this.disposed || this.crossing) return;
      this.hooks.close();
      this.apply();
      if (reopen) this.outcome(reopen);
    } catch {
      if (this.disposed) return;
      this.hud();
      this.missions();
      const status = document.querySelector('#story-world-status');
      if (status)
        status.textContent = this.commands.hasPending
          ? 'We could not confirm the save. Retry the same action.'
          : 'Progress could not be refreshed. Refresh before choosing again.';
    }
  }
}
export async function connectStoryWorld(hooks: Hooks) {
  const lifecycle = new AbortController();
  const leave = () => lifecycle.abort();
  window.addEventListener('pagehide', leave, { once: true });
  const response = await fetch('/__demo/bootstrap', {
    signal: AbortSignal.any([AbortSignal.timeout(10000), lifecycle.signal]),
    redirect: 'error',
  });
  if (!response.ok) throw new Error('Local demo required');
  const value: unknown = await response.json();
  if (!isRecord(value) || typeof value.profileId !== 'string' || typeof value.csrf !== 'string')
    throw new Error('Invalid local session');
  if (lifecycle.signal.aborted) throw new Error('Story page is no longer active');
  window.removeEventListener('pagehide', leave);
  return new StoryWorld(new StoryClient(value.profileId, value.csrf), value.profileId, hooks);
}
