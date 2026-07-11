//! Verification tolerances.
//!
//! Two floats count as equal when `|a - b| <= atol + rtol * max(|a|, |b|)` — the
//! same mixed absolute/relative rule `numpy.allclose` uses. A pure absolute
//! tolerance would reject large-magnitude metrics that differ only in the last
//! representable digit; a pure relative one would reject values legitimately near
//! zero. The defaults are deliberately tight: they absorb the last-bit float
//! noise of two independent but deterministic computations of the *same* report,
//! nothing more. A doctored metric moves far more than `1e-6` relative.

use serde::{Deserialize, Serialize};

/// Absolute tolerance component (default `1e-9`).
pub const DEFAULT_ATOL: f64 = 1e-9;
/// Relative tolerance component (default `1e-6`).
pub const DEFAULT_RTOL: f64 = 1e-6;

/// The comparison tolerances a [`Verifier`](crate::Verifier) is built with.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Absolute tolerance.
    #[serde(default = "default_atol")]
    pub atol: f64,
    /// Relative tolerance.
    #[serde(default = "default_rtol")]
    pub rtol: f64,
}

fn default_atol() -> f64 {
    DEFAULT_ATOL
}

fn default_rtol() -> f64 {
    DEFAULT_RTOL
}

impl Default for Config {
    fn default() -> Self {
        Self {
            atol: DEFAULT_ATOL,
            rtol: DEFAULT_RTOL,
        }
    }
}
