//! [`Verdict`] — the reproducible answer: does the claimed report hold up?

use serde::{Deserialize, Serialize};

/// A single field where the claimed report disagrees with the recomputed one,
/// beyond tolerance. `field` is a dotted/indexed path (`metrics.sharpe`,
/// `trades[3].pnl`, `trades[len]`). `delta` is `actual - claimed`, rounded to the
/// 1e-8 display grid — human-facing only, never part of a hash.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Mismatch {
    /// Dotted/indexed path to the disagreeing field.
    pub field: String,
    /// The value the claim asserted.
    pub claimed: f64,
    /// The value recomputation produced.
    pub actual: f64,
    /// `actual - claimed`, rounded for display.
    pub delta: f64,
}

/// The result of verifying a claim. Deterministic and self-contained: the same
/// `(claim, data)` yields a byte-identical verdict in every language, and the
/// three hashes let a verdict be cross-checked against a `wickra-proof` proof of
/// the same inputs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Verdict {
    /// `true` iff every numeric field of the claimed report agrees with the
    /// recomputed one within tolerance (`mismatches` is empty).
    pub matches: bool,
    /// Every field that disagreed, stably sorted by `field`.
    pub mismatches: Vec<Mismatch>,
    /// The exact backtest engine version that recomputed the report.
    pub engine_version: String,
    /// blake3 hex of the canonicalized claimed report.
    pub claimed_report_hash: String,
    /// blake3 hex of the canonicalized recomputed report.
    pub actual_report_hash: String,
    /// blake3 hex of `canonicalize({strategy, dataset_ref, candles,
    /// engine_version})` — the same inputs hash `wickra-proof` produces.
    pub inputs_hash: String,
}
