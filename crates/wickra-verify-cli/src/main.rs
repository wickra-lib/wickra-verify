//! The `wickra-verify` reference CLI.
//!
//! Recomputes a claimed backtest report with the pinned engine and confirms or
//! refutes it. Exits `0` when verified, `2` when refuted (so a doctored claim
//! turns a CI build red), and `1` on error.

mod args;
mod run;

use args::Cli;
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run::run(&cli) {
        Ok(output) => {
            print!("{}", output.text);
            ExitCode::from(output.code)
        }
        Err(err) => {
            eprintln!("wickra-verify: {err}");
            ExitCode::from(1)
        }
    }
}
