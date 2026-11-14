import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
import schema from '../../../contracts/schemas/v1.json';
import type {
  PairedCommand,
  PairedSnapshot,
  HouseholdProfilesSnapshot,
} from '../../../contracts/generated/contracts.ts';

const ajv = new Ajv2020({ allErrors: false, strict: true });
addFormats(ajv);
ajv.addSchema(schema);
const snapshotValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/PairedSnapshot` });
const commandValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/PairedCommand` });

export function isPairedSnapshot(value: unknown): value is PairedSnapshot {
  return snapshotValidator(value);
}
export function isPairedCommand(value: unknown): value is PairedCommand {
  return commandValidator(value);
}

const householdValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/HouseholdProfilesSnapshot` });
export function isHouseholdProfilesSnapshot(value: unknown): value is HouseholdProfilesSnapshot {
  return householdValidator(value);
}
