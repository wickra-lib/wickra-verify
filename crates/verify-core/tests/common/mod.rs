//! Shared fixtures for the verify-core integration tests: a valid embedded
//! strategy, a small deterministic candle universe, and the honest report that
//! strategy produces over it (recomputed with the real engine).
//!
//! Each integration-test binary pulls this module in and uses a different subset
//! of these helpers, so unused items in any single binary are expected.
#![allow(dead_code)]

use serde_json::{json, Value};
use std::collections::BTreeMap;
use verify_core::{Candle, Claim, DatasetRef, StrategySpec};

/// The symbol the sample strategy trades.
pub const SYMBOL: &str = "TEST";

/// A valid embedded `StrategySpec` (EMA cross) that trades [`SYMBOL`].
pub fn strategy_json() -> Value {
    json!({
        "symbol": SYMBOL,
        "timeframe": "1h",
        "indicators": {
            "ema_fast": { "type": "Ema", "params": [3] },
            "ema_slow": { "type": "Ema", "params": [8] }
        },
        "entry": { "cross_above": ["ema_fast", "ema_slow"] },
        "exit": { "cross_below": ["ema_fast", "ema_slow"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": { "trailing_stop_pct": 5.0 }
    })
}

/// Build a candle series from a list of closes; `open` is the previous close,
/// `high`/`low` are `max`/`min(open, close) ± 1`, volume is constant.
pub fn candles_from(closes: &[f64]) -> Vec<Candle> {
    closes
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
        .collect()
}

/// A deterministic 40-bar V-shaped universe (down to bar 10, then up) so the EMA
/// cross fires at least once.
pub fn sample_closes() -> Vec<f64> {
    (0..40)
        .map(|i| {
            if i <= 10 {
                120.0 - 2.0 * f64::from(i)
            } else {
                100.0 + 2.0 * f64::from(i - 10)
            }
        })
        .collect()
}

/// The sample universe keyed by symbol.
pub fn sample_data() -> BTreeMap<String, Vec<Candle>> {
    let mut data = BTreeMap::new();
    data.insert(SYMBOL.to_string(), candles_from(&sample_closes()));
    data
}

/// The honest report the sample strategy produces over `data`, recomputed with
/// the real engine — the ground truth an honest claim carries.
pub fn honest_report(data: &BTreeMap<String, Vec<Candle>>) -> Value {
    let spec: StrategySpec = serde_json::from_value(strategy_json()).unwrap();
    let candles = data.get(SYMBOL).expect("sample data has the symbol");
    let report = wickra_backtest_core::run(&spec, candles).expect("engine runs");
    serde_json::to_value(&report).expect("report serializes")
}

/// An honest, self-contained (`inline`) claim over `data`.
pub fn honest_claim(data: &BTreeMap<String, Vec<Candle>>) -> Claim {
    Claim {
        strategy: strategy_json(),
        dataset_ref: DatasetRef::Inline { data: data.clone() },
        claimed_report: honest_report(data),
    }
}
