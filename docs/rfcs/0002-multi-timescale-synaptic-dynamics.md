# MNCS Memory RFC 0002: Multi-Timescale Synaptic Dynamics

**Status:** Draft  
**Project:** `mncs-memory`  
**RFC:** 0002  
**Title:** Multi-Timescale Synaptic Dynamics  
**Authors:** MNCS Project  
**Created:** 2026-09-15  
**Extends:** RFC 0001 — Associative Micro-Model Memory Graph

## Abstract

RFC 0001 defines MNCS Memory as a dynamic graph of bounded computational memory cells connected by first-class synapses. This RFC extends that architecture by distinguishing **durable synaptic structure** from **rapidly changing effective connectivity**.

A synapse is therefore not treated as a single persistent scalar weight. Instead, its current computational influence MAY be produced from multiple state layers operating at different timescales:

```text
persistent / structural state
        +
adaptive / consolidating state
        +
fast / transient state
        +
current activation context
        ↓
effective synaptic state
```

The central hypothesis of this RFC is:

> **Useful memory behavior can emerge not only from what cells and connections exist, but from rapid, reversible changes in how strongly those connections participate in computation while information is propagating.**

Under this model, activation does not merely traverse a graph. Activation MAY temporarily change the graph's effective routing state, which then changes the path available to subsequent activation in the same or nearby activation epoch.

This makes MNCS Memory a bounded dynamical system rather than only a persistent associative graph.

---

## 1. Motivation

RFC 0001 already separates memory cells, latent ports, synapses, activation, reinforcement, decay, and consolidation. It also treats topology as part of memory itself.

However, a simple implementation can still collapse synaptic behavior into something approximately equivalent to:

```text
activation(A → B)
    =
activation(A)
× persistent_synapse_strength(A, B)
× other_factors
```

where `persistent_synapse_strength` changes only when reinforcement, decay, or consolidation occurs.

That is useful, but it leaves out another important form of state:

```text
what this connection means structurally
    !=
how strongly this connection should participate right now
```

A relationship may be stable and well evidenced while being temporarily irrelevant. A moderately strong relationship may become highly relevant under a specific local context. A recently traversed path may need temporary facilitation, inhibition, refractory behavior, or competition without changing durable memory.

This RFC therefore separates **learning** from **moment-to-moment effective computation**.

---

## 2. Biological Inspiration and Scope

Recent insect visual-system research provides a useful architectural analogy. Mansour et al. reported stimulus-dependent **synaptic high-frequency jumping** in the housefly *Musca domestica*, where photoreceptor-to-large-monopolar-cell transmission dynamically shifts toward higher frequencies during rapid saccade-like stimulation.

Reference:

> Mansour, N., Takalo, J., Kemppainen, J. et al. “Synaptic high-frequency jumping synchronises vision to high-speed behaviour.” *Nature Communications* 17, 3863 (2026). DOI: `10.1038/s41467-026-72509-2`.

This RFC does **not** claim that MNCS Memory should reproduce fly neurobiology, nor that the biological mechanism is literally equivalent to artificial neural-network weight updates.

The architectural lesson is narrower:

> A physically stable connection can exhibit strongly context-dependent effective behavior on a timescale much faster than durable structural learning.

MNCS Memory adopts that principle as an engineering hypothesis.

The distinction is important:

```text
biological observation
    → architectural inspiration
    ≠ biological simulation requirement
```

---

## 3. Relationship to RFC 0001

RFC 0001 remains the base graph architecture.

It defines:

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

RFC 0002 adds explicit semantics for **time-varying effective synaptic state**.

A synapse remains a first-class persistent graph object. RFC 0002 does not replace that object with an ephemeral weight.

Instead:

```text
persistent synapse
       │
       ├── durable semantic structure
       ├── evidence / confidence / provenance
       ├── long-term strength
       ├── adaptive state
       └── transient state
                │
                ▼
         effective behavior
```

