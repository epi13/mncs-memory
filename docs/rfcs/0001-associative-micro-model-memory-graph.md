# MNCS Memory RFC 0001: Associative Micro-Model Memory Graph

**Status:** Draft  
**Project:** `mncs-memory`  
**RFC:** 0001  
**Title:** Associative Micro-Model Memory Graph  
**Authors:** MNCS Project  
**Created:** 2026-09-06

## Abstract

This RFC defines an experimental memory architecture in which persistent machine memory is represented not primarily as records, embeddings, or a single monolithic learned model, but as a dynamic graph of small specialized computational units called **memory cells**.

Each memory cell contains or references a bounded **micro-model** specialized for the information, relationship, behavior, or semantic region represented by that cell.

Memory cells expose local semantic directions called **latent ports**. Ports may remain unconnected, may form explicit connections to ports on neighboring cells, or may become associated with relationships elsewhere in the graph.

Connections are first-class persistent objects called **synapses**. A synapse records the relationship between two memory structures together with strength, confidence, evidence, provenance, activation history, and lifecycle state.

The resulting system is intended to support:

- local specialization;
- sparse activation;
- associative recall;
- incremental learning;
- explicit uncertainty;
- inspectable provenance;
- relationship-to-relationship association;
- topology growth;
- bounded computation;
- natural reinforcement and decay;
- distributed placement; and
- memory behavior without requiring continual retraining of a monolithic model.

The central hypothesis of this RFC is:

> **Useful persistent machine memory can emerge from locally specialized machine behavior plus learned topology between those behaviors.**

No individual memory cell is required to represent the complete system. Global memory behavior emerges from the interaction of many bounded local units.

---

## 1. Motivation

Conventional machine-memory systems commonly store information as documents, records, chunks, vectors, key-value pairs, or some combination of these.

A general reasoner then retrieves candidate information and performs most of the semantic work itself.

MNCS Memory already experiments with reversing this relationship: memory performs administration, entity resolution, consolidation, conflict handling, ranking, and compression internally, returning bounded semantic results rather than exposing an undifferentiated store to the controlling reasoner.

This RFC extends that principle.

Rather than treating memory as a passive store operated upon by specialized processors, the memory substrate itself becomes a network of small specialized computational memories.

Conceptually:

```text
                 ┌───────────────┐
                 │  Memory Cell  │
                 │      A        │
                 └───────●───────┘
                     ╱   │   ╲
                    ╱    │    ╲
                   ╱     │     ╲
                  ▼      ▼      ▼

            latent ports / relationships

                  │      │
                  │      │
                  ▼      ▼

              other cells
```

The analogy to biological neurons is useful but limited.

A biological neuron is an extremely small computational element whose significance largely derives from its state and connectivity.

An MNCS memory cell follows the same architectural principle of **local simplicity plus network significance**, but the cell itself may contain substantially richer computation.

It MAY be implemented using:

- a deterministic MNCS operator;
- a state machine;
- a tiny classifier;
- a vector transform;
- a domain-specific specialist;
- a small neural network;
- a WASM module;
- another bounded model representation.

A memory cell MUST NOT be assumed to be a miniature general-purpose language model.

---

## 2. Relationship to Existing MNCS Memory

This RFC does not replace the existing memory model.

The existing persistent evidence layers remain authoritative:

```text
Observation
    ↓
Entity
    ↓
Claim
    ↓
AnswerCapsule
```

Observations remain append-oriented evidence. Claims remain versioned derived beliefs with explicit states such as `CURRENT`, `SUPERSEDED`, `CONTRADICTED`, `UNKNOWN`, and `INVALIDATED`.

The associative graph introduced by this RFC exists **above and alongside** those durable evidence structures.

A memory cell MAY therefore reference:

```text
Observation(s)
Entity(s)
Claim(s)
Relationship(s)
Other Memory Cells
Derived latent state
```

but MUST NOT silently rewrite the evidence from which it was derived.

Likewise, the existing `Specialist` abstraction remains useful:

```rust
trait Specialist {
    fn processor(&self) -> ProcessorRef;
    fn invoke(&self, request: &SpecialistRequest) -> SpecialistOutcome;
}
```

