"use strict";

// Tests over the wasm-pack (nodejs target) output, run by the WASM job after
// `wasm-pack build --target nodejs --out-dir pkg-node`:
//
//   golden   every golden/claims/*.json verifies, through the WebAssembly core
//            over the shared golden/data, to the byte-identical
//            golden/expected/<claim>.json the native bindings produce;
//   operating modes
//            a claim verifies the same whichever way its data arrives --
//            supplied with the command (`dataset_ref.kind === "files"`) or
//            embedded in the claim (`kind === "inline"`); only `inputs_hash`
//            differs, because it binds the dataset reference;
//   smoke    a doctored claim is refuted, the version matches the module
//            export, an unknown command is an in-band error.
//
// The require is hard: a missing build must fail the job, not skip it.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Verifier, version } = require("../pkg-node/wickra_verify_wasm.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const CLAIMS = path.join(GOLDEN, "claims");
const EXPECTED = path.join(GOLDEN, "expected");
const DATA = path.join(GOLDEN, "data");

const SAME = ["matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version"];

function claimFiles() {
  return fs
    .readdirSync(CLAIMS)
    .filter((f) => f.endsWith(".json"))
    .sort();
}

function loadData() {
  const data = {};
  for (const csv of fs.readdirSync(DATA).filter((f) => f.endsWith(".csv"))) {
    const candles = [];
    const lines = fs.readFileSync(path.join(DATA, csv), "utf-8").split(/\r?\n/);
    lines.forEach((line, idx) => {
      const trimmed = line.trim();
      if (!trimmed) return;
      const cols = trimmed.split(",").map((c) => c.trim());
      const time = Number.parseInt(cols[0], 10);
      if (Number.isNaN(time)) {
        if (idx === 0) return; // header row
        throw new Error(`bad timestamp in ${csv}: ${cols[0]}`);
      }
      candles.push({
        time,
        open: Number(cols[1]),
        high: Number(cols[2]),
        low: Number(cols[3]),
        close: Number(cols[4]),
        volume: Number(cols[5]),
      });
    });
    data[path.basename(csv, ".csv")] = candles;
  }
  return data;
}

const files = claimFiles();
const data = loadData();

test("golden fixtures are present", () => {
  assert.ok(files.length > 0, "golden/claims holds at least one claim");
});

for (const name of files) {
  const claim = JSON.parse(fs.readFileSync(path.join(CLAIMS, name), "utf-8"));

  test(`golden ${name} matches expected`, () => {
    const expected = fs.readFileSync(path.join(EXPECTED, name), "utf-8").trim();
    const got = new Verifier().command(JSON.stringify({ cmd: "verify", claim, data }));
    assert.strictEqual(got, expected);
  });

  test(`${name}: supplied and inline data verify alike`, () => {
    assert.strictEqual(claim.dataset_ref.kind, "files", "golden claims reference their data");
    const supplied = JSON.parse(new Verifier().command(JSON.stringify({ cmd: "verify", claim, data })));

    const inlineData = {};
    for (const symbol of claim.dataset_ref.symbols) inlineData[symbol] = data[symbol];
    const inlineClaim = { ...claim, dataset_ref: { kind: "inline", data: inlineData } };
    const inline = JSON.parse(new Verifier().command(JSON.stringify({ cmd: "verify", claim: inlineClaim })));

    for (const field of SAME) assert.deepStrictEqual(inline[field], supplied[field], field);
    assert.notStrictEqual(inline.inputs_hash, supplied.inputs_hash, "inputs_hash binds the dataset reference");
  });
}

test("a doctored claim is refuted via recomputation", () => {
  const name = files.find((f) => f.startsWith("fudged")) ?? files[0];
  const claim = JSON.parse(fs.readFileSync(path.join(CLAIMS, name), "utf-8"));
  claim.claimed_report.fees_paid = 99999.0;
  const verdict = JSON.parse(new Verifier().command(JSON.stringify({ cmd: "verify", claim, data })));
  assert.strictEqual(verdict.matches, false);
  assert.ok(verdict.mismatches.some((m) => m.field === "fees_paid"));
  assert.strictEqual(verdict.claimed_report_hash.length, 64);
  assert.strictEqual(verdict.inputs_hash.length, 64);
});

test("the version matches the module export", () => {
  assert.strictEqual(new Verifier().version(), version());
});

test("an unknown command is an in-band error", () => {
  const response = JSON.parse(new Verifier().command('{"cmd":"nope"}'));
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /nope/);
});
