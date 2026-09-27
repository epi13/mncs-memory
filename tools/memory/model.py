"""Input validation and canonical digests.

Observation inputs and semantic requests are validated here; digests
follow the historical record shapes (sha256 over compact JSON) so
identities remain stable across the migration.
"""

from __future__ import annotations

import hashlib
import json
from typing import Any


class MemoryError(Exception):
    """Invalid input or an unusable memory state."""


KINDS = ("preference", "fact", "decision", "event", "noise")
INTENTS = ("WHAT_SHOULD_I_KNOW_BEFORE", "WHAT_IS_KNOWN_ABOUT",
           "WHAT_CHANGED", "WHAT_CONFLICTS_WITH", "WHAT_SUPPORTS",
           "RAW_EPISODES", "UNCERTAINTY")


def digest(value: Any) -> str:
    # Insertion order is part of the identity: local construction order
    # matches the historical record field order, keeping claim and
    # escalation identities stable across the migration.
    canonical = json.dumps(value, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def validate_observation(document: Any) -> dict[str, Any]:
    if not isinstance(document, dict):
        raise MemoryError("observation must be an object")
    for key in ("id", "kind", "subject_alias", "topic", "value", "text"):
        if key not in document:
            raise MemoryError(f"observation missing {key!r}")
    if document["kind"].lower() not in KINDS:
        raise MemoryError(f"unknown kind {document['kind']!r}")
    for key in ("id", "subject_alias", "topic", "value", "text"):
        if not isinstance(document[key], str):
            raise MemoryError(f"observation {key!r} must be a string")
    evidence = document.get("evidence", "DIRECT")
    if evidence not in ("DIRECT", "SUPPORTED", "WEAK", "MISSING"):
        raise MemoryError(f"unknown evidence {evidence!r}")
    observed_at = document.get("observed_at", 0)
    if not isinstance(observed_at, int):
        raise MemoryError("observed_at must be int")
    clean = {"id": document["id"], "kind": document["kind"],
             "subject_alias": document["subject_alias"],
             "topic": document["topic"], "value": document["value"],
             "text": document["text"], "evidence": evidence,
             "observed_at": observed_at,
             "valid_from": observed_at,
             "explicit_change": bool(document.get("explicit_change",
                                                  False)),
             "refinement": bool(document.get("refinement", False)),
             "source": document.get("source", "")}
    clean["identity"] = "observation:" + clean["id"]
    content = {key: clean[key] for key in clean if key != "identity"}
    clean["digest"] = digest(content)
    return clean


def validate_request(document: Any) -> dict[str, Any]:
    if not isinstance(document, dict):
        raise MemoryError("request must be an object")
    intent = document.get("intent")
    if intent not in INTENTS:
        raise MemoryError(f"unknown intent {intent!r}")
    def get(key: str, default: Any) -> Any:
        value = document.get(key)
        return default if value is None else value

    # Corpus-era aliases for the same flags.
    for modern, legacy in (("include_provenance", "provenance_requested"),
                           ("include_raw_episodes", "raw_episodes")):
        if document.get(modern) is None and legacy in document:
            document = {**document, modern: document[legacy]}
    clean = {"intent": intent,
             "subject": document.get("subject"),
             "topic": document.get("topic"),
             "goal": get("goal", ""),
             "as_of": int(get("as_of", 0)),
             "freshness_required": int(get("freshness_required", 0)),
             "max_units": int(get("max_units", 700)),
             "minimum_confidence": int(get("minimum_confidence", 85)),
             "required_topics": list(get("required_topics", [])),
             "forbidden_topics": list(get("forbidden_topics", [])),
             "max_supporting_claims": int(get("max_supporting_claims", 4)),
             "include_provenance": bool(get("include_provenance", False)),
             "include_conflicts": bool(get("include_conflicts", False)),
             "include_uncertainty": bool(get("include_uncertainty", False)),
             "include_raw_episodes": bool(
                 get("include_raw_episodes", False))}
    if clean["max_units"] < 0 or clean["max_supporting_claims"] < 0:
        raise MemoryError("budgets must be nonnegative")
    return clean
