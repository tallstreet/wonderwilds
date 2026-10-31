import type { Facing } from './types.ts';

export const CUSTOM_TEXTURE = 'custom-character-v1';
export const CELL = 128;
export const SHEET_WIDTH = CELL * 3;
export const SHEET_HEIGHT = CELL * 4;
export function customFrame(
  facing: Facing,
  phase: number,
  moving: boolean,
  reducedMotion: boolean,
) {
  const row = { down: 0, left: 1, right: 2, up: 3 }[facing];
  return row * 3 + (moving && !reducedMotion ? 1 + (phase % 1 < 0.5 ? 0 : 1) : 0);
}
export function customFacing(dx: number, dy: number, previous: Facing): Facing {
  if (!dx && !dy) return previous;
  return Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'right' : 'left') : dy > 0 ? 'down' : 'up';
}
export function validSavedCharacter(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.length < 2_000_000 &&
    /^data:image\/png;base64,[A-Za-z0-9+/]+={0,2}$/.test(value)
  );
}
export async function loadCharacterImage(url: string) {
  const image = new Image();
  image.src = url;
  await image.decode();
  if (
    image.width < 192 ||
    image.height < 256 ||
    image.width > 4096 ||
    image.height > 4096 ||
    image.width * image.height > 12_000_000
  )
    throw new Error('Invalid image size');
  return image;
}
export interface CharacterCellDiagnostic {
  row: number;
  column: number;
  facing: Facing;
  pose: string;
  cell: { x: number; y: number; width: number; height: number };
  bounds: { x: number; y: number; width: number; height: number } | null;
  occupiedPercent: number;
  issues: string[];
}
export interface CharacterSheetDiagnostic {
  width: number;
  height: number;
  removedBackgroundPixels: number;
  cells: CharacterCellDiagnostic[];
}
export class CharacterSheetError extends Error {
  constructor(
    public diagnostic: CharacterSheetDiagnostic,
    public keyedPreview: string,
  ) {
    const failed = diagnostic.cells.filter((cell) => cell.issues.length);
    const first = failed[0];
    super(
      `The sheet could not be used: row ${first.row}, column ${first.column} (${first.facing}, ${first.pose}): ${first.issues.join('; ')}. Inspect the returned image below.`,
    );
    this.name = 'CharacterSheetError';
  }
}
export function inspectCharacterCells(
  data: Uint8ClampedArray,
  width: number,
  height: number,
): CharacterCellDiagnostic[] {
  const cells: CharacterCellDiagnostic[] = [];
  for (let frame = 0; frame < 12; frame++) {
    const col = frame % 3,
      row = Math.floor(frame / 3);
    const x = Math.round((col * width) / 3),
      y = Math.round((row * height) / 4);
    const w = Math.round(((col + 1) * width) / 3) - x,
      h = Math.round(((row + 1) * height) / 4) - y;
    let left = w,
      right = -1,
      top = h,
      bottom = -1,
      occupied = 0;
    for (let py = 0; py < h; py++)
      for (let px = 0; px < w; px++) {
        if (data[((y + py) * width + x + px) * 4 + 3] > 40) {
          left = Math.min(left, px);
          right = Math.max(right, px);
          top = Math.min(top, py);
          bottom = Math.max(bottom, py);
          occupied++;
        }
      }
    const issues: string[] = [];
    if (occupied < w * h * 0.015) issues.push('too little visible artwork');
    if (occupied > w * h * 0.8) issues.push('background still fills most of the cell');
    if (occupied && (left < 1 || top < 1 || right >= w - 1 || bottom >= h - 1))
      issues.push(
        `artwork touches cell edges (${[left < 1 && 'left', top < 1 && 'top', right >= w - 1 && 'right', bottom >= h - 1 && 'bottom'].filter(Boolean).join(', ')})`,
      );
    cells.push({
      row: row + 1,
      column: col + 1,
      facing: (['down', 'left', 'right', 'up'] as const)[row],
      pose: ['idle', 'left step', 'right step'][col],
      cell: { x, y, width: w, height: h },
      bounds: occupied
        ? { x: x + left, y: y + top, width: right - left + 1, height: bottom - top + 1 }
        : null,
      occupiedPercent: Math.round((occupied / (w * h)) * 10000) / 100,
      issues,
    });
  }
  return cells;
}
/** Convert the provider's opaque magenta sheet into bounded, grounded game frames. */
export async function prepareCharacter(url: string) {
  const image = await loadCharacterImage(url);
  const source = document.createElement('canvas');
  source.width = image.width;
  source.height = image.height;
  const context = source.getContext('2d', { willReadFrequently: true })!;
  context.drawImage(image, 0, 0);
  const pixels = context.getImageData(0, 0, source.width, source.height);
  let removedBackgroundPixels = 0;
  for (let i = 0; i < pixels.data.length; i += 4) {
    const [r, g, b] = pixels.data.subarray(i, i + 3);
    if (r > 150 && b > 150 && g < Math.min(r, b) * 0.65) {
      if (pixels.data[i + 3] > 0) removedBackgroundPixels++;
      pixels.data[i + 3] = 0;
    }
  }
  context.putImageData(pixels, 0, 0);
  const output = document.createElement('canvas');
  output.width = SHEET_WIDTH;
  output.height = SHEET_HEIGHT;
  const out = output.getContext('2d')!;
  const diagnostic: CharacterSheetDiagnostic = {
    width: image.width,
    height: image.height,
    removedBackgroundPixels,
    cells: inspectCharacterCells(pixels.data, image.width, image.height),
  };
  if (diagnostic.cells.some((cell) => cell.issues.length)) {
    throw new CharacterSheetError(diagnostic, source.toDataURL('image/png'));
  }
  const frames = diagnostic.cells.map((cell) => cell.bounds!);
  const scale = Math.min(
    104 / Math.max(...frames.map((f) => f.width)),
    112 / Math.max(...frames.map((f) => f.height)),
  );
  frames.forEach(({ x, y, width: bw, height: bh }, frame) => {
    const col = frame % 3,
      row = Math.floor(frame / 3);
    out.drawImage(
      source,
      x,
      y,
      bw,
      bh,
      col * CELL + (CELL - bw * scale) / 2,
      row * CELL + 120 - bh * scale,
      bw * scale,
      bh * scale,
    );
  });
  return output.toDataURL('image/png');
}
