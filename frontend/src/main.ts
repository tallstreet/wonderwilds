import { connectStoryWorld, type StoryWorld } from './story/world.ts';
import type Phaser from 'phaser';
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
let story: StoryWorld | undefined;
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
  if (button.id === 'fullscreen') return void toggleFullscreen();
  story?.handle(button);
});
async function boot() {
  const tools = document.querySelectorAll<HTMLButtonElement>('.game-hud button, .game-dock button');
  tools.forEach((button) => {
    button.disabled = true;
  });
  try {
    $('#story-source-status').textContent = 'Checking story source…';
    story = await connectStoryWorld({ open: openDialog, close: closeDialog, toast });
    game = (await story.start())!;
    tools.forEach((button) => {
      button.disabled = false;
    });
    toast('Open Missions to continue your story and follow a path.');
  } catch {
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
