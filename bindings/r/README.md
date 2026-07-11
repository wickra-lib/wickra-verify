# Wickra Verify — R

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, from R over the Wickra C ABI hub (`.Call`). A doctored
`claimed_report` cannot pass, because verification recomputes rather than
trusting the supplied numbers.

## Build / install

The package compiles a small C shim against the wickra-verify C ABI. Point it at
the C ABI header and shared library via environment variables:

```sh
cargo build -p wickra-verify-c            # build the native C ABI library
export WKVERIFY_INC=/path/to/bindings/c/include
export WKVERIFY_LIB=/path/to/target/release
R CMD INSTALL bindings/r
```

At run time the loader must find the shared library
(`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH`, or `PATH` on Windows).

## Usage

Everything goes through a verifier handle driven by JSON commands — the same
command protocol every Wickra binding shares.

```r
library(wickraverify)

verifier <- wkverify_new()

claim <- paste0(
  '{"strategy":', strategy_spec, ',',              # a wickra-backtest StrategySpec
  '"dataset_ref":{"kind":"inline","data":', data, '},',
  '"claimed_report":', report, '}'                 # the report being checked (untrusted)
)
verdict <- wkverify_command(verifier, paste0('{"cmd":"verify","claim":', claim, '}'))
cat(verdict)  # the full Verdict as JSON
```

## Commands

| `cmd`          | Payload            | Response                                |
|----------------|--------------------|-----------------------------------------|
| `verify`       | `{claim, data?}`   | the full `Verdict`                      |
| `explain`      | `{verdict}`        | `{ok:true,text:...}`                    |
| `canonicalize` | `{value}`          | `{ok:true,canonical:...}`              |
| `version`      | —                  | `{version:...,engine_version:...}`     |

For `files`-kind claims, supply the candle data under a top-level `data` key;
`inline` claims carry their data already.

Domain errors (a bad claim, an unknown command) come back in-band as
`{ok:false,error:...}`; only null/UTF-8/panic conditions raise an R error.

## License

MIT OR Apache-2.0.
