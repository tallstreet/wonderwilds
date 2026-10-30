import type { StoryCommand, StorySnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure } from '../api/client.ts';
import { isRecord } from '../types.ts';
import { isStoryCommand, isStorySnapshot } from './validation.ts';

const maximumResponseBytes = 64 * 1024;
const maximumCommandBytes = 4096;

/** One selected profile. Destroy before leaving/switching; late responses are discarded. */
export class StoryClient {
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
  snapshot(): Promise<StorySnapshot> {
    return this.request();
  }
  send(body: string): Promise<StorySnapshot> {
    if (body.length > maximumCommandBytes) throw new ApiFailure('INVALID_INPUT', 400);
    let parsed: unknown;
    try {
      parsed = JSON.parse(body);
    } catch {
      throw new ApiFailure('INVALID_INPUT', 400);
    }
    if (!isStoryCommand(parsed)) throw new ApiFailure('INVALID_INPUT', 400);
    return this.request(body);
  }
  private async request(body?: string): Promise<StorySnapshot> {
    if (!this.alive) throw new ApiFailure('CONTEXT_CHANGED', 0);
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 10000);
    this.requests.add(controller);
    try {
      const response = await this.fetcher(
        body === undefined ? '/api/v1/story' : '/api/v1/story/commands',
        {
          method: body === undefined ? 'GET' : 'POST',
          credentials: 'same-origin',
          redirect: 'error',
          headers: {
            Accept: 'application/json',
            'X-Wonderwilds-Profile': this.profileId,
            ...(body === undefined
              ? {}
              : { 'Content-Type': 'application/json', 'X-CSRF-Token': this.csrf }),
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
        const code = isRecord(value) ? value.code : undefined;
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
      if (!isStorySnapshot(value)) throw new ApiFailure('UNAVAILABLE', response.status);
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

export type PendingStore = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;
export type StoryPayload = StoryCommand['payload'];

/** Persist only the exact uncertain command, never a client-owned progress snapshot. */
export class StorySession {
  private current?: StorySnapshot;
  private pending?: string;
  private active = false;
  private reconcile = false;
  readonly storageKey: string;
  constructor(
    private readonly client: Pick<StoryClient, 'snapshot' | 'send'>,
    profileId: string,
    private readonly storage: PendingStore,
    private readonly commandId: () => string = () => crypto.randomUUID(),
  ) {
    this.storageKey = `wonderwilds.story.pending.v1:${profileId}`;
    const saved = storage.getItem(this.storageKey);
    if (saved !== null) {
      if (saved.length > maximumCommandBytes) throw new Error('Stored story command is invalid.');
      let value: unknown;
      try {
        value = JSON.parse(saved);
      } catch {
        throw new Error('Stored story command is invalid.');
      }
      if (!isStoryCommand(value)) throw new Error('Stored story command is invalid.');
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
    if (this.active) throw new Error('An action is still running.');
    this.active = true;
    try {
      this.current = await this.client.snapshot();
      this.reconcile = false;
      return this.current;
    } finally {
      this.active = false;
    }
  }
  async choose(payload: StoryPayload) {
    if (this.active || this.pending || this.reconcile || !this.current)
      throw new Error('Resolve the previous action before choosing another.');
    const command: StoryCommand = {
      schemaVersion: '1.0',
      commandId: this.commandId(),
      expectedRevision: this.current.revision,
      payload,
    };
    if (!isStoryCommand(command)) throw new Error('The story action is invalid.');
    const body = JSON.stringify(command);
    // If storage is blocked, fail before sending so reload cannot lose an uncertain write.
    this.storage.setItem(this.storageKey, body);
    this.pending = body;
    return this.retry();
  }
  async retry() {
    if (this.active || !this.pending) throw new Error('There is no action ready to retry.');
    this.active = true;
    try {
      const result = await this.client.send(this.pending);
      this.storage.removeItem(this.storageKey);
      this.pending = undefined;
      // A replay receipt can predate the latest authoritative snapshot.
      this.current = result;
      this.reconcile = true;
      return result;
    } catch (error) {
      // Auth/validation may reject this retry before checking an earlier committed receipt.
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
