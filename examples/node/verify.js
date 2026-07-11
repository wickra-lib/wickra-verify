// A runnable Node.js example: submit a claim whose report has been doctored and
// assert the binding refutes it. Verification recomputes the report from
// (strategy, data) and compares, so a fabricated number cannot pass.
//
//   ( cd bindings/node && npm install && npm run build )
//   ( cd examples/node && npm install && node verify.js )

"use strict";

const assert = require("node:assert");
const { Verifier, version } = require("wickra-verify");

const STRATEGY = {
  symbol: "AAA",
  timeframe: "1h",
  indicators: {
    ema_fast: { type: "Ema", params: [3] },
    ema_slow: { type: "Ema", params: [8] },
  },
  entry: { cross_above: ["ema_fast", "ema_slow"] },
  exit: { cross_below: ["ema_fast", "ema_slow"] },
  sizing: { type: "fixed_fraction", fraction: 0.95 },
  costs: { taker_bps: 5, slippage: { type: "fixed_bps", bps: 2 } },
  risk: {},
};

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const CLOSES = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128];

const candles = () =>
  CLOSES.map((close, i) => {
    const open = i === 0 ? close : CLOSES[i - 1];
    return {
      time: 1_700_000_000 + i * 3600,
      open,
      high: Math.max(open, close) + 1,
      low: Math.min(open, close) - 1,
      close,
      volume: 1000,
    };
  });

const claim = {
  strategy: STRATEGY,
  dataset_ref: { kind: "inline", data: { AAA: candles() } },
  // A fabricated report: a claimant asserts an inflated fees figure.
  claimed_report: { fees_paid: 99999.0 },
};

const verifier = new Verifier();
const verdict = JSON.parse(
  verifier.command(JSON.stringify({ cmd: "verify", claim })),
);

console.log("wickra-verify", version());
assert.strictEqual(verdict.matches, false, "a doctored report must be refuted");
console.log(`doctored claim: REFUTED (mismatch: ${verdict.mismatches[0].field})`);
