//! Inline tests for verify-core: canonicalization vectors, honest/fudged claim
//! verification, mismatch reporting, tolerance handling, and the command-JSON
//! boundary.

use crate::{
    canonicalize, compare, explain, hash, verify, Claim, DatasetRef, Error, Verdict, Verifier,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use wickra_backtest_core::{run, Candle, StrategySpec};

/// A small, valid EMA-cross strategy (the shape wickra-backtest accepts).
fn strategy() -> Value {
    json!({
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "indicators": {
            "ema_fast": { "type": "Ema", "params": [5] },
            "ema_slow": { "type": "Ema", "params": [15] }
        },
        "entry": { "cross_above": ["ema_fast", "ema_slow"] },
        "exit": { "cross_below": ["ema_fast", "ema_slow"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": { "trailing_stop_pct": 5.0 }
    })
}

/// A deterministic oscillating candle series long enough to warm up EMA(15) and
/// produce at least one crossing.
fn candles() -> Vec<Candle> {
    (0..40)
        .map(|i| {
            let t = f64::from(i);
            let base = 100.0 + (t * 0.4).sin() * 8.0;
            Candle {
                time: 1_700_000_000 + i64::from(i) * 3600,
                open: base,
                high: base + 1.0,
                low: base - 1.0,
                close: base + 0.5,
                volume: 1000.0,
            }
        })
        .collect()
}

fn data() -> BTreeMap<String, Vec<Candle>> {
    let mut m = BTreeMap::new();
    m.insert("BTCUSDT".to_string(), candles());
    m
}

/// The honest report for `strategy()` on `data()`, as JSON.
fn honest_report() -> Value {
    let s: StrategySpec = serde_json::from_value(strategy()).unwrap();
    let d = data();
    let report = run(&s, d.get(&s.symbol).unwrap()).unwrap();
    serde_json::to_value(&report).unwrap()
}

/// A claim carrying `report` as its (untrusted) `claimed_report`, with the real
/// candles inline.
fn claim_with(report: Value) -> Claim {
    Claim {
        strategy: strategy(),
        dataset_ref: DatasetRef::Inline { data: data() },
        claimed_report: report,
    }
}

/// Add `delta` to a top-level numeric field of a report Value.
fn bump_field(report: &Value, field: &str, delta: f64) -> Value {
    let mut r = report.clone();
    let obj = r.as_object_mut().unwrap();
    let current = obj.get(field).and_then(Value::as_f64).unwrap();
    obj.insert(field.to_string(), json!(current + delta));
    r
}

#[test]
fn canonicalize_sorts_keys_and_strips_whitespace() {
    let v = json!({ "z": 1, "a": 2, "m": { "y": 3, "b": 4 } });
    assert_eq!(
        canonicalize(&v).unwrap(),
        "{\"a\":2,\"m\":{\"b\":4,\"y\":3},\"z\":1}"
    );
}

#[test]
fn canonicalize_rounds_floats_and_keeps_integers() {
    let v = json!({ "b": 1.000_000_000_4, "a": 2, "c": 1.5 });
    assert_eq!(canonicalize(&v).unwrap(), "{\"a\":2,\"b\":1,\"c\":1.5}");
}

#[test]
fn canonicalize_is_deterministic() {
    let v = json!({ "report": [1.5, 2.5], "hash": "x" });
    assert_eq!(canonicalize(&v).unwrap(), canonicalize(&v).unwrap());
}

#[test]
fn hash_is_64_hex() {
    let h = hash(&canonicalize(&json!({ "a": 1 })).unwrap());
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn round_to_snaps_to_the_grid() {
    assert!((crate::canon::round_to(1.234_567_894, 1e-8) - 1.234_567_89).abs() < 1e-12);
    // Zero quantum is a no-op, never a division by zero.
    assert!((crate::canon::round_to(3.5, 0.0) - 3.5).abs() < f64::EPSILON);
}

#[test]
fn honest_claim_verifies_true() {
    let claim = claim_with(honest_report());
    let verdict = verify(&claim, &data()).unwrap();
    assert!(verdict.matches);
    assert!(verdict.mismatches.is_empty());
    assert_eq!(verdict.engine_version, wickra_backtest_core::version());
    assert_eq!(verdict.claimed_report_hash.len(), 64);
    assert_eq!(verdict.inputs_hash.len(), 64);
    // An honest claim canonicalizes identically to the recomputed report.
    assert_eq!(verdict.claimed_report_hash, verdict.actual_report_hash);
}

#[test]
fn verify_is_reproducible() {
    let claim = claim_with(honest_report());
    let a = verify(&claim, &data()).unwrap();
    let b = verify(&claim, &data()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn fudged_metric_is_refuted() {
    let fudged = bump_field(&honest_report(), "fees_paid", 5.0);
    let claim = claim_with(fudged);
    let verdict = verify(&claim, &data()).unwrap();
    assert!(!verdict.matches);
    assert!(verdict.mismatches.iter().any(|m| m.field == "fees_paid"));
    assert_ne!(verdict.claimed_report_hash, verdict.actual_report_hash);
}

#[test]
fn mismatch_carries_field_and_delta() {
    let fudged = bump_field(&honest_report(), "initial_capital", 250.0);
    let claim = claim_with(fudged);
    let verdict = verify(&claim, &data()).unwrap();
    let m = verdict
        .mismatches
        .iter()
        .find(|m| m.field == "initial_capital")
        .unwrap();
    // claimed = actual + 250 -> delta (actual - claimed) = -250.
    assert!((m.delta + 250.0).abs() < 1e-6);
}

#[test]
fn mismatches_are_sorted_by_field() {
    let mut fudged = bump_field(&honest_report(), "fees_paid", 5.0);
    fudged = bump_field(&fudged, "initial_capital", 5.0);
    let verdict = verify(&claim_with(fudged), &data()).unwrap();
    let fields: Vec<&str> = verdict
        .mismatches
        .iter()
        .map(|m| m.field.as_str())
        .collect();
    let mut sorted = fields.clone();
    sorted.sort_unstable();
    assert_eq!(fields, sorted);
}

#[test]
fn array_length_difference_is_reported() {
    let mut report = honest_report();
    // Append a clone of the first equity point -> length differs by one.
    let equity = report["equity"].as_array().unwrap().clone();
    assert!(!equity.is_empty(), "expected a non-empty equity curve");
    let mut extended = equity.clone();
    extended.push(equity[0].clone());
    report["equity"] = Value::Array(extended);
    let verdict = verify(&claim_with(report), &data()).unwrap();
    assert!(!verdict.matches);
    assert!(verdict.mismatches.iter().any(|m| m.field == "equity[len]"));
}

#[test]
fn missing_symbol_data_is_reported() {
    let claim = claim_with(honest_report());
    let empty: BTreeMap<String, Vec<Candle>> = BTreeMap::new();
    assert!(matches!(verify(&claim, &empty), Err(Error::Data(_))));
}

#[test]
fn bad_strategy_is_rejected() {
    let s = r#"{"strategy":42,"dataset_ref":{"kind":"files","symbols":["X"]},"claimed_report":{}}"#;
    assert!(matches!(Claim::from_json(s), Err(Error::BadClaim(_))));
}

#[test]
fn unknown_field_is_rejected() {
    let s = format!(
        r#"{{"strategy":{},"dataset_ref":{{"kind":"files","symbols":["BTCUSDT"]}},"claimed_report":{{"x":1}},"surprise":1}}"#,
        strategy()
    );
    assert!(matches!(Claim::from_json(&s), Err(Error::BadClaim(_))));
}

#[test]
fn inline_data_helper_distinguishes_ref_kinds() {
    let inline = claim_with(honest_report());
    assert!(inline.inline_data().is_some());
    let files = Claim {
        strategy: strategy(),
        dataset_ref: DatasetRef::Files {
            symbols: vec!["BTCUSDT".to_string()],
            hash: None,
        },
        claimed_report: honest_report(),
    };
    assert!(files.inline_data().is_none());
}

#[test]
fn compare_is_empty_for_identical_reports() {
    let r = honest_report();
    assert!(compare(&r, &r, 1e-9, 1e-6).is_empty());
}

#[test]
fn explain_verified_and_refuted() {
    let ok = verify(&claim_with(honest_report()), &data()).unwrap();
    assert!(explain(&ok).starts_with("VERIFIED:"));

    let bad = verify(
        &claim_with(bump_field(&honest_report(), "fees_paid", 5.0)),
        &data(),
    )
    .unwrap();
    let text = explain(&bad);
    assert!(text.starts_with("REFUTED:"));
    assert!(text.contains("fees_paid"));
}

#[test]
fn verifier_tolerance_can_accept_a_small_fudge() {
    let claim = claim_with(bump_field(&honest_report(), "fees_paid", 5.0));
    // Default tolerance refutes it.
    assert!(!verify(&claim, &data()).unwrap().matches);
    // A deliberately loose verifier accepts it.
    let loose = Verifier::new(r#"{"atol":1000.0,"rtol":1.0}"#).unwrap();
    assert!(loose.verify(&claim, &data()).unwrap().matches);
}

#[test]
fn verifier_new_falls_back_to_defaults() {
    let v = Verifier::new("{}").unwrap();
    assert!(
        v.verify(&claim_with(honest_report()), &data())
            .unwrap()
            .matches
    );
}

#[test]
fn command_json_verify_inline_matches_direct() {
    let claim = claim_with(honest_report());
    let direct = verify(&claim, &data()).unwrap();
    // No `data` field: the handle falls back to the claim's inline data.
    let req = json!({ "cmd": "verify", "claim": claim }).to_string();
    let out = Verifier::default().command_json(&req).unwrap();
    let parsed: Verdict = serde_json::from_str(&out).unwrap();
    assert_eq!(parsed, direct);
}

#[test]
fn command_json_verify_with_explicit_data() {
    let claim = claim_with(honest_report());
    let req = json!({ "cmd": "verify", "claim": claim, "data": data() }).to_string();
    let out = Verifier::default().command_json(&req).unwrap();
    let parsed: Verdict = serde_json::from_str(&out).unwrap();
    assert!(parsed.matches);
}

#[test]
fn command_json_explain_returns_text() {
    let verdict = verify(&claim_with(honest_report()), &data()).unwrap();
    let req = json!({ "cmd": "explain", "verdict": verdict }).to_string();
    let out = Verifier::default().command_json(&req).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(true));
    assert!(v["text"].as_str().unwrap().starts_with("VERIFIED:"));
}

#[test]
fn command_json_version_reports_both_versions() {
    let out = Verifier::default()
        .command_json(r#"{"cmd":"version"}"#)
        .unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["version"], json!(crate::version()));
    assert_eq!(v["engine_version"], json!(wickra_backtest_core::version()));
}

#[test]
fn command_json_canonicalize_exposes_the_string() {
    let out = Verifier::default()
        .command_json(r#"{"cmd":"canonicalize","value":{"b":1,"a":2}}"#)
        .unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(true));
    assert_eq!(v["canonical"], json!("{\"a\":2,\"b\":1}"));
}

#[test]
fn command_json_unknown_cmd_returns_error_envelope() {
    let out = Verifier::default()
        .command_json(r#"{"cmd":"nope"}"#)
        .unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(false));
    assert!(v["error"].as_str().unwrap().contains("nope"));
}
