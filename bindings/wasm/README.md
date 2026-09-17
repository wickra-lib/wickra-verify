<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/npm.svg)](https://www.npmjs.com/package/wickra-verify-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — WASM

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for WASM. `npm install wickra-verify-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, compiled to WebAssembly for the browser (and any WASM
host). A doctored `claimed_report` cannot pass, because verification recomputes
rather than trusting the supplied numbers. Built with [wasm-bindgen].

The backtest engine runs sequentially under WASM (no thread pool in a browser
sandbox), which is byte-identical to the native run — the exact cross-language
golden check.

## Install

```bash
npm install wickra-verify-wasm
```

### Building from this repository (contributors)

```sh
wasm-pack build --target web      # for browsers / bundlers
wasm-pack build --target nodejs   # for Node.js
```

The `pkg/` output (the `.wasm` binary plus the JS glue and TypeScript types) is
generated, not committed.

## Quick start

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```js
import init, { Verifier } from "wickra-verify-wasm";

await init(); // load the .wasm module (web target)

const verifier = new Verifier(); // default tolerances; new Verifier('{"atol":1e-9,"rtol":1e-6}') to override

const claim = {
  strategy: {/* a wickra-backtest StrategySpec */},
  dataset_ref: { kind: "inline", data: { BTCUSDT: [/* candles */] } },
  claimed_report: {/* the report being checked (untrusted) */},
};

const verdict = JSON.parse(verifier.command(JSON.stringify({ cmd: "verify", claim })));
console.log(verdict.matches ? "VERIFIED" : verdict.mismatches);
```

### Commands

| `cmd`          | Payload            | Response                                |
|----------------|--------------------|-----------------------------------------|
| `verify`       | `{claim, data?}`   | the full `Verdict`                      |
| `explain`      | `{verdict}`        | `{ok:true,text:...}`                    |
| `canonicalize` | `{value}`          | `{ok:true,canonical:...}`              |
| `version`      | —                  | `{version:...,engine_version:...}`     |

For `files`-kind claims, supply the candle data under a top-level `data` key;
`inline` claims carry their data already.

Domain errors (a bad claim, an unknown command) come back in-band as
`{ok:false,error:...}`; a malformed command envelope throws a JS error.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/wasm)

Wickra Verify ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-verify/blob/main/SECURITY.md>.

## Disclaimer

`wickra-verify` is research and engineering tooling, not financial advice. A
verdict attests only that a claimed report is (or is not) the deterministic
result of a given strategy over given data — it makes no claim about the quality,
profitability or future performance of any strategy, nor about whether the data
itself is genuine. Trading carries risk; you are responsible for your own
decisions. `wickra-verify` is free software you run yourself: no hosted service,
no data collection, no warranty.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-verify/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-verify/blob/main/LICENSE-MIT) at your option.
