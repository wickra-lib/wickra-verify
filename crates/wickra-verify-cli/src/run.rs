//! Load inputs, run the verification, and render the output.

use crate::args::{Cli, Format};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use verify_core::{explain, Candle, Claim, Config, Verifier};

/// The rendered output plus the process exit code (0 verified, 2 refuted).
pub struct Output {
    /// Text written to standard output.
    pub text: String,
    /// The process exit code.
    pub code: u8,
}

/// Load the claim and data, verify, and render.
pub fn run(cli: &Cli) -> Result<Output, String> {
    let claim = load_claim(&cli.claim)?;
    let verifier = build_verifier(cli.atol, cli.rtol)?;

    // Data source: an explicit `--data` directory, else the claim's inline data.
    let loaded;
    let data: &BTreeMap<String, Vec<Candle>> = if let Some(dir) = &cli.data {
        loaded = load_data(dir)?;
        &loaded
    } else {
        claim
            .inline_data()
            .ok_or_else(|| "claim has no inline data; pass --data <dir>".to_string())?
    };

    let verdict = verifier.verify(&claim, data).map_err(|e| e.to_string())?;

    let text = if cli.format == Format::Json && !cli.explain {
        let mut json = serde_json::to_string(&verdict).map_err(|e| e.to_string())?;
        json.push('\n');
        json
    } else {
        let mut text = explain(&verdict);
        text.push('\n');
        text
    };

    // Refuted claims exit 2 (CI-friendly), unless `--explain` forces success.
    let code = u8::from(!(cli.explain || verdict.matches)) * 2;
    Ok(Output { text, code })
}

/// Build a verifier from optional tolerance overrides.
fn build_verifier(atol: Option<f64>, rtol: Option<f64>) -> Result<Verifier, String> {
    let mut config = Config::default();
    if let Some(a) = atol {
        config.atol = a;
    }
    if let Some(r) = rtol {
        config.rtol = r;
    }
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    Verifier::new(&json).map_err(|e| e.to_string())
}

/// Read a file to a string with a contextual error.
fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))
}

/// Read and parse a claim file, choosing JSON or TOML by extension.
fn load_claim(path: &Path) -> Result<Claim, String> {
    let content = read(path)?;
    let is_toml = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    let claim = if is_toml {
        Claim::from_toml(&content)
    } else {
        Claim::from_json(&content)
    };
    claim.map_err(|e| e.to_string())
}

/// Load candles from a directory of `<SYMBOL>.csv` files.
fn load_data(path: &Path) -> Result<BTreeMap<String, Vec<Candle>>, String> {
    let mut data = BTreeMap::new();
    if path.is_dir() {
        let entries =
            fs::read_dir(path).map_err(|e| format!("read dir {}: {e}", path.display()))?;
        for entry in entries {
            let file = entry.map_err(|e| e.to_string())?.path();
            if file.extension().and_then(|e| e.to_str()) != Some("csv") {
                continue;
            }
            data.insert(symbol_of(&file)?, parse_csv(&read(&file)?)?);
        }
    } else {
        data.insert(symbol_of(path)?, parse_csv(&read(path)?)?);
    }
    Ok(data)
}

/// The symbol name is the file stem (`AAA.csv` -> `AAA`).
fn symbol_of(path: &Path) -> Result<String, String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("bad file name: {}", path.display()))
}

/// Parse OHLCV rows (`ts,open,high,low,close,volume`) into candles; a
/// non-numeric first row is treated as a header and skipped.
fn parse_csv(content: &str) -> Result<Vec<Candle>, String> {
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() < 6 {
            return Err(format!(
                "CSV line {}: expected 6 columns, got {}",
                idx + 1,
                cols.len()
            ));
        }
        let time = match cols[0].parse::<i64>() {
            Ok(t) => t,
            Err(_) if idx == 0 => continue, // header row
            Err(e) => return Err(format!("CSV line {}: bad timestamp: {e}", idx + 1)),
        };
        let field = |i: usize, name: &str| {
            cols[i]
                .parse::<f64>()
                .map_err(|e| format!("CSV line {}: {name}: {e}", idx + 1))
        };
        candles.push(Candle {
            time,
            open: field(1, "open")?,
            high: field(2, "high")?,
            low: field(3, "low")?,
            close: field(4, "close")?,
            volume: field(5, "volume")?,
        });
    }
    Ok(candles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_csv_with_a_header() {
        let csv = "ts,open,high,low,close,volume\n1,10,11,9,10.5,100\n2,10.5,12,10,11,200\n";
        let candles = parse_csv(csv).unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].time, 1);
        assert!((candles[1].close - 11.0).abs() < 1e-9);
    }

    #[test]
    fn parse_csv_rejects_a_short_row() {
        assert!(parse_csv("1,2,3\n").is_err());
    }

    #[test]
    fn symbol_is_the_file_stem() {
        assert_eq!(symbol_of(Path::new("AAA.csv")).unwrap(), "AAA");
    }

    #[test]
    fn build_verifier_applies_overrides() {
        // A loose verifier is built without error; behavior is covered in
        // verify-core. Just confirm construction succeeds with overrides.
        assert!(build_verifier(Some(1e-3), Some(1e-2)).is_ok());
        assert!(build_verifier(None, None).is_ok());
    }
}
