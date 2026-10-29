import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
import schema from '../../../contracts/schemas/v1.json';
import type { StoryCommand, StorySnapshot } from '../../../contracts/generated/contracts.ts';

const ajv = new Ajv2020({ allErrors: false, strict: true });
addFormats(ajv);
ajv.addSchema(schema);
const snapshotValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/StorySnapshot` });
const commandValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/StoryCommand` });

export function isStorySnapshot(value: unknown): value is StorySnapshot {
  return snapshotValidator(value);
}
export function isStoryCommand(value: unknown): value is StoryCommand {
  return commandValidator(value);
}
