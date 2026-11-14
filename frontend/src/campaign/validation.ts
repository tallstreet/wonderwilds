import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
import schema from '../../../contracts/schemas/v1.json';
import type { CampaignCommand, CampaignSnapshot } from '../../../contracts/generated/contracts.ts';

const ajv = new Ajv2020({ allErrors: false, strict: true });
addFormats(ajv);
ajv.addSchema(schema);
const snapshotValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/CampaignSnapshot` });
const commandValidator = ajv.compile({ $ref: `${schema.$id}#/$defs/CampaignCommand` });

export function isCampaignSnapshot(value: unknown): value is CampaignSnapshot {
  return snapshotValidator(value);
}
export function isCampaignCommand(value: unknown): value is CampaignCommand {
  return commandValidator(value);
}
