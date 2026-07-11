# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Repository scaffold, governance, and supply-chain baseline for `wickra-verify`.
- `verify-core`: the deterministic core — `Claim` / `Verdict` / `Mismatch` wire
  types, engine-recompute verification, tolerant field-by-field comparison, and
  wickra-proof-compatible canonicalization + blake3 hashing.
- `wickra-verify` CLI: `--claim` / `--data` / `--explain`, exit 2 on a refuted
  claim for CI gating.
- Ten language bindings — Rust, Python (PyO3), Node.js (napi), WASM
  (wasm-bindgen) natively, and C, C++, C#, Go, Java, R over the C ABI hub — all
  driving the same `verify` / `explain` / `canonicalize` / `version` commands.
- Byte-exact golden corpus (`golden/`) with a committed bless tool, and
  cross-language golden tests asserting every binding returns identical bytes.
- Rust integration suite (conformance, golden, CLI-equals-binding, property
  tests), four `cargo-fuzz` targets, and the `verify-bench` Criterion crate.
- One runnable example per language, a static in-browser WASM demo, and shared
  example data.
- CI across the full ten-language matrix on Linux/macOS/Windows, plus CodeQL,
  OpenSSF Scorecard, zizmor, link-check, benchmark and metadata-audit workflows;
  the tag-triggered release workflow is authored but unpublished.
- Documentation: architecture, claim format, verdict, canonicalization,
  determinism and a cookbook under `docs/`.

[Unreleased]: https://github.com/wickra-lib/wickra-verify/commits/main
