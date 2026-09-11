"""Operating-mode equivalence: a claim verifies the same whichever way its data arrives.

A ``Claim`` names its candles either by reference (``dataset_ref.kind ==
"files"``, the data supplied with the ``verify`` command) or inline
(``dataset_ref.kind == "inline"``, the data embedded in the claim). The verdict
must not depend on which: same ``matches``, same ``mismatches``, same report
hashes, same engine. Only ``inputs_hash`` may differ, because it binds the
dataset *reference* -- and it must differ, or the reference was not hashed.

The golden claims are ``files`` claims; each is re-issued inline here.
"""

import copy
import json
from pathlib import Path

import pytest

from wickra_verify import Verifier

GOLDEN = Path(__file__).resolve().parents[3] / "golden"
CLAIMS = GOLDEN / "claims"
DATA = GOLDEN / "data"

SAME = ("matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version")


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
def test_supplied_and_inline_data_verify_alike(claim_path: Path) -> None:
    claim = json.loads(claim_path.read_text())
    assert claim["dataset_ref"]["kind"] == "files", "golden claims reference their data"
    data = _load_data()
    symbols = claim["dataset_ref"]["symbols"]

    supplied = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": claim, "data": data})))

    inline_claim = copy.deepcopy(claim)
    inline_claim["dataset_ref"] = {"kind": "inline", "data": {s: data[s] for s in symbols}}
    inline = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": inline_claim})))

    for field in SAME:
        assert inline[field] == supplied[field], field
    assert inline["inputs_hash"] != supplied["inputs_hash"], "inputs_hash binds the dataset reference"


def test_modes_fixtures_present_or_skipped() -> None:
    if not _claim_files():
        pytest.skip("golden fixtures not present yet")
