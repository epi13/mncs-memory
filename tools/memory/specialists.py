"""Deterministic specialists: typed contracts, bounded decisions.

Faithful behavior port of the historical deterministic providers.
Each specialist declares a role, takes structured input, and returns a
bounded outcome with evidence and rationale. Text handling
(normalization, token overlap, alias maps) lives here at the effect
boundary; every semantic combination of the results is decided
natively. Rationale strings match the historical records verbatim so
audit trails stay comparable.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


def normalize(value: str) -> str:
    """Lowercase ASCII-alphanumeric tokens joined by single spaces."""
    cleaned = "".join(
        char.lower() if char.isascii() and (char.isalnum() or char == "_")
        else " " for char in value)
    return " ".join(cleaned.split())


def tokens(value: str) -> set[str]:
    return {token for token in normalize(value).split() if len(token) > 1}


def benchmark_tokens(value: str) -> set[str]:
    """Token rule used by the retrieval baselines (verbatim port)."""
    return {token.lower() for token in "".join(
        char if char.isascii() and char.isalnum() else " "
        for char in value).split() if len(token) > 1}


@dataclass
class SpecialistOutcome:
    decision: str
    confidence: int
    evidence: str
    rationale: str
    input_units: int = 0
    output_units: int = 0


@dataclass
class EntityResolution:
    mode: str  # "unique" | "ambiguous" | "unknown" | "faulty"
    entity_id: str | None = None
    candidates: list[str] = field(default_factory=list)
    outcome: SpecialistOutcome | None = None


PROCESSOR_ROLE = "deterministic-specialists"
PROCESSOR_IDENTITY = "mncs-memory.deterministic.reference"
REFERENCE_VERSION = "reference-v1"
FAULTY_VERSION = "faulty-entity-v1"


def processor_ref(version: str) -> dict[str, Any]:
    return {"role": PROCESSOR_ROLE, "identity": PROCESSOR_IDENTITY,
            "version": version}


class EntityResolver:
    """Normalized alias map with explicit ambiguity and fault injection.

    Every alias (canonical name plus declared aliases) keys to its
    entities, sorted by identity. A lookup hitting several entities is
    ambiguous unless fault resolution is armed, mirroring the history.
    """

    def __init__(self, entities: dict[str, dict[str, Any]] | list[dict[str, Any]],
                 faulty_alias: str | None = None,
                 faulty_target: str | None = None,
                 faulty_resolve_first: bool = False,
                 version: str = REFERENCE_VERSION) -> None:
        self.version = version
        self.faulty_alias = normalize(faulty_alias) \
            if faulty_alias is not None else None
        self.faulty_target = faulty_target
        self.faulty_resolve_first = faulty_resolve_first \
            or faulty_target is not None
        if isinstance(entities, list):
            entities = {entity["identity"]: entity for entity in entities}
        table: dict[str, list[str]] = {}
        for entity_id, entity in entities.items():
            names = [entity.get("canonical_name", "")] + \
                list(entity.get("aliases", []))
            for name in names:
                table.setdefault(normalize(name), [])
                if entity_id not in table[normalize(name)]:
                    table[normalize(name)].append(entity_id)
        self.table = {key: sorted(ids) for key, ids in table.items()}

    def resolve(self, alias: str) -> EntityResolution:
        if self.faulty_alias is not None and \
                normalize(alias) == self.faulty_alias:
            return EntityResolution(
                mode="faulty", entity_id=self.faulty_target,
                outcome=SpecialistOutcome(
                    self.faulty_target or "", 90, "FAIL",
                    "intentionally faulty provider selected a wrong "
                    "canonical entity"))
        matches = self.table.get(normalize(alias), [])
        if not matches:
            return EntityResolution(
                mode="unknown",
                outcome=SpecialistOutcome(
                    "UNKNOWN", 0, "UNKNOWN",
                    "no registered canonical entity matches alias"))
        if len(matches) > 1:
            if self.faulty_resolve_first:
                return EntityResolution(
                    mode="faulty", entity_id=matches[0],
                    candidates=list(matches),
                    outcome=SpecialistOutcome(
                        matches[0], 90, "FAIL",
                        "intentionally faulty provider selected first "
                        "ambiguous entity"))
            return EntityResolution(
                mode="ambiguous", candidates=list(matches),
                outcome=SpecialistOutcome(
                    "UNKNOWN", 40, "UNKNOWN",
                    "alias resolves to multiple entities"))
        return EntityResolution(
            mode="unique", entity_id=matches[0],
            outcome=SpecialistOutcome(
                matches[0], 100, "PASS", "unique canonical alias match"))

    def processor(self) -> dict[str, Any]:
        return processor_ref(self.version)


def worthiness(kind: str, text: str) -> SpecialistOutcome:
    """Noise kind or empty text is not worth a claim (verbatim rule)."""
    if kind.strip().lower() == "noise" or not text.strip():
        return SpecialistOutcome("NO", 100, "PASS",
                                 "noise or empty observation")
    return SpecialistOutcome("YES", 100, "PASS",
                             "bounded reference worthiness rule")


def relationship(existing: list[dict[str, Any]],
                 observation: dict[str, Any]) -> SpecialistOutcome:
    """Same-topic gate, then value/flag rules (verbatim behavior).

    No same-topic claim → UNRELATED. A matching value → SAME. Explicit
    observation flags win next (refinement, explicit change). Otherwise
    a different value on the same subject/topic is a CONTRADICTION.
    """
    same_topic = [claim for claim in existing
                  if claim.get("topic") == observation["topic"]]
    if not same_topic:
        return SpecialistOutcome("UNRELATED", 100, "PASS",
                                 "no same-topic claim")
    if any(claim.get("value") == observation["value"]
           for claim in same_topic):
        return SpecialistOutcome("SAME", 100, "PASS",
                                 "value matches an existing claim")
    if observation.get("refinement", False):
        return SpecialistOutcome("REFINES", 92, "PASS",
                                 "observation explicitly refines prior "
                                 "scope")
    if observation.get("explicit_change", False):
        return SpecialistOutcome("SUPERSEDES", 95, "PASS",
                                 "observation explicitly records a change")
    return SpecialistOutcome("CONTRADICTS", 90, "PASS",
                             "same subject/topic has a different value")


def route(intent: str) -> SpecialistOutcome:
    """Intent echo: the route decision is the intent itself."""
    return SpecialistOutcome(intent, 100, "PASS", "typed intent route")


def relevance(goal: str, topic: str, value: str) -> SpecialistOutcome:
    """Goal tokens ∩ (topic ∪ value) tokens; subject routed separately.

    Subject identity is deliberately excluded: including it would make
    every claim about one subject appear relevant to every goal for
    that subject.
    """
    goal_tokens = tokens(goal)
    matched = sorted(token for text in (topic, value)
                     for token in tokens(text)
                     if token in goal_tokens)
    matched = sorted(set(matched))
    return SpecialistOutcome(str(len(matched)), 100, "PASS",
                             "deterministic lexical feature extraction")
