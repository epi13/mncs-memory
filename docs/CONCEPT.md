# Concept

The experiment reverses the usual memory relationship. A controlling general
reasoner sends a typed semantic request such as `WHAT_SHOULD_I_KNOW_BEFORE`.
Memory performs administration, entity resolution, consolidation, conflict
handling, ranking, and compression internally. The normal result is a compact
answer capsule, not a list of vaguely similar records.

The hypothesis is falsifiable: deterministic operators and narrow specialists
may reduce irrelevant context without lowering useful recall. Simple retrieval
may win on some workloads. The benchmark therefore preserves raw context,
simple retrieval, and MNCS Memory outputs side by side, including negative
cases and explicit `UNKNOWN` results.

The first slice measures characters/structured fields, not tokenizer tokens.
No token savings claim is made.
