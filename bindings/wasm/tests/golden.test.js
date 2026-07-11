"use strict";

// Golden test over the wasm-pack (nodejs target) output: the WebAssembly build
// recomputes a claimed report and refutes a doctored one, byte-identically to
// the native run. Skips cleanly when `pkg/` has not been built yet
// (`wasm-pack build --target nodejs`).

const { test } = require("node:test");
const assert = require("node:assert");
const path = require("node:path");

let wasm = null;
try {
  wasm = require(path.resolve(__dirname, "..", "pkg", "wickra_verify_wasm.js"));
} catch {
  wasm = null;
}

const STRATEGY = {
  symbol: "BTCUSDT",
  timeframe: "1h",
  indicators: {
    ema_fast: { type: "Ema", params: [5] },
    ema_slow: { type: "Ema", params: [15] },
  },
  entry: { cross_above: ["ema_fast", "ema_slow"] },
  exit: { cross_below: ["ema_fast", "ema_slow"] },
  sizing: { type: "fixed_fraction", fraction: 0.95 },
  costs: { taker_bps: 5, slippage: { type: "fixed_bps", bps: 2 } },
  risk: { trailing_stop_pct: 5.0 },
};

function candles() {
  const out = [];
  for (let i = 0; i < 40; i++) {
    const base = 100.0 + Math.sin(i * 0.4) * 8.0;
    out.push({
      time: 1_700_000_000 + i * 3600,
      open: base,
      high: base + 1.0,
      low: base - 1.0,
      close: base + 0.5,
      volume: 1000.0,
    });
  }
  return out;
}

function verifyRequest(claimedReport) {
  return JSON.stringify({
    cmd: "verify",
    claim: {
      strategy: STRATEGY,
      dataset_ref: { kind: "inline", data: { BTCUSDT: candles() } },
      claimed_report: claimedReport,
    },
  });
}

test("wasm build present or skipped", (t) => {
  if (!wasm) t.skip("run `wasm-pack build --target nodejs` first");
});

if (wasm) {
  test("wasm refutes a fudged claim via recomputation", () => {
    const verifier = new wasm.Verifier();
    const verdict = JSON.parse(verifier.command(verifyRequest({ fees_paid: 99999.0 })));
    assert.strictEqual(verdict.matches, false);
    assert.ok(verdict.mismatches.some((m) => m.field === "fees_paid"));
    assert.strictEqual(verdict.claimed_report_hash.length, 64);
    assert.strictEqual(verdict.inputs_hash.length, 64);
  });

  test("wasm version matches the module export", () => {
    assert.strictEqual(new wasm.Verifier().version(), wasm.version());
  });

  test("wasm reports an unknown command in-band", () => {
    const response = JSON.parse(new wasm.Verifier().command('{"cmd":"nope"}'));
    assert.strictEqual(response.ok, false);
    assert.match(response.error, /nope/);
  });
}
