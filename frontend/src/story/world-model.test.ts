import assert from 'node:assert/strict';
import test from 'node:test';
import type { StorySceneInput, StorySnapshot } from '../../../contracts/generated/contracts.ts';
import workshop from '../../../backend/crates/director/src/story/fixtures/scene-workshop.json';
import {
  actionTarget,
  bridgeRepaired,
  storyBounds,
  storyMarkers,
  storyPlaces,
} from './world-model.ts';
import { findRoute, isBlocked } from '../navigation.ts';
function snapshot(locationId: StorySnapshot['locationId']): StorySnapshot {
  return {
    schemaVersion: '1.0',
    contentVersion: 'pip-bridge-1',
    revision: 3,
    locationId,
    commandBudgetExhausted: false,
    acknowledgeableEntryIds: [],
    sceneInput: structuredClone(workshop) as StorySceneInput,
    scene: { sceneId: 'scene', presentation: [], choices: [] },
  };
}
test('story art registry renders only present NPCs and legal resource/route handles', () => {
  const value = snapshot('workshop');
  const markers = storyMarkers(value);
  assert.ok(markers.some((marker) => marker.id === 'npc:woodcutter' && marker.sprite === 'cleo'));
  assert.ok(!markers.some((marker) => marker.id === 'npc:pip'));
  assert.ok(
    markers.some((marker) => marker.id === 'resource:spare_wood' && marker.prop === 'plank'),
  );
  assert.ok(!markers.some((marker) => marker.id === 'resource:fallen_wood'));
  value.sceneInput!.availableActions = [];
  assert.deepEqual(
    storyMarkers(value).map((marker) => marker.id),
    ['npc:woodcutter'],
  );
});
test('all authored marker approaches are reachable inside their authoritative location', () => {
  for (const location of ['bridge_bank', 'workshop', 'far_bank'] as const) {
    const value = snapshot(location);
    value.sceneInput!.here.characters = [
      { id: location === 'workshop' ? 'woodcutter' : 'pip', description: '', relationship: '' },
    ];
    value.sceneInput!.availableActions = [
      {
        id: 'one',
        description: '',
        action: {
          type: 'collect',
          sourceId: location === 'workshop' ? 'spare_wood' : 'fallen_wood',
          itemId: 'wood',
          quantity: 3,
        },
      },
      {
        id: 'two',
        description: '',
        action: {
          type: 'travel',
          destinationId: location === 'bridge_bank' ? 'workshop' : 'bridge_bank',
        },
      },
    ];
    if (location === 'far_bank') value.sceneInput!.availableActions.shift();
    for (const marker of storyMarkers(value)) {
      const target = { x: marker.x - 30, y: marker.y + 25 };
      const route = findRoute(
        storyPlaces[location].spawn,
        target,
        true,
        storyPlaces[location].region,
        storyBounds(location),
      );
      assert.ok(route.length, marker.id);
      const last = route.at(-1)!;
      assert.ok(Math.hypot(last.x - marker.x, last.y - marker.y) < 110, marker.id);
      assert.ok(route.every((point) => !storyBounds(location)(point.x, point.y)));
    }
  }
});
test('bank bounds hold after repair; confirmed crossing can use measured real bridge deck', () => {
  const value = snapshot('bridge_bank');
  assert.equal(bridgeRepaired(value), false);
  value.sceneInput!.context.memory.facts = ['The bridge is repaired.'];
  assert.equal(bridgeRepaired(value), true);
  assert.equal(storyBounds('bridge_bank')(650, 359), true);
  assert.equal(storyBounds('far_bank')(600, 359), true);
  const route = findRoute(
    { x: 560, y: 359 },
    { x: 850, y: 359 },
    true,
    'meadow',
    storyBounds('far_bank', true),
  );
  assert.ok(route.length);
  assert.ok(route.every((point) => !isBlocked(point.x, point.y, true)));
  assert.ok(route.some((point) => point.x > 780));
});
test('action bindings associate resources, recipes, NPCs and routes with their world markers', () => {
  assert.equal(
    actionTarget({ type: 'accompany', characterId: 'pip', destinationId: 'far_bank' }),
    'npc:pip',
  );
  assert.equal(
    actionTarget({ type: 'assemble', objectId: 'bridge', recipeId: 'repair_bridge' }),
    'object:bridge',
  );
  assert.equal(
    actionTarget({ type: 'exchange', characterId: 'woodcutter', recipeId: 'wood_to_planks' }),
    'npc:woodcutter',
  );
  assert.equal(actionTarget({ type: 'travel', destinationId: 'workshop' }), 'route:workshop');
});

