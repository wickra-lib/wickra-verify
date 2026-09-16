<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Verify — deterministically confirm or refute a claimed backtest report against its strategy and data, in ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/ci.svg)](https://github.com/wickra-lib/wickra-verify/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-verify)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/release.svg)](https://github.com/wickra-lib/wickra-verify/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-verify/license.svg)](https://github.com/wickra-lib/wickra-verify#license)

# Wickra Verify — C / C++

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for C / C++. `cargo build -p wickra-verify-c --release` — a prebuilt shared/static library plus a generated `wickra_verify.h`, no system dependencies.**

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `wickra-verify-core` as a tiny, JSON-shaped surface built as both a
`cdylib` (dynamic library) and a `staticlib`.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-verify/releases) — each archive
has `wickra_verify.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-verify-c --release
# -> target/release/libwickra_verify.{so,dylib} or wickra_verify.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

## Quick start

[`examples/c/verify.c`](https://github.com/wickra-lib/wickra-verify/blob/main/examples/c/verify.c) is the runnable example the CI smoke job executes; in full:

```c
/* A minimal C example: submit a claim to the wickra-verify C ABI whose report
 * has been doctored, and assert that verification refutes it. This is the whole
 * product in one file — verification recomputes the report from (strategy, data)
 * and compares, so a fabricated `claimed_report` cannot pass.
 *
 * No JSON parser is needed on the C side: the claim is assembled from string
 * literals, and the verdict is inspected with a substring search. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_verify.h"

/* An EMA-cross strategy trading symbol AAA. */
static const char *STRATEGY =
    "{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
    "\"risk\":{}}";

/* A short V-shaped price path so the fast/slow EMA cross fires at least once. */
static const char *DATA =
    "{\"AAA\":["
    "{\"time\":1700000000,\"open\":120,\"high\":121,\"low\":119,\"close\":120,\"volume\":1000},"
    "{\"time\":1700003600,\"open\":120,\"high\":121,\"low\":117,\"close\":118,\"volume\":1000},"
    "{\"time\":1700007200,\"open\":118,\"high\":119,\"low\":115,\"close\":116,\"volume\":1000},"
    "{\"time\":1700010800,\"open\":116,\"high\":117,\"low\":113,\"close\":114,\"volume\":1000},"
    "{\"time\":1700014400,\"open\":114,\"high\":115,\"low\":111,\"close\":112,\"volume\":1000},"
    "{\"time\":1700018000,\"open\":112,\"high\":113,\"low\":109,\"close\":110,\"volume\":1000},"
    "{\"time\":1700021600,\"open\":110,\"high\":111,\"low\":107,\"close\":108,\"volume\":1000},"
    "{\"time\":1700025200,\"open\":108,\"high\":113,\"low\":107,\"close\":112,\"volume\":1000},"
    "{\"time\":1700028800,\"open\":112,\"high\":117,\"low\":111,\"close\":116,\"volume\":1000},"
    "{\"time\":1700032400,\"open\":116,\"high\":121,\"low\":115,\"close\":120,\"volume\":1000},"
    "{\"time\":1700036000,\"open\":120,\"high\":125,\"low\":119,\"close\":124,\"volume\":1000},"
    "{\"time\":1700039600,\"open\":124,\"high\":129,\"low\":123,\"close\":128,\"volume\":1000}]}";

/* A fabricated report: a claimant asserts an inflated fees figure. */
static const char *CLAIMED_REPORT = "{\"fees_paid\":99999.0}";

/* Read a command response into a freshly malloc'd, NUL-terminated buffer using
 * the length-out protocol. Returns NULL on failure. */
static char *run(WickraVerify *verifier, const char *cmd) {
    int len = wickra_verify_command(verifier, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_verify_command(verifier, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraVerify *verifier = wickra_verify_new();
    if (!verifier) {
        fprintf(stderr, "failed to create verifier\n");
        return 1;
    }

    /* Assemble the verify command: an inline claim carrying its own data. */
    size_t cap = strlen(STRATEGY) + strlen(DATA) + strlen(CLAIMED_REPORT) + 128;
    char *cmd = (char *)malloc(cap);
    if (!cmd) {
        wickra_verify_free(verifier);
        return 1;
    }
    snprintf(cmd, cap,
             "{\"cmd\":\"verify\",\"claim\":{\"strategy\":%s,"
             "\"dataset_ref\":{\"kind\":\"inline\",\"data\":%s},"
             "\"claimed_report\":%s}}",
             STRATEGY, DATA, CLAIMED_REPORT);

    char *verdict = run(verifier, cmd);
    if (!verdict) {
        free(cmd);
        wickra_verify_free(verifier);
        return 1;
    }

    printf("wickra-verify %s\n", wickra_verify_version());
    /* The doctored report must be refuted: the verdict says matches:false. */
    int refuted = strstr(verdict, "\"matches\":false") != NULL;
    printf("verdict: %s\n", refuted ? "REFUTED (tamper caught)" : "matched?!");

    free(verdict);
    free(cmd);
    wickra_verify_free(verifier);

    if (!refuted) {
        fprintf(stderr, "a doctored report was not refuted\n");
        return 1;
    }
    return 0;
}
```

### Surface

```c
#include "wickra_verify.h"

WickraVerify *wickra_verify_new(void);
void          wickra_verify_free(WickraVerify *handle);
int32_t       wickra_verify_command(WickraVerify *handle,
                                    const char *cmd_json,
                                    char *out, size_t cap);
const char   *wickra_verify_version(void);
```

- **`wickra_verify_new`** creates a verifier handle with the default tolerances.
  Never fails.
- **`wickra_verify_free`** destroys a handle (null is a no-op).
- **`wickra_verify_command`** applies a command JSON and writes the response JSON
  into the caller's buffer using a length-out protocol (below).
- **`wickra_verify_version`** returns a static, NUL-terminated version string
  (do not free).

### Command / response protocol

Everything goes through `wickra_verify_command`. Commands are JSON objects with a
`"cmd"` field: `verify`, `explain`, `canonicalize`, `version`. Responses are
JSON, e.g. the full `Verdict` for `verify`, `{"ok":true,"text":...}` for
`explain`.

The response is returned via a caller-owned buffer with a length-out protocol —
the callee never allocates memory the caller must free:

1. Call with `out = NULL`, `cap = 0` to learn the response length `len`
   (excluding the terminating NUL).
2. Allocate `len + 1` bytes and call again; the response plus a NUL is written.

Whenever `len < cap`, the response is written on that call, so a
sufficiently-large buffer needs only one call.

Return codes:

| Return   | Meaning                                              |
|----------|------------------------------------------------------|
| `>= 0`   | Response length in bytes (excluding the NUL).        |
| `-1`     | A required pointer (`handle` or `cmd_json`) is null. |
| `-2`     | `cmd_json` is not valid UTF-8.                        |
| `-3`     | A panic was caught at the boundary.                  |

Domain errors (a bad claim, an unknown command) are **not** negative — they come
back in-band as `{"ok":false,"error":...}` JSON in the buffer.

### Header generation

`include/wickra_verify.h` is generated with [cbindgen] and committed; CI fails if
it drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --crate wickra-verify-c --output include/wickra_verify.h
```

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-verify/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-verify>
- **Docs** (guides, spec reference, cookbook): <https://verify.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-verify/tree/main/examples/c)

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

[cbindgen]: https://github.com/mozilla/cbindgen
