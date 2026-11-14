import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import Ajv2020 from '../../contracts/node_modules/ajv/dist/2020.js';
const read = (path) => JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));
const structural = new Ajv2020({ strict: true, allErrors: true }).compile(read('./schema.v2.json'));
const pack = read('./missing-moonlight.v2.json');
const existing = new Set(read('../creatures/friends.v1.json').creatures.map((x) => x.id));
const index = (rows) => {
  const map = new Map(rows.map((x) => [x.id, x]));
  assert.equal(map.size, rows.length, 'duplicate ID');
  return map;
};

function validate(p) {
  assert.ok(structural(p), JSON.stringify(structural.errors));
  const missions = index(p.missions), locations = index(p.locations), abilities = index(p.abilities);
  const trials = index(p.trials), companions = index(p.companions);
  index(p.tactics.moves); index(p.routes); index(p.mechanics); index(p.sideQuests); index(p.exchanges);
  assert.equal(p.mechanics.length, 12);
  assert.equal(companions.size, 12);
  assert.equal(trials.size, 8);
  assert.equal(new Set(p.trials.map((t) => t.sealId)).size, 8);
  assert.equal(p.starterChoice.ids.length, 3);
  assert.equal(new Set(p.starterChoice.ids).size, 3);
  for (const id of p.starterChoice.ids) assert.ok(companions.get(id)?.starter);
  for (const c of p.companions) {
    assert.ok(existing.has(c.id)); assert.ok(locations.has(c.habitatId));
    assert.ok(abilities.has(c.fieldAbility));
    assert.ok(p.tactics.moves.some((m) => m.id === c.signatureMove && m.affinity === c.affinity));
  }
  const visiting = new Set(), done = new Set();
  function visit(id) {
    assert.ok(missions.has(id), `unknown prerequisite ${id}`);
    assert.ok(!visiting.has(id), `cycle at ${id}`);
    if (done.has(id)) return;
    visiting.add(id);
    for (const prerequisite of missions.get(id).requires) visit(prerequisite);
    visiting.delete(id); done.add(id);
  }
  for (const m of p.missions) { visit(m.id); assert.ok(locations.has(m.locationId)); }
  const seals = new Set(p.trials.map((t) => t.sealId));
  for (const t of p.trials) {
    assert.ok(missions.has(t.missionId)); assert.equal(missions.get(t.missionId).locationId, t.locationId);
    assert.ok(t.hazardCycle.length > 0); assert.ok(t.targetProgress > 0);
    assert.equal(t.affinityPhases[0].atProgress, 0);
    for (const [i, phase] of t.affinityPhases.entries()) {
      assert.ok(p.companions.some((c) => c.affinity === phase.affinity));
      assert.ok(phase.atProgress < t.targetProgress);
      if (i) assert.ok(phase.atProgress > t.affinityPhases[i - 1].atProgress);
    }
  }
  for (const a of p.abilities) assert.ok(seals.has(a.sealId));
  for (const r of p.routes) {
    assert.ok(locations.has(r.fromId) && locations.has(r.toId)); assert.ok(seals.has(r.requiredSeal));
    if (r.abilityId) {
      assert.ok(abilities.has(r.abilityId));
      assert.equal(r.toolAlternative, abilities.get(r.abilityId).toolId, 'missing accessible traversal alternative');
    }
  }
  // Reachability uses route seal and ability licence requirements, not coordinates or prose.
  function reachable(ownedSeals) {
    const reached = new Set(['bridge_bank']);
    let changed;
    do {
      changed = false;
      for (const r of p.routes) {
        if (!ownedSeals.has(r.requiredSeal)) continue;
        if (r.abilityId && !ownedSeals.has(abilities.get(r.abilityId).sealId)) continue;
        if (reached.has(r.fromId) || (r.bidirectional && reached.has(r.toId))) {
          for (const id of [r.fromId, r.toId]) if (!reached.has(id)) { reached.add(id); changed = true; }
        }
      }
    } while (changed);
    return reached;
  }
  const completed = new Set(), earned = new Set();
  for (let round = 0; round < p.missions.length; round++) {
    for (const m of p.missions) {
      if (completed.has(m.id) || !m.requires.every((id) => completed.has(id))) continue;
      if (!reachable(earned).has(m.locationId)) continue;
      completed.add(m.id);
      for (const t of p.trials) if (t.missionId === m.id) earned.add(t.sealId);
    }
  }
  assert.equal(completed.size, missions.size, 'mission/route/ability softlock');
  assert.equal(reachable(earned).size, locations.size, 'unreachable final habitat');
  assert.equal(p.recruitment.length, companions.size);
  assert.equal(new Set(p.recruitment.map((r) => r.creatureId)).size, companions.size);
  for (const r of p.recruitment) {
    assert.ok(companions.has(r.creatureId)); assert.ok(completed.has(r.unlockedBy));
    assert.ok(reachable(earned).has(r.habitatId));
  }
  for (const q of [...p.sideQuests, ...p.rivalEncounters]) {
    assert.ok(completed.has(q.unlockedBy)); assert.ok(locations.has(q.locationId));
  }
  for (const puzzle of p.puzzles) assert.ok(missions.has(puzzle.missionId));
  for (const e of p.exchanges) assert.ok(completed.has(e.unlockedBy));
  for (const slot of p.progression.teamSlots) assert.ok(completed.has(slot.unlockedBy));
  let xp = 0;
  assert.equal(p.progression.levelThresholds[0], xp);
  for (const [i, m] of p.progression.milestones.entries()) {
    assert.ok(completed.has(m.unlockedBy)); assert.equal(m.requiredLevel, i + 1);
    xp += p.progression.xpPerMilestone;
    assert.equal(xp, p.progression.levelThresholds[i + 1]);
  }
  assert.equal(p.progression.milestones.length + 1, p.progression.levelThresholds.length);
  assert.equal(p.progression.supportedMultiplier, 1);
  assert.equal(p.progression.storyRequiresAcademicMastery, false);
  assert.equal(p.progression.storyRequiresLevel, false);
  assert.equal(p.resources.inactivityDecay, false);
  // Initial forage makes one of each optional supply without spending reserved quest goods.
  const goods = { ...p.resources.sources[0].yieldPerExpedition };
  for (const item of p.resources.items) for (const [id, count] of Object.entries(item.recipe)) {
    assert.ok((goods[id] ?? 0) >= count, `insufficient optional supply ${id}`);
    goods[id] -= count;
  }
}

