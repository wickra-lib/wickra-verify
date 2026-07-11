#![no_main]
//! Fuzz the parsing surface: arbitrary bytes are parsed as a `Claim` (JSON and
//! TOML) and as a verifier `Config`. None must panic; malformed input must
//! surface as a clean `Err`, never a crash.

use libfuzzer_sys::fuzz_target;
use verify_core::{Claim, Verifier};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    // A claim parses or errors cleanly from both encodings.
    let _ = Claim::from_json(text);
    let _ = Claim::from_toml(text);
    // The verifier config (`{"atol":..,"rtol":..}`) parses or errors cleanly.
    let _ = Verifier::new(text);
});
