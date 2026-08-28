use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let language_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("language"));
    let output = args.next().map(PathBuf::from);
    let report = mncs_memory::benchmark::run(language_root)?;
    let json = serde_json::to_string_pretty(&report)?;
    if let Some(path) = output {
        std::fs::write(path, json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}
