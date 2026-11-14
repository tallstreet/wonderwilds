import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import Ajv2020 from '../../contracts/node_modules/ajv/dist/2020.js';
const read = (name) => JSON.parse(readFileSync(new URL(name, import.meta.url), 'utf8'));
const schema = new Ajv2020({ strict: true, allErrors: true }).compile(process.env.CAMPAIGN_SCHEMA ? JSON.parse(readFileSync(process.env.CAMPAIGN_SCHEMA, 'utf8')) : read('./runtime.v1.schema.json'));
const guide = read('./field-guide.v1.json');
const pack = process.env.CAMPAIGN_PACK ? JSON.parse(readFileSync(process.env.CAMPAIGN_PACK, 'utf8')) : read('./runtime.v1.json');
const ids = (rows) => {
  const indexed = new Map(rows.map((row) => [row.id, row]));
  assert.equal(indexed.size, rows.length, 'duplicate id');
  return indexed;
};
export function validate(p) {
  assert.ok(schema(p), JSON.stringify(schema.errors));
  const locations = ids(p.locations), companions = ids(p.companions), actions = ids(p.actions), encounters = ids(p.encounters);
  assert.ok(locations.get(p.start_location)?.camp, 'start must have free recovery');
  assert.equal(companions.size, 12, 'preserve full companion catalogue');
  assert.deepEqual([...ids(guide.friends).keys()].sort(), [...companions.keys()].sort(), 'field guide must cover exact roster');
  for (const entry of guide.friends) {
    assert.ok(['wait','help','play'].includes(entry.preference), 'public respectful preference');
    assert.ok(locations.has(entry.location), 'field-guide habitat must be reachable');
    for (const key of ['name','habitat','clue','observation','approachHint']) assert.ok(typeof entry[key] === 'string' && entry[key].length > 0, 'missing public field-guide text');
    const observation = actions.get('observe_' + entry.id);
    assert.equal(observation?.discovery_for, entry.id, 'missing observation');
    assert.ok(observation.grants_flags.includes('discovery:' + entry.id + ':observed'));
    for (const method of ['wait','help','play']) {
      const approach = actions.get('approach_' + entry.id + '_' + method);
      assert.equal(approach?.discovery_for, entry.id, 'missing respectful approach');
      assert.ok(approach.requires_flags.includes('discovery:' + entry.id + ':observed'), 'approach requires observation');
      assert.deepEqual(approach.costs, {}, 'mismatch cannot consume a resource');
      assert.equal(approach.grants_flags.includes('discovery:' + entry.id + ':ready'), method === entry.preference, 'public preference must open help');
      if (method !== entry.preference) assert.ok(approach.grants_flags.includes('discovery:' + entry.id + ':clue_' + method), 'mismatch must reveal a saved clue');
    }
    const helps = p.actions.filter((a) => a.discovery_for === entry.id && a.learning);
    assert.ok(helps.length, 'discovery must open an existing contextual help task');
    for (const help of helps) assert.ok(help.requires_flags.includes('discovery:' + entry.id + ':ready'));
  }

  for (const starter of p.starters) assert.ok(companions.has(starter), 'unknown starter');
  assert.equal(new Set(p.actions.map((a) => a.grant_id)).size, actions.size, 'duplicate grant');
  const mainMissions = new Set(p.actions.map((a) => a.mission).filter((id) => id === 'welcome' || id === 'constellation-ending' || /^(bridge|bloom|echo|harvest|current|tide|beacon|accord)-(clue|project|trial)$/.test(id)));
  assert.equal(mainMissions.size, 26, 'all main missions need executable actions');
  assert.equal(p.encounters.filter((e) => e.id.startsWith('trial_')).length, 8, 'eight executable keeper boards');
  for (const starter of p.starters) assert.ok(actions.get('practice_' + starter)?.encounter, 'starter practice must be a playable encounter');
  const growthKeys = new Set();
  const stages = ['first_help','field_partnership','personal_project','wayfinder'];
  const gates = ['welcome_complete','echo_complete','current_complete','beacon_complete'];
  for (const companion of companions.keys()) for (let i = 0; i < stages.length; i++) {
    const training = p.actions.filter((a) => a.recipient === companion && a.milestone === stages[i]);
    assert.equal(training.length, 1, 'exactly one authored contextual growth definition');
    const action = training[0], key = companion + ':' + stages[i];
    assert.ok(!growthKeys.has(key), 'duplicate canonical growth milestone'); growthKeys.add(key);
    assert.ok(action.requires_flags.includes(gates[i]), 'growth must respect story unlock');
    assert.ok(action.grants_flags.includes('milestone:' + key), 'growth must persist recipient milestone');
    if (i) assert.ok(action.requires_flags.includes('milestone:' + companion + ':' + stages[i-1]), 'growth must remain in order for late recruits');
  }

  for (const [ability, tool] of Object.entries({brace:'brace_kit',bloom:'seed_kit',resonate:'tuning_fork',ferry:'folding_raft',illuminate:'lantern'})) {
    const methods = p.actions.filter((a) => a.field_ability === ability);
    assert.equal(methods.length, 2, 'each field ability needs companion and tool routes');
    const companion = methods.find((a) => a.field_method === 'companion');
    const free = methods.find((a) => a.field_method === 'tool');
    assert.ok(companion && free, 'missing field alternative');
    assert.ok(free.requires_flags.includes(tool), 'field tool must be granted before use');
    assert.ok(companion.grants_flags.some((f) => free.grants_flags.includes(f)), 'field alternatives must reach the same lasting outcome');
    assert.deepEqual(free.costs, {}, 'essential tool alternative must be free');
  }
  const flags = new Set(['starter_chosen', 'welcome_complete']);
  for (const a of p.actions) for (const f of a.grants_flags) flags.add(f);
  for (const e of p.encounters) for (const f of e.win_flags) flags.add(f);
  for (const a of p.actions) if (a.puzzle?.operation === 'check') flags.add(`puzzle:${a.puzzle.id}:solved`);
  for (const l of p.locations) for (const r of l.routes) {
    assert.ok(locations.has(r.to), 'unknown route destination');
    for (const f of r.requires_flags) assert.ok(flags.has(f), `unknown route flag ${f}`);
    assert.ok(locations.get(r.to).routes.some((back) => back.to === l.id), 'route must permit a return');
  }
  for (const a of p.actions) {
    assert.ok(locations.has(a.location), 'unknown action location');
    for (const f of a.requires_flags) assert.ok(flags.has(f), `unknown predicate ${f}`);
    assert.ok(!a.requires_flags.some((f) => a.forbids_flags.includes(f)), 'contradictory predicates');
    if (a.encounter) assert.ok(encounters.has(a.encounter), 'unknown encounter');
    if (a.recruit) assert.ok(companions.has(a.recruit), 'unknown first-slice recruit');
    assert.equal(a.learning === true, ['learn','training'].includes(a.kind), 'assessment must be an explicit invitation');
    if (a.learning) assert.ok(a.learning_context, 'learning context missing');
    if (a.recipient) assert.ok(companions.has(a.recipient), 'unknown growth recipient');
    if (a.discovery_for) assert.ok(companions.has(a.discovery_for), 'unknown discovery friend');
    if (a.milestone) assert.equal(a.kind, 'training', 'milestone must use training mechanic');
    if (a.puzzle) {
      assert.equal(a.kind, 'puzzle');
      assert.equal(a.forbids_flags.length, 0, 'puzzle controls must remain repeatable');
      assert.deepEqual(a.costs, {}, 'wrong puzzle choice cannot consume supplies');
      assert.deepEqual(a.rewards, {}, 'repeatable puzzle cannot farm supplies');
      if (a.puzzle.operation === 'check') {
        assert.ok(a.puzzle.goal, 'puzzle needs explicit goal');
        for (const [side, value] of Object.entries(a.puzzle.goal)) assert.ok(p.actions.some((control) => control.puzzle?.id === a.puzzle.id && control.puzzle.operation === `set_${side}` && control.puzzle.value === value), 'unreachable puzzle goal');
      }
    } else assert.ok(a.forbids_flags.length, 'ordinary reward action must be once-only');
    assert.ok(!a.requires_flags.some((f) => /mastery|level_[2-5]|starter_(pip|tansy|nib)/.test(f)), 'mandatory access cannot require mastery, level or starter');
  }
  for (const board of p.encounters) if (board.relay) {
    const { round_targets: rounds, required_affinities: affinities } = board.relay;
    assert.ok(rounds.every((value, index) => value > (rounds[index - 1] ?? 0)), 'relay rounds must advance');
    assert.equal(rounds.at(-1), board.target, 'relay must end at its board target');
    assert.ok(affinities <= 2 && affinities <= rounds.length, 'relay must fit two travelling friends');
  }
  // Monotone prerequisite closure is a reference/supply sanity check, not a runtime completion witness.
  const availableFlags = new Set(['starter_chosen','welcome_complete']);
  const supplies = { ...p.initial_supplies }, reached = new Set(), places = new Set([p.start_location]);
  for (let pass = 0; pass < p.actions.length + p.locations.length; pass++) {
    for (const id of [...places]) for (const r of locations.get(id).routes) if (r.requires_flags.every((f) => availableFlags.has(f))) places.add(r.to);
    for (const a of p.actions) {
      if (reached.has(a.id) || !places.has(a.location) || !a.requires_flags.every((f) => availableFlags.has(f))) continue;
      for (const [item, count] of Object.entries(a.costs)) assert.ok((supplies[item] ?? 0) >= count, `unfunded consumption ${a.id}:${item}`);
      for (const [item, count] of Object.entries(a.costs)) supplies[item] -= count;
      for (const [item, count] of Object.entries(a.rewards)) supplies[item] = (supplies[item] ?? 0) + count;
      for (const f of a.grants_flags) availableFlags.add(f);
      if (a.encounter) for (const f of encounters.get(a.encounter).win_flags) availableFlags.add(f);
      if (a.puzzle?.operation === 'check') availableFlags.add(`puzzle:${a.puzzle.id}:solved`);
      reached.add(a.id);
    }
  }
  assert.equal(reached.size, actions.size, `unreachable authored actions: ${p.actions.filter((a) => !reached.has(a.id)).map((a) => a.id)}`);
  assert.equal(supplies.quest_branch, 0, 'bridge branches must be conserved');
  assert.equal(supplies.quest_plank, 0, 'bridge planks must be conserved');
  assert.ok(availableFlags.has('world_bridge_restored') && availableFlags.has('world_garden_restored'));
  return true;
}
validate(pack);
const negatives = [
  (p) => { p.encounters.find((e) => e.id === 'rowan_final_board').relay = { round_targets: [3, 3, 9], required_affinities: 2 }; },
  (p) => { p.encounters.find((e) => e.id === 'rowan_final_board').relay = { round_targets: [3, 6, 8], required_affinities: 2 }; },
  (p) => { p.encounters.find((e) => e.id === 'rowan_final_board').relay = { round_targets: [3, 6, 9], required_affinities: 3 }; },
  (p) => { p.actions.find((a) => a.id === 'approach_pip_wait').grants_flags = ['wrong_ready']; },
  (p) => { p.actions.find((a) => a.id === 'approach_pip_play').costs.berry = 1; },
  (p) => { p.actions.find((a) => a.id === 'help_pip').requires_flags = ['starter_friendship_offered']; },
  (p) => { p.actions.find((a) => a.id === 'field_ferry_tool').requires_flags = ['current_seal']; },
  (p) => { p.actions.find((a) => a.id === 'field_ferry_companion').grants_flags = ['wrong_route']; },
  (p) => { p.actions.find((a) => a.id === 'constellation_gathering').mission = 'missing'; },
  (p) => { p.actions.find((a) => a.id === 'train_pip_wayfinder').requires_flags = ['welcome_complete']; },
  (p) => { p.actions.find((a) => a.id === 'train_pip_wayfinder').recipient = 'tansy'; },
  (p) => p.actions.push(structuredClone(p.actions[0])),
  (p) => { p.actions[1].grant_id = p.actions[0].grant_id; },
  (p) => { p.actions[0].location = 'missing'; },
  (p) => { p.locations[0].routes[0].to = 'missing'; },
  (p) => { p.actions[0].requires_flags = ['missing']; },
  (p) => { p.actions.find((a) => a.encounter).encounter = 'missing'; },
  (p) => { p.actions.find((a) => a.recruit).recruit = 'missing'; },
  (p) => { p.actions.find((a) => a.id === 'bridge_planks').costs.quest_branch = 4; },
  (p) => { p.actions.find((a) => a.id === 'water_check').puzzle.goal.near = 'impossible'; },
  (p) => { p.actions.find((a) => a.id === 'water_check').rewards.berry = 1; },
  (p) => { p.actions.find((a) => a.learning).learning = false; },
  (p) => { p.actions.find((a) => a.id === 'bridge_build').requires_flags = ['world_garden_restored']; },
];
for (const mutate of negatives) { const invalid = structuredClone(pack); mutate(invalid); assert.throws(() => validate(invalid)); }
console.log(`Runtime content: ${pack.actions.length} actions, ${pack.encounters.length} boards, ${negatives.length} negative cases passed. Rust campaign tests own actual transition/completion witnesses.`);
