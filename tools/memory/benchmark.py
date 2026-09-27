"""Corpus benchmark: raw context vs simple retrieval vs MNCS memory.

Faithful port of the historical benchmark harness. Three paths answer
the same frozen scenarios; the artifact records correctness against
the frozen oracle, required-fact recall, irrelevant inclusion, units
(characters), candidates, escalations, and UNKNOWN counts. The corpus
is bounded synthetic evidence, not a general performance claim.
"""

from __future__ import annotations

from typing import Any

from . import engine as engine_module
from . import model, specialists


def recall(answer: str, expected: list[str]) -> float:
    if not expected:
        return 1.0
    return sum(1 for value in expected if value in answer) / len(expected)


def irrelevant_inclusion(topics: list[str], expected: list[str]) -> int:
    allowed = set(expected)
    return sum(1 for topic in topics if topic not in allowed)


def raw_context(keeper: Any, scenario: dict[str, Any]) -> dict[str, Any]:
    request = scenario["request"]
    observations = [
        obs for obs in keeper.records("observations")
        if (request.get("subject") is None
            or obs["subject_alias"] == request["subject"]
            or request["subject"] in obs["text"])
        and (request.get("topic") is None
             or obs["topic"] == request["topic"])]
    answer = "\n".join(obs["text"] for obs in observations)
    correct = all(value in answer
                  for value in scenario["expected_values"]) and (
        scenario["expected_status"] == "UNKNOWN"
        or not scenario["expected_values"])
    return {"answer": answer, "status": "UNKNOWN",
            "correct_against_frozen_oracle": correct,
            "evaluated_as_answer": False,
            "required_fact_recall": recall(
                answer, scenario["expected_values"]),
            "irrelevant_fact_inclusion": irrelevant_inclusion(
                [obs["topic"] for obs in observations],
                scenario["expected_topics"]),
            "units": len(answer),
            "candidates_exposed": len(observations)}


def simple_retrieval(keeper: Any, scenario: dict[str, Any]
                     ) -> dict[str, Any]:
    request = scenario["request"]
    query = specialists.benchmark_tokens(
        f"{request.get('goal', '')} {request.get('topic') or ''} "
        f"{request.get('subject') or ''}")
    ranked = []
    for obs in keeper.records("observations"):
        if obs["kind"] == "noise":
            continue
        haystack = specialists.benchmark_tokens(
            f"{obs['text']} {obs['topic']} {obs['value']} "
            f"{obs['subject_alias']}")
        score = len(query & haystack)
        if score > 0:
            ranked.append((score, obs["identity"], obs))
    ranked.sort(key=lambda entry: (-entry[0], entry[1]))
    ranked = ranked[:3]
    answer = ";".join(f"{obs['topic']}={obs['value']}"
                      for _, _, obs in ranked)
    status = "UNKNOWN" if not ranked else "PASS"
    correct = status == scenario["expected_status"] and all(
        value in answer for value in scenario["expected_values"])
    return {"answer": answer, "status": status,
            "correct_against_frozen_oracle": correct,
            "evaluated_as_answer": True,
            "required_fact_recall": recall(
                answer, scenario["expected_values"]),
            "irrelevant_fact_inclusion": irrelevant_inclusion(
                [obs["topic"] for _, _, obs in ranked],
                scenario["expected_topics"]),
            "units": len(answer),
            "candidates_exposed": len(ranked)}


def capsule_correct(capsule: dict[str, Any],
                    scenario: dict[str, Any]) -> bool:
    return (capsule["status"] == scenario["expected_status"]
            and all(any(item["topic"] == topic
                        for item in capsule["items"])
                    for topic in scenario["expected_topics"])
            and all(value in capsule["answer"]
                    for value in scenario["expected_values"]))


def run_corpus(mncs: str, corpus: dict[str, Any]) -> dict[str, Any]:
    import tempfile
    from .store import MemoryStore
    keeper = MemoryStore(tempfile.mkdtemp(prefix="memory-bench-"))
    memory = engine_module.MemoryEngine(mncs, str(keeper.root),
                                        corpus.get("entities", {}))
    for observation in corpus["observations"]:
        memory.ingest(observation)
    cases = []
    aggregate = {"memory_correct": 0, "simple_retrieval_correct": 0,
                 "raw_answerable_by_context_scan": 0,
                 "memory_required_fact_recall": 0.0,
                 "simple_required_fact_recall": 0.0,
                 "raw_required_fact_recall": 0.0,
                 "memory_irrelevant_fact_inclusion": 0,
                 "simple_irrelevant_fact_inclusion": 0,
                 "memory_units_total": 0, "simple_units_total": 0,
                 "raw_units_total": 0, "memory_candidates_total": 0,
                 "simple_candidates_total": 0, "raw_candidates_total": 0,
                 "memory_escalation_cases": 0, "memory_unknown_cases": 0,
                 "specialist_invocations": 0}
    for scenario in corpus["queries"]:
        raw = raw_context(memory.store, scenario)
        simple = simple_retrieval(memory.store, scenario)
        capsule = memory.query(scenario["request"])
        memory_result = {
            "required_fact_recall": recall(
                capsule["answer"], scenario["expected_values"]),
            "irrelevant_fact_inclusion": irrelevant_inclusion(
                [item["topic"] for item in capsule["items"]],
                scenario["expected_topics"]),
            "correct_against_frozen_oracle": capsule_correct(
                capsule, scenario),
            "capsule": capsule}
        aggregate["memory_correct"] += int(
            memory_result["correct_against_frozen_oracle"])
        aggregate["simple_retrieval_correct"] += int(
            simple["correct_against_frozen_oracle"])
        aggregate["raw_answerable_by_context_scan"] += int(
            raw["correct_against_frozen_oracle"])
        aggregate["memory_required_fact_recall"] += \
            memory_result["required_fact_recall"]
        aggregate["simple_required_fact_recall"] += \
            simple["required_fact_recall"]
        aggregate["raw_required_fact_recall"] += raw["required_fact_recall"]
        aggregate["memory_irrelevant_fact_inclusion"] += \
            memory_result["irrelevant_fact_inclusion"]
        aggregate["simple_irrelevant_fact_inclusion"] += \
            simple["irrelevant_fact_inclusion"]
        aggregate["memory_units_total"] += capsule["units"]
        aggregate["simple_units_total"] += simple["units"]
        aggregate["raw_units_total"] += raw["units"]
        aggregate["memory_candidates_total"] += capsule[
            "candidates_exposed"]
        aggregate["simple_candidates_total"] += simple["candidates_exposed"]
        aggregate["raw_candidates_total"] += raw["candidates_exposed"]
        aggregate["memory_escalation_cases"] += int(
            capsule["escalation"]["kind"] != "none")
        aggregate["memory_unknown_cases"] += int(
            capsule["status"] == "UNKNOWN")
        cases.append({"id": scenario["id"], "note": scenario["note"],
                      "raw_context": raw, "simple_retrieval": simple,
                      "mncs_memory": memory_result})
    count = max(len(cases), 1)
    aggregate["memory_required_fact_recall"] /= count
    aggregate["simple_required_fact_recall"] /= count
    aggregate["raw_required_fact_recall"] /= count
    aggregate["specialist_invocations"] = len(
        memory.store.records("invocations"))
    return {"schema_version": "mncs-memory.benchmark/1",
            "corpus": corpus.get("name", ""),
            "scenario_count": len(cases), "cases": cases,
            "aggregate": aggregate,
            "epistemic_note": "Frozen bounded synthetic corpus; results "
                              "are comparative observations, not general "
                              "memory performance claims."}
