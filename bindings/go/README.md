<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-verify-go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — Go

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Go. `go get github.com/wickra-lib/wickra-verify-go` — over the C ABI via cgo, prebuilt library bundled in the module.**

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, from Go over the C ABI hub (cgo). A doctored
`claimed_report` cannot pass, because verification recomputes rather than
trusting the supplied numbers.

## Install

Use the published **`wickra-verify-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-verify-go
```

`wickra-verify-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_verify.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

The binding links the prebuilt C ABI library, staged per platform under
`lib/<goos>_<goarch>/`, with the header vendored under `include/`. Building from
source requires a C toolchain (cgo) and the staged native library.

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-verify-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_verify.so bindings/go/lib/linux_amd64/
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```go
package main

import (
    "encoding/json"
    "fmt"

    wickra "github.com/wickra-lib/wickra-verify-go"
)

func main() {
    v := wickra.New()
    defer v.Close()

    claim := map[string]any{
        "strategy":       strategySpec,                     // a wickra-backtest StrategySpec
        "dataset_ref":    map[string]any{"kind": "inline", "data": data},
        "claimed_report": report,                           // the report being checked (untrusted)
    }
    cmd, _ := json.Marshal(map[string]any{"cmd": "verify", "claim": claim})
    out, err := v.Command(string(cmd))
    if err != nil {
        panic(err)
    }
    fmt.Println(out) // the full Verdict as JSON
}
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
`{ok:false,error:...}`; only null/UTF-8/panic conditions produce a Go `error`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/go)

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