Durable evidence remains authoritative. Temporary effective state MUST NOT silently rewrite observations, claims, or durable topology.

---

## 4. Goals

The architecture defined by this RFC SHOULD permit MNCS Memory to:

1. Distinguish persistent connection strength from instantaneous effective connection strength.
2. Change routing behavior rapidly without treating every change as durable learning.
3. Allow propagation itself to alter subsequent propagation within bounded activation epochs.
4. Represent multiple plasticity timescales explicitly.
5. Permit transient facilitation, inhibition, refractoriness, competition, and context gating.
6. Consolidate useful transient dynamics into slower state only when justified by evidence and utility.
7. Avoid unnecessary churn in durable graph topology.
8. Preserve explicit uncertainty, provenance, replayability, and bounded computation.
9. Support heterogeneous micro-models as producers and consumers of modulation signals.
10. Remain compatible with distributed execution and MNCS capability constraints.
11. Keep effective routing state inspectable rather than hiding it inside an opaque global controller.
12. Make the hypothesis experimentally falsifiable against static-synapse baselines.

---

## 5. Non-Goals

This RFC does not require:

- biological realism;
- backpropagation through the complete graph;
- continuous floating-point weights;
- neural-network implementations of memory cells;
- global synchronous updates;
- permanent learning from every activation;
- one universal modulation rule;
- one universal timescale;
- treating activation strength as truth;
- treating transient state as evidence;
- removing RFC 0001 reinforcement, decay, consolidation, or topology growth.

This RFC also does not assume that rapidly changing effective connectivity will improve all workloads.

---

## 6. Terminology

### 6.1 Persistent Synaptic State

**Persistent synaptic state** describes durable graph structure that survives ordinary activation epochs.

It MAY include:

```text
relation kind
long-term strength
confidence
evidence
provenance
lifecycle state
historical utility
structural translation
rights constraints
```

Persistent state changes are graph mutations and MUST obey RFC 0001 provenance requirements.

---

### 6.2 Adaptive Synaptic State

**Adaptive synaptic state** is slower-changing state that is more persistent than an activation epoch but less authoritative than durable semantic structure.

It MAY represent:

```text
recent utility trend
recent co-activation
eligibility for consolidation
short-lived learned preference
recent failure history
context-specific adaptation
stability estimate
```

Adaptive state MAY survive across queries or sessions according to policy.

Adaptive state MUST NOT be confused with factual confidence.

---

### 6.3 Transient Synaptic State

**Transient synaptic state** is rapid, reversible state associated with current or recent activation.

It MAY include:

```text
facilitation
inhibition
refractory state
local gain
priority
competition score
recent activation trace
phase / logical-step state
context gate
prediction-error response
```

Transient state SHOULD normally decay, reset, or otherwise lose influence without requiring a durable graph mutation.

---

### 6.4 Effective Synaptic State

**Effective synaptic state** is the connection behavior used for a particular propagation decision.

Conceptually:

```text
W_effective(t)
    =
F(
  W_persistent,
  W_adaptive,
  W_transient(t),
  local_cell_state,
  stimulus,
  activation_context,
  budget,
  rights
)
```

`F` is intentionally not standardized by this RFC.

Implementations MAY use deterministic rules, bounded learned modulators, state machines, micro-model outputs, or other inspectable mechanisms.

---

### 6.5 Activation Epoch

An **activation epoch** is a bounded interval during which related propagation events share transient computational context.

An epoch MAY correspond to:

```text
one query
one ingestion event
one consolidation pass
one contradiction-resolution pass
one bounded propagation cascade
```

An implementation MUST define when transient state is created, updated, decayed, retained, or discarded.

---

## 7. Multi-Timescale State Model

A conceptual three-timescale model is:

```text
W_L       long-term / structural state
W_M       medium-term / adaptive state
W_F(t)    fast / transient state
```

with:

```text
W_effective(t) = F(W_L, W_M, W_F(t), S_t)
```

where `S_t` is the relevant local activation context.

