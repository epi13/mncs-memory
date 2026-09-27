# Architecture

```text
caller -> SemanticRequest -> typed query route
       -> bounded claims and specialist arbitration
       -> MNCS answer/budget policy -> AnswerCapsule

ObservationInput -> worthiness -> entity resolver -> relationship specialist
                 -> MNCS ingestion policy -> provenance-bearing Claim
                 -> JSONL MemoryStore
```

`MemoryEngine` is the orchestration boundary. It owns no unbounded model
prompt interface. A `SpecialistRequest` has a declared role and structured
input; a `SpecialistOutcome` returns a bounded decision, confidence, evidence,
and rationale. Every invocation is recorded with an input digest and provider
identity/version.

The host owns filesystem access, JSONL storage, content digests, typed provider
invocation, and conversion to MNCS finite/integer values. The semantic policy
is in `native/mncs/memory/`: lifecycle (worthiness, disposition, claim
states, freshness, candidacy, relationship matching, belief), recall
(relevance scoring, bounded required-first selection), and capsule
(answer gating, budget, escalation, status resolution). This keeps the
application adapter thin while retaining a practical boundary for
future model providers.

```text
source evidence
  -> Memory semantics (native/mncs/memory: what is a memory, how it
     relates, what is recalled and why)
  -> Store persistence (JSONL today, Store generations tomorrow;
     canonical records, fsync, latest-wins claim state)
  -> Lineage-ready provenance (derived_from, processor refs,
     supersedes/refines/contradicts links preserved, not re-graphed)
  -> bounded recall (budgeted capsules with reason codes)
  -> consuming systems (RAVEL/agents decide usefulness; Memory never
     plans, never manages context budgets beyond its own capsule)
```

Memory explicitly does NOT own: durable storage machinery (Store),
derivation graphs (Lineage), canonical identities (Commons),
learning algorithms and ranking models (Learn), experiment sweeps
(Lab), scheduling (Automation), text processing (host effect
boundary). Text equality, tokenization, alias maps, and rendering
stay host-side; every semantic combination is decided natively.

The store is replaceable. Nothing in the public query API requires a vector
database, graph database, or particular physical index.
