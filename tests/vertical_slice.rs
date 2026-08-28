use std::path::PathBuf;

use mncs_memory::scenarios;
use mncs_memory::{
    AnswerStatus, DeterministicSpecialists, Disposition, EvidenceLevel, LanguageRuntime,
    MemoryEngine, MemoryKind, ObservationInput, RelationHint, SemanticRequest,
};
use mncs_model::{ExecutionStatus, TransformationStatus};

fn language_root() -> PathBuf {
    std::env::var_os("MNCS_MEMORY_LANGUAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("language"))
}

fn corpus_engine() -> MemoryEngine {
    let corpus = scenarios::load();
    let mut engine =
        MemoryEngine::new(language_root(), corpus.entities).expect("language core loads");
    for observation in corpus.observations {
        engine
            .ingest(observation)
            .expect("corpus observation ingests");
    }
    engine
}

#[test]
fn linked_mncs_semantics_drive_all_six_ingestion_dispositions() {
    let language = LanguageRuntime::new(language_root()).expect("linked MNCS program");
    assert_eq!(
        language
            .ingestion(RelationHint::Unrelated, EvidenceLevel::Direct)
            .unwrap(),
        Disposition::New
    );
    assert_eq!(
        language
            .ingestion(RelationHint::Same, EvidenceLevel::Supported)
            .unwrap(),
        Disposition::Duplicate
    );
    assert_eq!(
        language
            .ingestion(RelationHint::Refines, EvidenceLevel::Direct)
            .unwrap(),
        Disposition::Refinement
    );
    assert_eq!(
        language
            .ingestion(RelationHint::Contradicts, EvidenceLevel::Direct)
            .unwrap(),
        Disposition::Contradiction
    );
    assert_eq!(
        language
            .ingestion(RelationHint::Supersedes, EvidenceLevel::Direct)
            .unwrap(),
        Disposition::Supersession
    );
    assert_eq!(
        language
            .ingestion(RelationHint::Contradicts, EvidenceLevel::Weak)
            .unwrap(),
        Disposition::Unknown
    );
}

#[test]
fn changing_mncs_policy_changes_the_host_decision() {
    let source_root = language_root();
    let root = std::env::temp_dir().join(format!(
        "mncs-memory-semantics-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let memory_root = root.join("mncs/memory");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&memory_root).unwrap();
    for module in [
        "core.mncs",
        "policy.mncs",
        "budget.mncs",
        "escalation.mncs",
        "query.mncs",
    ] {
        std::fs::copy(
            source_root.join("mncs/memory").join(module),
            memory_root.join(module),
        )
        .unwrap();
    }
    let policy_path = memory_root.join("policy.mncs");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    let changed = policy.replace(
        "WEAK => Disposition.UNKNOWN,\n            MISSING => Disposition.UNKNOWN,",
        "WEAK => Disposition.CONTRADICTION,\n            MISSING => Disposition.UNKNOWN,",
    );
    assert_ne!(policy, changed);
    std::fs::write(&policy_path, changed).unwrap();

    let altered = LanguageRuntime::new(&root).unwrap();
    assert_eq!(
        altered
            .ingestion(RelationHint::Contradicts, EvidenceLevel::Weak)
            .unwrap(),
        Disposition::Contradiction
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn corpus_exercises_duplicate_refinement_conflict_supersession_unknown() {
    let corpus = scenarios::load();
    let mut engine = MemoryEngine::new(language_root(), corpus.entities).unwrap();
    let mut dispositions = Vec::new();
    for observation in corpus.observations {
        dispositions.push((
            observation.id.clone(),
            engine.ingest(observation).unwrap().disposition,
        ));
    }
    let by_id = |id: &str| {
        dispositions
            .iter()
            .find(|(candidate, _)| candidate == id)
            .unwrap()
            .1
    };
    assert_eq!(by_id("repeat-1"), Disposition::New);
    assert_eq!(by_id("repeat-2"), Disposition::Duplicate);
    assert_eq!(by_id("change-1"), Disposition::Supersession);
    assert_eq!(by_id("contradiction-2"), Disposition::Contradiction);
    assert_eq!(by_id("refinement-2"), Disposition::Refinement);
    assert_eq!(by_id("ambiguous-alias"), Disposition::Unknown);
    assert_eq!(by_id("noise-1"), Disposition::Unknown);
}

#[test]
fn query_path_returns_capsules_and_keeps_conflicts_unknown() {
    let corpus = scenarios::load();
    let mut engine = corpus_engine();
    for scenario in corpus.queries {
        let capsule = engine.query(scenario.request).unwrap();
        assert_eq!(capsule.status, scenario.expected_status, "{}", scenario.id);
        for topic in scenario.expected_topics {
            assert!(
                capsule.items.iter().any(|item| item.topic == topic),
                "{} missing {topic}",
                scenario.id
            );
        }
        for value in scenario.expected_values {
            assert!(
                capsule.answer.contains(&value),
                "{} missing {value}",
                scenario.id
            );
        }
        assert!(capsule.units <= capsule.budget || capsule.status == AnswerStatus::Unknown);
    }
}

#[test]
fn provenance_is_opt_in_and_raw_history_is_explicit() {
    let corpus = scenarios::load();
    let mut engine = corpus_engine();
    let provenance = engine
        .query(
            corpus
                .queries
                .iter()
                .find(|scenario| scenario.id == "provenance")
                .unwrap()
                .request
                .clone(),
        )
        .unwrap();
    assert!(provenance.provenance_included);
    assert!(provenance
        .items
        .iter()
        .all(|item| !item.supporting_observations.is_empty()));
    let compact_request = SemanticRequest {
        subject: Some("entity:alice".to_owned()),
        topic: Some("editor".to_owned()),
        goal: "Alice editor".to_owned(),
        ..SemanticRequest::default()
    };
    let compact = engine.query(compact_request).unwrap();
    assert!(!compact.provenance_included);
    assert!(compact
        .items
        .iter()
        .all(|item| item.supporting_observations.is_empty()));
}

#[test]
fn persistent_store_reloads_append_only_observations_and_latest_claim_state() {
    let corpus = scenarios::load();
    let root = std::env::temp_dir().join(format!("mncs-memory-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let language = LanguageRuntime::new(language_root()).unwrap();
    let store = mncs_memory::MemoryStore::open(&root).unwrap();
    let mut engine = MemoryEngine::with_store(
        language,
        store,
        corpus.entities.clone(),
        Box::new(DeterministicSpecialists::new(corpus.entities.clone())),
    );
    engine.ingest(corpus.observations[0].clone()).unwrap();
    engine.ingest(corpus.observations[2].clone()).unwrap();
    assert_eq!(engine.store().observations().len(), 2);
    let reopened = mncs_memory::MemoryStore::open(&root).unwrap();
    assert_eq!(reopened.observations().len(), 2);
    assert!(reopened
        .claims()
        .any(|claim| claim.state == mncs_memory::ClaimState::Superseded));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn faulty_entity_specialist_is_replayed_from_retained_observation() {
    let corpus = scenarios::load();
    let language = LanguageRuntime::new(language_root()).unwrap();
    let specialists = DeterministicSpecialists::new(corpus.entities.clone())
        .faulty_alias_target(&corpus.replay.alias, &corpus.replay.faulty_target);
    let mut engine = MemoryEngine::with_store(
        language,
        mncs_memory::MemoryStore::in_memory(),
        corpus.entities.clone(),
        Box::new(specialists),
    );
    let observation = corpus
        .observations
        .iter()
        .find(|observation| observation.id == corpus.replay.observation_id)
        .unwrap()
        .clone();
    let old = engine.ingest(observation).unwrap();
    assert_eq!(old.disposition, Disposition::New);
    assert!(engine
        .store()
        .claims()
        .any(|claim| claim.subject == corpus.replay.faulty_target));
    let replacement = Box::new(DeterministicSpecialists::new(corpus.entities.clone()));
    let replay = engine.replay_with(replacement).unwrap();
    assert_eq!(replay.old_version, "faulty-entity-v1");
    assert_eq!(replay.new_version, "reference-v1");
    assert!(!replay.affected_claims.is_empty());
    assert!(!replay.recomputed_claims.is_empty());
    assert!(engine.store().claims().any(|claim| {
        claim.subject == corpus.replay.expected_correct_subject
            && claim.processor.version == "reference-v1"
    }));
    assert_eq!(engine.store().observations().len(), 1);
}

#[test]
fn supported_backend_realizations_are_attempted_for_mncs_core() {
    let language = LanguageRuntime::new(language_root()).unwrap();
    let observations = language.backend_matrix().unwrap();
    assert_eq!(observations.len(), 2);
    assert!(
        observations.iter().all(|observation| {
            observation.compilation == TransformationStatus::Pass
                && observation.execution == ExecutionStatus::Returned
        }),
        "backend observations: {observations:?}"
    );
}

#[test]
fn benchmark_preserves_a_non_winning_simple_retrieval_baseline() {
    let report = mncs_memory::benchmark::run(language_root()).unwrap();
    assert_eq!(report.scenario_count, 11);
    assert_eq!(report.aggregate.memory_correct, 11);
    assert_eq!(report.aggregate.simple_retrieval_correct, 6);
    assert_eq!(report.aggregate.memory_irrelevant_fact_inclusion, 0);
    assert!(report.aggregate.simple_irrelevant_fact_inclusion > 0);
    assert!(report.aggregate.memory_units_total < report.aggregate.simple_units_total);
    assert!(report.aggregate.memory_candidates_total < report.aggregate.simple_candidates_total);
}

#[allow(dead_code)]
fn _typed_observation_fixture() -> ObservationInput {
    ObservationInput {
        id: "fixture".to_owned(),
        subject_alias: "alice".to_owned(),
        topic: "demo".to_owned(),
        value: "value".to_owned(),
        text: "Alice has a value.".to_owned(),
        observed_at: 1,
        kind: MemoryKind::Fact,
        evidence: EvidenceLevel::Direct,
        explicit_change: false,
        refinement: false,
        source: "test".to_owned(),
    }
}
