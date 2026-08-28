use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::model::{
    Claim, EscalationEvent, IngestionRecord, Observation, ProcessorInvocationRecord,
};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage record is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    root: Option<PathBuf>,
    observations: Vec<Observation>,
    claims: BTreeMap<String, Claim>,
    ingestions: Vec<IngestionRecord>,
    escalations: Vec<EscalationEvent>,
    invocations: Vec<ProcessorInvocationRecord>,
}

impl MemoryStore {
    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let mut store = Self {
            root: Some(root.clone()),
            ..Self::default()
        };
        store.observations = read_jsonl(root.join("observations.jsonl"))?;
        for claim in read_jsonl::<Claim>(root.join("claims.jsonl"))? {
            store.claims.insert(claim.identity.clone(), claim);
        }
        store.ingestions = read_jsonl(root.join("ingestions.jsonl"))?;
        store.escalations = read_jsonl(root.join("escalations.jsonl"))?;
        store.invocations = read_jsonl(root.join("specialist-invocations.jsonl"))?;
        Ok(store)
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    pub fn claims(&self) -> impl Iterator<Item = &Claim> {
        self.claims.values()
    }

    pub fn ingestions(&self) -> &[IngestionRecord] {
        &self.ingestions
    }

    pub fn escalations(&self) -> &[EscalationEvent] {
        &self.escalations
    }

    pub fn invocations(&self) -> &[ProcessorInvocationRecord] {
        &self.invocations
    }

    pub(crate) fn append_observation(
        &mut self,
        observation: Observation,
    ) -> Result<(), StorageError> {
        if self
            .observations
            .iter()
            .any(|existing| existing.identity == observation.identity)
        {
            return Ok(());
        }
        append_jsonl(self.root.as_deref(), "observations.jsonl", &observation)?;
        self.observations.push(observation);
        Ok(())
    }

    pub(crate) fn append_claim(&mut self, claim: Claim) -> Result<(), StorageError> {
        append_jsonl(self.root.as_deref(), "claims.jsonl", &claim)?;
        self.claims.insert(claim.identity.clone(), claim);
        Ok(())
    }

    pub(crate) fn append_ingestion(&mut self, record: IngestionRecord) -> Result<(), StorageError> {
        append_jsonl(self.root.as_deref(), "ingestions.jsonl", &record)?;
        self.ingestions.push(record);
        Ok(())
    }

    pub(crate) fn append_escalation(&mut self, event: EscalationEvent) -> Result<(), StorageError> {
        append_jsonl(self.root.as_deref(), "escalations.jsonl", &event)?;
        self.escalations.push(event);
        Ok(())
    }

    pub(crate) fn append_invocation(
        &mut self,
        invocation: ProcessorInvocationRecord,
    ) -> Result<(), StorageError> {
        append_jsonl(
            self.root.as_deref(),
            "specialist-invocations.jsonl",
            &invocation,
        )?;
        self.invocations.push(invocation);
        Ok(())
    }

    pub(crate) fn replace_claim_state(
        &mut self,
        identity: &str,
        state: crate::model::ClaimState,
    ) -> Result<Option<Claim>, StorageError> {
        let Some(mut claim) = self.claims.get(identity).cloned() else {
            return Ok(None);
        };
        claim.state = state;
        self.append_claim(claim.clone())?;
        Ok(Some(claim))
    }
}

fn append_jsonl<T: Serialize>(
    root: Option<&Path>,
    file_name: &str,
    value: &T,
) -> Result<(), StorageError> {
    let Some(root) = root else {
        return Ok(());
    };
    fs::create_dir_all(root)?;
    let path = root.join(file_name);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn read_jsonl<T: DeserializeOwned>(path: PathBuf) -> Result<Vec<T>, StorageError> {
    let Ok(file) = File::open(path) else {
        return Ok(Vec::new());
    };
    BufReader::new(file)
        .lines()
        .filter(|line| line.as_ref().map_or(true, |line| !line.trim().is_empty()))
        .map(|line| Ok(serde_json::from_str(&line?)?))
        .collect()
}

pub(crate) fn digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("memory model is serializable");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}
