# Verdict and tolerance

`verify(claim, data)` returns a `Verdict`:

```json
{
  "matches": false,
  "mismatches": [
    { "field": "metrics.sharpe", "claimed": 1.3599035, "actual": 0.3599035, "delta": -1 }
  ],
  "engine_version": "0.1.0",
  "claimed_report_hash": "bdd72fcb…",
  "actual_report_hash": "97ccadfd…",
  "inputs_hash": "5f01450b…"
}
```

## Fields

- **`matches`** — `true` iff there are zero mismatches.
- **`mismatches`** — a list of `{field, claimed, actual, delta}`, **sorted by
  `field`** so the verdict is byte-identical everywhere. `field` is a dotted path
  (`metrics.sharpe`); array divergence is reported as `field[len]`.
- **`engine_version`** — the `wickra-backtest` version the verdict was reached
  under (recorded, never hidden).
- **`claimed_report_hash` / `actual_report_hash` / `inputs_hash`** — 64-hex
  blake3 digests over the canonical forms. For an honest claim the first two are
  equal; `inputs_hash` is the blake3 of `{strategy, dataset_ref, candles,
  engine_version}` and equals the wickra-proof hash of the same inputs.

## Tolerance

Two floating-point leaves count as equal when

```
|a − b| ≤ atol + rtol · max(|a|, |b|)
```

— the mixed absolute/relative rule `numpy.allclose` uses. The defaults are
deliberately tight: `atol = 1e-9`, `rtol = 1e-6`. They absorb the last-bit float
noise of two independent-but-deterministic computations of the *same* report and
nothing more; a doctored metric moves far more than `1e-6` relative. Override via
the CLI's `--atol` / `--rtol` or a binding `Config`.

A pure absolute tolerance would reject large-magnitude metrics that differ only
in the last representable digit; a pure relative one would reject values
legitimately near zero — hence the mixed rule.

## `explain`

`explain(verdict)` renders a deterministic one-shot summary:

```
REFUTED: 1 field(s) disagree with the recomputed report (engine 0.1.0):
  metrics.sharpe: claimed 1.3599035, actual 0.3599035 (delta -1)
```

An honest verdict renders `VERIFIED: claimed report matches the recomputed
report (engine 0.1.0).`

## Exit codes (CLI)

`wickra-verify` exits **0** when verified, **2** when refuted (a CI-friendly
failure), and **1** on error. `--explain` renders the mismatches and forces exit
`0`. See [Cookbook.md](Cookbook.md) for the CI-gate pattern.
