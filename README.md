<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-verify)

---

# Wickra Verify

**Verify any backtest. Paste a `(strategy, data, claimed report)` triple and get
a deterministic verdict — confirmed or refuted — that anyone can recompute in ten
languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib).** Built on the
> same deterministic backtest engine and ten-language binding surface as
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-proof](https://github.com/wickra-lib/wickra-proof) and the rest.

`wickra-verify` takes a **claim** — a strategy spec, the candle data it was run
over, and the `BacktestReport` someone says that run produced — and **recomputes
the report** with the pinned [`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest)
engine. It then compares the claimed report against the fresh one, field by
field, within an explicit tolerance, and returns a **verdict**: `verified: true`
if every metric matches, or a stable, sorted list of the exact mismatches if not.

It is a **free anti-fraud tool against doctored backtests** — not a hosted
service, not a SaaS, not a backend. A CLI plus ten language bindings, and an
optional **static** in-browser WASM demo. Nothing you paste ever leaves your
machine.

## Why it exists

A backtest report is just numbers in a screenshot until someone can reproduce it.
`wickra-verify` makes reproduction a one-command check: it does not trust the
claimed numbers, it recomputes them from the strategy and data and compares. A
fudged Sharpe, an inflated PnL, a quietly-changed parameter — all surface as a
concrete mismatch. The same core logic runs identically in Rust, Python, Node.js,
WASM, C, C++, C#, Go, Java and R, so a verdict reached in one language is
byte-identical everywhere.

## Determinism is the product

- **Recompute, never trust** — the verdict comes from re-running the backtest,
  not from comparing two supplied hashes.
- **Explicit tolerance** — floats are compared within a fixed tolerance, never
  bitwise, so legitimate last-ULP noise does not cause false refutals.
- **Stable, sorted mismatches** — the mismatch list is ordered by field so the
  verdict is byte-identical across languages and between the parallel (native)
  and sequential (WASM) code paths.
- **Engine-version bound** — the verdict records the `wickra-backtest` version it
  was reached under; a different engine produces a different, clearly-labelled
  result by design.

## Status

**Pre-release — under construction.** This repository is being built out to full
wickra-grade: the core, the CLI, all ten language bindings, a byte-exact golden
corpus, property + fuzz tests, benchmarks and one runnable example per language.
Track progress in [ROADMAP.md](ROADMAP.md).

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

`wickra-verify` is research and engineering tooling, not financial advice. A
verdict attests only that a claimed report is (or is not) the deterministic
result of a given strategy over given data — it makes no claim about the quality,
profitability or future performance of any strategy, nor about whether the data
itself is genuine. Trading carries risk; you are responsible for your own
decisions.
