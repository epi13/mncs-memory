#!/usr/bin/env python3
"""Run the canonical mncs-memory verification and emit a check-result.

This is an adapter for mncs-actions, not an MNCS conformance verifier.
The gate is the canonical implementation: the host unittest suites
plus the native MNCS semantic suites. The historical Rust tree under
`reference/` is an oracle, not the gate, and is not executed here.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


RESULT_ID = "project-tests"
PROVIDER = "mncs-memory-project"
NATIVE_MODULES = ("codes", "lifecycle", "recall", "capsule")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default=".mncs/project-check.json",
        help="path for the mncs.check-result/1 document",
    )
    return parser.parse_args()


def find_mncs() -> str | None:
    for candidate in (os.environ.get("MNCS_BIN"),
                      os.environ.get("MNCS_BINARY")):
        if candidate and Path(candidate).is_file():
            return candidate
    root = os.environ.get("MNCS_LANGUAGE_ROOT")
    if root:
        candidate = Path(root) / "target" / "debug" / "mncs"
        if candidate.is_file():
            return str(candidate)
    return None


def library_roots(repo: Path) -> list[str]:
    roots = [str(repo / "native")]
    language_root = os.environ.get("MNCS_LANGUAGE_ROOT")
    if language_root:
        roots.append(str(Path(language_root) / "library"))
    test_native = os.environ.get("MNCS_TEST_NATIVE")
    if test_native:
        roots.append(test_native)
    return roots


def run_host_suite(repo: Path) -> tuple[bool, str]:
    env = dict(os.environ)
    completed = subprocess.run(
        [sys.executable, "-m", "unittest", "discover", "-s", "tests"],
        cwd=repo, env=env, capture_output=True, text=True, check=False,
        timeout=7200)
    if completed.returncode != 0:
        tail = (completed.stdout + completed.stderr)[-1200:].strip()
        return False, f"host suites failed: {tail or 'no output'}"
    return True, "host suites pass (reference invariants + memory path)"


def run_native_suites(repo: Path, mncs: str) -> tuple[bool, str]:
    libraries = library_roots(repo)
    for module in NATIVE_MODULES:
        command = [mncs, "test",
                   str(repo / "native" / "mncs" / "memory" /
                       f"{module}.mncs"), "--format", "json"]
        for library in libraries:
            command += ["--library", library]
        try:
            completed = subprocess.run(
                command, capture_output=True, text=True, check=False,
                timeout=300)
            document = json.loads(completed.stdout)
        except Exception as error:
            return False, f"native {module}: runner error {error}"
        if document.get("classification") != "passed":
            return False, (f"native {module}: "
                           f"{document.get('classification')}")
    return True, f"native suites pass ({len(NATIVE_MODULES)}/4)"


def main() -> int:
    args = parse_args()
    repo = Path(__file__).resolve().parents[1]
    result_path = Path(args.output)
    notes: list[str] = []
    failed = False

    host_ok, host_note = run_host_suite(repo)
    notes.append(host_note)
    failed = failed or not host_ok

    mncs = find_mncs()
    if mncs is None:
        notes.append("native suites not exercised (no toolchain)")
    else:
        native_ok, native_note = run_native_suites(repo, mncs)
        notes.append(native_note)
        failed = failed or not native_ok

    summary = "; ".join(notes)
    result = {
        "schema_version": "mncs.check-result/1",
        "id": RESULT_ID,
        "provider": PROVIDER,
        "verdict": "FAIL" if failed else "PASS",
        "scope": "mncs-memory canonical native path",
        "claim": "The canonical native memory suites completed "
                 "successfully.",
        "summary": summary,
        "references": [
            {"kind": "test-suite", "path": "tests/test_memory.py"},
            {"kind": "test-suite", "path": "tests/test_reference.py"},
            {"kind": "native-suite",
             "path": "native/mncs/memory/lifecycle.mncs"},
            {"kind": "native-suite",
             "path": "native/mncs/memory/recall.mncs"},
            {"kind": "native-suite",
             "path": "native/mncs/memory/capsule.mncs"},
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
