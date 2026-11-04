import { test } from "node:test";
import assert from "node:assert/strict";
import isNumber from "is-number";
test("embedded dependency behavior", () => assert.equal(isNumber(42), true));
