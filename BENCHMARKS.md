# Benchmarks

`wickra-verify` ships a Criterion bench crate (`crates/verify-bench`) measuring
the cost of `verify` (recompute the backtest with the `wickra-backtest` engine +
compare the report field by field), and of `compare` and `canonicalize` in
isolation.

`verify` is dominated by the engine recomputation and scales roughly linearly
with the number of bars; the field comparison and canonical hashing are a small,
near-constant tail. The figures below are indicative, from a local run on Windows
(`x86_64`, release profile); the authoritative numbers come from the nightly
`bench.yml` run on the CI runners.

| Operation | Input | Time (indicative) |
|-----------|-------|-------------------|
| `verify` | 200 bars, 2 indicators | ~2.8 ms |
| `verify` | 1 000 bars, 2 indicators | ~12 ms |
| `verify` | 5 000 bars, 2 indicators | ~79 ms |
| `verify` | 5 000 bars, 10 indicators | ~89 ms |
| `compare` | small vs large report | sub-millisecond |
| `canonicalize` | small vs large report | sub-millisecond |

`verify` from 200 → 5 000 bars grows about linearly (≈ 28× the time for 25× the
bars), as expected from the engine recomputation; adding indicators (2 → 10) is a
modest per-bar cost. `compare` and `canonicalize` are negligible next to the
recomputation — which is the point: verification's cost is *re-running the
backtest*, not checking the numbers.

Run locally:

```bash
cargo bench -p verify-bench
```
