# Golden fixtures

Frozen `(claim → verdict)` pairs that pin verify-core's determinism across the
Rust integration tests **and** every language binding. The product is the
verdict: the same claim over the same data must recompute to the same `Verdict`
— identical `matches`, `mismatches`, and blake3 hashes — in Rust, Python,
Node.js, WASM, C, C++, C#, Go, Java and R.

## Layout

- `data/BTCUSDT.csv` — the deterministic universe, one `ts,open,high,low,close,volume`
  row per bar, 240 bars. Every binding re-reads this file and supplies it as the
  verification data, so a doctored `claimed_report` is caught against the *real*
  recomputed report.
- `claims/*.json` — five `Claim`s over that data: one honest, four doctored.
  Each references the data by `dataset_ref: {"kind":"files","symbols":["BTCUSDT"]}`
  and is verified with the golden `data/` supplied out of band.
- `expected/*.json` — one blessed `Verdict` per claim: the exact canonical
  `verify` response, including the real `claimed_report_hash` /
  `actual_report_hash` / `inputs_hash`.

## Data formula

Each bar (`i = 0 … 239`) is a fixed function; `time[i] = 1_700_000_000 + i·3600`,
`volume = 1000`, `open[i]` is the previous close (`open[0] = close[0]`), and
`high`/`low` bracket the bar by one unit:

```
close[i] = 100 + 12·sin(i·0.10) + 6·sin(i·0.031)
open[i]  = close[i-1]        (open[0] = close[0])
high[i]  = max(open, close) + 1
low[i]   = min(open, close) − 1
```

Two blended sine waves so the fast/slow EMA cross repeatedly and the equity
curve draws down and recovers — enough structure for Sharpe, drawdown and the
trade log to be non-trivial. Floats are written in their shortest round-tripping
decimal form, so the CSV every binding re-reads parses back to exactly the
candles the fixtures were blessed from.

## The claims

The strategy is a fast/slow EMA cross (`Ema(5)` over `Ema(15)`) with taker fees,
slippage and a 5% trailing stop. `honest.json` carries the true report; each
doctored claim is derived from it by changing **one thing**, so the verdict
isolates exactly what a claimant tried to inflate:

| Claim              | Change vs. the honest report                              | Verdict                                   |
| ------------------ | --------------------------------------------------------- | ----------------------------------------- |
| `honest`           | none                                                       | `matches: true`, no mismatches            |
| `fudged_sharpe`    | `metrics.sharpe` + 1.0 (`0.3599035 → 1.3599035`)          | `matches: false`, 1 mismatch on `metrics.sharpe` |
| `fudged_return`    | `metrics.return_pct` + 25.0 (`50.999… → 75.999…`)         | `matches: false`, 1 mismatch on `metrics.return_pct` |
| `fudged_drawdown`  | `metrics.max_drawdown` − 0.1 (`4.0497 → 3.9497`)          | `matches: false`, 1 mismatch on `metrics.max_drawdown` |
| `wrong_data`       | summary `metrics` + `fees_paid` swapped for a different price series' | `matches: false`, mismatches across the metrics and fees — a report that cannot belong to this data |

Recomputation ignores the claimed numbers and re-runs the engine, so every fudge
is caught no matter how plausible it looks.

## Blessing

The fixtures are a pure function of `data/` plus the engine — **never edit them
by hand**. Regenerate them from the repository root:

```sh
cargo run -p verify-core --example bless_golden
```

The bless tool (`crates/verify-core/examples/bless_golden.rs`) writes the CSV,
runs the real `wickra-backtest` engine to get the honest report, derives the four
doctored claims from it, and blesses each expected `Verdict` through the same
`Verifier` command path the bindings drive. Regenerating after an intentional
engine or schema change is expected; an *unexplained* diff means determinism
broke and the change must be understood before committing.
