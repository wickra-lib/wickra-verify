# Wickra Verify examples — R

Runnable R examples for the [Wickra Verify R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-verify-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/verify.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `verify.R` | A runnable R example: submit a claim whose report has been doctored and assert the binding refutes it. |
