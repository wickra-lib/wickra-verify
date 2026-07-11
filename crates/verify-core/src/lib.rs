//! # verify-core
//!
//! Verify any backtest: given a [`Claim`] — a strategy, the candle data it ran
//! on, and the report it is *said* to have produced — recompute the report with
//! the pinned `wickra-backtest` engine and confirm or refute the claim field by
//! field. A doctored `claimed_report` cannot pass, because verification
//! recomputes rather than trusting the supplied numbers.
//!
//! The result is a [`Verdict`]: `matches` plus every [`Mismatch`], the engine
//! version, and three blake3 hashes. Canonicalization is byte-for-byte identical
//! to `wickra-proof`, so a verdict's `inputs_hash` equals the proof hash of the
//! same inputs — a verify result and a proof are cross-checkable.
//!
//! [`canonicalize`]/[`hash`] are the single source of hash stability; [`verify`]
//! and [`explain`] are the core operations; [`Verifier`] exposes the same
//! `command_json` boundary the ten language bindings forward verbatim.

mod canon;
mod claim;
mod compare;
mod config;
mod error;
mod verdict;
mod verify;

pub use canon::{canonicalize, hash};
pub use claim::{BacktestReport, Candle, Claim, DatasetRef, StrategySpec};
pub use compare::compare;
pub use config::Config;
pub use error::{Error, Result};
pub use verdict::{Mismatch, Verdict};
pub use verify::{explain, verify, Verifier};

/// The verify-core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests;
