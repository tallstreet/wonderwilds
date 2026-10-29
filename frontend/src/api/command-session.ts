import type { CommandPayload, GameSnapshot } from '../../../contracts/generated/contracts.ts';
import { ApiFailure, type GameClient, type PreparedCommand } from './client.ts';

type PendingCommand = { readonly key: string; readonly command: PreparedCommand };
type SessionState =
  | { status: 'idle' }
  | { status: 'sending' | 'awaiting-retry'; pending: PendingCommand }
  | { status: 'refreshing'; pending?: PendingCommand };

/** One profile's command lifecycle. UI code never reconstructs an uncertain write. */
export class CommandSession {
  private state: SessionState = { status: 'idle' };
  constructor(
    private readonly client: Pick<GameClient, 'prepare' | 'send' | 'snapshot'>,
    private current: GameSnapshot,
    private readonly commandId: () => string = () => crypto.randomUUID(),
  ) {}
  get snapshot() {
    return this.current;
  }
  get status() {
    return this.state.status;
  }
  get busy() {
    return this.status === 'sending' || this.status === 'refreshing';
  }
  get hasPending() {
    return this.pending !== undefined;
  }
  private get pending() {
    return this.state.status === 'idle' ? undefined : this.state.pending;
  }
  async send(payload: CommandPayload): Promise<GameSnapshot> {
    if (this.busy) throw new Error('An action is still saving');
    const key = JSON.stringify(payload);
    if (this.pending && this.pending.key !== key) throw new Error('Retry the pending action first');
    const pending = this.pending ?? {
      key,
      command: this.client.prepare({
        schemaVersion: '1.0',
        commandId: this.commandId(),
        expectedRevision: this.current.revision,
        payload,
      }),
    };
    return this.submit(pending);
  }
  async retry(): Promise<GameSnapshot | undefined> {
    if (this.busy) throw new Error('An action is still saving');
    return this.pending ? this.submit(this.pending) : undefined;
  }
  private async submit(pending: PendingCommand): Promise<GameSnapshot> {
    this.state = { status: 'sending', pending };
    try {
      this.current = await this.client.send(pending.command);
      this.state = { status: 'idle' };
      return this.current;
    } catch (error) {
      this.state =
        error instanceof ApiFailure && !['UNAVAILABLE', 'RATE_LIMITED'].includes(error.code)
          ? { status: 'idle' }
          : { status: 'awaiting-retry', pending };
      throw error;
    }
  }
  async refresh(): Promise<GameSnapshot> {
    if (this.busy) throw new Error('An action is still saving');
    const pending = this.pending;
    this.state = { status: 'refreshing', pending };
    try {
      this.current = await this.client.snapshot();
      this.state = { status: 'idle' };
      return this.current;
    } catch (error) {
      this.state = pending ? { status: 'awaiting-retry', pending } : { status: 'idle' };
      throw error;
    }
  }
}
