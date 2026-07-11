#![no_main]
//! Fuzz the field-by-field comparator. Arbitrary bytes are parsed as one or two
//! JSON values and compared under a range of tolerances. `compare` must never
//! panic on arbitrary nested JSON, must be reflexive (a value compared to itself
//! yields no mismatch), and must be order-insensitive on its arguments up to the
//! sign of the reported delta.

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use verify_core::compare;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    // Accept either a two-element `[a, b]` array (two reports) or a single value
    // (compared against itself).
    let (a, b): (Value, Value) = match serde_json::from_str::<Value>(text) {
        Ok(Value::Array(mut items)) if items.len() == 2 => {
            let b = items.pop().unwrap();
            let a = items.pop().unwrap();
            (a, b)
        }
        Ok(value) => (value.clone(), value),
        Err(_) => return,
    };

    // Never panics on arbitrary nested JSON.
    let ab = compare(&a, &b, 1e-9, 1e-6);
    let ba = compare(&b, &a, 1e-9, 1e-6);

    // Reflexivity: a value compared to itself has no mismatches.
    assert!(compare(&a, &a, 1e-9, 1e-6).is_empty());
    assert!(compare(&b, &b, 1e-9, 1e-6).is_empty());

    // The set of diverging fields is independent of argument order.
    let mut fields_ab: Vec<&str> = ab.iter().map(|m| m.field.as_str()).collect();
    let mut fields_ba: Vec<&str> = ba.iter().map(|m| m.field.as_str()).collect();
    fields_ab.sort_unstable();
    fields_ba.sort_unstable();
    assert_eq!(
        fields_ab, fields_ba,
        "compare is not argument-order symmetric"
    );

    // A looser tolerance never reports more mismatches than a tighter one.
    let loose = compare(&a, &b, 1e3, 1.0);
    assert!(
        loose.len() <= ab.len(),
        "a looser tolerance flagged more fields"
    );
});
