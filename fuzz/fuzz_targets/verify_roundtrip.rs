#![no_main]
//! Fuzz the verify contract with genuine inputs. The fuzz bytes drive a bounded,
//! always-valid candle universe under a fixed strategy; the report is recomputed
//! with the real engine and packaged as an honest claim, which must always
//! verify with zero mismatches and be deterministic on repeat. This pins the
//! core invariant — an honest claim always confirms against its own inputs —
//! across an unbounded range of price paths.

use libfuzzer_sys::fuzz_target;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use verify_core::{verify, Candle, Claim, DatasetRef};
use wickra_backtest_core::{run, StrategySpec};

const SYMBOL: &str = "F";

fn strategy_value() -> Value {
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

fuzz_target!(|data: &[u8]| {
    // Need enough bars to warm the slow EMA; pad the fuzz bytes deterministically.
    let mut closes: Vec<f64> = data.iter().map(|&b| 50.0 + f64::from(b)).collect();
    while closes.len() < 16 {
        closes.push(100.0);
    }

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

    let mut universe = BTreeMap::new();
    universe.insert(SYMBOL.to_string(), candles);

    // Recompute the honest report with the real engine and package it as a claim.
    let spec: StrategySpec = serde_json::from_value(strategy_value()).unwrap();
    let report = run(&spec, &universe[SYMBOL]).expect("a bounded, valid universe always runs");
    let claim = Claim {
        strategy: strategy_value(),
        dataset_ref: DatasetRef::Inline {
            data: universe.clone(),
        },
        claimed_report: serde_json::to_value(&report).unwrap(),
    };

    let verdict = verify(&claim, &universe).expect("verify recomputes without error");
    assert!(
        verdict.matches,
        "an honest claim must verify against its own inputs"
    );
    assert!(
        verdict.mismatches.is_empty(),
        "an honest claim has no mismatches"
    );
    assert_eq!(
        verdict.claimed_report_hash, verdict.actual_report_hash,
        "an honest claim's claimed and actual report hashes coincide"
    );

    // Determinism: re-verifying is byte-identical.
    let again = verify(&claim, &universe).unwrap();
    assert_eq!(
        verdict.inputs_hash, again.inputs_hash,
        "verify is deterministic"
    );
    assert_eq!(verdict.actual_report_hash, again.actual_report_hash);
});
