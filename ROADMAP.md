# Roadmap

`wickra-verify` targets full wickra-grade parity with its sibling products
(`wickra-backtest` / `wickra-proof` / `wickra-screener`): the same versions, the
same structure, the same tests / fuzz / golden / examples / bindings / CI.

## Pre-1.0 (0.1.x)

- [x] Repository scaffold, governance, supply-chain and licensing baseline.
- [ ] `verify-core`: `Claim`, `Verdict`, canonicalization, tolerance-based
      field comparison, `verify()`, and the `command_json` boundary.
- [ ] Reference CLI (`wickra-verify`): verify a claim against a data directory,
      text or JSON output, non-zero exit on refutal.
- [ ] Ten language bindings over the JSON-over-C-ABI boundary -- native Rust,
      Python, Node.js, WASM, plus a C ABI hub for C, C++, C#, Go, Java, R.
- [ ] Byte-exact golden corpus (honest + fudged claims), conformance /
      property / fuzz tests, benchmarks, one runnable example per language.
- [ ] CI across all ten languages on three OSes; CodeQL, Scorecard, zizmor.
- [ ] Optional static in-browser WASM demo (no server).

## Later

- Migrate canonicalization onto `wickra-proof` as a shared dependency once both
  are released, so a verdict and a proof share the exact same fingerprint.
- First release to the language registries (USER-GO gated).

Trading tooling only -- no financial advice; see the disclaimer in the README.
