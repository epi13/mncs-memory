"""Memory engine: ingestion, recall, belief, and replay.

Ports the historical Rust orchestration with identical semantics. Text
handling (equality, containment, token overlap, alias maps, rendering)
stays host-side at the effect boundary; every semantic combination —
disposition, state transitions, eligibility, scores, selection,
budgets, gates, escalation, belief — is decided by native MNCS calls.
"""

from __future__ import annotations

from typing import Any

from . import codes
from . import model
from . import native as bridge
from . import specialists
from .store import MemoryStore

LIFECYCLE_PROGRAM = bridge.memory_program("lifecycle.mncs")
LIFECYCLE_MODULE = "mncs.memory.lifecycle.v1"
RECALL_PROGRAM = bridge.memory_program("recall.mncs")
RECALL_MODULE = "mncs.memory.recall.v1"
CAPSULE_PROGRAM = bridge.memory_program("capsule.mncs")
CAPSULE_MODULE = "mncs.memory.capsule.v1"


class EngineError(Exception):
    """A memory operation cannot complete honestly."""


def _native(fn_module: str, program: str, module: str, function: str,
            args: list[dict[str, Any]], mncs: str) -> Any:
    _ = fn_module
    try:
        return bridge.call_value(mncs=mncs, program=program, module=module,
                                 function=function, args=args)
    except bridge.NativeError as error:
        raise EngineError(f"native {function} failed: {error}")


def _native_record(program: str, module: str, function: str,
                   args: list[dict[str, Any]], mncs: str) -> dict[str, Any]:
    try:
        return bridge.call_record(mncs=mncs, program=program, module=module,
                                  function=function, args=args)
    except bridge.NativeError as error:
        raise EngineError(f"native {function} failed: {error}")


