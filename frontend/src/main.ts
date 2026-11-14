import { connectStoryWorld, type StoryWorld } from './story/world.ts';
import type Phaser from 'phaser';
import { CampaignWorld, portrait } from './campaign/world.ts';
import { MissionLearningClient } from './mission-learning/client.ts';
import { PairedClient } from './paired/client.ts';
import { allowedProfile } from './paired/panel.ts';
import type { HouseholdProfile } from '../../contracts/generated/contracts.ts';
import { CampaignClient } from './campaign/client.ts';
import { adventureUrl, earlierAdventureRequested } from './campaign/presentation.ts';
import './style.css';
import './redesign.css';
import './fullscreen.css';
import './exploration.css';
import './dialogue.css';
function $<T extends HTMLElement = HTMLElement>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Missing required element: ${selector}`);
  return element;
}
const escape = (s: string) =>
  String(s).replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] ?? c,
  );
let game: Phaser.Game;
let story: StoryWorld | CampaignWorld | undefined;
let returnFocus: HTMLElement | null = null;
let toastTimer: ReturnType<typeof setTimeout> | undefined;
$('#app').innerHTML =
  `<main class="game-shell"><div id="game" role="application" aria-label="Clover Meadow. Tap to walk or use arrow keys and WASD. Press Space near a friend. Open Missions to find an activity."></div>
