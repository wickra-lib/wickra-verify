## Plain-R tests for the wickra-verify R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickraverify)

strategy <- paste0(
  '{"symbol":"BTCUSDT","timeframe":"1h",',
  '"indicators":{"ema_fast":{"type":"Ema","params":[5]},',
  '"ema_slow":{"type":"Ema","params":[15]}},',
  '"entry":{"cross_above":["ema_fast","ema_slow"]},',
  '"exit":{"cross_below":["ema_fast","ema_slow"]},',
  '"sizing":{"type":"fixed_fraction","fraction":0.95},',
  '"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},',
  '"risk":{"trailing_stop_pct":5.0}}'
)

candles <- function() {
  parts <- vapply(0:39, function(i) {
    b <- 100.0 + sin(i * 0.4) * 8.0
    paste0(
      '{"time":', format(1700000000 + i * 3600, scientific = FALSE),
      ',"open":', b, ',"high":', b + 1.0, ',"low":', b - 1.0,
      ',"close":', b + 0.5, ',"volume":1000.0}'
    )
  }, character(1))
  paste0("[", paste(parts, collapse = ","), "]")
}

dataset_ref <- paste0('{"kind":"inline","data":{"BTCUSDT":', candles(), '}}')

verify_request <- function(claimed_report) {
  claim <- paste0(
    '{"strategy":', strategy, ',"dataset_ref":', dataset_ref,
    ',"claimed_report":', claimed_report, '}'
  )
  paste0('{"cmd":"verify","claim":', claim, '}')
}

hex_field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '":"[0-9a-f]{64}"'), json))
  stopifnot(length(m) == 1)
  m
}

## version
stopifnot(nzchar(wkverify_version()))

## a fudged claim is refuted via recomputation
verifier <- wkverify_new()
verdict <- wkverify_command(verifier, verify_request('{"fees_paid":99999.0}'))
stopifnot(grepl('"matches":false', verdict, fixed = TRUE))
stopifnot(grepl('"field":"fees_paid"', verdict, fixed = TRUE))
stopifnot(nchar(hex_field(verdict, "claimed_report_hash")) == 64 + nchar('"claimed_report_hash":""'))
stopifnot(nchar(hex_field(verdict, "inputs_hash")) == 64 + nchar('"inputs_hash":""'))

## an unknown command is an in-band error, not a hard error
inband <- wkverify_command(verifier, '{"cmd":"nope"}')
stopifnot(grepl('"ok":false', inband, fixed = TRUE))

## cross-language golden parity: for each committed golden/claims/*.json, verify
## over the shared golden/data and assert the response equals
## golden/expected/<claim>.json byte-for-byte. The binding returns the core's
## canonical command output verbatim, so byte equality is the exact
## cross-language parity check. The fixtures arrive in a later phase; until then
## the golden section is skipped.
golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(8)) {
    g <- file.path(d, "golden")
    if (dir.exists(file.path(g, "claims"))) {
      return(g)
    }
    d <- dirname(d)
  }
  NULL
}

load_golden_data <- function(g) {
  data_dir <- file.path(g, "data")
  if (!dir.exists(data_dir)) {
    return(NULL)
  }
  parts <- character(0)
  for (csv in sort(list.files(data_dir, pattern = "\\.csv$", full.names = TRUE))) {
    rows <- character(0)
    lines <- readLines(csv, warn = FALSE)
    for (idx in seq_along(lines)) {
      line <- trimws(lines[idx])
      if (!nzchar(line)) next
      cols <- trimws(strsplit(line, ",")[[1]])
      if (length(cols) < 6) next
      t <- suppressWarnings(as.integer(cols[1]))
      if (is.na(t)) next
      rows <- c(rows, paste0(
        '{"time":', cols[1], ',"open":', cols[2], ',"high":', cols[3],
        ',"low":', cols[4], ',"close":', cols[5], ',"volume":', cols[6], '}'
      ))
    }
    name <- sub("\\.csv$", "", basename(csv))
    parts <- c(parts, paste0('"', name, '":[', paste(rows, collapse = ","), "]"))
  }
  paste0("{", paste(parts, collapse = ","), "}")
}

g <- golden_dir()
if (!is.null(g)) {
  data_json <- load_golden_data(g)
  for (claim_path in list.files(file.path(g, "claims"), pattern = "\\.json$", full.names = TRUE)) {
    name <- basename(claim_path)
    claim_json <- trimws(paste(readLines(claim_path, warn = FALSE), collapse = "\n"))
    expected <- trimws(paste(
      readLines(file.path(g, "expected", name), warn = FALSE), collapse = "\n"
    ))
    envelope <- if (!is.null(data_json)) {
      paste0('{"cmd":"verify","claim":', claim_json, ',"data":', data_json, '}')
    } else {
      paste0('{"cmd":"verify","claim":', claim_json, '}')
    }
    gverifier <- wkverify_new()
    got <- wkverify_command(gverifier, envelope)
    stopifnot(identical(trimws(got), expected))
  }
}

cat("wickra-verify R tests passed\n")
