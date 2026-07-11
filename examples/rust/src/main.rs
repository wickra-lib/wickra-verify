//! A runnable Rust example: confirm an honest backtest report, then show that a
//! doctored one is refuted — verification recomputes the report from
//! `(strategy, data)` and compares, so a fabricated number cannot pass.
//!
//! ```bash
//! cargo run -p wickra-verify-example
//! ```

use std::collections::BTreeMap;

use serde_json::{json, Value};
use verify_core::{verify, Candle, Claim, DatasetRef};
use wickra_backtest_core::{run, StrategySpec};

const SYMBOL: &str = "AAA";

fn strategy() -> Value {
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
        "risk": {}
    })
}

/// A short V-shaped price path so the fast/slow EMA cross fires at least once.
fn candles() -> Vec<Candle> {
    let closes = [
        120.0, 118.0, 116.0, 114.0, 112.0, 110.0, 108.0, 112.0, 116.0, 120.0, 124.0, 128.0,
    ];
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

fn main() {
    let mut data = BTreeMap::new();
    data.insert(SYMBOL.to_string(), candles());

    // The honest report, recomputed with the real engine.
    let spec: StrategySpec = serde_json::from_value(strategy()).expect("valid strategy");
    let report = run(&spec, &data[SYMBOL]).expect("engine runs");
    let honest_report = serde_json::to_value(&report).expect("report serializes");

    println!("wickra-verify {}", verify_core::version());

    // An honest claim confirms.
    let honest = Claim {
        strategy: strategy(),
        dataset_ref: DatasetRef::Inline { data: data.clone() },
        claimed_report: honest_report.clone(),
    };
    let verdict = verify(&honest, &data).expect("verify");
    assert!(verdict.matches, "an honest claim must verify");
    println!("honest claim: VERIFIED");

    // A single doctored field is caught.
    let mut doctored_report = honest_report;
    doctored_report["fees_paid"] = json!(99_999.0);
    let doctored = Claim {
        strategy: strategy(),
        dataset_ref: DatasetRef::Inline { data: data.clone() },
        claimed_report: doctored_report,
    };
    let verdict = verify(&doctored, &data).expect("verify");
    assert!(!verdict.matches, "a doctored claim must be refuted");
    println!(
        "doctored claim: REFUTED ({} mismatch: {})",
        verdict.mismatches.len(),
        verdict.mismatches[0].field
    );
}