The current architecture already expects deterministic operators, tiny classifiers, specialist models, larger specialists, and ultimately general reasoner escalation.

Under this RFC, a `MemoryCell` may be thought of as a **persistent graph-level instance whose computation is supplied by one of those specialist mechanisms**.

In simplified form:

```text
MemoryCell
    ├── identity
    ├── state
    ├── evidence
    ├── ports
    ├── synapses
    └── processor → Specialist
```

---

## 3. Goals

The architecture defined by this RFC SHOULD permit MNCS Memory to:

1. Represent memory as many small specialized computational units.
2. Activate only a small portion of the complete memory graph for a typical request.
3. Learn relationships without globally retraining the memory system.
4. Preserve explicit evidence and provenance for learned associations.
5. Allow relationships themselves to participate in semantic association.
6. Discover new graph structure from compatible previously unconnected latent directions.
7. Reinforce useful pathways through repeated successful activation.
8. Weaken or retire persistently unhelpful relationships.
9. Maintain explicit uncertainty rather than manufacturing certainty.
10. Allow individual cells to be inspected, replayed, moved, replaced, or invalidated.
11. Permit heterogeneous implementations of micro-models.
12. Allow memory to become increasingly useful as topology develops.
13. Remain compatible with bounded MNCS execution and capability control.
14. Permit eventual distributed execution across MNCS Fabric nodes.

---

## 4. Non-Goals

This RFC does not require:

- simulation of biological neural tissue;
- backpropagation across the complete graph;
- one shared global embedding model;
- a single universal vector dimension;
- replacement of the current claim/evidence store;
- autonomous modification of immutable observations;
- every memory cell to contain a neural network;
- every latent port to terminate at another cell;
- every association to represent factual truth;
- global activation for every query.

This RFC also does not claim that the architecture will outperform conventional retrieval.

Performance and usefulness MUST remain experimentally falsifiable.

---

## 5. Terminology

### 5.1 Memory Cell

A **Memory Cell** is the primary graph node.

A cell represents a bounded semantic or behavioral specialization and contains or references the micro-model responsible for evaluating stimuli relevant to it.

Conceptually:

```text
MemoryCell {
    id
    processor
    state
    evidence
    ports[]
    activation_state
    lifecycle
}
```

A cell SHOULD be narrow enough that its activation and evaluation are significantly cheaper than invoking a general reasoner.

---

## 6. Micro-Model

A **Micro-Model** is the bounded computational mechanism associated with a memory cell.

A micro-model MAY be learned or deterministic.

Examples include:

```text
entity recognizer
relationship recognizer
temporal pattern
preference representation
semantic classifier
numeric trend
tiny neural model
MNCS program
rule machine
domain-specific estimator
```

The micro-model SHOULD be **attuned to its memory**.

That is, its structure or parameters SHOULD reflect the information represented by its cell rather than attempting to provide general intelligence.

---

## 7. Stimulus

A **Stimulus** is typed input presented to one or more memory cells.

Stimuli MAY originate from:

- ingestion;
- queries;
- active neighboring cells;
- consolidation;
- contradiction resolution;
- time;
- external state changes;
- relationship activation.

Example:

```text
Stimulus {
    kind
    semantic_payload
    source
    activation_budget
    provenance
}
```

Cells MUST be permitted to return:

```text
MATCH
NO_MATCH
PARTIAL
UNKNOWN
```

or equivalent typed states.

`UNKNOWN` is a first-class valid outcome.

---

## 8. Activation

**Activation** represents temporary relevance of a cell to the currently propagating stimulus.

Activation is not equivalent to truth.

A possible conceptual activation value is:

```text
0.0 ≤ activation ≤ 1.0
```

but implementations MAY use another bounded representation.

A cell MAY derive activation from:

```text
local_match
× synapse_strength
× confidence
× relevance
× recency
× provenance_quality
× activation_budget
```

This formula is illustrative rather than normative.

---

## 9. Latent Ports

A **Latent Port** is an exposed semantic direction belonging to a memory cell.

The port represents something the cell has learned to distinguish or respond to.

For example:

