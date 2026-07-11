<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-verify)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codeql.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-verify)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/best-practices.svg)](https://www.bestpractices.dev/)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/provenance.svg)](https://github.com/wickra-lib/wickra-verify/attestations)
[![Verified across 10 languages](https://img.shields.io/badge/verified%20across-10%20languages-3b82f6)](#use-in-any-language)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/docs.svg)](https://wickra.org)

---

# Wickra Verify

**Verify any backtest. Hand over a `(strategy, data, claimed report)` triple and
get a deterministic verdict — confirmed or refuted — that anyone can recompute in
ten languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib).** Built on the
> same deterministic backtest engine and ten-language binding surface as
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-proof](https://github.com/wickra-lib/wickra-proof) and the rest.

`wickra-verify` takes a **claim** — a strategy spec, the candle data it was run
over, and the `BacktestReport` someone says that run produced — and **recomputes
the report** with the pinned [`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest)
engine. It then compares the claimed report against the fresh one, field by
field, within an explicit tolerance, and returns a **verdict**: `matches: true`
if every metric agrees, or a stable, sorted list of the exact mismatches if not.

It is a **free anti-fraud tool against doctored backtests** — not a hosted
service, not a SaaS, not a backend. A CLI plus ten language bindings, and an
optional **static** in-browser WASM demo. Nothing you submit ever leaves your
machine.

## Determinism is the product

- **Recompute, never trust** — the verdict comes from re-running the backtest,
  not from comparing two supplied numbers. A fudged Sharpe, an inflated PnL, a
  quietly-changed parameter all surface as a concrete mismatch.
- **Explicit tolerance** — floats are compared within a fixed
  `atol + rtol·max(|a|,|b|)` tolerance (the `numpy.allclose` rule), never
  bitwise, so legitimate last-ULP noise never causes a false refutal.
- **Stable, sorted mismatches** — the mismatch list is ordered by field, so the
  verdict is byte-identical across all ten languages.
- **Canonical hashes** — every verdict carries the blake3 hashes of the claimed
  report, the recomputed report and the full inputs, under the same
  canonicalization [`wickra-proof`](https://github.com/wickra-lib/wickra-proof)
  uses; a verdict's `inputs_hash` equals the proof hash of the same inputs.

## Status

**Pre-release — functionally complete, CI-verified, not yet published.** The
core, the CLI, all ten language bindings, the byte-exact golden corpus, the
property + fuzz suites, the benchmarks and one runnable example per language are
built and green across Linux, macOS and Windows. Packages are not yet on the
registries. Track progress in [ROADMAP.md](ROADMAP.md).

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — how the pieces fit together.
- [`docs/CLAIM_FORMAT.md`](docs/CLAIM_FORMAT.md) — the `Claim` and its embedded
  `StrategySpec` / `BacktestReport`.
- [`docs/VERDICT.md`](docs/VERDICT.md) — the `Verdict`, mismatches and tolerance.
- [`docs/CANONICALIZATION.md`](docs/CANONICALIZATION.md) — the hashing contract
  shared with wickra-proof.
- [`docs/DETERMINISM.md`](docs/DETERMINISM.md) — why the verdict is identical
  everywhere.
- [`docs/Cookbook.md`](docs/Cookbook.md) — recipes, including "gate a claim in
  CI: exit 2 = fraud".

## Quickstart

```bash
# Verify a claim: recompute the report and compare it, field by field.
# --claim is a JSON/TOML Claim; --data a CSV or a directory of <SYMBOL>.csv files
# (omit --data when the claim carries its candles inline).
cargo run -p wickra-verify -- \
  --claim examples/data/claims/fudged.json \
  --data examples/data/candles

# Exit 0 = verified, 2 = refuted (CI-friendly), 1 = error.
# --explain renders the mismatches and forces exit 0.
```

The example claim inflates `fees_paid`, so the verdict is **refuted** and names
the `fees_paid` mismatch — a fabricated number cannot pass a recomputation.

## Claim format

A **claim** is the assertion to be checked — "this strategy on this data produced
this report":

- **`strategy`** — the embedded [`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest)
  `StrategySpec` (indicators, entry/exit rules, sizing, costs, risk).
- **`dataset_ref`** — where the candles come from: `inline` (embedded per symbol,
  fully self-contained) or `files` (named symbols resolved out of band).
- **`claimed_report`** — the `BacktestReport` the claimant asserts this run
  produced. **Untrusted**: verification recomputes and compares, so a doctored
  `claimed_report` cannot pass.

Full schema in [`docs/CLAIM_FORMAT.md`](docs/CLAIM_FORMAT.md).

## Verdict and tolerance

`verify` returns a `Verdict`:

- **`matches`** — `true` iff every compared field agrees within tolerance.
- **`mismatches`** — a field-sorted list of `{field, claimed, actual, delta}`.
- **`claimed_report_hash` / `actual_report_hash` / `inputs_hash`** — 64-hex
  blake3 digests over the canonical forms.
- **`engine_version`** — the `wickra-backtest` version the verdict was reached
  under.

Two floats count as equal when `|a − b| ≤ atol + rtol·max(|a|,|b|)`, with tight
defaults (`atol = 1e-9`, `rtol = 1e-6`) that absorb last-bit float noise and
nothing more. See [`docs/VERDICT.md`](docs/VERDICT.md).

## Canonicalization and hashing

The hashes are only as trustworthy as the serialization they run over, so
canonicalization is byte-for-byte the contract
[`wickra-proof`](https://github.com/wickra-lib/wickra-proof) uses (see
[`crates/verify-core/src/canon.rs`](crates/verify-core/src/canon.rs)): keys
sorted by code point, no structural whitespace, floats quantized to `1e-8` with
trailing zeros trimmed and whole values collapsed to integers, no `NaN`/`±inf`.
`blake3` over that canonical string yields each 64-hex hash. A verdict's
`inputs_hash` therefore equals the wickra-proof hash of the same inputs — the two
products share one determinism moat. See
[`docs/CANONICALIZATION.md`](docs/CANONICALIZATION.md).

## Anti-fraud use in CI

Because a refuted claim exits `2`, verification drops straight into a pipeline:

```bash
# Fails the job (exit 2) if the committed report does not match a fresh run.
wickra-verify --claim claim.json --data candles/
```

Wire this into a strategy repo's CI and a doctored `claimed_report` can never be
merged — the recomputation is the gate. Recipes in
[`docs/Cookbook.md`](docs/Cookbook.md).

## Use in any language

The core is a JSON-over-C-ABI data API (`Verifier::command`) exposed natively in
Rust, Python, Node.js and WASM, and over the C ABI hub in C, C++, C#, Go, Java
and R. Every binding drives the same `verify` / `explain` / `canonicalize` /
`version` commands and returns the core's canonical response verbatim; the
cross-language golden tests assert byte-for-byte equality. One runnable example
per language lives under [`examples/`](examples); per-binding quickstarts are in
each `bindings/<lang>/README.md`.

| Language | Binding | Package |
| -------- | ------- | ------- |
| Rust | `verify-core` (native) | crates.io |
| Python | PyO3 (native) | PyPI |
| Node.js | napi (native) | npm |
| WASM | wasm-bindgen (native) | npm |
| C / C++ | C ABI | header + library |
| C# | C ABI (P/Invoke) | NuGet |
| Go | C ABI (cgo) | Go module |
| Java | C ABI (FFM/Panama) | Maven |
| R | C ABI (`.Call`) | R-universe |

## Project layout

```
crates/verify-core          the library: claim + compare + canonicalize + verify
crates/wickra-verify-cli    reference CLI (verify), binary `wickra-verify`
crates/verify-bench         Criterion benchmarks
bindings/{c,python,node,wasm,go,csharp,java,r}   ten-language surface
golden/                     fixed claims -> expected verdicts (byte-exact)
examples/                   runnable per-language demos + static web demo
fuzz/                       cargo-fuzz targets (claim parse, compare, canonicalize, verify)
```

## Building from source

```bash
cargo build --workspace
cargo test --workspace --all-features
```

Each binding builds with its own toolchain; see `bindings/<lang>/README.md`. The
C-ABI consumers (C/C++, C#, Go, Java, R) need the C ABI library first:
`cargo build --release -p wickra-verify-c`.

## Requirements

Rust **1.86** (workspace) / **1.88** (Node binding). Per-binding toolchains:
Python 3.9+, Node.js 22+, .NET 8, JDK 22+, Go 1.23+, R release, and a C11/C++14
compiler with CMake for the C example.

## Benchmarks

Criterion benchmarks for `verify`, `compare` and `canonicalize` live in
`crates/verify-bench`; numbers and methodology are in
[BENCHMARKS.md](BENCHMARKS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Every change runs the full CI matrix (all
ten languages × three OSes) plus CodeQL, Scorecard and zizmor.

## Security

Report vulnerabilities per [SECURITY.md](SECURITY.md). The threat model is in
[THREAT_MODEL.md](THREAT_MODEL.md).

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

`wickra-verify` is research and engineering tooling, not financial advice. A
verdict attests only that a claimed report is (or is not) the deterministic
result of a given strategy over given data — it makes no claim about the quality,
profitability or future performance of any strategy, nor about whether the data
itself is genuine. Trading carries risk; you are responsible for your own
decisions. `wickra-verify` is free software you run yourself: no hosted service,
no data collection, no warranty.
