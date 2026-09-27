"""Host suite for the canonical native mncs-memory path.

Every memory decision (disposition, state transition, score,
selection, budget, gate, escalation, belief) comes from a native
`mncs call`. The historical Rust implementation serves as the
behavioral oracle: the corpus dispositions, query expectations,
replay versions, and benchmark aggregates below are the documented
Rust outcomes, reproduced here through MNCS.
"""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
TOOLS = REPO / "tools"
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

from memory import benchmark, codes, engine, model, native, specialists  # noqa: E402
from memory.store import MemoryStore, StoreError  # noqa: E402

MNCS = native.find_mncs()


def need_mncs(test):
    return unittest.skipIf(MNCS is None, "mncs binary unavailable")(test)


def corpus() -> dict[str, Any]:
    with open(REPO / "corpus" / "adversarial.json",
              encoding="utf-8") as handle:
        return json.load(handle)


def observations() -> list[dict[str, Any]]:
    return corpus()["observations"]


class Case(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="memory-test-"))
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def make_engine(self, entities=None):
        data = corpus()
        return engine.MemoryEngine(
            MNCS, str(self.tmp / "state"),
            entities if entities is not None else data["entities"])

    def ingest_all(self, memory):
        records = []
        for observation in observations():
            records.append((observation["id"], memory.ingest(observation)))
        return dict(records)


class CodesMirrorTests(Case):
    @need_mncs
    def test_native_codes_match_host(self) -> None:
        pairs = [("disp_new", codes.DISP_NEW),
                 ("disp_contradiction", codes.DISP_CONTRADICTION),
                 ("disp_unknown", codes.DISP_UNKNOWN),
                 ("state_current", codes.STATE_CURRENT),
                 ("state_invalidated", codes.STATE_INVALIDATED),
                 ("ans_pass", codes.ANS_PASS),
                 ("budget_insufficient", codes.BUDGET_INSUFFICIENT),
                 ("esc_specialist", codes.ESC_SPECIALIST),
                 ("rel_ambiguous", codes.REL_AMBIGUOUS),
                 ("ev_missing", codes.EV_MISSING)]
        for function, expected in pairs:
            document = native.call_function(
                mncs=MNCS,
                program=str(REPO / "native" / "mncs" / "memory" /
                            "codes.mncs"),
                module="mncs.memory.codes.v1", function=function, args=[])
            value = native.plain(document["call"]["returned"][0])
            self.assertEqual(value, expected, function)


class IngestionTests(Case):
    @need_mncs
    def test_corpus_dispositions_by_observation(self) -> None:
        memory = self.make_engine()
        ingestions = self.ingest_all(memory)
        # Oracle dispositions from the historical implementation.
        expected = {
            "repeat-1": "New", "repeat-2": "Duplicate",
            "change-1": "Supersession", "contradiction-1": "New",
            "contradiction-2": "Contradiction", "refinement-1": "New",
            "refinement-2": "Refinement", "ambiguous-alias": "Unknown",
            "two-names-lee": "New", "two-names-morgan": "New",
            "stale-claim": "New", "superseded-claim": "New",
            "superseding-claim": "Supersession", "similar-vector": "New",
            "similar-vector-irrelevant": "New", "multi-1": "New",
            "multi-2": "New", "weak-1": "New", "weak-2": "Unknown",
            "noise-1": "Unknown", "replay-alias": "New",
        }
        for obs_id, disposition in expected.items():
            self.assertEqual(ingestions[obs_id]["disposition"],
                             disposition, obs_id)

    @need_mncs
    def test_claims_carry_provenance_and_states(self) -> None:
        memory = self.make_engine()
        self.ingest_all(memory)
        by_obs = {}
        for claim in memory.claims():
            by_obs.setdefault(claim["observation_id"], []).append(claim)
        dup = by_obs["observation:repeat-2"][0]
        self.assertEqual(dup["state"], codes.STATE_DUPLICATE)
        self.assertEqual(dup["derived_from"], ["observation:repeat-2"])
        self.assertEqual(dup["processor"]["version"], "reference-v1")
        contra = by_obs["observation:contradiction-2"][0]
        self.assertEqual(contra["state"], codes.STATE_UNRESOLVED)
        kinds = {link["kind"] for link in contra["relationships"]}
        self.assertIn("Contradicts", kinds)
        # Supersession transitions the prior claim, preserving history.
        old = [claim for claim in memory.claims()
               if claim["observation_id"] == "observation:superseded-claim"]
        self.assertTrue(old)
        self.assertEqual(old[0]["state"], codes.STATE_SUPERSEDED)
        new = by_obs["observation:superseding-claim"][0]
        self.assertEqual(new["state"], codes.STATE_CURRENT)

    @need_mncs
    def test_conflicting_memories_coexist(self) -> None:
        memory = self.make_engine()
        self.ingest_all(memory)
        values = sorted(
            claim["value"] for claim in memory.claims()
            if claim["subject"] == "entity:alice"
            and claim["topic"] == "timezone"
            and claim["state"] in (codes.STATE_CURRENT,
                                   codes.STATE_UNRESOLVED))
        self.assertEqual(values, ["America/Los_Angeles", "UTC"])
        belief = memory.current_belief("entity:alice", "timezone")
        self.assertEqual(belief, {"status": "UNKNOWN", "value": None})

    @need_mncs
    def test_distinct_texts_do_not_collapse(self) -> None:
        memory = self.make_engine()
        self.ingest_all(memory)
        identities = {claim["identity"] for claim in memory.claims()}
        # 20 observations, noise/ambiguous produce no claims: every claim
        # identity is distinct even where values repeat.
        self.assertEqual(len(identities), len(memory.claims()))
        self.assertGreater(len(identities), 10)


