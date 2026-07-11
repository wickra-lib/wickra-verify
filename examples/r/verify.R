# A runnable R example: submit a claim whose report has been doctored and assert
# the binding refutes it. Verification recomputes the report from (strategy,
# data) and compares, so a fabricated number cannot pass.
#
#   cargo build -p wickra-verify-c --release
#   export WKVERIFY_INC="$PWD/bindings/c/include"
#   export WKVERIFY_LIB="$PWD/target/release"
#   export LD_LIBRARY_PATH="$WKVERIFY_LIB:$LD_LIBRARY_PATH"   # PATH on Windows
#   R CMD INSTALL bindings/r
#   Rscript examples/r/verify.R

library(wickraverify)

strategy <- paste0(
  '{"symbol":"AAA","timeframe":"1h",',
  '"indicators":{"ema_fast":{"type":"Ema","params":[3]},',
  '"ema_slow":{"type":"Ema","params":[8]}},',
  '"entry":{"cross_above":["ema_fast","ema_slow"]},',
  '"exit":{"cross_below":["ema_fast","ema_slow"]},',
  '"sizing":{"type":"fixed_fraction","fraction":0.95},',
  '"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},',
  '"risk":{}}'
)

# A short V-shaped price path so the fast/slow EMA cross fires at least once.
closes <- c(120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128)

candle <- function(i) {
  close <- closes[i]
  open <- if (i == 1) close else closes[i - 1]
  paste0(
    '{"time":', 1700000000 + (i - 1) * 3600,
    ',"open":', open,
    ',"high":', max(open, close) + 1,
    ',"low":', min(open, close) - 1,
    ',"close":', close, ',"volume":1000}'
  )
}

data <- paste0(
  '{"AAA":[',
  paste(vapply(seq_along(closes), candle, character(1)), collapse = ","),
  "]}"
)

# An inline claim carrying a fabricated report (an inflated fees figure).
claim <- paste0(
  '{"strategy":', strategy,
  ',"dataset_ref":{"kind":"inline","data":', data, "}",
  ',"claimed_report":{"fees_paid":99999.0}}'
)

verifier <- wkverify_new()
verdict <- wkverify_command(verifier, paste0(
  '{"cmd":"verify","claim":', claim, "}"
))

cat("wickra-verify", wkverify_version(), "\n")
stopifnot(grepl('"matches":false', verdict, fixed = TRUE))
cat("doctored claim: REFUTED (tamper caught)\n")
