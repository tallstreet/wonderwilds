import type { Encounter } from './types.ts';
import { isRecord } from './types.ts';
export const regions = [
  {
    id: 'meadow',
    name: 'Clover Meadow',
    subtitle: 'Every big adventure starts with a little helping paw.',
    color: '#73b99b',
    icon: '✿',
    companion: 'Pip',
    description: 'A tiny explorer with a very big heart.',
    fur: '#ffcd82',
  },
  {
    id: 'orchard',
    name: 'Peachy Orchard',
    subtitle: 'Follow the peach trees. There’s kindness to grow.',
    color: '#e6a16c',
    icon: '❀',
    companion: 'Cleo',
    description: 'The cleverest gardener in a rainbow cape.',
    fur: '#c4aff3',
  },
  {
    id: 'cove',
    name: 'Starlight Cove',
    subtitle: 'Little waves, bright ideas, and new friends.',
    color: '#7caed3',
    icon: '✧',
    companion: 'Luna',
    description: 'A brave sea kitten who follows the stars.',
    fur: '#a9dbec',
  },
];
export const curriculum = [
  {
    id: 'missing-10',
    name: 'Make a whole to 10',
    stage: 'Foundation',
    goal: 'Count on and find a missing part using objects.',
  },
  {
    id: 'share-6',
    name: 'Share fairly',
    stage: 'Foundation',
    goal: 'Share a small collection equally, one at a time.',
  },
  {
    id: 'count-1',
    name: 'Count forwards',
    stage: 'Foundation',
    goal: 'Continue a sequence of consecutive numbers.',
  },
  {
    id: 'missing-20',
    name: 'Make a whole to 20',
    stage: 'Year 1',
    goal: 'Use known parts to find a missing quantity.',
  },
  {
    id: 'share-12',
    name: 'Build equal groups',
    stage: 'Year 1',
    goal: 'Make equal groups and count each group.',
  },
  {
    id: 'count-2',
    name: 'Count in twos',
    stage: 'Year 1',
    goal: 'Recognise and continue a number pattern in twos.',
  },
  {
    id: 'missing-30',
    name: 'Bridge to the next ten',
    stage: 'Year 2 • stretch',
    goal: 'Use a visual model to find a missing part beyond 20.',
  },
  {
    id: 'share-15',
    name: 'Connect groups and totals',
    stage: 'Year 2 • stretch',
    goal: 'Relate a total to equal groups using a concrete model.',
  },
  {
    id: 'count-5',
    name: 'Count in fives',
    stage: 'Year 1–2',
    goal: 'Continue a sequence with a constant step of five.',
  },
];
export const encounters: Encounter[] = regions.flatMap((r, i): Encounter[] => [
  {
    id: `${r.id}-bridge`,
    region: r.id,
    type: 'missing',
    skillId: curriculum[i * 3].id,
    title: ['A bridge for Pip', 'The orchard stepping stones', 'Light the way home'][i],
    short: ['Fix the little bridge', 'Make a stepping-stone path', 'Light the lighthouse'][i],
    character: r.companion,
    total: [10, 16, 24][i],
    given: [6, 9, 17][i],
    prompt: [
      'Pip wants to cross! The bridge needs 10 planks. It has 6. Add the missing planks.',
      'Cleo needs 16 stones for her path. There are 9. Add the missing stones.',
      'Luna’s light needs 24 crystals. There are 17. Add the missing crystals.',
    ][i],
    reward: 'A helping-paw star',
    x: 690,
    y: 355,
  },
  {
    id: `${r.id}-picnic`,
    region: r.id,
    type: 'share',
    skillId: curriculum[i * 3 + 1].id,
    title: ['The berry picnic', 'A peach for every paw', 'Seashell surprises'][i],
    short: ['Share the picnic berries', 'Share the juicy peaches', 'Share the shiny shells'][i],
    character: r.companion,
    total: [6, 12, 15][i],
    groups: [2, 3, 3][i],
    prompt: [
      `Two friends are having a picnic. Share 6 berries so both friends have the same number.`,
      `Three kittens picked 12 peaches. Share them so every kitten has the same number.`,
      `Three friends found 15 shells. Give every friend the same number.`,
    ][i],
    reward: 'A sharing star',
    x: 285,
    y: 235,
  },
  {
    id: `${r.id}-garden`,
    region: r.id,
    type: 'pattern',
    skillId: curriculum[i * 3 + 2].id,
    title: ['Wake the flower garden', 'The hopping trail', 'A trail of stars'][i],
    short: ['Wake the sleepy flowers', 'Finish the hopping trail', 'Complete the star trail'][i],
    character: r.companion,
    start: [1, 2, 5][i],
    step: [1, 2, 5][i],
    length: 5,
    prompt: [
      'The flowers wake up in counting order. Which number comes next?',
      'Hop in twos along the trail. Which number comes next?',
      'The stars shine in groups of five. Which number comes next?',
    ][i],
    reward: 'A discovery star',
    x: 870,
    y: 220,
  },
]);
export function answerFor(e: Encounter) {
  return e.type === 'missing'
    ? e.total - e.given
    : e.type === 'share'
      ? e.total / e.groups
      : e.start + e.step * (e.length - 1);
}
export function validateEncounter(e: unknown): boolean {
  if (
    !isRecord(e) ||
    !['missing', 'share', 'pattern'].includes(String(e.type)) ||
    !curriculum.some((s) => s.id === e.skillId) ||
    !regions.some((r) => r.id === e.region)
  )
    return false;
  if (typeof e.id !== 'string' || typeof e.prompt !== 'string' || e.prompt.length > 300)
    return false;
  if (e.type === 'missing')
    return (
      typeof e.total === 'number' &&
      Number.isInteger(e.total) &&
      e.total <= 30 &&
      typeof e.given === 'number' &&
      Number.isInteger(e.given) &&
      e.given >= 0 &&
      e.given < e.total
    );
  if (e.type === 'share')
    return (
      typeof e.total === 'number' &&
      Number.isInteger(e.total) &&
      e.total > 0 &&
      e.total <= 20 &&
      typeof e.groups === 'number' &&
      [2, 3, 4].includes(e.groups) &&
      e.total % e.groups === 0
    );
  return (
    typeof e.start === 'number' &&
    Number.isInteger(e.start) &&
    e.start >= 0 &&
    typeof e.step === 'number' &&
    [1, 2, 5].includes(e.step) &&
    e.length === 5 &&
    e.start + e.step * (e.length - 1) <= 30
  );
}
export function isCorrect(e: Encounter, value: unknown) {
  return e.type === 'share'
    ? Array.isArray(value) && value.length === e.groups && value.every((n) => n === answerFor(e))
    : value === answerFor(e);
}
interface LegacyState {
  version: number;
  region: string;
  done: string[];
  evidence: { skillId: string; supported: boolean }[];
  sound: boolean;
  hero: string;
}
export const initialState = (): LegacyState => ({
  version: 1,
  region: 'meadow',
  done: [],
  evidence: [],
  sound: false,
  hero: 'rose',
});
export function restoreState(raw: string): LegacyState {
  try {
    const s: unknown = JSON.parse(raw);
    if (!isRecord(s) || s.version !== 1) return initialState();
    const done = Array.isArray(s.done)
      ? [
          ...new Set(
            s.done.filter(
              (id: unknown): id is string =>
                typeof id === 'string' && encounters.some((e) => e.id === id),
            ),
          ),
        ]
      : [];
    return {
      ...initialState(),
      region:
        typeof s.region === 'string' && regions.some((r) => r.id === s.region)
          ? s.region
          : 'meadow',
      done,
      evidence: Array.isArray(s.evidence)
        ? s.evidence
            .filter(
              (e: unknown): e is { skillId: string; supported: boolean } =>
                isRecord(e) &&
                typeof e.skillId === 'string' &&
                curriculum.some((c) => c.id === e.skillId) &&
                typeof e.supported === 'boolean',
            )
            .slice(-200)
        : [],
      sound: s.sound === true,
      hero:
        typeof s.hero === 'string' && ['rose', 'sky', 'gold'].includes(s.hero) ? s.hero : 'rose',
    };
  } catch {
    return initialState();
  }
}
export function regionUnlocked(id: string, state: { done: string[] }) {
  const i = regions.findIndex((r) => r.id === id);
  return (
    i === 0 ||
    (i > 0 &&
      encounters
        .filter((e) => e.region === regions[i - 1].id)
        .every((e) => state.done.includes(e.id)))
  );
}
