import type Phaser from 'phaser';
import { MissionLearningClient, MissionLearningSession } from '../mission-learning/client.ts';
import { activityIcon } from '../mission-learning/icons.ts';
import { mountMissionActivity } from '../mission-learning/activity.ts';
import { CampaignClient, CampaignSession, type CampaignPayload } from './client.ts';
import { createWorld } from '../world.ts';
import type { Progress, WorldEncounter, WorldDecoration } from '../types.ts';
import type {
  HouseholdProfile,
  CampaignCompanion,
} from '../../../contracts/generated/contracts.ts';
import { PairedClient } from '../paired/client.ts';
import { PairedPanel, profileLabel } from '../paired/panel.ts';
import fieldGuide from '../../../content/campaign/field-guide.v1.json';
import { characterSprite, campaignCharacterArt } from '../character-sprites.ts';
import './campaign.css';
import { projectBoard } from './projects.ts';
import { challengeStory, storySoFar } from './story.ts';
import {
  beaconFlags,
  companionAffinities,
  signatureCost,
  encounterPhase,
  savedPuzzles,
  journalHistory,
  adventureUrl,
  canVisitEarlier,
  nextObjective,
  missionBrief,
  nextTrail,
  openingDialogue,
  campaignPages,
  newlyEarnedWayfinders,
  changedCompanionAppearance,
} from './presentation.ts';

type Hooks = { open(html: string): void; close(): void; toast(message: string): void };
const escape = (s: string) =>
  String(s).replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );
