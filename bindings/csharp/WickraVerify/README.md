# Wickra Verify — C#

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, from .NET over the Wickra C ABI (P/Invoke). A doctored
`claimed_report` cannot pass, because verification recomputes rather than
trusting the supplied numbers.

## Install

```sh
dotnet add package Wickra.Verify
```

The correct native library ships in the NuGet package under
`runtimes/<rid>/native/` and is resolved automatically for your platform.

## Usage

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares.

```csharp
using System.Text.Json;
using Wickra.Verify;

using var verifier = new Verifier();

var claim = new
{
    strategy = strategySpec,   // a wickra-backtest StrategySpec
    dataset_ref = new { kind = "inline", data = candles },
    claimed_report = report,   // the report being checked (untrusted)
};
string json = JsonSerializer.Serialize(new { cmd = "verify", claim });
string verdict = verifier.Command(json); // the full Verdict as JSON
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
`{ok:false,error:...}`; only null/UTF-8/panic conditions throw
`InvalidOperationException`.

## License

MIT OR Apache-2.0.
