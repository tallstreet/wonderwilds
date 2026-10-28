import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
import type { DirectorProposal } from './generated/contracts.ts';

// Both runtimes execute these same checked-in cases; a pass requires real rejection.
const schema = JSON.parse(readFileSync(new URL('./schemas/v1.json', import.meta.url), 'utf8'));
const cases = JSON.parse(readFileSync(new URL('./fixtures/cases.json', import.meta.url), 'utf8')) as {
  name: string; contract: string; valid: boolean; value: unknown;
}[];
const ajv = new Ajv2020.default({ allErrors: true, strict: true });
addFormats.default(ajv);
ajv.addSchema(schema);
for (const entry of cases) {
  const validate = ajv.compile({ $ref: `${schema.$id}#/$defs/${entry.contract}` });
  assert.equal(validate(entry.value), entry.valid, `${entry.name}: ${JSON.stringify(validate.errors)}`);
}
// Compile-time consumer fixture: browser wire types never contain trusted answers.
const proposal: Pick<DirectorProposal, 'schemaVersion'> = { schemaVersion: '1.0' };
assert.equal(proposal.schemaVersion, '1.0');
console.log(`${cases.length} shared contract fixtures passed in TypeScript/Ajv`);