```text
             ───── plant
            /
fireweed ● ───── Alaska
            \
             ───── harvest
              \
               ─── tea
```

These lines do not necessarily represent explicit stored facts.

They may represent directions of possible semantic relation.

A port MAY be:

```text
BOUND
UNBOUND
CANDIDATE
DORMANT
RETIRED
```

An `UNBOUND` port is valid.

This is important.

The system MUST NOT require every learned latent direction to resolve immediately to another memory cell.

An unbound port means approximately:

> The cell recognizes a meaningful semantic direction for which the memory graph does not yet possess a sufficiently compatible neighbor.

---

## 10. Synapses

A **Synapse** is a persistent connection between graph structures.

The simplest form connects two ports:

```text
Cell A : Port α
        │
        │ Synapse
        │
Cell B : Port β
```

A synapse SHOULD contain at minimum:

```text
Synapse {
    id
    endpoints
    relation_kind
    strength
    confidence
    evidence
    provenance
    activation_count
    useful_activation_count
    created_at
    last_activated_at
    lifecycle_state
}
```

Synaptic strength MUST NOT be treated as factual certainty.

A strongly activated relationship may still be contradicted or poorly evidenced.

Confidence, topology, and truth state remain distinct dimensions.

---

## 11. First-Class Relationship Objects

Relationships MUST be capable of becoming first-class semantic objects.

Instead of reducing:

```text
A ───────── B
```

to merely an adjacency entry, MNCS Memory SHOULD permit:

```text
A ──────── ●R₁ ──────── B
```

where `R₁` possesses its own:

```text
identity
representation
state
evidence
activation
provenance
semantic ports
```

This allows memory to reason not only about objects but about the similarity between relationships.

---

## 12. Relationship-to-Relationship Association

One of the defining features of this RFC is that relationships MAY associate with other relationships.

For example:

```text
fireweed ── grows_in ── Alaska
                │
                │ semantic similarity
                │
blueberry ─ grows_in ── Alaska
```

The association exists between the two `grows_in` relationship instances even if no direct relationship between `fireweed` and `blueberry` has yet been created.

This permits activation to propagate through **relationship space**.

Conceptually:

```text
A ──●R1── B
    │
    │
    ●
    │
    │
C ──●R2── D
```

This architecture permits the graph to discover:

```text
R1 resembles R2
```

independently of:

```text
A resembles C
B resembles D
```

---

## 13. Local Latent Spaces

MNCS Memory MUST NOT require one global latent space.

Each memory cell MAY define its own local latent representation:

```text
Cell A → LA
Cell B → LB
Cell C → LC
```

Connections between cells MAY therefore contain learned or deterministic mappings:

```text
T(A→B)
```

rather than assuming identical coordinates.

This permits heterogeneous micro-models to coexist.

For example:

```text
tiny neural classifier
        │
        │ translator
        ▼
symbolic MNCS cell
        │
        │ relation
        ▼
temporal state machine
```

The system therefore learns correspondence between local representational spaces rather than forcing all memory into a universal embedding.

---

## 14. Latent Compatibility Discovery

An unbound port MAY become connected when another port becomes sufficiently compatible.

Conceptually:

```text
Cell A ───── α →

                    ← β ───── Cell B
```

If:

```text
compatibility(α, β) ≥ connection_threshold
```

the manager MAY propose:

```text
Cell A ─── α ═══ β ─── Cell B
```

Connection creation SHOULD normally require evidence beyond raw similarity.

Possible inputs include:

```text
semantic compatibility
co-activation
temporal correlation
shared provenance
successful prediction
successful retrieval
repeated reasoning utility
explicit relationship evidence
```

A single similarity score SHOULD NOT normally establish a durable high-confidence relationship.

---

## 15. Sparse Activation

The memory graph SHOULD be sparsely activated.

For a graph containing:

```text
N cells
```

a normal request SHOULD attempt to evaluate:

```text
k << N
```

cells.

Activation propagation MUST therefore have a budget.

A budget MAY constrain:

```text
maximum cells
maximum hops
maximum compute
maximum wall time
maximum memory
maximum specialist escalation
maximum relationship expansion
```

