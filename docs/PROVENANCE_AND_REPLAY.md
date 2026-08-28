# Provenance and replay

Every claim retains `derived_from` observation identities and a `ProcessorRef`
containing role, identity, and version. It also preserves relationship edges
for supersession, refinement, and contradiction. Escalation events retain the
requesting layer, uncertainty rationale, bounded exposed context, and
resolution state.

The replay demonstration installs an intentionally faulty entity resolver that
maps the `operator` alias to Bob. The observation itself remains unchanged. A
replacement `reference-v1` resolver causes claims produced by the old processor
version to be appended as `INVALIDATED`, then recomputes claims from the
retained observation. The new claim points at the memory-repo entity and has
the replacement processor version.

This is selective by processor version at the current slice. Descendant graph
traversal and partial replay of only affected query capsules are future work.
