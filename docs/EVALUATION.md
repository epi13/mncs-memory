# Evaluation

The checked-in corpus is `corpus/adversarial.json`. It contains 20 observations
and 11 query cases covering repeated/paraphrased duplicates, temporal change,
contradiction, refinement, ambiguous aliases, similar names, stale evidence,
supersession, irrelevant similarity, multi-memory queries, insufficient
evidence, budget pressure, provenance, explicit raw history, faulty behavior,
and escalation. It is bounded synthetic evidence, not a general benchmark.

The harness compares:

- **Raw context**: matching observation text is exposed without an answer
  computation; `evaluated_as_answer` is false.
- **Simple retrieval**: top three lexical observations are exposed as a
  deliberately simple retrieval/answer baseline.
- **MNCS Memory**: structured ingestion, consolidation, typed specialists,
  MNCS policy decisions, and bounded capsules.

The generated artifact records frozen-oracle correctness, required-fact recall,
irrelevant inclusion, units, candidate counts, escalations, unknown results,
and specialist invocation count. Units are characters in rendered capsules or
baseline strings.

Current bounded result:

| path | frozen cases correct | required-fact recall | irrelevant topics included | units | candidates |
| --- | ---: | ---: | ---: | ---: | ---: |
| MNCS Memory | 11/11 | 1.00 | 0 | 280 | 23 |
| Simple retrieval | 6/11 | 1.00 | 16 | 700 | 33 |
| Raw context | not an answer path | 0.45 scan recall | not interpreted | 84 | 3 |

Retrieval's required-fact recall is not sufficient evidence of a correct
answer: it frequently includes stale, contradicted, or irrelevant facts. The
raw path is intentionally not scored as if a reasoner had already performed
the missing semantic work.
