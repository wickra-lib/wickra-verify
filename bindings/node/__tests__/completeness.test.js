"use strict";

// Parity guard: the Node binding must expose the full public surface of the
// verifier, so an export dropped in a refactor fails loudly here (mirrors the
// completeness checks in the Python and R bindings).

const { test } = require("node:test");
const assert = require("node:assert");
const wickra = require("../index.js");

// The napi-rs loader adds its own diagnostics to the module object
// (`__napiBindingTarget` since @napi-rs/cli 3.10); they belong to the loader,
// not to the binding's surface, and are left out of the comparison.
const exported = () =>
  Object.keys(wickra)
    .filter((name) => !name.startsWith("__"))
    .sort();

test("module exposes Verifier and version", () => {
  assert.strictEqual(typeof wickra.Verifier, "function");
  assert.strictEqual(typeof wickra.version, "function");
});

test("Verifier exposes command and version", () => {
  for (const name of ["command", "version"]) {
    assert.strictEqual(
      typeof wickra.Verifier.prototype[name],
      "function",
      `Verifier is missing ${name}`,
    );
  }
});

test("module surface is exactly {Verifier, version}", () => {
  assert.deepStrictEqual(exported(), ["Verifier", "version"]);
});

test("Verifier surface is exactly {command, version}", () => {
  const methods = Object.getOwnPropertyNames(wickra.Verifier.prototype)
    .filter((name) => name !== "constructor")
    .sort();
  assert.deepStrictEqual(methods, ["command", "version"]);
});
