# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **The Examples job runs every example and checks what it prints.** It
  used to parse the Python, Node and R files and `cargo check` the Rust one;
  the Go, Java and C# examples were never built, and the WASM demo never
  looked at. Now each example is executed against the library it ships with
  and must print `doctored claim: REFUTED` -- an example that starts and
  prints nothing fails. The browser demo moves from `examples/web/` to
  `examples/wasm/`, the name every sibling uses, and its module is
  parse-checked with `node --check`.

- **The engine pin moves from a git rev to the published release.**
  `wickra-backtest-core` is `=0.1.4` from crates.io in the workspace and in
  the fuzz manifest, which is the same source code as the pinned rev
  `d58e357` minus one Java pom. The rev was the defect: cargo treats "this git
  URL, default branch" and "this git URL at rev X" as two sources, the fuzz
  manifest pinned the branch while the workspace pinned the rev, and the fuzz
  build carried two copies of the engine -- `verify_roundtrip` failed to
  compile with `expected &[Candle], found &Vec<Candle>`, two `Candle` types
  that share nothing. A registry version cannot split that way, and
  `cargo publish` needs one anyway.

- **The core crate is renamed `verify-core` -> `wickra-verify-core`.** It was
  one of the last two core crates in the organisation without the `wickra-`
  prefix, and the other one just cost wickra-proof its crate name: `proof-core`
  1.0.0 was published on crates.io by an unrelated project on 2026-09-09, before
  that repository had released anything. `verify-core` is still free; the point
  of taking the prefix now is that an unprefixed generic name is a name someone
  else can reach first, and the failure only shows up at `cargo publish`, after
  the tag.

  **Nothing a user types changes.** The CLI binary keeps the name
  `wickra-verify`, and the Python, npm, NuGet, Maven and R package names never
  carried the crate name. The Rust library is now
  `cargo add wickra-verify-core`, and `use verify_core::` becomes
  `use wickra_verify_core::`.

### Fixed

- **Maven Central would have rejected the first publish, after the job reported
  success.** `<scm>` and `<developers>` are validated by Central and their
  absence is refused outright; the `release` profile did not exist at all, so
  `mvn -Prelease deploy` matched no profile, warned, and deployed bare -- no
  sources jar, no javadoc jar, no signatures, and no publishing plugin to send
  them with. The profile now carries the same four plugins the rest of the
  organisation publishes with, including `waitUntil=published` so a green job
  means the artefact is on the repository rather than merely accepted. The two
  licences are split into separate entries, since `MIT OR Apache-2.0` in one
  `<name>` is an SPDX expression, not a licence Central recognises.

- **The engine was pinned by name, not by revision.** `wickra-backtest-core`
  came from a branch with no `rev`, so it tracked whatever upstream had last
  pushed. Beyond the drift, it makes this crate unusable by a consumer that
  pins the engine: cargo treats "this URL, default branch" and "this URL at rev
  X" as two sources, so a downstream pin produces two copies of
  `wickra-backtest-core` in one graph, and two copies share no types. wickra-zk
  hit exactly that against wickra-proof.

  Pinning moved the linked engine from `0.1.0` to `0.1.4`, so the goldens are
  re-blessed through `cargo run -p wickra-verify-core --example bless_golden`. Every
  verdict keeps its meaning: `honest` still matches, and the four doctored
  claims still fail with the same mismatch counts.

- **A dependency nothing used and that could not resolve.** `wickra-data =
  "0.9"` was declared for "the CLI's data input" and referenced by no crate.
  The pin was also unreachable -- `wickra-data` is published at `1.0.x` -- so
  it never produced a Dependabot PR either. Removed, with the reason recorded
  in the manifest: `Candle` comes from the engine, so adding it back would mean
  two crates defining the same row.

### Added
- Repository scaffold, governance, and supply-chain baseline for `wickra-verify`.
- `wickra-verify-core`: the deterministic core — `Claim` / `Verdict` / `Mismatch` wire
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
