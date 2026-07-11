# Benchmarks

`wickra-verify` ships a Criterion bench crate (`crates/verify-bench`) measuring
the cost of `verify` (recompute the backtest + compare field by field) and
`canonicalize` over the golden fixtures.

Numbers are filled in from the `bench.yml` nightly run once the core and bench
crate land; until then this file is a placeholder so the layout matches the rest
of the ecosystem.

| Operation | Input | Time | Notes |
|-----------|-------|------|-------|
| `verify`  | golden honest claim | _TBD_ | recompute + compare |
| `canonicalize` | golden verdict | _TBD_ | canonical JSON + blake3 |

Run locally:

```bash
cargo bench -p verify-bench
```
