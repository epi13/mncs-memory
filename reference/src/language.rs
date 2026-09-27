use std::fs;
use std::path::{Path, PathBuf};

use mncs_codegen::{execute_backend, lower_with_backend, plan_for_backend};
use mncs_compiler::{ModuleResolver, ReferenceCompiler};
use mncs_model::{
    ArtifactRepresentation, CompilerArtifactRef, ExecutionPolicy, ExecutionRequest,
    ExecutionStatus, ExecutionTarget, ExecutionValue, Program, TransformationStatus,
    EXECUTION_REQUEST_SCHEMA_VERSION, SSA_SCHEMA_VERSION,
};
use mncs_syntax::{
    SourceArtifactKind as SyntaxSourceArtifactKind, SourceEnvelope, SourceOrigin, SourceOriginKind,
};
use serde::Serialize;
use thiserror::Error;

use crate::model::{
    AnswerStatus, BudgetStatus, Disposition, EscalationDecision, EvidenceLevel, RelationHint,
};

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("unable to read MNCS source: {0}")]
    Io(#[from] std::io::Error),
    #[error("MNCS semantic source is invalid: {0}")]
    Invalid(String),
    #[error("MNCS semantic call failed: {0}")]
    Execution(String),
    #[error("MNCS backend realization failed: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackendObservation {
    pub backend: String,
    pub compilation: TransformationStatus,
    pub execution: ExecutionStatus,
    pub artifact_identity: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct LanguageRuntime {
    source_root: PathBuf,
    core_path: PathBuf,
    program: Program,
    semantic_identity: String,
}

impl LanguageRuntime {
    pub fn from_workspace_root(root: impl Into<PathBuf>) -> Result<Self, SemanticError> {
        let root = root.into();
        Self::new(root)
    }

    pub fn new(source_root: impl Into<PathBuf>) -> Result<Self, SemanticError> {
        let source_root = source_root.into();
        let core_path = source_root.join("mncs/memory/core.mncs");
        let text = fs::read_to_string(&core_path)?;
        let envelope = SourceEnvelope::new(
            SyntaxSourceArtifactKind::Program,
            core_path.to_string_lossy().to_string(),
            SourceOrigin {
                kind: SourceOriginKind::Path,
                locator: Some(core_path.to_string_lossy().to_string()),
            },
            text,
        );
        let resolver = FileModuleResolver {
            root: source_root.clone(),
        };
        let compiler = ReferenceCompiler::default();
        let front_end = compiler.front_end_with_resolver(envelope, &resolver);
        if !front_end.is_valid() {
            return Err(SemanticError::Invalid(format_diagnostics(
                &front_end.diagnostics,
            )));
        }
        let program = front_end
            .program
            .ok_or_else(|| SemanticError::Invalid("front end produced no program".to_owned()))?;
        let semantic_identity = program
            .content_fingerprint()
            .map_err(|error| SemanticError::Invalid(error.to_string()))?;
        Ok(Self {
            source_root,
            core_path,
            program,
            semantic_identity,
        })
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub fn core_path(&self) -> &Path {
        &self.core_path
    }

    pub fn semantic_identity(&self) -> &str {
        &self.semantic_identity
    }

    pub fn ingestion(
        &self,
        relation: RelationHint,
        evidence: EvidenceLevel,
    ) -> Result<Disposition, SemanticError> {
        let output = self.call_finite(
            "apply_ingestion",
            vec![
                self.enum_value("Relation", relation_name(relation))?,
                self.enum_value("Evidence", evidence_name(evidence))?,
            ],
        )?;
        disposition_from_name(&output)
    }

    pub fn budget(
        &self,
        required_units: i64,
        budget_units: i64,
        required_items: i64,
    ) -> Result<BudgetStatus, SemanticError> {
        let output = self.call_finite(
            "apply_budget",
            vec![
                integer(required_units),
                integer(budget_units),
                integer(required_items),
            ],
        )?;
        match output.as_str() {
            "FITS" => Ok(BudgetStatus::Fits),
            "INSUFFICIENT" => Ok(BudgetStatus::Insufficient),
            "UNKNOWN" => Ok(BudgetStatus::Unknown),
            other => Err(SemanticError::Execution(format!(
                "unknown budget result {other:?}"
            ))),
        }
    }

    pub fn escalation(
        &self,
        layer: &str,
        confidence: i64,
    ) -> Result<EscalationDecision, SemanticError> {
        let output = self.call_finite(
            "apply_escalation",
            vec![self.enum_value("Layer", layer)?, integer(confidence)],
        )?;
        match output.as_str() {
            "NONE" => Ok(EscalationDecision::None),
            "SPECIALIST" => Ok(EscalationDecision::Specialist),
            "GENERAL_REASONER" => Ok(EscalationDecision::GeneralReasoner),
            "UNKNOWN" => Ok(EscalationDecision::Unknown),
            other => Err(SemanticError::Execution(format!(
                "unknown escalation result {other:?}"
            ))),
        }
    }

    pub fn answer(
        &self,
        confidence: i64,
        minimum_confidence: i64,
        freshness: i64,
        freshness_required: i64,
    ) -> Result<AnswerStatus, SemanticError> {
        let output = self.call_finite(
            "apply_answer",
            vec![
                integer(confidence),
                integer(minimum_confidence),
                integer(freshness),
                integer(freshness_required),
            ],
        )?;
        match output.as_str() {
            "PASS" => Ok(AnswerStatus::Pass),
            "UNKNOWN" => Ok(AnswerStatus::Unknown),
            "FAIL" => Ok(AnswerStatus::Fail),
            other => Err(SemanticError::Execution(format!(
                "unknown answer result {other:?}"
            ))),
        }
    }

    pub fn score(
        &self,
        topic_match: i64,
        subject_match: i64,
        confidence: i64,
        freshness: i64,
    ) -> Result<i64, SemanticError> {
        let result = self.call(
            "rank_candidate",
            vec![
                integer(topic_match),
                integer(subject_match),
                integer(confidence),
                integer(freshness),
            ],
        )?;
        match result.first() {
            Some(ExecutionValue::Integer { value, .. }) => Ok(*value as i64),
            other => Err(SemanticError::Execution(format!(
                "expected integer score, got {other:?}"
            ))),
        }
    }

    pub fn backend_matrix(&self) -> Result<Vec<BackendObservation>, SemanticError> {
        let mut observations = Vec::new();
        for backend in ["mncs-research-bytecode", "mncs-portable-wasm-mvp"] {
            let ssa = self
                .program
                .lower_to_ssa()
                .map_err(|error| SemanticError::Backend(error.to_string()))?;
            let fingerprint = ssa
                .fingerprint()
                .map_err(|error| SemanticError::Backend(error.to_string()))?;
            let selected = CompilerArtifactRef::new(
                ArtifactRepresentation::SelectedSsa,
                SSA_SCHEMA_VERSION,
                fingerprint,
            );
            let Some(plan) = plan_for_backend(backend, selected.clone()) else {
                observations.push(BackendObservation {
                    backend: backend.to_owned(),
                    compilation: TransformationStatus::Unknown,
                    execution: ExecutionStatus::Unsupported,
                    artifact_identity: None,
                    note: "backend adapter unavailable".to_owned(),
                });
                continue;
            };
            let lowered = lower_with_backend(backend, &self.program, &ssa, selected, &plan);
            let Some(artifact) = lowered.artifact else {
                observations.push(BackendObservation {
                    backend: backend.to_owned(),
                    compilation: lowered.status,
                    execution: ExecutionStatus::Unsupported,
                    artifact_identity: lowered.artifact_ref.map(|id| id.identity.0),
                    note: format!("lowering refused: {:?}", lowered.diagnostics),
                });
                continue;
            };
            let request = ExecutionRequest {
                schema_version: EXECUTION_REQUEST_SCHEMA_VERSION.to_owned(),
                target: ExecutionTarget {
                    module: "mncs.memory.core".to_owned(),
                    function: "rank_candidate".to_owned(),
                },
                arguments: vec![integer(1), integer(1), integer(90), integer(1)],
                step_budget: 2_000,
                policy: ExecutionPolicy::default(),
            };
            let execution = execute_backend(&artifact, &request);
            observations.push(BackendObservation {
                backend: backend.to_owned(),
                compilation: lowered.status,
                execution: execution.status,
                artifact_identity: Some(artifact.identity.0),
                note: "bounded backend execution of the MNCS rank operator".to_owned(),
            });
        }
        Ok(observations)
    }

    fn call_finite(
        &self,
        function: &str,
        arguments: Vec<ExecutionValue>,
    ) -> Result<String, SemanticError> {
        let result = self.call(function, arguments)?;
        match result.first() {
            Some(ExecutionValue::Finite {
                type_identity,
                variant_identity,
                ..
            }) => self
                .program
                .finite_types
                .iter()
                .find(|finite| finite.identity == *type_identity)
                .and_then(|finite| {
                    finite
                        .variants
                        .iter()
                        .find(|variant| variant.identity == *variant_identity)
                })
                .map(|variant| variant.name.clone())
                .ok_or_else(|| {
                    SemanticError::Execution(
                        "returned finite value has no declaration identity".to_owned(),
                    )
                }),
            other => Err(SemanticError::Execution(format!(
                "expected finite result, got {other:?}"
            ))),
        }
    }

    fn call(
        &self,
        function: &str,
        arguments: Vec<ExecutionValue>,
    ) -> Result<Vec<ExecutionValue>, SemanticError> {
        let request = ExecutionRequest {
            schema_version: EXECUTION_REQUEST_SCHEMA_VERSION.to_owned(),
            target: ExecutionTarget {
                module: "mncs.memory.core".to_owned(),
                function: function.to_owned(),
            },
            arguments,
            step_budget: 10_000,
            policy: ExecutionPolicy::default(),
        };
        let result = mncs_model::execute_with_policy(&self.program, &request);
        if result.status != ExecutionStatus::Returned {
            return Err(SemanticError::Execution(
                result
                    .failure
                    .map(|failure| failure.reason)
                    .unwrap_or_else(|| format!("status was {:?}", result.status)),
            ));
        }
        Ok(result.returned)
    }

    fn enum_value(
        &self,
        type_name: &str,
        variant_name: &str,
    ) -> Result<ExecutionValue, SemanticError> {
        let finite = self
            .program
            .finite_types
            .iter()
            .find(|finite| finite.name == type_name)
            .ok_or_else(|| SemanticError::Invalid(format!("missing finite type {type_name}")))?;
        let variant = finite
            .variants
            .iter()
            .find(|variant| variant.name == variant_name)
            .ok_or_else(|| SemanticError::Invalid(format!("missing {type_name}.{variant_name}")))?;
        Ok(ExecutionValue::Finite {
            type_identity: finite.identity.clone(),
            variant_identity: variant.identity.clone(),
            discriminant: variant.discriminant,
            payload: Vec::new(),
        })
    }
}

fn integer(value: i64) -> ExecutionValue {
    ExecutionValue::Integer {
        value: value.into(),
        ty: mncs_model::IntegerType {
            bits: 64,
            signed: true,
        },
    }
}

fn relation_name(value: RelationHint) -> &'static str {
    match value {
        RelationHint::Unrelated => "UNRELATED",
        RelationHint::Same => "SAME",
        RelationHint::Refines => "REFINES",
        RelationHint::Contradicts => "CONTRADICTS",
        RelationHint::Supersedes => "SUPERSEDES",
        RelationHint::Ambiguous => "AMBIGUOUS",
    }
}

fn evidence_name(value: EvidenceLevel) -> &'static str {
    match value {
        EvidenceLevel::Direct => "DIRECT",
        EvidenceLevel::Supported => "SUPPORTED",
        EvidenceLevel::Weak => "WEAK",
        EvidenceLevel::Missing => "MISSING",
    }
}

