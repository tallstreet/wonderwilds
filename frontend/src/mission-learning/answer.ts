import { normaliseAnswer } from '../learning/answer-text.ts';

const small = [
  'zero',
  'one',
  'two',
  'three',
  'four',
  'five',
  'six',
  'seven',
  'eight',
  'nine',
  'ten',
  'eleven',
  'twelve',
  'thirteen',
  'fourteen',
  'fifteen',
  'sixteen',
  'seventeen',
  'eighteen',
  'nineteen',
];
const tens = ['', '', 'twenty', 'thirty', 'forty', 'fifty', 'sixty', 'seventy', 'eighty', 'ninety'];
function words(value: number): string {
  if (value < 20) return small[value]!;
  if (value < 100)
    return `${tens[Math.floor(value / 10)]}${value % 10 ? ` ${words(value % 10)}` : ''}`;
  if (value < 1000)
    return `${words(Math.floor(value / 100))} hundred${value % 100 ? ` and ${words(value % 100)}` : ''}`;
  if (value < 1000000)
    return `${words(Math.floor(value / 1000))} thousand${value % 1000 ? `${value % 1000 < 100 ? ' and ' : ' '}${words(value % 1000)}` : ''}`;
  return 'one million';
}
const clean = (text: string) =>
  normaliseAnswer(text)
    .replace(/([a-z])-([a-z])/g, '$1 $2')
    .replace(/\s+/g, ' ');
function aliases(label: string): string[] {
  const names = [label];
  const addNumber = (digits: string) => {
    const value = Number(digits);
    if (Number.isSafeInteger(value) && value >= 0 && value <= 1000000) {
      const spelling = words(value);
      names.push(spelling, spelling.replace(/ and /g, ' '));
    }
  };
  if (/^\d{1,7}$/.test(label)) addNumber(label);
  if (/[a-z]/i.test(label)) {
    names.push(
      label.replace(/\b\d{1,7}\b/g, (digits) =>
        Number(digits) <= 1000000 ? words(Number(digits)) : digits,
      ),
    );
  }
  const fraction = /^(\d{1,2})\/(\d{1,2})$/.exec(label);
  if (fraction) {
    const numerator = Number(fraction[1]),
      denominator = Number(fraction[2]);
    const parts = [
      '',
      '',
      'half',
      'third',
      'quarter',
      'fifth',
      'sixth',
      'seventh',
      'eighth',
      'ninth',
      'tenth',
      'eleventh',
      'twelfth',
    ];
    const part = parts[denominator];
    if (part)
      names.push(
        `${words(numerator)} ${numerator === 1 ? part : denominator === 2 ? 'halves' : `${part}s`}`,
      );
    if (numerator === 1 && denominator === 2) names.push('a half', 'half');
  }
  const decimal = /^(\d{1,6})\.(\d{1,3})$/.exec(label);
  if (decimal) {
    const digits = [...decimal[2]!].map((digit) => words(Number(digit))).join(' ');
    names.push(`${words(Number(decimal[1]))} point ${digits}`);
    if (Number(decimal[1]) === 0) names.push(`point ${digits}`);
  }
  const symbols: Record<string, string[]> = {
    '<': ['less than'],
    '>': ['greater than'],
    '=': ['equals', 'equal', 'equal to'],
    '●': ['circle'],
    '▲': ['triangle'],
    '★': ['star'],
    '■': ['square'],
    '◆': ['diamond'],
    '✿': ['flower'],
  };
  names.push(...(symbols[label] ?? []));
  return names;
}

/** Match only authored card labels and explicit spoken forms. Never solve or submit a task. */
export function spokenCard(
  text: string,
  choices: { id: string; label: string }[],
): string | undefined {
  const transcript = clean(text);
  const ids = new Set(
    choices
      .filter((choice) => aliases(choice.label).some((alias) => clean(alias) === transcript))
      .map((choice) => choice.id),
  );
  return ids.size === 1 ? [...ids][0] : undefined;
}
