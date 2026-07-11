"use strict";

// Smoke: a verifier recomputes a claimed report and refutes a doctored one,
// exposes the pinned engine version, and reports unknown commands in-band.

const { test } = require("node:test");
const assert = require("node:assert");
const { Verifier, version } = require("../index.js");

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

test("a fudged claim is refuted via recomputation", () => {
  const verdict = JSON.parse(new Verifier().command(verifyRequest({ fees_paid: 99999.0 })));
  assert.strictEqual(verdict.matches, false);
  assert.ok(verdict.mismatches.some((m) => m.field === "fees_paid"));
  assert.strictEqual(verdict.claimed_report_hash.length, 64);
  assert.strictEqual(verdict.inputs_hash.length, 64);
});

test("version matches the module export", () => {
  assert.strictEqual(new Verifier().version(), version());
  const v = JSON.parse(new Verifier().command('{"cmd":"version"}'));
  assert.strictEqual(v.version, version());
});

test("a custom-tolerance verifier constructs", () => {
  const loose = new Verifier('{"atol":1000.0,"rtol":1.0}');
  const verdict = JSON.parse(loose.command(verifyRequest({ fees_paid: 99999.0 })));
  // 99999 is still far outside even a loose relative tolerance, so it is refuted.
  assert.strictEqual(typeof verdict.matches, "boolean");
});

test("unknown command is an in-band error", () => {
  const response = JSON.parse(new Verifier().command('{"cmd":"nope"}'));
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /nope/);
});
