import { connectBackend, completedWorldQuests, type BackendWorld } from './backend-world.ts';
import type Phaser from 'phaser';
import type { Activity, Encounter } from './types.ts';
import './style.css';
import './redesign.css';
import './fullscreen.css';
import './exploration.css';
import { mapImageUrl } from './mapImages.ts';
import { createWorld } from './world.ts';
import { heroes, heroPortrait } from './art.ts';
import { propArt } from './sprites.ts';
import { maps, biomes, isMapId, type MapId } from './maps.ts';
import { answerFor, isCorrect } from './content.ts';
import {
  activeEncounters,
  freshState,
  readProgress,
  writeProgress,
  completeEncounter,
  skillSummary,
} from './progress.ts';
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
let storage: Storage | undefined;
try {
  storage = window.localStorage;
} catch {}
let { state, available } = readProgress(storage);
let game: Phaser.Game;
let session: Activity | null = null;
let backend: BackendWorld | undefined;
let returnFocus: HTMLElement | null = null;
let toastTimer: ReturnType<typeof setTimeout> | undefined;
function persist() {
  if (backend) {
    backend.saveView(state);
    backend.updateHud();
    return;
  }
  available = writeProgress(state, storage);
  $('#save-status').textContent = available
    ? 'Progress saved on this device'
    : 'Session only — saving is unavailable';
}
$('#app').innerHTML =
  `<main class="game-shell"><div id="game" role="application" aria-label="Clover Meadow. Tap to walk or use arrow keys and WASD. Press Space near a friend. Open Missions to find an activity."></div>
<header class="game-hud"><div class="location-badge"><img class="app-logo" src="/assets/starwhisker-logo-v1.png" width="64" height="64" alt=""/><div><small>STARWHISKER ISLAND</small><h1>Clover Meadow</h1></div></div><nav aria-label="Game tools"><div class="hud-stars" aria-label="Hero stars"><span>✦</span><strong id="star-count">0 / 3</strong></div><button class="hud-button" id="explore" aria-label="Explore 75 places">◎<span>Explore</span></button><button class="hud-button" id="journal" aria-label="My journal" title="My journal">▧<span>Journal</span></button><button class="hud-button" id="grownup" aria-label="For grown-ups" title="For grown-ups">⚙<span>Grown-ups</span></button><button class="hud-button fullscreen-button" id="fullscreen" aria-label="Enter full screen" title="Enter full screen">⛶</button></nav></header>
<footer class="game-dock"><button class="mission-button" id="missions"><span class="mission-symbol">✦</span><span><small>YOUR ADVENTURE</small><strong id="mission-status">Three ways to be a hero</strong></span><span class="mission-open">Missions ↗</span></button><div class="game-guidance"><span>Tap to explore <b>·</b> Arrows to walk <b>·</b> Space to chat</span><small id="save-status" role="status"></small></div></footer></main><dialog id="dialog" aria-labelledby="dialog-title"></dialog><div id="toast" role="status"></div>`;
const dialog = $<HTMLDialogElement>('#dialog');
function render() {
  const mapName = maps[state.region as MapId].name;
  $('.location-badge h1').textContent = mapName;
  $('#game').setAttribute(
    'aria-label',
    `${mapName}. Tap to walk or use arrow keys and WASD. Follow the paths off the edge to explore the next map. Press Space near a friend.`,
  );
  $('#star-count').textContent = `${state.done.length} / 3`;
  $('#mission-status').textContent =
    state.done.length === 3
      ? 'A meadow full of friendship!'
      : `${3 - state.done.length} little missions await`;
  persist();
}
function missions() {
  if (backend) return backend.missions();
  openDialog(
    `<div class="mission-modal-art" aria-hidden="true"><img src="/assets/hero-squad.png" alt=""/></div><span class="pill">YOUR FIRST ADVENTURE</span><h2 id="dialog-title">Heroes, assemble!</h2><p class="muted">Choose a mission and we’ll walk there together.</p><div class="progress-track"><span style="width:${(state.done.length / 3) * 100}%"></span></div><div class="modal-quests">${activeEncounters.map((e, i) => `<button class="quest ${state.done.includes(e.id) ? 'complete' : ''}" data-visit="${e.id}"><span class="quest-icon">${state.done.includes(e.id) ? '✓' : ['▥', '♥', '✿'][i]}</span><span><strong>${e.short}</strong><small>${state.done.includes(e.id) ? 'Play again' : e.type === 'pattern' && !state.done.includes('meadow-bridge') ? 'First, fix the bridge' : ['Build & count', 'Share & care', 'Notice & grow'][i]}</small></span><span class="quest-arrow">→</span></button>`).join('')}</div>${state.done.length === 3 ? '<button class="celebrate-link" id="celebrate">✦ Celebrate our friendship</button>' : ''}<p class="tiny-note">Every little try is a brave one.</p>`,
  );
}
function explore() {
  openDialog(
    `<span class="pill">A WORLD OF LITTLE ADVENTURES</span><h2 id="dialog-title">75 places to explore</h2><p class="muted">Choose somewhere to visit, or follow the paths to walk between places. Story missions await in Clover Meadow.</p><label for="place-search">Find a place</label><input id="place-search" type="search" placeholder="Search places or environments…"/><div id="place-library"></div>`,
  );
  const show = () => {
    const query = $<HTMLInputElement>('#place-search').value.trim().toLowerCase();
    const groups = ['Island favourites', ...biomes.map((b) => b.name)];
    $('#place-library').innerHTML =
      groups
        .map((group, index) => {
          const places = Object.entries(maps).filter(
            ([, map]) =>
              (index === 0 ? map.biome === undefined : map.biome === index - 1) &&
              `${map.name} ${group}`.toLowerCase().includes(query),
          );
          return places.length
            ? `<section class="place-group"><h3>${group}</h3><div class="place-grid">${places.map(([id, map]) => `<button class="place-card" data-place="${id}" ${id === state.region ? 'aria-current="location"' : ''}><img src="${mapImageUrl(id)}" alt="" loading="lazy" decoding="async" width="224" height="138"/><strong>${map.name}</strong><small>${id === state.region ? 'You are here' : 'Visit →'}</small></button>`).join('')}</div></section>`
            : '';
        })
        .join('') || '<p>No places found. Try another name.</p>';
  };
  $('#place-search').addEventListener('input', show);
  show();
}
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
  if (backend?.ownsActivity) {
    void backend.leave();
    return;
  }
  dialog.close();
  session = null;
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
function speak(text: string) {
  if (!('speechSynthesis' in window)) {
    toast('Read-aloud is not available in this browser.');
    return;
  }
  window.speechSynthesis.cancel();
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.rate = 0.85;
  window.speechSynthesis.speak(utterance);
}
function startActivity(e: Encounter) {
  if (e.type === 'pattern' && !state.done.includes('meadow-bridge')) {
    toast('First, help Pip fix the bridge to reach the garden.');
    return;
  }
  const base = {
    attempts: 0,
    hints: 0,
    assisted: false,
    finished: false,
    feedback: '',
    hintText: '',
  };
  session =
    e.type === 'share'
      ? { ...base, kind: 'share', encounter: e, value: Array<number>(e.groups).fill(0) }
      : e.type === 'missing'
        ? { ...base, kind: 'missing', encounter: e, value: 0 }
        : { ...base, kind: 'pattern', encounter: e, value: 0 };
  openDialog(
    `<div class="activity-top"><span class="pill">${e.type === 'missing' ? 'BUILD & COUNT' : e.type === 'share' ? 'SHARE & CARE' : 'NOTICE & GROW'}</span><button class="read-aloud" id="read-prompt" aria-label="Read instructions aloud">◖)) Listen</button></div><div class="activity-title">${heroPortrait('pip', 76)}<div><h2 id="dialog-title">${e.title}</h2><p>${e.prompt}</p></div></div><div id="activity-body"></div><p id="feedback" class="feedback" aria-live="polite"></p><div class="activity-actions"><button class="secondary" id="hint">♡ A little hint</button><button class="primary" id="check">Check my idea <span>→</span></button></div><button class="text-button" id="together">Let’s do it together</button><p class="tiny-note">Take your time. You can always try again.</p>`,
  );
  drawActivity();
}
function drawActivity() {
  if (!session) return;
  const focused = document.activeElement?.id;
  if (session.kind === 'missing') {
    const { encounter: e, value: v } = session;
    $('#activity-body').innerHTML =
      `<div class="bridge-scene"><div class="river-ripples">≈ &nbsp;&nbsp; ≈ &nbsp;&nbsp; ≈</div><div class="plank-row">${Array.from({ length: e.total }, (_, i) => `<span class="plank ${i < e.given ? 'existing' : i < e.given + v ? 'added' : 'empty'}">${i < e.given + v ? propArt('plank', 80) + `<b>${i + 1}</b>` : ''}</span>`).join('')}</div><div class="river-ripples">≈ &nbsp;&nbsp; ≈ &nbsp;&nbsp; ≈</div></div>${v > e.total - e.given ? `<div class="extra-planks">${Array.from({ length: v - (e.total - e.given) }, () => propArt('plank', 45)).join('')}<small>These extra planks don’t fit. Take some back.</small></div>` : ''}<div class="quantity-control"><button id="minus" aria-label="Remove a plank" ${v === 0 ? 'disabled' : ''}>−</button><div><strong>${v}</strong><span>planks added</span></div><button id="plus" aria-label="Add a plank" ${v === e.total ? 'disabled' : ''}>+</button></div><p class="model-note">${e.given} ready + ${v} added = <strong>${e.given + v} planks</strong></p>`;
  }
  if (session.kind === 'share') {
    const { encounter: e, value: v } = session;
    $('#activity-body').innerHTML =
      `<div class="berry-basket"><span>Our berry basket</span><div>${Array.from({ length: e.total - v.reduce((a, b) => a + b, 0) }, () => propArt('berry', 42)).join('') || 'All berries shared!'}</div><small>${e.total - v.reduce((a, b) => a + b, 0)} left to share</small></div><div class="plates">${v.map((n, i) => `<div class="plate-card"><strong>${i === 0 ? 'Pip’s' : 'Your'} plate</strong><button class="plate" id="plate-${i}" data-plate="${i}" aria-label="Give one berry to ${i === 0 ? 'Pip' : 'your hero'}" ${v.reduce((a, b) => a + b, 0) === e.total ? 'disabled' : ''}><span>${Array.from({ length: n }, () => propArt('berry', 36)).join('') || '+'}</span><small>${n} berries</small></button><button class="text-button" id="remove-${i}" data-remove="${i}" ${n === 0 ? 'disabled' : ''}>Take one back</button></div>`).join('')}</div><p class="model-note">Tap a plate to give it one berry.</p>`;
  }
  if (session.kind === 'pattern') {
    const { encounter: e, value: v } = session;
    $('#activity-body').innerHTML = `<div class="flower-sequence">${Array.from(
      { length: e.length },
      (_, i) => {
        const n = i === e.length - 1 ? v : e.start + e.step * i;
        return `<div class="flower-pot ${i === e.length - 1 ? 'your-flower' : ''}"><div class="petals">${n ? Array.from({ length: n }, () => propArt('flower', 35)).join('') : '?'}</div><div class="pot">${n || '?'}</div></div>`;
      },
    ).join(
      '',
    )}</div><p class="model-note">Grow the last flower group. Count each little flower.</p><div class="quantity-control"><button id="minus" aria-label="Remove a flower" ${v === 0 ? 'disabled' : ''}>−</button><div><strong>${v}</strong><span>flowers</span></div><button id="plus" aria-label="Add a flower" ${v === 10 ? 'disabled' : ''}>+</button></div>`;
  }
  $('#feedback').textContent = session.feedback || session.hintText;
  $('#feedback').classList.toggle('hint-feedback', !!session.hintText && !session.feedback);
  if (focused) document.getElementById(focused)?.focus();
}
const hints = {
  missing: [
    'Count the empty spaces on the bridge. Each space needs one plank.',
    'Start at 6. Count on: 7, 8, 9, 10. Add one plank for each number.',
    'Four more planks will make 10. Add 4 and count the whole bridge.',
  ],
  share: [
    'Give one berry to Pip, then one to yourself. Keep taking turns.',
    'Check the two plates. Do they have the same number of berries?',
    'Six berries shared between two friends makes 3 berries on each plate.',
  ],
  pattern: [
    'Count the flowers from left to right. Each group has one more.',
    'The groups have 1, 2, 3, then 4 flowers. What comes after 4?',
    'The last group needs 5 flowers. Add them one by one.',
  ],
};
function check() {
  if (!session || session.finished) return;
  session.attempts++;
  if (!isCorrect(session.encounter, session.value)) {
    session.feedback =
      session.encounter.type === 'share'
        ? 'Let’s compare. Share every berry, with the same number on both plates.'
        : session.encounter.type === 'missing'
          ? 'Let’s count again. We need exactly 10 planks in our bridge.'
          : 'Look at the groups. Each new group has one more flower.';
    if (session.attempts >= 2) session.feedback += ' A little hint can help.';
    drawActivity();
    return;
  }
  session.finished = true;
  state = completeEncounter(state, session.encounter, session);
  render();
  game.events.emit('refresh');
  const e = session.encounter;
  dialog.innerHTML = `<button class="close" aria-label="Close dialog">×</button><div class="success"><div class="success-star">✦</div><span class="pill">A HELPING-PAW STAR</span><h2 id="dialog-title">${e.type === 'missing' ? 'A bridge to new adventures!' : e.type === 'share' ? 'A picnic for two!' : 'Look at your garden bloom!'}</h2><p>${e.type === 'missing' ? 'You added 4 planks. Now 6 + 4 makes 10. Let’s cross together!' : e.type === 'share' ? 'You shared 6 berries equally. Three for Pip and three for you!' : '1, 2, 3, 4, 5! Your bright idea woke the flowers.'}</p><div>${heroPortrait('pip', 120)}</div><button class="primary" id="return-world">${state.done.length === 3 ? 'Meet your adventure buddy' : 'Back to our adventure'} →</button><p class="tiny-note">${session.hints || session.assisted ? 'Learning together is a lovely way to grow.' : 'You found your own way. Nicely done!'}</p></div>`;
  $('#return-world').focus();
}
function celebration() {
  openDialog(
    `<div class="success"><span class="pill">CHAPTER COMPLETE</span><div class="friendship-art">${heroPortrait(state.hero, 135)}<span>♡</span>${heroPortrait('pip', 135)}</div><h2 id="dialog-title">Small paws. Big friendship.</h2><p>You repaired a bridge, shared a picnic, and made a garden bloom. ${escape(state.companionName)} is your adventure buddy now!</p><div class="three-stars">✦ ✦ ✦</div><button class="primary" id="keep-exploring">Keep exploring →</button><button class="text-button" id="open-journal">Visit my journal</button></div>`,
  );
}
function journal() {
  if (backend) return backend.journal(state);
  openDialog(
    `<span class="pill">OUR LITTLE MEMORIES</span><h2 id="dialog-title">My adventure journal</h2><p class="muted">Meet the Starwhisker team. Pick a hero for your next adventure.</p><p><a href="/characters.html">Meet all twelve Wonderwilds friends</a></p><div class="journal-hero">${heroPortrait(state.hero, 110)}<div><h3>${heroes[state.hero].name} the ${heroes[state.hero].species}</h3><p class="hero-description">${heroes[state.hero].description}</p></div></div><h3 class="hero-picker-heading">Choose your little hero</h3><div class="hero-choices">${Object.entries(
      heroes,
    )
      .map(
        ([id, h]) =>
          `<button class="hero-choice ${state.hero === id ? 'selected' : ''}" data-hero="${id}" aria-label="Play as ${h.name} the ${h.species}" aria-pressed="${state.hero === id}">${heroPortrait(id, 76)}<strong>${h.name}</strong><small>${h.species}</small></button>`,
      )
      .join(
        '',
      )}</div><div class="journal-entries">${activeEncounters.map((e, i) => `<div class="journal-entry ${state.done.includes(e.id) ? 'earned' : ''}"><span>${state.done.includes(e.id) ? ['▥', '♥', '✿'][i] : '◇'}</span><strong>${e.short}</strong><small>${state.done.includes(e.id) ? 'A lovely memory' : 'A discovery waiting for you'}</small></div>`).join('')}</div><div class="friend-name-form"><label for="companion-name">${state.done.length === 3 ? 'Your adventure buddy' : 'Your meadow friend'}</label><div><input id="companion-name" maxlength="20" value="${escape(state.companionName)}" aria-label="Companion name"/><button class="secondary" id="save-name">Save name</button></div><p class="tiny-note">${state.done.length === 3 ? 'Pip will follow you around the meadow.' : 'Finish the three activities and Pip will join your team.'}</p></div>`,
  );
}
function grownup() {
  if (backend) return backend.grownup();
  openDialog(
    `<span class="pill">A WINDOW INTO THEIR LEARNING</span><h2 id="dialog-title">Little steps, real discoveries</h2><p class="muted">A Foundation-level curriculum sample. Progress stays in this browser; no account or backend is connected.</p><div class="learning-summary"><strong>${state.done.length}/3 <small>story activities</small></strong><strong>${state.evidence.length}<small>completed practice sessions</small></strong></div><div class="skill-list">${skillSummary(
      state,
    )
      .map(
        (s) =>
          `<article><h3>${s.name}</h3><p>${s.goal}</p><div><span>${s.independent} without hints</span><span>${s.supported} with support</span></div></article>`,
      )
      .join(
        '',
      )}</div><p class="evidence-note">These are practice observations, not a mastery score. Hints and “do it together” are recorded as support. Replays keep the same quantities, so repeated success does not demonstrate transfer.</p><details><summary>What gets recorded?</summary><p>Each completed session stores the skill, encounter, representation, answer-check attempts, hint count, assisted completion, and completion time. Exiting an activity does not record completion. Story rewards are earned with or without support.</p><p>Activities are authored and fixed for this milestone. After two unsuccessful checks, feedback encourages a hint. Adaptive variants and AI are future work.</p></details><div class="grownup-actions"><button class="primary" id="export">Download learning record</button><button class="secondary danger" id="reset">Reset adventure</button></div><p class="tiny-note">${available ? 'Saved on this device. Clearing browser data removes this record.' : 'Saving is unavailable. Download the record before closing this page.'}</p>`,
  );
}
function exportData() {
  const a = document.createElement('a');
  const url = URL.createObjectURL(
    new Blob([JSON.stringify({ exportedAt: new Date().toISOString(), ...state }, null, 2)], {
      type: 'application/json',
    }),
  );
  a.href = url;
  a.download = 'starwhisker-learning-record.json';
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  toast('Learning record downloaded.');
}
document.addEventListener('click', (event) => {
  if (!(event.target instanceof Element)) return;
  const button = event.target.closest('button');
  if (!button) return;
  if (backend?.working) return;
  if (button.classList.contains('close')) return closeDialog();
  if (button.id === 'explore') return explore();
  if (isMapId(button.dataset.place)) {
    state.region = button.dataset.place;
    state.position = { x: 160, y: 490 };
    closeDialog();
    render();
    game.events.emit('refresh');
    toast(`Welcome to ${maps[state.region].name}!`);
    return;
  }
  if (button.dataset.visit) {
    const e = activeEncounters.find((e) => e.id === button.dataset.visit);
    if (!e) return;
    if (e.type === 'pattern' && !state.done.includes('meadow-bridge'))
      return toast('Help Pip fix the bridge first. The garden is across the river.');
    if (dialog.open) closeDialog();
    game.events.emit('visit', e);
    return;
  }
  if (button.dataset.hero) {
    if (!Object.hasOwn(heroes, button.dataset.hero)) return;
    state.hero = button.dataset.hero;
    render();
    game.events.emit('refresh');
    dialog.close();
    journal();
    return;
  }
  if (button.dataset.plate !== undefined && session?.kind === 'share') {
    const n = +button.dataset.plate;
    if (!Number.isInteger(n) || n < 0 || n >= session.value.length) return;
    if (session.value.reduce((a, b) => a + b, 0) < session.encounter.total) session.value[n]++;
    session.feedback = '';
    drawActivity();
    return;
  }
  if (button.dataset.remove !== undefined && session?.kind === 'share') {
    const n = +button.dataset.remove;
    if (!Number.isInteger(n) || n < 0 || n >= session.value.length) return;
    session.value[n] = Math.max(0, session.value[n] - 1);
    session.feedback = '';
    drawActivity();
    return;
  }
  switch (button.id) {
    case 'missions':
      missions();
      break;
    case 'fullscreen':
      toggleFullscreen();
      break;
    case 'journal':
    case 'open-journal':
      if (dialog.open) closeDialog();
      journal();
      break;
    case 'grownup':
      grownup();
      break;
    case 'plus':
      if (!session || session.kind === 'share') return;
      session.value = Math.min(session.value + 1, 10);
      session.feedback = '';
      drawActivity();
      break;
    case 'minus':
      if (!session || session.kind === 'share') return;
      session.value = Math.max(0, session.value - 1);
      session.feedback = '';
      drawActivity();
      break;
    case 'hint':
      if (!session) return;
      session.hints = Math.min(3, session.hints + 1);
      session.feedback = '';
      session.hintText = hints[session.encounter.type][session.hints - 1];
      drawActivity();
      break;
    case 'together':
      if (!session) return;
      session.assisted = true;
      if (session.kind === 'share')
        session.value = Array<number>(session.encounter.groups).fill(answerFor(session.encounter));
      else session.value = answerFor(session.encounter);
      session.feedback = '';
      session.hintText = hints[session.encounter.type][2] + ' Count with Pip, then check our idea.';
      drawActivity();
      break;
    case 'check':
      check();
      break;
    case 'read-prompt':
      if (!session) return;
      speak(session.encounter.prompt);
      break;
    case 'return-world': {
      const all = state.done.length === 3;
      closeDialog();
      if (all) celebration();
      break;
    }
    case 'keep-exploring':
      closeDialog();
      break;
    case 'celebrate':
      if (dialog.open) closeDialog();
      celebration();
      break;
    case 'save-name': {
      const name = $<HTMLInputElement>('#companion-name').value.trim();
      if (!name) return toast('Choose a name for your friend.');
      state.companionName = name.slice(0, 20);
      render();
      toast('Your friend’s name is saved.');
      break;
    }
    case 'export':
      exportData();
      break;
    case 'reset':
      dialog.innerHTML = `<h2 id="dialog-title">Start a fresh adventure?</h2><p>This removes the saved stars, companion name, and learning record from this browser. You can download the record first.</p><div class="grownup-actions"><button class="primary" id="cancel-reset">Keep my adventure</button><button class="secondary danger" id="confirm-reset">Reset everything</button></div>`;
      $('#cancel-reset').focus();
      break;
    case 'cancel-reset':
      dialog.close();
      grownup();
      break;
    case 'confirm-reset':
      state = freshState();
      closeDialog();
      render();
      game.events.emit('reset-position');
      game.events.emit('refresh');
      toast('A fresh adventure is ready.');
      break;
  }
});
async function boot() {
  const tools = document.querySelectorAll<HTMLButtonElement>('.game-hud button, .game-dock button');
  tools.forEach((button) => {
    button.disabled = true;
  });
  const guestOnly = new URLSearchParams(location.search).get('guest') === '1';
  if (import.meta.env.DEV && !guestOnly) {
    $('#save-status').textContent = 'Connecting to your adventure…';
    try {
      backend = await connectBackend({
        open: openDialog,
        close: closeDialog,
        toast,
        visit: (quest) => game.events.emit('visit', quest),
        apply: (snapshot) => {
          state.done = completedWorldQuests(snapshot);
          render();
          game?.events.emit('refresh');
        },
      });
    } catch {
      $('#save-status').textContent = 'The local server is unavailable. Reload to reconnect.';
      openDialog(
        '<h2 id="dialog-title">Let’s reconnect your adventure</h2><p>The local server could not load your progress. Start the local demo, then reload this page.</p><button class="primary" id="reconnect-world">Reconnect</button>',
      );
      $('#reconnect-world').onclick = () => location.reload();
      return;
    }
  }
  if (backend) {
    state = freshState();
    state.done = completedWorldQuests(backend.snapshot);
    backend.restoreView(state);
  }
  render();
  game = createWorld(
    'game',
    () => state,
    (quest) => {
      if (backend) backend.encounter(quest);
      else {
        const encounter = activeEncounters.find((e) => e.id === quest.id);
        if (encounter) startActivity(encounter);
      }
    },
    () =>
      backend
        ? backend.missions()
        : state.done.length === 3
          ? celebration()
          : toast('Three helping-paw stars will make a wonderful friendship!'),
    (position) => {
      state.position = position;
      persist();
    },
    (region, position) => {
      state.region = region;
      state.position = position;
      render();
      toast(`Welcome to ${maps[region].name}! Follow the paths to explore.`);
    },
    backend
      ? { encounters: () => backend!.markers, companions: () => backend!.companions() }
      : undefined,
  );
  tools.forEach((button) => {
    button.disabled = false;
  });
  if (backend)
    toast(
      backend.snapshot.pendingEncounters.length
        ? 'Welcome back! Open Missions to continue your activity.'
        : 'Find Pip, or open Missions to start your adventure.',
    );
}
void boot();
window.addEventListener('pagehide', () => {
  if (game) persist();
});
