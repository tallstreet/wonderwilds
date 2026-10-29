import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../../contracts/package.json', import.meta.url));
const Ajv2020 = require('ajv/dist/2020.js').default;
const addFormats = require('ajv-formats').default;
const schema = JSON.parse(await readFile(new URL('../../contracts/schemas/v1.json', import.meta.url), 'utf8'));
const catalog = JSON.parse(await readFile(new URL('./catalog.v1.json', import.meta.url), 'utf8'));
const ajv = new Ajv2020({ allErrors: true, strict: true });
addFormats(ajv);
ajv.addSchema(schema);
const check = (name, value) => {
  const validate = ajv.compile({ $ref: `${schema.$id}#/$defs/${name}` });
  assert.ok(validate(value), `${name}: ${JSON.stringify(validate.errors)}`);
};
for (const entry of catalog.quests) {
  const document = JSON.parse(await readFile(new URL(`../../${entry.documentPath}`, import.meta.url), 'utf8'));
  check('QuestDocument', document);
}
for (const slot of catalog.activitySlots) check('QuestActivitySlot', slot);
check('QuestGenerationContext', JSON.parse(await readFile(new URL('./generation-context.example.json', import.meta.url), 'utf8')));
check('QuestGenerationProposal', JSON.parse(await readFile(new URL('./generation-proposal.example.json', import.meta.url), 'utf8')));
console.log('Four quest documents, activity slots and future generation examples pass shared schemas.');
