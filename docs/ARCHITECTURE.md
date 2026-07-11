# Architecture

`wickra-verify` is one deterministic Rust core plus a reference CLI and ten
language bindings over a single JSON-over-C-ABI command boundary.

```
                      ┌─────────────────────────────────────────┐
                      │            verify-core (Rust)            │
   Claim ───────────► │  claim.rs   parse + validate the claim   │ ──► Verdict
   (strategy, data,   │  verify.rs  recompute report via engine  │
    claimed_report)   │  compare.rs field-by-field, tolerant     │
                      │  canon.rs   canonical JSON + blake3       │
                      │  verdict.rs matches, mismatches, hashes   │
                      └───────────────────┬─────────────────────┘
                                          │  wickra-backtest engine (pinned)
                                          ▼
              recomputed BacktestReport ──► compared against claimed_report
```

## The core (`crates/verify-core`)

- **`claim.rs`** — the `Claim` wire type: `{strategy, dataset_ref, claimed_report}`
  with `deny_unknown_fields`. `dataset_ref` is `Inline` (embedded candles) or
  `Files` (named symbols resolved out of band).
- **`verify.rs`** — `verify(claim, data)` deserializes the embedded
  `StrategySpec`, re-runs the `wickra-backtest` engine over the candles, serializes
  the result, and hands both reports to `compare`. Also hosts the `Verifier`
  command handle (`verify` / `explain` / `canonicalize` / `version`).
- **`compare.rs`** — walks two `serde_json::Value`s, flags numeric leaves that
  differ beyond tolerance and array-length divergence, returns a field-sorted
  `Vec<Mismatch>`.
- **`canon.rs`** — byte-for-byte the wickra-proof canonicalization; `blake3` over
  the canonical string yields the report/inputs hashes.
- **`verdict.rs`** — the `Verdict` and `Mismatch` wire types.
- **`config.rs`** — the comparison tolerances (`atol`, `rtol`).

## The command boundary

Every binding drives the same envelope-based command surface
(`Verifier::command_json`):

| `cmd` | Payload | Response |
| ----- | ------- | -------- |
| `verify` | `{claim, data?}` | the full `Verdict` |
| `explain` | `{verdict}` | `{ok:true, text:...}` |
| `canonicalize` | `{value}` | `{ok:true, canonical:...}` |
| `version` | — | `{version, engine_version}` |

Domain errors (a bad claim, an unknown command) come back in-band as
`{ok:false, error:...}`; only null/UTF-8/panic conditions escalate through the
C ABI as a negative length. The response is always the core's canonical JSON, so
every binding returns byte-identical bytes.

## The binding surface

`verify-core` is exposed **natively** in Rust, Python (PyO3), Node.js (napi) and
WASM (wasm-bindgen), and over a **C ABI hub** — a `cdylib` + generated header —
in C, C++, C#, Go, Java and R. The C-ABI consumers all speak the same four
functions (`wickra_verify_new/free/command/version`); each language wraps them in
an idiomatic handle. See each `bindings/<lang>/README.md`.

## Determinism as an invariant

The golden corpus (`golden/`) pins a fixed set of claims to their expected
verdicts, and the cross-language golden tests assert every binding reproduces
those bytes. See [DETERMINISM.md](DETERMINISM.md).
