"""A runnable Python example: submit a claim whose report has been doctored and
assert the binding refutes it. Verification recomputes the report from
(strategy, data) and compares, so a fabricated number cannot pass.

    pip install wickra-verify
    python examples/python/verify.py
"""

import json

from wickra_verify import Verifier

STRATEGY = {
    "symbol": "AAA",
    "timeframe": "1h",
    "indicators": {
        "ema_fast": {"type": "Ema", "params": [3]},
        "ema_slow": {"type": "Ema", "params": [8]},
    },
    "entry": {"cross_above": ["ema_fast", "ema_slow"]},
    "exit": {"cross_below": ["ema_fast", "ema_slow"]},
    "sizing": {"type": "fixed_fraction", "fraction": 0.95},
    "costs": {"taker_bps": 5, "slippage": {"type": "fixed_bps", "bps": 2}},
    "risk": {},
}

# A short V-shaped price path so the fast/slow EMA cross fires at least once.
CLOSES = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128]


def _candles() -> list[dict]:
    out = []
    for i, close in enumerate(CLOSES):
        open_ = close if i == 0 else CLOSES[i - 1]
        out.append(
            {
                "time": 1_700_000_000 + i * 3600,
                "open": open_,
                "high": max(open_, close) + 1,
                "low": min(open_, close) - 1,
                "close": close,
                "volume": 1000,
            }
        )
    return out


def main() -> None:
    verifier = Verifier()
    claim = {
        "strategy": STRATEGY,
        "dataset_ref": {"kind": "inline", "data": {"AAA": _candles()}},
        # A fabricated report: a claimant asserts an inflated fees figure.
        "claimed_report": {"fees_paid": 99999.0},
    }

    verdict = json.loads(
        verifier.command(json.dumps({"cmd": "verify", "claim": claim}))
    )
    print(f"wickra-verify {Verifier.version()}")
    assert verdict["matches"] is False, "a doctored report must be refuted"
    field = verdict["mismatches"][0]["field"]
    print(f"doctored claim: REFUTED (mismatch: {field})")


if __name__ == "__main__":
    main()
