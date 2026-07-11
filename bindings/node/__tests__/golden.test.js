"use strict";

// Golden cross-language test: every golden claim verifies to its expected
// verdict. The fixtures live in the repository-root `golden/` directory (shared
// by every binding). The suite is a no-op until the golden phase lands.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Verifier } = require("../index.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const CLAIMS = path.join(GOLDEN, "claims");
const EXPECTED = path.join(GOLDEN, "expected");
const DATA = path.join(GOLDEN, "data");

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

test("golden fixtures present or skipped", (t) => {
  if (files.length === 0) t.skip("golden fixtures not present yet");
});

for (const name of files) {
  test(`golden ${name} matches expected`, () => {
    const claim = JSON.parse(fs.readFileSync(path.join(CLAIMS, name), "utf-8"));
    const expected = JSON.parse(fs.readFileSync(path.join(EXPECTED, name), "utf-8"));
    const envelope = { cmd: "verify", claim };
    const data = loadData();
    if (Object.keys(data).length > 0) envelope.data = data;
    const verdict = JSON.parse(new Verifier().command(JSON.stringify(envelope)));
    assert.deepStrictEqual(verdict, expected);
  });
}