The three layers are not required to use the same representation.

For example:

```text
W_L  → bounded persistent score + evidence
W_M  → typed adaptive state machine
W_F  → temporary facilitation / inhibition flags
```

The architecture therefore does not require that every timescale be represented as a scalar numeric weight.

---

## 8. Synapse Representation

RFC 0001's conceptual `Synapse` type SHOULD be extended approximately as follows:

```text
Synapse {
    id
    endpoints
    relation_kind

    persistent_strength
    confidence
    evidence
    provenance

    adaptive_state
    transient_policy

    activation_count
    useful_activation_count
    created_at
    last_activated_at
    lifecycle_state
}
```

Transient state itself MAY live outside the durable synapse record:

```text
TransientSynapseState {
    synapse_id
    epoch_id
    facilitation
    inhibition
    refractory
    local_gain
    activation_trace
    expires_at_or_step
    producer
}
```

This separation is preferred when transient activity would otherwise cause excessive durable writes.

---

## 9. Effective Strength Is Not Truth

The following MUST remain separate:

```text
factual confidence
persistent relationship strength
adaptive utility
transient effective strength
activation
```

A connection MAY have:

```text
confidence = high
persistent_strength = high
effective_strength = low
```

because it is currently irrelevant.

Another connection MAY have:

```text
confidence = moderate
persistent_strength = moderate
effective_strength = temporarily high
```

because the current stimulus strongly gates that relationship.

Neither case changes the truth state of the underlying evidence.

---

## 10. Propagation-Conditioned Routing

Under RFC 0002, propagation MAY change effective routing before the activation cascade ends.

Conceptually:

```text
initial state

A ──0.6── B
│
└──0.5── C

stimulus activates A
        ↓
A locally evaluates context
        ↓
A→B transiently facilitated
A→C transiently inhibited
        ↓
subsequent propagation sees

A ──0.9── B
│
└──0.2── C
```

The durable graph has not necessarily changed.

Later in the same epoch, activation of `B` MAY alter another connection:

```text
B activation
    ↓
B→D transient state changes
    ↓
next propagation step uses new effective state
```

Thus:

> **The path taken through memory may influence the path that becomes available next.**

This is a defining property of RFC 0002.

---

## 11. Order and History Dependence

Because propagation can modify transient state, activation order MAY matter.

In general:

```text
A then B
```

need not produce the same effective graph state as:

```text
B then A
```

This history dependence is permitted, but MUST remain bounded and inspectable.

Implementations SHOULD expose enough trace information to explain material routing changes.

Deterministic implementations SHOULD preserve event ordering under replay.

---

## 12. Fast Facilitation and Inhibition

A synapse MAY become temporarily easier or harder to traverse.

Possible facilitation signals include:

```text
strong local match
recent useful activation
compatible neighboring activation
query context
prediction success
novelty relevance
shared active relationship pattern
```

Possible inhibition signals include:

```text
recent failed activation
contradiction boundary
budget pressure
redundancy
high fan-out competition
refractory state
rights constraint
known irrelevant context
```

Facilitation and inhibition SHOULD be typed where practical rather than compressed into an unexplained scalar.

---

## 13. Refractory and Cooldown Behavior

Repeated activation can otherwise create self-reinforcing loops.

A synapse, port, relationship, or cell MAY therefore enter a temporary refractory or cooldown state after activation.

Conceptually:

```text
ACTIVE
  ↓
REFRACTORY
  ↓
AVAILABLE
```

Refractory state MAY reduce:

```text
immediate re-entry
oscillation
echo reinforcement
hub domination
budget capture
```

This is a computational stability mechanism, not a claim of biological equivalence.

---

## 14. Local Competition

Neighboring pathways SHOULD be permitted to compete for limited activation budget.

Instead of independently evaluating every outgoing edge:

```text
A → B
A → C
A → D
A → E
```

an implementation MAY compute a bounded local competition:

