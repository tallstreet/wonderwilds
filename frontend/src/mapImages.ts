import { isMapId, maps } from './maps.ts';

export function mapImageUrl(region: string): string | undefined {
  if (!isMapId(region)) return undefined;
  const originals: Record<string, string> = {
    meadow: 'clover-meadow-v2.png',
    orchard: 'peachy-orchard-v2.png',
    cove: 'starlight-cove-v2.png',
  };
  return maps[region].biome === undefined
    ? `/assets/${originals[region]}`
    : `/assets/places/${region}.webp`;
}

/** Deduplicate in-flight loads; failures can retry on the next visit. */
export function createImageRequests(load: (key: string, url: string) => Promise<void>) {
  const pending = new Map<string, Promise<void>>();
  return (key: string, url: string) => {
    const existing = pending.get(key);
    if (existing) return existing;
    const request = load(key, url).finally(() => pending.delete(key));
    pending.set(key, request);
    return request;
  };
}
