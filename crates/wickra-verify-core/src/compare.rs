//! Field-by-field tolerance comparison of two backtest reports.
//!
//! Both reports are the same `BacktestReport` type, so their JSON trees have the
//! same shape — every object has the same keys. The only shape divergence that
//! can occur is array length (a claim asserting a different number of trades or
//! equity points than the recomputed run), which is itself a mismatch worth
//! reporting. Numeric leaves are compared with a mixed absolute/relative
//! tolerance; non-numeric leaves (strings, bools, nulls) are structurally
//! identical by construction and skipped.

use crate::canon::round_to;
use crate::verdict::Mismatch;
use serde_json::Value;

/// The 1e-8 display grid for the human-facing `delta` field.
const DISPLAY_QUANTUM: f64 = 1e-8;

/// Two floats are equal when `|a - b| <= atol + rtol * max(|a|, |b|)`.
fn close(a: f64, b: f64, atol: f64, rtol: f64) -> bool {
    (a - b).abs() <= atol + rtol * a.abs().max(b.abs())
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn walk(
    path: &str,
    claimed: &Value,
    actual: &Value,
    atol: f64,
    rtol: f64,
    out: &mut Vec<Mismatch>,
) {
    match (claimed, actual) {
        (Value::Number(a), Value::Number(b)) => {
            // Both parsed from the same numeric field; as_f64 is total here.
            let (a, b) = (
                a.as_f64().unwrap_or(f64::NAN),
                b.as_f64().unwrap_or(f64::NAN),
            );
            if !close(a, b, atol, rtol) {
                out.push(Mismatch {
                    field: path.to_string(),
                    claimed: a,
                    actual: b,
                    delta: round_to(b - a, DISPLAY_QUANTUM),
                });
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                let (la, lb) = (a.len() as f64, b.len() as f64);
                out.push(Mismatch {
                    field: format!("{path}[len]"),
                    claimed: la,
                    actual: lb,
                    delta: round_to(lb - la, DISPLAY_QUANTUM),
                });
            }
            for (i, (ai, bi)) in a.iter().zip(b.iter()).enumerate() {
                walk(&format!("{path}[{i}]"), ai, bi, atol, rtol, out);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            // Same struct type -> same keys; `b.get` is always `Some`.
            for (key, av) in a {
                if let Some(bv) = b.get(key) {
                    walk(&join(path, key), av, bv, atol, rtol, out);
                }
            }
        }
        // Non-numeric, structurally identical leaves: nothing to compare.
        _ => {}
    }
}

/// Compare `claimed` against `actual` (both a report as a JSON value) field by
/// field, returning every numeric disagreement (and any array-length
/// disagreement) beyond the given tolerance, stably sorted by field path. An
/// empty result means the claim holds.
#[must_use]
pub fn compare(claimed: &Value, actual: &Value, atol: f64, rtol: f64) -> Vec<Mismatch> {
    let mut out = Vec::new();
    walk("", claimed, actual, atol, rtol, &mut out);
    out.sort_by(|x, y| x.field.cmp(&y.field));
    out
}
