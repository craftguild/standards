const { test } = require("node:test");
const assert = require("node:assert/strict");
const ts = require("typescript");
const { classify } = require("../npm-tests.cjs");

test("comments, literals and regular methods are implementation", () => {
  const source = '// test("name")\nconst text = "describe()";\nconst ok = /value/.test(text);';
  assert.deepEqual(classify(source, "product.ts", ts), { mixed: false });
});
test("test calls, nesting and named aliases require an unmeasured result", () => {
  for (const source of ['test("name", () => {});', 'describe("name", () => it.skip("case", () => {}));',
    'import { test as scenario } from "node:test"; scenario("case", () => {});',
    'import * as runner from "node:test"; runner.test("case", () => {});',
    'const { test: scenario } = require("node:test"); scenario("case", () => {});']) {
    assert.deepEqual(classify(source, "product.ts", ts), { mixed: true });
  }
});
test("invalid TypeScript is not silently counted", () => {
  assert.throws(() => classify("function broken(", "product.ts", ts));
});
