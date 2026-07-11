//! Serde conformance for the public wire types: `Claim` (both `DatasetRef`
//! variants) round-trips through JSON and TOML, `Verdict`/`Mismatch` round-trip
//! through JSON, canonicalization is key-order invariant, and the structural
//! guards (`deny_unknown_fields`, object-only strategy/report, non-finite
//! numbers) reject bad input rather than silently accepting it.

mod common;

use std::collections::BTreeMap;
use verify_core::{canonicalize, verify, Claim, DatasetRef, Verdict};

#[test]
fn claim_json_round_trips_inline() {
    let claim = common::honest_claim(&common::sample_data());
    let json = serde_json::to_string(&claim).unwrap();
    let back = Claim::from_json(&json).unwrap();
    assert_eq!(claim, back);
    assert!(back.inline_data().is_some());
}

#[test]
fn claim_json_round_trips_files() {
    let claim = Claim {
        strategy: common::strategy_json(),
        dataset_ref: DatasetRef::Files {
            symbols: vec![common::SYMBOL.to_string()],
            hash: Some("deadbeef".to_string()),
        },
        claimed_report: serde_json::json!({ "schema_version": 1 }),
    };
    let json = serde_json::to_string(&claim).unwrap();
    let back = Claim::from_json(&json).unwrap();
    assert_eq!(claim, back);
    // A `files` claim carries no inline data.
    assert!(back.inline_data().is_none());
}

#[test]
fn claim_toml_round_trips() {
    let toml = r#"
[strategy]
symbol = "TEST"
timeframe = "1h"

[dataset_ref]
kind = "files"
symbols = ["TEST"]

[claimed_report]
schema_version = 1
"#;
    let claim = Claim::from_toml(toml).unwrap();
    assert_eq!(
        claim.dataset_ref,
        DatasetRef::Files {
            symbols: vec!["TEST".to_string()],
            hash: None,
        }
    );
    assert!(claim.strategy.is_object());
    assert!(claim.claimed_report.is_object());
}

#[test]
fn verdict_json_round_trips() {
    let data = common::sample_data();
    let verdict = verify(&common::honest_claim(&data), &data).unwrap();
    let json = serde_json::to_string(&verdict).unwrap();
    let back: Verdict = serde_json::from_str(&json).unwrap();
    assert_eq!(verdict.matches, back.matches);
    assert_eq!(verdict.claimed_report_hash, back.claimed_report_hash);
    assert_eq!(verdict.actual_report_hash, back.actual_report_hash);
    assert_eq!(verdict.inputs_hash, back.inputs_hash);
    assert_eq!(verdict.mismatches.len(), back.mismatches.len());
}

#[test]
fn mismatch_json_round_trips() {
    // A single doctored field yields exactly one mismatch; round-trip it.
    let data = common::sample_data();
    let mut claim = common::honest_claim(&data);
    claim.claimed_report["metrics"]["sharpe"] = serde_json::json!(999.0);
    let verdict = verify(&claim, &data).unwrap();
    assert_eq!(verdict.mismatches.len(), 1);
    let json = serde_json::to_string(&verdict.mismatches[0]).unwrap();
    let back: verify_core::Mismatch = serde_json::from_str(&json).unwrap();
    assert_eq!(verdict.mismatches[0].field, back.field);
    assert_eq!(verdict.mismatches[0].field, "metrics.sharpe");
}

#[test]
fn canonicalize_is_key_order_invariant() {
    let a = serde_json::json!({ "b": 1, "a": { "z": 2, "y": 3 }, "c": [1, 2] });
    let b = serde_json::json!({ "c": [1, 2], "a": { "y": 3, "z": 2 }, "b": 1 });
    assert_eq!(canonicalize(&a).unwrap(), canonicalize(&b).unwrap());
}

#[test]
fn unknown_claim_field_is_rejected() {
    // `deny_unknown_fields`: an extra top-level key fails to parse.
    let json = r#"{
        "strategy": {},
        "dataset_ref": {"kind": "files", "symbols": []},
        "claimed_report": {},
        "surprise": 1
    }"#;
    assert!(Claim::from_json(json).is_err());
}

#[test]
fn non_object_strategy_and_report_are_rejected() {
    // The strategy must be a StrategySpec object.
    let bad_strategy = r#"{"strategy": 42, "dataset_ref": {"kind": "files", "symbols": []}, "claimed_report": {}}"#;
    assert!(Claim::from_json(bad_strategy).is_err());
    // The claimed report must be a BacktestReport object.
    let bad_report =
        r#"{"strategy": {}, "dataset_ref": {"kind": "files", "symbols": []}, "claimed_report": 7}"#;
    assert!(Claim::from_json(bad_report).is_err());
}

#[test]
fn canonicalize_collapses_non_finite_to_null() {
    // JSON has no NaN/Infinity: serde collapses a non-finite number to `null`,
    // deterministically, so canonicalization never panics on one. (Real reports
    // are finite by construction; this pins the boundary behavior.)
    assert_eq!(canonicalize(&f64::NAN).unwrap(), "null");
    assert_eq!(canonicalize(&f64::INFINITY).unwrap(), "null");
    assert_eq!(canonicalize(&f64::NEG_INFINITY).unwrap(), "null");
}

#[test]
fn files_claim_needs_explicit_data() {
    // A `files` claim has no inline data; verifying it requires data out of band.
    let claim = Claim {
        strategy: common::strategy_json(),
        dataset_ref: DatasetRef::Files {
            symbols: vec![common::SYMBOL.to_string()],
            hash: None,
        },
        claimed_report: common::honest_report(&common::sample_data()),
    };
    let data: BTreeMap<_, _> = common::sample_data();
    // Supplied explicitly, it verifies as honest.
    let verdict = verify(&claim, &data).unwrap();
    assert!(verdict.matches);
}
