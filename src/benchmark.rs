use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::model::{AnswerCapsule, AnswerStatus, Observation, SemanticRequest};
use crate::scenarios::{self, Corpus, ScenarioQuery};
use crate::{EngineError, MemoryEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub schema_version: String,
    pub corpus: String,
    pub scenario_count: usize,
    pub cases: Vec<BenchmarkCase>,
    pub aggregate: BenchmarkAggregate,
    pub epistemic_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCase {
    pub id: String,
    pub note: String,
    pub raw_context: BaselineResult,
    pub simple_retrieval: BaselineResult,
    pub mncs_memory: MemoryResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResult {
    pub answer: String,
    pub status: AnswerStatus,
    pub correct_against_frozen_oracle: bool,
    pub evaluated_as_answer: bool,
    pub required_fact_recall: f64,
    pub irrelevant_fact_inclusion: usize,
    pub units: usize,
    pub candidates_exposed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub capsule: AnswerCapsule,
    pub correct_against_frozen_oracle: bool,
    pub required_fact_recall: f64,
    pub irrelevant_fact_inclusion: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkAggregate {
    pub memory_correct: usize,
    pub simple_retrieval_correct: usize,
    pub raw_answerable_by_context_scan: usize,
    pub memory_required_fact_recall: f64,
    pub simple_required_fact_recall: f64,
    pub raw_required_fact_recall: f64,
    pub memory_irrelevant_fact_inclusion: usize,
    pub simple_irrelevant_fact_inclusion: usize,
    pub memory_units_total: usize,
    pub simple_units_total: usize,
    pub raw_units_total: usize,
    pub memory_candidates_total: usize,
    pub simple_candidates_total: usize,
    pub raw_candidates_total: usize,
    pub memory_escalation_cases: usize,
    pub memory_unknown_cases: usize,
    pub specialist_invocations: usize,
}

pub fn run(language_root: impl AsRef<Path>) -> Result<BenchmarkReport, EngineError> {
    let corpus = scenarios::load();
    run_corpus(language_root, &corpus)
}

pub fn run_corpus(
    language_root: impl AsRef<Path>,
    corpus: &Corpus,
) -> Result<BenchmarkReport, EngineError> {
    let mut engine = MemoryEngine::new(
        language_root.as_ref().to_path_buf(),
        corpus.entities.clone(),
    )?;
    for observation in &corpus.observations {
        engine.ingest(observation.clone())?;
    }
    let mut cases = Vec::new();
    let mut aggregate = BenchmarkAggregate::default();
    for scenario in &corpus.queries {
        let raw = raw_context(&engine, scenario);
        let simple = simple_retrieval(&engine, scenario);
        let capsule = engine.query(scenario.request.clone())?;
        let memory = MemoryResult {
            required_fact_recall: recall(&capsule.answer, &scenario.expected_values),
            irrelevant_fact_inclusion: irrelevant_inclusion(
                capsule.items.iter().map(|item| item.topic.as_str()),
                &scenario.expected_topics,
            ),
            correct_against_frozen_oracle: capsule_correct(&capsule, scenario),
            capsule,
        };
        aggregate.memory_correct += usize::from(memory.correct_against_frozen_oracle);
        aggregate.simple_retrieval_correct += usize::from(simple.correct_against_frozen_oracle);
        aggregate.raw_answerable_by_context_scan += usize::from(raw.correct_against_frozen_oracle);
        aggregate.memory_required_fact_recall += memory.required_fact_recall;
        aggregate.simple_required_fact_recall += simple.required_fact_recall;
        aggregate.raw_required_fact_recall += raw.required_fact_recall;
        aggregate.memory_irrelevant_fact_inclusion += memory.irrelevant_fact_inclusion;
        aggregate.simple_irrelevant_fact_inclusion += simple.irrelevant_fact_inclusion;
        aggregate.memory_units_total += memory.capsule.units;
        aggregate.simple_units_total += simple.units;
        aggregate.raw_units_total += raw.units;
        aggregate.memory_candidates_total += memory.capsule.candidates_exposed;
        aggregate.simple_candidates_total += simple.candidates_exposed;
        aggregate.raw_candidates_total += raw.candidates_exposed;
        aggregate.memory_escalation_cases +=
            usize::from(memory.capsule.escalation != crate::model::EscalationDecision::None);
        aggregate.memory_unknown_cases +=
            usize::from(memory.capsule.status == AnswerStatus::Unknown);
        cases.push(BenchmarkCase {
            id: scenario.id.clone(),
            note: scenario.note.clone(),
            raw_context: raw,
            simple_retrieval: simple,
            mncs_memory: memory,
        });
    }
    aggregate.specialist_invocations = engine.store().invocations().len();
    let denominator = cases.len().max(1) as f64;
    aggregate.memory_required_fact_recall /= denominator;
    aggregate.simple_required_fact_recall /= denominator;
    aggregate.raw_required_fact_recall /= denominator;
    Ok(BenchmarkReport {
        schema_version: "mncs-memory.benchmark/0.1".to_owned(),
        corpus: corpus.name.clone(),
        scenario_count: cases.len(),
        cases,
        aggregate,
        epistemic_note: "Frozen bounded synthetic corpus; results are comparative observations, not general memory performance claims.".to_owned(),
    })
}

fn raw_context(engine: &MemoryEngine, scenario: &ScenarioQuery) -> BaselineResult {
    let observations = engine
        .store()
        .observations()
        .iter()
        .filter(|observation| raw_matches(observation, &scenario.request))
        .collect::<Vec<_>>();
    let answer = observations
        .iter()
        .map(|observation| observation.input.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    BaselineResult {
        correct_against_frozen_oracle: scenario
            .expected_values
            .iter()
            .all(|value| answer.contains(value))
            && (scenario.expected_status == AnswerStatus::Unknown
                || scenario.expected_values.is_empty()),
        evaluated_as_answer: false,
        required_fact_recall: recall(&answer, &scenario.expected_values),
        irrelevant_fact_inclusion: irrelevant_inclusion(
            observations
                .iter()
                .map(|observation| observation.input.topic.as_str()),
            &scenario.expected_topics,
        ),
        units: answer.len(),
        candidates_exposed: observations.len(),
        answer,
        status: AnswerStatus::Unknown,
    }
}

fn simple_retrieval(engine: &MemoryEngine, scenario: &ScenarioQuery) -> BaselineResult {
    let query_tokens = tokens(&format!(
        "{} {} {}",
        scenario.request.goal,
        scenario.request.topic.clone().unwrap_or_default(),
        scenario.request.subject.clone().unwrap_or_default()
    ));
    let mut ranked = engine
        .store()
        .observations()
        .iter()
        .filter(|observation| observation.input.kind != crate::model::MemoryKind::Noise)
        .map(|observation| {
            let haystack = tokens(&format!(
                "{} {} {} {}",
                observation.input.text,
                observation.input.topic,
                observation.input.value,
                observation.input.subject_alias
            ));
            let score = query_tokens.intersection(&haystack).count();
            (score, observation)
        })
        .filter(|(score, _)| *score > 0)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.identity.cmp(&right.1.identity))
    });
    ranked.truncate(3);
    let answer = ranked
        .iter()
        .map(|(_, observation)| format!("{}={}", observation.input.topic, observation.input.value))
        .collect::<Vec<_>>()
        .join(";");
    let status = if ranked.is_empty() {
        AnswerStatus::Unknown
    } else {
        AnswerStatus::Pass
    };
    BaselineResult {
        correct_against_frozen_oracle: status == scenario.expected_status
            && scenario
                .expected_values
                .iter()
                .all(|value| answer.contains(value)),
        evaluated_as_answer: true,
        required_fact_recall: recall(&answer, &scenario.expected_values),
        irrelevant_fact_inclusion: irrelevant_inclusion(
            ranked
                .iter()
                .map(|(_, observation)| observation.input.topic.as_str()),
            &scenario.expected_topics,
        ),
        units: answer.len(),
        candidates_exposed: ranked.len(),
        answer,
        status,
    }
}

fn capsule_correct(capsule: &AnswerCapsule, scenario: &ScenarioQuery) -> bool {
    capsule.status == scenario.expected_status
        && scenario
            .expected_topics
            .iter()
            .all(|topic| capsule.items.iter().any(|item| &item.topic == topic))
        && scenario
            .expected_values
            .iter()
            .all(|value| capsule.answer.contains(value))
}

fn raw_matches(observation: &Observation, request: &SemanticRequest) -> bool {
    request.subject.as_deref().is_none_or(|subject| {
        observation.input.subject_alias == subject || observation.input.text.contains(subject)
    }) && request
        .topic
        .as_deref()
        .is_none_or(|topic| observation.input.topic == topic)
}

fn recall(answer: &str, expected: &[String]) -> f64 {
    if expected.is_empty() {
        return 1.0;
    }
    expected
        .iter()
        .filter(|value| answer.contains(*value))
        .count() as f64
        / expected.len() as f64
}

fn irrelevant_inclusion<'a>(topics: impl Iterator<Item = &'a str>, expected: &[String]) -> usize {
    let expected = expected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    topics.filter(|topic| !expected.contains(topic)).count()
}

fn tokens(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| token.len() > 1)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}
