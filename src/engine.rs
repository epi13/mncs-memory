use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use thiserror::Error;

use crate::language::{LanguageRuntime, SemanticError};
use crate::model::*;
use crate::providers::{invocation, Specialist, SpecialistOutcome, SpecialistRequest};
use crate::storage::{digest, MemoryStore, StorageError};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Semantic(#[from] SemanticError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("specialist returned an invalid decision: {0}")]
    InvalidSpecialistDecision(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngestionReport {
    pub observation_identity: String,
    pub disposition: Disposition,
    pub claim_identity: Option<String>,
    pub escalation: Option<EscalationEvent>,
    pub specialist_invocations: Vec<ProcessorInvocationRecord>,
}

pub struct MemoryEngine {
    language: LanguageRuntime,
    store: MemoryStore,
    entities: BTreeMap<String, Entity>,
    specialists: Box<dyn Specialist>,
}

impl MemoryEngine {
    pub fn new(
        language_root: impl Into<std::path::PathBuf>,
        entities: impl IntoIterator<Item = Entity>,
    ) -> Result<Self, EngineError> {
        let entities: BTreeMap<String, Entity> = entities
            .into_iter()
            .map(|entity| (entity.identity.clone(), entity))
            .collect();
        let specialist_entities = entities.values().cloned().collect::<Vec<_>>();
        Ok(Self {
            language: LanguageRuntime::new(language_root)?,
            store: MemoryStore::in_memory(),
            entities,
            specialists: Box::new(crate::providers::DeterministicSpecialists::new(
                specialist_entities,
            )),
        })
    }

    pub fn with_store(
        language: LanguageRuntime,
        store: MemoryStore,
        entities: impl IntoIterator<Item = Entity>,
        specialists: Box<dyn Specialist>,
    ) -> Self {
        Self {
            language,
            store,
            entities: entities
                .into_iter()
                .map(|entity| (entity.identity.clone(), entity))
                .collect(),
            specialists,
        }
    }

    pub fn set_specialist(&mut self, specialist: Box<dyn Specialist>) {
        self.specialists = specialist;
    }

    pub fn language(&self) -> &LanguageRuntime {
        &self.language
    }

    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut MemoryStore {
        &mut self.store
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn ingest(&mut self, input: ObservationInput) -> Result<IngestionReport, EngineError> {
        let observation = Observation {
            identity: format!("observation:{}", input.id),
            immutable_digest: digest(&input),
            input: input.clone(),
        };
        self.store.append_observation(observation.clone())?;
        let mut invocations = Vec::new();

        let (worthiness, invocation) = self.run_specialist(SpecialistRequest::Worthiness {
            observation: input.clone(),
        })?;
        invocations.push(invocation);
        if worthiness.decision == "NO" {
            let record = IngestionRecord {
                observation,
                disposition: Disposition::Unknown,
                claim: None,
                processor_invocations: invocations.clone(),
                escalation: None,
            };
            self.store.append_ingestion(record)?;
            return Ok(IngestionReport {
                observation_identity: format!("observation:{}", input.id),
                disposition: Disposition::Unknown,
                claim_identity: None,
                escalation: None,
                specialist_invocations: invocations,
            });
        }

        let (entity_outcome, invocation) =
            self.run_specialist(SpecialistRequest::EntityResolution {
                alias: input.subject_alias.clone(),
            })?;
        invocations.push(invocation);
        let Some(entity) = self.entities.get(&entity_outcome.decision).cloned() else {
            let escalation = self.record_escalation(
                "entity-resolver",
                "deterministic",
                &entity_outcome,
                "entity identity is unresolved; no raw store was exposed",
            )?;
            let record = IngestionRecord {
                observation,
                disposition: Disposition::Unknown,
                claim: None,
                processor_invocations: invocations.clone(),
                escalation: Some(escalation.clone()),
            };
            self.store.append_ingestion(record)?;
            return Ok(IngestionReport {
                observation_identity: format!("observation:{}", input.id),
                disposition: Disposition::Unknown,
                claim_identity: None,
                escalation: Some(escalation),
                specialist_invocations: invocations,
            });
        };

        let existing = self
            .store
            .claims()
            .filter(|claim| {
                claim.subject == entity.identity
                    && claim.state != ClaimState::Invalidated
                    && claim.state != ClaimState::Superseded
                    && claim.state != ClaimState::Refined
                    && claim.state != ClaimState::Duplicate
            })
            .cloned()
            .collect::<Vec<_>>();
        let (relationship, invocation) = self.run_specialist(SpecialistRequest::Relationship {
            observation: input.clone(),
            existing: existing.clone(),
        })?;
        invocations.push(invocation);
        let relation = relation_from_name(&relationship.decision)?;
        let disposition = self.language.ingestion(relation, input.evidence)?;
        let escalation = if relationship.confidence < 85 || disposition == Disposition::Unknown {
            Some(self.record_escalation(
                "ingestion-policy",
                "deterministic",
                &relationship,
                "relationship/evidence did not meet the MNCS policy threshold",
            )?)
        } else {
            None
        };

        let related = related_claims(&existing, &input.topic, &input.value, disposition);
        let processor = self.specialists.processor();
        let claim = Claim {
            identity: format!(
                "claim:{}",
                digest(&(
                    observation.identity.clone(),
                    entity.identity.clone(),
                    disposition,
                    &processor
                ))
            ),
            subject: entity.identity.clone(),
            topic: input.topic.clone(),
            value: input.value.clone(),
            kind: input.kind,
            state: state_for(disposition),
            disposition,
            confidence: input.evidence.score().min(relationship.confidence),
            evidence: input.evidence,
            valid_from: input.observed_at,
            valid_until: None,
            derived_from: vec![observation.identity.clone()],
            processor,
            supersedes: if disposition == Disposition::Supersession {
                related.iter().map(|claim| claim.identity.clone()).collect()
            } else {
                Vec::new()
            },
            refines: if disposition == Disposition::Refinement {
                related.iter().map(|claim| claim.identity.clone()).collect()
            } else {
                Vec::new()
            },
            contradicts: if disposition == Disposition::Contradiction {
                related.iter().map(|claim| claim.identity.clone()).collect()
            } else {
                Vec::new()
            },
            created_at: input.observed_at,
            last_confirmed: input.observed_at,
        };

        if disposition == Disposition::Refinement {
            for prior in &related {
                self.store
                    .replace_claim_state(&prior.identity, ClaimState::Refined)?;
            }
        }
        if disposition == Disposition::Supersession {
            for prior in &related {
                self.store
                    .replace_claim_state(&prior.identity, ClaimState::Superseded)?;
            }
        }
        self.store.append_claim(claim.clone())?;
        let record = IngestionRecord {
            observation,
            disposition,
            claim: Some(claim.clone()),
            processor_invocations: invocations.clone(),
            escalation: escalation.clone(),
        };
        self.store.append_ingestion(record)?;
        if let Some(event) = &escalation {
            // The event was appended by record_escalation; this branch keeps
            // the relationship obvious for callers inspecting the report.
            debug_assert!(!event.identity.is_empty());
        }
        Ok(IngestionReport {
            observation_identity: format!("observation:{}", input.id),
            disposition,
            claim_identity: Some(claim.identity),
            escalation,
            specialist_invocations: invocations,
        })
    }

    pub fn query(&mut self, request: SemanticRequest) -> Result<AnswerCapsule, EngineError> {
        let (_route, route_invocation) = self.run_specialist(SpecialistRequest::QueryRoute {
            intent: request.intent,
        })?;

        if request.raw_episodes || request.intent == QueryIntent::RawEpisodes {
            return self.raw_episode_capsule(&request, route_invocation);
        }

        let mut candidates = self
            .store
            .claims()
            .filter(|claim| is_query_candidate(claim, &request))
            .cloned()
            .collect::<Vec<_>>();
        let source_claims_exposed = candidates.len();
        let mut scored = Vec::new();
        for claim in candidates.drain(..) {
            let (relevance, invocation) = self.run_specialist(SpecialistRequest::Relevance {
                goal: request.goal.clone(),
                subject: claim.subject.clone(),
                topic: claim.topic.clone(),
                value: claim.value.clone(),
            })?;
            let lexical = relevance.decision.parse::<i64>().unwrap_or(0);
            if !request.required_topics.is_empty()
                && !request
                    .required_topics
                    .iter()
                    .any(|topic| topic == &claim.topic)
                && lexical == 0
            {
                continue;
            }
            let topic_match =
                i64::from(request.topic.as_deref() == Some(claim.topic.as_str())) + lexical;
            let subject_match =
                i64::from(request.subject.as_deref() == Some(claim.subject.as_str()));
            let freshness = freshness_score(&claim, &request);
            let score =
                self.language
                    .score(topic_match, subject_match, claim.confidence, freshness)?;
            scored.push((score, claim, invocation));
        }
        scored.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.identity.cmp(&right.1.identity))
        });
        let conflict = has_conflict(&scored);
        let mut uncertainty = Vec::new();
        let escalation = if conflict && !request.include_conflicts {
            uncertainty.push("multiple active values remain unresolved".to_owned());
            self.record_query_escalation("contradiction-arbitrator", "compact conflict set")?
        } else if scored.is_empty() {
            uncertainty.push("no eligible current claim met the query filters".to_owned());
            self.record_query_escalation("query-specialist", "bounded query features")?
        } else {
            EscalationDecision::None
        };

        let mut selected = select_claims(scored, &request);
        if conflict && !request.include_conflicts {
            selected.clear();
        }
        let items = selected
            .iter()
            .map(|claim| answer_item(claim, &request))
            .collect::<Vec<_>>();
        let answer = render_items(&items);
        let required_topics_preserved = request
            .required_topics
            .iter()
            .all(|topic| items.iter().any(|item| item.topic == *topic));
        let required_units = items
            .iter()
            .filter(|item| {
                request
                    .required_topics
                    .iter()
                    .any(|topic| topic == &item.topic)
            })
            .map(item_units)
            .sum::<usize>();
        let budget_status = self.language.budget(
            answer.len() as i64,
            request.max_units as i64,
            items.len() as i64,
        )?;
        let mut status = if selected.is_empty() || !required_topics_preserved {
            AnswerStatus::Unknown
        } else {
            let aggregate_confidence = selected
                .iter()
                .map(|claim| claim.confidence)
                .min()
                .unwrap_or(0);
            let freshness = selected
                .iter()
                .map(|claim| freshness_score(claim, &request))
                .min()
                .unwrap_or(0);
            self.language.answer(
                aggregate_confidence,
                request.minimum_confidence,
                freshness,
                i64::from(request.freshness_required.is_some()),
            )?
        };
        if budget_status != BudgetStatus::Fits || required_units > request.max_units {
            status = AnswerStatus::Unknown;
            uncertainty.push("answer budget cannot preserve all selected information".to_owned());
        }
        if conflict && request.include_conflicts {
            status = AnswerStatus::Unknown;
            uncertainty.push("conflict set returned for deliberate inspection".to_owned());
        }
        if request.include_uncertainty
            && selected
                .iter()
                .any(|claim| claim.state == ClaimState::Unknown)
        {
            uncertainty.push("one or more candidate claims carry UNKNOWN state".to_owned());
        }
        let answer = if status == AnswerStatus::Unknown
            && conflict
            && request.include_conflicts
            && budget_status == BudgetStatus::Fits
        {
            answer
        } else {
            fit_status_answer(status, &answer, request.max_units)
        };
        let units = answer.len();
        Ok(AnswerCapsule {
            schema_version: "mncs-memory.answer-capsule/0.1".to_owned(),
            status,
            answer,
            units,
            items,
            budget: request.max_units,
            required_topics_preserved,
            candidates_exposed: source_claims_exposed,
            source_claims_exposed,
            escalation,
            uncertainty,
            provenance_included: request.provenance_requested,
        })
    }

    pub fn current_belief(&self, subject: &str, topic: &str) -> BeliefResult {
        let claims = self
            .store
            .claims()
            .filter(|claim| {
                claim.subject == subject
                    && claim.topic == topic
                    && matches!(claim.state, ClaimState::Current | ClaimState::Unresolved)
            })
            .cloned()
            .collect::<Vec<_>>();
        let distinct_values = claims
            .iter()
            .map(|claim| &claim.value)
            .collect::<BTreeSet<_>>();
        let status = if claims.is_empty() || distinct_values.len() > 1 {
            AnswerStatus::Unknown
        } else {
            AnswerStatus::Pass
        };
        BeliefResult { status, claims }
    }

    pub fn replay_with(
        &mut self,
        replacement: Box<dyn Specialist>,
    ) -> Result<ReplayReport, EngineError> {
        let old_version = self
            .store
            .claims()
            .next()
            .map(|claim| claim.processor.version.clone())
            .unwrap_or_else(|| "none".to_owned());
        let affected = self
            .store
            .claims()
            .filter(|claim| {
                claim.processor.version == old_version && claim.state != ClaimState::Invalidated
            })
            .map(|claim| claim.identity.clone())
            .collect::<Vec<_>>();
        for identity in &affected {
            self.store
                .replace_claim_state(identity, ClaimState::Invalidated)?;
        }
        let new_version = replacement.processor().version;
        self.specialists = replacement;
        let observations = self.store.observations().to_vec();
        for observation in observations {
            self.ingest(observation.input)?;
        }
        let recomputed = self
            .store
            .claims()
            .filter(|claim| {
                claim.processor.version == new_version && claim.state != ClaimState::Invalidated
            })
            .map(|claim| claim.identity.clone())
            .collect::<Vec<_>>();
        Ok(ReplayReport {
            processor_role: "deterministic-specialists".to_owned(),
            old_version,
            new_version,
            affected_claims: affected.clone(),
            invalidated_claims: affected,
            recomputed_claims: recomputed,
            unchanged_observations: self.store.observations().len(),
        })
    }

    fn run_specialist(
        &mut self,
        request: SpecialistRequest,
    ) -> Result<(SpecialistOutcome, ProcessorInvocationRecord), EngineError> {
        let outcome = self.specialists.invoke(&request);
        let invocation = invocation(self.specialists.processor(), &request, &outcome);
        self.store.append_invocation(invocation.clone())?;
        Ok((outcome, invocation))
    }

    fn record_escalation(
        &mut self,
        requested_by: &str,
        from_layer: &str,
        outcome: &SpecialistOutcome,
        exposed_context: &str,
    ) -> Result<EscalationEvent, EngineError> {
        let decision = self
            .language
            .escalation(&from_layer.to_ascii_uppercase(), outcome.confidence)?;
        let (to_layer, resolution) = match decision {
            EscalationDecision::None => ("none", "resolved at current layer"),
            EscalationDecision::Specialist => {
                ("specialist", "UNKNOWN; stronger specialist unavailable")
            }
            EscalationDecision::GeneralReasoner => (
                "general-reasoner",
                "UNKNOWN; caller must decide whether to escalate",
            ),
            EscalationDecision::Unknown => ("unknown", "UNKNOWN; escalation authority exhausted"),
        };
        let event = EscalationEvent {
            identity: format!(
                "escalation:{}",
                digest(&(requested_by, outcome, exposed_context))
            ),
            requested_by: requested_by.to_owned(),
            from_layer: from_layer.to_owned(),
            to_layer: to_layer.to_owned(),
            uncertainty: outcome.rationale.clone(),
            exposed_context: exposed_context.to_owned(),
            resolution: resolution.to_owned(),
        };
        self.store.append_escalation(event.clone())?;
        Ok(event)
    }

    fn record_query_escalation(
        &mut self,
        requested_by: &str,
        exposed_context: &str,
    ) -> Result<EscalationDecision, EngineError> {
        let outcome = SpecialistOutcome {
            decision: "UNKNOWN".to_owned(),
            confidence: 60,
            evidence: Verdict::Unknown,
            rationale: "query arbitration requires unresolved context".to_owned(),
            input_units: exposed_context.len(),
            output_units: 7,
        };
        let event =
            self.record_escalation(requested_by, "SPECIALIST", &outcome, exposed_context)?;
        Ok(match event.to_layer.as_str() {
            "general-reasoner" => EscalationDecision::GeneralReasoner,
            "specialist" => EscalationDecision::Specialist,
            "none" => EscalationDecision::None,
            _ => EscalationDecision::Unknown,
        })
    }

    fn raw_episode_capsule(
        &self,
        request: &SemanticRequest,
        _route_invocation: ProcessorInvocationRecord,
    ) -> Result<AnswerCapsule, EngineError> {
        let observations = self
            .store
            .observations()
            .iter()
            .filter(|observation| {
                request.subject.as_deref().is_none_or(|subject| {
                    observation.input.subject_alias == subject
                        || observation.input.text.contains(subject)
                }) && request
                    .topic
                    .as_deref()
                    .is_none_or(|topic| observation.input.topic == topic)
            })
            .collect::<Vec<_>>();
        let answer = observations
            .iter()
            .map(|observation| observation.input.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let fits = self.language.budget(
            answer.len() as i64,
            request.max_units as i64,
            observations.len() as i64,
        )? == BudgetStatus::Fits;
        let status = if fits {
            AnswerStatus::Pass
        } else {
            AnswerStatus::Unknown
        };
        let answer = fit_status_answer(status, &answer, request.max_units);
        let units = answer.len();
        Ok(AnswerCapsule {
            schema_version: "mncs-memory.answer-capsule/0.1".to_owned(),
            status,
            answer,
            units,
            budget: request.max_units,
            required_topics_preserved: fits,
            candidates_exposed: observations.len(),
            source_claims_exposed: 0,
            items: Vec::new(),
            escalation: EscalationDecision::None,
            uncertainty: if fits {
                Vec::new()
            } else {
                vec!["raw episode delivery exceeded budget".to_owned()]
            },
            provenance_included: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BeliefResult {
    pub status: AnswerStatus,
    pub claims: Vec<Claim>,
}

fn relation_from_name(name: &str) -> Result<RelationHint, EngineError> {
    match name {
        "UNRELATED" => Ok(RelationHint::Unrelated),
        "SAME" => Ok(RelationHint::Same),
        "REFINES" => Ok(RelationHint::Refines),
        "CONTRADICTS" => Ok(RelationHint::Contradicts),
        "SUPERSEDES" => Ok(RelationHint::Supersedes),
        "AMBIGUOUS" | "UNKNOWN" => Ok(RelationHint::Ambiguous),
        other => Err(EngineError::InvalidSpecialistDecision(other.to_owned())),
    }
}

fn state_for(disposition: Disposition) -> ClaimState {
    match disposition {
        Disposition::New | Disposition::Refinement | Disposition::Supersession => {
            ClaimState::Current
        }
        Disposition::Duplicate => ClaimState::Duplicate,
        Disposition::Contradiction => ClaimState::Unresolved,
        Disposition::Unknown => ClaimState::Unknown,
    }
}

fn related_claims<'a>(
    claims: &'a [Claim],
    topic: &str,
    value: &str,
    disposition: Disposition,
) -> Vec<&'a Claim> {
    claims
        .iter()
        .filter(|claim| {
            claim.topic == topic
                && match disposition {
                    Disposition::Duplicate => claim.value == value,
                    Disposition::Contradiction => claim.value != value,
                    Disposition::Refinement | Disposition::Supersession => true,
                    _ => false,
                }
        })
        .collect()
}

fn is_query_candidate(claim: &Claim, request: &SemanticRequest) -> bool {
    if !matches!(claim.state, ClaimState::Current | ClaimState::Unresolved) {
        return false;
    }
    if claim.confidence < request.minimum_confidence && !request.include_uncertainty {
        return false;
    }
    if request
        .subject
        .as_deref()
        .is_some_and(|subject| subject != claim.subject)
    {
        return false;
    }
    if request
        .topic
        .as_deref()
        .is_some_and(|topic| topic != claim.topic)
    {
        return false;
    }
    if request
        .forbidden_topics
        .iter()
        .any(|topic| topic == &claim.topic)
    {
        return false;
    }
    if request
        .freshness_required
        .is_some_and(|required| claim.valid_from < request.as_of - required)
    {
        return false;
    }
    true
}

fn freshness_score(claim: &Claim, request: &SemanticRequest) -> i64 {
    match request.freshness_required {
        Some(required) if required > 0 => {
            let age = request.as_of.saturating_sub(claim.valid_from);
            if age <= required {
                100
            } else {
                0
            }
        }
        _ => 100,
    }
}

fn has_conflict(scored: &[(i64, Claim, ProcessorInvocationRecord)]) -> bool {
    let mut values = BTreeMap::<(&str, &str), BTreeSet<&str>>::new();
    for (_, claim, _) in scored {
        values
            .entry((&claim.subject, &claim.topic))
            .or_default()
            .insert(&claim.value);
    }
    values.values().any(|values| values.len() > 1)
}

fn select_claims(
    scored: Vec<(i64, Claim, ProcessorInvocationRecord)>,
    request: &SemanticRequest,
) -> Vec<Claim> {
    let mut selected = Vec::new();
    for (_, claim, _) in &scored {
        if request
            .required_topics
            .iter()
            .any(|topic| topic == &claim.topic)
            && !selected
                .iter()
                .any(|selected: &Claim| selected.topic == claim.topic)
        {
            selected.push(claim.clone());
        }
    }
    for (_, claim, _) in scored {
        if selected.len() >= request.max_supporting_claims {
            break;
        }
        if !selected
            .iter()
            .any(|selected: &Claim| selected.identity == claim.identity)
        {
            selected.push(claim);
        }
    }
    selected
}

fn answer_item(claim: &Claim, request: &SemanticRequest) -> AnswerItem {
    AnswerItem {
        claim_identity: claim.identity.clone(),
        subject: claim.subject.clone(),
        topic: claim.topic.clone(),
        value: claim.value.clone(),
        confidence: claim.confidence,
        valid_from: claim.valid_from,
        stale: freshness_score(claim, request) == 0,
        supporting_claims: if request.provenance_requested {
            claim
                .supersedes
                .iter()
                .chain(claim.refines.iter())
                .chain(claim.contradicts.iter())
                .cloned()
                .collect()
        } else {
            Vec::new()
        },
        supporting_observations: if request.provenance_requested {
            claim.derived_from.clone()
        } else {
            Vec::new()
        },
    }
}

fn item_units(item: &AnswerItem) -> usize {
    item.subject.len() + item.topic.len() + item.value.len() + 3
}

fn render_items(items: &[AnswerItem]) -> String {
    items
        .iter()
        .map(|item| format!("{}={}", item.topic, item.value))
        .collect::<Vec<_>>()
        .join(";")
}

fn fit_status_answer(status: AnswerStatus, answer: &str, budget: usize) -> String {
    if status == AnswerStatus::Pass && answer.len() <= budget {
        return answer.to_owned();
    }
    let marker = match status {
        AnswerStatus::Pass => "UNKNOWN",
        AnswerStatus::Unknown => "UNKNOWN",
        AnswerStatus::Fail => "FAIL",
    };
    marker.chars().take(budget).collect()
}
