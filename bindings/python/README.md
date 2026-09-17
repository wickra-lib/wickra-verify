<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/pypi.svg)](https://pypi.org/project/wickra-verify/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — Python

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Python. `pip install wickra-verify` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, field by field. A doctored `claimed_report` cannot pass,
because verification recomputes rather than trusting the supplied numbers.

## Install

```bash
pip install wickra-verify
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

## Quick start

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares, so this Python front-end drives the exact
same core as the native CLI.

```python
import json
from wickra_verify import Verifier

verifier = Verifier()  # default tolerances; Verifier('{"atol":1e-9,"rtol":1e-6}') to override

claim = {
    "strategy": {...},                       # a wickra-backtest StrategySpec
    "dataset_ref": {"kind": "inline", "data": {"BTCUSDT": [...]}},
    "claimed_report": {...},                 # the report being checked (untrusted)
}

verdict = json.loads(verifier.command(json.dumps({"cmd": "verify", "claim": claim})))
if verdict["matches"]:
    print("VERIFIED")
else:
    for m in verdict["mismatches"]:
        print(f"{m['field']}: claimed {m['claimed']}, actual {m['actual']}")
```

### Commands

| `cmd`          | Payload                     | Response                                  |
|----------------|-----------------------------|-------------------------------------------|
| `verify`       | `{claim, data?}`            | the full `Verdict`                        |
| `explain`      | `{verdict}`                 | `{"ok":true,"text":...}`                  |
| `canonicalize` | `{value}`                   | `{"ok":true,"canonical":...}`             |
| `version`      | —                           | `{"version":...,"engine_version":...}`    |

For `files`-kind claims, supply the candle data under a top-level `data` key
(`{symbol: [candle, ...]}`); `inline` claims carry their data already.

Domain errors (a bad claim, an unknown command) come back in-band as
`{"ok":false,"error":...}`. A malformed command envelope raises `ValueError`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/python)

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
