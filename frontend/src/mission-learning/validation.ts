import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
import schema from '../../../contracts/schemas/v1.json';
import type {
  MissionLearningCommand,
  MissionLearningSnapshot,
} from '../../../contracts/generated/contracts.ts';

const ajv = new Ajv2020({ allErrors: false, strict: true, strictRequired: false });
addFormats(ajv);
ajv.addSchema(schema);
const snapshotValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/MissionLearningSnapshot` });
const commandValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/MissionLearningCommand` });

export function isMissionLearningSnapshot(value: unknown): value is MissionLearningSnapshot {
  return snapshotValidator(value);
}

export function isMissionLearningCommand(value: unknown): value is MissionLearningCommand {
  return commandValidator(value);
}
