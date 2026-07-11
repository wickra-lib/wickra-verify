# Wickra Verify — Go

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, from Go over the C ABI hub (cgo). A doctored
`claimed_report` cannot pass, because verification recomputes rather than
trusting the supplied numbers.

## Install

```sh
go get github.com/wickra-lib/wickra-verify-go
```

The binding links the prebuilt C ABI library, staged per platform under
`lib/<goos>_<goarch>/`, with the header vendored under `include/`. Building from
source requires a C toolchain (cgo) and the staged native library.

## Usage

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```go
package main

import (
    "encoding/json"
    "fmt"

    wickra "github.com/wickra-lib/wickra-verify-go"
)

func main() {
    v := wickra.New()
    defer v.Close()

    claim := map[string]any{
        "strategy":       strategySpec,                     // a wickra-backtest StrategySpec
        "dataset_ref":    map[string]any{"kind": "inline", "data": data},
        "claimed_report": report,                           // the report being checked (untrusted)
    }
    cmd, _ := json.Marshal(map[string]any{"cmd": "verify", "claim": claim})
    out, err := v.Command(string(cmd))
    if err != nil {
        panic(err)
    }
    fmt.Println(out) // the full Verdict as JSON
}
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
`{ok:false,error:...}`; only null/UTF-8/panic conditions produce a Go `error`.

## License

MIT OR Apache-2.0.