test('series resource and assembly placements remain reachable and unknown IDs are not invented', () => {
  for (const [location, sourceId, objectId] of [
    ['far_bank', 'berry_bush', 'far_lantern'],
    ['far_bank', 'glow_stones', 'far_lantern'],
    ['bridge_bank', 'unknown_source', 'picnic_table'],
    ['bridge_bank', 'unknown_source', 'near_lanterns'],
  ] as const) {
    const value = snapshot(location);
    value.sceneInput!.here.characters = [];
    value.sceneInput!.availableActions = [
      {
        id: 'collect',
        description: '',
        action: { type: 'collect', sourceId, itemId: 'test', quantity: 3 },
      },
      { id: 'assemble', description: '', action: { type: 'assemble', objectId, recipeId: 'test' } },
    ];
    const markers = storyMarkers(value);
    assert.ok(!markers.some((marker) => marker.id === 'resource:unknown_source'));
    for (const marker of markers) {
      const route = findRoute(
        storyPlaces[location].spawn,
        { x: marker.x - 30, y: marker.y + 25 },
        true,
        'meadow',
        storyBounds(location),
      );
      assert.ok(route.length);
      assert.ok(Math.hypot(route.at(-1)!.x - marker.x, route.at(-1)!.y - marker.y) < 110);
      assert.ok(route.every((point) => !storyBounds(location)(point.x, point.y)));
    }
  }
});

test('picnic and lantern scenery comes only from confirmed persistent facts', async () => {
  const { storyDecorations } = await import('./world-model.ts');
  const value = snapshot('bridge_bank');
  assert.deepEqual(storyDecorations(value), []);
  value.sceneInput!.context.memory.facts.push(
    'The picnic table is ready.',
    'Two lanterns light the near bank.',
    'One lantern lights the far bank.',
  );
  const complete = storyDecorations(value);
  assert.deepEqual(
    complete.map((item) => item.id),
    ['picnic_table', 'near_lantern_1', 'near_lantern_2', 'far_lantern'],
  );
  value.locationId = 'far_bank';
  assert.deepEqual(
    storyDecorations(value),
    complete,
    'Both banks share the same completed meadow scenery',
  );
  value.locationId = 'workshop';
  assert.deepEqual(storyDecorations(value), []);
});

test('shared crossings support both directions with followers starting behind and landing on the right side', async () => {
  const { crossingRoute } = await import('./world-model.ts');
  for (const [from, to] of [
    ['bridge_bank', 'far_bank'],
    ['far_bank', 'bridge_bank'],
  ] as const) {
    const crossing = crossingRoute(from, to)!;
    const route = findRoute(crossing.start, crossing.end, true, 'meadow', storyBounds(to, true));
    assert.ok(route.length);
    assert.equal(storyBounds(to)(crossing.end.x, crossing.end.y), false);
    assert.equal(Math.sign(crossing.followerOffset), -Math.sign(crossing.end.x - crossing.start.x));
    assert.ok(route.every((point) => !isBlocked(point.x, point.y, true)));
  }
  assert.equal(crossingRoute('bridge_bank', 'workshop'), undefined);
});

test('next-story guidance comes from the current accept choice and preserves completed goal', async () => {
  const { nextStoryLabel } = await import('./world-model.ts');
  const value = snapshot('far_bank');
  value.sceneInput!.mission!.status = 'completed';
  value.sceneInput!.availableActions = [
    {
      id: 'opaque-next-handle',
      description: 'Start a picnic.',
      action: { type: 'accept_mission' },
    },
  ];
  value.scene!.choices = [
    { actionId: 'opaque-next-handle', label: 'Let’s prepare a picnic together' },
  ];
  assert.equal(nextStoryLabel(value), 'Let’s prepare a picnic together');
  assert.equal(value.sceneInput!.mission!.goal, workshop.mission.goal);
  value.sceneInput!.availableActions = [];
  assert.equal(nextStoryLabel(value), undefined);
});

