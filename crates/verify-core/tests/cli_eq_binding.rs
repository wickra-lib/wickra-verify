//! The free functions and the command surface agree. `verify()` and the
//! `{"cmd":"verify"}` command must produce the same canonical verdict, and
//! `explain()` must produce the same text the `{"cmd":"explain"}` command
//! returns — this is what lets the CLI (free functions) and the ten bindings
//! (command surface) be interchangeable.

mod common;

use serde_json::{json, Value};
use verify_core::{canonicalize, explain, verify, Verifier};

fn verify_via_command(claim: &verify_core::Claim, data: &Value) -> String {
    let cmd = json!({ "cmd": "verify", "claim": claim, "data": data }).to_string();
    Verifier::default().command_json(&cmd).unwrap()
}

#[test]
fn free_verify_equals_command_verify_honest() {
    let data = common::sample_data();
    let data_value = serde_json::to_value(&data).unwrap();
    let claim = common::honest_claim(&data);

    let verdict = verify(&claim, &data).unwrap();
    let free = canonicalize(&verdict).unwrap();
    let via_command = verify_via_command(&claim, &data_value);

    assert_eq!(free, via_command);
    assert!(verdict.matches);
}

#[test]
fn free_verify_equals_command_verify_doctored() {
    let data = common::sample_data();
    let data_value = serde_json::to_value(&data).unwrap();
    let mut claim = common::honest_claim(&data);
    claim.claimed_report["fees_paid"] = json!(123_456.0);

    let verdict = verify(&claim, &data).unwrap();
    let free = canonicalize(&verdict).unwrap();
    let via_command = verify_via_command(&claim, &data_value);

    assert_eq!(free, via_command);
    assert!(!verdict.matches);
    assert_eq!(verdict.mismatches.len(), 1);
    assert_eq!(verdict.mismatches[0].field, "fees_paid");
}

#[test]
fn free_explain_equals_command_explain() {
    let data = common::sample_data();
    let mut claim = common::honest_claim(&data);
    claim.claimed_report["metrics"]["return_pct"] = json!(9999.0);
    let verdict = verify(&claim, &data).unwrap();

    let free_text = explain(&verdict);

    let cmd = json!({ "cmd": "explain", "verdict": verdict }).to_string();
    let out: Value =
        serde_json::from_str(&Verifier::default().command_json(&cmd).unwrap()).unwrap();
    assert_eq!(out["ok"], json!(true));
    assert_eq!(out["text"].as_str().unwrap(), free_text);
    assert!(free_text.starts_with("REFUTED"));
}

#[test]
fn command_canonicalize_matches_free_canonicalize() {
    // The `canonicalize` command wraps the free `canonicalize`.
    let value = json!({ "b": 2, "a": 1 });
    let cmd = json!({ "cmd": "canonicalize", "value": value }).to_string();
    let out: Value =
        serde_json::from_str(&Verifier::default().command_json(&cmd).unwrap()).unwrap();
    assert_eq!(
        out["canonical"].as_str().unwrap(),
        canonicalize(&value).unwrap()
    );
}
