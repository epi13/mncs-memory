# Query model

`SemanticRequest` makes delivery constraints first class: typed intent,
optional canonical subject/topic, goal text, `as_of`, freshness requirement,
maximum character/structured-unit budget, minimum confidence, required and
forbidden topics, maximum supporting claims, and opt-in provenance, conflict,
uncertainty, or raw-episode delivery.

Normal query execution routes the intent, filters to current eligible claims,
extracts relevance features, invokes the MNCS rank policy, detects active
conflicts, applies the answer-confidence gate, renders the smallest selected
capsule, and applies the MNCS budget decision. A too-small budget returns
`UNKNOWN` and states that required information could not be preserved.

Provenance is omitted by default. When requested, an answer item includes the
claim's supporting claim links and observation identities. `RAW_EPISODES` is a
deliberate alternate request, not an accidental fallback.
