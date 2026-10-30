import { sceneSourceLabel } from './source-status.ts';
import { nextStoryLabel } from './world-model.ts';
import type { StorySnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import { isRecord } from '../types.ts';
import { StoryClient, StorySession } from './client.ts';
import './story.css';

const root = document.querySelector<HTMLElement>('#story')!;
function element<K extends keyof HTMLElementTagNameMap>(tag: K, text = '', className = '') {
  const node = document.createElement(tag);
  node.textContent = text;
  node.className = className;
  return node;
}
const header = element('header', '', 'story-header');
header.append(element('p', 'WONDERWILDS · STORY WORKSHOP', 'eyebrow'));
header.append(element('h1', 'Adventures with Pip'));
header.append(
  element('p', 'Follow connected stories, help your friends, and see the world change.'),
);
const home = element('a', 'Return to the adventure');
home.href = '/';
header.append(home);
const status = element('p', 'Connecting to your story…', 'story-status');
status.id = 'story-status';
status.setAttribute('role', 'status');
const sourceStatus = element('p', 'Checking story source…', 'story-note');
sourceStatus.id = 'story-source-status';
sourceStatus.setAttribute('role', 'status');
const controls = element('nav', '', 'story-toolbar');
controls.setAttribute('aria-label', 'Story connection');
const content = element('div');
const footer = element(
  'p',
  'Local development story · saved separately from your adventure and activities.',
  'story-note',
);
root.append(header, status, sourceStatus, controls, content, footer);
let session: StorySession | undefined;
let client: StoryClient | undefined;
let destroyed = false;

function button(text: string, action: () => Promise<unknown>, disabled = false) {
  const result = element('button', text);
  result.type = 'button';
  result.disabled = disabled;
  result.onclick = () => void run(action);
  return result;
}
function showError(error: unknown) {
  if (destroyed) return;
  if (!session?.snapshot) sourceStatus.textContent = 'Story source unavailable';
  status.textContent =
    error instanceof ApiFailure && error.code === 'REVISION_CONFLICT'
      ? 'The story changed in another tab. Refresh progress before choosing again.'
      : error instanceof ApiFailure && ['UNAUTHENTICATED', 'FORBIDDEN'].includes(error.code)
        ? 'This story session is unavailable. Restart the local demo and refresh this page.'
        : session?.hasPending
          ? 'We could not confirm the save. Retry the same action to find out what happened.'
          : 'The story could not connect. Refresh progress to try again. Local demo setup is required.';
}
function renderControls() {
  controls.replaceChildren();
  if (!session) return;
  controls.append(button('Refresh progress', () => session!.refresh(), session.busy));
  if (session.hasPending)
    controls.append(
      button(
        'Retry the same action',
        async () => {
          await session!.retry();
          return session!.refresh();
        },
        session.busy,
      ),
    );
}
async function run(action: () => Promise<unknown>) {
  if (destroyed || session?.busy) return;
  let work: Promise<unknown>;
  try {
    work = action();
    status.textContent = 'Saving your place…';
    render();
    await work;
    if (destroyed) return;
    render();
    connected();
    content.querySelector<HTMLElement>('h2')?.focus();
  } catch (error) {
    render();
    showError(error);
  }
}
function connected() {
  if (!session?.snapshot) return;
  status.textContent = session.hasPending
    ? 'A previous action still needs confirmation. Retry it before choosing another.'
    : `Story saved · revision ${session.snapshot.revision}`;
}
const locations: Record<string, string> = {
  bridge_bank: 'The bridge bank',
  workshop: 'The woodcutter’s workshop',
  far_bank: 'The far bank',
  cove: 'Starlight Cove',
  fern_hollow: 'Fern Hollow',
  mossy_clearing: 'Mossy Clearing',
  owl_grove: 'Owl Grove',
  acorn_trail: 'Acorn Trail',
};
function render() {
  renderControls();
  const snapshot = session?.snapshot;
  if (!snapshot) return;
  sourceStatus.textContent = sceneSourceLabel(snapshot);
  content.replaceChildren();
  if (snapshot.commandBudgetExhausted || !snapshot.sceneInput || !snapshot.scene) {
    const terminal = element('section', '', 'story-card');
    terminal.append(element('h2', 'This development story has reached its save limit'));
    terminal.append(
      element('p', 'Your saved progress is preserved. Return to the adventure or stop here.'),
    );
    content.append(terminal);
    return;
  }
  renderSnapshot(snapshot);
}
function renderSnapshot(snapshot: StorySnapshot) {
  const input = snapshot.sceneInput!;
  const scene = snapshot.scene!;
  const disabled = Boolean(session?.busy || session?.hasPending || session?.needsRefresh);
  const journey = element('ol', '', 'story-map');
  journey.setAttribute('aria-label', 'Story places');
  for (const [id, label] of Object.entries(locations)) {
    const place = element('li', label);
    if (id === snapshot.locationId) place.setAttribute('aria-current', 'location');
    journey.append(place);
  }
  content.append(journey);
  const columns = element('div', '', 'story-columns');
  const scenePanel = element('section', '', `story-card scene-${snapshot.locationId}`);
  const heading = element('h2', locations[snapshot.locationId] ?? snapshot.locationId);
  heading.tabIndex = -1;
  scenePanel.append(heading, element('p', input.here.description, 'location-description'));
  for (const entry of scene.presentation) {
    const line = element('div', '', 'story-line');
    if (entry.type === 'dialogue')
      line.append(element('strong', entry.speakerId === 'pip' ? 'Pip' : 'The woodcutter'));
    line.append(element('p', entry.text));
    scenePanel.append(line);
  }
  const actions = element('div', '', 'story-actions');
  const acknowledge = snapshot.acknowledgeableEntryIds[0];
  if (acknowledge) {
    actions.append(
      button(
        'Continue',
        async () => {
          await session!.choose({
            type: 'acknowledge',
            sceneId: scene.sceneId,
            entryId: acknowledge,
          });
          return session!.refresh();
        },
        disabled,
      ),
    );
  } else {
    // Keep eligible choices intact, but place pausing after ways to move the story on.
    const choices = [...scene.choices].sort((a, b) => {
      const isPause = (actionId: string) =>
        input.availableActions.find((candidate) => candidate.id === actionId)?.action.type ===
        'cancel_mission';
      return Number(isPause(a.actionId)) - Number(isPause(b.actionId));
    });
    for (const choice of choices) {
      const action = input.availableActions.find((candidate) => candidate.id === choice.actionId);
      if (!action) continue;
      const control = button(
        choice.label,
        async () => {
          await session!.choose({ type: 'choose', actionId: choice.actionId });
          return session!.refresh();
        },
        disabled,
      );
      control.dataset.storyAction = action.action.type;
      if (
        ['accept_mission', 'collect', 'exchange', 'assemble', 'accompany'].includes(
          action.action.type,
        )
      )
        control.classList.add('story-action-primary');
      if ('destinationId' in action.action)
        control.dataset.destination = action.action.destinationId;
      if ('sourceId' in action.action) control.dataset.source = action.action.sourceId;
      actions.append(control);
    }
  }
  scenePanel.append(actions);
  columns.append(scenePanel);
  const aside = element('aside', '', 'story-card story-progress');
  aside.append(element('h2', 'Your mission'));
  const mission = input.mission;
  if (mission) {
    aside.append(element('h3', mission.goal));
    aside.append(element('p', mission.narrative.premise));
    const state = element(
      'p',
      mission.status === 'completed'
        ? `Mission complete! ${mission.narrative.payoff}`
        : mission.status === 'abandoned'
          ? 'Mission paused. Your progress is saved.'
          : 'One step at a time.',
      'mission-state',
    );
    state.dataset.missionStatus = mission.status;
    aside.append(state);
    const milestones = element('ol', '', 'milestones');
    for (const label of mission.completed)
      milestones.append(element('li', `✓ ${label}`, 'complete'));
    if (mission.current) {
      const current = element('li', mission.current, 'current');
      current.setAttribute('aria-current', 'step');
      milestones.append(current);
    }
    for (const label of mission.remaining) milestones.append(element('li', label));
    aside.append(milestones);
  } else {
    aside.append(element('p', 'Pip has an idea. Accept the mission when you’re ready.'));
  }
  const next = nextStoryLabel(snapshot);
  if (next) aside.append(element('p', next, 'next-story'));
  if (input.context.memory.recentMissions.length) {
    aside.append(element('h3', 'Our adventures so far'));
    const history = element('ul');
    for (const memory of input.context.memory.recentMissions) history.append(element('li', memory));
    aside.append(history);
  }
  aside.append(element('h3', 'What you’re carrying'));
  const inventory = element('ul', '', 'inventory');
  inventory.id = 'story-inventory';
  for (const item of input.context.inventory)
    inventory.append(element('li', `${item.quantity} ${item.itemId}`));
  if (!input.context.inventory.length) inventory.append(element('li', 'Your bag is empty.'));
  aside.append(inventory);
  columns.append(aside);
  content.append(columns);
}
async function start() {
  try {
    const response = await fetch('/__demo/bootstrap', {
      signal: AbortSignal.timeout(10000),
      redirect: 'error',
    });
    if (!response.ok) throw new Error('No local demo.');
    const bootstrap: unknown = await response.json();
    if (
      !isRecord(bootstrap) ||
      typeof bootstrap.profileId !== 'string' ||
      !/^[a-f0-9-]{36}$/.test(bootstrap.profileId) ||
      typeof bootstrap.csrf !== 'string' ||
      bootstrap.csrf.length < 32 ||
      bootstrap.csrf.length > 256
    )
      throw new Error('Invalid bootstrap.');
    if (destroyed) return;
    client = new StoryClient(bootstrap.profileId, bootstrap.csrf);
    session = new StorySession(client, bootstrap.profileId, sessionStorage);
    await session.refresh();
    if (destroyed) return;
    render();
    connected();
  } catch (error) {
    showError(error);
    renderControls();
  }
}
window.addEventListener('pagehide', () => {
  destroyed = true;
  client?.destroy();
});
// A restored page has a deliberately destroyed client; fetch fresh session context.
window.addEventListener('pageshow', (event) => {
  if (event.persisted) window.location.reload();
});
void start();