Example:

```text
ActivationBudget {
    cells: 64
    hops: 4
    compute_units: 1000
}
```

A cell MUST be permitted to stop propagation when additional activation is unlikely to justify its cost.

---

## 16. Propagation

An activated cell evaluates its stimulus locally.

The cell MAY then emit activation through selected ports.

Conceptually:

```text
stimulus
   │
   ▼
 Cell A
 /  │  \
▼   ▼   ▼
B   C   D
    │
    ▼
    E
```

Propagation MUST NOT imply exhaustive traversal.

Neighbors SHOULD compete for limited activation budget.

The manager SHOULD favor pathways using factors such as:

```text
local relevance
synapse strength
confidence
recency
expected utility
novelty
evidence quality
activation history
cost
```

---

## 17. Plasticity

The graph SHOULD be capable of modifying its topology from experience.

This property is termed **plasticity**.

Plasticity may include:

```text
strengthen synapse
weaken synapse
create candidate synapse
promote candidate synapse
split cell
merge cells
create cell
retire cell
create latent port
retire latent port
create relationship association
```

Plasticity MUST preserve provenance describing why the modification occurred.

---

## 18. Reinforcement

A pathway MAY strengthen when its activation contributes to a useful outcome.

For example:

```text
query
 ↓
A → C → F → AnswerCapsule
```

If this path repeatedly contributes relevant evidence, the appropriate synapses MAY accumulate reinforcement.

Reinforcement SHOULD be based on observable utility rather than mere frequency.

Repeated activation alone does not prove usefulness.

---

## 19. Decay

Relationships MAY weaken when they are:

- unused;
- repeatedly irrelevant;
- superseded;
- contradicted;
- based on stale evidence;
- associated with invalidated claims.

Decay MUST NOT delete immutable evidence.

It changes the graph's willingness to activate the relationship.

A decayed connection SHOULD be recoverable if underlying evidence remains valid.

---

## 20. Consolidation

**Consolidation** converts repeated or sufficiently stable patterns into more durable graph structures.

Example:

```text
A → B → C
A → B → C
A → B → C
A → B → C
```

may result in:

```text
        ┌─────────────┐
        │ new Cell X  │
        │ pattern ABC │
        └─────────────┘
```

or:

```text
A ═══ B ═══ C
```

with strengthened relationships.

Consolidation MAY:

```text
create a new cell
specialize an existing cell
split a broad cell
merge redundant cells
create a relationship abstraction
promote latent associations
```

Consolidation MUST remain replayable from its evidence where practical.

---

## 21. Cell Formation

New memory cells MAY form when the manager observes a stable semantic structure not adequately represented by existing cells.

Formation SHOULD require evidence of one or more of:

```text
repeated activation
semantic novelty
persistent unresolved port
stable co-occurrence
relationship reuse
high retrieval utility
repeated general-reasoner escalation
```

The goal is not maximum node count.

The goal is useful specialization.

---

## 22. Cell Splitting

A cell SHOULD be eligible for splitting when it becomes too semantically broad.

For example:

```text
Cell: "Alaska plants"
```

may eventually specialize into:

```text
fireweed
blueberry
spruce
devil's club
```

if this improves bounded recall and processing.

The original cell MAY remain as a higher-order abstraction.

---

## 23. Cell Merging

Two cells MAY merge when they are demonstrated to represent the same semantic specialization.

Merge decisions MUST respect:

```text
identity
provenance
conflicts
rights
claim state
processor compatibility
```

A merge MUST NOT destroy the evidence histories of either source cell.

---

## 24. Memory Metabolism

The component coordinating this graph is termed the **Memory Manager**.

Conceptually, the manager operates a memory metabolism:

```text
                  stimulus
                     │
                     ▼
                  routing
                     │
                     ▼
              candidate cells
                     │
                     ▼
             local evaluation
                     │
                     ▼
          sparse graph propagation
                     │
            ┌────────┴────────┐
            ▼                 ▼
       useful paths      unresolved novelty
            │                 │
            ▼                 ▼
      reinforcement      consolidation
            │                 │
            └────────┬────────┘
                     ▼
                 topology
                  evolves
```

