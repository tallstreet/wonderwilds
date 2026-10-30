import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spokenCard } from './answer.ts';

test('spoken cards accept explicit number, fraction, decimal and symbol forms', () => {
  for (const [transcript, label] of [
    ['five', '5'],
    ['marker group one', 'Marker group 1'],
    ['twenty-one', '21'],
    ['thirty thousand', '30000'],
    ['one hundred and six', '106'],
    ['one hundred six', '106'],
    ['one thousand and five', '1005'],
    ['one million', '1000000'],
    ['three quarters', '3/4'],
    ['one half', '1/2'],
    ['seven tenths', '7/10'],
    ['zero point five seven five', '0.575'],
    ['point three seven', '0.37'],
    ['less than', '<'],
    ['triangle', '▲'],
    ['Red cup!', 'Red cup'],
  ])
    assert.equal(spokenCard(transcript!, [{ id: 'card', label: label! }]), 'card', transcript);
});

test('spoken cards reject ambiguous, inferred and malformed answers', () => {
  const choices = [
    { id: 'a', label: '21' },
    { id: 'b', label: '22' },
  ];
  for (const text of [
    'twenty one or twenty two',
    'I think twenty one',
    'twenty twenty one',
    'and twenty one',
    '-21',
    'infinity',
    '',
  ]) {
    assert.equal(spokenCard(text, choices), undefined, text);
  }
  assert.equal(
    spokenCard('twenty one', [...choices, { id: 'other', label: 'twenty-one' }]),
    undefined,
  );
  assert.equal(
    spokenCard('one half', [
      { id: 'a', label: '1/2' },
      { id: 'b', label: 'one half' },
    ]),
    undefined,
  );
  assert.equal(spokenCard('zero', [{ id: 'a', label: '0' }]), 'a');
});