class QueryTests(Case):
    @need_mncs
    def test_all_corpus_queries_match_expectations(self) -> None:
        data = corpus()
        memory = self.make_engine(data["entities"])
        self.ingest_all(memory)
        for scenario in data["queries"]:
            capsule = memory.query(scenario["request"])
            self.assertEqual(capsule["status"],
                             scenario["expected_status"], scenario["id"])
            for topic in scenario["expected_topics"]:
                self.assertTrue(
                    any(item["topic"] == topic
                        for item in capsule["items"]),
                    f"{scenario['id']} missing topic {topic}")
            for value in scenario["expected_values"]:
                self.assertIn(value, capsule["answer"], scenario["id"])

    @need_mncs
    def test_recall_is_bounded_and_explained(self) -> None:
        data = corpus()
        memory = self.make_engine(data["entities"])
        self.ingest_all(memory)
        request = {"intent": "WHAT_IS_KNOWN_ABOUT",
                   "subject": "memory-repo", "max_supporting_claims": 1,
                   "max_units": 700}
        capsule = memory.query(request)
        self.assertLessEqual(len(capsule["items"]), 1)
        self.assertLessEqual(capsule["units"], 700)

    @need_mncs
    def test_repeat_query_is_deterministic(self) -> None:
        data = corpus()
        memory = self.make_engine(data["entities"])
        self.ingest_all(memory)
        request = data["queries"][0]["request"]
        first = memory.query(request)
        second = memory.query(request)
        self.assertEqual(first, second)

    @need_mncs
    def test_scope_does_not_leak(self) -> None:
        data = corpus()
        memory = self.make_engine(data["entities"])
        self.ingest_all(memory)
        capsule = memory.query({"intent": "WHAT_IS_KNOWN_ABOUT",
                                "subject": "scheduler"})
        for item in capsule["items"]:
            self.assertEqual(item["subject"], "scheduler")


