//! [`Claim`] — the assertion to be checked: "this strategy on this data produced
//! this report."

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub use wickra_backtest_core::{BacktestReport, Candle, StrategySpec};

/// Where the candle data behind a claim comes from.
///
/// `Inline` carries the candles verbatim, so a claim is fully self-contained and
/// portable. `Files` names the symbols (and an optional content hash) the caller
/// must resolve out of band — the verifier never fetches data itself. The tag is
/// the `kind` field: `{"kind":"inline","data":{...}}` /
/// `{"kind":"files","symbols":[...]}`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetRef {
    /// The candles, embedded per symbol.
    Inline {
        /// Candle series keyed by symbol.
        data: BTreeMap<String, Vec<Candle>>,
    },
    /// A reference to external files the caller resolves.
    Files {
        /// The symbols the claim needs.
        symbols: Vec<String>,
        /// Optional content hash of the referenced data, for provenance.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hash: Option<String>,
    },
}

/// A claim to verify: a strategy, the data it ran on, and the report it is said
/// to have produced. `claimed_report` is untrusted — verification recomputes the
/// report from `strategy` + data and compares, so a doctored `claimed_report`
/// cannot pass.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// The embedded wickra-backtest `StrategySpec`, kept as raw JSON so
    /// verify-core stays decoupled from backtest struct internals across the FFI
    /// boundary.
    pub strategy: Value,
    /// Where the candle data comes from.
    pub dataset_ref: DatasetRef,
    /// The report the claimant asserts this run produced (untrusted).
    pub claimed_report: Value,
}

impl Claim {
    /// Parse a `Claim` from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        let claim: Self = serde_json::from_str(s).map_err(|e| Error::BadClaim(e.to_string()))?;
        claim.validate()?;
        Ok(claim)
    }

    /// Parse a `Claim` from TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        let claim: Self = toml::from_str(s).map_err(|e| Error::Parse(e.to_string()))?;
        claim.validate()?;
        Ok(claim)
    }

    /// The inline candle data, if this claim carries it. `Files` claims return
    /// `None` — their data must be supplied to [`verify`](crate::verify)
    /// explicitly.
    #[must_use]
    pub fn inline_data(&self) -> Option<&BTreeMap<String, Vec<Candle>>> {
        match &self.dataset_ref {
            DatasetRef::Inline { data } => Some(data),
            DatasetRef::Files { .. } => None,
        }
    }

    /// Validate structural invariants: the embedded strategy must be a JSON
    /// object (a `StrategySpec`), and the claimed report a JSON object.
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.strategy.is_object() {
            return Err(Error::BadClaim(
                "strategy must be a JSON object (a StrategySpec)".to_string(),
            ));
        }
        if !self.claimed_report.is_object() {
            return Err(Error::BadReport(
                "claimed_report must be a JSON object (a BacktestReport)".to_string(),
            ));
        }
        Ok(())
    }
}
