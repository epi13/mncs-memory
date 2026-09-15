# MNCS Memory RFCs

This directory contains architectural RFCs for `mncs-memory`.

## RFC index

| RFC | Title | Status | Relationship |
|---|---|---|---|
| [0001](./0001-associative-micro-model-memory-graph.md) | Associative Micro-Model Memory Graph | Draft | Base graph architecture: memory cells, latent ports, persistent synapses, relationship objects, sparse activation, reinforcement, decay, consolidation, and evolving topology. |
| [0002](./0002-multi-timescale-synaptic-dynamics.md) | Multi-Timescale Synaptic Dynamics | Draft | Extends RFC 0001 with fast transient effective connectivity, adaptive state, propagation-conditioned routing, eligibility traces, refractory behavior, and explicit separation between temporary routing state and durable learning. |

## Architectural relationship

RFC 0001 defines the persistent anatomy of the associative memory graph.

RFC 0002 defines a dynamic execution layer over that anatomy:

```text
RFC 0001
persistent memory cells + persistent semantic topology
                    │
                    ▼
RFC 0002
adaptive state + transient state + activation context
                    │
                    ▼
effective topology at time t
                    │
                    ▼
propagation may alter later propagation
```

The durable evidence model remains authoritative in both RFCs. Temporary or adaptive routing state does not silently rewrite observations, claims, confidence, or persistent topology.

A useful shorthand is:

> **RFC 0001 provides the anatomy; RFC 0002 provides the fast-changing computational physiology.**