```text
candidate effective utilities
        ↓
normalize / rank / gate
        ↓
activate only useful subset
```

Competition MAY itself alter transient state.

This gives the memory graph a local routing mechanism without requiring a global attention controller.

---

## 15. Micro-Models as Modulators

A memory cell's micro-model MAY emit both semantic output and bounded modulation output.

Conceptually:

```text
CellOutcome {
    semantic_result
    confidence
    outgoing_modulation[]
    transient_cell_state
    propagation_request
}
```

A micro-model MAY therefore say approximately:

```text
MATCH
and temporarily prefer port α
and suppress port β
```

or:

```text
UNKNOWN
but raise novelty on unbound port γ
```

The modulation output MUST remain subject to activation budget, rights, provenance policy, and stability constraints.

A cell MUST NOT gain unrestricted authority to rewrite arbitrary graph state.

---

## 16. Transient State Is Not Durable Learning

A core invariant of this RFC is:

```text
fast state change != learned memory
```

Transient facilitation MUST NOT automatically become persistent reinforcement.

Promotion to slower state SHOULD require additional evidence such as:

```text
repeated utility across epochs
successful prediction
successful retrieval
stable co-activation
independent evidence
low contradiction
reproducibility
explicit consolidation policy
```

This protects MNCS Memory from converting momentary context into durable semantic bias.

---

## 17. Eligibility and Consolidation Bridge

Transient or adaptive state MAY record **eligibility** for slower learning.

Conceptually:

```text
useful transient event
        ↓
eligibility trace
        ↓
repeated independent support
        ↓
adaptive state
        ↓
consolidation review
        ↓
persistent mutation
```

An eligibility trace is not itself evidence of truth.

It is evidence that a pathway may deserve evaluation for durable change.

This separates:

```text
what worked right now
```

from:

```text
what the memory system should retain structurally
```

---

## 18. Decay Across Timescales

Different state layers SHOULD decay independently.

For example:

```text
transient state
    → milliseconds / logical steps / one epoch

adaptive state
    → minutes / sessions / bounded number of epochs

persistent state
    → explicit decay / reinforcement / invalidation policy
```

These are conceptual timescales. MNCS implementations MAY use logical time rather than wall-clock time.

A useful system SHOULD be able to forget temporary routing context without forgetting durable evidence.

---

## 19. State Scope

Transient state MAY be scoped to:

```text
one synapse
one relationship
one cell
one local neighborhood
one activation epoch
one query context
one actor / capability context
```

Global transient state SHOULD be avoided unless experimentally justified.

The default architecture SHOULD favor local state because local state is easier to bound, inspect, distribute, replay, and secure.

---

## 20. Rights and Capability Boundaries

Dynamic effective connectivity MUST NOT bypass MNCS rights.

A transient increase in effective strength cannot create authority.

Formally:

```text
effective_relevance != permission
```

Rights checks MUST remain capable of stopping propagation even when local dynamics strongly favor a protected edge.

Transient state derived from one capability context SHOULD NOT leak into another context unless explicitly permitted.

---

## 21. Stability Requirements

Fast-changing effective connectivity creates new failure modes.

Implementations MUST bound dynamic amplification.

Possible controls include:

```text
maximum effective gain
minimum inhibition floor
normalization
local activation conservation
refractory periods
maximum transient lifetime
maximum modulation fan-out
oscillation detection
cycle budgets
per-epoch mutation budgets
transient-state memory budgets
```

A system MUST NOT allow transient state to create unbounded graph-wide activation.

---

## 22. Anti-Echo Requirement

RFC 0001 identifies echo reinforcement as a failure mode. RFC 0002 makes that risk more immediate because activation can influence near-term routing.

Implementations SHOULD distinguish:

```text
independent support
```

from:

```text
support caused only by the system's own recent activation
```

Self-generated activation MUST NOT be repeatedly counted as independent evidence.

Transient facilitation SHOULD decay or become refractory when it is sustained only by a closed activation loop.

---

## 23. Provenance and Trace

