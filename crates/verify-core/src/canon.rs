//! Deterministic JSON canonicalization — the single source of a verdict hash's
//! stability across all ten language bindings.
//!
//! This is byte-for-byte the same canonicalization `wickra-proof` uses, on
//! purpose: a verdict's `inputs_hash` over `(strategy, data)` must equal the
//! proof hash the same inputs would produce, so a verify result and a proof are
//! cross-checkable. The rules (each normative):
//!
//! 1. Object keys are sorted ascending by Unicode code point (`BTreeMap`).
//! 2. No whitespace: `,` and `:` separators, nothing else.
//! 3. Every float is quantized to 1e-8 by decimal rounding (`{:.8}`), trailing
//!    zeros trimmed, a whole-valued float collapsed to its integer token
//!    (`1.0` -> `"1"`) because JSON in most host languages cannot preserve the
//!    `.0`. Magnitudes at or above `2^52 * 1e-8` use the shortest round-trippable
//!    form instead, keeping canonicalization a fixed point.
//! 4. `NaN` / `±inf` cannot occur: `serde_json` never yields them from a parsed
//!    or `to_value`-converted finite report.
//! 5. Strings use `serde_json`'s standard escaping.
//! 6. Array order is preserved (it is meaning-bearing).

use crate::error::{Error, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Format a float in a fixed, cross-language-stable, idempotent decimal form:
/// eight fractional digits, trailing zeros trimmed, a whole value collapsed to
/// its bare integer token (`1.0` -> `"1"`). A host language cannot distinguish
/// `1.0` from `1` in JSON, so the integer form is the only one every language
/// can reproduce. Negative zero normalizes to `0`.
///
/// Quantization to 1e-8 is done purely by `{:.8}` decimal rounding — no separate
/// binary-grid step. Rounding to eight decimals is a fixed point only while the
/// 1e-8 grid is coarser than the f64 ULP, i.e. `|x| < 2^52 * 1e-8`. At or above
/// that magnitude the grid is finer than f64 can represent, so emit the shortest
/// round-trippable form (`Display`, always positional for f64) instead, which
/// re-parses to the same value under `serde_json`'s `float_roundtrip` parser.
fn format_f64(x: f64) -> String {
    // 2^52 * 1e-8: the magnitude at which the f64 ULP first reaches the 1e-8
    // grid. Below it, decimal rounding to eight places is a fixed point.
    const GRID_RESOLUTION_LIMIT: f64 = 45_035_996.273_704_96;
    let x = if x == 0.0 { 0.0 } else { x };
    if x.abs() >= GRID_RESOLUTION_LIMIT {
        return format!("{x}");
    }
    let mut s = format!("{x:.8}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    // A small negative that rounds to zero at eight decimals formats as
    // `-0.00000000`, which trims to `-0`. Re-parsing `-0` yields `-0.0`, which
    // the `x == 0.0` guard collapses to `0` — so the signed form would break
    // idempotence. Drop the sign whenever the rounded value is zero.
    if s == "-0" {
        s.clear();
        s.push('0');
    }
    s
}

fn write_number(out: &mut String, n: &serde_json::Number) {
    if let Some(i) = n.as_i64() {
        write!(out, "{i}").expect("writing to a String is infallible");
    } else if let Some(u) = n.as_u64() {
        write!(out, "{u}").expect("writing to a String is infallible");
    } else {
        // Neither i64 nor u64 -> a finite f64 (serde_json rejects NaN/inf at
        // parse time and never yields an unrepresentable number here).
        let f = n.as_f64().unwrap_or(0.0);
        out.push_str(&format_f64(f));
    }
}

fn write_string(out: &mut String, s: &str) {
    // serde_json's string serializer is the reference escaping; reuse it.
    let encoded = Value::String(s.to_string()).to_string();
    out.push_str(&encoded);
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => write_number(out, n),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let sorted: BTreeMap<&String, &Value> = map.iter().collect();
            out.push('{');
            for (i, (key, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, val);
            }
            out.push('}');
        }
    }
}

/// The canonical, whitespace-free, key-sorted string form of a `serde_json`
/// value. Byte-identical across languages; the input to the blake3 hash.
fn canonicalize_value(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

/// Produce the canonical string form of any serializable value (a report, a
/// verdict, or the `(strategy, data)` inputs). The value is first converted to a
/// `serde_json::Value` and then canonicalized, so the output is independent of
/// struct field order and depends only on the data.
pub fn canonicalize<T: Serialize>(value: &T) -> Result<String> {
    let json = serde_json::to_value(value).map_err(|e| Error::BadReport(e.to_string()))?;
    Ok(canonicalize_value(&json))
}

/// The lowercase 64-hex blake3 of a canonical string (no prefix). Pairs with
/// [`canonicalize`]: `hash(&canonicalize(x)?)` is the stable digest of `x`.
pub fn hash(canonical: &str) -> String {
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

/// Round `x` to a multiple of quantum `q` (`round_to(x, q) = (x / q).round() * q`).
///
/// Used only for the human-facing `delta` field of a [`Mismatch`](crate::Mismatch):
/// a tidy rounded difference to display, never part of the canonical hash. `q`
/// must be non-zero; a zero quantum returns `x` unchanged.
pub fn round_to(x: f64, q: f64) -> f64 {
    if q == 0.0 {
        return x;
    }
    (x / q).round() * q
}
