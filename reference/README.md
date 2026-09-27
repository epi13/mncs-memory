# Reference oracle (non-canonical)

This tree is the historical Rust implementation of mncs-memory,
preserved as an **executable oracle**. It is not the canonical
implementation and is not on any production path:

- canonical semantics live in `../native/mncs/memory/` (MNCS);
- the thin host bridge is `../tools/memory/`;
- conformance is gated by `../scripts/mncs-project-check.py`, which
  never invokes this tree.

The pinned pre-modern toolchain dependency
(`mncs-language` rev `8447dad`) and the 0.6-era policy sources in
`../language/` are retained here solely so this oracle still builds
and reproduces its documented behavior:

```bash
cd reference
cargo test --offline
cargo run --offline --bin mncs-memory-benchmark -- ../language
```

Keep while parity questions remain. Behavioral expectations derived
from this oracle are captured in `../tests/test_memory.py`; do not
extend this tree with new semantics.
