# Pressure ledger (memory workload)

Genuine gaps found while building the canonical native implementation.
`mncs-language` and `mncs-compiler` are read-only; active-campaign
repositories were inspected, never modified.

## MEM-P1: no text type in MNCS (non-blocking, by design)

Memory content (subjects, topics, values, goals) is text. The language
has no string type, so native code decides over precomputed match
flags, counts, and codes while the host owns normalization, equality,
containment, tokenization, and rendering at the effect boundary. This
is the documented split, not a workaround — but any future native
text processing (tokenizers, matchers) belongs to a shared owner, not
to Memory-private string code.

## MEM-P2: recall width is a fixed bound (non-blocking)

Native selection runs over at most 64 ranked candidates
(`RECALL_WIDTH`; wider inputs are refused explicitly, never silently
truncated). Corpora filter before ranking; widening the window needs
dynamic collections, not bigger literals.

## MEM-P3: per-candidate native calls (non-blocking)

Recall scores each candidate with one `mncs call` (~4s warm). Fine for
corpus scale (23 candidates); thousand-claim recall wants batched
array entry points or a retained session, not per-row subprocesses.

## MEM-P4: small-state Store surface (non-blocking)

Memory persists atomic JSONL today (fsync per append; latest-wins
claim state; strict reload with corrupt-line identity). The durable
home is Store generations with a staged memory handshake; that
integration is future scope. Same stance as Learn (LEARN-P4).

## MEM-P5: Lineage relations for supersession (non-blocking)

Consolidation/correction preserve parent identities
(`derived_from`, relationship links, processor versions) ready for
Lineage, but no Lineage links are written yet — the API mapping
(many-to-one derivation, stale-descendant detection) is unmapped.

## MEM-P6: time model (non-blocking)

Freshness uses i64 instants with saturating age. No decay, retention
expiry, or duration arithmetic is implemented; when retention arrives
it should align with the project-wide time abstraction rather than a
Memory-private clock.

## MEM-P7: ranking stays heuristic (by design)

Relevance is deterministic lexical overlap. Adaptive/learned ranking
would integrate through Learn; no trained scorer is smuggled into
deterministic recall.

## Closed during this campaign

- Old-toolchain (0.6-era) policy modules ported to the current
  language generation with identical decision matrices.
- Wrapping score arithmetic preserved exactly (bounded features).
- Strict `u64`/`i64` separation, binding-only indexing,
  single-assignment, and reserved words (`over`, `next`, `clean`-class
  names) worked around without touching Language/Compiler.
- `mncs call` has no `--cache-dir` for `test` (call-only flag);
  cache configuration applies to the call path.
