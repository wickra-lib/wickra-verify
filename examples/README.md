# Wickra Verify examples

A runnable example in every language. Each one submits a claim — an EMA-cross
strategy on a short V-shaped price path for symbol `AAA`, together with a report
that has been **doctored** (an inflated `fees_paid`) — and asserts the verifier
refutes it. Verification recomputes the report from the strategy and data and
compares field by field, so a fabricated number cannot pass; the verdict is
`matches: false` with the `fees_paid` mismatch.

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: confirm an honest backtest report, then show that a doctored one is refuted — verification recomputes the report from `(strategy, data)` and compares, so a fabricated number c |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-verify-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `verify.c` | A minimal C example: submit a claim to the wickra-verify C ABI whose report |
| `verify.cpp` | A minimal C++ example: submit a claim to the wickra-verify C ABI whose report has been doctored, and assert that verification refutes it. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Verify
```

| Example | What it does |
| --- | --- |
| `Verify/Program.cs` | A runnable .NET example: submit a claim whose report has been doctored and assert the binding refutes it. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `verify.go` | A runnable Go example: submit a claim whose report has been doctored and assert the binding refutes it. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/verify.R
```

| Example | What it does |
| --- | --- |
| `verify.R` | A runnable R example: submit a claim whose report has been doctored and assert the binding refutes it. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Verify.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Verify
```

| Example | What it does |
| --- | --- |
| `Verify.java` | A runnable Java example: submit a claim whose report has been doctored and assert the binding refutes it. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-verify
python examples/python/verify.py
```

| Example | What it does |
| --- | --- |
| `verify.py` | A runnable Python example: submit a claim whose report has been doctored and |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/verify.js
```

| Example | What it does |
| --- | --- |
| `verify.js` | A runnable Node.js example: submit a claim whose report has been doctored and assert the binding refutes it. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `app.js` | Browser demo wiring: load the wasm verifier, prefill an example claim, and verify it on demand — all client-side. |
| `index.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): . The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).

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
