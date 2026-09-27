# Rust retirement

## What Rust used to own

The historical implementation (`reference/`, previously the repository
root) owned the entire vertical slice: JSON types (`model.rs`),
JSONL persistence (`storage.rs`), deterministic specialists
(`providers.rs`), the old-toolchain language bridge (`language.rs`,
pinned `mncs-language` rev `8447dad`), orchestration
(`engine.rs`: ingestion, query, belief, replay), benchmark/scenario
harness (`benchmark.rs`, `scenarios.rs`, three `src/bin/` drivers),
and the canonical test gate (`tests/vertical_slice.rs`, 9 tests).

## What moved into native MNCS

Every semantic decision, now under `native/mncs/memory/` with test
blocks and no Rust in the call path:

- ingestion disposition matrix, claim states, prior transitions,
  worthiness, freshness, candidate predicate, relationship-match
  predicate, belief gate (`lifecycle.mncs`);
- relevance score formula incl. wrapping arithmetic, required-first
  bounded selection (`recall.mncs`);
- answer gate, budget policy, escalation policy, status resolution,
  fit markers (`capsule.mncs`).

## What moved to the thin host

`tools/memory/` owns only transport: JSONL appends with fsync,
sha256 digests, string equality/containment/tokenization, alias maps,
host-side sorting (score desc, identity asc), text rendering, CLI.
It encodes no learning/memory policy; disagreements between the
specialist contracts and the native policy raise instead of
resolving silently.

## What was replaced by current infrastructure

- Private JSONL persistence stays file-based (atomic appends), but
  the durable-artifact story is now Store generations (pressure
  MEM-P4); no new Rust storage work will happen here.
- Provenance travels as `derived_from` + processor refs preserved for
  Lineage; no private derivation graph is maintained.
- Tests run under `mncs-test` (native) and `unittest` (host), not the
  Rust harness.

## What Rust remains, and why

`reference/` keeps the exact historical package (bins, benchmark,
scenarios, vertical-slice suite: 9/9 green) building offline against
its pinned toolchain, as an **executable oracle**. It is not
imported, invoked, or gated by normal operation: the conformance
check (`scripts/mncs-project-check.py`) runs only the native + host
suites. Keep it while parity questions remain; delete it when the
oracle has no further questions to answer.

The 0.6-era policy sources in `language/` are retained solely as the
reference runtime's policy input. They are superseded by
`native/mncs/memory/` and must not be extended; no production path
reads them except the oracle.

## Why it is not semantic authority

The old `.mncs` policy modules targeted a superseded compiler
generation; the orchestration lived in Rust where MNCS can now
express it. Behavioral parity was verified against the running Rust
oracle (20/20 corpus dispositions, 11/11 query expectations, replay
versions, benchmark aggregates) before retirement, then the
production path was switched to MNCS.
