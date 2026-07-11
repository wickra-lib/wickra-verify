# Threat Model

`wickra-verify` is an anti-fraud tool: it recomputes a claimed backtest report
and confirms or refutes it. This document states what it defends against, what it
does not, and the trust boundaries.

## Assets

- The **integrity of the verdict**: a `verified: true` must mean the claimed
  report genuinely is the deterministic result of the claimed strategy over the
  claimed data, under the pinned engine version.
- **Determinism**: the verdict must be byte-identical across all ten languages
  and between the native and WASM code paths.

## Actors

- **Honest verifier** -- pastes a claim to check it; wants a trustworthy verdict.
- **Fraudulent claimant** -- publishes a doctored report and hopes it passes.
- **Untrusted input** -- claim JSON and candle data of arbitrary shape/size.

## Core threat: a doctored report passing verification

The central threat is a fabricated report -- hand-edited metrics, an inflated
Sharpe or PnL, a quietly-altered parameter -- being accepted. This is defeated by
**recomputation**: the verdict is derived from re-running the backtest from the
claimed strategy and data, not from trusting the supplied numbers or any supplied
hash. Any deviation beyond the explicit tolerance surfaces as a concrete
mismatch. There is no code path that accepts a claimed metric without recomputing
it.

## Trust boundaries and non-goals

- **Data provenance is out of scope.** A verdict attests that a report follows
  deterministically from *the data the verifier supplied* -- not that the data is
  genuine market data. A claimant who also fabricates the candle series is
  outside what verification can detect; use a trusted data source.
- **No hosted service, no accounts, no keys.** There is no upload backend, no
  telemetry, no server-side data acceptance. The CLI and bindings run entirely
  locally; the optional web demo runs entirely in the browser. There is no
  secret to steal and no endpoint to attack.
- **No financial claim.** A verdict says nothing about a strategy's quality or
  future performance.

## Attack surface

The only untrusted input is the parsing of the `Claim` (strategy + claimed
report) and candle data JSON, plus the recomputation itself. `serde_json` rejects
`NaN`/`±inf` at parse time; the core contains no `unsafe` (the C ABI hub
re-allows it locally only to cross the FFI boundary) and no unbounded allocation
driven directly by attacker-controlled counts beyond what the backtest engine
itself bounds.

## Reporting

See [SECURITY.md](SECURITY.md).