Durable graph mutations retain RFC 0001's full provenance requirement.

Transient state does not necessarily require a durable record for every micro-update, but material routing decisions SHOULD remain explainable.

An activation trace MAY record:

```text
epoch_id
event_order
source cell
candidate synapses
persistent state summary
adaptive state summary
transient modulation
effective decision
budget consumed
rights result
processor identity/version
```

Implementations MAY compact this trace after the epoch.

The goal is to avoid two extremes:

```text
record every internal scalar forever
```

and:

```text
make dynamic routing completely opaque
```

---

## 24. Replay

Deterministic dynamic routing SHOULD be replayable from:

```text
initial durable graph state
+ initial adaptive state
+ ordered stimuli
+ processor identities / versions
+ modulation rules
+ logical event order
+ deterministic seeds where required
```

Replay MAY reconstruct transient state rather than storing every transient state directly.

For learned stochastic modulators, exact bitwise replay is not always required, but provenance and bounded behavioral equivalence SHOULD remain testable.

---

## 25. Distribution

Distributed cells introduce latency and ordering questions.

RFC 0002 therefore distinguishes semantic time from transport time.

Remote message arrival order MUST NOT silently define memory semantics where deterministic ordering is required.

Distributed implementations MAY use:

```text
logical epochs
logical steps
causal event identifiers
bounded local clocks
ordered neighborhood updates
snapshot + delta propagation
```

The system SHOULD prefer local transient dynamics when possible and avoid requiring global synchronization for every effective-weight update.

---

## 26. Query Behavior

A query under RFC 0002 conceptually becomes:

```text
semantic request
      │
      ▼
activation seed
      │
      ▼
local cell evaluation
      │
      ▼
transient edge modulation
      │
      ▼
propagation using effective state
      │
      ├──────────────┐
      ▼              │
next cells           │
      │              │
      ▼              │
more local modulation│
      │              │
      └──────────────┘
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

The query therefore changes a bounded temporary computational state while it is being answered.

---

## 27. Suggested Core Types

RFC 0002 introduces the following conceptual types:

```text
SynapsePersistentState
SynapseAdaptiveState
TransientSynapseState
EffectiveSynapseState
ActivationEpoch
ModulationSignal
EligibilityTrace
DynamicRoutingTrace
```

An illustrative representation is:

```rust
struct SynapsePersistentState {
    strength: f32,
    confidence: f32,
    evidence: Vec<EvidenceRef>,
    provenance: Provenance,
}

struct SynapseAdaptiveState {
    utility: f32,
    eligibility: f32,
    recent_failures: u32,
}

struct TransientSynapseState {
    epoch: EpochId,
    facilitation: f32,
    inhibition: f32,
    refractory: bool,
    activation_trace: f32,
}