test('picnic prepares the existing meadow blanket and only confirmed readiness adds food', async () => {
  const { storySceneryFlags, storyDecorations } = await import('./world-model.ts');
  const value = snapshot('bridge_bank');
  value.sceneInput!.here.characters = [];
  value.sceneInput!.availableActions = [
    {
      id: 'prepare',
      description: '',
      action: { type: 'assemble', objectId: 'picnic_table', recipeId: 'prepare_picnic' },
    },
  ];
  const marker = storyMarkers(value)[0];
  assert.deepEqual(
    { x: marker.x, y: marker.y, prop: marker.prop },
    { x: 359, y: 272, prop: 'berry' },
  );
  assert.ok(!storySceneryFlags(value).includes('meadow-picnic'));
  value.sceneInput!.context.memory.facts.push(
    'The bridge is repaired.',
    'The picnic table is ready.',
  );
  assert.deepEqual(storySceneryFlags(value), ['meadow-bridge', 'meadow-picnic']);
  assert.deepEqual(storyDecorations(value)[0], {
    id: 'picnic_table',
    kind: 'picnic',
    x: 359,
    y: 272,
  });
  assert.deepEqual(
    value.sceneInput!.context.inventory,
    workshop.context.inventory,
    'Scenery projection does not consume or invent resources',
  );
});

test('new mission actions bind supported targets and Bramble has distinct art', () => {
  assert.equal(
    actionTarget({ type: 'inspect', objectId: 'humming_hollow' }),
    'object:humming_hollow',
  );
  assert.equal(
    actionTarget({ type: 'choose', choiceId: 'instrument_home', optionId: 'woods' }),
    'object:instrument_home',
  );
  assert.equal(
    actionTarget({ type: 'talk', characterId: 'bramble', topicId: 'visit_wish' }),
    'npc:bramble',
  );
  assert.equal(actionTarget({ type: 'invite', characterId: 'pip' }), 'npc:pip');
  const value = snapshot('workshop');
  value.sceneInput!.here.characters.push({ id: 'bramble', description: '', relationship: '' });
  const markers = storyMarkers(value);
  assert.equal(markers.find((marker) => marker.id === 'npc:bramble')!.sprite, 'bramble');
  assert.notEqual(markers.find((marker) => marker.id === 'npc:woodcutter')!.sprite, 'bramble');
});

test('clue, choice, preparation and gathering markers are reachable at both gathering locations', () => {
  const places = {
    bridge_bank: [
      'corner_site',
      'corner_style',
      'corner_frame',
      'corner_cushions',
      'gathering_place',
    ],
    far_bank: [
      'humming_hollow',
      'trembling_leaf',
      'mossy_stump',
      'hollow_branch',
      'instrument_home',
    ],
    workshop: ['workshop_path'],
  } as const;
  for (const location of ['bridge_bank', 'far_bank', 'workshop'] as const) {
    const value = snapshot(location);
    value.sceneInput!.here.characters = [{ id: 'bramble', description: '', relationship: '' }];
    value.sceneInput!.context.memory.facts.push(`The gathering is at ${location}.`);
    const objects = [
      ...places[location],
      ...(location === 'workshop' ? [] : ['gathering_seats', 'gathering_music']),
    ];
    value.sceneInput!.availableActions = objects.map((objectId, i) => ({
      id: `action-${i}`,
      description: '',
      action: { type: 'assemble', objectId, recipeId: 'test' },
    }));
    const markers = storyMarkers(value);
    assert.equal(markers.length, objects.length + 1);
    for (const marker of markers) {
      const route = findRoute(
        storyPlaces[location].spawn,
        { x: marker.x - 30, y: marker.y + 25 },
        true,
        storyPlaces[location].region,
        storyBounds(location),
      );
      assert.ok(route.length, marker.id);
      assert.ok(
        Math.hypot(route.at(-1)!.x - marker.x, route.at(-1)!.y - marker.y) < 110,
        marker.id,
      );
      assert.ok(
        route.every((point) => !storyBounds(location)(point.x, point.y)),
        marker.id,
      );
    }
  }
});

