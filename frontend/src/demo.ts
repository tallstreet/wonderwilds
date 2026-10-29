import { ApiFailure, GameClient } from './api/client.ts';
import { CommandSession } from './api/command-session.ts';
import type { CommandPayload, GameSnapshot } from '../../contracts/generated/contracts.ts';
import { mountActivity } from './learning/activity.ts';
import { authoredPresentation } from './learning/presentation.ts';
import story from '../../content/story/adventure.v1.json';
import { meadowFriends } from './backend-world-model.ts';
import './demo.css';

const root = document.querySelector<HTMLElement>('#demo')!;
function element<K extends keyof HTMLElementTagNameMap>(tag: K, text = '', className = '') {
  const node = document.createElement(tag);
  node.textContent = text;
  node.className = className;
  return node;
}
const header = element('header');
header.append(
  element('p', 'WONDERWILDS · LOCAL DEVELOPMENT'),
  element('h1', 'Your next adventure'),
);
header.append(
  element(
    'p',
    'Explore the server-backed Meadow story, make friends, and inspect learning progress. Draft content · synthetic local profile.',
  ),
);
const toolbar = element('nav', '', 'demo-toolbar');
const guest = element('a', 'Return to the guest world');
guest.href = '/';
toolbar.append(guest);
const status = element('p', 'Connecting to the local backend…', 'demo-status');
status.setAttribute('role', 'status');
const content = element('div');
root.append(header, toolbar, status, content);
let csrf: string | undefined;
const client = new GameClient(() => csrf);
let snapshot: GameSnapshot;
let activity: ReturnType<typeof mountActivity> | undefined;
let commands: CommandSession | undefined;
function button(label: string, action: () => Promise<void>, disabled = false) {
  const node = element('button', label);
  node.disabled = disabled;
  node.onclick = () => {
    if (commands && !commands.busy) void action().catch(showError);
  };
  return node;
}
function showError(error: unknown) {
  status.textContent =
    error instanceof ApiFailure && error.code === 'REVISION_CONFLICT'
      ? 'Progress changed in another tab. Refresh progress before choosing your next action.'
      : error instanceof ApiFailure && error.code === 'FORBIDDEN'
        ? 'This activity is still locked. Complete its prerequisites first.'
        : 'The backend could not finish that action. Retry the same action, or refresh progress to reconnect.';
}
async function send(payload: CommandPayload) {
  if (!commands) throw new Error('Adventure is still connecting');
  try {
    snapshot = await commands.send(payload);
  } catch (error) {
    showError(error);
    throw error;
  }
}
async function refresh() {
  if (!commands || commands.busy) return;
  snapshot = await commands.refresh();
  render();
  connected();
}
function connected() {
  status.textContent = `Connected · saved in PostgreSQL · revision ${snapshot.revision}`;
}
toolbar.append(button('Refresh progress', refresh));
const completed = (id: string) => {
  const beat = story.beats.find((b) => b.id === id);
  return Boolean(beat && beat.completionFlags.every((flag) => snapshot.storyFlags.includes(flag)));
};
function render() {
  activity?.destroy();
  activity = undefined;
  content.replaceChildren();
  const encounter = snapshot.pendingEncounters[0];
  if (encounter) {
    const presentation = authoredPresentation(encounter);
    if (!presentation) {
      content.append(element('p', 'This activity has no supported presentation.'));
      return;
    }
    const host = element('section');
    content.append(host);
    activity = mountActivity(
      host,
      encounter,
      {
        async onAttempt(answer) {
          await send({ type: 'attempt', encounterId: encounter.id, answer });
          if (!snapshot.pendingEncounters.some((e) => e.id === encounter.id)) {
            render();
            status.textContent =
              'Activity completed! Your story rewards and learning record are saved.';
            return { message: 'Completed', result: 'correct' };
          }
          return {
            message: 'Not quite yet. Change your answer and try again.',
            result: 'incorrect',
          };
        },
        async onSupport(kind) {
          await send({ type: 'hint', encounterId: encounter.id, supportKind: kind });
          return {
            message: `${presentation.prompt} Take one step at a time with a helper. This support has been recorded.`,
          };
        },
        async onAbandon() {
          await send({ type: 'abandon', encounterId: encounter.id });
          render();
          status.textContent =
            'Activity left. You can choose another adventure; no completion reward was granted.';
        },
      },
      presentation,
    );
    activity.focus();
  } else {
    content.append(element('h2', 'Clover Meadow story'));
    const grid = element('div', '', 'demo-grid');
    for (const beat of story.beats.filter((b) => b.chapterId === 'meadow')) {
      const done = completed(beat.id);
      const ready =
        [...beat.requiresBeatIds, ...beat.finaleSteps].every(completed) &&
        beat.requiredCreatureIds.every((id) => snapshot.ownedCreatures.includes(id));
      const card = element('article', '', 'demo-card');
      card.append(
        element('h3', beat.title),
        element('p', beat.dialogue),
        element(
          'p',
          `${beat.skillId} · ${done ? 'Completed' : ready ? 'Ready to play' : 'Complete earlier story activities to unlock'}`,
          'demo-meta',
        ),
      );
      card.append(
        button(
          done ? 'Completed' : 'Start activity',
          async () => {
            await send({ type: 'start_encounter', beatId: beat.id });
            render();
          },
          done || !ready,
        ),
      );
      grid.append(card);
    }
    content.append(grid, element('h2', 'Make friends'));
    const friends = element('div', '', 'demo-grid');
    for (const quest of meadowFriends) {
      const owned = snapshot.ownedCreatures.includes(quest.creatureId);
      const ready = quest.requiresBeatIds.every(completed);
      const card = element('article', '', 'demo-card');
      card.append(element('h3', quest.title), element('p', quest.prompt));
      card.append(
        element(
          'p',
          owned
            ? 'Your friend'
            : ready
              ? 'Ready to help'
              : `Unlock after ${quest.requiresBeatIds.map((id) => story.beats.find((b) => b.id === id)?.title ?? id).join(', ')}`,
          'demo-meta',
        ),
      );
      card.append(
        button(
          owned ? 'Choose as companion' : 'Help this friend',
          async () => {
            await send(
              owned
                ? { type: 'select_companion', creatureId: quest.creatureId }
                : { type: 'start_encounter', beatId: quest.id },
            );
            render();
            if (owned)
              status.textContent = `${quest.creatureId} selected as your server-side companion. World rendering is still pending.`;
          },
          !owned && !ready,
        ),
      );
      friends.append(card);
    }
    content.append(friends);
  }
  const progress = element('section', '', 'demo-panel');
  progress.append(
    element('h2', 'Learning and rewards'),
    element(
      'p',
      `Saved revision ${snapshot.revision} · Friends: ${snapshot.ownedCreatures.join(', ') || 'none yet'}`,
    ),
  );
  progress.append(
    element(
      'p',
      `Inventory: ${snapshot.inventory.map((i) => `${i.itemId} × ${i.quantity}`).join(', ') || 'empty'}`,
    ),
  );
  for (const summary of snapshot.learnerSummaries)
    progress.append(element('p', `${summary.skillId} · ${summary.status}: ${summary.explanation}`));
  if (!snapshot.learnerSummaries.length)
    progress.append(
      element('p', 'Complete an activity to see server-assessed learning observations.'),
    );
  const details = element('details');
  details.append(
    element('summary', 'Inspect saved server snapshot'),
    element('pre', JSON.stringify(snapshot, null, 2)),
  );
  progress.append(details);
  content.append(progress);
  const limits = element('section', '', 'demo-panel');
  limits.append(
    element('h2', 'What is available here'),
    element(
      'p',
      'This demo connects the existing Meadow API, assessment, recorded support, abandonment, friendship rewards, companion selection and learning summaries. Progress survives reloads and server restarts. Guest saves remain separate.',
    ),
    element(
      'p',
      'Orchard/Cove travel, world movement sync, transformations, the director, account login, save import, and parent lifecycle screens still need API or browser integration. They are not playable here. Content and mastery policies remain experimental; no live AI is connected.',
    ),
  );
  content.append(limits);
}
async function start() {
  const response = await fetch('/__demo/bootstrap');
  if (!response.ok || !response.headers.get('content-type')?.includes('application/json'))
    throw new Error('Demo not started');
  const session = (await response.json()) as { profileId: string; csrf: string };
  csrf = session.csrf;
  client.selectProfile(session.profileId);
  snapshot = await client.snapshot();
  commands = new CommandSession(client, snapshot);
  render();
  connected();
}
void start().catch(() => {
  status.textContent =
    'Start the backend demo with: sh scripts/local-demo.sh, then reload this page.';
});