The manager SHOULD coordinate behavior.

It SHOULD NOT contain all semantic intelligence itself.

---

## 25. Emergent Global Memory

No individual cell is required to model the complete world known by MNCS Memory.

Instead:

```text
global memory
    =
local cell behavior
    +
cell state
    +
relationship objects
    +
graph topology
    +
activation dynamics
```

This RFC treats topology as part of memory itself.

Consequently:

> The memory represented by MNCS is not completely contained in its nodes.

Some memory exists in:

```text
which nodes connect
how they connect
how strongly they connect
which relationship classes resemble one another
how activation propagates
how those structures change through experience
```

---

## 26. Provenance

Every durable graph mutation MUST be attributable.

At minimum, a mutation SHOULD record:

```text
producer
processor version
input evidence
previous state
new state
logical time
reason
confidence
```

A learned association without explainable lineage SHOULD remain lower-confidence or experimental.

Graph state MUST NOT become an opaque substitute for MNCS provenance.

---

## 27. Contradiction

Activation strength MUST NOT erase contradiction.

If one portion of the graph supports:

```text
X → value A
```

and another supports:

```text
X → value B
```

the graph MUST preserve both where underlying evidence requires it.

The current MNCS Memory principle remains:

```text
contradiction != forced choice
```

Ordinary belief resolution MAY therefore produce:

```text
UNKNOWN
```

until sufficient resolution exists.

---

## 28. Determinism and Replay

Where practical, graph mutations SHOULD support deterministic replay.

Given:

```text
initial graph state
+ immutable observations
+ processor identities
+ processor versions
+ ordered mutations
```

MNCS Memory SHOULD be capable of explaining how a current topology was produced.

Learned models may prevent bit-identical reconstruction in all implementations, but the provenance chain MUST remain inspectable.

---

## 29. Distribution

A memory cell MUST be conceptually independent of physical placement.

Future implementations MAY place cells across:

```text
local process
multiple cores
multiple machines
MNCS Fabric workers
persistent cold storage
specialized accelerators
```

A cell identity MUST therefore remain stable independent of location.

Synapses MUST identify logical endpoints rather than depending solely on process-local pointers.

This allows:

```text
Cell A → worker-02
Cell B → worker-03
Cell C → controller
```

without changing the semantic graph.

---

## 30. Migration

Because cells are bounded, the system SHOULD eventually permit an individual cell to be:

```text
serialized
verified
transferred
loaded
activated
retired
replaced
```

independently of the entire graph.

Migration MUST preserve:

```text
identity
provenance
processor requirements
rights
schema version
synapse references
```

---

## 31. Heterogeneous Execution

Cells MAY require different execution mechanisms.

For example:

```text
Cell A → MNCS WASM
Cell B → CPU classifier
Cell C → CUDA/PTX micro-model
Cell D → deterministic Rust fallback
Cell E → remote specialist
```

The semantic graph MUST not depend on one execution backend.

This gives MNCS Memory a natural way to apply pressure to the broader MNCS language and execution ecosystem.

---

## 32. Escalation

A memory cell MAY be unable to resolve a stimulus.

It MUST be permitted to respond:

```text
UNKNOWN
```

or request escalation.

Escalation MAY progress through:

```text
deterministic operator
        ↓
tiny model
        ↓
specialist model
        ↓
larger memory specialist
        ↓
general reasoner
```

This preserves the existing MNCS micro-model replacement ladder.

Escalation SHOULD be bounded and observable.

The general reasoner MUST NOT silently receive the complete memory graph.

---

## 33. Query Behavior

A query does not directly request all relevant nodes.

Instead:

```text
semantic request
      │
      ▼
activation seed
      │
      ▼
associative propagation
      │
      ▼
candidate evidence
      │
      ▼
conflict / confidence / provenance handling
      │
      ▼
AnswerCapsule
```

This preserves the existing MNCS goal that the normal consumer receives compact, administered memory rather than raw vaguely similar records.

---

## 34. Suggested Core Types

The exact implementation is deferred, but the conceptual types are:

