import { characterPortrait } from '../character-art.ts';
import { sceneSourceLabel } from './source-status.ts';
import type Phaser from 'phaser';
import type {
  MissionLearningAnswer,
  MissionLearningSnapshot,
  StorySnapshot,
} from '../../../contracts/generated/contracts.ts';
import type { WorldEncounter } from '../types.ts';
import { isRecord } from '../types.ts';
import { freshState } from '../progress.ts';
import { createWorld } from '../world.ts';
import { StoryClient, StorySession, type StoryPayload } from './client.ts';
import { MissionLearningClient, MissionLearningSession } from '../mission-learning/client.ts';
import { mountMissionActivity } from '../mission-learning/activity.ts';
import '../mission-learning/world.css';
import { createRecorder, type AnswerRecorder } from '../learning/voice.ts';
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
  walkingStoryExit,
  neighbours,
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
  private learning: MissionLearningSession;
  private learningAvailable = false;
  private learningActivity?: ReturnType<typeof mountMissionActivity>;
  private learningBusy = false;
  private storyAfterLearningNeedsRefresh = false;
  private learningEncounter?: WorldEncounter;
  constructor(
    private client: StoryClient,
    profile: string,
    private hooks: Hooks,
    learningClient: MissionLearningClient,
    private recorder?: AnswerRecorder,
  ) {
    this.commands = new StorySession(client, profile, sessionStorage);
    this.learning = new MissionLearningSession(learningClient, profile, sessionStorage);
    document.querySelector('dialog')?.addEventListener('close', () => {
      this.learningActivity?.destroy();
      this.learningActivity = undefined;
    });
    this.state.hero = 'rose';
    document.querySelector<HTMLElement>('.hud-stars')!.style.display = 'none';
    window.addEventListener('pagehide', () => {
      this.disposed = true;
      this.learningActivity?.destroy();
      this.recorder?.cancel();
      this.client.destroy();
      learningClient.destroy();
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
    // Reloads preserve uncertain commands. Reconcile the exact saved request in
    // the background instead of waiting for the child to find a retry button.
    void this.refreshLearning()
      .then(() => (this.learning.hasPending ? this.retryMissionLearning() : undefined))
      .catch(() => undefined);
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
        inputLocked: () => this.movementLocked(),
        observe: (point, followers) => {
          this.positionMetadata(point.x, point.y);
          if (!this.locked()) {
            const exit = walkingStoryExit(this.snapshot, point);
            if (exit) void this.choose({ type: 'choose', actionId: exit.id });
          }
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
    document.querySelector('#story-source-status')!.textContent = sceneSourceLabel(snapshot);
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
      : this.commands.needsRefresh || this.storyAfterLearningNeedsRefresh
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
  private async refreshLearning() {
    const snapshot = await this.learning.refresh();
    this.learningAvailable = true;
    return snapshot;
  }
  private async missionLearningCommand(
    payload: Parameters<MissionLearningSession['choose']>[0],
  ): Promise<MissionLearningSnapshot> {
    if (this.learningBusy) throw new Error('Practice is already saving.');
    this.learningBusy = true;
    try {
      if (this.learning.needsRefresh && !this.learning.hasPending) await this.refreshLearning();
      const result = await this.learning.choose(payload);
      this.learningAvailable = true;
      await this.syncLearningStory(result);
      return result;
    } finally {
      this.learningBusy = false;
    }
  }
  private async retryMissionLearning(): Promise<MissionLearningSnapshot> {
    if (this.learningBusy) throw new Error('Practice is already saving.');
    this.learningBusy = true;
    try {
      const result = await this.learning.retry();
      this.learningAvailable = true;
      try {
        const fresh = await this.refreshLearning();
        await this.syncLearningStory(fresh);
        return fresh;
      } catch {
        await this.syncLearningStory(result);
        return result;
      }
    } finally {
      this.learningBusy = false;
    }
  }
  private async syncLearningStory(snapshot: MissionLearningSnapshot) {
    if (snapshot.activeActivity || !snapshot.lastFeedback?.completed) return;
    this.storyAfterLearningNeedsRefresh = true;
    try {
      await this.commands.refresh();
      if (this.disposed) return;
      this.storyAfterLearningNeedsRefresh = false;
      this.apply();
    } catch {
      // The learning receipt is confirmed. Keep it and recover the story read.
      this.hud();
    }
  }
  private showLearningActivity(snapshot = this.learning.snapshot) {
    const activity = snapshot?.activeActivity;
    if (!activity) return this.missions();
    this.learningActivity?.destroy();
    this.hooks.open(
      `<span class="pill">OUR NEXT MISSION STEP</span><div id="mission-practice"></div>`,
    );
    this.learningActivity = mountMissionActivity(
      document.querySelector<HTMLElement>('#mission-practice')!,
      activity,
      snapshot,
      {
        recorder: this.recorder,
        onReplace: (updated) => this.showLearningActivity(updated),
        onAttempt: (answer: MissionLearningAnswer) =>
          this.missionLearningCommand({
            type: 'answer',
            encounterId: activity.id,
            variantId: activity.variantId,
            answer,
          }),
        onSupport: (kind) =>
          this.missionLearningCommand({
            type: 'support',
            encounterId: activity.id,
            kind,
          }),
        onRetry: () => this.retryMissionLearning(),
        onRefresh: () => this.refreshLearning(),
        hasPending: () => this.learning.hasPending,
        onAbandon: async () => {
          await this.missionLearningCommand({ type: 'abandon', encounterId: activity.id });
        },
        onFinishTogether: activity.storyStep
          ? () =>
              this.missionLearningCommand({
                type: 'finish_together',
                encounterId: activity.id,
                variantId: activity.variantId,
              })
          : undefined,
        onContinue: () => {
          if (!this.practiceDialogOpen()) return;
          this.learningActivity?.destroy();
          this.learningActivity = undefined;
          const completed = this.learning.snapshot?.lastFeedback?.completed;
          this.learningEncounter = undefined;
          this.storyAfterLearningNeedsRefresh = true;
          this.hooks.close();
          void this.run(async () => {
            await this.commands.refresh();
            await this.refreshLearning();
            if (completed) this.hooks.toast('Nice work! Your adventure is ready to continue.');
          });
        },
      },
    );
    this.learningActivity.focus();
  }
  private practiceDialogOpen() {
    return Boolean(document.querySelector('#dialog[open] #mission-practice'));
  }
  private missionPanelOpen() {
    return Boolean(document.querySelector('#dialog[open] [data-story-missions-panel]'));
  }
  private learningPanel() {
    const snapshot = this.learning.snapshot;
    if (!this.learningAvailable || !snapshot) {
      if (!this.learning.hasPending && !this.learning.needsRefresh) return '';
      return `<section class="story-learning-panel" aria-labelledby="story-learning-title"><span class="pill">PRACTICE SAVE</span><h3 id="story-learning-title">${this.learning.hasPending ? 'One practice action needs confirmation' : 'Refresh practice progress'}</h3><p>Your story can continue while practice reconnects.</p>${this.learning.hasPending ? '<button class="primary" data-learning-retry>Retry the same practice save</button>' : '<button class="secondary" data-learning-refresh>Refresh practice progress</button>'}</section>`;
    }
    const active = snapshot.activeActivity;
    const skills = snapshot.skills
      .filter((skill) => skill.supportedByMission)
      .map((skill) => {
        const status = skill.activityCompleted
          ? 'Activity complete ✓'
          : {
              locked: 'Coming later',
              available: 'Ready to try',
              practising: 'Practising',
              ready_for_next: 'Ready for another skill',
              unavailable: 'Not available in this story',
            }[skill.status];
        return `<li><strong>${escape(skill.objective)} · ${status}</strong><span>${skill.activityCompleted ? 'Finished. We’ll try something different next.' : `${skill.correctChecks} of ${skill.eligibleChecks} independent checks · ${escape(skill.nextStep)}`}</span></li>`;
      })
      .join('');
    const callToAction = this.learning.hasPending
      ? '<button class="primary" data-learning-retry>Retry the same practice save</button>'
      : this.learning.needsRefresh
        ? '<button class="secondary" data-learning-refresh>Refresh practice progress</button>'
        : active
          ? '<button class="primary" data-learning-resume>Resume the mission task</button>'
          : '';
    return `<section class="story-learning-panel" aria-labelledby="story-learning-title"><span class="pill">MISSION TASKS</span><h3 id="story-learning-title">${escape(active?.storyStep?.label ?? active?.presentation.title ?? 'Learning through our adventures')}</h3><p>${escape(active?.storyStep?.introduction ?? 'Help with a mission step to discover what comes next.')}</p>${callToAction}<details class="story-learning-progress"><summary>See learning progress</summary><ul>${skills}</ul><p>This guides what to try next. It is not a mastery score.</p></details></section>`;
  }
  private locked() {
    return (
      this.movementLocked() ||
      this.learningBusy ||
      this.learning.hasPending ||
      this.storyAfterLearningNeedsRefresh
    );
  }
  private movementLocked() {
    // Walking changes only local position. Pending learning saves still block
    // story commands and automatic exits, but must not freeze the character.
    return (
      this.commands.busy ||
      this.commands.hasPending ||
      this.commands.needsRefresh ||
      this.crossing ||
      this.snapshot.commandBudgetExhausted
    );
  }
  handle(button: HTMLButtonElement) {
    if (button.id === 'bag') {
      this.bag();
      return;
    }
    if (button.id === 'missions') {
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
  private bag() {
    const items =
      this.snapshot.sceneInput?.context.inventory.filter((item) => item.quantity > 0) ?? [];
    this.hooks.open(
      `<section class="story-bag" aria-labelledby="dialog-title"><span class="pill">YOUR ADVENTURE</span><h2 id="dialog-title">My bag</h2><p>The things you’re carrying for your adventures.</p>${items.length ? `<ul class="story-bag-items" aria-label="Things in your bag">${items.map((item) => `<li><strong>${escape(item.itemId.replace(/[_-]+/g, ' '))}</strong><span class="story-bag-quantity" aria-label="Quantity: ${item.quantity}">× ${item.quantity}</span></li>`).join('')}</ul>` : '<div class="story-bag-empty"><p><strong>Your bag is empty.</strong></p><p>Supplies you collect on your adventures will appear here.</p></div>'}${this.commands.hasPending || this.commands.needsRefresh || this.storyAfterLearningNeedsRefresh ? '<p role="status">Showing your last confirmed supplies. Open Missions to confirm your latest progress.</p>' : ''}</section>`,
    );
  }
  missions() {
    this.learningActivity?.destroy();
    this.learningActivity = undefined;
    if (this.crossing) return;
    if (this.snapshot.commandBudgetExhausted) {
      this.hooks.open(
        '<h2 id="dialog-title">This development story has reached its save limit</h2><p>Your saved progress is preserved. No further story actions are available in this run.</p>',
      );
      return;
    }
    const input = this.snapshot.sceneInput;
    const mission = input?.mission;
    const neighboursPanel = input?.neighbourMissions?.length
      ? `<section><h3>Neighbourhood missions · ${input.neighbourMissions.filter((q) => q.status === 'open').length + (mission && mission.status !== 'completed' ? 1 : 0)} / 4 open</h3><p>New missions archive the oldest neighbourhood mission when space is needed. Visit its friend to resume; progress is kept.</p><ul class="story-checklist">${input.neighbourMissions.map((q) => `<li><div><strong>${escape(q.title)}</strong> · ${escape(q.status)}${q.storySource ? ` · ${q.storySource === 'llm' ? 'AI-generated story' : 'Offline story fallback'}` : ''}<br>${escape(q.status === 'completed' ? q.payoff : q.inspected ? `Return to ${q.generatedStory?.name ?? neighbours[q.characterId]?.name ?? q.characterId}` : q.task)} · ${escape(storyPlaces[q.locationId].name)}</div></li>`).join('')}</ul></section>`
      : '';
    const pipPresent = this.markers.some((marker) => marker.id === 'npc:pip');
    const missionControls = (input?.availableActions ?? []).filter(
      (item) =>
        item.action.type === 'cancel_mission' ||
        (!pipPresent && item.action.type === 'accept_mission'),
    );
    this.hooks.open(
      `<header class="story-journal-heading"><span class="story-journal-icon" aria-hidden="true">✦</span><span class="pill" data-story-missions-panel>PIP’S STORY ADVENTURE</span><h2 id="dialog-title">${escape(mission?.goal ?? 'A way across, together')}</h2></header><p class="story-premise">${escape(mission?.narrative.premise ?? 'Find Pip by the bridge to begin.')}</p>${mission ? `<p data-story-mission-status="${mission.status}">${mission.status === 'completed' ? `Mission complete · ${escape(mission.narrative.payoff)}` : mission.status === 'abandoned' ? 'Mission paused · your progress is saved.' : escape(mission.current ?? '')}</p><ul class="story-checklist">${mission.completed.map((text) => `<li class="is-complete"><span aria-hidden="true">✓</span>${escape(text)}</li>`).join('')}${mission.remaining.map((text) => `<li><span aria-hidden="true">○</span>${escape(text)}</li>`).join('')}</ul>` : ''}${nextStoryLabel(this.snapshot) ? `<p data-story-next>${escape(nextStoryLabel(this.snapshot)!)}</p>` : ''}${input?.context.memory.recentMissions.length ? `<details data-story-history><summary>Our adventures so far</summary><ul>${input.context.memory.recentMissions.map((text) => `<li>${escape(text)}</li>`).join('')}</ul></details>` : ''}${neighboursPanel}<h3 class="story-section-title">Where shall we go?</h3><div class="modal-quests story-destinations">${this.markers.map((marker) => `<button class="quest" data-story-visit="${escape(marker.id)}" ${this.locked() ? 'disabled' : ''}><span class="story-destination-icon" aria-hidden="true">${marker.sprite ? characterPortrait(marker.sprite, 48) : marker.id.startsWith('route:') ? '↗' : '✦'}</span><span><strong>${escape(marker.character)}</strong><small>${marker.id.startsWith('npc:') ? 'Walk over & chat' : 'Follow the path'}</small></span><span class="quest-arrow" aria-hidden="true">→</span></button>`).join('')}</div>${missionControls.map((item) => `<button class="secondary" data-story-action="${item.action.type}" data-story-mission-handle="${escape(item.id)}" ${this.locked() ? 'disabled' : ''}>${escape(this.snapshot.scene?.choices.find((choice) => choice.actionId === item.id)?.label ?? item.description)}</button>`).join('')}<footer class="story-journal-footer"><p id="story-world-status" role="status">${this.commands.hasPending ? 'A previous save needs confirmation. Retry the same action.' : this.commands.needsRefresh ? 'Refresh before choosing another action.' : this.snapshot.commandBudgetExhausted ? 'This development story reached its save limit.' : 'Your adventure is saved as you go.'}</p><button class="secondary" data-story-refresh>Refresh progress</button>${this.commands.hasPending ? '<button class="primary" data-story-retry>Retry the same action</button>' : ''}</footer>`,
    );
    const learningPanel = this.learningPanel();
    if (learningPanel)
      document.querySelector('#dialog')?.insertAdjacentHTML('beforeend', learningPanel);
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
    const resume = document.querySelector<HTMLButtonElement>('[data-learning-resume]');
    if (resume) resume.onclick = () => this.showLearningActivity();
    const practiceRetry = document.querySelector<HTMLButtonElement>('[data-learning-retry]');
    if (practiceRetry)
      practiceRetry.onclick = () =>
        void this.retryMissionLearning()
          .then((snapshot) => {
            if (!this.missionPanelOpen()) return;
            if (snapshot.activeActivity) this.showLearningActivity(snapshot);
            else this.missions();
          })
          .catch(() => {
            if (this.missionPanelOpen()) this.missions();
          });
    const learningRefresh = document.querySelector<HTMLButtonElement>('[data-learning-refresh]');
    if (learningRefresh)
      learningRefresh.onclick = () =>
        void this.refreshLearning()
          .then(() => {
            if (this.missionPanelOpen()) this.missions();
          })
          .catch(() => {
            if (this.missionPanelOpen()) this.missions();
          });
    if (!this.learning.snapshot && !this.learningBusy)
      void this.refreshLearning()
        .then(() => {
          if (this.missionPanelOpen()) this.missions();
        })
        .catch(() => {});
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
    if (!ack && actions.every((item) => item.action.type === 'cancel_mission')) {
      this.hooks.close();
      this.game.events.emit(
        'speech',
        marker,
        entries.map((entry) => entry.text),
      );
      return;
    }
    this.hooks.open(
      `<header class="story-conversation-heading">${marker.sprite ? characterPortrait(marker.sprite, 88) : '<span class="story-object-icon" aria-hidden="true">✦</span>'}<div><span class="pill">${escape(storyPlaces[this.snapshot.locationId].name)}</span><h2 id="dialog-title">${escape(marker.character)}</h2></div></header><div data-story-dialogue>${entries.map((entry) => `<p>${escape(entry.text)}</p>`).join('')}</div><div class="grownup-actions">${ack ? '<button class="primary" data-story-continue>Continue</button>' : actions.map((item) => `<button class="${item.action.type === 'cancel_mission' ? 'secondary' : 'primary'}" data-story-action="${item.action.type}" data-story-handle="${escape(item.id)}">${escape(scene.choices.find((choice) => choice.actionId === item.id)?.label ?? item.description)}</button>`).join('')}</div><p id="story-world-status" role="status"></p>`,
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
    this.game.events.emit('speech', marker, texts);
  }
  private async beginMissionStep(actionId: string, marker?: WorldEncounter) {
    if (this.locked()) return;
    try {
      const learning = await this.refreshLearning();
      if (this.disposed) return;
      if (this.learning.hasPending || this.learning.needsRefresh) return this.missions();
      if (learning.activeActivity) return this.showLearningActivity(learning);
      // An unsupported frontier must never make a story inaccessible.
      if (!learning.frontierSkillId) {
        await this.run(async () => {
          await this.commands.choose({ type: 'choose', actionId });
          await this.commands.refresh();
        });
        return;
      }
      const next = await this.missionLearningCommand({
        type: 'begin_step',
        actionId,
        storyRevision: this.snapshot.revision,
      });
      if (!this.disposed) {
        this.learningEncounter = marker;
        this.showLearningActivity(next);
      }
    } catch {
      if (!this.disposed) this.missions();
      const status = document.querySelector('#story-world-status');
      if (status)
        status.textContent =
          'This mission task could not open. Refresh progress or retry the same task save.';
    }
  }
  private async choose(payload: StoryPayload, marker?: WorldEncounter) {
    const action =
      payload.type === 'choose'
        ? this.snapshot.sceneInput?.availableActions.find((item) => item.id === payload.actionId)
            ?.action
        : undefined;
    const activeMission = this.snapshot.sceneInput?.mission?.status === 'active';
    const activeNeighbour =
      action?.type === 'inspect' &&
      action.objectId.startsWith('neighbour_') &&
      this.snapshot.sceneInput?.neighbourMissions?.some(
        (mission) => mission.status === 'open' && mission.locationId === this.snapshot.locationId,
      );
    if (
      action &&
      (activeMission || activeNeighbour) &&
      ['inspect', 'collect', 'exchange', 'assemble'].includes(action.type)
    ) {
      await this.beginMissionStep(payload.type === 'choose' ? payload.actionId : '', marker);
      return;
    }
    if (action?.type === 'travel')
      document.querySelector('#save-status')!.textContent = 'Opening the next area…';
    const previousLocation = this.snapshot.locationId;
    const wasCompleted = this.snapshot.sceneInput?.mission?.status === 'completed';
    await this.run(
      async () => {
        await this.commands.choose(payload);
        await this.commands.refresh();
        void this.refreshLearning().catch(() => undefined);
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
      this.storyAfterLearningNeedsRefresh = false;
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
  return new StoryWorld(
    new StoryClient(value.profileId, value.csrf),
    value.profileId,
    hooks,
    new MissionLearningClient(value.profileId, value.csrf),
    value.voiceInput === true &&
      typeof MediaRecorder !== 'undefined' &&
      typeof navigator.mediaDevices?.getUserMedia === 'function'
      ? createRecorder(value.csrf, value.profileId)
      : undefined,
  );
}
