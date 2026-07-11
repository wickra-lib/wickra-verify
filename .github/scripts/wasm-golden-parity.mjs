// WASM golden-parity check (run in CI after `wasm-pack build --target nodejs`).
//
// Verify every committed golden/claims/*.json through the WebAssembly core over
// the shared golden/data and assert the response is byte-identical to
// golden/expected/<claim>.json — the same canonical Verdict the native bindings
// produce. The wasm Verifier returns the core's canonical command_json string
// verbatim, so byte equality is the exact cross-language parity check. The honest
// claim confirms; every doctored claim is refuted.

import { createRequire } from "node:module";
import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..");

// wasm-pack --target nodejs emits a CommonJS module.
const { Verifier } = require(
  path.join(repo, "bindings", "wasm", "pkg-node", "wickra_verify_wasm.js"),
);

const G = path.join(repo, "golden");

// Load every golden/data/<SYMBOL>.csv into a symbol-keyed candle map.
function loadData() {
  const data = {};
  const dir = path.join(G, "data");
  if (!fs.existsSync(dir)) return data;
  for (const csv of fs.readdirSync(dir).filter((n) => n.endsWith(".csv"))) {
    const candles = [];
    const lines = fs.readFileSync(path.join(dir, csv), "utf8").split(/\r?\n/);
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

const data = loadData();
const claimNames = fs
  .readdirSync(path.join(G, "claims"))
  .filter((n) => n.endsWith(".json"))
  .sort();
assert.ok(claimNames.length > 0, "expected at least one golden claim");

const verifier = new Verifier();
let honestSeen = false;
for (const name of claimNames) {
  const claim = JSON.parse(fs.readFileSync(path.join(G, "claims", name), "utf8"));
  const envelope = { cmd: "verify", claim };
  if (Object.keys(data).length > 0) envelope.data = data;

  const got = verifier.command(JSON.stringify(envelope));
  const expected = fs
    .readFileSync(path.join(G, "expected", name), "utf8")
    .trim();
  assert.strictEqual(got.trim(), expected, `golden mismatch for ${name}`);

  const verdict = JSON.parse(got);
  if (name === "honest.json") {
    assert.strictEqual(verdict.matches, true, "honest golden claim must verify");
    honestSeen = true;
  } else {
    assert.strictEqual(verdict.matches, false, `${name} must be refuted`);
  }
}
assert.ok(honestSeen, "the honest golden claim must be present");

console.log(`wasm golden parity: ${claimNames.length} golden verdicts match`);
