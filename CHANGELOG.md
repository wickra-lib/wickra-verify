# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The Java binding loads the library it ships.** The jar carries the native
  library under `native/<os>-<arch>/` -- the release pipeline stages every
  platform there -- but the loader only ever looked at `-Dnative.lib.dir` and
  the working directory, so a Maven Central consumer got a jar it could not
  load without pointing the JVM at a library it had to build itself. The loader
  now resolves in wickra's order: `-Dnative.lib.dir` when set, the bundled copy
  extracted to a temporary file, every `target/release` or `target/debug` up
  the tree from the working directory and the class's own location, then the
  bare name.

### Changed

- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the live-demo and ecosystem lines, no separate H1), the
  License section carries wickra's wording and its `### Contribution` clause,
  and the shared sections run in wickra's order. Each binding README is
  `Install`, `Quick start`, `Benchmark`, `Documentation`, `Security`,
  `Disclaimer`, `License` with the product's own surface and protocol notes
  as subsections; the registry pages that render them now say how to report a
  vulnerability and under which licence the package ships.
  `examples/README.md` lists every language the way wickra's does, with the
  commands the CI examples job runs; the per-language example READMEs,
  `fuzz/README.md` and the `## Editing the docs` section of
  `docs/README.md` exist as they do in wickra.

### Changed

- **wickra-backtest-core 0.1.6.** The pin moves from `=0.1.4` to the release
  the family is on, in the workspace and in `fuzz/Cargo.toml`; the lock
  follows. A cross-repo scan lined the 24 wickra-lib repositories up, and the
  rest is what this one spelled differently: the C example's
  `CMAKE_CXX_STANDARD` 14 where the family builds with 17, the fuzz job on a
  rolling nightly rather than the family's pinned `nightly-2026-07-01`, and the
  example job's `dotnet-version`, which now reads `8.0.x`.

### Changed

- **The Python 3.9 CI row installs no pytest.** pytest 9.x requires 3.10, so
  the 3.9 row could only pin 8.4.2, which is below the fix for
  GHSA-6w46-j5rx-g56g and has no backport. The dev requirements are locked
  twice now (`ci-dev-py3.txt`, `ci-dev-py39.txt`, both hash-pinned), the 3.9
  lock carries maturin alone, and the row runs the suite through
  `run_without_pytest.py` -- the same modules, rewritten as plain functions
  with plain asserts, which 3.10 and up still run under pytest.

- **uv 0.12.15 for the lockfile script.** `scripts/update-lockfiles.sh`
  bootstraps 0.12.15 (was 0.12.13); the pin and all four release
  checksums move together, taken from the release's `.sha256` files.

## [0.1.2] - 2026-09-13

### Fixed

- **The R package builds on Windows again.** `bindings/r/.Rbuildignore`
  excluded `src/Makevars.win` from the source tarball, so every Windows build
  on r-universe linked the package object without the C ABI import library
  (`undefined reference to wickra_verify_version`) and failed to load. The
  file ships with the package now, and the ignore list names the staged
  files as configure actually names them.
- **The exported R functions are documented.** `wkverify_new`,
  `wkverify_command` and `wkverify_version` carried roxygen comments but no
  generated `man/` pages, which `R CMD check` reported as a WARNING on every
  platform.

## [0.1.1] - 2026-09-13

### Fixed

- **The release front, after the first release ran it.** The v0.1.0 Maven
  Central deployment was accepted and then took longer than the plugin's
  30-minute wait to publish, so the job failed while the artifact went live
  anyway, and the assets, the provenance and the GitHub Release behind it
  never ran; a re-run could not repair that, because Central refuses a second
  deployment of the same version. The Maven step now skips a version already
  on the repository and waits up to two hours. The gate's sweep of the tagged
  commit grades only the newest run per workflow again (a first run cancelled
  by its successor no longer makes a green commit unreleasable) while still
  counting a cancelled run as red. The semver check covers the library crate
  only: `cargo-semver-checks` errors on the CLI, a binary with no public API,
  the moment it is on crates.io. Two `setup-go` pins named v6.0.0 beside a
  v7.0.0 SHA.
- **The Python CI tools come from the hash-locked requirements**, compiled
  universally from 3.9 up; the 3.9 job could not resolve a lock compiled by a
  newer interpreter (`iniconfig` 2.3.0 requires 3.10).

The packages are the same code as 0.1.0; this release exists so that the
GitHub Release, its assets and its provenance are produced for the version
on every registry.

## [0.1.0] - 2026-09-13

### Changed

- **Every binding tests both operating modes, and the C ABI is tested at
  all.** A claim names its candles by reference (`dataset_ref.kind: files`,
  the data supplied with the `verify` command) or inline (embedded in the
  claim); the verdict must not depend on which, and only `inputs_hash` may
  differ, because it binds the reference. Python, Node, C#, Go, Java, R, WASM
  and C now re-issue every golden claim inline and assert exactly that. The C
  ABI, which six of the ten reaches sit on, had no test beside its examples;
  `examples/c/golden_test.c` runs the golden corpus and the mode check under
  ctest, with the claim list globbed by CMake. The WASM tests load the
  nodejs build directly and run under `node --test` in the WASM job, which
  replaces the parity script that used to do only the golden half.

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

[Unreleased]: https://github.com/wickra-lib/wickra-verify/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/wickra-lib/wickra-verify/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra-verify/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra-verify/releases/tag/v0.1.0