const name = (s: string) =>
  s
    .replaceAll('_', ' ')
    .replaceAll('-', ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
export function portrait(id: string) {
  const campaignArt = campaignCharacterArt(id);
  if (campaignArt) {
    const [x, y, width, height] = campaignArt.frame;
    const posX = (x / (campaignArt.width - width)) * 100 || 0,
      posY = (y / (campaignArt.height - height)) * 100 || 0;
    return `<div class="campaign-portrait" role="img" aria-label="${escape(name(id))}" style="background-image:url('${campaignArt.url}');background-size:${(campaignArt.width / width) * 100}% ${(campaignArt.height / height) * 100}%;background-position:${posX}% ${posY}%"></div>`;
  }
  const sprite = characterSprite(id);
  if (!sprite) return '';
  const region = sprite.texture.replace('friends-', '');
  return `<div class="campaign-portrait" role="img" aria-label="${escape(name(id))}" style="background-image:url('/assets/characters/${region}.png');background-position:${sprite.index % 2 ? '100%' : '0'} ${sprite.index > 1 ? '100%' : '0'}"></div>`;
}
export class CampaignWorld {
  private game?: Phaser.Game;
  private panel = 'missions';
  private location = '';
  private status = '';
  private selected?: string;
  private guided = false;
  private projectPiece = 'short';
  private exploring = false;
  private savingAge = false;
  private refreshNeeded = false;
  private pairing?: PairedPanel;
  private disposed = false;
  private confirmedEffect?: { id: string; text: string };
  private formNotice?: {
    kind: 'earned' | 'changed';
    id: string;
    before: string;
    after: string;
    animate?: boolean;
  };
  private learningBaseline?: CampaignCompanion[];
  private pendingAppearance?: { id: string; before: CampaignCompanion[] };
  private playerLabel = 'This player';
  private readonly commands: CampaignSession;
  private readonly learning: MissionLearningSession;
  private learningActivity?: ReturnType<typeof mountMissionActivity>;
  private state: Progress = {
    version: 1,
    region: 'meadow',
    hero: 'clover',
    companionName: '',
    done: [],
    evidence: [],
    position: { x: 200, y: 530 },
  };
  constructor(
    private client: CampaignClient,
    profile: string,
    private hooks: Hooks,
    learningClient: MissionLearningClient,
    paired?: { profiles: HouseholdProfile[]; client: PairedClient },
  ) {
    window.addEventListener('pageshow', (event) => {
      if (event.persisted) location.reload();
    });
    this.commands = new CampaignSession(client, profile, localStorage);
    this.learning = new MissionLearningSession(learningClient, profile, localStorage);
    if (paired) {
      this.playerLabel = profileLabel(paired.profiles, profile);
      this.pairing = new PairedPanel(
        paired.client,
        profile,
        paired.profiles,
        () => {
          if (this.panel === 'paired') this.render();
        },
        () => this.canPair(),
        portrait,
      );
    }
    window.addEventListener(
      'pagehide',
      () => {
        this.disposed = true;
        client.destroy();
        learningClient.destroy();
        this.game?.destroy(true);
        this.learningActivity?.destroy();
      },
      { once: true },
    );
  }
  get snapshot() {
    return this.commands.snapshot!;
  }
  private get campaign() {
    return this.snapshot.campaign!;
  }
  async load() {
    await this.commands.refresh();
    return this.snapshot;
  }
  async adopt() {
    if (this.commands.hasPending) await this.commands.retry();
    else await this.commands.choose({ type: 'adopt' });
    await this.commands.refresh();
  }
  async start() {
    if (!this.commands.snapshot) await this.load();
    if (this.commands.hasPending) {
      try {
        await this.commands.retry();
        await this.commands.refresh();
      } catch {
        this.status = 'Your last move needs confirmation. Retry to keep it safe.';
      }
    }
    this.apply();
    this.game = createWorld(
      'game',
      () => this.state,
      (marker) => this.showAction(marker.id),
      () => this.missions(),
      (point) => {
        this.state.position = point;
      },
      undefined,
      {
        encounters: () => this.markers(),
        companions: () =>
          this.campaign.state.team.map((id) => ({ sprite: this.friendSprite(id), name: name(id) })),
        decorations: () => this.decorations(),
        observe: (point, followers) => {
          const element = document.querySelector<HTMLElement>('#game');
          if (element) {
            element.dataset.campaignPlayerX = String(point.x);
            element.dataset.campaignPlayerY = String(point.y);
            element.dataset.campaignFollowers = JSON.stringify(followers);
          }
        },
        allowExits: false,
        inputLocked: () =>
          this.commands.busy ||
          this.commands.hasPending ||
          this.commands.needsRefresh ||
          this.refreshNeeded,
        blocked: (x) =>
          this.campaign.state.location === 'bridge_bank' &&
          !this.campaign.state.flags.includes('world_bridge_restored') &&
          x > 600,
      },
    );
    this.hud();
    await this.learning.refresh();
    if (this.learning.hasPending) {
      try {
        await this.learning.retry();
        await this.learning.refresh();
      } catch {
        this.status = 'Your practice move needs a retry.';
      }
    }
    if (this.learning.snapshot?.activeActivity) this.showLearning();
    else this.missions();
    return this.game;
  }
  private friendSprite(id: string) {
    const friend = this.campaign.companions.find((c) => c.id === id);
    const form = id + '-wayfinder';
    return friend?.appearance === 'wayfinder' && characterSprite(form) ? form : id;
  }
  private markers(): WorldEncounter[] {
    const location = this.campaign.state.location;
    const sites: Record<
      string,
      { x: number; y: number; shape?: 'stones' | 'leaf' | 'hollow' | 'lantern'; prop?: string }
    > = {
      welcome: { x: 510, y: 390 },
      starter: { x: 430, y: 440 },
      bridge_clue: { x: 510, y: 325, shape: 'stones' },
      find_branch_1: { x: 290, y: 330, prop: 'plank' },
      find_branch_2: { x: 620, y: 510, prop: 'plank' },
      find_branch_3: { x: 780, y: 340, prop: 'plank' },
      bridge_mounts: { x: 540, y: 325, shape: 'stones' },
      bridge_planks: { x: 330, y: 370, prop: 'plank' },
      bloom_beds: { x: 570, y: 330, shape: 'leaf' },
      water_: { x: 490, y: 410, shape: 'stones' },
      bridge_gather: { x: 310, y: 500, prop: 'plank' },
      bridge_craft: { x: 425, y: 510, prop: 'plank' },
      bridge_build: { x: 550, y: 360, prop: 'plank' },
      bridge_trial: { x: 540, y: 410, shape: 'lantern' },
      bridge_restore: { x: 540, y: 365, shape: 'lantern' },
      bridge_alcove: { x: 500, y: 540, shape: 'hollow' },
      bloom_clue: { x: 400, y: 330, shape: 'leaf' },
      bloom_water: { x: 490, y: 410, shape: 'stones' },
      bloom_trial: { x: 650, y: 430, shape: 'lantern' },
      bloom_restore: { x: 650, y: 430, shape: 'leaf' },
    };
    const speakerAction = this.campaign.legal_actions.find((action) =>
      action.dialogue.some((d) => d.speaker === 'Rowan'),
    );
    const markers: WorldEncounter[] = [
      {
        id: 'object:beacon',
        region: this.state.region,
        x: location === 'flower_garden' ? 650 : 550,
        y: location === 'flower_garden' ? 300 : 315,
        short: 'Moonlight beacon',
        character: 'Beacon',
        sprite: this.campaign.state.flags.includes(beaconFlags[location] ?? 'unlit')
          ? 'beacon-restored'
          : 'beacon-dim',
      },
      ...(this.snapshot.locations.find((l) => l.id === location)?.camp
        ? [
            {
              id: 'object:camp',
              region: this.state.region,
              x: 560,
              y: 370,
              short: 'Trail camp',
              character: 'Camp',
              sprite: 'campaign-camp',
            },
          ]
        : []),
      ...(location === 'flower_garden'
        ? [
            {
              id: 'object:garden',
              region: this.state.region,
              x: 650,
              y: 490,
              short: 'Garden channels',
              character: 'Irrigation garden',
              sprite: 'garden-irrigation',
            },
          ]
        : []),
      {
        id: speakerAction?.id ?? 'npc:rowan',
        passive: !speakerAction,
        region: this.state.region,
        x: location === 'flower_garden' ? 280 : 190,
        y: location === 'flower_garden' ? 420 : 350,
        short: 'Rowan and Moss',
        character: 'Rowan',
        sprite: 'rowan',
      },
      {
        id: 'npc:moss',
        passive: true,
        region: this.state.region,
        x: location === 'flower_garden' ? 245 : 160,
        y: location === 'flower_garden' ? 440 : 420,
        short: 'Moss',
        character: 'Moss',
        sprite: 'moss',
      },
      ...(location === 'flower_garden'
        ? [
            {
              id: 'object:collector',
              region: this.state.region,
              x: 780,
              y: 410,
              short: 'Moonlight collector',
              character: 'Collector cart',
              sprite: 'collector-cart',
            },
          ]
        : []),
      {
        id: 'npc:hazel',
        passive: true,
        region: this.state.region,
        x: location === 'flower_garden' ? 400 : 510,
        y: location === 'flower_garden' ? 420 : 390,
        short: 'Hazel',
        character: 'Hazel',
        sprite: 'hazel',
      },
    ];
    for (const [index, id] of this.campaign.travel.entries())
      markers.push({
        id: `travel:${id}`,
        region: this.state.region,
        x: index ? 900 : 160,
        y: 490 + Math.max(0, index - 1) * 60,
        short: `Path to ${this.snapshot.locations.find((l) => l.id === id)?.name ?? name(id)}`,
        character: 'Trail sign',
        prop: 'arch',
      });
    const objective = nextObjective(
      this.campaign.legal_actions,
      this.campaign.state.grants,
      this.campaign.state.flags,
      this.campaign.state.starter,
    );
    const worldActions = this.campaign.legal_actions.filter(
      (action) =>
        action.id !== 'bridge_gather' &&
        (action === objective ||
          action.id.startsWith('find_branch_') ||
          action.kind === 'observe' ||
          action.kind === 'recruit' ||
          action.puzzle),
    );
    for (const [index, action] of worldActions.entries()) {
      const key = Object.keys(sites)
        .sort((a, b) => b.length - a.length)
        .find((key) => action.id.startsWith(key));
      if (action.kind === 'recruit' && action.recruit === this.campaign.state.starter) continue;
      const speaker = action.dialogue[0]?.speaker;
      if (!key && speaker === 'Hazel' && ['observe', 'learn', 'practice'].includes(action.kind)) {
        const hazel = markers.find((m) => m.id === 'npc:hazel');
        if (hazel && hazel.passive) {
          hazel.id = action.id;
          hazel.passive = false;
          hazel.short = action.label;
          continue;
        }
      }
      if (speaker === 'Rowan' && speakerAction?.id === action.id) continue;
      const site = key
        ? sites[key]!
        : {
            x: 260 + (index % 3) * 115,
            y: 290 + Math.floor(index / 3) * 90,
            shape: 'stones' as const,
          };
      markers.push({
        id: action.id,
        region: this.state.region,
        ...site,
        short: action.label,
        character: action.label,
        ...(!site.prop && !site.shape && action.kind !== 'recruit'
          ? { shape: 'stones' as const }
          : {}),
        ...(action.kind === 'recruit' ? { sprite: action.recruit ?? 'pip' } : {}),
        ...(action.id === 'bridge_planks' ? { sprite: 'woodcutter' } : {}),
      });
    }
    const laterProps: Record<string, string[]> = {
      old_watchtower: ['chime-tower'],
      shell_beach: this.campaign.state.flags.includes('world_refuge_restored')
        ? ['tide-refuge']
        : ['collector-cart'],
      old_mill: ['orchard-mill', 'mill-automaton'],
      courier_pond: ['garden-irrigation', 'reed'],
      bridge_island: ['ferry-raft'],
      orchard_hub: ['ferry-raft'],
      observatory: ['vell', 'observatory-machine'],
      harbour_lookout: ['lighthouse-lens'],
      star_island: ['sanctuary'],
    };
    for (const [index, sprite] of (laterProps[location] ?? []).entries())
      markers.push({
        id: `object:${sprite}`,
        region: this.state.region,
        x: 500 + index * 150,
        y: 370,
        short: name(sprite),
        character: name(sprite),
        sprite,
      });
    for (const [index, friend] of fieldGuide.friends
      .filter((friend) => friend.location === location)
      .entries()) {
      const ready = this.campaign.legal_actions.find(
        (action) => action.id === `habitat_cache_${friend.id}`,
      );
      const opened = this.campaign.state.flags.includes(`cache:${friend.id}`);
      if (ready || opened)
        markers.push({
          id: ready?.id ?? `object:cache:${friend.id}`,
          region: this.state.region,
          x: 650 + index * 95,
          y: 520,
          short: `${friend.name}’s habitat cache`,
          character: `${friend.name}’s cache`,
          sprite: opened ? 'habitat-cache-open' : 'habitat-cache-closed',
        });
    }
    for (const marker of markers) if (marker.id.startsWith('object:')) marker.passive = true;
    return markers.filter(
      (marker) =>
        !(marker.id === 'object:beacon' && !beaconFlags[location]) &&
        !(marker.id === 'npc:hazel' && location !== 'bridge_bank') &&
        !(['npc:rowan', 'npc:moss'].includes(marker.id) && !speakerAction),
    );
  }
  private decorations(): WorldDecoration[] {
    const { flags, location } = this.campaign.state;
    if (
      location === 'flower_garden' &&
      flags.includes('world_garden_restored') &&
      flags.includes('garden_ordered')
    )
      return [
        { id: 'ordered-bed-one', kind: 'leaf', x: 450, y: 330 },
        { id: 'ordered-bed-two', kind: 'leaf', x: 550, y: 330 },
        { id: 'ordered-bed-three', kind: 'leaf', x: 650, y: 330 },
        { id: 'ordered-path', kind: 'trail', x: 550, y: 420 },
      ];
    if (location === 'flower_garden')
      return flags.includes('world_garden_restored')
        ? [
            { id: 'restored-garden', kind: 'leaf', x: 520, y: 330 },
            { id: 'garden-beacon', kind: 'lantern', x: 650, y: 410 },
            { id: 'garden-flowers', kind: 'leaf', x: 720, y: 480 },
          ]
        : [
            { id: 'dry-garden', kind: 'hollow', x: 520, y: 330 },
            { id: 'dry-irrigation', kind: 'stones', x: 650, y: 410 },
          ];
    if (flags.includes('world_constellation_visible'))
      return [
        { id: 'constellation-stars', kind: 'stars', x: 520, y: 260 },
        { id: 'celebration-bunting', kind: 'bunting', x: 500, y: 370 },
      ];
    if (location !== 'bridge_bank' && location !== 'picnic_clearing')
      return beaconFlags[location] && flags.includes(beaconFlags[location]!)
        ? [
            { id: 'restored-local-beacon', kind: 'lantern', x: 550, y: 315 },
            { id: 'living-habitat', kind: 'leaf', x: 650, y: 420 },
          ]
        : [];
    if (location === 'picnic_clearing')
      return [
        { id: 'camp', kind: 'picnic', x: 620, y: 470 },
        { id: 'old-stump', kind: 'hollow', x: 290, y: 300 },
        { id: 'fallen-leaves', kind: 'leaf', x: 780, y: 310 },
      ];
    return flags.includes('world_bridge_restored')
      ? [{ id: 'bridge-beacon', kind: 'lantern', x: 540, y: 325 }]
      : flags.includes('world_bridge_planked')
        ? [{ id: 'bridge-materials', kind: 'supplies', x: 530, y: 400 }]
        : [];
  }
  private apply() {
    const state = this.campaign.state;
    const location = this.snapshot.locations.find((p) => p.id === state.location);
    if (this.location !== state.location) {
      this.state.position = { x: 200, y: 530 };
      this.location = state.location;
    }
    const regions: Record<string, string> = {
      bridge_bank: 'meadow',
      picnic_clearing: 'explore-0-0',
      flower_garden: 'explore-0-1',
      echo_hollow: 'explore-0-2',
      old_watchtower: 'explore-0-3',
      orchard_hub: 'orchard',
      old_mill: 'orchard',
      courier_pond: 'explore-1-0',
      garland_ridge: 'explore-1-1',
      bridge_island: 'explore-1-2',
      cove_hub: 'cove',
      shell_beach: 'explore-2-0',
      harbour_lookout: 'cove',
      observatory: 'cove',
      hidden_tide_garden: 'explore-2-1',
      star_island: 'explore-2-2',
    };
    this.state.region = regions[state.location] ?? 'meadow';
    this.state.done = state.flags.includes('world_bridge_restored') ? ['meadow-bridge'] : [];
    const heading = document.querySelector('h1');
    if (heading)
      heading.textContent =
        (location?.name ?? name(state.location)) +
        (state.location === 'flower_garden'
          ? state.flags.includes('garden_ordered')
            ? ' · Ordered beds'
            : state.flags.includes('garden_wild')
              ? ' · Wildflower paths'
              : ''
          : '');
    document
      .querySelector('#game')
      ?.setAttribute(
        'aria-label',
        `${location?.name ?? name(state.location)}. Arrow keys to walk, Space to interact. Open Missions for all actions.`,
      );
    const canvas = document.querySelector<HTMLElement>('#game');
    if (canvas) {
      canvas.dataset.campaignLocation = state.location;
      canvas.dataset.campaignRevision = String(this.snapshot.revision);
      canvas.dataset.campaignFlags = JSON.stringify(state.flags);
    }
    this.game?.events.emit('authoritative-position');
    this.hud();
  }
  private hud() {
    const flags = this.campaign.state.flags;
    const seals = flags.filter((f) => f.endsWith('_seal') || f.startsWith('seal_'));
    const set = (id: string, text: string) => {
      const node = document.querySelector(id);
      if (node) node.textContent = text;
    };
    document.body.classList.add('campaign-active');
    const label = document.querySelector('.mission-open');
    if (label) label.textContent = 'Missions →';
    set('#star-count', `${seals.length} seals`);
    set('#story-source-status', 'Authored · Missing Moonlight');
    const players = document.querySelector('#players');
    if (players) players.textContent = this.playerLabel + ' · Play together';
    set(
      '#save-status',
      this.commands.hasPending
        ? 'Move awaiting confirmation'
        : this.commands.busy
          ? 'Saving…'
          : 'Adventure saved',
    );
    set(
      '#mission-status',
      this.campaign.state.encounter
        ? challengeStory(this.campaign.state.encounter.id).title
        : missionBrief(
            flags,
            nextObjective(
              this.campaign.legal_actions,
              this.campaign.state.grants,
              this.campaign.state.flags,
              this.campaign.state.starter,
            )?.mission,
          ).title,
    );
    set(
      '#quest-direction',
      nextObjective(
        this.campaign.legal_actions,
        this.campaign.state.grants,
        this.campaign.state.flags,
        this.campaign.state.starter,
      )?.label ?? 'Open your mission for the next path',
    );
  }
  private canPair() {
    return (
      !this.commands.busy &&
      !this.commands.hasPending &&
      !this.commands.needsRefresh &&
      !this.refreshNeeded &&
      !this.learning.hasPending &&
      !this.learning.snapshot?.activeActivity &&
      !this.campaign.state.encounter
    );
  }
  handle(button: HTMLButtonElement) {
    if (button.dataset.projectPiece) {
      this.projectPiece = button.dataset.projectPiece;
      this.render();
      return;
    }
    if (button.id === 'campaign-explore') {
      this.exploring = true;
      this.hooks.close();
      document.querySelector<HTMLCanvasElement>('#game canvas')?.focus();
      return;
    }

    if (button.id === 'campaign-save-age' || button.id === 'campaign-retry-age') {
      void this.saveAge();
      return;
    }
    if (this.pairing?.handle(button)) return;
    if (button.id === 'players') {
      this.panel = 'paired';
      this.render();
      return;
    }
    if (button.dataset.campaignTab) {
      window.speechSynthesis?.cancel();
      this.panel = button.dataset.campaignTab;
      this.render();
      return;
    }
    if (button.dataset.campaignPage) {
      const [id, page] = JSON.parse(button.dataset.campaignPage) as [string, number];
      this.showAction(id, page);
      return;
    }
    if (button.id === 'campaign-visit-earlier') {
      if (
        canVisitEarlier(
          this.commands.hasPending || Boolean(this.pairing?.unresolved),
          this.learning.hasPending,
          Boolean(this.learning.snapshot?.activeActivity),
        )
      )
        location.assign(adventureUrl(location.href, true));
      else {
        this.status =
          'Finish or retry your current activity before visiting the earlier adventure.';
        this.render();
      }
      return;
    }
    if (button.id === 'campaign-read') {
      const unavailable = () => {
        const status = document.querySelector('#campaign-read-status');
        if (status)
          status.textContent = 'Read aloud is unavailable. The whole message is written above.';
      };
      try {
        window.speechSynthesis?.cancel();
        const text = button.dataset.campaignRead;
        if (text && typeof SpeechSynthesisUtterance !== 'undefined') {
          const utterance = new SpeechSynthesisUtterance(text);
          utterance.rate = 0.85;
          utterance.onerror = unavailable;
          window.speechSynthesis.speak(utterance);
        } else unavailable();
      } catch {
        unavailable();
      }
      return;
    }
    if (button.dataset.campaignCommand) {
      const marker = this.markers().find((marker) => marker.id === button.dataset.campaignCommand);
      if (marker && this.game && this.campaign.state.starter) {
        this.exploring = true;
        this.hooks.close();
        this.game.events.emit('visit', marker);
      } else this.showAction(button.dataset.campaignCommand);
      return;
    }
    if (button.dataset.campaignPayload) {
      const payload = JSON.parse(button.dataset.campaignPayload) as CampaignPayload;
      const path =
        payload.type === 'travel'
          ? this.markers().find((marker) => marker.id === `travel:${payload.location_id}`)
          : undefined;
      if (path && this.game) {
        this.exploring = true;
        this.hooks.close();
        this.game.events.emit('visit', path);
      } else void this.run(payload);
      return;
    }
    if (button.dataset.campaignSelect) {
      this.selected = button.dataset.campaignSelect;
      this.render();
      return;
    }
    if (button.id === 'campaign-guided') {
      this.guided = !this.guided;
      this.render();
      return;
    }
    if (button.id === 'campaign-dismiss-form') {
      this.formNotice = undefined;
      this.render();
      return;
    }
    if (button.id === 'campaign-refresh') {
      void this.refreshAdventure();
      return;
    }
    if (button.id === 'campaign-retry') {
      void this.retry();
      return;
    }
    if (button.id === 'missions' && this.learning.snapshot?.activeActivity) {
      this.showLearning();
      return;
    }
    if (button.id === 'missions' || button.id === 'bag' || button.id === 'character') {
      this.panel = button.id === 'bag' ? 'bag' : button.id === 'character' ? 'team' : 'missions';
      this.render();
    }
  }
  missions() {
    this.panel = 'missions';
    this.render();
  }
  private button(label: string, payload: CampaignPayload, disabled = false, primary = false) {
    return `<button class="${primary ? 'campaign-primary' : 'campaign-secondary'}" ${disabled ? 'disabled' : ''} data-campaign-move="${payload.type === 'encounter_turn' ? payload.movement : ''}" data-campaign-payload="${escape(JSON.stringify(payload))}">${escape(label)}</button>`;
  }
  private showAction(id: string, page = 0) {
    if (!this.learning.snapshot?.learnerAge) return this.render();
    window.speechSynthesis?.cancel();
    const action = this.campaign.legal_actions.find((a) => a.id === id);
    if (!action) {
      if (id.startsWith('npc:') || id.startsWith('object:')) return this.missions();
      if (id.startsWith('travel:')) void this.run({ type: 'travel', location_id: id.slice(7) });
      return;
    }
    if (id.startsWith('find_branch_')) {
      this.exploring = true;
      void this.run({ type: 'action', action_id: id });
      return;
    }
    const lines =
      openingDialogue(id) ??
      (action.dialogue.length ? action.dialogue : [{ speaker: 'Hazel', text: action.label }]);
    const index = Math.max(0, Math.min(page, lines.length - 1));
    const line = lines[index]!;
    const speakerId = line.speaker.toLowerCase().split(' ')[0]!;
    const final = index === lines.length - 1;
    const brief = missionBrief(this.campaign.state.flags, action.mission);
    const readAloud =
      typeof window.speechSynthesis !== 'undefined'
        ? `<button id="campaign-read" class="campaign-speaker" aria-label="Read this aloud" title="Read this aloud" data-campaign-read="${escape(line.text)}"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><path d="M11 5 6 9H3v6h3l5 4V5Z"/><path d="M15 8a6 6 0 0 1 0 8M18 5a10 10 0 0 1 0 14"/></svg></button>`
        : '';
    this.hooks.open(
      `<section class="campaign-panel"><span class="campaign-eyebrow">${escape(name(action.mission || 'Moonlight trail'))}</span><h2 id="dialog-title" tabindex="-1">${escape(action.label)}</h2><p class="campaign-purpose"><strong>Our goal:</strong> ${escape(brief.goal)}</p><div class="campaign-dialogue">${portrait(speakerId)}<strong>${escape(line.speaker)}</strong><div class="campaign-dialogue-text"><p>${escape(line.text)}</p>${readAloud}</div></div><p class="campaign-note">${index + 1} of ${lines.length}</p>${readAloud ? '' : '<p class="campaign-note">Read aloud is unavailable in this browser. All dialogue is shown as text.</p>'}<p id="campaign-read-status" role="status"></p><div class="campaign-actions">${index > 0 ? `<button class="campaign-secondary" data-campaign-page="${escape(JSON.stringify([id, index - 1]))}">Previous</button>` : ''}${!final ? `<button class="campaign-primary" data-campaign-page="${escape(JSON.stringify([id, index + 1]))}">Next</button>` : this.button(action.learning ? 'Practise together' : action.encounter ? 'Let’s begin' : action.label, { type: 'action', action_id: id }, false, true)}${final && action.recruit ? this.button('Maybe another time', { type: 'action', action_id: `decline:${action.id}` }) : ''}<button class="campaign-secondary" data-campaign-tab="missions">Back to adventure</button></div></section>`,
    );
    document.querySelector<HTMLElement>('#dialog-title')?.focus();
  }

  private confirmFormPresentation() {
    if (this.learningBaseline) {
      const earned = newlyEarnedWayfinders(this.learningBaseline, this.campaign.companions)[0];
      this.learningBaseline = undefined;
      if (earned && characterSprite(earned.id + '-wayfinder')) {
        this.formNotice = {
          kind: 'earned',
          id: earned.id,
          before: earned.id,
          after: earned.id + '-wayfinder',
        };
        this.game?.events.emit('campaign-pose', this.friendSprite(earned.id), 'cheer');
      }
    }
    if (this.pendingAppearance) {
      const change = changedCompanionAppearance(
        this.pendingAppearance.before,
        this.campaign.companions,
        this.pendingAppearance.id,
      );
      this.pendingAppearance = undefined;
      if (change) {
        this.formNotice = { kind: 'changed', ...change, animate: true };
        this.game?.events.emit('campaign-pose', change.after, 'cheer');
      }
    }
  }
  private formPresentation() {
    const notice = this.formNotice;
    if (!notice) return '';
    const animate = notice.animate === true;
    notice.animate = false;
    const friend =
      fieldGuide.friends.find((friend) => friend.id === notice.id)?.name ?? name(notice.id);
    const current = notice.after.endsWith('-wayfinder') ? 'Wayfinder' : 'original';
    return `<section class="campaign-form-notice" role="status" aria-live="polite" data-form-notice="${notice.kind}" data-form-friend="${escape(notice.id)}" aria-labelledby="campaign-form-title"><h3 id="campaign-form-title">${escape(notice.kind === 'earned' ? friend + ' Wayfinder earned!' : friend + '’s ' + current + ' appearance')}</h3>${notice.kind === 'earned' ? `<p>${escape(friend)} reached level five. Their Wayfinder form is ready to choose. Your friend keeps their current appearance until you choose another.</p><div class="campaign-form-comparison"><figure>${portrait(notice.before)}<figcaption>Original</figcaption></figure><span aria-hidden="true">→</span><figure>${portrait(notice.after)}<figcaption>${escape(friend)} Wayfinder</figcaption></figure></div>` : `<p>Your confirmed appearance choice is saved. You can choose either appearance again at camp.</p><div class="campaign-form-transition ${animate ? 'is-transition' : ''}" aria-label="${escape(friend + ' now uses their ' + current + ' appearance')}"><div class="campaign-form-previous" aria-hidden="true">${portrait(notice.before)}</div><div class="campaign-form-current">${portrait(notice.after)}</div></div>`}<div class="campaign-actions"><button data-campaign-tab="team">Choose my friend’s appearance</button><button id="campaign-dismiss-form">Continue the adventure</button></div></section>`;
  }
  private render() {
    if (this.disposed) return;
    if (
      !this.learning.snapshot?.activeActivity &&
      (!this.learning.snapshot?.learnerAge || this.panel === 'settings')
    ) {
      this.showAgeSetup();
      return;
    }
    const tabs = campaignPages(
      Boolean(this.campaign.state.starter),
      Boolean(this.campaign.state.encounter),
    );
    if (!this.campaign.state.starter || this.campaign.state.encounter) this.panel = 'missions';
    const titles: Record<string, string> = {
      team: 'Your friends',
      bag: 'Your supplies',
      care: 'Rest and care',
      journal: 'Your story so far',
      guide: 'Discover new friends',
      paired: 'Play together on this device',
      more: 'Friends & more',
    };
    this.hooks.open(
      `<section class="campaign-panel"><span class="campaign-eyebrow">The Missing Moonlight</span><nav class="campaign-tabs" aria-label="Adventure pages">${tabs.map(([id, label]) => `<button id="campaign-${id}" data-campaign-tab="${id}" ${this.panel === id ? 'aria-current="page"' : ''}>${label}</button>`).join('')}</nav><h2 id="dialog-title">${this.campaign.state.encounter ? escape(challengeStory(this.campaign.state.encounter.id).title) : (titles[this.panel] ?? 'Your mission')}</h2>${this.formPresentation()}${this.confirmedEffect && !this.campaign.state.encounter ? `<div class="campaign-confirmed-effect" role="status">${portrait(this.confirmedEffect.id)}<p>${escape(this.confirmedEffect.text)}</p></div>` : ''}${this.commands.hasPending ? '<div class="campaign-hazard"><strong>Your last move needs confirmation</strong><p>Retry your saved move to continue from the same place.</p><div class="campaign-actions"><button id="campaign-retry">Retry saved move</button></div></div>' : this.commands.needsRefresh || this.refreshNeeded ? '<div class="campaign-hazard"><strong>Your move is saved. Let’s refresh the adventure.</strong><p>Load the latest saved world before choosing another move.</p><div class="campaign-actions"><button id="campaign-refresh">Refresh adventure</button></div></div>' : this.body()}<p class="campaign-status" role="status">${escape(this.status)}</p></section>`,
    );
    document.querySelector<HTMLElement>('#dialog-title')?.setAttribute('tabindex', '-1');
    document.querySelector<HTMLElement>('#dialog-title')?.focus();
  }
  private showAgeSetup() {
    const age = this.learning.snapshot?.learnerAge;
    this.hooks.open(
      `<section class="campaign-panel"><span class="campaign-eyebrow">For a grown-up · ${escape(this.playerLabel)}</span><h2 id="dialog-title" tabindex="-1">Find a comfortable starting point</h2><p>How old is the player? We’ll start with skills around that age, try earlier skills if they are tricky, and move ahead when independent answers show they are ready.</p><p>This is a starting guess, not a test result. Help is always available and the adventure keeps moving.</p><label for="campaign-age">Player’s age</label><select id="campaign-age" ${this.savingAge || this.learning.hasPending ? 'disabled' : ''}><option value="">Choose an age</option>${Array.from(
        { length: 9 },
        (_, i) => i + 4,
      )
        .map(
          (value) =>
            `<option value="${value}" ${age === value ? 'selected' : ''}>${value === 4 ? '4 or younger' : value === 12 ? '12 or older' : value + ' years'}</option>`,
        )
        .join(
          '',
        )}</select><p class="campaign-note">We save only this age band for this player, not a date of birth. The current practice pack covers Kindergarten to Year Six. You can change this in Friends & more → Learning setup.</p><div class="campaign-actions"><button id="${this.learning.hasPending ? 'campaign-retry-age' : 'campaign-save-age'}" ${this.savingAge ? 'disabled' : ''}>${this.savingAge ? 'Saving…' : this.learning.hasPending ? 'Retry saved setup' : 'Save and continue'}</button>${age && !this.learning.hasPending ? '<button data-campaign-tab="missions">Back to my mission</button>' : ''}</div><p class="campaign-status" role="status">${escape(this.status)}</p></section>`,
    );
    document.querySelector<HTMLElement>('#dialog-title')?.focus();
  }
  private async saveAge() {
    if (this.savingAge) return;
    const age = Number(document.querySelector<HTMLSelectElement>('#campaign-age')?.value);
    if (!this.learning.hasPending && (!Number.isInteger(age) || age < 4 || age > 12)) {
      this.status = 'Please choose an age to set a starting point.';
      this.showAgeSetup();
      return;
    }
    this.savingAge = true;
    this.status = '';
    this.showAgeSetup();
    try {
      if (this.learning.hasPending) await this.learning.retry();
      else await this.learningCommand({ type: 'set_age', age });
      await this.learning.refresh();
      this.panel = 'missions';
    } catch {
      this.status =
        'The starting point could not be confirmed. Try again; your adventure stays saved.';
    } finally {
      this.savingAge = false;
      this.render();
    }
  }
  private body(): string {
    const c = this.campaign,
      s = c.state;
    if (this.panel === 'paired')
      return (
        this.pairing?.render() ??
        '<p>Shared-device play is unavailable. Your solo adventure stays saved.</p>'
      );
    if (s.encounter) return this.encounter();
    if (!s.starter) {
      const brief = missionBrief(s.flags, 'welcome');
      const meet = c.legal_actions.find((a) => a.id === 'welcome_meet');
      return `<div class="campaign-objective"><h3>${escape(brief.title)}</h3><p>${escape(brief.story)}</p><p><strong>First:</strong> Meet Hazel, then choose one friend to help you. Together you will investigate the broken bridge.</p></div>${meet ? `<div class="campaign-actions"><button data-campaign-command="welcome_meet">Meet Hazel and hear the plan</button></div>` : `<p>Choose one travelling friend. Every friend can complete the adventure, and you can meet the others later.</p><div class="campaign-grid">${this.snapshot.starters.map((friend) => `<article class="campaign-card">${portrait(friend.id)}<h3>${escape(friend.name)}</h3><p>${escape(({ pip: 'Find hidden clues with a little light.', tansy: 'Help plants grow and make new paths.', nib: 'Build and steady things together.' } as Record<string, string>)[friend.id] ?? friend.affinity)}</p><div class="campaign-actions">${this.button(`Choose ${friend.name}`, { type: 'choose_starter', companion_id: friend.id })}</div></article>`).join('')}</div>`}`;
    }
    if (this.panel === 'more') {
      const pages = [
        ['team', 'Friends', 'See your companions and choose who travels with you at camp.'],
        ['bag', 'Bag', 'See the supplies you have collected.'],
        ['care', 'Camp', 'Rest, make supplies and care for your friends at camp.'],
        ['journal', 'Journal', 'Remember what happened and which beacons you restored.'],
        ['guide', 'Field guide', 'Find clues about friends you can meet later.'],
        ...(this.pairing
          ? [['paired', 'Play together', 'Take turns with another saved player on this device.']]
          : []),
        ['settings', 'Learning setup', 'A grown-up can update the starting age estimate.'],
      ];
      return `<p>These are optional tools for your adventure. Your next mission is always under Your mission.</p><div class="campaign-grid">${pages.map(([id, label, description]) => `<article class="campaign-card"><h3>${label}</h3><p>${description}</p><div class="campaign-actions"><button id="campaign-${id}" data-campaign-tab="${id}">Open ${label}</button></div></article>`).join('')}</div>`;
    }
    if (this.panel === 'guide')
      return `<p>There are twelve friends to discover. Watch quietly, notice what each friend enjoys and choose a respectful way to approach.</p><p>${c.companions.length} of ${fieldGuide.friends.length} friends have joined you.</p><div class="campaign-grid">${fieldGuide.friends
        .map((friend) => {
          const owned = c.companions.find((c) => c.id === friend.id);
          const observed = s.flags.includes(`discovery:${friend.id}:observed`);
          return `<article class="campaign-card">${owned ? portrait(this.friendSprite(friend.id)) : '<div class="campaign-silhouette" aria-hidden="true">?</div>'}<h3>${escape(friend.name)}</h3><p>${owned ? `Your friend · Level ${owned.level}` : `Look in ${escape(friend.habitat)}`}</p><p>${escape(friend.clue)}</p><p><strong>Try:</strong> ${escape(friend.approachHint)}</p>${observed && !owned ? `<p>${escape(friend.observation)}</p>` : ''}${s.flags.includes(`discovery:${friend.id}:ready`) && !owned ? '<p>Your friend is ready for a helping invitation.</p>' : ''}</article>`;
        })
        .join('')}</div>`;
    if (this.panel === 'bag')
      return `<p>Supplies for helping, making and exploring.</p>${
        Object.entries(s.items)
          .map(
            ([id, n]) =>
              `<div class="campaign-stat"><span>${escape(name(id))}</span><strong>${n}</strong></div>`,
          )
          .join('') || '<p class="campaign-empty">Your bag is ready for discoveries.</p>'
      }`;
    if (
      (this.panel === 'team' || this.panel === 'care') &&
      !this.snapshot.locations.find((l) => l.id === s.location)?.camp
    )
      return '<p class="campaign-empty">Follow a path back to camp to rest, care for your friends or change your team.</p>';
    if (this.panel === 'team')
      return `<p>Your travelling friends share the adventure. Change your team between activities.</p><div class="campaign-grid">${c.companions.map((friend) => `<article class="campaign-card">${portrait(this.friendSprite(friend.id))}${friend.level >= 5 && Boolean(characterSprite(friend.id + '-wayfinder')) ? `<p>Wayfinder form earned</p>${portrait(friend.id + '-wayfinder')}` : ''}<h3>${escape(name(friend.id))}</h3><p>Level ${friend.level} · Energy ${s.energy[friend.id] ?? 0}</p><p>${s.team.includes(friend.id) ? 'Travelling with you' : 'Resting at camp'}</p><div class="campaign-actions">${friend.level >= 5 && characterSprite(friend.id + '-wayfinder') ? this.button(friend.appearance === 'wayfinder' ? 'Use original appearance' : 'Use Wayfinder appearance', { type: 'care', companion_id: friend.id, kind: friend.appearance === 'wayfinder' ? 'appearance_base' : 'appearance_wayfinder' }) : ''}${this.button('Travel together', { type: 'set_team', companion_ids: [friend.id] })}${friend.level >= 3 ? this.button('Technique: first signature free', { type: 'care', companion_id: friend.id, kind: 'specialize_free_signature' }) + this.button('Technique: stronger partner rally', { type: 'care', companion_id: friend.id, kind: 'specialize_rally' }) : ''}${s.flags.includes('bridge_seal') && s.team.length === 1 && !s.team.includes(friend.id) ? this.button('Join the team', { type: 'set_team', companion_ids: [...s.team, friend.id] }) : ''}</div></article>`).join('')}</div>`;
    if (this.panel === 'care')
      return `<p>A quiet moment together helps everyone get ready for the next trail.</p><div class="campaign-actions">${this.button('Rest at camp', { type: 'camp' })}${['snack', 'shelter_patch', 'trail_marker'].map((recipe) => this.button(`Make ${name(recipe)}`, { type: 'craft', recipe })).join('')}</div><div class="campaign-grid">${s.team
        .map(
          (id) =>
            `<article class="campaign-card">${portrait(this.friendSprite(id))}<h3>${escape(name(id))}</h3><p>${escape(fieldGuide.friends.find((friend) => friend.id === id)?.approachHint ?? 'Take a quiet moment together.')}</p><p>${
              s.flags.filter((flag) => flag.startsWith(`memory:care:${id}:`)).length
                ? 'Remembered together: ' +
                  s.flags
                    .filter((flag) => flag.startsWith(`memory:care:${id}:`))
                    .map((flag) => name(flag.split(':').at(-1)!))
                    .map(escape)
                    .join(' · ')
                : 'Your next care moment can become a happy memory.'
            }</p><div class="campaign-actions">${this.button('Play together', { type: 'care', companion_id: id, kind: 'play' })}${this.button('Sit quietly together', { type: 'care', companion_id: id, kind: 'quiet_rest' })}${this.button('Help with grooming', { type: 'care', companion_id: id, kind: 'groom' })}${this.button('Remember a favourite place', { type: 'care', companion_id: id, kind: 'favourite_place' })}</div></article>`,
        )
        .join('')}</div>`;
    if (this.panel === 'journal')
      return `<h3>Your next step</h3><p>${escape(nextObjective(c.legal_actions, s.grants, s.flags, s.starter)?.label ?? (s.flags.includes('finale_complete') ? (c.companions.length === fieldGuide.friends.length && c.companions.every((friend) => friend.level >= 5) ? 'Every friend has reached their Wayfinder milestone. Explore optional habitat caches or revisit keeper trials together.' : 'Explore your friends’ remaining invitations.') : 'Follow an open path to continue.'))}</p><h3>Keeper seals</h3><div class="campaign-seals">${
        ['bridge', 'bloom', 'echo', 'harvest', 'current', 'tide', 'beacon', 'accord']
          .filter((id) => s.flags.includes(id + '_seal'))
          .map(
            (id) =>
              `<article class="campaign-seal">${portrait(id + '-seal')}<strong>${escape(name(id))}</strong></article>`,
          )
          .join('') || '<p>Your first keeper light is waiting.</p>'
      }</div><h3>Our trail so far</h3><ul>${
        journalHistory(s.flags)
          .map((text) => `<li>${escape(text)}</li>`)
          .join('') || '<li>Your first chapter is waiting.</li>'
      }</ul><div class="campaign-actions"><button id="campaign-visit-earlier" ${canVisitEarlier(this.commands.hasPending || Boolean(this.pairing?.unresolved), this.learning.hasPending, Boolean(this.learning.snapshot?.activeActivity)) ? '' : 'disabled'}>Visit my earlier adventure</button></div><p class="campaign-note">Both adventures stay saved. Finish any current practice or pending move before switching.</p>`;
    const puzzles = savedPuzzles(s.flags);
    const waterStatus = puzzles
      .map(
        (puzzle) =>
          `<section class="campaign-hazard" aria-label="${escape(puzzle.title)}"><strong>${escape(puzzle.title)}</strong>${puzzle.settings.map((setting) => `<p>${escape(name(setting.setting))} → ${escape(name(setting.value))}</p>`).join('')}<p>${puzzle.status === 'solved' ? 'Solved: the mechanism is working.' : puzzle.status === 'blocked' ? 'That arrangement is blocked. Adjust the settings and check again.' : 'Settings saved. Check the mechanism when you are ready.'}</p></section>`,
      )
      .join('');
    const objective = nextObjective(c.legal_actions, s.grants, s.flags, s.starter);
    const brief = missionBrief(s.flags, objective?.mission);
    if (
      objective?.id === 'bridge_gather' ||
      (s.location === 'picnic_clearing' &&
        s.flags.includes('bridge_clue_complete') &&
        !s.flags.includes('bridge_gather_done'))
    ) {
      const found = s.flags.filter((flag) => /^find_branch_[123]$/.test(flag)).length;
      return `<section class="campaign-objective"><h3>Search the clearing for fallen branches</h3><p>The crossing needs three sound branches. Hazel spotted some in the picnic clearing on this side of the river.</p><p><strong>Branches found: ${found} / 3</strong></p><p>Look by the stump, the picnic blanket and the fallen leaves. Walk to a branch and pick it up.</p><div class="campaign-actions"><button id="campaign-explore" class="campaign-primary">Explore this place</button>${
        s.location === 'bridge_bank'
          ? this.button('Take the path to Picnic Clearing', {
              type: 'travel',
              location_id: 'picnic_clearing',
            })
          : this.campaign.legal_actions
              .filter((action) => action.id.startsWith('find_branch_'))
              .map(
                (action) =>
                  `<button data-campaign-command="${action.id}">Walk to branch ${action.id.at(-1)}</button>`,
              )
              .join('')
      }</div></section>`;
    }
    const trail = !objective ? nextTrail(s.flags, s.location, c.travel) : undefined;
    const placeName = (id: string) =>
      this.snapshot.locations.find((p) => p.id === id)?.name ?? name(id);
    const choices = objective?.puzzle
      ? c.legal_actions.filter((action) => action.puzzle?.id === objective.puzzle?.id)
      : objective?.kind === 'choose'
        ? c.legal_actions.filter(
            (action) => action.kind === 'choose' && action.mission === objective.mission,
          )
        : [];
    const others = c.legal_actions.filter(
      (action) =>
        action.id !== 'bridge_gather' && action !== objective && !choices.includes(action),
    );
    return `${storySoFar(s.flags) ? `<section class="campaign-dialogue" aria-label="Story so far"><strong>The story so far</strong><p>${escape(storySoFar(s.flags)!)}</p></section>` : ''}<section class="campaign-objective" aria-label="Current mission"><span class="campaign-eyebrow">Our mission</span><h3>${escape(brief.title)}</h3><p>${escape(brief.story)}</p><p><strong>What we’re trying to do:</strong> ${escape(brief.goal)}</p><div class="campaign-actions"><button id="campaign-explore" class="campaign-primary">Explore this place</button></div><h3>Where to look next</h3>${choices.length ? `<p>${objective?.puzzle ? 'Try the controls below, then check your arrangement. You can change it and try again.' : 'Choose how you would like to finish this part of the adventure.'}</p><div class="campaign-actions">${choices.map((action) => `<button data-campaign-command="${escape(action.id)}">${escape(action.label)}</button>`).join('')}</div>` : objective ? `<p>${escape(objective.label)}${objective.learning ? ' — a short activity will help us finish this step. Help is always available.' : '.'}</p><div class="campaign-actions"><button data-campaign-command="${escape(objective.id)}">${escape(objective.label)}</button></div>` : trail ? `<p>Head towards ${escape(placeName(trail.target))}. Take the path to ${escape(placeName(trail.step))} next.</p><div class="campaign-actions">${this.button('Go to ' + placeName(trail.step), { type: 'travel', location_id: trail.step })}</div>` : '<p>Explore the open paths or your friends’ invitations below.</p>'}</section>${waterStatus}${c.travel.length ? '<h3>Where to go</h3><p>These paths take you to another place. Your progress here stays saved.</p>' : ''}<div class="campaign-actions">${c.travel
      .filter((id) => id !== trail?.step)
      .map((id) =>
        this.button(
          'Go to ' + (this.snapshot.locations.find((p) => p.id === id)?.name ?? name(id)),
          { type: 'travel', location_id: id },
        ),
      )
      .join(
        '',
      )}</div>${others.length ? `<details class="campaign-optional"><summary>Other things to try here (${others.length})</summary><p>Extra practice, discoveries and visits. You can come back to these.</p><div class="campaign-actions">${others.map((action) => `<button data-campaign-command="${escape(action.id)}">${escape(action.label)}</button>`).join('')}</div></details>` : ''}`;
  }
  private encounter(): string {
    const s = this.campaign.state,
      e = s.encounter!,
      d = this.campaign.encounter_definition!;
    const phase = encounterPhase(d, e.progress);
    const story = challengeStory(e.id);
    const project = projectBoard(e.id, s.flags, this.projectPiece, (label, movement, disabled) =>
      this.button(
        label,
        { type: 'encounter_turn', companion_id: s.team[0]!, movement },
        disabled,
        ['bridge_cross', 'garden_check'].includes(movement),
      ),
    );
    if (project)
      return `${project}<div class="campaign-actions">${this.button('Help with one step', { type: 'encounter_turn', companion_id: s.team[0]!, movement: 'guided' })}${this.button('Return to camp', { type: 'encounter_turn', companion_id: s.team[0]!, movement: 'retreat' })}</div>`;
    const relay = d.relay
      ? `<div class="campaign-hazard"><strong>Relay round ${1 + Math.min(d.relay.round_targets.length - 1, d.relay.round_targets.filter((target) => e.progress >= target).length)} of ${d.relay.round_targets.length}</strong><p>Round goals: ${d.relay.round_targets.join(' → ')} progress. Bring ${d.relay.required_affinities} different affinities.</p><p>Contributed affinities: ${
          s.flags
            .filter((flag) => flag.startsWith(`relay:${e.id}:affinity:`))
            .map((flag) => escape(name(flag.split(':').at(-1)!)))
            .join(', ') || 'none yet'
        }</p></div>`
      : '';
    const actor = s.team.includes(this.selected ?? '') ? this.selected! : s.team[0]!;
    const affinity =
      this.campaign.companions.find((friend) => friend.id === actor)?.affinity ??
      companionAffinities[actor] ??
      'gleam';
    const freeSignature =
      s.flags.includes(`technique:${actor}:free_signature`) &&
      !s.flags.includes(`free_signature_used:${actor}`);
    const signatureEnergy = freeSignature ? 0 : signatureCost(affinity);
    const descriptions: Record<string, string> = {
      focus: 'Make steady progress on the goal. No energy needed.',
      guard: 'Block up to 2 pressure from this hazard. Uses 1 energy.',
      rest: 'Recover up to 2 energy. Does not advance the goal.',
      signature: `Use ${name(actor)}’s special ${name(affinity)} move to advance the goal${affinity === phase.affinity ? ' with a matching-approach bonus' : ''}. Uses ${signatureEnergy} energy.`,
      rally: 'Boost another friend’s next progress move. Uses 1 energy. Requires level 2.',
      tool: `Add 2 progress and block 1 pressure. No energy needed. ${e.tool_charges} charges left.`,
      snack: 'Use one snack to recover up to 2 energy.',
      shelter_patch: 'Use one patch to block up to 2 pressure from this hazard.',
      trail_marker: 'Use one marker to boost the next progress move.',
    };
    const moves = [
      'focus',
      'guard',
      'rest',
      'signature',
      'rally',
      'tool',
      'snack',
      'shelter_patch',
      'trail_marker',
    ]
      .map((movement) => {
        const cost =
          movement === 'signature'
            ? signatureEnergy
            : ['guard', 'rally'].includes(movement)
              ? 1
              : 0;
        const reason =
          movement === 'rally' &&
          (this.campaign.companions.find((friend) => friend.id === actor)?.level ?? 1) < 2
            ? 'Available at level 2.'
            : movement === 'tool' && e.tool_charges === 0
              ? 'No charges left.'
              : ['snack', 'shelter_patch', 'trail_marker'].includes(movement) &&
                  !(s.items[movement] ?? 0)
                ? 'None in your bag.'
                : (s.energy[actor] ?? 0) < cost
                  ? `Needs ${cost} energy. Try Rest first.`
                  : '';
        const label = movement === 'tool' ? `Keeper tool (${e.tool_charges})` : name(movement);
        const button = this.button(
          label,
          { type: 'encounter_turn', companion_id: actor, movement },
          Boolean(reason),
          movement === 'focus',
        ).replace('<button ', `<button aria-describedby="move-${movement}-help" `);
        return `<div class="campaign-move">${button}<p id="move-${movement}-help">${escape(descriptions[movement]!)}${reason ? `<strong>${escape(reason)}</strong>` : ''}</p></div>`;
      })
      .join('');
    return `<div class="campaign-trial-art" aria-hidden="true">${portrait(this.friendSprite(actor))}<span>✦ · ✦ · ✦</span>${portrait('collector-cart')}${this.confirmedEffect ? `<div class="campaign-effect-bitmap" aria-hidden="true">${portrait(this.confirmedEffect.id)}</div>` : ''}</div>${this.confirmedEffect ? `<p role="status">${escape(this.confirmedEffect.text)}</p>` : ''}${relay}<p>${escape(story.goal)} You can take your time choosing each move.</p><div class="campaign-stat"><span>${escape(story.progress)}</span><strong>${e.progress} / ${d.target}</strong></div><progress aria-label="${escape(story.progress)}" class="campaign-meter" value="${e.progress}" max="${d.target}">${e.progress} / ${d.target}</progress><div class="campaign-stat"><span>Pressure</span><strong>${e.pressure} / ${d.pressure_limit}</strong></div><div class="campaign-stat"><span>Turns</span><strong>${e.turns} / ${d.turn_limit}</strong></div><div class="campaign-hazard">${portrait('affinity-' + phase.affinity)}<strong>Helpful approach: ${escape(name(phase.affinity))}</strong><p>${phase.next ? `At ${phase.next.threshold} progress, the challenge changes to ${escape(name(phase.next.affinity))}.` : 'This is the final phase.'}</p><strong>Next hazard: ${d.hazards[e.turns % d.hazards.length] ?? 0} pressure</strong>${s.flags.includes(`encounter_reveal:${e.id}:${e.turns}`) ? `<p>Gleam reveals the following hazard too: ${d.hazards[(e.turns + 1) % d.hazards.length] ?? 0} pressure.</p>` : ''}<p>Each move takes one turn, then the next hazard adds pressure. Finish the goal before pressure or turns run out.</p></div><div class="campaign-actions">${this.button('Return to camp', { type: 'encounter_turn', companion_id: actor, movement: 'retreat' })}${this.button(
      'Let’s do this together',
      { type: 'encounter_turn', companion_id: actor, movement: 'guided' },
    )
      .replace('campaign-secondary', 'campaign-secondary campaign-support')
      .replace(
        '>Let’s',
        '>' + activityIcon('together').outerHTML + 'Let’s',
      )}<button class="campaign-secondary campaign-support" id="campaign-guided" aria-pressed="${this.guided}">${activityIcon('hint').outerHTML}${this.guided ? 'Hide trial hints' : 'Let’s plan together'}</button></div>${this.guided ? `<div class="campaign-dialogue"><strong>Plan your next move</strong><p>You have ${e.tool_charges} keeper tool charges left. Focus makes progress without spending energy. Guard helps with a strong hazard; Rest recovers energy. A signature matching the helpful approach gets a bonus.</p></div>` : ''}<section class="campaign-trial-section"><h3>1. Who will help?</h3><p>Choose the friend who will make your next move. Choosing a friend does not use a turn.</p><div class="campaign-actions campaign-team-choice">${s.team.map((id) => `<button data-campaign-select="${escape(id)}" aria-pressed="${actor === id}">${escape(name(id))} · ${s.energy[id] ?? 0} energy${actor === id ? ' · Selected' : ''}</button>`).join('')}</div></section><section class="campaign-trial-section"><h3>2. Choose a move for ${escape(name(actor))}</h3><p>Choose one action below. For help, “Let’s do this together” chooses and plays one move for you; “Let’s plan together” shows hints without using a turn.</p><div class="campaign-actions campaign-moves">${moves}</div></section>`;
  }
  private async run(payload: CampaignPayload) {
    if (!this.learning.snapshot?.learnerAge) return this.render();
    if (this.commands.busy || this.commands.hasPending) return;
    if (this.commands.needsRefresh || this.refreshNeeded) {
      this.render();
      return;
    }
    this.status = '';
    this.confirmedEffect = undefined;
    this.formNotice = undefined;
    if (
      payload.type === 'action' &&
      this.campaign.legal_actions.find((a) => a.id === payload.action_id)?.learning
    ) {
      await this.beginLearning(payload.action_id);
      return;
    }
    document
      .querySelectorAll<HTMLButtonElement>('#dialog button:not(.close)')
      .forEach((b) => (b.disabled = true));
    const before = this.campaign.state;
    if (
      payload.type === 'care' &&
      ['appearance_base', 'appearance_wayfinder'].includes(payload.kind)
    )
      this.pendingAppearance = {
        id: payload.companion_id,
        before: this.campaign.companions.map((friend) => ({ ...friend })),
      };
    let writeConfirmed = false;
    try {
      await this.commands.choose(payload);
      writeConfirmed = true;
      await this.commands.refresh();
      this.apply();
      this.confirmFormPresentation();
      const after = this.campaign.state;
      if (payload.type === 'encounter_turn' && payload.movement === 'signature') {
        const affinity =
          this.campaign.companions.find((friend) => friend.id === payload.companion_id)?.affinity ??
          companionAffinities[payload.companion_id] ??
          'gleam';
        this.confirmedEffect = {
          id: 'effect-' + affinity,
          text: `${name(payload.companion_id)} used their ${name(affinity)} signature. ${after.encounter ? `Progress: ${after.encounter.progress}.` : 'The goal is complete.'}`,
        };
      }
      if (
        before.encounter &&
        !after.encounter &&
        after.grants.includes(`win:${before.encounter.id}`)
      )
        this.confirmedEffect = {
          id: 'effect-celebration',
          text: challengeStory(before.encounter.id, after.flags).outcome,
        };
      if (payload.type === 'encounter_turn' && payload.movement === 'signature')
        this.game?.events.emit('campaign-pose', payload.companion_id, 'signature');
      if (
        before.encounter &&
        !after.encounter &&
        after.grants.includes(`win:${before.encounter.id}`)
      )
        for (const id of after.team) this.game?.events.emit('campaign-pose', id, 'cheer');
      if (payload.type === 'camp')
        for (const id of after.team) this.game?.events.emit('campaign-pose', id, 'rest');
      if (payload.type === 'care' && ['quiet_rest', 'play'].includes(payload.kind))
        this.game?.events.emit(
          'campaign-pose',
          payload.companion_id,
          payload.kind === 'play' ? 'cheer' : 'rest',
        );
      this.status =
        before.encounter && !after.encounter
          ? after.grants.includes(`win:${before.encounter.id}`) &&
            !before.grants.includes(`win:${before.encounter.id}`)
            ? challengeStory(before.encounter.id, after.flags).outcome
            : !after.grants.includes(`win:${before.encounter.id}`) ||
                after.location !== before.location ||
                (payload.type === 'encounter_turn' && payload.movement === 'retreat')
              ? 'Back at camp. Rest with your friends and try the trial again.'
              : 'Practice complete. Your discoveries are safely saved.'
          : 'Saved. Your adventure continues.';
      if (payload.type === 'action' && payload.action_id.startsWith('find_branch_')) {
        const count = after.flags.filter((flag) => /^find_branch_[123]$/.test(flag)).length;
        this.status =
          count === 3
            ? 'Three fallen branches found! Bring them back to Bridge Bank and meet the woodcutter.'
            : `Found a fallen branch: ${count} of 3. Keep exploring the clearing.`;
      }
      if (payload.type === 'care') {
        const friend = fieldGuide.friends.find((friend) => friend.id === payload.companion_id);
        const labels: Record<string, string> = {
          play: 'played together',
          groom: 'shared a gentle grooming moment',
          quiet_rest: 'rested quietly together',
          favourite_place: `remembered ${friend?.habitat ?? 'a favourite place'} together`,
        };
        if (labels[payload.kind])
          this.status = `Saved. You and ${friend?.name ?? name(payload.companion_id)} ${labels[payload.kind]}.`;
      }
    } catch {
      if (!writeConfirmed && !this.commands.hasPending) this.pendingAppearance = undefined;
      this.status = this.commands.hasPending
        ? 'We could not confirm that move. Retry the saved move.'
        : payload.type === 'encounter_turn' && payload.movement.startsWith('bridge_')
          ? 'That piece does not fit yet. Match the plank length to the gap, then brace both banks before crossing.'
          : 'That move is not available yet. Check the project instructions or ask for help.';
      try {
        if (!this.commands.hasPending) {
          await this.commands.refresh();
          if (writeConfirmed) {
            this.apply();
            this.confirmFormPresentation();
          }
        }
      } catch {
        this.status = 'Reconnect to load your adventure.';
      }
    }
    if (
      this.exploring &&
      !this.campaign.state.encounter &&
      !this.commands.hasPending &&
      !this.refreshNeeded &&
      !this.commands.needsRefresh
    ) {
      this.hooks.close();
      this.hooks.toast(this.status);
    } else this.render();
  }

  private async learningCommand(payload: Parameters<MissionLearningSession['choose']>[0]) {
    if (this.learning.needsRefresh) await this.learning.refresh();
    return this.learning.choose(payload);
  }
  private async beginLearning(actionId: string) {
    try {
      await this.learning.refresh();
      if (this.learning.hasPending) {
        await this.learning.retry();
        await this.learning.refresh();
      }
      if (!this.learning.snapshot?.activeActivity)
        await this.learningCommand({
          type: 'begin_campaign_step',
          actionId,
          campaignRevision: this.snapshot.revision,
        });
      this.showLearning();
    } catch {
      this.status = 'Practice could not connect. Your adventure is saved. Try again.';
      this.render();
    }
  }
  private showLearning(snapshot = this.learning.snapshot) {
    const activity = snapshot?.activeActivity;
    if (!activity || !snapshot) return this.missions();
    this.learningBaseline ??= this.campaign.companions.map((friend) => ({ ...friend }));
    this.learningActivity?.destroy();
    this.hooks.open(
      '<section class="campaign-panel"><span class="campaign-eyebrow">A little practice, a big adventure</span><div id="mission-practice"></div></section>',
    );
    this.learningActivity = mountMissionActivity(
      document.querySelector<HTMLElement>('#mission-practice')!,
      activity,
      snapshot,
      {
        onReplace: (updated) => this.showLearning(updated),
        onAttempt: (answer) =>
          this.learningCommand({
            type: 'answer',
            encounterId: activity.id,
            variantId: activity.variantId,
            answer,
          }),
        onSupport: (kind) =>
          this.learningCommand({ type: 'support', encounterId: activity.id, kind }),
        onRetry: () => this.learning.retry(),
        onRefresh: () => this.learning.refresh(),
        hasPending: () => this.learning.hasPending,
        onAbandon: async () => {
          await this.learningCommand({ type: 'abandon', encounterId: activity.id });
        },
        onFinishTogether: () =>
          this.learningCommand({
            type: 'finish_together',
            encounterId: activity.id,
            variantId: activity.variantId,
          }),
        onContinue: () => {
          this.learningActivity?.destroy();
          void this.commands
            .refresh()
            .then(() => {
              this.refreshNeeded = false;
              this.apply();
              this.confirmFormPresentation();
              if (this.exploring) {
                this.hooks.close();
                this.hooks.toast('That step is finished. Explore the next part of the trail.');
              } else this.missions();
            })
            .catch(() => {
              this.refreshNeeded = true;
              this.status = 'Your practice is saved. Refresh the adventure to continue.';
              this.render();
            });
        },
      },
    );
    this.learningActivity.focus();
  }
  private async refreshAdventure() {
    if (this.commands.busy || this.commands.hasPending) return;
    document
      .querySelectorAll<HTMLButtonElement>('#dialog button:not(.close)')
      .forEach((button) => (button.disabled = true));
    try {
      await this.commands.refresh();
      await this.learning.refresh();
      this.refreshNeeded = false;
      this.apply();
      this.confirmFormPresentation();
      this.status = 'Adventure refreshed. Your saved progress is ready.';
    } catch {
      this.refreshNeeded = true;
      this.status = 'The adventure could not refresh. Try again when connected.';
    }
    this.render();
  }
  private async retry() {
    let writeConfirmed = false;
    try {
      await this.commands.retry();
      writeConfirmed = true;
      await this.commands.refresh();
      this.apply();
      this.confirmFormPresentation();
      this.status = 'Your move is safely saved.';
    } catch {
      if (!writeConfirmed && !this.commands.hasPending) this.pendingAppearance = undefined;
      this.status = 'Still waiting to confirm. Your saved move is ready to retry.';
    }
    this.render();
  }
}