// Offline witness search for the basic level-one rules: no upgrades, XP grind or consumables.
// It deliberately excludes optional rally/upgrades; those need separate runtime verification.
function solve(p, trial, companion) {
  const moveIds = [...p.tactics.universalMoves, companion.signatureMove];
  const moves = p.tactics.moves.filter((m) => moveIds.includes(m.id));
  const tool = p.tactics.keeperTool, maxEnergy = p.tactics.levelEnergy[0];
  let frontier = [{ progress: 0, pressure: 0, energy: maxEnergy, charges: tool.charges, path: [] }];
  for (let turn = 0; turn < trial.roundLimit; turn++) {
    const next = new Map();
    for (const state of frontier) for (const action of [...moves, { id: 'keeper_tool' }]) {
      const isTool = action.id === 'keeper_tool';
      if (isTool ? state.charges === 0 : state.energy < action.energy) continue;
      const s = { ...state, path: [...state.path, action.id] };
      let shield = 0;
      if (isTool) { s.charges--; s.progress += tool.progress; shield = tool.pressureReduction; }
      else {
        s.energy -= action.energy;
        s.progress += action.progress + (action.progress > 0 && action.affinity === trial.affinityPhases.filter((phase) => phase.atProgress <= state.progress).at(-1).affinity ? 2 : 0);
        if (action.effect === 'recover_energy_2') s.energy = Math.min(maxEnergy, s.energy + 2);
        if (action.effect.startsWith('reduce_pressure_')) shield = Number(action.effect.at(-1));
      }
      if (s.progress >= trial.targetProgress) return s.path;
      s.pressure += Math.max(0, trial.hazardCycle[turn % trial.hazardCycle.length] - shield);
      if (s.pressure >= trial.pressureLimit) continue;
      const key = [s.progress, s.pressure, s.energy, s.charges].join(':');
      if (!next.has(key)) next.set(key, s);
    }
    frontier = [...next.values()];
  }
  return null;
}
validate(pack);
let witnesses = 0;
for (const trial of pack.trials) for (const id of pack.starterChoice.ids) {
  assert.ok(solve(pack, trial, pack.companions.find((c) => c.id === id)), `${trial.id} blocks ${id}`);
  witnesses++;
}
const broken = [
  (p) => { p.releaseEnabled = true; },
  (p) => { p.missions[0].requires = ['accord-trial']; },
  (p) => { p.routes[1].requiredSeal = 'seal-accord'; },
  (p) => { p.routes.find((r) => r.abilityId).toolAlternative = null; },
  (p) => { p.progression.xpPerMilestone = 1; },
  (p) => { p.companions[0].id = 'unknown'; },
  (p) => { p.recruitment[0].creatureId = p.recruitment[1].creatureId; },
  (p) => { p.resources.sources[0].yieldPerExpedition.berry = 0; },
  (p) => { p.missions[0].unexpected = true; },
  (p) => { p.progression.storyRequiresAcademicMastery = true; },
];
for (const mutate of broken) { const p = structuredClone(pack); mutate(p); assert.throws(() => validate(p)); }
const impossible = { ...pack.trials[0], targetProgress: 1000 };
assert.equal(solve(pack, impossible, pack.companions[0]), null);
console.log(`V2: ${pack.missions.length} missions, 8 trials, 12 companions and all route/ability gates checked; ${witnesses} level-one solo starter/trial witnesses; ${broken.length + 1} negative cases rejected. Offline design checks only.`);
console.log('Example Pip/Bridge witness:', solve(pack, pack.trials[0], pack.companions[0]).join(' → '));
