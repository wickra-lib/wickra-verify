<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/nuget.svg)](https://www.nuget.org/packages/Wickra.Verify)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — C#

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for C#. `dotnet add package Wickra.Verify` — prebuilt native library, no system dependencies.**

The wickra-verify core for .NET, over the C ABI via P/Invoke. The native library ships
inside the NuGet package for every supported runtime identifier, so there is
nothing to install alongside it.

## Install

```bash
dotnet add package Wickra.Verify
```

The native library ships prebuilt per platform under `runtimes/<rid>/native/`,
selected automatically. There is nothing to compile. Targets .NET 8 and later.

```sh
dotnet add package WickraVerify
```

### Building from this repository (contributors)

Requires the .NET SDK and a Rust toolchain:

```sh
cargo build -p wickra-verify-c --release
dotnet test bindings/csharp
```

The test project resolves the freshly built native library from `target/release`.

## Quick start

The binding is a thin, faithful surface over the same command boundary every
other binding drives, so a request built here produces the same canonical bytes
it would in Rust, Python or Go.

```csharp
using WickraVerify;

using var handle = new Verifier();
string response = handle.Command("""{"cmd":"version"}""");
Console.WriteLine(response);
```

`Verifier` owns a native handle and implements `IDisposable`; the `using` above is
what releases it. Dropping the reference without disposing leaks the handle
until the finalizer runs.

### What travels with the package

The managed assembly, the native C ABI library for each supported runtime
identifier, and both licence texts. The package declares `MIT OR Apache-2.0`
and carries `LICENSE-MIT` and `LICENSE-APACHE` to match.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of `[LibraryImport]` P/Invoke over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/csharp/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/csharp)

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
