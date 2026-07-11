//! The single error type of the verify core.
//!
//! Every fallible entry point returns [`Result`]. The variants map one-to-one to
//! the ways verification can fail before a verdict is even reached: a malformed
//! claim, a malformed report, unusable data, or the backtest engine refusing to
//! run. A *refuted* claim is **not** an error — that is a successful [`Verdict`]
//! with `matches == false`.
//!
//! [`Verdict`]: crate::Verdict

use thiserror::Error;

/// The result of a verify-core operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong while verifying a claim.
#[derive(Debug, Error)]
pub enum Error {
    /// The claim could not be parsed or is internally inconsistent (unknown
    /// field, missing strategy, empty dataset reference).
    #[error("invalid claim: {0}")]
    BadClaim(String),

    /// The claimed report could not be parsed, serialized, or canonicalized.
    #[error("invalid report: {0}")]
    BadReport(String),

    /// The referenced candle data is missing, empty, or does not match the
    /// symbols the claim requires.
    #[error("data error: {0}")]
    Data(String),

    /// The backtest engine refused to run on the given strategy and data.
    #[error("backtest engine error: {0}")]
    Backtest(String),

    /// A command envelope (or its embedded JSON) could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),
}
