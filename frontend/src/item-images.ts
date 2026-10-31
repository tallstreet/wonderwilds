import type Phaser from 'phaser';
import { items, itemTexture, rememberItemImage } from './items.ts';

/** Cosmetic-only, bounded browser cache. Failure must never block a pickup. */
export class ItemImages {
  private queue: string[] = [];
  private seen = new Set<string>();
  private running = false;
  private disposed = false;
  private request?: AbortController;
  constructor(
    private game: Phaser.Game,
    private profile: string,
    private csrf: string,
    private enabled: boolean,
    private updated: (id: string) => void,
  ) {}
  destroy() {
    this.disposed = true;
    this.request?.abort();
  }
  sync(ids: string[]) {
    for (const id of ids) {
      if (!Object.hasOwn(items, id) || this.seen.has(id)) continue;
      this.seen.add(id);
      this.queue.push(id);
    }
    void this.drain();
  }
  private async drain() {
    if (this.running || this.disposed) return;
    this.running = true;
    try {
      while (this.queue.length && !this.disposed) {
        const id = this.queue.shift()!;
        const key = `wonderwilds.item-art.v1:${this.profile}:${id}`;
        try {
          let saved: string | null = null;
          try {
            saved = localStorage.getItem(key);
          } catch {
            /* Session art still works. */
          }
          if (
            saved &&
            (saved.length > 500_000 || !/^data:image\/png;base64,[A-Za-z0-9+/]+={0,2}$/.test(saved))
          )
            saved = null;
          if (!saved && this.enabled) {
            this.request = new AbortController();
            const timer = setTimeout(() => this.request?.abort(), 125000);
            try {
              const response = await fetch('/__demo/item', {
                method: 'POST',
                signal: this.request.signal,
                headers: {
                  'Content-Type': 'application/json',
                  'X-CSRF-Token': this.csrf,
                  'X-Wonderwilds-Profile': this.profile,
                },
                body: JSON.stringify({ itemId: id }),
              });
              if (!response.ok) throw new Error('Item artwork unavailable');
              const blob = await response.blob();
              if (blob.type !== 'image/png' || blob.size > 8 * 1024 * 1024)
                throw new Error('Invalid artwork');
              const url = URL.createObjectURL(blob);
              try {
                saved = await prepareItemImage(url);
              } finally {
                URL.revokeObjectURL(url);
              }
            } finally {
              clearTimeout(timer);
              this.request = undefined;
            }
            if (saved && !this.disposed) {
              try {
                localStorage.setItem(key, saved);
              } catch {
                /* Session only. */
              }
            }
          }
          if (saved && !this.disposed) {
            const image = new Image();
            image.src = saved;
            await image.decode();
            if (image.width !== 256 || image.height !== 256 || this.disposed) continue;
            rememberItemImage(id, saved);
            if (!this.game.textures.exists(itemTexture(id)))
              this.game.textures.addImage(itemTexture(id), image);
            this.updated(id);
          }
        } catch {
          /* Use bundled artwork; retry on the next visit/reload. */
        }
        if (this.enabled && this.queue.length && !this.disposed)
          await new Promise((resolve) => setTimeout(resolve, 1600));
      }
    } finally {
      this.running = false;
    }
  }
}
export async function prepareItemImage(url: string) {
  const image = new Image();
  image.src = url;
  await image.decode();
  if (
    image.width < 256 ||
    image.height < 256 ||
    image.width > 4096 ||
    image.height > 4096 ||
    image.width * image.height > 12_000_000
  )
    throw new Error('Invalid artwork size');
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 256;
  const context = canvas.getContext('2d', { willReadFrequently: true })!;
  context.drawImage(image, 0, 0, 256, 256);
  const pixels = context.getImageData(0, 0, 256, 256);
  let visible = 0;
  for (let i = 0; i < pixels.data.length; i += 4) {
    const [r, g, b] = pixels.data.subarray(i, i + 3);
    if (r > 150 && b > 150 && g < Math.min(r, b) * 0.65) pixels.data[i + 3] = 0;
    if (pixels.data[i + 3] > 40) visible++;
  }
  if (visible < 400 || visible > 256 * 256 * 0.85) throw new Error('Unusable artwork background');
  context.putImageData(pixels, 0, 0);
  return canvas.toDataURL('image/png');
}