class PersistenceTests(Case):
    @need_mncs
    def test_restart_recovers_canonical_state(self) -> None:
        data = corpus()
        first = self.make_engine(data["entities"])
        self.ingest_all(first)
        before = first.query(data["queries"][0]["request"])
        second = engine.MemoryEngine(MNCS, str(self.tmp / "state"),
                                     data["entities"])
        after = second.query(data["queries"][0]["request"])
        self.assertEqual(before, after)
        self.assertEqual(len(second.claims()), len(first.claims()))

    @need_mncs
    def test_store_files_are_append_only_jsonl(self) -> None:
        memory = self.make_engine()
        self.ingest_all(memory)
        state = Path(self.tmp / "state")
        for name in ("observations", "claims", "ingestions",
                     "escalations", "invocations"):
            path = state / f"{name}.jsonl"
            self.assertTrue(path.is_file(), name)
            for line in path.read_text(encoding="utf-8").splitlines():
                if line.strip():
                    json.loads(line)

    @need_mncs
    def test_corrupt_tail_is_explainable(self) -> None:
        memory = self.make_engine()
        self.ingest_all(memory)
        with open(self.tmp / "state" / "claims.jsonl", "a",
                  encoding="utf-8") as handle:
            handle.write("{not json\n")
        with self.assertRaises(StoreError) as raised:
            MemoryStore(str(self.tmp / "state")).reload()
        self.assertIn("claims.jsonl line", str(raised.exception))

    @need_mncs
    def test_replay_from_retained_observations(self) -> None:
        data = corpus()
        memory = self.make_engine(data["entities"])
        self.ingest_all(memory)
        faulty = specialists.EntityResolver(
            data["entities"], faulty_alias="operator",
            faulty_target="entity:bob", version="faulty-entity-v1")
        report = memory.replay(faulty, observations())
        self.assertEqual(report["processor"], "faulty-entity-v1")
        by_obs: dict[str, list[dict[str, Any]]] = {}
        for claim in memory.claims():
            by_obs.setdefault(claim["observation_id"], []).append(claim)
        old = [claim for claim in by_obs["observation:replay-alias"]
               if claim["processor"]["version"] == "reference-v1"]
        self.assertTrue(old)
        self.assertEqual(old[0]["state"], codes.STATE_INVALIDATED)
        new = [claim for claim in by_obs["observation:replay-alias"]
               if claim["processor"]["version"] == "faulty-entity-v1"]
        self.assertTrue(new)
        self.assertEqual(new[0]["entity_id"], "entity:bob")
        self.assertEqual(
            new[0]["observation_id"], "observation:replay-alias")


class BenchmarkTests(Case):
    @need_mncs
    def test_benchmark_parity_with_oracle(self) -> None:
        report = benchmark.run_corpus(MNCS, corpus())
        aggregate = report["aggregate"]
        self.assertEqual(report["scenario_count"], 11)
        # Documented Rust outcomes, reproduced through MNCS.
        self.assertEqual(aggregate["memory_correct"], 11)
        self.assertEqual(aggregate["simple_retrieval_correct"], 6)
        self.assertEqual(
            aggregate["memory_irrelevant_fact_inclusion"], 0)
        self.assertGreater(
            aggregate["simple_irrelevant_fact_inclusion"], 0)
        self.assertLess(aggregate["memory_units_total"],
                        aggregate["simple_units_total"])
        self.assertLess(aggregate["memory_candidates_total"],
                        aggregate["simple_candidates_total"])
        self.assertAlmostEqual(
            aggregate["memory_required_fact_recall"], 1.0)


class ValidationTests(unittest.TestCase):
    def test_rejects_bad_inputs(self) -> None:
        with self.assertRaises(model.MemoryError):
            model.validate_observation({"id": "x"})
        with self.assertRaises(model.MemoryError):
            model.validate_observation({
                "id": "x", "kind": "vibe", "subject_alias": "a",
                "topic": "t", "value": "v", "text": "t"})
        with self.assertRaises(model.MemoryError):
            model.validate_request({"intent": "REMEMBER_EVERYTHING"})

    def test_specialist_contracts(self) -> None:
        self.assertEqual(
            specialists.worthiness("noise", "hello").decision, "NO")
        self.assertEqual(
            specialists.worthiness("fact", "  ").decision, "NO")
        self.assertEqual(
            specialists.worthiness("fact", "hello").decision, "YES")
        same = [{"topic": "t", "value": "abc"}]
        self.assertEqual(
            specialists.relationship(
                same, {"topic": "t", "value": "abc"}).decision, "SAME")
        self.assertEqual(
            specialists.relationship(
                same, {"topic": "t", "value": "abd",
                       "refinement": True}).decision, "REFINES")
        self.assertEqual(
            specialists.relationship(
                same, {"topic": "t", "value": "abd",
                       "explicit_change": True}).decision, "SUPERSEDES")
        self.assertEqual(
            specialists.relationship(
                same, {"topic": "t", "value": "abd"}).decision,
            "CONTRADICTS")
        self.assertEqual(
            specialists.relationship(
                same, {"topic": "u", "value": "abc"}).decision,
            "UNRELATED")
        self.assertEqual(
            specialists.relationship([], {"topic": "t", "value": "v",
                                           "explicit_change": True,
                                           "refinement": True}).decision,
            "UNRELATED")
        self.assertEqual(
            specialists.relevance("stay dry", "", "Use an umbrella"
                                  ).decision, "0")
        self.assertEqual(
            specialists.relevance("umbrella rain", "", "Take umbrella"
                                  ).decision, "1")


if __name__ == "__main__":
    unittest.main()
