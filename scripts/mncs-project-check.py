#!/usr/bin/env python3
"""Run the bounded project test and emit an MNCS check-result document.

This is an adapter for mncs-actions, not an MNCS conformance verifier.  The
result is deliberately scoped to the repository's executable vertical-slice
test suite.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


TEST_COMMAND = ("cargo", "test", "--test", "vertical_slice")
RESULT_ID = "project-tests"
PROVIDER = "mncs-memory-project"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default=".mncs/project-check.json",
        help="path for the mncs.check-result/1 document",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    result_path = Path(args.output)

    try:
        completed = subprocess.run(TEST_COMMAND, check=False)
    except OSError as error:
        exit_code = 127
        summary = f"Could not execute {' '.join(TEST_COMMAND)}: {error}"
    else:
        exit_code = completed.returncode
        if exit_code == 0:
            summary = "cargo test --test vertical_slice passed."
        else:
            summary = f"cargo test --test vertical_slice failed with exit code {exit_code}."

    result = {
        "schema_version": "mncs.check-result/1",
        "id": RESULT_ID,
        "provider": PROVIDER,
        "verdict": "PASS" if exit_code == 0 else "FAIL",
        "scope": "mncs-memory bounded vertical slice",
        "claim": "The repository's bounded vertical-slice test suite completed successfully.",
        "summary": summary,
        "references": [
            {
                "kind": "test-suite",
                "path": "tests/vertical_slice.rs",
            }
        ],
    }

    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    # run-check needs the provider command itself to complete so it can
    # validate and aggregate the explicit FAIL result.  The test outcome is
    # carried by the result document; aggregation remains the boundary gate.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
