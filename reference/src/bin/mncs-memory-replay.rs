use std::path::PathBuf;

use mncs_memory::{
    scenarios, DeterministicSpecialists, LanguageRuntime, MemoryEngine, MemoryStore,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let language_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("language"));
    let output = args.next().map(PathBuf::from);
    let corpus = scenarios::load();
    let specialists = DeterministicSpecialists::new(corpus.entities.clone())
        .faulty_alias_target(&corpus.replay.alias, &corpus.replay.faulty_target);
    let language = LanguageRuntime::new(language_root)?;
    let mut engine = MemoryEngine::with_store(
        language,
        MemoryStore::in_memory(),
        corpus.entities.clone(),
        Box::new(specialists),
    );
    let observation = corpus
        .observations
        .iter()
        .find(|observation| observation.id == corpus.replay.observation_id)
        .ok_or("replay observation missing")?
        .clone();
    engine.ingest(observation)?;
    let report = engine.replay_with(Box::new(DeterministicSpecialists::new(corpus.entities)))?;
    let json = serde_json::to_string_pretty(&report)?;
    if let Some(path) = output {
        std::fs::write(path, json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}
