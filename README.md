# MNCS Memory

[![MNCS memory tests](docs/mncs-badge.svg)](https://github.com/epi13/mncs-memory/actions/workflows/mncs-family.yml)

MNCS Memory is a bounded research implementation of an intelligent memory
subsystem for a general reasoner. The caller asks a semantic question and
receives a compact answer capsule; it does not normally receive a raw memory
dump.

This repository contains a working local vertical slice, not a production
memory service. It currently provides:

- append-oriented immutable observations in JSONL;
- canonical entities, semantic claims, explicit current/superseded/conflicted
  state, and provenance-bearing derivations;
- typed specialist contracts with deterministic reference specialists;
- ingestion that distinguishes `NEW`, `DUPLICATE`, `REFINEMENT`,
  `CONTRADICTION`, `SUPERSESSION`, and `UNKNOWN`;
- semantic query routing, relevance arbitration, contradiction/freshness
  handling, opt-in provenance, and bounded answer capsules;
- explicit escalation records and an intentionally faulty-specialist replay;
- raw-context, simple lexical-retrieval, and MNCS Memory benchmark paths;
- authoritative ingestion, budget, escalation, answer, and ranking decisions
  executed from linked MNCS Language source modules.

## Run it

The Rust host pins the executable MNCS Language crates to current main snapshot
`8447dad057107d812fa7f46af92f70ded74ee457`.

```bash
cargo test --test vertical_slice
cargo run --bin mncs-memory-benchmark -- language artifacts/benchmark.json
cargo run --bin mncs-memory-replay -- language artifacts/replay.json
cargo run --bin mncs-memory-backends -- language artifacts/backend-matrix.json
```

The benchmark, replay, and backend-matrix artifacts are inspectable JSON.
`language/mncs/memory/core.mncs`
links the policy modules and is called by the Rust host for every semantic
decision; it is not a decorative example.

## MNCS Actions integration

The repository uses the pinned `mncs-actions` family workflow to run the
bounded vertical-slice test suite, package its result and execution evidence,
and render the badge above. The declared boundary is intentionally only
`mncs-memory-vertical-slice`: a `PASS` means `cargo test --test vertical_slice`
passed. It does not claim full MNCS conformance, rights/provenance review, or
promotion authority.

The machine-readable badge sidecar is [`docs/mncs-badge.json`](docs/mncs-badge.json).
Workflow evidence is retained in the corresponding GitHub Actions run artifact.

## Scope

The deterministic reference specialists, body/SSA execution, JSONL store,
query capsule, replay path, benchmark, research-bytecode realization, and
portable-WASM realization run locally without a cloud model. The corpus is
small and synthetic, so it is evidence about this bounded workload only.

There are no neural micro-model weights, vector indexes, distributed storage,
online training, production durability guarantees, or general-reasoner
integration in this first slice. See [the research questions](docs/RESEARCH_QUESTIONS.md)
and [the roadmap](docs/ROADMAP.md).
