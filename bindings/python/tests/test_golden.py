"""Golden cross-language test: every golden claim verifies to its expected verdict.

The golden fixtures live in the repository-root ``golden/`` directory (shared by
every binding). Each ``golden/claims/<name>.json`` is a ``Claim``; its expected
``Verdict`` is ``golden/expected/<name>.json``. Candle data referenced by
``files`` claims is loaded from ``golden/data/<SYMBOL>.csv``.

Plain functions with plain asserts, no test framework: the Python 3.9 CI row
runs this module through ``run_without_pytest.py`` (pytest 9.x needs 3.10, and
8.x is below the fix for GHSA-6w46-j5rx-g56g), and 3.10 and up run it under
pytest, which collects plain ``test_*`` functions all the same.
"""

import json
from pathlib import Path

from wickra_verify import Verifier

GOLDEN = Path(__file__).resolve().parents[3] / "golden"
CLAIMS = GOLDEN / "claims"
EXPECTED = GOLDEN / "expected"
DATA = GOLDEN / "data"


def claim_files() -> list:
    return sorted(CLAIMS.glob("*.json"))


def load_data() -> dict:
    data = {}
    for csv in sorted(DATA.glob("*.csv")):
        candles = []
        for idx, line in enumerate(csv.read_text().splitlines()):
            line = line.strip()
            if not line:
                continue
            cols = [c.strip() for c in line.split(",")]
            try:
                time = int(cols[0])
            except ValueError:
                if idx == 0:
                    continue  # header row
                raise
            candles.append(
                {
                    "time": time,
                    "open": float(cols[1]),
                    "high": float(cols[2]),
                    "low": float(cols[3]),
                    "close": float(cols[4]),
                    "volume": float(cols[5]),
                }
            )
        data[csv.stem] = candles
    return data


def test_golden_fixtures_are_present() -> None:
    assert claim_files(), "golden/claims holds at least one claim"
    assert load_data(), "golden/data holds at least one series"


def test_golden_claims_match_expected() -> None:
    data = load_data()
    for claim_path in claim_files():
        claim = json.loads(claim_path.read_text())
        expected = json.loads((EXPECTED / claim_path.name).read_text())
        verdict = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": claim, "data": data})))
        assert verdict == expected, claim_path.name
