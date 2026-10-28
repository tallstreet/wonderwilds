import type { Progress, Evidence, Encounter, Attempt } from './types.ts';
import { isRecord } from './types.ts';
type SaveStorage = Pick<Storage, 'getItem' | 'setItem'>;
import { heroes } from './art.ts';
import { isMapId } from './maps.ts';
import { isBlocked } from './navigation.ts';
import { encounters, curriculum } from './content.ts';
export const activeEncounters = encounters.filter((e) => e.region === 'meadow');
export const SAVE_KEY = 'starwhisker.meadow.v1';
export const freshState = (): Progress => ({
  version: 1,
  region: 'meadow',
  hero: 'rose',
  companionName: 'Pip',
  done: [],
  evidence: [],
  position: { x: 160, y: 490 },
});
const ids = new Set(activeEncounters.map((e) => e.id));
export function validEvidence(e: unknown): e is Evidence {
  if (!isRecord(e)) return false;
  const encounter = activeEncounters.find((item) => item.id === e?.encounterId);
  return (
    !!encounter &&
    e.skillId === encounter.skillId &&
    e.variant === 'meadow-v1' &&
    e.representation ===
      { missing: 'countable-planks', share: 'equal-plates', pattern: 'flower-sequence' }[
        encounter.type
      ] &&
    typeof e.attempts === 'number' &&
    Number.isInteger(e.attempts) &&
    e.attempts >= 1 &&
    e.attempts <= 10000 &&
    typeof e.hints === 'number' &&
    Number.isInteger(e.hints) &&
    e.hints >= 0 &&
    e.hints <= 3 &&
    typeof e.assisted === 'boolean' &&
    typeof e.supported === 'boolean' &&
    e.supported === (e.hints > 0 || e.assisted) &&
    typeof e.completedAt === 'string' &&
    Number.isFinite(Date.parse(e.completedAt))
  );
}
export function restoreProgress(raw: string | null): Progress {
  const blank = freshState();
  try {
    const s: unknown = JSON.parse(raw ?? 'null');
    if (!isRecord(s) || s.version !== 1) return blank;
    const done = Array.isArray(s.done)
      ? [
          ...new Set(
            s.done.filter((id: unknown): id is string => typeof id === 'string' && ids.has(id)),
          ),
        ]
      : [];
    const p = s.position;
    const position =
      isRecord(p) &&
      typeof p.x === 'number' &&
      typeof p.y === 'number' &&
      Number.isFinite(p.x) &&
      Number.isFinite(p.y) &&
      p.x >= 40 &&
      p.x <= 1080 &&
      p.y >= 90 &&
      p.y <= 615
        ? { x: p.x, y: p.y }
        : blank.position;
    const result: Progress = {
      ...blank,
      region: isMapId(s.region) ? s.region : 'meadow',
      hero: typeof s.hero === 'string' && Object.hasOwn(heroes, s.hero) ? s.hero : 'rose',
      companionName:
        typeof s.companionName === 'string' && s.companionName.trim()
          ? s.companionName.trim().slice(0, 20)
          : 'Pip',
      done,
      evidence: Array.isArray(s.evidence) ? s.evidence.filter(validEvidence).slice(-200) : [],
      position,
    };
    if (isBlocked(position.x, position.y, done.includes('meadow-bridge'), result.region))
      result.position = blank.position;
    return result;
  } catch {
    return blank;
  }
}
export function completeEncounter(
  state: Progress,
  encounter: Encounter,
  session: Attempt,
  now = new Date().toISOString(),
) {
  if (!ids.has(encounter.id)) throw new Error('Unknown encounter');
  const record = {
    encounterId: encounter.id,
    skillId: encounter.skillId,
    variant: 'meadow-v1',
    representation: {
      missing: 'countable-planks',
      share: 'equal-plates',
      pattern: 'flower-sequence',
    }[encounter.type],
    attempts: session.attempts,
    hints: session.hints,
    assisted: session.assisted,
    supported: session.hints > 0 || session.assisted,
    completedAt: now,
  };
  if (!validEvidence(record)) throw new Error('Invalid learning evidence');
  return {
    ...state,
    done: [...new Set([...state.done, encounter.id])],
    evidence: [...state.evidence, record].slice(-200),
  };
}
export function writeProgress(state: Progress, storage: SaveStorage | undefined) {
  if (!storage) return false;
  try {
    storage.setItem(SAVE_KEY, JSON.stringify(state));
    return true;
  } catch {
    return false;
  }
}
export function readProgress(storage: SaveStorage | undefined) {
  if (!storage) return { state: freshState(), available: false };
  try {
    return { state: restoreProgress(storage.getItem(SAVE_KEY)), available: true };
  } catch {
    return { state: freshState(), available: false };
  }
}
export function skillSummary(state: Progress) {
  return curriculum
    .filter((c) => activeEncounters.some((e) => e.skillId === c.id))
    .map((c) => {
      const records = state.evidence.filter((e) => e.skillId === c.id);
      return {
        ...c,
        completed: records.length,
        independent: records.filter((e) => !e.supported).length,
        supported: records.filter((e) => e.supported).length,
      };
    });
}
