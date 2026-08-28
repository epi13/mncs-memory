# Memory model

The implemented layers are:

1. `Observation`: append-oriented input with source, logical observation time,
   immutable digest, and the original structured input.
2. `Entity`: canonical identity plus aliases. Ambiguous aliases remain
   unresolved.
3. `Claim`: a derived subject/topic/value record with validity, confidence,
   evidence, state, lineage, processor identity/version, relationship links,
   and timestamps.
4. `AnswerCapsule`: query-specific output with status, budget, selected items,
   candidate count, uncertainty, escalation, and optional provenance.

Observations are not rewritten. Claims are versioned append records; a state
transition appends the newer claim record and the store reloads the latest
state for an identity. This keeps raw evidence available for replay while
allowing derived beliefs to change.

States are intentionally not collapsed into truth values:
`CURRENT`, `REFINED`, `SUPERSEDED`, `CONTRADICTED`, `UNRESOLVED`, `DUPLICATE`,
`UNKNOWN`, and `INVALIDATED` are observable. A contradiction keeps both values
available for deliberate conflict inspection, but ordinary belief queries
return `UNKNOWN`.