test('persistent story choices render distinct outcomes and preparation stays at its selected bank', async () => {
  const { storyDecorations } = await import('./world-model.ts');
  for (const [instrument, style, gathering] of [
    ['far_bank', 'leafy_nook', 'bridge_bank'],
    ['bridge_bank', 'star_canopy', 'far_bank'],
  ] as const) {
    const value = snapshot('bridge_bank');
    value.sceneInput!.context.memory.facts = [
      `The wind instrument is at ${instrument}.`,
      `Pip's cosy corner uses ${style}.`,
      `The gathering is at ${gathering}.`,
      'Gathering seats are ready.',
      'Gathering music is ready.',
      'The moonlight gathering has begun.',
    ];
    const decorations = storyDecorations(value);
    assert.equal(
      decorations.find((d) => d.id === 'wind_instrument')!.x > 780,
      instrument === 'far_bank',
    );
    assert.equal(
      decorations.find((d) => d.id === 'cosy_corner')!.kind,
      style === 'leafy_nook' ? 'corner-leaves' : 'corner-stars',
    );
    assert.ok(
      decorations
        .filter((d) => d.id.startsWith('gathering_') || d.id === 'moonlight_gathering')
        .every((d) => d.x > 780 === (gathering === 'far_bank')),
    );
    value.locationId = 'far_bank';
    assert.deepEqual(
      storyDecorations(value),
      decorations,
      'Persistent outcomes remain visible across the shared meadow',
    );
  }
  const unfinished = snapshot('bridge_bank');
  unfinished.sceneInput!.context.memory.facts = ['The gathering is at far_bank.'];
  assert.deepEqual(
    storyDecorations(unfinished),
    [],
    'Choosing a place does not claim setup is finished',
  );
});

test('Bramble arrives from the workshop on a reachable near-bank route', async () => {
  const { crossingRoute } = await import('./world-model.ts');
  const route = crossingRoute('workshop', 'bridge_bank')!;
  assert.ok(route);
  const points = findRoute(
    route.start,
    route.end,
    true,
    'meadow',
    storyBounds('bridge_bank', true),
  );
  assert.ok(points.length);
  assert.ok(points.every((point) => !isBlocked(point.x, point.y, true, 'meadow')));
  assert.equal(storyBounds('bridge_bank')(route.end.x, route.end.y), false);
  assert.equal(route.followerOffset, -40);
});

test('the instrument must be carried to its chosen home before its lasting decoration appears', async () => {
  const { storyDecorations } = await import('./world-model.ts');
  for (const location of ['bridge_bank', 'far_bank'] as const) {
    const value = snapshot(location);
    value.sceneInput!.here.characters = [];
    value.sceneInput!.context.memory.facts = [`The chosen wind instrument home is ${location}.`];
    value.sceneInput!.availableActions = [
      {
        id: 'place',
        description: '',
        action: {
          type: 'assemble',
          objectId: 'wind_instrument',
          recipeId: 'place_instrument',
        },
      },
    ];
    const marker = storyMarkers(value)[0];
    assert.equal(marker.id, 'object:wind_instrument');
    assert.equal(marker.x > 780, location === 'far_bank');
    assert.deepEqual(storyDecorations(value), []);
    value.sceneInput!.context.memory.facts.push(`The wind instrument is at ${location}.`);
    assert.equal(storyDecorations(value)[0].id, 'wind_instrument');
  }
});

test('both cosy corner construction markers retain the selected design', () => {
  for (const style of ['leafy_nook', 'star_canopy']) {
    const value = snapshot('bridge_bank');
    value.sceneInput!.here.characters = [];
    value.sceneInput!.context.memory.facts = [`The chosen cosy corner design is ${style}.`];
    value.sceneInput!.availableActions = ['corner_frame', 'corner_cushions'].map((objectId) => ({
      id: objectId,
      description: '',
      action: { type: 'assemble', objectId, recipeId: 'test' },
    }));
    const markers = storyMarkers(value);
    assert.equal(markers.length, 2);
    assert.ok(
      markers.every(
        (marker) => marker.shape === (style === 'leafy_nook' ? 'corner-leaves' : 'corner-stars'),
      ),
    );
  }
});
