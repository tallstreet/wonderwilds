import type {
  EncounterInstance,
  GameCommand,
  GameSnapshot,
} from '../../../contracts/generated/contracts.ts';
import { isRecord } from '../types.ts';

export type FailureCode =
  | 'INVALID_INPUT'
  | 'UNAUTHENTICATED'
  | 'FORBIDDEN'
  | 'NOT_FOUND'
  | 'REVISION_CONFLICT'
  | 'RATE_LIMITED'
  | 'UNAVAILABLE'
  | 'CONTEXT_CHANGED';
export class ApiFailure extends Error {
  constructor(
    public readonly code: FailureCode,
    public readonly status: number,
    public readonly correlationId?: string,
  ) {
    super(
      code === 'REVISION_CONFLICT' ? 'Progress changed. Refresh and reconcile this action.' : code,
    );
  }
}
export interface PreparedCommand {
  readonly body: string;
  readonly commandId: string;
  readonly context: number;
}
const failureCodes = new Set<string>([
  'INVALID_INPUT',
  'UNAUTHENTICATED',
  'FORBIDDEN',
  'NOT_FOUND',
  'REVISION_CONFLICT',
  'RATE_LIMITED',
  'UNAVAILABLE',
]);

/** Requests use the parent session cookie; profile identity never comes from command bodies. */
export class GameClient {
  private context = 0;
  private profileId: string | undefined;
  private pending = new Set<AbortController>();
  private commands = new Map<string, string>();
  constructor(
    private readonly csrf: () => string | undefined,
    private readonly fetcher: typeof fetch = fetch,
    private readonly base = '/api/v1',
  ) {
    if (
      !base.startsWith('/') ||
      base.startsWith('//') ||
      base.includes('\\') ||
      Array.from(base).some(
        (character) => character.charCodeAt(0) <= 32 || character.charCodeAt(0) === 127,
      ) ||
      base.includes('?') ||
      base.includes('#') ||
      new URL(base, 'https://wonderwilds.invalid').origin !== 'https://wonderwilds.invalid'
    )
      throw new Error('API base must be a same-origin path');
  }
  /** Call before profile switching or logout; late responses cannot update the next profile. */
  clearContext() {
    this.context++;
    for (const request of this.pending) request.abort();
    this.pending.clear();
    this.commands.clear();
    this.profileId = undefined;
  }
  selectProfile(profileId: string) {
    if (!profileId) throw new ApiFailure('INVALID_INPUT', 400);
    this.clearContext();
    this.profileId = profileId;
  }
  prepare(command: GameCommand): PreparedCommand {
    if (
      command.schemaVersion !== '1.0' ||
      !command.commandId ||
      !Number.isSafeInteger(command.expectedRevision) ||
      command.expectedRevision < 0
    )
      throw new ApiFailure('INVALID_INPUT', 400);
    const body = JSON.stringify(command);
    const previous = this.commands.get(command.commandId);
    if (previous !== undefined && previous !== body) throw new ApiFailure('REVISION_CONFLICT', 409);
    if (this.commands.size >= 2000 && previous === undefined)
      throw new ApiFailure('RATE_LIMITED', 429);
    this.commands.set(command.commandId, body);
    return Object.freeze({ body, commandId: command.commandId, context: this.context });
  }
  async snapshot(): Promise<GameSnapshot> {
    return this.readSnapshot(await this.request('/game'));
  }
  async encounter(id: string): Promise<EncounterInstance> {
    const value = await this.request(`/encounters/${encodeURIComponent(id)}`);
    if (
      !isRecord(value) ||
      value.schemaVersion !== '1.0' ||
      value.id !== id ||
      !isRecord(value.parameters)
    )
      throw new ApiFailure('UNAVAILABLE', 503);
    return value as unknown as EncounterInstance;
  }
  /** Retry this same prepared object after uncertain transport failure. Never increment its revision. */
  async send(command: PreparedCommand): Promise<GameSnapshot> {
    if (command.context !== this.context || this.commands.get(command.commandId) !== command.body)
      throw new ApiFailure('CONTEXT_CHANGED', 0);
    return this.readSnapshot(await this.request('/commands', command.body));
  }
  private readSnapshot(value: unknown): GameSnapshot {
    if (
      !isRecord(value) ||
      value.schemaVersion !== '1.0' ||
      !Number.isSafeInteger(value.revision) ||
      typeof value.contentVersion !== 'string' ||
      typeof value.rulesVersion !== 'string' ||
      !Array.isArray(value.storyFlags) ||
      !Array.isArray(value.ownedCreatures) ||
      !Array.isArray(value.inventory) ||
      !Array.isArray(value.pendingEncounters) ||
      !Array.isArray(value.learnerSummaries)
    )
      throw new ApiFailure('UNAVAILABLE', 503);
    return value as unknown as GameSnapshot;
  }
  private async request(path: string, body?: string): Promise<unknown> {
    const context = this.context;
    const controller = new AbortController();
    const headers: Record<string, string> = { Accept: 'application/json' };
    if (this.profileId) headers['X-Wonderwilds-Profile'] = this.profileId;
    if (body !== undefined) {
      const token = this.csrf();
      if (!token) throw new ApiFailure('UNAUTHENTICATED', 401);
      headers['Content-Type'] = 'application/json';
      headers['X-CSRF-Token'] = token;
    }
    this.pending.add(controller);
    try {
      const response = await this.fetcher(`${this.base}${path}`, {
        method: body === undefined ? 'GET' : 'POST',
        credentials: 'same-origin',
        redirect: 'error',
        headers,
        body,
        signal: controller.signal,
      });
      if (context !== this.context) throw new ApiFailure('CONTEXT_CHANGED', 0);
      let value: unknown;
      try {
        value = await response.json();
      } catch {
        throw new ApiFailure('UNAVAILABLE', response.status);
      }
      // Parsing is asynchronous too: a profile change can occur after headers arrive.
      if (context !== this.context) throw new ApiFailure('CONTEXT_CHANGED', 0);
      if (!response.ok) {
        const code =
          isRecord(value) && typeof value.code === 'string' && failureCodes.has(value.code)
            ? (value.code as FailureCode)
            : 'UNAVAILABLE';
        throw new ApiFailure(
          code,
          response.status,
          isRecord(value) && typeof value.correlationId === 'string'
            ? value.correlationId
            : undefined,
        );
      }
      return value;
    } catch (error) {
      if (context !== this.context) throw new ApiFailure('CONTEXT_CHANGED', 0);
      if (error instanceof ApiFailure) throw error;
      throw new ApiFailure('UNAVAILABLE', 0);
    } finally {
      this.pending.delete(controller);
    }
  }
}
