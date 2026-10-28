import { restoreProgress, SAVE_KEY } from '../progress.ts';
import { isRecord } from '../types.ts';
import type { Progress } from '../types.ts';

type MigrationStorage = Pick<Storage, 'getItem' | 'setItem'>;
export interface LegacyImport {
  sourceKey: typeof SAVE_KEY;
  sourceVersion: 1;
  digest: string;
  original: string;
  progress: Progress;
  evidenceProvenance: 'legacy_unverified';
  skillMapping: Readonly<Record<string, string>>;
}
export type MigrationInspection =
  | { status: 'absent' | 'storage_unavailable' }
  | { status: 'corrupt'; original: string }
  | { status: 'available'; data: LegacyImport };
export interface ImportAcknowledgement {
  receiptId: string;
  profileId: string;
  sourceDigest: string;
  evidenceProvenance: 'legacy_unverified';
}
export type ImportTransport = (
  data: LegacyImport,
  profileId: string,
) => Promise<ImportAcknowledgement>;
const receiptKey = (profileId: string) => `wonderwilds.import.v1.${encodeURIComponent(profileId)}`;

/** Read only. Even malformed saves remain intact and downloadable. */
export async function inspectLegacy(
  storage: MigrationStorage | undefined,
): Promise<MigrationInspection> {
  let original: string | null;
  try {
    if (!storage) return { status: 'storage_unavailable' };
    original = storage.getItem(SAVE_KEY);
  } catch {
    return { status: 'storage_unavailable' };
  }
  if (original === null) return { status: 'absent' };
  let parsed: unknown;
  try {
    parsed = JSON.parse(original);
  } catch {
    return { status: 'corrupt', original };
  }
  if (
    !isRecord(parsed) ||
    parsed.version !== 1 ||
    !Array.isArray(parsed.done) ||
    !Array.isArray(parsed.evidence)
  )
    return { status: 'corrupt', original };
  const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(original));
  const digest = Array.from(new Uint8Array(bytes), (byte) =>
    byte.toString(16).padStart(2, '0'),
  ).join('');
  return {
    status: 'available',
    data: {
      sourceKey: SAVE_KEY,
      sourceVersion: 1,
      digest,
      original,
      progress: restoreProgress(original),
      evidenceProvenance: 'legacy_unverified',
      skillMapping: { 'missing-10': 'M06', 'share-6': 'M08', 'count-1': 'M05' },
    },
  };
}
export function originalSaveBlob(
  inspection: Extract<MigrationInspection, { status: 'available' | 'corrupt' }>,
): Blob {
  return new Blob(
    [inspection.status === 'corrupt' ? inspection.original : inspection.data.original],
    { type: 'application/json' },
  );
}

/** No endpoint is invented here. Enable only with a real server import implementation. */
export async function importLegacy(
  data: LegacyImport,
  profileId: string,
  storage: MigrationStorage,
  transport: ImportTransport,
): Promise<{ receipt: ImportAcknowledgement; persisted: boolean; alreadyImported: boolean }> {
  if (!profileId) throw new Error('A parent-owned profile is required');
  let previous: string | null;
  try {
    previous = storage.getItem(receiptKey(profileId));
  } catch {
    throw new Error(
      'Cannot check migration receipt; preserve the original and retry when storage is available',
    );
  }
  if (previous !== null) {
    let receipt: unknown;
    try {
      receipt = JSON.parse(previous);
    } catch {
      throw new Error('Migration receipt needs recovery before another import');
    }
    if (!validReceipt(receipt, data, profileId))
      throw new Error(
        'This profile already has a different import receipt; reconcile before importing',
      );
    return { receipt, persisted: true, alreadyImported: true };
  }
  const receipt = await transport(data, profileId);
  if (!validReceipt(receipt, data, profileId))
    throw new Error('Server did not acknowledge this exact legacy import');
  try {
    storage.setItem(receiptKey(profileId), JSON.stringify(receipt));
    return { receipt, persisted: true, alreadyImported: false };
  } catch {
    return { receipt, persisted: false, alreadyImported: false };
  }
}
function validReceipt(
  value: unknown,
  data: LegacyImport,
  profileId: string,
): value is ImportAcknowledgement {
  return (
    isRecord(value) &&
    typeof value.receiptId === 'string' &&
    value.receiptId.length > 0 &&
    value.profileId === profileId &&
    value.sourceDigest === data.digest &&
    value.evidenceProvenance === 'legacy_unverified'
  );
}
