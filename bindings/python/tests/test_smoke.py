"""Smoke test: construct a verifier, verify a claim, parse the verdict."""

import json
import math

from wickra_verify import Verifier, __version__

STRATEGY = {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "indicators": {
        "ema_fast": {"type": "Ema", "params": [5]},
        "ema_slow": {"type": "Ema", "params": [15]},
    },
    "entry": {"cross_above": ["ema_fast", "ema_slow"]},
    "exit": {"cross_below": ["ema_fast", "ema_slow"]},
    "sizing": {"type": "fixed_fraction", "fraction": 0.95},
    "costs": {"taker_bps": 5, "slippage": {"type": "fixed_bps", "bps": 2}},
    "risk": {"trailing_stop_pct": 5.0},
}


def _candles() -> list[dict]:
    out = []
    for i in range(40):
        base = 100.0 + math.sin(i * 0.4) * 8.0
        out.append(
            {
                "time": 1_700_000_000 + i * 3600,
                "open": base,
                "high": base + 1.0,
                "low": base - 1.0,
                "close": base + 0.5,
                "volume": 1000.0,
            }
        )
    return out


def _verify_request(claimed_report: dict) -> str:
    return json.dumps(
        {
            "cmd": "verify",
            "claim": {
                "strategy": STRATEGY,
                "dataset_ref": {"kind": "inline", "data": {"BTCUSDT": _candles()}},
                "claimed_report": claimed_report,
            },
        }
    )


def test_fudged_claim_is_refuted() -> None:
    verifier = Verifier()
    # A deliberately wrong fees_paid must be caught by recomputation.
    verdict = json.loads(verifier.command(_verify_request({"fees_paid": 99999.0})))
    assert verdict["matches"] is False
    assert any(m["field"] == "fees_paid" for m in verdict["mismatches"])
    assert len(verdict["engine_version"]) > 0
    assert len(verdict["claimed_report_hash"]) == 64
    assert len(verdict["inputs_hash"]) == 64


def test_version_matches_module() -> None:
    assert Verifier.version() == __version__
    reported = json.loads(Verifier().command('{"cmd":"version"}'))
    assert reported["version"] == __version__


def test_unknown_command_is_in_band_error() -> None:
    verifier = Verifier()
    response = json.loads(verifier.command('{"cmd":"nope"}'))
    assert response["ok"] is False
    assert "nope" in response["error"]
