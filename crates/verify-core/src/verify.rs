//! `verify` / `explain` and the [`Verifier`] command-JSON handle.

use crate::canon::{canonicalize, hash};
use crate::claim::Claim;
use crate::compare::compare;
use crate::config::{Config, DEFAULT_ATOL, DEFAULT_RTOL};
use crate::error::{Error, Result};
use crate::verdict::Verdict;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use wickra_backtest_core::{run, version as engine_version, Candle, StrategySpec};

/// Recompute the report from `claim.strategy` + `data` and compare it, field by
/// field, against `claim.claimed_report` under the given tolerances.
fn verify_with(
    claim: &Claim,
    data: &BTreeMap<String, Vec<Candle>>,
    atol: f64,
    rtol: f64,
) -> Result<Verdict> {
    // Structural validation (strategy + claimed_report are objects). Callers that
    // build a Claim via from_json already ran this, but the command-JSON path
    // deserializes a Claim directly, so re-check here.
    claim.validate()?;
    let engine = engine_version().to_string();

    let strategy: StrategySpec = serde_json::from_value(claim.strategy.clone())
        .map_err(|e| Error::BadClaim(e.to_string()))?;
    let candles = data
        .get(&strategy.symbol)
        .ok_or_else(|| Error::Data(format!("no candles for symbol {}", strategy.symbol)))?;

    let actual = run(&strategy, candles).map_err(|e| Error::Backtest(e.to_string()))?;
    let actual_value =
        serde_json::to_value(&actual).map_err(|e| Error::BadReport(e.to_string()))?;

    let mismatches = compare(&claim.claimed_report, &actual_value, atol, rtol);

    let claimed_report_hash = hash(&canonicalize(&claim.claimed_report)?);
    let actual_report_hash = hash(&canonicalize(&actual_value)?);
    let inputs = json!({
        "strategy": claim.strategy,
        "dataset_ref": claim.dataset_ref,
        "candles": serde_json::to_value(data).map_err(|e| Error::Data(e.to_string()))?,
        "engine_version": engine,
    });
    let inputs_hash = hash(&canonicalize(&inputs)?);

    Ok(Verdict {
        matches: mismatches.is_empty(),
        mismatches,
        engine_version: engine,
        claimed_report_hash,
        actual_report_hash,
        inputs_hash,
    })
}

/// Verify a claim against candle data with the default tolerances
/// (`atol = 1e-9`, `rtol = 1e-6`). Recomputes the report and compares — a
/// doctored `claimed_report` is refuted, an honest one is confirmed.
pub fn verify(claim: &Claim, data: &BTreeMap<String, Vec<Candle>>) -> Result<Verdict> {
    verify_with(claim, data, DEFAULT_ATOL, DEFAULT_RTOL)
}

/// A human-readable, deterministic one-shot summary of a verdict.
#[must_use]
pub fn explain(verdict: &Verdict) -> String {
    if verdict.matches {
        return format!(
            "VERIFIED: claimed report matches the recomputed report (engine {}).",
            verdict.engine_version
        );
    }
    let mut lines = format!(
        "REFUTED: {} field(s) disagree with the recomputed report (engine {}):",
        verdict.mismatches.len(),
        verdict.engine_version
    );
    for m in &verdict.mismatches {
        write!(
            lines,
            "\n  {}: claimed {}, actual {} (delta {})",
            m.field, m.claimed, m.actual, m.delta
        )
        .expect("writing to a String is infallible");
    }
    lines
}

/// Command-JSON handle carrying the comparison tolerances. Holds no other state,
/// but is handle-shaped so the ten language bindings share the same surface as
/// screener/proof/terminal.
#[derive(Debug, Clone, Copy)]
pub struct Verifier {
    atol: f64,
    rtol: f64,
}

impl Default for Verifier {
    fn default() -> Self {
        Self {
            atol: DEFAULT_ATOL,
            rtol: DEFAULT_RTOL,
        }
    }
}

#[derive(Deserialize)]
struct VerifyReq {
    claim: Claim,
    #[serde(default)]
    data: Option<BTreeMap<String, Vec<Candle>>>,
}

#[derive(Deserialize)]
struct ExplainReq {
    verdict: Verdict,
}

impl Verifier {
    /// Build a verifier from a `Config` JSON string (`{"atol":..,"rtol":..}`).
    /// Missing fields fall back to the defaults.
    pub fn new(config_json: &str) -> Result<Self> {
        let config: Config =
            serde_json::from_str(config_json).map_err(|e| Error::Parse(e.to_string()))?;
        Ok(Self {
            atol: config.atol,
            rtol: config.rtol,
        })
    }

    /// The verify-core crate version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Verify a claim against candle data with this verifier's tolerances.
    pub fn verify(&self, claim: &Claim, data: &BTreeMap<String, Vec<Candle>>) -> Result<Verdict> {
        verify_with(claim, data, self.atol, self.rtol)
    }

    /// Dispatch a command envelope `{"cmd": ...}` and return a canonical JSON
    /// string. Unknown commands and errors return an error envelope, never a
    /// panic.
    pub fn command_json(&self, cmd_json: &str) -> Result<String> {
        let value = dispatch(self, cmd_json);
        canonicalize(&value)
    }
}

fn dispatch(verifier: &Verifier, cmd_json: &str) -> Value {
    match dispatch_inner(verifier, cmd_json) {
        Ok(v) => v,
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

fn dispatch_inner(verifier: &Verifier, cmd_json: &str) -> Result<Value> {
    let env: Value = serde_json::from_str(cmd_json).map_err(|e| Error::Parse(e.to_string()))?;
    let cmd = env.get("cmd").and_then(Value::as_str).unwrap_or("");
    match cmd {
        "verify" => {
            let req: VerifyReq =
                serde_json::from_value(env).map_err(|e| Error::Parse(e.to_string()))?;
            // Fall back to the claim's inline data when none is supplied.
            let verdict = if let Some(data) = req.data {
                verifier.verify(&req.claim, &data)?
            } else {
                let data = req.claim.inline_data().ok_or_else(|| {
                    Error::Data("no data supplied and claim is not inline".to_string())
                })?;
                verifier.verify(&req.claim, data)?
            };
            serde_json::to_value(verdict).map_err(|e| Error::BadReport(e.to_string()))
        }
        "explain" => {
            let req: ExplainReq =
                serde_json::from_value(env).map_err(|e| Error::Parse(e.to_string()))?;
            Ok(json!({ "ok": true, "text": explain(&req.verdict) }))
        }
        "canonicalize" => {
            let value = env.get("value").cloned().unwrap_or(Value::Null);
            Ok(json!({ "ok": true, "canonical": canonicalize(&value)? }))
        }
        "version" => Ok(json!({
            "version": Verifier::version(),
            "engine_version": engine_version(),
        })),
        other => Err(Error::Parse(format!("unknown cmd: {other}"))),
    }
}
