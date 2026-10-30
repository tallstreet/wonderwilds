import type {
  MissionLearningCommand,
  MissionLearningSnapshot,
} from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import { isMissionLearningCommand, isMissionLearningSnapshot } from './validation.ts';

const maximumResponseBytes = 64 * 1024;
const maximumCommandBytes = 4096;

/** Profile-scoped mission-learning transport; no learner identity is accepted in a command body. */
export class MissionLearningClient {
  private alive = true;
  private requests = new Set<AbortController>();

  constructor(
    private readonly profileId: string,
    private readonly csrf: string,
    private readonly fetcher: typeof fetch = (...args) => fetch(...args),
  ) {}

  destroy() {
    this.alive = false;
    for (const request of this.requests) request.abort();
    this.requests.clear();
  }

  snapshot(): Promise<MissionLearningSnapshot> {
    return this.request();
  }

  send(body: string): Promise<MissionLearningSnapshot> {
    if (body.length > maximumCommandBytes) throw new ApiFailure('INVALID_INPUT', 400);
    let value: unknown;
    try {
      value = JSON.parse(body);
    } catch {
      throw new ApiFailure('INVALID_INPUT', 400);
    }
    if (!isMissionLearningCommand(value)) throw new ApiFailure('INVALID_INPUT', 400);
    return this.request(body);
  }

  private async request(body?: string): Promise<MissionLearningSnapshot> {
    if (!this.alive) throw new ApiFailure('CONTEXT_CHANGED', 0);
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 10000);
    this.requests.add(controller);
    try {
      const response = await this.fetcher(
        body === undefined ? '/api/v1/mission-learning' : '/api/v1/mission-learning/commands',
        {
          method: body === undefined ? 'GET' : 'POST',
          credentials: 'same-origin',
          redirect: 'error',
          headers: {
            Accept: 'application/json',
            'X-Wonderwilds-Profile': this.profileId,
            ...(body === undefined
              ? {}
              : {
                  'Content-Type': 'application/json',
                  'X-CSRF-Token': this.csrf,
                }),
          },
          body,
          signal: controller.signal,
        },
      );
      const reader = response.body?.getReader();
      if (!reader) throw new ApiFailure('UNAVAILABLE', response.status);
      const decoder = new TextDecoder('utf-8', { fatal: true });
      let text = '';
      let length = 0;
      try {
        while (true) {
          const chunk = await reader.read();
          if (chunk.done) break;
          length += chunk.value.byteLength;
          if (length > maximumResponseBytes) {
            await reader.cancel();
            throw new ApiFailure('UNAVAILABLE', response.status);
          }
          text += decoder.decode(chunk.value, { stream: true });
        }
        text += decoder.decode();
      } finally {
        reader.releaseLock();
      }
      if (!this.alive) throw new ApiFailure('CONTEXT_CHANGED', 0);
      const value: unknown = JSON.parse(text);
      if (!response.ok) {
        const code = value && typeof value === 'object' ? (value as { code?: unknown }).code : null;
        if (
          code === 'INVALID_INPUT' ||
          code === 'UNAUTHENTICATED' ||
          code === 'FORBIDDEN' ||
          code === 'NOT_FOUND' ||
          code === 'REVISION_CONFLICT' ||
          code === 'RATE_LIMITED'
        )
          throw new ApiFailure(code, response.status);
        throw new ApiFailure('UNAVAILABLE', response.status);
      }
      if (!isMissionLearningSnapshot(value)) throw new ApiFailure('UNAVAILABLE', response.status);
      return value;
    } catch (error) {
      if (!this.alive) throw new ApiFailure('CONTEXT_CHANGED', 0);
      if (error instanceof ApiFailure) throw error;
      throw new ApiFailure('UNAVAILABLE', 0);
    } finally {
      clearTimeout(timer);
      this.requests.delete(controller);
    }
  }
}

export type MissionLearningPendingStore = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

/** Saves the exact uncertain request before sending it so reloads can retry the same command. */
export class MissionLearningSession {
  private current?: MissionLearningSnapshot;
  private pending?: string;
  private active = false;
  private reconcile = false;
  private reading?: Promise<MissionLearningSnapshot>;
  readonly storageKey: string;

  constructor(
    private readonly client: Pick<MissionLearningClient, 'snapshot' | 'send'>,
    profileId: string,
    private readonly storage: MissionLearningPendingStore,
    private readonly commandId: () => string = () => crypto.randomUUID(),
  ) {
    this.storageKey = `wonderwilds.mission-learning.pending.v1:${profileId}`;
    const saved = storage.getItem(this.storageKey);
    if (saved !== null) {
      if (saved.length > maximumCommandBytes)
        throw new Error('Stored practice command is invalid.');
      let value: unknown;
      try {
        value = JSON.parse(saved);
      } catch {
        throw new Error('Stored practice command is invalid.');
      }
      if (!isMissionLearningCommand(value)) throw new Error('Stored practice command is invalid.');
      this.pending = saved;
    }
  }

  get snapshot() {
    return this.current;
  }

  get busy() {
    return this.active;
  }

  get hasPending() {
    return this.pending !== undefined;
  }

  get needsRefresh() {
    return this.reconcile;
  }

  async refresh() {
    // Background reads and an activity's preflight read may overlap. Share the
    // read, while still forbidding reads from racing an in-flight command.
    if (this.reading) return this.reading;
    if (this.active) throw new Error('A practice action is still running.');
    this.active = true;
    this.reading = Promise.resolve()
      .then(() => this.client.snapshot())
      .then((snapshot) => {
        this.current = snapshot;
        this.reconcile = false;
        return snapshot;
      })
      .finally(() => {
        this.active = false;
        this.reading = undefined;
      });
    return this.reading;
  }

  async choose(payload: MissionLearningCommand['payload']) {
    while (this.reading) await this.reading;
    if (this.active || this.pending || this.reconcile || !this.current)
      throw new Error('Refresh practice progress before choosing another action.');
    const command: MissionLearningCommand = {
      schemaVersion: '1.0',
      commandId: this.commandId(),
      expectedRevision: this.current.revision,
      payload,
    };
    if (!isMissionLearningCommand(command)) throw new Error('The practice action is invalid.');
    const body = JSON.stringify(command);
    this.storage.setItem(this.storageKey, body);
    this.pending = body;
    return this.retry();
  }

  async retry() {
    if (this.active || !this.pending)
      throw new Error('There is no practice action ready to retry.');
    this.active = true;
    try {
      const result = await this.client.send(this.pending);
      this.storage.removeItem(this.storageKey);
      this.pending = undefined;
      this.current = result;
      this.reconcile = true;
      return result;
    } catch (error) {
      if (error instanceof ApiFailure && error.code === 'REVISION_CONFLICT') {
        this.storage.removeItem(this.storageKey);
        this.pending = undefined;
        this.reconcile = true;
      }
      throw error;
    } finally {
      this.active = false;
    }
  }
}
