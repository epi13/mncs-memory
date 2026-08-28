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
is in `language/mncs/memory/`: policy, budget, escalation, query, and the
linked core entry point. This keeps the application adapter thin while
retaining a practical boundary for future model providers.

The store is replaceable. Nothing in the public query API requires a vector
database, graph database, or particular physical index.
