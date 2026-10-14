//! Resolver and expander fixtures: the output must match the checked-in JSON exactly.
//! Regenerate with `UPDATE_FIXTURES=1 cargo test`, then review every change by hand.
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn files(dir: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance").join(dir);
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    out.sort();
    out
}

fn compare(file: &Path, suffix: &str, actual: &Value) {
    let path = file.with_extension(suffix);
    let mut text = serde_json::to_string_pretty(actual).unwrap();
    text.push('\n');
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &text).unwrap();
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(on_disk == text, "{} differs from the resolver:\n{text}", path.display());
}

#[test]
fn effective_settings_match_the_fixtures() {
    for file in files("resolve") {
        let doc = madl::load(&file).unwrap();
        assert!(madl::validate(&doc).is_ok(), "{} should be valid", file.display());
        let resolved = madl::resolve(&madl::expand(&doc));
        compare(&file, "resolved.json", &serde_json::to_value(&resolved).unwrap());
    }
}

#[test]
fn expanded_patterns_match_the_fixtures() {
    for file in files("expand") {
        let doc = madl::expand(&madl::load(&file).unwrap());
        let connections: Vec<Value> = doc
            .connections
            .iter()
            .map(|c| json!([c.from, madl::expand::kind_name(c.kind), c.to, c.pattern, c.max_rounds, c.via]))
            .collect();
        let channels: Vec<Value> = doc.channels.iter().map(|c| json!([c.id, c.medium, c.access])).collect();
        compare(&file, "expanded.json", &json!({ "connections": connections, "channels": channels }));
    }
}
