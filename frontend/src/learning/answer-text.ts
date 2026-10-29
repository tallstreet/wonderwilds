const numbers = [
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
  'twenty',
];
export function normaliseAnswer(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/[.!?]+$/, '')
    .trim();
}
export function spokenNumber(text: string, maximum: number): number | undefined {
  const clean = normaliseAnswer(text);
  const value = /^\d{1,2}$/.test(clean) ? Number(clean) : numbers.indexOf(clean);
  return Number.isInteger(value) && value >= 0 && value <= maximum ? value : undefined;
}
export function matchingChoice(
  text: string,
  choices: { id: string; label: string }[],
): string | undefined {
  const clean = normaliseAnswer(text);
  const number = spokenNumber(text, 20);
  const matches = choices.filter(
    (c) =>
      normaliseAnswer(c.label) === clean ||
      (number !== undefined && /^\d+$/.test(c.label) && Number(c.label) === number),
  );
  return matches.length === 1 ? matches[0].id : undefined;
}
