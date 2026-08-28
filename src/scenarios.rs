use serde::{Deserialize, Serialize};

use crate::model::{ObservationInput, SemanticRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    pub schema_version: String,
    pub name: String,
    pub entities: Vec<crate::model::Entity>,
    pub observations: Vec<ObservationInput>,
    pub queries: Vec<ScenarioQuery>,
    pub replay: ReplayScenario,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioQuery {
    pub id: String,
    pub request: SemanticRequest,
    pub expected_status: crate::model::AnswerStatus,
    pub expected_topics: Vec<String>,
    pub expected_values: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayScenario {
    pub observation_id: String,
    pub alias: String,
    pub faulty_target: String,
    pub expected_correct_subject: String,
}

pub fn load() -> Corpus {
    serde_json::from_str(include_str!("../corpus/adversarial.json"))
        .expect("checked-in adversarial corpus must be valid")
}
