import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  EncounterInstance,
  PersonalQuestOffer,
  PersonalQuestView,
} from '../../contracts/generated/contracts.ts';
import {
  activePersonalQuest,
  personalQuestDetails,
  personalQuestLibrary,
  personalQuestOffers,
  personalQuestScene,
} from './personal-quests.ts';
import { authoredPresentation } from './learning/presentation.ts';

const offer = (available = true): PersonalQuestOffer => ({
  questId: 'pip-lantern-trail',
  documentVersion: '1',
  companionId: 'pip',
  title: 'Pip’s lantern trail',
  minimumCharacterLevel: 4,
  available,
  completed: false,
});
const quest = (): PersonalQuestView => ({
  questId: 'pip-lantern-trail',
  documentVersion: '1',
  episodeId: 'personal-1',
  companionId: 'pip',
  title: 'Pip’s lantern trail',
  node: {
    id: 'which-way',
    type: 'choice',
    text: 'A lantern glows beside the path. Which way shall we look?',
    choices: [
      { id: 'pond', label: 'Look by the pond', nextNodeId: 'pond-help' },
      { id: 'trees', label: 'Explore the trees', nextNodeId: 'tree-help' },
    ],
  },
});

test('Older snapshots and unowned companions cannot invent personal adventures', () => {
  assert.equal(personalQuestDetails('pip', { ownedCreatures: ['pip'] }), '');
  assert.equal(personalQuestScene({ ownedCreatures: ['pip'] }), undefined);
  const unowned = {
    ownedCreatures: [],
    personalQuestOffers: [offer()],
    personalQuest: quest(),
  };
  assert.deepEqual(personalQuestOffers(unowned), []);
  assert.equal(activePersonalQuest(unowned), undefined);
  assert.equal(personalQuestDetails('pip', unowned), '');
  assert.doesNotMatch(personalQuestLibrary(unowned), /data-server-personal-quest=/);
});

test('The journal uses confirmed availability and shows the authored unlock requirement', () => {
  const snapshot = { ownedCreatures: ['pip'], personalQuestOffers: [offer(false)] };
  assert.match(personalQuestDetails('pip', snapshot), /Unlock at character level 4/);
  assert.match(personalQuestDetails('pip', snapshot), /disabled/);
  assert.doesNotMatch(personalQuestDetails('pip', snapshot), /Start Pip/);
  snapshot.personalQuestOffers = [offer()];
  assert.match(personalQuestDetails('pip', snapshot), /Start Pip/);
  assert.doesNotMatch(personalQuestDetails('pip', snapshot), /disabled/);
});

test('A current episode renders meaningful alternatives and prevents replacing it implicitly', () => {
  const snapshot = {
    ownedCreatures: ['pip'],
    personalQuestOffers: [offer()],
    personalQuest: quest(),
  };
  const before = structuredClone(snapshot);
  const html = personalQuestScene(snapshot)!;
  assert.match(html, /data-server-personal-choice="pond"/);
  assert.match(html, /data-server-personal-choice="trees"/);
  assert.match(html, /Look by the pond/);
  assert.match(html, /Explore the trees/);
  assert.doesNotMatch(html, /server-personal-activity/);
  assert.match(html, /Closing this window keeps your place/);
  assert.match(personalQuestLibrary(snapshot), /Continue Pip/);
  assert.doesNotMatch(personalQuestLibrary(snapshot), /data-server-personal-quest=/);
  assert.deepEqual(snapshot, before);
});

