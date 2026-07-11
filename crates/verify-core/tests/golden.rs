//! Golden parity: load the committed `golden/{claims,data}`, verify through the
//! same canonical command surface every binding uses, and assert the response is
//! byte-for-byte identical to `golden/expected/*.json`. This is the Rust anchor
//! of the cross-language determinism guarantee; the ten bindings assert the same
//! bytes.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use verify_core::{Candle, Verifier};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

/// Parse an OHLCV CSV (`ts,open,high,low,close,volume`, header row skipped).
fn parse_csv(content: &str) -> Vec<Candle> {
    content
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            let time = cols.first()?.parse::<i64>().ok()?; // the header row fails here and is skipped
            Some(Candle {
                time,
                open: cols[1].parse().unwrap(),
                high: cols[2].parse().unwrap(),
                low: cols[3].parse().unwrap(),
                close: cols[4].parse().unwrap(),
                volume: cols[5].parse().unwrap(),
            })
        })
        .collect()
}

/// Load every `golden/data/<SYMBOL>.csv` into a symbol-keyed map.
fn load_data(dir: &Path) -> BTreeMap<String, Vec<Candle>> {
    let mut data = BTreeMap::new();
    for entry in fs::read_dir(dir.join("data")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            let symbol = path.file_stem().unwrap().to_string_lossy().into_owned();
            data.insert(symbol, parse_csv(&fs::read_to_string(&path).unwrap()));
        }
    }
    data
}

#[test]
fn golden_verdicts_are_byte_identical() {
    let dir = golden_dir();
    let data = load_data(&dir);
    let data_value: Value = serde_json::to_value(&data).unwrap();

    let verifier = Verifier::default();
    let mut count = 0;
    let mut honest_seen = false;
    for entry in fs::read_dir(dir.join("claims")).unwrap() {
        let claim_path = entry.unwrap().path();
        if claim_path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = claim_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let claim_value: Value =
            serde_json::from_str(&fs::read_to_string(&claim_path).unwrap()).unwrap();

        // Verify through the exact canonical command surface the bindings use.
        let cmd = json!({ "cmd": "verify", "claim": claim_value, "data": data_value }).to_string();
        let got = verifier.command_json(&cmd).unwrap();

        let expected = fs::read_to_string(dir.join("expected").join(&name)).unwrap();
        assert_eq!(
            got.trim(),
            expected.trim(),
            "golden verdict mismatch for {name}"
        );

        // The honest claim verifies; every doctored claim is refuted.
        let verdict: Value = serde_json::from_str(&got).unwrap();
        let matches = verdict["matches"].as_bool().unwrap();
        if name == "honest.json" {
            assert!(matches, "honest golden claim must verify");
            honest_seen = true;
        } else {
            assert!(!matches, "doctored golden claim {name} must be refuted");
        }
        count += 1;
    }
    assert_eq!(count, 5, "expected exactly five golden claims");
    assert!(honest_seen, "the honest golden claim must be present");
}
