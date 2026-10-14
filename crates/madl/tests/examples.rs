//! Every example is a valid document.
use std::path::{Path, PathBuf};

fn examples() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut out = Vec::new();
    let mut dirs = vec![root];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.to_string_lossy().ends_with(".madl.yaml") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_example_is_valid() {
    let files = examples();
    assert!(files.len() >= 7, "examples went missing: {files:?}");
    let mut problems = Vec::new();
    for file in files {
        match madl::load(&file) {
            Err(e) => problems.push(format!("{}: {e}", file.display())),
            Ok(doc) => {
                for d in madl::validate(&doc).diagnostics {
                    problems.push(format!("{}: {d}", file.display()));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
