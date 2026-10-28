import test from 'node:test';
import { access } from 'node:fs/promises';
import assert from 'node:assert/strict';
import { createImageRequests, mapImageUrl } from './mapImages.ts';
import { maps } from './maps.ts';

test('Each destination has a distinct image URL and unknown IDs are rejected', () => {
  const urls = Object.keys(maps).map(mapImageUrl);
  assert.equal(urls.length, 75);
  assert.equal(new Set(urls).size, 75);
  assert.ok(urls.every(Boolean));
  assert.equal(mapImageUrl('../unknown'), undefined);
  assert.equal(mapImageUrl('__proto__'), undefined);
  assert.equal(mapImageUrl('meadow'), '/assets/clover-meadow-v2.png');
});

test('Concurrent visits share a request, independent destinations load separately, and failures retry', async () => {
  const completions = new Map<string, { resolve(): void; reject(error: Error): void }>();
  let count = 0;
  const request = createImageRequests((key) => {
    count++;
    return new Promise<void>((resolve, reject) => completions.set(key, { resolve, reject }));
  });
  const first = request('first', '/first.webp');
  assert.equal(request('first', '/first.webp'), first);
  const second = request('second', '/second.webp');
  assert.equal(count, 2);
  completions.get('second')!.resolve();
  await second;
  completions.get('first')!.reject(new Error('offline'));
  await assert.rejects(first, /offline/);
  const retry = request('first', '/first.webp');
  assert.equal(count, 3);
  completions.get('first')!.resolve();
  await retry;
});

test('Every destination image is shipped with the app', async () => {
  await Promise.all(
    Object.keys(maps).map(async (region) => {
      const url = mapImageUrl(region)!;
      await access(new URL(`../public${url}`, import.meta.url));
    }),
  );
});
