"""Compiler bridge: `mncs call` plus the typed-value wire.

Memory semantics flow through this module. The host encodes
arguments, invokes the native function, and decodes the returned
record. A subprocess that fails to return a decision is an evaluation
error, never a fabricated decision.
"""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DIR = REPO_ROOT / "native"


class NativeError(Exception):
    """The compiler failed to return a usable memory value."""


def find_mncs(explicit: str | None = None) -> str | None:
    if explicit:
        return explicit
    env = os.environ.get("MNCS_BIN") or os.environ.get("MNCS_BINARY")
    if env:
        return env
    root = os.environ.get("MNCS_LANGUAGE_ROOT")
    if root:
        candidate = Path(root) / "target" / "debug" / "mncs"
        if candidate.is_file():
            return str(candidate)
    return None


def default_libraries(extra: list[str] | None = None) -> list[str]:
    libraries: list[str] = []
    language_root = os.environ.get("MNCS_LANGUAGE_ROOT")
    if language_root:
        candidate = Path(language_root) / "library"
        if candidate.is_dir():
            libraries.append(str(candidate))
    test_native = os.environ.get("MNCS_TEST_NATIVE")
    if test_native:
        libraries.append(test_native)
    for item in extra or []:
        if item not in libraries:
            libraries.append(item)
    if str(NATIVE_DIR) not in libraries:
        libraries.append(str(NATIVE_DIR))
    return libraries


def cache_dir(explicit: str | None = None) -> str:
    if explicit:
        return explicit
    env = os.environ.get("MNCS_CACHE_DIR")
    if env:
        return env
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(
        os.path.expanduser("~"), ".cache")
    return os.path.join(base, "mncs-memory")


def call_function(*, mncs: str, program: str, module: str, function: str,
                  args: list[dict[str, Any]],
                  libraries: list[str] | None = None,
                  timeout_s: int = 120,
                  cache: str | None = None) -> dict[str, Any]:
    """Invoke a native function; return the decoded `call` document."""
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False,
                                     encoding="utf-8") as handle:
        json.dump(args, handle)
        args_path = handle.name
    command = [mncs, "call", program, "--module", module,
               "--function", function, "--args", args_path,
               "--cache-dir", cache or cache_dir()]
    for library in default_libraries(libraries):
        command += ["--library", library]
    try:
        completed = subprocess.run(command, capture_output=True, text=True,
                                   timeout=timeout_s)
    except (OSError, subprocess.SubprocessError) as error:
        raise NativeError(f"cannot start compiler: {error}")
    finally:
        try:
            os.unlink(args_path)
        except OSError:
            pass
    try:
        return json.loads(completed.stdout)
    except ValueError:
        raise NativeError(f"compiler returned non-JSON: "
                          f"{completed.stderr.strip()[:400]}")


def plain(value: dict[str, Any]) -> Any:
    """Decode one wire value into plain JSON scalars/collections."""
    if not isinstance(value, dict) or len(value) != 1:
        raise NativeError(f"unexpected wire value {value!r}")
    tag, body = next(iter(value.items()))
    if tag == "integer":
        return body["value"]
    if tag == "boolean":
        return body["value"]
    if tag == "finite":
        payload = body.get("payload")
        return {"variant": body["variant"], "type": body.get("type"),
                "payload": plain(payload) if payload is not None else None,
                "discriminant": body.get("discriminant")}
    if tag == "record":
        fields = body["fields"]
        if isinstance(fields, dict):
            return {name: plain(item) for name, item in fields.items()}
        return {name: plain(item) for name, item in fields}
    if tag == "sequence":
        return [plain(item) for item in body["values"]]
    raise NativeError(f"unknown wire tag {tag!r}")


def decode_record(returned: list[dict[str, Any]]) -> dict[str, Any]:
    if not returned:
        raise NativeError("compiler returned no values")
    decoded = plain(returned[0])
    if not isinstance(decoded, dict):
        raise NativeError("native function did not return a record")
    return decoded


def call_value(*, mncs: str, program: str, module: str, function: str,
                 args: list[dict[str, Any]],
                 libraries: list[str] | None = None,
                 timeout_s: int = 120) -> Any:
    """Call a native function returning a scalar; raise on failure."""
    document = call_function(mncs=mncs, program=program, module=module,
                             function=function, args=args,
                             libraries=libraries, timeout_s=timeout_s)
    if document.get("status") != "returned":
        raise NativeError(f"native call failed: "
                          f"{document.get('error', document)}")
    returned = document["call"]["returned"]
    if not returned:
        raise NativeError("compiler returned no values")
    return plain(returned[0])


def call_record(*, mncs: str, program: str, module: str, function: str,
                args: list[dict[str, Any]],
                libraries: list[str] | None = None,
                timeout_s: int = 120) -> dict[str, Any]:
    """Call a native function returning a record; raise on failure."""
    document = call_function(mncs=mncs, program=program, module=module,
                             function=function, args=args,
                             libraries=libraries, timeout_s=timeout_s)
    if document.get("status") != "returned":
        raise NativeError(f"native call failed: "
                          f"{document.get('error', document)}")
    return decode_record(document["call"]["returned"])


def integer(value: int) -> dict[str, Any]:
    return {"integer": {"value": int(value)}}


def boolean(value: bool) -> dict[str, Any]:
    return {"boolean": {"value": bool(value)}}


def memory_program(name: str) -> str:
    return str(NATIVE_DIR / "mncs" / "memory" / name)
