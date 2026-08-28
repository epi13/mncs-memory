//! A bounded research implementation of semantic memory.
//!
//! The host owns append-oriented storage, provider orchestration, and the
//! translation between observations and typed MNCS values.  Decisions that
//! define memory semantics are executed from `language/mncs/memory/*.mncs`.
//! This is intentionally a local, deterministic proving ground rather than a
//! production memory service.

mod engine;
mod language;
mod model;
mod providers;
mod storage;

pub mod benchmark;
pub mod scenarios;

pub use engine::{BeliefResult, EngineError, IngestionReport, MemoryEngine};
pub use language::{BackendObservation, LanguageRuntime, SemanticError};
pub use model::*;
pub use providers::{
    DeterministicSpecialists, Specialist, SpecialistInvocation, SpecialistOutcome,
    SpecialistRequest,
};
pub use storage::{MemoryStore, StorageError};
