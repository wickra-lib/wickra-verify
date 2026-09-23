<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-verify)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — Java

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Java. `org.wickra:wickra-verify` — prebuilt native library inside the jar, no JNI, no system dependencies.**

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, on the JVM over the Wickra C ABI via the Foreign Function
& Memory API (FFM/Panama, JDK 22+). A doctored `claimed_report` cannot pass,
because verification recomputes rather than trusting the supplied numbers.

## Requirements

- **Java 22 or later** (the FFM API is final since Java 22; no preview flag).
- The FFM API is *restricted*: pass `--enable-native-access=ALL-UNNAMED` when you
  run your application to silence the native-access warning.

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-verify</artifactId>
  <version>0.1.4</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-verify:0.1.4")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

### Building from this repository (contributors)

```sh
cargo build -p wickra-verify-c        # build the native C ABI library
mvn -Dnative.lib.dir=../../target/debug test
```

## Quick start

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```java
import org.wickra.verify.Verifier;

try (Verifier verifier = new Verifier()) {
    String claim = "{"
        + "\"strategy\":" + strategySpec + ","          // a wickra-backtest StrategySpec
        + "\"dataset_ref\":{\"kind\":\"inline\",\"data\":" + data + "},"
        + "\"claimed_report\":" + report                // the report being checked (untrusted)
        + "}";
    String verdict = verifier.command("{\"cmd\":\"verify\",\"claim\":" + claim + "}");
    System.out.println(verdict); // the full Verdict as JSON
}
```

FFM needs native access enabled at runtime:

```sh
java --enable-native-access=ALL-UNNAMED ...
```

The native library is located via the `native.lib.dir` system property (the
Cargo `target/` directory in dev/CI), or the platform library path.

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
`{ok:false,error:...}`; only null/UTF-8/panic conditions throw.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/java)

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