class MemoryEngine:
    def __init__(self, mncs: str, state_dir: str,
                 entities: dict[str, dict[str, Any]],
                 resolver: specialists.EntityResolver | None = None
                 ) -> None:
        self.mncs = mncs
        self.store = MemoryStore(state_dir)
        self.store.reload()
        if isinstance(entities, list):
            entities = {entity["identity"]: entity for entity in entities}
        self.entities = entities
        self.resolver = resolver or specialists.EntityResolver(entities)
        self._claim_sequence = self._max_claim_version()

    # -- native helpers -------------------------------------------------

    def _disposition(self, relation: int, evidence: int) -> int:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "decide_ingestion",
                       [bridge.integer(relation),
                        bridge.integer(evidence)], self.mncs)

    def _state_for(self, disposition: int) -> int:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "state_for", [bridge.integer(disposition)],
                       self.mncs)

    def _transition_prior(self, prior: int, disposition: int) -> int:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "transition_prior",
                       [bridge.integer(prior),
                        bridge.integer(disposition)], self.mncs)

    def _freshness(self, as_of: int, valid_from: int, required: int,
                   has_required: bool) -> int:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "freshness",
                       [bridge.integer(as_of), bridge.integer(valid_from),
                        bridge.integer(required),
                        bridge.boolean(has_required)], self.mncs)

    def _candidate_ok(self, state: int, confidence: int,
                      min_confidence: int, include_uncertainty: bool,
                      subject_match: bool, topic_match: bool,
                      forbidden_hit: bool, fresh_ok: bool) -> bool:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "candidate_ok",
                       [bridge.integer(state), bridge.integer(confidence),
                        bridge.integer(min_confidence),
                        bridge.boolean(include_uncertainty),
                        bridge.boolean(subject_match),
                        bridge.boolean(topic_match),
                        bridge.boolean(forbidden_hit),
                        bridge.boolean(fresh_ok)], self.mncs)

    def _related_match(self, disposition: int, same_topic: bool,
                       same_value: bool) -> bool:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "related_match",
                       [bridge.integer(disposition),
                        bridge.boolean(same_topic),
                        bridge.boolean(same_value)], self.mncs)

    def _worthy(self, kind_noise: bool, text_empty: bool) -> bool:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "worthy",
                       [bridge.boolean(kind_noise),
                        bridge.boolean(text_empty)], self.mncs)

    def _belief_gate(self, n_claims: int, n_distinct: int) -> int:
        return _native("lifecycle", LIFECYCLE_PROGRAM, LIFECYCLE_MODULE,
                       "belief_gate",
                       [bridge.integer(n_claims),
                        bridge.integer(n_distinct)], self.mncs)

    def _score(self, topic_match: int, subject_match: int,
               confidence: int, freshness: int) -> int:
        return _native("recall", RECALL_PROGRAM, RECALL_MODULE, "score",
                       [bridge.integer(topic_match),
                        bridge.integer(subject_match),
                        bridge.integer(confidence),
                        bridge.integer(freshness)], self.mncs)

    def _native_select(self, required_first: list[int],
                       cap: int) -> dict[str, Any]:
        n = len(required_first)
        if n > codes.RECALL_WIDTH:
            raise EngineError(
                f"recall width {n} exceeds {codes.RECALL_WIDTH}; "
                "filter before ranking")
        padded = [bridge.integer(v) for v in required_first]
        padded += [bridge.integer(0)] * (codes.RECALL_WIDTH - n)
        return _native_record(RECALL_PROGRAM, RECALL_MODULE,
                              "select_claims",
                       [bridge.integer(n),
                        {"sequence": {"values": padded}},
                        bridge.integer(cap)], self.mncs)

    def _gate(self, confidence: int, minimum: int, freshness: int,
              required: int) -> int:
        return _native("capsule", CAPSULE_PROGRAM, CAPSULE_MODULE, "gate",
                       [bridge.integer(confidence),
                        bridge.integer(minimum),
                        bridge.integer(freshness),
                        bridge.integer(required)], self.mncs)

    def _budget(self, required_units: int, budget_units: int,
                required_items: int) -> int:
        return _native("capsule", CAPSULE_PROGRAM, CAPSULE_MODULE,
                       "decide_budget",
                       [bridge.integer(required_units),
                        bridge.integer(budget_units),
                        bridge.integer(required_items)], self.mncs)

    def _escalation(self, layer: int, confidence: int) -> int:
        return _native("capsule", CAPSULE_PROGRAM, CAPSULE_MODULE,
                       "decide_escalation",
                       [bridge.integer(layer),
                        bridge.integer(confidence)], self.mncs)

    def _resolve(self, gated: int, budget: int, units_fit: bool,
                 conflict_inspect: bool, unknown_state: bool,
                 include_uncertainty: bool) -> dict[str, Any]:
        return _native_record(CAPSULE_PROGRAM, CAPSULE_MODULE,
                              "resolve_status",
                       [bridge.integer(gated), bridge.integer(budget),
                        bridge.boolean(units_fit),
                        bridge.boolean(conflict_inspect),
                        bridge.boolean(unknown_state),
                        bridge.boolean(include_uncertainty)], self.mncs)

    def _fit(self, status: int, answer_len: int, budget: int,
             conflict_inspect: bool = False) -> int:
        return _native("capsule", CAPSULE_PROGRAM, CAPSULE_MODULE,
                       "fit_marker",
                       [bridge.integer(status),
                        bridge.integer(answer_len),
                        bridge.integer(budget),
                        bridge.boolean(conflict_inspect)], self.mncs)

    # -- claims ----------------------------------------------------------

    def _max_claim_version(self) -> int:
        version = 0
        for claim in self.store.records("claims"):
            version = max(version, int(claim.get("claim_version", 0)))
        return version

    def claims(self) -> list[dict[str, Any]]:
        return sorted(self.store.records("claims"),
                      key=lambda claim: claim.get("identity", ""))

    # -- ingestion -------------------------------------------------------

    def ingest(self, document: dict[str, Any]) -> dict[str, Any]:
        obs = model.validate_observation(document)
        self.store.append("observations", obs)
        worth = specialists.worthiness(obs["kind"], obs["text"])
        invocations = [self._record_invocation(
            "worthiness", obs, worth, "deterministic")]
        policy = self._worthy(obs["kind"].lower() == "noise",
                              not obs["text"].strip())
        if (worth.decision == "NO") != (not policy):
            raise EngineError("worthiness contract/policy disagreement")
        if worth.decision == "NO":
            return self._ingest_unknown(obs, None, invocations, None)
        resolution = self.resolver.resolve(obs["subject_alias"])
        outcome = resolution.outcome or specialists.SpecialistOutcome(
            "UNKNOWN", 0, "UNKNOWN", "no resolution outcome")
        invocations.append(self._record_invocation(
            "entity-resolution", obs, outcome, "deterministic"))
        escalation_id = None
        # A faulty resolver still resolves: like the historical engine,
        # a decision naming a known entity produces a claim under the
        # faulty processor version (replay invalidates it by version).
        # Only an unresolvable decision degrades to Unknown.
        entity = self.entities.get(outcome.decision)
        if entity is None:
            record = self._record_escalation(
                "entity-resolver", codes.LAYER_DETERMINISTIC,
                outcome.confidence, outcome,
                self._live_claim_identities(),
                rationale="entity identity is unresolved; no raw store "
                          "was exposed")
            escalation_id = record["identity"]
            return self._ingest_unknown(obs, resolution, invocations,
                                        escalation_id)
        entity_id = entity["identity"]
        existing = [claim for claim in self.claims()
                    if claim.get("subject") == entity_id
                    and claim.get("state") not in (
                        codes.STATE_INVALIDATED, codes.STATE_SUPERSEDED,
                        codes.STATE_REFINED, codes.STATE_DUPLICATE)]
        relation = specialists.relationship(existing, obs)
        invocations.append(self._record_invocation(
            "relationship", obs, relation, "deterministic"))
        relation_code = {"UNRELATED": codes.REL_UNRELATED,
                         "SAME": codes.REL_SAME,
                         "REFINES": codes.REL_REFINES,
                         "CONTRADICTS": codes.REL_CONTRADICTS,
                         "SUPERSEDES": codes.REL_SUPERSEDES,
                         }.get(relation.decision)
        if relation_code is None:
            raise EngineError(
                f"unknown relation {relation.decision!r}")
        rel_conf = relation.confidence
        related = existing
        evidence_code = {"DIRECT": codes.EV_DIRECT,
                         "SUPPORTED": codes.EV_SUPPORTED,
                         "WEAK": codes.EV_WEAK,
                         "MISSING": codes.EV_MISSING}[obs["evidence"]]
        disposition = self._disposition(relation_code, evidence_code)
        confidence = min(codes.EVIDENCE_SCORE[evidence_code], rel_conf)
        escalation_id = None
        if confidence < 85 or disposition == codes.DISP_UNKNOWN:
            record = self._record_escalation(
                "ingestion-policy", codes.LAYER_DETERMINISTIC, confidence,
                relation,
                self._live_claim_identities(),
                rationale="relationship/evidence did not meet the MNCS "
                          "policy threshold")
            escalation_id = record["identity"]
        if disposition == codes.DISP_UNKNOWN:
            return self._ingest_unknown(obs, resolution, invocations,
                                        escalation_id)
        return self._commit_claim(obs, entity_id, related, disposition,
                                  confidence, invocations, escalation_id)

    def _ingest_unknown(self, obs: dict[str, Any], resolution: Any,
                        invocations: list | None,
                        escalation_id: str | None) -> dict[str, Any]:
        record = {"observation_id": obs["identity"],
                  "disposition": codes.DISPOSITIONS[codes.DISP_UNKNOWN],
                  "claim_id": None, "escalation": escalation_id,
                  "invocations": invocations or []}
        self.store.append("ingestions", record)
        return {"observation_id": obs["identity"],
                "disposition": codes.DISPOSITIONS[codes.DISP_UNKNOWN],
                "claim_id": None, "escalation": escalation_id}

    def _commit_claim(self, obs: dict[str, Any], entity_id: str,
                      related: list[dict[str, Any]], disposition: int,
                      confidence: int, invocations: list,
                      escalation_id: str | None) -> dict[str, Any]:
        processor = self.resolver.processor()
        identity = "claim:" + model.digest(
            [obs["identity"], entity_id,
             codes.DISPOSITIONS[disposition], processor])
        state = self._state_for(disposition)
        links = []
        for claim in related:
            same_topic = claim.get("topic") == obs["topic"]
            same_value = claim.get("value") == obs["value"]
            if not self._related_match(disposition, same_topic,
                                       same_value):
                continue
            kind = {codes.DISP_NEW: "New", codes.DISP_DUPLICATE: "Same",
                    codes.DISP_REFINEMENT: "Refines",
                    codes.DISP_CONTRADICTION: "Contradicts",
                    codes.DISP_SUPERSESSION: "Supersedes"}.get(disposition)
            if kind is None:
                continue
            links.append({"kind": kind, "target": claim["identity"]})
            if disposition in (codes.DISP_REFINEMENT,
                               codes.DISP_SUPERSESSION):
                self._claim_sequence += 1
                updated = dict(claim)
                updated["state"] = self._transition_prior(
                    claim["state"], disposition)
                updated["claim_version"] = self._claim_sequence
                self.store.replace_claim_state(claim["identity"], updated)
        self._claim_sequence += 1
        record = {"identity": identity, "observation_id": obs["identity"],
                  "entity_id": entity_id,
                  "disposition": codes.DISPOSITIONS[disposition],
                  "state": state, "evidence": obs["evidence"],
                  "confidence": confidence, "subject": entity_id,
                  "topic": obs["topic"], "value": obs["value"],
                  "source_observation_id": obs["identity"],
                  "claim_version": self._claim_sequence,
                  "derived_from": [obs["identity"]], "processor": processor,
                  "relationships": links,
                  "timestamps": {"observed_at": obs["observed_at"],
                                 "valid_from": obs["valid_from"],
                                 "claim_version": self._claim_sequence}}
        self.store.append("claims", record)
        ingestion = {"observation_id": obs["identity"],
                     "disposition": codes.DISPOSITIONS[disposition],
                     "claim_id": identity, "escalation": escalation_id,
                     "invocations": invocations}
        self.store.append("ingestions", ingestion)
        return {"observation_id": obs["identity"],
                "disposition": codes.DISPOSITIONS[disposition],
                "claim_id": identity, "escalation": escalation_id}

    # -- recall ----------------------------------------------------------

    def _record_invocation(self, role: str, obs: dict[str, Any],
                           outcome: specialists.SpecialistOutcome,
                           provider: str) -> dict[str, Any]:
        record = {
            "role": role, "provider": provider,
            "input_digest": model.digest(obs),
            "outcome": {"decision": outcome.decision,
                        "confidence": outcome.confidence,
                        "evidence": outcome.evidence,
                        "rationale": outcome.rationale,
                        "input_units": outcome.input_units,
                        "output_units": outcome.output_units}}
        self.store.append("invocations", record)
        return record

    def _record_escalation(self, requested_by: str, from_layer: int,
                           policy_confidence: int,
                           outcome: specialists.SpecialistOutcome,
                           exposed: list[str],
                           rationale: str | None = None
                           ) -> dict[str, Any]:
        # The layer policy runs natively; unlike the history, the
        # decided kind is kept on the record instead of discarded.
        text = rationale if rationale is not None else outcome.rationale
        decided = self._to_layer(self._escalation(from_layer,
                                                 policy_confidence))
        record = {"identity": "escalation:" + model.digest(
            [requested_by,
             {"decision": outcome.decision,
              "confidence": outcome.confidence,
              "evidence": outcome.evidence,
              "rationale": text,
              "input_units": outcome.input_units,
              "output_units": outcome.output_units}, exposed]),
            "requested_by": requested_by,
            "outcome": {"decision": outcome.decision,
                        "confidence": outcome.confidence,
                        "evidence": outcome.evidence,
                        "rationale": text,
                        "input_units": outcome.input_units,
                        "output_units": outcome.output_units},
            "exposed_context": exposed, "resolved": False,
            "layer": decided}
        self.store.append("escalations", record)
        return record

    def _live_claim_identities(self) -> list[str]:
        return sorted(claim["identity"] for claim in self.claims())

    def _to_layer(self, layer: int) -> dict[str, str]:
        if layer == codes.ESC_NONE:
            return {"kind": "none",
                    "reason": "resolved at current layer"}
        if layer == codes.ESC_SPECIALIST:
            return {"kind": "specialist",
                    "reason": "UNKNOWN; stronger specialist unavailable"}
        if layer == codes.ESC_GENERAL:
            return {"kind": "general-reasoner",
                    "reason": "UNKNOWN; caller must decide whether to "
                              "escalate"}
        return {"kind": "unknown",
                "reason": "UNKNOWN; escalation authority exhausted"}

    def _record_query_escalation(self, requested_by: str,
                                 exposed: list[str]
                                 ) -> dict[str, str]:
        outcome = specialists.SpecialistOutcome(
            "UNKNOWN", 60, "UNKNOWN",
            "query arbitration requires unresolved context")
        record = self._record_escalation(requested_by,
                                           codes.LAYER_SPECIALIST, 60,
                                           outcome, exposed)
        return record["layer"]

    def query(self, document: dict[str, Any]) -> dict[str, Any]:
        request = model.validate_request(document)
        route = specialists.route(request["intent"])
        self._record_invocation(
            "route", {"intent": request["intent"]}, route,
            "deterministic")
        if request["intent"] == "RAW_EPISODES":
            return self._raw_episodes(request)
        candidates = self._candidates(request)
        scored = []
        for claim in candidates:
            topic_match = self._topic_match(request, claim)
            subject_match = 1 if self._subject_match(request, claim) else 0
            fresh = self._freshness(request["as_of"],
                                    claim["timestamps"]["valid_from"],
                                    request["freshness_required"],
                                    request["freshness_required"] > 0)
            score = self._score(topic_match, subject_match,
                                claim["confidence"], fresh)
            scored.append((score, claim["identity"], claim))
        scored.sort(key=lambda entry: (-entry[0], entry[1]))
        groups: dict[tuple[str, str], set[str]] = {}
        for _, _, claim in scored:
            groups.setdefault((claim["subject"], claim["topic"]),
                              set()).add(claim["value"])
        conflict = any(len(values) > 1 for values in groups.values())
        uncertainty: list[str] = []
        status = codes.ANS_PASS
        escalation = {"kind": "none", "reason": "resolved at current layer"}
        if conflict and not request["include_conflicts"]:
            escalation = self._record_query_escalation(
                "query-arbitration",
                sorted({claim["identity"] for _, _, claim in scored}))
            uncertainty.append("unresolved-conflict")
            selected: list[dict[str, Any]] = []
        else:
            selected = self._select(request, scored)
        if conflict and request["include_conflicts"]:
            uncertainty.append("deliberate-conflict-inspection")
        required_preserved = all(
            any(claim["topic"] == topic for _, _, claim in selected)
            for topic in request["required_topics"])
        if not selected or not required_preserved:
            status = codes.ANS_UNKNOWN
            if not any(entry == "unresolved-conflict"
                       for entry in uncertainty):
                if not scored:
                    uncertainty.append("no-eligible-claims")
                else:
                    uncertainty.append("unknown-state")
        # The gate runs over the selected claims, mirroring the
        # historical pipeline (minimum selected confidence/freshness),
        # never over hardcoded constants.
        aggregate_confidence = min(
            [claim["confidence"] for _, _, claim in selected] or [0])
        aggregate_freshness = min(
            [self._freshness(request["as_of"],
                             claim["timestamps"]["valid_from"],
                             request["freshness_required"],
                             request["freshness_required"] > 0)
             for _, _, claim in selected] or [0])
        gated = self._gate(aggregate_confidence,
                           request["minimum_confidence"],
                           aggregate_freshness,
                           request["freshness_required"])
        required_units = sum(
            len(claim["subject"]) + len(claim["topic"])
            + len(claim["value"]) + 3
            for _, _, claim in scored
            if claim["topic"] in request["required_topics"])
        budget = self._budget(len(self._render(selected)),
                              request["max_units"], len(selected))
        resolution = self._resolve(
            gated if status == codes.ANS_PASS else codes.ANS_UNKNOWN,
            budget,
            required_units <= request["max_units"],
            conflict and request["include_conflicts"],
            any(claim["state"] == codes.STATE_UNKNOWN for _, _, claim in
                scored) and request["include_uncertainty"],
            request["include_uncertainty"])
        status = resolution["status"]
        reason = codes.UNCERTAINTY[resolution["uncertainty"]]
        if reason is not None and reason not in uncertainty:
            uncertainty.append(reason)
        answer = self._render(selected)
        marker = self._fit(status, len(answer), request["max_units"],
                           conflict and request["include_conflicts"])
        if marker != codes.ANS_PASS:
            answer = codes.ANSWERS[marker][:request["max_units"]]
        elif len(answer) > request["max_units"]:
            answer = answer[:request["max_units"]]
        items = [self._answer_item(request, claim) for _, _, claim in
                 selected]
        return {"status": codes.ANSWERS[status], "answer": answer,
                "items": items, "units": len(answer),
                "candidates_exposed": len(scored),
                "uncertainty": uncertainty, "escalation": escalation,
                "provenance_included": request["include_provenance"],
                "conflict_included": conflict and
                request["include_conflicts"],
                "raw_episodes_included": False}

    def _topic_match(self, request: dict[str, Any],
                     claim: dict[str, Any]) -> int:
        goal = " ".join([request["goal"]] + request["required_topics"]
                        + [request["topic"] or ""])
        lexical = specialists.relevance(goal, request["topic"] or "",
                                        claim["value"])
        self._record_invocation(
            "relevance", {"goal": goal, "topic": request["topic"],
                          "value": claim["value"]}, lexical,
            "deterministic")
        count = int(lexical.decision)
        if request["topic"] is not None and \
                request["topic"] == claim["topic"]:
            count += 1
        return count

    def _subject_match(self, request: dict[str, Any],
                       claim: dict[str, Any]) -> bool:
        if request["subject"] is None:
            return True
        return request["subject"] == claim["subject"]

    def _candidates(self, request: dict[str, Any]) -> list[dict[str, Any]]:
        found = []
        for claim in self.claims():
            if not self._candidate_ok(
                    claim["state"], claim["confidence"],
                    request["minimum_confidence"],
                    request["include_uncertainty"],
                    self._subject_match(request, claim),
                    request["topic"] is None
                    or request["topic"] == claim["topic"],
                    claim["topic"] in request["forbidden_topics"],
                    self._freshness(
                        request["as_of"],
                        claim["timestamps"]["valid_from"],
                        request["freshness_required"],
                        request["freshness_required"] > 0) > 0):
                continue
            if request["required_topics"] and \
                    claim["topic"] not in request["required_topics"]:
                goal = " ".join([request["goal"]]
                                + request["required_topics"]
                                + [request["topic"] or ""])
                lexical = specialists.relevance(
                    goal, request["topic"] or "", claim["value"])
                self._record_invocation(
                    "relevance", {"goal": goal,
                                  "topic": request["topic"],
                                  "value": claim["value"]}, lexical,
                    "deterministic")
                if int(lexical.decision) == 0:
                    continue
            found.append(claim)
        return found

    def _select(self, request: dict[str, Any],
                scored: list[tuple[int, str, dict[str, Any]]]
                ) -> list[tuple[int, str, dict[str, Any]]]:
        required_first = []
        seen_required: set[str] = set()
        for _, _, claim in scored:
            if claim["topic"] in request["required_topics"] \
                    and claim["topic"] not in seen_required:
                seen_required.add(claim["topic"])
                required_first.append(1)
            else:
                required_first.append(0)
        selection = self._native_select(required_first,
                                        request["max_supporting_claims"])
        mask = selection["mask"]
        return [entry for flag, entry in zip(mask, scored) if flag == 1]

    def _render(self, selected: list[tuple[int, str, dict[str, Any]]
                                     ]) -> str:
        return ";".join(f"{claim['topic']}={claim['value']}"
                        for _, _, claim in selected)

    def _answer_item(self, request: dict[str, Any],
                     claim: dict[str, Any]) -> dict[str, Any]:
        item = {"subject": claim["subject"], "topic": claim["topic"],
                "value": claim["value"], "confidence": claim["confidence"],
                "derived_from": list(claim["derived_from"]),
                "processor": dict(claim["processor"])}
        if request["include_provenance"]:
            item["provenance"] = {
                "supporting_claims": [
                    link["target"] for link in claim["relationships"]
                    if link["kind"] in ("Refines", "Same")],
                "observation_identities": list(claim["derived_from"])}
        return item

    def _raw_episodes(self, request: dict[str, Any]) -> dict[str, Any]:
        observations = [
            obs for obs in self.store.records("observations")
            if (request["subject"] is None
                or obs["subject_alias"] == request["subject"]
                or request["subject"] in obs["text"])
            and (request["topic"] is None
                 or obs["topic"] == request["topic"])]
        answer = "\n".join(obs["text"] for obs in observations)
        budget = self._budget(len(answer), request["max_units"],
                              len(observations))
        if budget != codes.BUDGET_FITS:
            status = codes.ANS_UNKNOWN
            uncertainty: list[str] = ["required-information-over-budget"]
        else:
            status = codes.ANS_PASS
            uncertainty = []
        marker = self._fit(status, len(answer), request["max_units"])
        if marker != codes.ANS_PASS:
            answer = codes.ANSWERS[marker][:request["max_units"]]
        elif len(answer) > request["max_units"]:
            answer = answer[:request["max_units"]]
        return {"status": codes.ANSWERS[status], "answer": answer,
                "items": [], "units": len(answer),
                "candidates_exposed": len(observations),
                "uncertainty": uncertainty,
                "escalation": {"kind": "none",
                               "reason": "resolved at current layer"},
                "provenance_included": False, "conflict_included": False,
                "raw_episodes_included": True}

    # -- belief / replay --------------------------------------------------

    def current_belief(self, subject: str, topic: str) -> dict[str, Any]:
        matching = [claim for claim in self.claims()
                    if claim["subject"] == subject
                    and claim["topic"] == topic
                    and claim["state"] in (codes.STATE_CURRENT,
                                           codes.STATE_UNRESOLVED)]
        values = sorted({claim["value"] for claim in matching})
        status = self._belief_gate(len(matching), len(values))
        if status == codes.ANS_PASS:
            return {"status": "PASS", "value": values[0]}
        return {"status": "UNKNOWN", "value": None}

    def replay(self, resolver: specialists.EntityResolver,
               observations: list[dict[str, Any]]) -> dict[str, Any]:
        """Invalidate claims from older processors, then recompute."""
        current = resolver.processor()["version"]
        invalidated = 0
        for claim in self.claims():
            if claim["processor"].get("version") == current:
                continue
            if claim["state"] == codes.STATE_INVALIDATED:
                continue
            self._claim_sequence += 1
            updated = dict(claim)
            updated["state"] = codes.STATE_INVALIDATED
            updated["claim_version"] = self._claim_sequence
            self.store.replace_claim_state(claim["identity"], updated)
            invalidated += 1
        self.resolver = resolver
        recomputed = 0
        for document in observations:
            record = self.ingest(document)
            if record.get("claim_id") is not None:
                recomputed += 1
        return {"invalidated": invalidated, "recomputed": recomputed,
                "processor": current}
