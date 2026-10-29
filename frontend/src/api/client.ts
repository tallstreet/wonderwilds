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

const questText = (value: unknown, maximum: number) =>
  typeof value === 'string' && value.length > 0 && [...value].length <= maximum;
const questId = (value: unknown) =>
  typeof value === 'string' && value.length <= 120 && /^[a-z][a-z0-9-]*$/.test(value);
const exactFields = (value: Record<string, unknown>, fields: string[]) =>
  fields.every((field) => Object.hasOwn(value, field)) &&
  Object.keys(value).every((field) => fields.includes(field));
const personalBinding = (value: unknown) =>
  isRecord(value) &&
  exactFields(value, ['questId', 'documentVersion', 'nodeId', 'episodeId']) &&
  questId(value.questId) &&
  questText(value.documentVersion, 40) &&
  questId(value.nodeId) &&
  questText(value.episodeId, 120);
const personalNode = (value: unknown): boolean => {
  if (!isRecord(value) || !questId(value.id) || !questText(value.text, 600)) return false;
  if (value.type === 'choice') {
    return (
      exactFields(value, ['id', 'type', 'text', 'choices']) &&
      Array.isArray(value.choices) &&
      value.choices.length >= 2 &&
      value.choices.length <= 3 &&
      value.choices.every(
        (choice) =>
          isRecord(choice) &&
          exactFields(choice, ['id', 'label', 'nextNodeId']) &&
          questId(choice.id) &&
          questText(choice.label, 120) &&
          questId(choice.nextNodeId),
      ) &&
      new Set(value.choices.map((choice) => choice.id)).size === value.choices.length
    );
  }
  if (value.type === 'activity')
    return (
      exactFields(value, ['id', 'type', 'text', 'activitySlotId', 'nextNodeId']) &&
      questId(value.activitySlotId) &&
      questId(value.nextNodeId)
    );
  if (value.type === 'handoff')
    return (
      exactFields(value, ['id', 'type', 'text', 'continuationTags']) &&
      Array.isArray(value.continuationTags) &&
      value.continuationTags.length >= 1 &&
      value.continuationTags.length <= 5 &&
      value.continuationTags.every(questId) &&
      new Set(value.continuationTags).size === value.continuationTags.length
    );
  return false;
};

/** Requests use the parent session cookie; profile identity never comes from command bodies. */
export class GameClient {
  private context = 0;
  private profileId: string | undefined;
  private pending = new Set<AbortController>();
  private commands = new Map<string, string>();
  constructor(
    private readonly csrf: () => string | undefined,
    private readonly fetcher: typeof fetch = (...args) => fetch(...args),
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
      !isRecord(value.parameters) ||
      (value.personalQuest !== undefined && !personalBinding(value.personalQuest))
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
    const id = (item: unknown): item is string =>
      typeof item === 'string' && item.length > 0 && item.length <= 120;
    const optionalList = (
      items: unknown,
      maximum: number,
      valid: (item: unknown) => boolean,
      key: (item: unknown) => string,
    ) =>
      items === undefined ||
      (Array.isArray(items) &&
        items.length <= maximum &&
        items.every(valid) &&
        new Set(items.map(key)).size === items.length);
    const boundedInteger = (item: unknown, maximum = 1_000_000_000, minimum = 0) =>
      Number.isSafeInteger(item) && Number(item) >= minimum && Number(item) <= maximum;
    const trainingOffer = (item: unknown) =>
      isRecord(item) &&
      id(item.trainingId) &&
      boundedInteger(item.requiredLevel, 5, 1) &&
      boundedInteger(item.xp) &&
      id(item.reactionId);
    if (
      !optionalList(
        value.personalQuestOffers,
        12,
        (offer) =>
          isRecord(offer) &&
          exactFields(offer, [
            'questId',
            'documentVersion',
            'title',
            'companionId',
            'minimumCharacterLevel',
            'available',
            'completed',
          ]) &&
          questId(offer.questId) &&
          questText(offer.documentVersion, 40) &&
          questText(offer.title, 120) &&
          questId(offer.companionId) &&
          boundedInteger(offer.minimumCharacterLevel, 5, 1) &&
          typeof offer.available === 'boolean' &&
          typeof offer.completed === 'boolean',
        (offer) => (offer as { questId: string }).questId,
      ) ||
      (value.personalQuest !== undefined &&
        (!isRecord(value.personalQuest) ||
          !exactFields(value.personalQuest, [
            'questId',
            'documentVersion',
            'episodeId',
            'title',
            'companionId',
            'node',
          ]) ||
          !questId(value.personalQuest.questId) ||
          !questText(value.personalQuest.documentVersion, 40) ||
          !questText(value.personalQuest.episodeId, 120) ||
          !questText(value.personalQuest.title, 120) ||
          !questId(value.personalQuest.companionId) ||
          !personalNode(value.personalQuest.node))) ||
      value.pendingEncounters.some(
        (encounter) =>
          isRecord(encounter) &&
          encounter.personalQuest !== undefined &&
          (!personalBinding(encounter.personalQuest) ||
            !isRecord(value.personalQuest) ||
            !isRecord(value.personalQuest.node) ||
            !isRecord(encounter.personalQuest) ||
            encounter.personalQuest.questId !== value.personalQuest.questId ||
            encounter.personalQuest.documentVersion !== value.personalQuest.documentVersion ||
            encounter.personalQuest.nodeId !== value.personalQuest.node.id ||
            encounter.personalQuest.episodeId !== value.personalQuest.episodeId),
      ) ||
      (value.teamCapacity !== undefined && !boundedInteger(value.teamCapacity, 2)) ||
      (value.explorerProgression !== undefined &&
        (!isRecord(value.explorerProgression) ||
          !boundedInteger(value.explorerProgression.xp) ||
          !boundedInteger(value.explorerProgression.level, 100, 1))) ||
      !optionalList(
        value.characterProgression,
        50,
        (entry) =>
          isRecord(entry) &&
          id(entry.creatureId) &&
          boundedInteger(entry.xp) &&
          boundedInteger(entry.level, 5, 1) &&
          (entry.nextLevelXp === undefined || boundedInteger(entry.nextLevelXp)) &&
          (entry.reactionId === undefined || id(entry.reactionId)) &&
          Array.isArray(entry.completedTrainingIds) &&
          optionalList(entry.completedTrainingIds, 100, id, (item) => item as string) &&
          Array.isArray(entry.availableTraining) &&
          optionalList(
            entry.availableTraining,
            100,
            trainingOffer,
            (item) => (item as { trainingId: string }).trainingId,
          ),
        (entry) => (entry as { creatureId: string }).creatureId,
      ) ||
      !optionalList(
        value.friendship,
        50,
        (entry) =>
          isRecord(entry) &&
          id(entry.creatureId) &&
          Number.isInteger(entry.points) &&
          Number(entry.points) >= 0 &&
          Number(entry.points) <= 6,
        (entry) => (entry as { creatureId: string }).creatureId,
      ) ||
      !optionalList(value.transformedCreatures, 50, id, (entry) => entry as string) ||
      !optionalList(
        value.availableHelpingMoments,
        200,
        (entry) => isRecord(entry) && id(entry.creatureId) && id(entry.completedQuestId),
        (entry) => JSON.stringify(entry),
      )
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
