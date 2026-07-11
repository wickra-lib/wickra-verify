# Cookbook

Practical recipes. All of these run locally — nothing is uploaded.

## Verify a claim from the CLI

```bash
# Files-kind claim: candles supplied via --data.
wickra-verify --claim claim.json --data candles/

# Inline claim (candles embedded): no --data needed.
wickra-verify --claim claim.json

# Render the mismatches instead of just an exit code.
wickra-verify --claim claim.json --data candles/ --explain
```

Exit **0** = verified, **2** = refuted, **1** = error.

## Gate a claim in CI: exit 2 = fraud

Because a refuted claim exits `2`, verification is a one-line CI gate. Commit the
strategy, its data, and the report you publish; the job fails if the report is
not the deterministic result of a fresh run:

```yaml
# .github/workflows/verify-claim.yml
- name: Verify the published backtest report
  run: wickra-verify --claim reports/latest.json --data data/candles
```

A doctored `claimed_report` — an inflated Sharpe, a fudged PnL, a silently-changed
parameter — can never be merged: the recomputation is the gate.

## Verify from any language

Every binding drives the same command envelope. Node.js:

```js
const { Verifier } = require("wickra-verify");
const verdict = JSON.parse(
  new Verifier().command(JSON.stringify({ cmd: "verify", claim })),
);
if (!verdict.matches) console.error("refuted:", verdict.mismatches);
```

Python:

```python
from wickra_verify import Verifier
import json
verdict = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": claim})))
assert verdict["matches"], verdict["mismatches"]
```

The C-ABI languages (C, C++, C#, Go, Java, R) call the same `command` on their
`Verifier` handle. See each `bindings/<lang>/README.md`.

## Loosen the tolerance

If two engines legitimately differ by more than the tight defaults (e.g. a
different BLAS), widen the tolerance:

```bash
wickra-verify --claim claim.json --data candles/ --atol 1e-6 --rtol 1e-4
```

## Check what a report canonicalizes to

```bash
echo '{"cmd":"canonicalize","value":{"b":2,"a":1}}' | wickra-verify-repl   # (illustrative)
```

or in Rust: `verify_core::canonicalize(&value)?`. Same bytes the hash is taken
over — see [CANONICALIZATION.md](CANONICALIZATION.md).

## Try it in the browser

`examples/web/` is a static WebAssembly page: paste a claim, press **Verify**, see
the verdict — all client-side, no server. Build with
`wasm-pack build --target web` and serve the folder. See
[`examples/web/README.md`](../examples/web/README.md).