```text
MemoryCell
LatentPort
Synapse
Relationship
Activation
Stimulus
ActivationBudget
GraphMutation
ConsolidationEvent
CellProcessor
```

A rough initial representation might resemble:

```rust
struct MemoryCell {
    id: CellId,
    processor: ProcessorRef,
    evidence: Vec<EvidenceRef>,
    ports: Vec<PortId>,
    state: CellState,
}

struct LatentPort {
    id: PortId,
    cell: CellId,
    representation: LatentRepresentation,
    state: PortState,
}

struct Synapse {
    id: SynapseId,
    left: Endpoint,
    right: Endpoint,
    strength: f32,
    confidence: f32,
    evidence: Vec<EvidenceRef>,
    state: SynapseState,
}

enum Endpoint {
    Port(PortId),
    Relationship(RelationshipId),
}
```

This is illustrative.

The RFC defines semantics, not Rust ABI.

---

## 35. Required Invariants

A conforming implementation MUST preserve the following invariants.

### Evidence invariant

Derived topology cannot silently modify immutable source observations.

### Provenance invariant

Durable graph mutations have attributable lineage.

### Uncertainty invariant

The architecture permits explicit `UNKNOWN`.

### Sparse-compute invariant

Graph traversal is bounded.

### Placement invariant

Semantic identity does not depend on physical execution location.

### Relationship invariant

Relationships may exist as first-class objects.

### Local-space invariant

Implementations cannot require every cell to share one universal latent coordinate system.

### Unbound-port invariant

A semantically meaningful port may exist without a current graph neighbor.

---

## 36. Experimental Hypotheses

This RFC intentionally defines falsifiable hypotheses.

### H1 — Sparse associative recall

A graph of bounded micro-models can recover relevant persistent memory while activating substantially less state than exhaustive retrieval.

### H2 — Local specialization

Small specialists can perform portions of memory administration more efficiently than repeated invocation of a general reasoner.

### H3 — Topological learning

Useful memory behavior can improve through graph topology changes without globally retraining all participating models.

### H4 — Relationship association

Associating relationships with relationships improves abstraction and recall for some workloads.

### H5 — Local latent spaces

Useful semantic composition can emerge without requiring every memory cell to share one global embedding space.

### H6 — Emergent memory

System-level recall quality can exceed the capability of individual participating cells because useful behavior is encoded partly in graph topology.

Any of these hypotheses MAY prove false without invalidating the entire MNCS Memory project.

---

## 37. Evaluation

Benchmarks SHOULD compare at minimum:

```text
raw context
simple retrieval
vector retrieval
current MNCS Memory
micro-model graph
```

Evaluation SHOULD measure:

```text
useful recall
irrelevant context
activation count
compute consumed
latency
memory footprint
graph growth
false associations
UNKNOWN correctness
provenance completeness
replayability
general-reasoner escalation rate
```

Particular attention MUST be paid to runaway associative behavior.

A system that finds every concept related to every other concept is not demonstrating useful memory.

---

## 38. Failure Modes

Implementations MUST actively test for:

### Association explosion

Too many ports become connected.

### Hub collapse

Certain generic cells become connected to nearly everything.

### Semantic drift

Repeated reinforcement causes a cell to move away from its original evidence.

### Echo reinforcement

A relationship becomes strong because its own activation repeatedly causes further activation.

### Stale topology

Old relationships continue dominating after underlying evidence changes.

### Hidden contradiction

Strong graph weights suppress conflicting evidence.

### Model obesity

Cells continually grow until they become general-purpose models.

### Manager centralization

The memory manager accumulates enough semantics that cells become passive records again.

### Unbounded propagation

A small stimulus causes graph-wide activation.

These are architectural failures, not merely optimization problems.

---

## 39. Security and Rights

Graph activation MUST remain subject to MNCS capability, rights, and provenance constraints.

The existence of a synapse MUST NOT imply authority to expose its endpoint.

A cell may know that another semantic region exists while lacking rights to inspect or disclose it.

Conceptually:

```text
semantic connectivity != access authority
```

Activation across a protected boundary SHOULD stop, redact, or emit a typed constrained result according to the applicable MNCS policy.

---

## 40. Implementation Strategy

