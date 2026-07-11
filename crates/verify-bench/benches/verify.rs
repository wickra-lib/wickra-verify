//! Criterion benchmarks for the verify core.
//!
//! `verify` is measured across the cross-product of bar counts {200, 1k, 5k} and
//! indicator counts {2, 10}, so the report captures how recomputing a backtest
//! and comparing it field by field scales with both dataset length and strategy
//! width. `compare` and `canonicalize` — the two halves of the verdict's cost —
//! are measured in isolation on a small and a large report, taken from genuine
//! engine runs so the shapes match production output exactly.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use verify_core::{canonicalize, compare, verify, Candle, Claim, DatasetRef};
use wickra_backtest_core::{run, StrategySpec};

const SYMBOL: &str = "BENCH";

/// A strategy trading [`SYMBOL`] with `n` EMA indicators (`n >= 2`). Entry and
/// exit cross the two fastest; the rest are computed but unreferenced, so the
/// indicator-count axis reflects real per-bar indicator work.
fn strategy(n: usize) -> Value {
    let mut indicators = serde_json::Map::new();
    for i in 0..n {
        indicators.insert(
            format!("ema{i}"),
            json!({ "type": "Ema", "params": [3 + i] }),
        );
    }
    json!({
        "symbol": SYMBOL,
        "timeframe": "1h",
        "indicators": indicators,
        "entry": { "cross_above": ["ema0", "ema1"] },
        "exit": { "cross_below": ["ema0", "ema1"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": {}
    })
}

/// A deterministic, non-degenerate `bars`-long candle universe.
fn universe(bars: usize) -> BTreeMap<String, Vec<Candle>> {
    let closes: Vec<f64> = (0..bars)
        .map(|i| 100.0 + 10.0 * ((i as f64) * 0.1).sin())
        .collect();
    let candles: Vec<Candle> = closes
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let o = if i == 0 { c } else { closes[i - 1] };
            Candle {
                time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
                open: o,
                high: o.max(c) + 1.0,
                low: o.min(c) - 1.0,
                close: c,
                volume: 1000.0,
            }
        })
        .collect();
    let mut data = BTreeMap::new();
    data.insert(SYMBOL.to_string(), candles);
    data
}

/// The honest report `strategy(n)` produces over `data`, as JSON.
fn honest_report(n: usize, data: &BTreeMap<String, Vec<Candle>>) -> Value {
    let spec: StrategySpec = serde_json::from_value(strategy(n)).unwrap();
    let report = run(&spec, &data[SYMBOL]).unwrap();
    serde_json::to_value(&report).unwrap()
}

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify");
    for &bars in &[200usize, 1_000, 5_000] {
        let data = universe(bars);
        group.throughput(Throughput::Elements(bars as u64));
        for &indicators in &[2usize, 10] {
            let claim = Claim {
                strategy: strategy(indicators),
                dataset_ref: DatasetRef::Inline { data: data.clone() },
                claimed_report: honest_report(indicators, &data),
            };
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{bars}bars_{indicators}ind")),
                &(claim, &data),
                |b, (claim, data)| b.iter(|| verify(claim, data).unwrap()),
            );
        }
    }
    group.finish();
}

fn bench_compare(c: &mut Criterion) {
    // Comparing a genuine report against itself is the confirm path (zero
    // mismatches) — the work verify does on every honest claim.
    let small = honest_report(2, &universe(200));
    let large = honest_report(2, &universe(5_000));
    let mut group = c.benchmark_group("compare");
    for (name, report) in [("small_report", &small), ("large_report", &large)] {
        group.bench_function(name, |b| b.iter(|| compare(report, report, 1e-9, 1e-6)));
    }
    group.finish();
}

fn bench_canonicalize(c: &mut Criterion) {
    let small = honest_report(2, &universe(200));
    let large = honest_report(2, &universe(5_000));
    let mut group = c.benchmark_group("canonicalize");
    for (name, report) in [("small_report", &small), ("large_report", &large)] {
        group.bench_function(name, |b| b.iter(|| canonicalize(report).unwrap()));
    }
    group.finish();
}

criterion_group!(benches, bench_verify, bench_compare, bench_canonicalize);
criterion_main!(benches);
