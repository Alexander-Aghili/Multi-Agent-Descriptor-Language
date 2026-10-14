//! The conformance fixtures in `conformance/`: every implementation runs these.
use madl::Diagnostic;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance")
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(d) = dirs.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "yaml") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// What a reader reports for a file: a load error, or the validator's diagnostics.
fn diagnostics(file: &Path) -> Vec<Diagnostic> {
    match madl::load(file) {
        Err(e) => vec![e.0],
        Ok(doc) => madl::validate(&doc).diagnostics,
    }
}

#[test]
fn valid_documents_have_no_errors() {
    for file in yaml_files(&root().join("valid")) {
        let found = diagnostics(&file);
        assert!(found.is_empty(), "{}: {found:?}", file.display());
    }
}

#[test]
fn each_invalid_document_reports_its_error() {
    let files: Vec<PathBuf> = yaml_files(&root().join("invalid"))
        .into_iter()
        .filter(|f| f.with_extension("expected.json").exists())
        .collect();
    assert!(files.len() >= 40, "fixtures went missing");
    let mut failures = Vec::new();
    for file in files {
        let expected: Value =
            serde_json::from_str(&std::fs::read_to_string(file.with_extension("expected.json")).unwrap()).unwrap();
        let found = diagnostics(&file);
        let hit = found.iter().any(|d| d.code == expected["code"] && d.path == expected["path"]);
        if !hit {
            let got: Vec<String> = found.iter().map(|d| d.to_string()).collect();
            failures.push(format!(
                "{}: expected {} at {}, got {got:?}",
                file.display(),
                expected["code"],
                expected["path"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
