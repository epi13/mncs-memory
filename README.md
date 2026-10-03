# MNCS Memory

<!-- MNCS:generated:begin -->
<!-- MNCS:generated:end -->

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

The canonical implementation is native MNCS with a thin host bridge.
Every semantic decision (disposition, claim states, relevance scoring,
selection, budget, gating, escalation, belief) is executed from
`native/mncs/memory/` through the current toolchain; the host owns
only files, digests, text handling, and the CLI.

```bash
export MNCS_LANGUAGE_ROOT=../mncs-language
export MNCS_TEST_NATIVE=../mncs-test/native
export MNCS_BIN=../mncs-language/target/debug/mncs
export MNCS_CACHE_DIR=~/.cache/mncs-memory
python3 -m unittest tests.test_memory
python3 tools/mncs_memory.py benchmark --corpus corpus/adversarial.json
python3 scripts/mncs-project-check.py --output /tmp/memory-check.json
```

The historical Rust implementation is preserved as an executable
oracle under `reference/` (9/9 of its own suite green) but is not on
any production path and is not gated by conformance. See
[`docs/RUST_RETIREMENT.md`](docs/RUST_RETIREMENT.md).

```text
native/mncs/memory/   canonical semantics (codes, lifecycle, recall, capsule)
tools/memory/         thin host bridge (JSONL store, specialists, engine, CLI)
tools/mncs_memory.py  CLI entry point
tests/test_memory.py  host suite (oracle parity, corpus, properties)
corpus/adversarial.json  frozen 21-observation / 11-query fixture
reference/            historical Rust oracle (non-canonical)
language/             0.6-era policy sources (superseded; oracle input only)
```

## MNCS Actions integration

The repository uses the pinned `mncs-actions` family workflow to run the
canonical suites, package results and execution evidence, and render the
badge above. The declared boundary is intentionally only
`mncs-memory-vertical-slice`: a `PASS` means the host suites plus the
native MNCS semantic suites passed. It does not claim full MNCS
conformance, rights/provenance review, or promotion authority.

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
