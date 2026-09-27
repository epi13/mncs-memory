"""JSONL memory store: atomic appends, strict reload, latest-wins claims.

Canonical state is five JSONL files (observations, claims, ingestions,
escalations, invocations). Every append is followed by fsync, bounding
interruption loss to the final record. Reload skips blank lines and
fails loudly on a corrupt line (with its file and line number) rather
than guessing. Claim state is latest-record-wins per identity, which is
exactly the versioned-append model: history is preserved in the file,
current state is the last word.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Iterator


FILES = ("observations", "claims", "ingestions", "escalations",
         "invocations")


class StoreError(Exception):
    """Canonical memory state cannot be read or written."""


class MemoryStore:
    def __init__(self, root: str | Path) -> None:
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self._buffers: dict[str, list[dict[str, Any]]] = {
            name: [] for name in FILES}

    def append(self, kind: str, record: dict[str, Any]) -> None:
        if kind not in self._buffers:
            raise StoreError(f"unknown store file {kind!r}")
        path = self.root / f"{kind}.jsonl"
        line = json.dumps(record, sort_keys=True, separators=(",", ":"))
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")
            handle.flush()
            os.fsync(handle.fileno())
        self._buffers[kind].append(record)

    def reload(self) -> None:
        """Re-read every file; latest claim state wins per identity."""
        buffers: dict[str, list[dict[str, Any]]] = {
            name: [] for name in FILES}
        for name in FILES:
            path = self.root / f"{name}.jsonl"
            if not path.is_file():
                continue
            with open(path, encoding="utf-8") as handle:
                for number, line in enumerate(handle, start=1):
                    if not line.strip():
                        continue
                    try:
                        buffers[name].append(json.loads(line))
                    except ValueError as error:
                        raise StoreError(
                            f"{path.name} line {number} is corrupt: "
                            f"{error}; canonical state is unchanged")
        claims: dict[str, dict[str, Any]] = {}
        for claim in buffers["claims"]:
            identity = claim.get("identity")
            if not isinstance(identity, str):
                raise StoreError("claim record without identity")
            claims[identity] = claim
        self._buffers = buffers
        self._buffers["claims"] = list(claims.values())

    def records(self, kind: str) -> list[dict[str, Any]]:
        return list(self._buffers[kind])

    def iter_records(self, kind: str) -> Iterator[dict[str, Any]]:
        return iter(self._buffers[kind])

    def replace_claim_state(self, identity: str,
                            record: dict[str, Any]) -> None:
        """Append a new state record; history stays in the file."""
        if record.get("identity") != identity:
            raise StoreError("claim identity mismatch on replace")
        self.append("claims", record)
        claims = [claim for claim in self._buffers["claims"]
                  if claim.get("identity") != identity]
        claims.append(record)
        self._buffers["claims"] = claims

    def counts(self) -> dict[str, int]:
        return {name: len(self._buffers[name]) for name in FILES}
