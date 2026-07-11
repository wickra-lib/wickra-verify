//! Property tests: over a wide range of bounded, valid candle universes the
//! verify contract never panics, an honest claim always verifies with zero
//! mismatches, a single doctored field yields exactly one mismatch, and
//! canonicalization is idempotent and key-order invariant.

mod common;

use proptest::prelude::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use verify_core::{canonicalize, verify, Claim, DatasetRef};

fn data_from(closes: &[f64]) -> BTreeMap<String, Vec<verify_core::Candle>> {
    let mut data = BTreeMap::new();
    data.insert(common::SYMBOL.to_string(), common::candles_from(closes));
    data
}

fn claim_with(report: Value, data: &BTreeMap<String, Vec<verify_core::Candle>>) -> Claim {
    Claim {
        strategy: common::strategy_json(),
        dataset_ref: DatasetRef::Inline { data: data.clone() },
        claimed_report: report,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// An honest claim over any bounded universe verifies with zero mismatches.
    #[test]
    fn honest_claim_verifies(closes in prop::collection::vec(50.0f64..200.0f64, 16..64)) {
        let data = data_from(&closes);
        let report = common::honest_report(&data);
        let verdict = verify(&claim_with(report, &data), &data).unwrap();
        prop_assert!(verdict.matches);
        prop_assert!(verdict.mismatches.is_empty());
        // Hashes are 64-hex blake3 digests, and an honest claim's claimed and
        // actual report hashes coincide.
        prop_assert_eq!(verdict.claimed_report_hash.len(), 64);
        prop_assert_eq!(&verdict.claimed_report_hash, &verdict.actual_report_hash);
    }

    /// Perturbing exactly one leaf beyond tolerance yields exactly one mismatch.
    #[test]
    fn one_doctored_field_yields_one_mismatch(
        closes in prop::collection::vec(50.0f64..200.0f64, 16..64),
        bump in 1.0f64..1000.0f64,
    ) {
        let data = data_from(&closes);
        let mut report = common::honest_report(&data);
        let honest_cap = report["initial_capital"].as_f64().unwrap();
        report["initial_capital"] = json!(honest_cap + bump);

        let verdict = verify(&claim_with(report, &data), &data).unwrap();
        prop_assert!(!verdict.matches);
        prop_assert_eq!(verdict.mismatches.len(), 1);
        prop_assert_eq!(&verdict.mismatches[0].field, "initial_capital");
    }

    /// Canonicalization is idempotent: re-canonicalizing a canonical string's
    /// parse yields the identical string.
    #[test]
    fn canonicalize_is_idempotent(closes in prop::collection::vec(50.0f64..200.0f64, 16..48)) {
        let data = data_from(&closes);
        let report = common::honest_report(&data);
        let once = canonicalize(&report).unwrap();
        let reparsed: Value = serde_json::from_str(&once).unwrap();
        let twice = canonicalize(&reparsed).unwrap();
        prop_assert_eq!(once, twice);
    }
}

#[test]
fn canonicalize_is_key_order_invariant() {
    // Independent of the branching above: shuffled key order canonicalizes equal.
    let a = json!({ "z": 1, "a": { "n": 2, "m": 3 }, "k": [3, 2, 1] });
    let b = json!({ "k": [3, 2, 1], "a": { "m": 3, "n": 2 }, "z": 1 });
    assert_eq!(canonicalize(&a).unwrap(), canonicalize(&b).unwrap());
}
