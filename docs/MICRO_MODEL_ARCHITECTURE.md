# Micro-model architecture

The provider contract is:

```rust
trait Specialist {
    fn processor(&self) -> ProcessorRef;
    fn invoke(&self, request: &SpecialistRequest) -> SpecialistOutcome;
}
```

The reference implementation is deterministic and currently covers worthiness,
alias/entity resolution, relationship classification, query intent routing,
and lexical relevance feature extraction.

The contract records role, provider identity, provider version, input schema,
input digest, bounded input/output units, confidence, and evidence. `UNKNOWN`
is a valid outcome. The intentionally faulty entity resolver is a test double,
not an attempted model.

The planned replacement ladder is deterministic operator, tiny classifier,
specialist model, larger memory specialist, and finally the general reasoner.
The MNCS escalation module decides when the next layer is warranted; the host
records the request and bounded context exposed. No current implementation
silently exposes the whole store to a larger model.
