# Canonicalization and hashing

Every hash in a verdict is a `blake3` digest over a **canonical** serialization.
Canonicalization is the load-bearing contract: it must be byte-for-byte identical
in all ten languages, or the hashes diverge. `wickra-verify` reproduces exactly
the canonicalization [`wickra-proof`](https://github.com/wickra-lib/wickra-proof)
uses, so the two products share one determinism moat — a verdict's `inputs_hash`
equals the wickra-proof hash of the same inputs.

The rules (see [`crates/verify-core/src/canon.rs`](../crates/verify-core/src/canon.rs)):

1. **Object keys** are sorted ascending by Unicode code point.
2. **No structural whitespace** — no spaces or newlines between tokens.
3. **Floats** are quantized to a `1e-8` grid by decimal rounding
   (`{:.8}`), trailing zeros are trimmed, whole values collapse to their integer
   token, and negative zero is normalized to `0`. Magnitudes at or above the
   point where the f64 ULP reaches the `1e-8` grid
   (`45_035_996.273_704_96`) fall back to the shortest round-trippable form, so
   canonicalization stays a fixed point.
4. **`NaN` / `±inf`** cannot appear in a number position — JSON has no such
   token, and serde collapses a non-finite `f64` to `null` deterministically.
5. **Arrays** keep their order; **strings** use standard JSON escaping.

`blake3` over that canonical string yields the lowercase 64-hex digest (no
prefix). `hash(canonicalize(x))` is the stable digest of any serializable `x`.

## Fixed point

Canonicalization is idempotent: parsing a canonical string and re-canonicalizing
yields identical bytes. The `canonicalize` fuzz target pins this property, and the
`compare_reports` / `verify_roundtrip` targets pin that comparison and
verification never panic and that an honest claim always confirms. The
`proptest_invariants` suite pins idempotence and key-order invariance over random
reports.

## Why quantize?

Two independent, deterministic computations of the *same* report can differ in
the last ULP (different summation order, different platform math). Quantizing to
`1e-8` before hashing makes the canonical form — the exact bytes the hash is taken
over — identical, so the determinism contract is *canonical* stability, not raw
f64 bit-equality. The float **comparison** in a verdict uses a tolerance
([VERDICT.md](VERDICT.md)); the float **hashing** uses quantization. They are
separate mechanisms serving the same goal: no false divergence.
