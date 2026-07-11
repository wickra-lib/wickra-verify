# Determinism

The product is a verdict that is **the same everywhere**. A claim verified in
Python must reach byte-identical bytes when verified in Go, C#, R or the browser
over WASM. This document is the map of what guarantees that.

## The three mechanisms

1. **Recompute, never trust.** A verdict is derived by re-running the
   `wickra-backtest` engine over the claim's `(strategy, data)`, not by comparing
   two supplied numbers. The engine is deterministic; the same inputs always fold
   to the same report.
2. **Quantized canonical hashing.** Report and inputs hashes are `blake3` over a
   canonical serialization with floats quantized to `1e-8`
   ([CANONICALIZATION.md](CANONICALIZATION.md)), so last-ULP noise never changes
   a hash.
3. **Tolerant, sorted comparison.** Field comparison uses a fixed
   `atol + rtol·max` tolerance ([VERDICT.md](VERDICT.md)) and emits a mismatch
   list **sorted by field**, so the verdict bytes do not depend on hash-map or
   iteration order.

## Why the bindings agree

Every binding is a thin shell over the same `verify-core` compiled code. Each
drives the `Verifier::command_json` surface and returns the core's canonical JSON
**verbatim** — no language re-implements the logic, so there is nothing to drift.
The native bindings (Python, Node, WASM) link the Rust core directly; the C-ABI
consumers (C, C++, C#, Go, Java, R) call into the same `cdylib`.

## How it is pinned

- **Golden corpus** (`golden/`) — a fixed set of claims (one honest, several
  doctored) frozen to their expected verdicts, produced by a committed bless tool
  running the real engine. Every binding's golden test verifies each claim and
  asserts the response equals `golden/expected/<claim>.json` byte-for-byte. The
  Rust `tests/golden.rs`, the per-binding golden tests, and CI's
  `wasm-golden-parity.mjs` all assert the same bytes.
- **Property tests** (`tests/proptest_invariants.rs`) — over random universes: an
  honest claim always confirms with zero mismatches, one doctored field yields
  exactly one mismatch, canonicalization is idempotent and key-order invariant.
- **Fuzz targets** (`fuzz/`) — parse/compare/canonicalize/verify never panic;
  canonicalization is a fixed point; an honest claim over any bounded price path
  always confirms and is deterministic on repeat.
- **CI** runs the full binding matrix across Linux, macOS and Windows on every
  change, so a determinism regression in any language on any OS fails the build.

## Engine-version binding

The verdict records `engine_version`. Upgrading `wickra-backtest` can legitimately
change a report; the version travels with the verdict so divergence is labelled,
never silent. Two verdicts are only comparable at the same `engine_version`.
