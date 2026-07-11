// Browser demo wiring: load the wasm verifier, prefill an example claim, and
// verify it on demand — all client-side. Build the pkg first (see README.md):
//
//   ( cd bindings/wasm && wasm-pack build --target web --out-dir ../../examples/web/pkg )
//
// then serve this directory over http (ES modules do not load from file://):
//
//   python -m http.server -d examples/web

import init, { Verifier } from "./pkg/wickra_verify_wasm.js";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const CLOSES = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128];
const candles = CLOSES.map((close, i) => {
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

// A claim whose report has been doctored: an inflated fees figure. Verification
// recomputes the real report and refutes it.
const EXAMPLE_CLAIM = {
  strategy: {
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
  },
  dataset_ref: { kind: "inline", data: { AAA: candles } },
  claimed_report: { fees_paid: 99999.0 },
};

const $ = (id) => document.getElementById(id);

function show(ok, message) {
  const badge = $("badge");
  badge.textContent = ok ? "VERIFIED" : "REFUTED";
  badge.className = ok ? "ok" : "bad";
  $("output").textContent = message;
}

async function main() {
  await init();

  const verifier = new Verifier();
  const info = JSON.parse(verifier.command(JSON.stringify({ cmd: "version" })));
  $("version").textContent = `v${info.version} (engine ${info.engine_version})`;
  $("claim").value = JSON.stringify(EXAMPLE_CLAIM, null, 2);

  $("verify").addEventListener("click", () => {
    let claim;
    try {
      claim = JSON.parse($("claim").value);
    } catch (err) {
      show(false, `Invalid JSON: ${err.message}`);
      return;
    }

    let response;
    try {
      response = verifier.command(JSON.stringify({ cmd: "verify", claim }));
    } catch (err) {
      show(false, `Command failed: ${err.message}`);
      return;
    }

    const verdict = JSON.parse(response);
    if (verdict.ok === false) {
      show(false, verdict.error);
      return;
    }

    show(verdict.matches, JSON.stringify(verdict, null, 2));
  });
}

main().catch((err) => {
  $("output").textContent = `Failed to load the verifier: ${err.message}`;
});
