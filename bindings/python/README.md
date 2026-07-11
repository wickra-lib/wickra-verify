# Wickra Verify — Python

Recompute a claimed backtest report with the deterministic Wickra engine and
confirm or refute it, field by field. A doctored `claimed_report` cannot pass,
because verification recomputes rather than trusting the supplied numbers.

## Install

```sh
pip install wickra-verify
```

## Usage

Everything goes through a `Verifier` driven by JSON commands — the same command
protocol every Wickra binding shares, so this Python front-end drives the exact
same core as the native CLI.

```python
import json
from wickra_verify import Verifier

verifier = Verifier()  # default tolerances; Verifier('{"atol":1e-9,"rtol":1e-6}') to override

claim = {
    "strategy": {...},                       # a wickra-backtest StrategySpec
    "dataset_ref": {"kind": "inline", "data": {"BTCUSDT": [...]}},
    "claimed_report": {...},                 # the report being checked (untrusted)
}

verdict = json.loads(verifier.command(json.dumps({"cmd": "verify", "claim": claim})))
if verdict["matches"]:
    print("VERIFIED")
else:
    for m in verdict["mismatches"]:
        print(f"{m['field']}: claimed {m['claimed']}, actual {m['actual']}")
```

## Commands

| `cmd`          | Payload                     | Response                                  |
|----------------|-----------------------------|-------------------------------------------|
| `verify`       | `{claim, data?}`            | the full `Verdict`                        |
| `explain`      | `{verdict}`                 | `{"ok":true,"text":...}`                  |
| `canonicalize` | `{value}`                   | `{"ok":true,"canonical":...}`             |
| `version`      | —                           | `{"version":...,"engine_version":...}`    |

For `files`-kind claims, supply the candle data under a top-level `data` key
(`{symbol: [candle, ...]}`); `inline` claims carry their data already.

Domain errors (a bad claim, an unknown command) come back in-band as
`{"ok":false,"error":...}`. A malformed command envelope raises `ValueError`.

## License

MIT OR Apache-2.0.
