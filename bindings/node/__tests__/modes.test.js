"use strict";

// Operating-mode equivalence: a claim verifies the same whichever way its data
// arrives. A claim names its candles by reference (`dataset_ref.kind ===
// "files"`, the data supplied with the `verify` command) or inline
// (`dataset_ref.kind === "inline"`, embedded in the claim). The verdict must
// not depend on which: same matches, mismatches, report hashes and engine.
// Only `inputs_hash` may differ, because it binds the dataset *reference* --
// and it must differ, or the reference was not hashed.
//
// The golden claims are `files` claims; each is re-issued inline here.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Verifier } = require("../index.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const CLAIMS = path.join(GOLDEN, "claims");
const DATA = path.join(GOLDEN, "data");

const SAME = ["matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version"];

function claimFiles() {
  if (!fs.existsSync(CLAIMS)) return [];
  return fs
    .readdirSync(CLAIMS)
    .filter((f) => f.endsWith(".json"))
    .sort();
}

function loadData() {
  const data = {};
  if (!fs.existsSync(DATA)) return data;
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

test("mode fixtures present or skipped", (t) => {
  if (files.length === 0) t.skip("golden fixtures not present yet");
});

for (const name of files) {
  test(`${name}: supplied and inline data verify alike`, () => {
    const claim = JSON.parse(fs.readFileSync(path.join(CLAIMS, name), "utf-8"));
    assert.strictEqual(claim.dataset_ref.kind, "files", "golden claims reference their data");
    const data = loadData();

    const supplied = JSON.parse(new Verifier().command(JSON.stringify({ cmd: "verify", claim, data })));

    const inlineData = {};
    for (const symbol of claim.dataset_ref.symbols) inlineData[symbol] = data[symbol];
    const inlineClaim = { ...claim, dataset_ref: { kind: "inline", data: inlineData } };
    const inline = JSON.parse(new Verifier().command(JSON.stringify({ cmd: "verify", claim: inlineClaim })));

    for (const field of SAME) assert.deepStrictEqual(inline[field], supplied[field], field);
    assert.notStrictEqual(inline.inputs_hash, supplied.inputs_hash, "inputs_hash binds the dataset reference");
  });
}
