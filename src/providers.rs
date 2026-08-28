use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::model::{
    Claim, Entity, MemoryKind, ObservationInput, ProcessorInvocationRecord, ProcessorRef,
    QueryIntent, RelationHint, Verdict,
};

pub type SpecialistInvocation = ProcessorInvocationRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialistRequest {
    Worthiness {
        observation: ObservationInput,
    },
    EntityResolution {
        alias: String,
    },
    Relationship {
        observation: ObservationInput,
        existing: Vec<Claim>,
    },
    QueryRoute {
        intent: QueryIntent,
    },
    Relevance {
        goal: String,
        subject: String,
        topic: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialistOutcome {
    pub decision: String,
    pub confidence: i64,
    pub evidence: Verdict,
    pub rationale: String,
    pub input_units: usize,
    pub output_units: usize,
}

pub trait Specialist: Send + Sync {
    fn processor(&self) -> ProcessorRef;
    fn invoke(&self, request: &SpecialistRequest) -> SpecialistOutcome;
}

#[derive(Debug, Clone)]
pub struct DeterministicSpecialists {
    entities: BTreeMap<String, Vec<Entity>>,
    faulty_entity_resolution: bool,
    faulty_alias_target: Option<(String, String)>,
}

impl DeterministicSpecialists {
    pub fn new(entities: impl IntoIterator<Item = Entity>) -> Self {
        let mut aliases: BTreeMap<String, Vec<Entity>> = BTreeMap::new();
        for entity in entities {
            for alias in
                std::iter::once(entity.canonical_name.clone()).chain(entity.aliases.clone())
            {
                aliases
                    .entry(normalize(&alias))
                    .or_default()
                    .push(entity.clone());
            }
        }
        for matches in aliases.values_mut() {
            matches.sort_by(|left, right| left.identity.cmp(&right.identity));
            matches.dedup_by(|left, right| left.identity == right.identity);
        }
        Self {
            entities: aliases,
            faulty_entity_resolution: false,
            faulty_alias_target: None,
        }
    }

    pub fn faulty_entity_resolver(mut self, enabled: bool) -> Self {
        self.faulty_entity_resolution = enabled;
        self
    }

    pub fn faulty_alias_target(
        mut self,
        alias: impl Into<String>,
        target_identity: impl Into<String>,
    ) -> Self {
        self.faulty_entity_resolution = true;
        self.faulty_alias_target = Some((normalize(&alias.into()), target_identity.into()));
        self
    }

    pub fn entity_matches(&self, alias: &str) -> &[Entity] {
        self.entities
            .get(&normalize(alias))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

impl Specialist for DeterministicSpecialists {
    fn processor(&self) -> ProcessorRef {
        ProcessorRef {
            role: "deterministic-specialists".to_owned(),
            identity: "mncs-memory.deterministic.reference".to_owned(),
            version: if self.faulty_entity_resolution {
                "faulty-entity-v1".to_owned()
            } else {
                "reference-v1".to_owned()
            },
        }
    }

    fn invoke(&self, request: &SpecialistRequest) -> SpecialistOutcome {
        let input = serde_json::to_string(request).unwrap_or_default();
        let input_units = input.len();
        let mut outcome = match request {
            SpecialistRequest::Worthiness { observation } => {
                if observation.kind == MemoryKind::Noise || observation.text.trim().is_empty() {
                    SpecialistOutcome {
                        decision: "NO".to_owned(),
                        confidence: 100,
                        evidence: Verdict::Pass,
                        rationale: "noise or empty observation".to_owned(),
                        input_units,
                        output_units: 2,
                    }
                } else {
                    SpecialistOutcome {
                        decision: "YES".to_owned(),
                        confidence: 100,
                        evidence: Verdict::Pass,
                        rationale: "bounded reference worthiness rule".to_owned(),
                        input_units,
                        output_units: 3,
                    }
                }
            }
            SpecialistRequest::EntityResolution { alias } => {
                let matches = self.entity_matches(alias);
                if self
                    .faulty_alias_target
                    .as_ref()
                    .is_some_and(|(faulty_alias, _)| faulty_alias == &normalize(alias))
                {
                    let target = self
                        .faulty_alias_target
                        .as_ref()
                        .map(|(_, target)| target.clone())
                        .unwrap_or_default();
                    SpecialistOutcome {
                        decision: target,
                        confidence: 90,
                        evidence: Verdict::Fail,
                        rationale:
                            "intentionally faulty provider selected a wrong canonical entity"
                                .to_owned(),
                        input_units,
                        output_units: 8,
                    }
                } else if matches.is_empty() {
                    SpecialistOutcome {
                        decision: "UNKNOWN".to_owned(),
                        confidence: 0,
                        evidence: Verdict::Unknown,
                        rationale: "no registered canonical entity matches alias".to_owned(),
                        input_units,
                        output_units: 7,
                    }
                } else if matches.len() > 1 {
                    let decision = if self.faulty_entity_resolution {
                        matches[0].identity.clone()
                    } else {
                        "UNKNOWN".to_owned()
                    };
                    SpecialistOutcome {
                        decision,
                        confidence: if self.faulty_entity_resolution {
                            90
                        } else {
                            40
                        },
                        evidence: if self.faulty_entity_resolution {
                            Verdict::Fail
                        } else {
                            Verdict::Unknown
                        },
                        rationale: if self.faulty_entity_resolution {
                            "intentionally faulty provider selected first ambiguous entity"
                                .to_owned()
                        } else {
                            "alias resolves to multiple entities".to_owned()
                        },
                        input_units,
                        output_units: 8,
                    }
                } else {
                    SpecialistOutcome {
                        decision: matches[0].identity.clone(),
                        confidence: 100,
                        evidence: Verdict::Pass,
                        rationale: "unique canonical alias match".to_owned(),
                        input_units,
                        output_units: matches[0].identity.len(),
                    }
                }
            }
            SpecialistRequest::Relationship {
                observation,
                existing,
            } => {
                let same_topic = existing
                    .iter()
                    .filter(|claim| claim.topic == observation.topic)
                    .collect::<Vec<_>>();
                if same_topic.is_empty() {
                    relation(RelationHint::Unrelated, 100, "no same-topic claim")
                } else if same_topic
                    .iter()
                    .any(|claim| claim.value == observation.value)
                {
                    relation(RelationHint::Same, 100, "value matches an existing claim")
                } else if observation.refinement {
                    relation(
                        RelationHint::Refines,
                        92,
                        "observation explicitly refines prior scope",
                    )
                } else if observation.explicit_change {
                    relation(
                        RelationHint::Supersedes,
                        95,
                        "observation explicitly records a change",
                    )
                } else {
                    relation(
                        RelationHint::Contradicts,
                        90,
                        "same subject/topic has a different value",
                    )
                }
            }
            SpecialistRequest::QueryRoute { intent } => SpecialistOutcome {
                decision: format!("{intent:?}"),
                confidence: 100,
                evidence: Verdict::Pass,
                rationale: "typed intent route".to_owned(),
                input_units,
                output_units: 8,
            },
            SpecialistRequest::Relevance {
                goal,
                subject: _,
                topic,
                value,
            } => {
                let goal_tokens = tokens(goal);
                let mut matched = BTreeSet::new();
                // Subject identity is routed separately. Including it in the
                // lexical feature would make every claim about one subject
                // appear relevant to every goal for that subject.
                for text in [topic, value] {
                    for token in tokens(text) {
                        if goal_tokens.contains(&token) {
                            matched.insert(token);
                        }
                    }
                }
                SpecialistOutcome {
                    decision: matched.len().to_string(),
                    confidence: 100,
                    evidence: Verdict::Pass,
                    rationale: "deterministic lexical feature extraction".to_owned(),
                    input_units,
                    output_units: 1,
                }
            }
        };
        outcome.input_units = input_units;
        outcome.output_units = outcome.decision.len().max(outcome.output_units);
        outcome
    }
}

fn relation(relation: RelationHint, confidence: i64, rationale: &str) -> SpecialistOutcome {
    SpecialistOutcome {
        decision: format!("{relation:?}").to_ascii_uppercase(),
        confidence,
        evidence: Verdict::Pass,
        rationale: rationale.to_owned(),
        input_units: 0,
        output_units: 0,
    }
}

pub fn invocation(
    specialist: ProcessorRef,
    request: &SpecialistRequest,
    outcome: &SpecialistOutcome,
) -> SpecialistInvocation {
    let input = serde_json::to_string(request).unwrap_or_default();
    SpecialistInvocation {
        specialist,
        input_schema: "mncs-memory.specialist-request/0.1".to_owned(),
        input_digest: digest(&input),
        output: outcome.decision.clone(),
        confidence: outcome.confidence,
        evidence: outcome.evidence,
        input_units: outcome.input_units,
        output_units: outcome.output_units,
    }
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokens(value: &str) -> BTreeSet<String> {
    normalize(value)
        .split_whitespace()
        .filter(|token| token.len() > 1)
        .map(str::to_owned)
        .collect()
}

fn digest(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}