Initial implementation SHOULD deliberately remain small.

### Phase 0 — Specification

Define:

```text
MemoryCell
LatentPort
Synapse
Activation
GraphMutation
```

without learned topology.

### Phase 1 — Deterministic Graph

Use deterministic specialists only.

Demonstrate:

```text
cell creation
port creation
explicit synapse creation
bounded activation
provenance
replay
```

### Phase 2 — Reinforcement and Decay

Add measurable:

```text
activation history
utility
strengthening
weakening
retirement
```

### Phase 3 — Latent Compatibility

Permit unbound ports and candidate port discovery.

Do not automatically promote similarity into trusted memory.

### Phase 4 — Relationship Space

Represent synapses/relationships as first-class semantic objects and permit relationship-to-relationship association.

### Phase 5 — Consolidation

Allow graph evidence to propose:

```text
new cells
splits
merges
relationship abstractions
```

### Phase 6 — Learned Micro-Models

Introduce tiny learned specialists where benchmarks justify them.

### Phase 7 — Distributed Cells

Permit cell placement and activation through MNCS Fabric.

---

## 41. Architectural Principle

The architecture introduced by this RFC can be summarized as:

```text
experience
    │
    ▼
durable evidence
    │
    ▼
specialized memory cells
    │
    ▼
local latent structure
    │
    ▼
learned relationships
    │
    ▼
sparse associative activation
    │
    ▼
consolidation / decay / reinforcement
    │
    ▼
evolving topology
    │
    ▼
emergent persistent memory
```

Or more compactly:

> **MNCS Memory stores knowledge both in specialized computational units and in the evolving topology between them.**

---

## 42. Open Questions

The following intentionally remain unresolved:

1. What is the minimum useful memory-cell primitive?
2. When should one cell represent an entity versus a relationship versus a broader pattern?
3. How should local latent representations be compared across heterogeneous micro-models?
4. What evidence threshold promotes an unbound port into a synapse?
5. How quickly should weak synapses decay?
6. Should relationship objects themselves ever become full memory cells?
7. How should topology reinforcement distinguish causation from repeated co-activation?
8. What mechanism prevents high-degree semantic hubs?
9. When should cells split?
10. When should cells merge?
11. How should processor replacement affect existing latent ports?
12. Can cells be transferred between machines without loss of behavioral equivalence?
13. What subset of graph evolution can be expressed directly in `mncs-language`?
14. Can the manager itself eventually be expressed primarily as MNCS programs rather than host-language orchestration?
15. How much intelligence emerges from topology before learned cell models become necessary?

---

## 43. Acceptance Criteria

RFC 0001 should be considered experimentally implemented when `mncs-memory` can demonstrate all of the following in automated tests:

```text
[ ] Multiple independently executable memory cells
[ ] At least one unbound latent port
[ ] Port-to-port synapse formation
[ ] Bounded sparse activation
[ ] Activation across multiple cells
[ ] Explicit UNKNOWN propagation
[ ] Synapse reinforcement
[ ] Synapse decay
[ ] Full provenance for graph mutations
[ ] Replay of deterministic topology evolution
[ ] Relationship represented as a first-class object
[ ] Relationship-to-relationship association
[ ] Consolidation producing or proposing a new cell
[ ] Evidence survives cell/synapse invalidation
[ ] Graph query produces an AnswerCapsule
[ ] Benchmark against simple retrieval
[ ] No unrestricted exposure of the complete graph to an escalated reasoner
```

Distribution, heterogeneous learned models, and automatic topology optimization SHOULD remain later milestones rather than blockers for initial RFC conformance.

---

## 44. Final Principle

The blue nodes in the conceptual model are not merely records and are not merely neurons.

They are **small memories capable of behavior**.

The lines connecting them are not merely database edges and are not merely numeric weights.

They are **persistent learned semantic structure**.

The combination produces a system in which:

```text
no cell contains the whole memory

no global latent space is required

not every learned direction must already have a neighbor

relationships can themselves relate

only locally relevant memory needs to wake up

and the structure of the network is itself part of what MNCS remembers
```

That is the architecture proposed by RFC 0001.
