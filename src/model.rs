use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryKind {
    Preference,
    Fact,
    Decision,
    Event,
    Noise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceLevel {
    Direct,
    Supported,
    Weak,
    Missing,
}

impl EvidenceLevel {
    pub fn score(self) -> i64 {
        match self {
            Self::Direct => 100,
            Self::Supported => 85,
            Self::Weak => 55,
            Self::Missing => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationHint {
    Unrelated,
    Same,
    Refines,
    Contradicts,
    Supersedes,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Disposition {
    New,
    Duplicate,
    Refinement,
    Contradiction,
    Supersession,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimState {
    Current,
    Refined,
    Superseded,
    Contradicted,
    Unresolved,
    Duplicate,
    Unknown,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryIntent {
    WhatShouldIKnowBefore,
    WhatIsKnownAbout,
    WhatChanged,
    WhatConflictsWith,
    WhatSupports,
    RawEpisodes,
    Uncertainty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnswerStatus {
    Pass,
    Unknown,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EscalationDecision {
    None,
    Specialist,
    GeneralReasoner,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BudgetStatus {
    Fits,
    Insufficient,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationInput {
    pub id: String,
    pub subject_alias: String,
    pub topic: String,
    pub value: String,
    pub text: String,
    pub observed_at: i64,
    pub kind: MemoryKind,
    pub evidence: EvidenceLevel,
    #[serde(default)]
    pub explicit_change: bool,
    #[serde(default)]
    pub refinement: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub identity: String,
    pub input: ObservationInput,
    pub immutable_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub identity: String,
    pub canonical_name: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessorRef {
    pub role: String,
    pub identity: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub identity: String,
    pub subject: String,
    pub topic: String,
    pub value: String,
    pub kind: MemoryKind,
    pub state: ClaimState,
    pub disposition: Disposition,
    pub confidence: i64,
    pub evidence: EvidenceLevel,
    pub valid_from: i64,
    pub valid_until: Option<i64>,
    pub derived_from: Vec<String>,
    pub processor: ProcessorRef,
    pub supersedes: Vec<String>,
    pub refines: Vec<String>,
    pub contradicts: Vec<String>,
    pub created_at: i64,
    pub last_confirmed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationEvent {
    pub identity: String,
    pub requested_by: String,
    pub from_layer: String,
    pub to_layer: String,
    pub uncertainty: String,
    pub exposed_context: String,
    pub resolution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRequest {
    pub intent: QueryIntent,
    pub subject: Option<String>,
    pub topic: Option<String>,
    pub goal: String,
    pub as_of: i64,
    pub max_units: usize,
    pub minimum_confidence: i64,
    pub required_topics: Vec<String>,
    pub forbidden_topics: Vec<String>,
    pub provenance_requested: bool,
    pub freshness_required: Option<i64>,
    pub max_supporting_claims: usize,
    pub raw_episodes: bool,
    pub include_conflicts: bool,
    pub include_uncertainty: bool,
}

impl Default for SemanticRequest {
    fn default() -> Self {
        Self {
            intent: QueryIntent::WhatShouldIKnowBefore,
            subject: None,
            topic: None,
            goal: String::new(),
            as_of: i64::MAX,
            max_units: 120,
            minimum_confidence: 85,
            required_topics: Vec::new(),
            forbidden_topics: Vec::new(),
            provenance_requested: false,
            freshness_required: None,
            max_supporting_claims: 4,
            raw_episodes: false,
            include_conflicts: false,
            include_uncertainty: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerItem {
    pub claim_identity: String,
    pub subject: String,
    pub topic: String,
    pub value: String,
    pub confidence: i64,
    pub valid_from: i64,
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_observations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerCapsule {
    pub schema_version: String,
    pub status: AnswerStatus,
    pub answer: String,
    pub items: Vec<AnswerItem>,
    pub units: usize,
    pub budget: usize,
    pub required_topics_preserved: bool,
    pub candidates_exposed: usize,
    pub source_claims_exposed: usize,
    pub escalation: EscalationDecision,
    pub uncertainty: Vec<String>,
    pub provenance_included: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionRecord {
    pub observation: Observation,
    pub disposition: Disposition,
    pub claim: Option<Claim>,
    pub processor_invocations: Vec<ProcessorInvocationRecord>,
    pub escalation: Option<EscalationEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessorInvocationRecord {
    pub specialist: ProcessorRef,
    pub input_schema: String,
    pub input_digest: String,
    pub output: String,
    pub confidence: i64,
    pub evidence: Verdict,
    pub input_units: usize,
    pub output_units: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayReport {
    pub processor_role: String,
    pub old_version: String,
    pub new_version: String,
    pub affected_claims: Vec<String>,
    pub invalidated_claims: Vec<String>,
    pub recomputed_claims: Vec<String>,
    pub unchanged_observations: usize,
}
