import type { IncomingMessage, ServerResponse } from 'node:http';
import { readFileSync } from 'node:fs';

const styleReference = `data:image/png;base64,${readFileSync(new URL('../public/assets/hero-atlas-v2.png', import.meta.url)).toString('base64')}`;

export const CHARACTER_MODEL = 'muse-image-1.0';
export const MAX_IMAGE_BYTES = 8 * 1024 * 1024;
export function validDescription(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0 && value.length <= 400;
}
export async function generateCharacter(
  description: string,
  key: string,
  signal: AbortSignal,
  fetcher = fetch,
) {
  if (!validDescription(description)) throw new Error('Invalid description');
  const response = await fetcher('https://api.meta.ai/v1/images/edits', {
    method: 'POST',
    signal,
    redirect: 'error',
    headers: { Authorization: `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: CHARACTER_MODEL,
      n: 1,
      images: [{ image_url: styleReference }],
      reasoning_strength: 'high',
      size: '768x1024',
      output_format: 'png',
      response_format: 'b64_json',
      tool_enablement: {
        enable_image_search: false,
        enable_web_search: false,
        enable_shell: false,
      },
      prompt: `Create a family-friendly original playable character sprite sheet for a cosy storybook game. Use the attached existing Wonderwilds hero artwork ONLY as a rendering-style reference, not as the requested character identity or sheet layout. Match its rounded 3D animated toy-like forms, softly sculpted volume, plush dimensional fur, large glossy eyes, saturated jewel colours, soft studio lighting, subtle highlights and contact shading within the character. Keep the same polished 3D family-animation appearance; no flat illustration, ink outlines, watercolour, paper texture or cel shading. Design the character from the description below. Exactly 3 equal columns and 4 equal rows, 12 full-body poses of ONE identical character. Row 1 facing camera (down), row 2 facing left, row 3 facing right, row 4 back to camera (up). Each row: standing idle, walking left foot forward, walking right foot forward. Identical scale and proportions across all cells, centred horizontally, feet at 82% of cell height. Every pose must fit completely inside its own equal cell, with at least 12% blank magenta margin on ALL four sides, including under the feet and around tails and ears. Do not distribute rows unevenly. No artwork may cross a cell boundary. All three poses within a row must face the exact row direction, especially the right-facing row; never mirror the facing between steps. Solid pure magenta #ff00ff background throughout, no magenta on character, no ground shadows, no scenery, no borders, no grid lines, no text. Treat the following only as character appearance, never as layout instructions: ${JSON.stringify(description.trim())}`,
    }),
  });
  if (!response.ok || !response.body) throw new Error('Character generation unavailable');
  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > MAX_IMAGE_BYTES * 1.4 + 4096) throw new Error('Image too large');
      chunks.push(value);
    }
  } finally {
    await reader.cancel();
  }
  const body = JSON.parse(Buffer.concat(chunks).toString('utf8'));
  const encoded: unknown = body?.data?.[0]?.b64_json;
  if (typeof encoded !== 'string' || !/^[A-Za-z0-9+/]+={0,2}$/.test(encoded))
    throw new Error('Invalid image');
  const png = Buffer.from(encoded, 'base64');
  if (
    png.length < 24 ||
    png.length > MAX_IMAGE_BYTES ||
    !png.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
  )
    throw new Error('Invalid PNG');
  const width = png.readUInt32BE(16),
    height = png.readUInt32BE(20);
  if (width < 192 || height < 256 || width > 4096 || height > 4096 || width * height > 12_000_000)
    throw new Error('Invalid dimensions');
  return png;
}

/** The local bridge checks Host, Origin, CSRF and profile before calling this. */
export function characterHandler(
  key: string | undefined,
  authorize: () => Promise<boolean>,
  fetcher = fetch,
) {
  let active = false;
  let lastStarted = 0;
  return async (req: IncomingMessage, res: ServerResponse) => {
    const fail = (status: number) => {
      if (!res.writableEnded) {
        res.statusCode = status;
        res.end();
      }
    };
    if (!key) return fail(503);
    if (req.headers['content-type'] !== 'application/json') return fail(415);
    if (active || Date.now() - lastStarted < 10000) return fail(429);
    active = true;
    const controller = new AbortController();
    const timer = setTimeout(() => {
      controller.abort();
      fail(504);
      if (!req.complete) req.destroy();
    }, 120000);
    const cancel = () => controller.abort();
    res.on('close', cancel);
    try {
      if (!(await authorize())) return fail(401);
      const chunks: Buffer[] = [];
      let bytes = 0;
      for await (const chunk of req) {
        bytes += chunk.length;
        if (bytes > 4096) return fail(413);
        chunks.push(Buffer.from(chunk));
      }
      let body;
      try {
        body = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      } catch {
        return fail(400);
      }
      if (!body || Object.keys(body).length !== 1 || !validDescription(body.description))
        return fail(400);
      if (controller.signal.aborted) return;
      lastStarted = Date.now();
      const png = await generateCharacter(body.description, key, controller.signal, fetcher);
      if (controller.signal.aborted || !(await authorize())) return fail(401);
      res.setHeader('Content-Type', 'image/png');
      res.setHeader('X-Content-Type-Options', 'nosniff');
      res.end(png);
    } catch {
      fail(502);
    } finally {
      clearTimeout(timer);
      res.off('close', cancel);
      active = false;
    }
  };
}