<header class="game-hud"><div class="location-badge"><img class="app-logo" src="/assets/starwhisker-logo-v1.png" width="64" height="64" alt=""/><div><small>WONDERWILDS</small><h1>Clover Meadow</h1></div><button class="location-customize" id="character" aria-label="Customize my character" title="Customize my character"></button></div><nav aria-label="Game tools"><div class="hud-stars" aria-label="Hero stars"><span>✦</span><strong id="star-count">0 / 3</strong></div><button class="hud-button" id="bag" aria-label="Open my bag" title="My bag"><svg width="25" height="25" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 7V6a4 4 0 0 1 8 0v1M6 7h12l2 14H4L6 7Z"/><path d="M9 12a3 3 0 0 0 6 0"/></svg><span>Bag</span></button><button class="hud-button fullscreen-button" id="fullscreen" aria-label="Enter full screen" title="Enter full screen">⛶</button></nav></header>
<footer class="game-dock"><button class="mission-button" id="missions"><span class="mission-symbol">✦</span><span><small>YOUR ADVENTURE</small><strong id="mission-status">Three ways to be a hero</strong></span><span class="mission-open">Missions ↗</span></button><div class="game-guidance"><span id="quest-direction" role="status"></span><button id="quest-guide" class="secondary" hidden>Follow path</button><span>Tap to explore <b>·</b> Arrows to walk <b>·</b> Space to chat</span><small id="story-source-status" role="status">Preauthored story</small><small id="save-status" role="status"></small></div></footer></main><dialog id="dialog" aria-labelledby="dialog-title"></dialog><div id="toast" role="status"></div>`;
const dialog = $<HTMLDialogElement>('#dialog');
async function toggleFullscreen() {
  try {
    if (document.fullscreenElement) await document.exitFullscreen();
    else if (document.documentElement.requestFullscreen)
      await document.documentElement.requestFullscreen();
    else toast('Your adventure already fills this browser window.');
  } catch {
    toast('Full screen is unavailable here. Your game still fills the window.');
  }
}
document.addEventListener('fullscreenchange', () => {
  const button = $('#fullscreen');
  button.setAttribute(
    'aria-label',
    document.fullscreenElement ? 'Exit full screen' : 'Enter full screen',
  );
  button.title = button.getAttribute('aria-label') ?? '';
});
function openDialog(html: string) {
  if (
    !dialog.contains(document.activeElement) &&
    document.activeElement instanceof HTMLElement &&
    document.activeElement.isConnected
  )
    returnFocus = document.activeElement;
  dialog.innerHTML = `<button class="close" aria-label="Close dialog">×</button>${html}`;
  dialog.showModal();
  const scene = game?.scene.getScene('island');
  if (scene) {
    scene.input.keyboard?.clearCaptures();
    if (scene.input.keyboard) scene.input.keyboard.enabled = false;
    scene.input.keyboard?.resetKeys();
  }
  dialog.querySelector<HTMLElement>('button:not(.close),input')?.focus();
}
function closeDialog() {
  dialog.close();
  window.speechSynthesis?.cancel();
  const scene = game?.scene.getScene('island');
  if (scene) {
    scene.input.keyboard?.resetKeys();
    if (scene.input.keyboard) scene.input.keyboard.enabled = true;
    scene.input.keyboard?.addCapture(['UP', 'DOWN', 'LEFT', 'RIGHT', 'SPACE']);
  }
  returnFocus?.focus();
}
dialog.addEventListener('cancel', (e) => {
  e.preventDefault();
  closeDialog();
});
function toast(message: string) {
  $('#toast').textContent = message;
  $('#toast').classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => $('#toast').classList.remove('show'), 4200);
}
document.addEventListener('click', (event) => {
  if (!(event.target instanceof Element)) return;
  const button = event.target.closest('button');
  if (!button) return;
  if (button.classList.contains('close')) return closeDialog();
  if (button.id === 'campaign-return') {
    location.assign(adventureUrl(location.href, false));
    return;
  }
  if (button.id === 'fullscreen') return void toggleFullscreen();
  story?.handle(button);
});
async function boot() {
  let startupStage = 'session';
  const tools = document.querySelectorAll<HTMLButtonElement>('.game-hud button, .game-dock button');
  tools.forEach((button) => {
    button.disabled = true;
  });
  try {
    $('#story-source-status').textContent = 'Checking story source…';
    const hooks = { open: openDialog, close: closeDialog, toast };
    const response = await fetch('/__demo/bootstrap', {
      redirect: 'error',
      signal: AbortSignal.timeout(10000),
    });
    if (!response.ok) throw new Error('Session unavailable');
    const bootstrap: unknown = await response.json();
    if (
      !bootstrap ||
      typeof bootstrap !== 'object' ||
      !('profileId' in bootstrap) ||
      !('csrf' in bootstrap) ||
      typeof bootstrap.profileId !== 'string' ||
      typeof bootstrap.csrf !== 'string'
    )
      throw new Error('Invalid session');
    let profiles: HouseholdProfile[] = [];
    const requestedPlayer = new URL(location.href).searchParams.get('player');
    if (!earlierAdventureRequested(location.href) || requestedPlayer) {
      const directory = new PairedClient(bootstrap.profileId, bootstrap.csrf);
      try {
        profiles = (await directory.profiles()).profiles;
      } catch {
        if (requestedPlayer)
          throw new Error(
            'The selected player could not be verified. Reconnect to choose a household player.',
          );
      } finally {
        directory.destroy();
      }
    }
    const selectedProfile = allowedProfile(requestedPlayer, bootstrap.profileId, profiles);
    if (earlierAdventureRequested(location.href)) {
      startupStage = 'earlier adventure';
      story = await connectStoryWorld(hooks, selectedProfile);
      const button = document.createElement('button');
      button.id = 'campaign-return';
      button.className = 'hud-button';
      button.textContent = 'Moonlight trail';
      button.setAttribute('aria-label', 'Return to The Missing Moonlight');
      document.querySelector('.game-hud nav')?.prepend(button);
    } else {
      const campaign = new CampaignWorld(
        new CampaignClient(selectedProfile, bootstrap.csrf),
        selectedProfile,
        hooks,
        new MissionLearningClient(selectedProfile, bootstrap.csrf),
        { profiles, client: new PairedClient(selectedProfile, bootstrap.csrf) },
      );
      startupStage = 'campaign snapshot';
      const snapshot = await campaign.load();
      if (snapshot.adopted) story = campaign;
      else {
        const adopt = await new Promise<boolean>((resolve) => {
          const listeners = new AbortController();
          const choose = (adopt: boolean) => {
            listeners.abort();
            resolve(adopt);
          };
          dialog.addEventListener('close', () => choose(false), { signal: listeners.signal });
          openDialog(
            `<section class="campaign-panel"><span class="campaign-eyebrow">A new adventure awaits</span><h2 id="dialog-title">Where has the moonlight gone?</h2><div class="campaign-intro-art">${['pip', 'tansy', 'nib'].map(portrait).join('')}</div><p>Meet Hazel, choose a companion and follow a trail of fading light. Restore the bridge and garden together, one small act of courage at a time.</p><p>Your earlier stories and friends stay saved.</p>${snapshot.adoptionBlocked ? '<p class="campaign-hazard">Finish your current activity in the earlier adventure before starting this chapter.</p>' : ''}<div class="campaign-actions"><button id="campaign-start" ${snapshot.adoptionBlocked ? 'disabled' : ''}>Begin The Missing Moonlight</button><button id="campaign-legacy">Continue my earlier adventure</button></div></section>`,
          );
          document
            .querySelector('#campaign-start')
            ?.addEventListener('click', () => choose(true), { signal: listeners.signal });
          document
            .querySelector('#campaign-legacy')
            ?.addEventListener('click', () => choose(false), { signal: listeners.signal });
        });
        closeDialog();
        if (adopt) {
          startupStage = 'campaign adoption';
          await campaign.adopt();
          story = campaign;
        } else story = await connectStoryWorld(hooks, selectedProfile);
      }
    }
    if (story instanceof CampaignWorld) {
      const players = document.createElement('button');
      players.id = 'players';
      players.className = 'hud-button';
      players.textContent = 'Choose player';
      document.querySelector('.game-hud nav')?.prepend(players);
    }
    startupStage = 'world renderer';
    game = (await story.start())!;
    tools.forEach((button) => {
      button.disabled = false;
    });
    toast('Open Missions to continue your story and follow a path.');
  } catch (error) {
    console.error(
      'Adventure startup failed at ' + startupStage,
      error instanceof Error ? error.message : 'Unknown error',
    );
    $('#story-source-status').textContent = 'Story source unavailable';
    const wrongLocalAddress = location.hostname === 'localhost';
    const localAddress = new URL(location.href);
    localAddress.hostname = '127.0.0.1';
    $('#save-status').textContent = wrongLocalAddress
      ? 'The local demo requires the 127.0.0.1 address.'
      : 'Story startup failed. Reconnect to try again.';
    openDialog(
      `<h2 id="dialog-title">Reconnect Pip’s story</h2>${wrongLocalAddress ? `<p>The demo session cannot connect through localhost. Open the same story at 127.0.0.1.</p><p><a class="primary" href="${escape(localAddress.href)}">Open the local story address</a></p>` : '<p>The local session, saved story, or browser storage could not be loaded. Check that the local demo is running, then reconnect.</p>'}<button class="primary" id="reconnect-story">Reconnect</button>`,
    );
    $('#reconnect-story').onclick = () => location.reload();
  }
}
void boot();
