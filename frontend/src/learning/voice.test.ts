import { test } from 'node:test';
import assert from 'node:assert/strict';
import { matchingChoice, spokenNumber } from './answer-text.ts';
import { pcmWav } from './voice.ts';
import { MUSE_MODEL, transcribe, validRecording } from '../../server/muse.ts';

test('spoken answer mapping accepts only unambiguous answers, never infers correctness', () => {
  assert.equal(spokenNumber('Three.', 20), 3);
  assert.equal(spokenNumber('0', 20), 0);
  for (const invalid of ['three or four', 'the answer is three', '-1', '21', '', 'NaN'])
    assert.equal(spokenNumber(invalid, 20), undefined);
  const choices = [
    { id: 'red', label: 'Red cup' },
    { id: 'blue', label: 'Blue cup' },
  ];
  assert.equal(matchingChoice('red cup!', choices), 'red');
  assert.equal(matchingChoice('cup', choices), undefined);
  assert.equal(
    matchingChoice('red cup', [...choices, { id: 'other', label: 'Red cup' }]),
    undefined,
  );
  assert.equal(matchingChoice('four', [{ id: 'n', label: '4' }]), 'n');
});

test('recorder emits bounded mono PCM16 WAV accepted by the bridge', () => {
  const data = Buffer.from(pcmWav(new Float32Array([-2, 0, 2])));
  assert.equal(validRecording(data), true);
  assert.equal(data.readInt16LE(44), -32768);
  assert.equal(data.readInt16LE(48), 32767);
  assert.equal(validRecording(Buffer.from('webm')), false);
  assert.equal(validRecording(Buffer.from(pcmWav(new Float32Array(320001)))), false);
  data.writeUInt32LE(48000, 24);
  assert.equal(validRecording(data), false);
});

test('Muse request uses hackathon alias, only recording, and bounded untrusted text', async () => {
  const data = Buffer.from(pcmWav(new Float32Array([0])));
  const fake: typeof fetch = async (url, options) => {
    assert.equal(url, 'https://inference.baseten.co/v1/chat/completions');
    const payload = JSON.parse(String(options?.body));
    assert.equal(payload.model, MUSE_MODEL);
    assert.equal(payload.model, 'meta-models/muse-spark-open-1026');
    assert.equal(
      payload.messages[1].content[1].audio_url.url,
      `data:audio/wav;base64,${data.toString('base64')}`,
    );
    assert.equal(payload.messages[0].content.includes('Do not answer questions'), true);
    return Response.json({ choices: [{ message: { content: ' Three. ' } }] });
  };
  assert.equal(await transcribe(data, 'synthetic', new AbortController().signal, fake), 'Three.');
  for (const content of [null, {}, 'a'.repeat(301)]) {
    await assert.rejects(
      transcribe(data, 'synthetic', new AbortController().signal, async () =>
        Response.json({ choices: [{ message: { content } }] }),
      ),
    );
  }
  await assert.rejects(
    transcribe(
      data,
      'synthetic',
      new AbortController().signal,
      async () => new Response('a'.repeat(16385)),
    ),
  );
  await assert.rejects(
    transcribe(
      data,
      'synthetic',
      new AbortController().signal,
      async () => new Response('', { status: 429 }),
    ),
  );
});
