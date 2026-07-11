# Wickra Verify — WASM

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, compiled to WebAssembly for the browser (and any WASM
host). A doctored `claimed_report` cannot pass, because verification recomputes
rather than trusting the supplied numbers. Built with [wasm-bindgen].

The backtest engine runs sequentially under WASM (no thread pool in a browser
sandbox), which is byte-identical to the native run — the exact cross-language
golden check.

## Build

```sh
wasm-pack build --target web      # for browsers / bundlers
wasm-pack build --target nodejs   # for Node.js
```

The `pkg/` output (the `.wasm` binary plus the JS glue and TypeScript types) is
generated, not committed.

## Usage

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```js
import init, { Verifier } from "./pkg/wickra_verify_wasm.js";

await init(); // load the .wasm module (web target)

const verifier = new Verifier(); // default tolerances; new Verifier('{"atol":1e-9,"rtol":1e-6}') to override

const claim = {
  strategy: {/* a wickra-backtest StrategySpec */},
  dataset_ref: { kind: "inline", data: { BTCUSDT: [/* candles */] } },
  claimed_report: {/* the report being checked (untrusted) */},
};

const verdict = JSON.parse(verifier.command(JSON.stringify({ cmd: "verify", claim })));
console.log(verdict.matches ? "VERIFIED" : verdict.mismatches);
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
`{ok:false,error:...}`; a malformed command envelope throws a JS error.

## License

MIT OR Apache-2.0.

[wasm-bindgen]: https://github.com/rustwasm/wasm-bindgen
