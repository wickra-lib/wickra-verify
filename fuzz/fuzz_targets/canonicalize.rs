#![no_main]
//! Fuzz the canonicalizer — the determinism moat verify-core shares with
//! wickra-proof. Arbitrary bytes are parsed as a JSON value and canonicalized.
//! The canonical form must never panic, must be idempotent, and must never leak
//! a non-finite token that was not present verbatim in the input.

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use verify_core::canonicalize;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return;
    };
    let canonical = canonicalize(&value).expect("canonicalization of a parsed value is total");

    // No non-finite token can appear unless it was a quoted string in the input:
    // quantization keeps every number finite, and a bare NaN/Infinity is never
    // emitted for a number.
    for token in ["NaN", "Infinity", "-Infinity"] {
        assert!(
            !canonical.contains(token) || value.to_string().contains(token),
            "canonical form leaked a non-finite token not present in the input"
        );
    }

    // Idempotence: re-parsing and re-canonicalizing is stable.
    if let Ok(reparsed) = serde_json::from_str::<Value>(&canonical) {
        assert_eq!(canonicalize(&reparsed).unwrap(), canonical);
    }
});
