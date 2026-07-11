# Examples

A runnable example in every language. Each one submits a claim — an EMA-cross
strategy on a short V-shaped price path for symbol `AAA`, together with a report
that has been **doctored** (an inflated `fees_paid`) — and asserts the verifier
refutes it. Verification recomputes the report from the strategy and data and
compares field by field, so a fabricated number cannot pass; the verdict is
`matches: false` with the `fees_paid` mismatch.

The Rust example additionally confirms an *honest* claim first (it links the
engine directly), showing both sides of the guarantee.

| Language | Path | Run |
|----------|------|-----|
| Rust | [`rust/`](rust/) | `cargo run -p wickra-verify-example` |
| Python | [`python/verify.py`](python/verify.py) | `pip install wickra-verify && python examples/python/verify.py` |
| Node.js | [`node/`](node/) | `cd examples/node && npm install && node verify.js` |
| C / C++ | [`c/`](c/) | see below |
| Go | [`go/`](go/) | `cd examples/go && go run .` |
| .NET | [`csharp/Verify/`](csharp/Verify/) | `dotnet run --project examples/csharp/Verify` |
| Java | [`java/Verify.java`](java/Verify.java) | see the header comment |
| R | [`r/verify.R`](r/verify.R) | `Rscript examples/r/verify.R` |
| Web | [`web/`](web/) | static WebAssembly demo — see [`web/README.md`](web/README.md) |

The native bindings (Python, Node.js) load their own compiled library. The
bindings that go through the C ABI (Go, .NET, Java, R, and the C/C++ example
itself) need the C ABI library built first:

```bash
cargo build --release -p wickra-verify-c
```

## C / C++

The C and C++ examples build with CMake and run under ctest:

```bash
cargo build --release -p wickra-verify-c
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_verify.dll` next to each executable, since
there is no rpath.

## Data

The examples above carry their strategy, candles and claimed report inline so
each file runs on its own. The same fixture is also written out under
[`data/`](data/) for tooling and the CLI:

| File | What it is |
|------|------------|
| [`data/candles/AAA.csv`](data/candles/AAA.csv) | the 12-bar V-shaped price path (`ts,open,high,low,close,volume`) |
| [`data/claims/fudged.json`](data/claims/fudged.json) | a `Claim` (`files` dataset ref) whose `claimed_report` inflates `fees_paid` |

Verify it with the CLI, pointing `--data` at the candle directory:

```bash
cargo build --release -p wickra-verify
./target/release/wickra-verify --claim examples/data/claims/fudged.json --data examples/data/candles
```

## Expected output

Every example prints the version and the verdict for the doctored claim:

```text
wickra-verify 0.1.0
doctored claim: REFUTED (mismatch: fees_paid)
```

The Rust example prints both cases:

```text
wickra-verify 0.1.0
honest claim: VERIFIED
doctored claim: REFUTED (1 mismatch: fees_paid)
```

The CLI exits `2` on a refuted claim (a CI-friendly failure) and `0` when the
claim verifies; `--explain` renders the mismatch and forces exit `0`.
