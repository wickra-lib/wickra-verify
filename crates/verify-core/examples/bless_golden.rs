//! Regenerate the repository-root `golden/` fixtures from the real engine.
//!
//! The golden set pins verify-core's determinism across the Rust integration
//! tests and all ten language bindings. It is produced here, never edited by
//! hand: this example runs the actual wickra-backtest engine over a fixed
//! deterministic universe, freezes the honest report, derives the doctored
//! claims from it by changing one thing at a time, and blesses each expected
//! `Verdict` through the same `Verifier` command path the bindings drive.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo run -p verify-core --example bless_golden
//! ```
//!
//! Everything downstream (`golden/claims`, `golden/expected`) is a pure function
//! of `golden/data` plus this program, so the fixtures are reproducible.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use verify_core::Verifier;
use wickra_backtest_core::{run, Candle, StrategySpec};

/// Number of bars in the golden universe. 240 is enough for the moving-average
/// cross to trade repeatedly and for the metrics (Sharpe, drawdown) to settle.
const BARS: usize = 240;
/// The single golden symbol; the strategy trades it.
const SYMBOL: &str = "BTCUSDT";
/// First bar time; each bar is one hour.
const T0: i64 = 1_700_000_000;
const HOUR: i64 = 3600;

/// The golden close path: two blended sine waves so the fast/slow EMA cross
/// several times and the equity curve draws down and recovers. Every value is a
/// finite `f64` that round-trips through its shortest decimal form, so the CSV
/// the bindings re-read parses back to exactly these candles.
fn close_at(i: usize, scale: f64, phase: f64) -> f64 {
    let x = i as f64;
    scale * (100.0 + 12.0 * (x * 0.10 + phase).sin() + 6.0 * (x * 0.031).sin())
}

/// Build a candle series from a close path. `open` is the previous close;
/// `high`/`low` bracket the bar by one unit; volume is constant.
fn series(scale: f64, phase: f64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(BARS);
    let mut prev = close_at(0, scale, phase);
    for i in 0..BARS {
        let close = close_at(i, scale, phase);
        let open = if i == 0 { close } else { prev };
        let high = open.max(close) + 1.0;
        let low = open.min(close) - 1.0;
        candles.push(Candle {
            time: T0 + i64::try_from(i).expect("bar index fits i64") * HOUR,
            open,
            high,
            low,
            close,
            volume: 1000.0,
        });
        prev = close;
    }
    candles
}

