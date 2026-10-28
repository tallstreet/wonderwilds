import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  freshState,
  restoreProgress,
  readProgress,
  writeProgress,
} from "../../../frontend/src/progress.ts";

const raw = (name: string) => readFileSync(new URL(name, import.meta.url), "utf8");

test("legacy valid save fixtures round trip without losing story, cosmetics or evidence", () => {
  for (const name of ["fresh-v1.json", "completed-v1.json"]) {
    const input = raw(name);
    assert.deepEqual(restoreProgress(input), JSON.parse(input));
  }
});

test("malformed and contradictory saves have the existing safe recovery semantics", () => {
  assert.deepEqual(restoreProgress(raw("malformed-save.txt")), freshState());
  const restored = restoreProgress(raw("invalid-v1.json"));
  assert.equal(restored.hero, "rose");
  assert.equal(restored.region, "meadow");
  assert.deepEqual(restored.done, ["meadow-bridge"]);
  assert.deepEqual(restored.evidence, []);
  assert.deepEqual(restored.position, freshState().position);
});

test("blocked storage retains an in-memory fresh game and reports writes unavailable", () => {
  const fixture = JSON.parse(raw("storage-blocked.json"));
  const storage = {
    getItem(): string {
      throw new Error(fixture.getItemThrows);
    },
    setItem(): void {
      throw new Error(fixture.setItemThrows);
    },
  };
  const read = readProgress(storage);
  assert.equal(read.available, fixture.expectedAvailable);
  assert.deepEqual(read.state, freshState());
  assert.equal(writeProgress(freshState(), storage), fixture.expectedWriteSuccess);
});

test("captured artwork bytes match the migration baseline inventory", () => {
  const inventory = JSON.parse(raw("asset-inventory.json"));
  assert.equal(inventory.assets.length, 10);
  for (const asset of inventory.assets) {
    const bytes = readFileSync(new URL(`../../../${asset.path}`, import.meta.url));
    assert.equal(bytes.length, asset.bytes, asset.path);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), asset.sha256, asset.path);
  }
});
