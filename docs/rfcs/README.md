# MNCS Memory RFCs

This directory contains architectural RFCs for `mncs-memory`.

## RFC index

| RFC | Title | Status | Relationship |
|---|---|---|---|
| [0001](./0001-associative-micro-model-memory-graph.md) | Associative Micro-Model Memory Graph | Draft | Base graph architecture plus the micro-model execution contract: typed probabilistic decisions, cell-internal slow/fast/transient state, bounded online adaptation, calibration, consolidation, and vectorizable logical cells. |
| [0002](./0002-multi-timescale-synaptic-dynamics.md) | Multi-Timescale Synaptic Dynamics | Draft | Extends RFC 0001 specifically at the synapse layer with fast transient effective connectivity, adaptive synaptic state, propagation-conditioned routing, eligibility traces, refractory behavior, and separation between temporary routing state and durable synaptic learning. |

## Architectural relationship

RFC 0001 defines the persistent anatomy of the associative memory graph **and** the execution/adaptation contract inside a memory cell.

RFC 0002 defines a dynamic execution layer specifically for the synapses connecting that anatomy:

```text
RFC 0001
cells + topology + cell-internal adaptive state
                    │
                    ▼
RFC 0002
synaptic adaptive state + synaptic transient state + activation context
                    │
                    ▼
effective topology at time t
                    │
                    ▼
propagation may alter later propagation
```

The durable evidence model remains authoritative in both RFCs. Temporary or adaptive routing state does not silently rewrite observations, claims, confidence, or persistent topology.

A useful shorthand is:

> **RFC 0001 provides the anatomy and cell-local adaptive machinery; RFC 0002 provides the fast-changing physiology of the connections between cells.**
