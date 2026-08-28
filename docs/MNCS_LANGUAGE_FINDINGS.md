# MNCS Language findings

This first workload uses current main snapshot `8447dad` and made no upstream
source edits. The existing language was sufficient for a bounded semantic
core, so no one-off compiler or project-local policy interpreter was added.

| Requirement | Existing capability | Problem observed | Change made | Coverage | Remaining limitation |
| --- | --- | --- | --- | --- | --- |
| six-way memory disposition | Profile 0.6 enums, exhaustive `match`, bounded executable bodies | none | Expressed in `policy.mncs` | `linked_mncs_semantics_drive_all_six_ingestion_dispositions`; body/SSA calls | no generic sum types or optional values in this slice |
| split semantic policy into modules | Profile 0.6 imports and elaboration-time linking | none | `core.mncs` links policy, budget, escalation, and query modules | `LanguageRuntime::new` and linked calls | resolver is a host filesystem adapter |
| bounded ranking arithmetic | explicit arithmetic-intent operators and backend refusal | checked `*` was refused by portable WASM for the score operator | changed score formula to intentional wrapping `*%`/`+%` because scores are bounded features | backend realization test | score range is bounded experiment, not a general numeric proof |
| negative sentinel values | current expression grammar | unary negative literal was not needed after score contract narrowed | removed sentinel and made invalid score inputs return zero; host supplies nonnegative features | backend matrix and ranking calls | negative literals remain a possible future syntax improvement |
| maps, strings, persistence, provider dispatch | host adapters are appropriate | no language pressure; these are effects/representation boundaries | kept them in thin Rust host code | storage, specialist, replay tests | no claim current MNCS is ready for arbitrary text processing |

No language/compiler/stdlib regression fix was justified by the memory
workload. The local `mncs-language` checkout also contained unrelated dirty
Profile 0.10 generic-parameter work during this run; it was preserved and not
modified. The memory repository pins a clean current-main commit for
reproducible compilation.