test('Activities launch through server controls, and handoffs continue through the authored bank', () => {
  const current = quest();
  current.node = {
    id: 'pond-help',
    type: 'activity',
    text: 'Help the friends make a little path.',
    activitySlotId: 'pip-number-help',
    nextNodeId: 'reflection',
  };
  const snapshot = {
    ownedCreatures: ['pip'],
    personalQuestOffers: [offer()],
    personalQuest: current,
  };
  assert.match(personalQuestScene(snapshot)!, /id="server-personal-activity"/);
  assert.match(personalQuestScene(snapshot)!, /Help Pip together/);
  assert.doesNotMatch(personalQuestScene(snapshot)!, /data-server-personal-choice/);
  current.node = {
    id: 'another-day',
    type: 'handoff',
    text: 'We can look for another light next time.',
    continuationTags: ['night-sky', 'friendly-paths'],
  };
  snapshot.personalQuestOffers[0].completed = true;
  const handoff = personalQuestScene(snapshot)!;
  assert.match(handoff, /What shall we explore next\?/);
  assert.match(handoff, /Revisit Pip/);
  assert.match(handoff, /without another keepsake/);
  assert.match(handoff, /night sky · friendly paths/);
  assert.doesNotMatch(handoff, /game over|the end|server-personal-abandon/i);
});

test('Quest narration, titles, identifiers and choice labels render as escaped text', () => {
  const current = quest();
  current.title = '<script>alert(1)</script>';
  current.node.text = '<img src=x onerror=alert(1)>';
  if (current.node.type === 'choice') {
    current.node.choices[0].id = '"><img src=x>';
    current.node.choices[0].label = '<b>Choose me</b>';
  }
  const html = personalQuestScene({ ownedCreatures: ['pip'], personalQuest: current })!;
  assert.doesNotMatch(html, /<script|<img|<b>/);
  assert.match(html, /&lt;script&gt;/);
  assert.match(html, /&lt;b&gt;Choose me&lt;\/b&gt;/);
  assert.match(html, /data-server-personal-choice="&quot;&gt;&lt;img/);
});

test('Personal activities retain skill meaning without unrelated bridge narration', () => {
  const encounter: EncounterInstance = {
    schemaVersion: '1.0',
    id: 'enc-personal',
    contentVersion: 'test',
    skillId: 'M06',
    templateId: 'missing_quantity',
    parameters: { total: 5, given: 2 },
    permittedActions: ['attempt', 'hint', 'abandon'],
    support: { hintCount: 0, workedExample: false, readAloud: false },
    personalQuest: {
      questId: 'pip-lantern-trail',
      documentVersion: '1',
      nodeId: 'pond-help',
      episodeId: 'personal-1',
    },
  };
  assert.match(authoredPresentation(encounter)!.prompt, /make 5 altogether/);
  assert.doesNotMatch(authoredPresentation(encounter)!.prompt, /bridge/);
  delete encounter.personalQuest;
  assert.equal(authoredPresentation(encounter)!.prompt, 'Help Pip finish the bridge!');
});

test('Continuity acknowledgements and recommendation text are escaped and interests remain explicit', () => {
  const state = {
    ownedCreatures: ['pip'],
    questInterests: ['night-sky'],
    questInterestOptions: ['night-sky', 'woodland-trails'],
    personalQuestOffers: [
      {
        questId: 'followup',
        documentVersion: '1',
        title: 'Follow up',
        companionId: 'pip',
        minimumCharacterLevel: 4,
        available: false,
        completed: false,
        lockedReason: 'Follow a matching clue first',
      },
    ],
  };
  const library = personalQuestLibrary(state);
  assert.match(library, /Follow a matching clue first/);
  assert.match(library, /data-server-quest-interest="night-sky" aria-pressed="true"/);
  const scene = personalQuestScene({
    ...state,
    personalQuest: {
      questId: 'followup',
      documentVersion: '1',
      episodeId: 'episode-2',
      title: 'Follow up',
      companionId: 'pip',
      acknowledgement: 'You chose <img src=x>.',
      node: {
        id: 'start',
        type: 'choice',
        text: 'A path',
        choices: [
          { id: 'a', label: 'One', nextNodeId: 'x' },
          { id: 'b', label: 'Two', nextNodeId: 'y' },
        ],
      },
    },
  });
  assert.match(scene ?? '', /You chose &lt;img/);
  assert.doesNotMatch(scene ?? '', /<img src=x/);
});
