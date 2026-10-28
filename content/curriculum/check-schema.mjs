import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import Ajv2020 from '../../contracts/node_modules/ajv/dist/2020.js';
import addFormats from '../../contracts/node_modules/ajv-formats/dist/index.js';

const read = (path) => JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));
const schema = read('../../contracts/schemas/v1.json');
const ajv = new Ajv2020({ allErrors: true, strict: true });
addFormats(ajv);
ajv.addSchema(schema);
const check = ajv.compile({ $ref: `${schema.$id}#/$defs/SkillNode` });
const nodes = read('./seed-v1.json').nodes;
for (const node of nodes) {
  assert.equal(check(node), true, `${node.id}: ${JSON.stringify(check.errors)}`);
}
assert.equal(check({ ...nodes[0], unexpected: true }), false);
assert.equal(check({ ...nodes[0], reviewStatus: 'released' }), false);
const manifestCheck = ajv.compile({ $ref: `${schema.$id}#/$defs/ContentManifest` });
assert.equal(manifestCheck(read('./manifest-v1.json')), true, JSON.stringify(manifestCheck.errors));
console.log('33 SkillNode@1 instances and ContentManifest@1 pass shared schemas; 2 invalid nodes rejected.');
