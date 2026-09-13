"""Operating-mode equivalence: a claim verifies the same whichever way its data arrives.

A ``Claim`` names its candles either by reference (``dataset_ref.kind ==
"files"``, the data supplied with the ``verify`` command) or inline
(``dataset_ref.kind == "inline"``, the data embedded in the claim). The verdict
must not depend on which: same ``matches``, same ``mismatches``, same report
hashes, same engine. Only ``inputs_hash`` may differ, because it binds the
dataset *reference* -- and it must differ, or the reference was not hashed.

The golden claims are ``files`` claims; each is re-issued inline here. Plain
functions, no test framework, for the same reason as ``test_golden``.
"""

import copy
import json

from wickra_verify import Verifier

from test_golden import claim_files, load_data

SAME = ("matches", "mismatches", "claimed_report_hash", "actual_report_hash", "engine_version")


def test_supplied_and_inline_data_verify_alike() -> None:
    data = load_data()
    for claim_path in claim_files():
        claim = json.loads(claim_path.read_text())
        assert claim["dataset_ref"]["kind"] == "files", "golden claims reference their data"
        symbols = claim["dataset_ref"]["symbols"]

        supplied = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": claim, "data": data})))

        inline_claim = copy.deepcopy(claim)
        inline_claim["dataset_ref"] = {"kind": "inline", "data": {s: data[s] for s in symbols}}
        inline = json.loads(Verifier().command(json.dumps({"cmd": "verify", "claim": inline_claim})))

        for field in SAME:
            assert inline[field] == supplied[field], f"{claim_path.name}: {field}"
        assert inline["inputs_hash"] != supplied["inputs_hash"], "inputs_hash binds the dataset reference"
