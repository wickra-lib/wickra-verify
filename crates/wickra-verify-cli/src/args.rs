//! CLI argument parsing.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Verify a claimed backtest report by recomputing it.
///
/// Exit codes: `0` when the claim is verified, `2` when it is refuted (so a
/// doctored claim turns a CI build red), `1` on any error. `--explain` always
/// exits `0`.
#[derive(Parser, Debug)]
#[command(name = "wickra-verify", version, about)]
pub struct Cli {
    /// Path to the claim (JSON or TOML, chosen by extension).
    #[arg(long)]
    pub claim: PathBuf,

    /// Directory of `<SYMBOL>.csv` candle files. Required for `files` claims;
    /// optional for `inline` claims (their embedded data is used instead).
    #[arg(long)]
    pub data: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,

    /// Absolute tolerance override (default `1e-9`).
    #[arg(long)]
    pub atol: Option<f64>,

    /// Relative tolerance override (default `1e-6`).
    #[arg(long)]
    pub rtol: Option<f64>,

    /// Print the human-readable explanation and always exit `0`, even when the
    /// claim is refuted.
    #[arg(long)]
    pub explain: bool,
}

/// The output format.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Format {
    /// A human-readable verdict (the `explain` text).
    Text,
    /// The full `Verdict` as JSON.
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn arg_config_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn claim_is_required() {
        assert!(Cli::try_parse_from(["wickra-verify"]).is_err());
    }

    #[test]
    fn defaults_are_text_and_no_data() {
        let cli = Cli::try_parse_from(["wickra-verify", "--claim", "c.json"]).unwrap();
        assert_eq!(cli.format, Format::Text);
        assert!(cli.data.is_none());
        assert!(!cli.explain);
    }

    #[test]
    fn tolerances_and_format_parse() {
        let cli = Cli::try_parse_from([
            "wickra-verify",
            "--claim",
            "c.json",
            "--format",
            "json",
            "--atol",
            "1e-6",
            "--rtol",
            "1e-3",
            "--explain",
        ])
        .unwrap();
        assert_eq!(cli.format, Format::Json);
        assert!(cli.explain);
        assert!((cli.atol.unwrap() - 1e-6).abs() < f64::EPSILON);
        assert!((cli.rtol.unwrap() - 1e-3).abs() < f64::EPSILON);
    }
}