fn disposition_from_name(name: &str) -> Result<Disposition, SemanticError> {
    match name {
        "NEW" => Ok(Disposition::New),
        "DUPLICATE" => Ok(Disposition::Duplicate),
        "REFINEMENT" => Ok(Disposition::Refinement),
        "CONTRADICTION" => Ok(Disposition::Contradiction),
        "SUPERSESSION" => Ok(Disposition::Supersession),
        "UNKNOWN" => Ok(Disposition::Unknown),
        other => Err(SemanticError::Execution(format!(
            "unknown ingestion result {other:?}"
        ))),
    }
}

fn format_diagnostics(diagnostics: &[mncs_syntax::SourceDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .collect::<Vec<_>>()
        .join("; ")
}

struct FileModuleResolver {
    root: PathBuf,
}

impl ModuleResolver for FileModuleResolver {
    fn resolve(&self, module: &str) -> Option<SourceEnvelope> {
        let path = self.root.join(format!("{}.mncs", module.replace('.', "/")));
        let text = fs::read_to_string(&path).ok()?;
        Some(SourceEnvelope::new(
            SyntaxSourceArtifactKind::Program,
            path.to_string_lossy().to_string(),
            SourceOrigin {
                kind: SourceOriginKind::Path,
                locator: Some(path.to_string_lossy().to_string()),
            },
            text,
        ))
    }
}
