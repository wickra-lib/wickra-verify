"""Golden cross-language test: every golden claim verifies to its expected verdict.

The golden fixtures live in the repository-root ``golden/`` directory (shared by
every binding). Each ``golden/claims/<name>.json`` is a ``Claim``; its expected
``Verdict`` is ``golden/expected/<name>.json``. Candle data referenced by
``files`` claims is loaded from ``golden/data/<SYMBOL>.csv``.

The test skips cleanly when the fixtures are not present, so it is green before
the golden phase lands and becomes active once it does.
"""

import json
from pathlib import Path

import pytest

from wickra_verify import Verifier

GOLDEN = Path(__file__).resolve().parents[3] / "golden"
CLAIMS = GOLDEN / "claims"
EXPECTED = GOLDEN / "expected"
DATA = GOLDEN / "data"


def _claim_files() -> list[Path]:
    return sorted(CLAIMS.glob("*.json")) if CLAIMS.is_dir() else []


def _load_data() -> dict:
    data: dict[str, list[dict]] = {}
    if not DATA.is_dir():
        return data
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


@pytest.mark.parametrize("claim_path", _claim_files(), ids=lambda p: p.stem)
def test_golden_claim_matches_expected(claim_path: Path) -> None:
    claim = json.loads(claim_path.read_text())
    expected = json.loads((EXPECTED / claim_path.name).read_text())

    envelope: dict = {"cmd": "verify", "claim": claim}
    data = _load_data()
    if data:
        envelope["data"] = data

    verdict = json.loads(Verifier().command(json.dumps(envelope)))
    assert verdict == expected


def test_golden_fixtures_present_or_skipped() -> None:
    if not _claim_files():
        pytest.skip("golden fixtures not present yet")
