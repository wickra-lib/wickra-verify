# Architecture

`wickra-verify` is a library-shaped product: one small deterministic core, a
reference CLI, and ten thin language bindings that all reach the same core over a
single JSON string boundary. It mirrors the structure of `wickra-backtest` (its
runtime dependency), `wickra-proof` and `wickra-screener`.

## Layers

```
                 +---------------------------------------------+
  (claim, data)  |  verify-core                                |
  ------------>  |   claim.rs    Claim { strategy, report, .. }|
                 |   verify.rs   verify() -> Verdict           |
                 |   compare.rs  field-by-field within tolerance|--> Verdict
                 |   canon.rs    canonicalize() + blake3        |    { verified, mismatches, .. }
                 |   command.rs  command_json(&str) -> String   |
                 +---------------+-----------------------------+
                                 | depends on
                 +---------------v--------------+
                 |  wickra-backtest (pinned)    |  run(spec, candles) -> BacktestReport
                 |  engine_version = <pinned>   |
                 +------------------------------+

  CLI --+
  C  ---+  every surface passes a command JSON string in and gets a JSON
  .. ---+  string out, verbatim -- the basis for byte-identical cross-language golden.
```

## The verify pipeline

1. Parse the `Claim`: the strategy spec (identical JSON to `wickra-backtest`'s
   `StrategySpec`), a data commitment, and the claimed `BacktestReport`.
2. Recompute: run the strategy over the supplied candle data with the pinned
   `wickra-backtest` engine, producing a fresh `BacktestReport`.
3. Compare (`compare.rs`): walk the claimed and fresh reports field by field,
   comparing floats within a fixed tolerance and everything else exactly.
   Collect any mismatch into a stably-sorted list.
4. Emit a `Verdict`: `verified: true` if the mismatch list is empty, else the
   list of `{ field, claimed, actual }`, plus the `engine_version` it was reached
   under and a canonical hash of the inputs.

## The command boundary

Every consumer speaks the same envelope. A stateless `Verifier` handle exposes
`command_json(&str) -> String`, which parses a `{"cmd": ..}` request, dispatches
(`verify` / `canonicalize` / `version`), and returns a canonical JSON string. The
handle holds no state, no RNG and no clock.

## Where determinism is enforced

- **Tolerance-based float comparison** (`compare.rs`) -- never bitwise, so the
  parallel (native) and sequential (WASM) engine paths agree.
- **Stable mismatch ordering** -- sorted by field, so the verdict is
  byte-identical across all ten languages.
- **Canonicalization** (`canon.rs`) -- the same rules `wickra-proof` uses, so a
  verdict's input fingerprint matches a proof's.
- **Engine-version pinning** -- folded into the verdict; a different engine
  yields a different, labelled result rather than a silent divergence.

## See also

- [THREAT_MODEL.md](THREAT_MODEL.md) -- what a verdict does and does not attest.
- [ROADMAP.md](ROADMAP.md) -- build-out status.