struct EffectiveSynapseState {
    permitted: bool,
    propagation_score: f32,
    reason: EffectiveStateReason,
}
```

This is illustrative only.

The RFC defines semantic separation, not a Rust ABI or mandatory numeric representation.

---

## 28. Required Invariants

A conforming implementation MUST preserve the following invariants.

### Evidence invariant

Transient or adaptive state cannot silently rewrite immutable source evidence.

### Truth-separation invariant

Effective connection strength is not factual confidence.

### Timescale invariant

The implementation exposes a semantic distinction between rapid effective state and durable synaptic state.

### Reversibility invariant

Fast state MUST have a defined decay, reset, expiry, or bounded carry-forward policy.

### Consolidation invariant

Transient changes do not automatically become durable learning.

### Boundedness invariant

Dynamic modulation remains subject to activation, compute, state, and fan-out budgets.

### Rights invariant

Dynamic routing cannot bypass capability constraints.

### Inspectability invariant

Material dynamic-routing decisions can be explained or replayed to a useful degree.

### Anti-echo invariant

Self-generated activation is not repeatedly promoted as independent semantic evidence.

---

## 29. Experimental Hypotheses

### H1 — Context-dependent effective routing

A graph with transient effective synaptic state can route context-sensitive queries more selectively than the same graph using only static persistent strengths.

### H2 — Lower durable churn

Fast reversible state can improve adaptation while requiring fewer persistent graph mutations.

### H3 — Multi-timescale advantage

Separating fast, adaptive, and persistent state improves stability or utility compared with a single-timescale reinforcement model.

### H4 — Propagation-conditioned computation

Allowing current propagation to alter subsequent routing improves at least some sequential, temporal, relational, or ambiguous workloads.

### H5 — Local modulation sufficiency

Useful dynamic behavior can emerge primarily from local cell and synapse state without a global attention controller.

### H6 — Consolidation quality

Eligibility traces derived from repeated useful transient dynamics can improve durable topology without promoting one-off context into persistent bias.

### H7 — Sparse dynamic computation

Transient routing can reduce the number of cells evaluated while maintaining or improving answer quality.

Any hypothesis MAY fail without invalidating RFC 0001.

---

## 30. Evaluation

Benchmarks SHOULD compare at minimum:

```text
static RFC 0001 graph
RFC 0001 + persistent reinforcement only
RFC 0002 fast-state routing
RFC 0002 multi-timescale routing
simple retrieval baseline
current MNCS Memory baseline
```

Evaluation SHOULD measure:

```text
useful recall
precision
irrelevant activation
cells activated
synapses evaluated
latency
compute consumed
transient state size
persistent mutation count
oscillation rate
loop suppression
route stability
context sensitivity
UNKNOWN correctness
provenance completeness
replay quality
consolidation error
rights violations
```

Particular attention SHOULD be paid to whether dynamic routing merely adds complexity without measurable benefit.

---

## 31. Failure Modes

Implementations MUST test for at least the following.

### Runaway facilitation

A useful path amplifies itself until it dominates unrelated requests.

### Oscillation

Transient activation repeatedly flips between competing pathways.

### Context poisoning

One activation epoch leaves transient state that improperly biases unrelated later work.

### Hidden durable learning

Transient implementation details accidentally persist and become de facto unproven memory.

### Dynamic hub collapse

Generic cells receive temporary gain so often that they dominate most propagation.

### Refractory starvation

Cooldown rules suppress pathways that remain legitimately useful.

### Order fragility

Small irrelevant changes in event ordering produce unstable semantic outputs.

### State explosion

Transient state costs more memory or compute than the graph it is intended to improve.

### Replay opacity

Dynamic behavior cannot be explained after the fact.

### Rights bleed

Transient state created under one capability context influences another unauthorized context.

These are architectural failures, not merely tuning problems.

---

## 32. Implementation Strategy

### Phase 0 — Specification and Static Baseline

Preserve RFC 0001 behavior and define:

```text
ActivationEpoch
TransientSynapseState
EffectiveSynapseState
ModulationSignal
```

Create deterministic baseline tests with no dynamic modulation.

### Phase 1 — Deterministic Fast Modulation

Add bounded deterministic rules for:

```text
facilitation
inhibition
refractory behavior
local competition
expiry / reset
```

No learned modulator is required.

### Phase 2 — Adaptive State

Add medium-timescale state for:

```text
recent utility
eligibility traces
recent failures
bounded cross-epoch adaptation
```

Keep this state distinct from persistent graph semantics.

### Phase 3 — Consolidation Bridge

Permit repeated useful adaptive patterns to propose RFC 0001 graph mutations.

Promotion MUST remain evidence-aware and replayable.

### Phase 4 — Micro-Model Modulators

Allow bounded cell specialists to emit typed modulation signals.

Benchmark learned modulators against deterministic rules.

### Phase 5 — Dynamic Relationship Space

Permit first-class relationship objects to carry their own transient state and influence relationship-to-relationship propagation.

### Phase 6 — Distributed Dynamic Routing

Extend activation epochs and causal ordering across MNCS Fabric while minimizing global synchronization.

---

## 33. Acceptance Criteria

RFC 0002 should be considered experimentally implemented when automated tests demonstrate all of the following:

```text
[ ] Persistent and transient synaptic state are represented separately
[ ] Effective connection state can differ from persistent strength
[ ] Fast state can change during an activation epoch
[ ] A propagation event can alter a later propagation decision in the same epoch
[ ] Fast facilitation is bounded
[ ] Fast inhibition is bounded
[ ] Refractory or equivalent anti-loop behavior exists
[ ] Transient state has an explicit expiry / reset policy
[ ] One-off transient state does not automatically mutate durable graph state
[ ] Eligibility can be recorded without asserting truth
[ ] Repeated useful dynamics can propose slower consolidation
[ ] Rights checks dominate effective routing
[ ] Dynamic routing remains within activation budgets
[ ] Material routing decisions are traceable
[ ] Deterministic fast-state behavior can be replayed
[ ] Static RFC 0001 and RFC 0002 modes can be benchmarked against each other
[ ] At least one benchmark shows whether dynamic routing helps or fails to help
[ ] Echo / oscillation / state-leak failure tests exist
```

Learned modulators and distributed dynamic routing SHOULD remain later milestones rather than blockers for initial RFC conformance.

---

## 34. Open Questions

1. What is the smallest useful transient synaptic state representation?
2. Should fast state live on synapses, ports, relationship objects, cells, or some combination?
3. How many timescales are actually useful in practice?
4. Should adaptive state survive process restart by default?
5. Which transient signals should be numeric versus typed symbolic states?
6. What is the correct boundary between local micro-model modulation and manager-level routing policy?
7. How should dynamic routing interact with `UNKNOWN`?
8. Can local competition replace some centralized ranking currently performed by the manager?
9. What anti-oscillation mechanism provides the best stability without suppressing useful recurrence?
10. How should transient state behave when a durable claim becomes contradicted or invalidated mid-epoch?
11. How much routing history must be retained for useful replay?
12. Can transient relationship-to-relationship dynamics improve abstraction?
13. What deterministic MNCS-language primitives are needed to express fast state transitions directly?
14. Can `mncs-test` define first-class dynamic-routing and multi-timescale-memory checks?
15. Can `mncs-debug` visualize effective topology separately from persistent topology?
16. What pressures does multi-timescale state place on MNCS Fabric causal ordering and distributed execution?
17. Can useful emergent behavior arise from purely deterministic local modulation before any learned fast-weight mechanism is introduced?

---

## 35. Architectural Principle

RFC 0001 establishes:

> **MNCS Memory stores knowledge both in specialized computational units and in the evolving topology between them.**

RFC 0002 extends that principle:

> **The computational meaning of that topology may change rapidly with context even when the durable topology itself does not change.**

The complete conceptual system therefore becomes:

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
persistent semantic topology
    │
    ├───────────────┐
    ▼               │
activation          │
    │               │
    ▼               │
fast local state    │
    │               │
    ▼               │
effective topology  │
    │               │
    ▼               │
next propagation ───┘
    │
    ▼
eligibility / adaptive state
    │
    ▼
consolidation when justified
    │
    ▼
evolving persistent topology
```

Or more compactly:

```text
persistent memory
      +
rapid contextual physiology
      =
dynamic associative computation
```

---

## 36. Final Principle

The lines connecting memory cells are not only stored relationships.

They are also potential **stateful computational channels**.

A connection can remain semantically stable while its immediate computational influence changes.

Therefore the active MNCS Memory graph at time `t` is not fully described by its durable nodes and edges alone:

```text
ActiveMemory(t)
    =
PersistentGraph
    + AdaptiveState
    + TransientState(t)
    + CellState(t)
    + ActivationContext(t)
```

The important consequence is:

> **Information does not merely propagate through MNCS Memory. Its propagation may temporarily reshape the effective substrate through which the next information propagates.**

That is the architecture proposed by RFC 0002.
