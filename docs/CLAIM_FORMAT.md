# Claim format

A **claim** is the assertion `wickra-verify` checks: *this strategy, over this
data, produced this report*. It is a JSON (or TOML) object with exactly three
fields (`deny_unknown_fields` — any extra key is rejected):

```json
{
  "strategy": { /* a wickra-backtest StrategySpec */ },
  "dataset_ref": { "kind": "inline", "data": { "BTCUSDT": [ /* candles */ ] } },
  "claimed_report": { /* a wickra-backtest BacktestReport (untrusted) */ }
}
```

## `strategy` — a `StrategySpec`

The embedded [`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest)
`StrategySpec`, kept as raw JSON so verify-core stays decoupled from the engine's
struct internals across the FFI boundary. It carries the symbol, timeframe,
indicators, entry/exit rules, sizing, costs and risk. Example:

```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "indicators": {
    "ema_fast": { "type": "Ema", "params": [5] },
    "ema_slow": { "type": "Ema", "params": [15] }
  },
  "entry": { "cross_above": ["ema_fast", "ema_slow"] },
  "exit":  { "cross_below": ["ema_fast", "ema_slow"] },
  "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
  "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
  "risk": { "trailing_stop_pct": 5.0 }
}
```

It must be a JSON object (a scalar or array is rejected with `BadClaim`).

## `dataset_ref` — where the candles come from

An internally-tagged enum (`kind`):

- **`inline`** — the candles travel with the claim, keyed by symbol, so the claim
  is fully self-contained and portable:
  `{"kind":"inline","data":{"BTCUSDT":[{"time":…,"open":…,…}, …]}}`.
- **`files`** — the claim names the symbols it needs (and an optional content
  `hash` for provenance); the caller supplies the candles out of band (the CLI's
  `--data <dir>`, a binding's `data` argument). `{"kind":"files","symbols":["BTCUSDT"]}`.

A `files` claim carries no inline data; verification requires the data to be
supplied explicitly.

## `claimed_report` — a `BacktestReport` (untrusted)

The report the claimant asserts the run produced. It must be a JSON object, but
its contents are **never trusted**: verification recomputes the report from
`strategy` + data and compares. A `BacktestReport` contains `schema_version`,
`metrics` (`pnl`, `return_pct`, `sharpe`, `sortino`, `calmar`, `max_drawdown`,
`win_rate`, `profit_factor`, `num_trades`), `trades`, `equity`, `fees_paid` and
`initial_capital`.

## Parsing

`Claim::from_json` and `Claim::from_toml` parse and validate; both reject unknown
fields, a non-object strategy (`BadClaim`) and a non-object report (`BadReport`).
The resulting verdict is described in [VERDICT.md](VERDICT.md).