/// The strategy under test: a fast/slow EMA cross with costs and a trailing
/// stop. Kept as raw JSON so the golden claims embed it verbatim.
fn strategy() -> Value {
    json!({
        "symbol": SYMBOL,
        "timeframe": "1h",
        "indicators": {
            "ema_fast": { "type": "Ema", "params": [5] },
            "ema_slow": { "type": "Ema", "params": [15] }
        },
        "entry": { "cross_above": ["ema_fast", "ema_slow"] },
        "exit": { "cross_below": ["ema_fast", "ema_slow"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": { "trailing_stop_pct": 5.0 }
    })
}

/// Run the engine and return the report as JSON.
fn report_of(candles: &[Candle]) -> Value {
    let spec: StrategySpec =
        serde_json::from_value(strategy()).expect("strategy is a valid StrategySpec");
    let report = run(&spec, candles).expect("engine runs over the golden data");
    serde_json::to_value(&report).expect("report serializes")
}

/// Write a CSV for one symbol, shortest-decimal so it round-trips.
fn write_csv(dir: &Path, candles: &[Candle]) {
    let mut out = String::from("ts,open,high,low,close,volume\n");
    for c in candles {
        writeln!(
            out,
            "{},{},{},{},{},{}",
            c.time, c.open, c.high, c.low, c.close, c.volume
        )
        .expect("writing to a String is infallible");
    }
    fs::write(dir.join(format!("{SYMBOL}.csv")), out).expect("write csv");
}

/// A claim envelope with a `files` dataset reference; the data is supplied out
/// of band (from `golden/data`), exactly as the bindings drive it.
fn claim(report: &Value) -> Value {
    json!({
        "strategy": strategy(),
        "dataset_ref": { "kind": "files", "symbols": [SYMBOL] },
        "claimed_report": report,
    })
}

/// Clone `report` and overwrite one metric leaf, returning the doctored report.
fn fudge_metric(report: &Value, field: &str, new_value: Value) -> Value {
    let mut doctored = report.clone();
    doctored["metrics"][field] = new_value;
    doctored
}

fn write_json(path: &Path, value: &Value) {
    let mut text = serde_json::to_string_pretty(value).expect("serialize fixture");
    text.push('\n');
    fs::write(path, text).expect("write fixture");
}

fn main() {
    let root = repo_root();
    let data_dir = root.join("golden/data");
    let claims_dir = root.join("golden/claims");
    let expected_dir = root.join("golden/expected");
    for dir in [&data_dir, &claims_dir, &expected_dir] {
        fs::create_dir_all(dir).expect("create golden dir");
    }

    // The honest universe and its true report.
    let candles = series(1.0, 0.0);
    write_csv(&data_dir, &candles);
    let honest = report_of(&candles);

    // A second, unrelated price path; its summary metrics stand in for a report
    // that never came from the golden data.
    let other = report_of(&series(1.07, 0.9));
    let other_metrics = other["metrics"].clone();
    let other_fees = other["fees_paid"].clone();

    // The honest metrics, read back for the single-field fudges.
    let sharpe = honest["metrics"]["sharpe"]
        .as_f64()
        .expect("sharpe is a number");
    let ret = honest["metrics"]["return_pct"]
        .as_f64()
        .expect("return_pct is a number");
    let dd = honest["metrics"]["max_drawdown"]
        .as_f64()
        .expect("max_drawdown is a number");

    // `wrong_data`: the honest trades and equity, but the summary metrics and
    // fees of a different price series — a report that cannot belong to this data.
    let mut wrong_data = honest.clone();
    wrong_data["metrics"] = other_metrics;
    wrong_data["fees_paid"] = other_fees;

    let claims: Vec<(&str, Value)> = vec![
        ("honest", honest.clone()),
        (
            "fudged_sharpe",
            fudge_metric(&honest, "sharpe", json!(sharpe + 1.0)),
        ),
        (
            "fudged_return",
            fudge_metric(&honest, "return_pct", json!(ret + 25.0)),
        ),
        (
            "fudged_drawdown",
            fudge_metric(&honest, "max_drawdown", json!(dd - 0.1)),
        ),
        ("wrong_data", wrong_data),
    ];

    // A single verifier with the default tolerances drives every bless, exactly
    // as the bindings do.
    let verifier = Verifier::default();
    let data: BTreeMap<String, Vec<Candle>> = {
        let mut map = BTreeMap::new();
        map.insert(SYMBOL.to_string(), candles.clone());
        map
    };
    let data_value = serde_json::to_value(&data).expect("data serializes");

    for (name, report) in &claims {
        let claim_value = claim(report);
        write_json(&claims_dir.join(format!("{name}.json")), &claim_value);

        let envelope = json!({ "cmd": "verify", "claim": claim_value, "data": data_value });
        let verdict = verifier
            .command_json(&serde_json::to_string(&envelope).expect("envelope serializes"))
            .expect("verify command succeeds");
        // The expected file is the exact canonical verdict, one trailing newline.
        let parsed: Value = serde_json::from_str(&verdict).expect("verdict is JSON");
        println!(
            "{name}: matches={} mismatches={}",
            parsed["matches"].as_bool().unwrap_or(false),
            parsed["mismatches"].as_array().map_or(0, Vec::len)
        );
        let mut text = verdict;
        text.push('\n');
        fs::write(expected_dir.join(format!("{name}.json")), text).expect("write expected");
    }
    println!("blessed {} golden claims", claims.len());
}

/// Walk up from the example's manifest dir to the repository root (the directory
/// that holds `.git`). Falls back to the current directory.
fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if dir.join(".git").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}
