//! `madl`: validate, resolve, expand and draw Multi-Agent Descriptor Language documents.
use clap::{Parser, Subcommand};
use madl::{Diagnostic, Document, Options};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "madl", version, about = "Validate, resolve, expand and draw MADL documents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Let `$ref` reach files outside the document's folder.
    #[arg(long, global = true)]
    allow_outside: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Check documents against the schema and the spec's rules. Exits 1 on any error.
    Validate {
        files: Vec<PathBuf>,
        /// Print diagnostics as JSON lines.
        #[arg(long)]
        json: bool,
    },
    /// Print every agent's effective settings, with the source of each value.
    Resolve { file: PathBuf },
    /// Print the document with `$ref`s resolved and patterns expanded.
    Expand {
        file: PathBuf,
        /// Print JSON instead of YAML.
        #[arg(long)]
        json: bool,
    },
    /// Print a Mermaid flowchart of the system.
    Graph { file: PathBuf },
    /// Print the JSON Schema of a document.
    Schema,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = Options { allow_outside: cli.allow_outside };
    let result = match cli.command {
        Command::Validate { files, json } => validate(&files, json, &options),
        Command::Resolve { file } => checked(&file, &options).map(|doc| {
            let resolved = madl::resolve(&madl::expand(&doc));
            println!("{}", serde_json::to_string_pretty(&resolved).expect("resolved settings serialise"));
        }),
        Command::Expand { file, json } => checked(&file, &options).map(|doc| print_document(&madl::expand(&doc), json)),
        Command::Graph { file } => {
            checked(&file, &options).map(|doc| print!("{}", madl::mermaid::flowchart(&madl::expand(&doc))))
        }
        Command::Schema => {
            print!("{}", madl::schema::json());
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

/// Diagnostics for one file: its load error, or the validator's findings.
fn diagnose(file: &Path, options: &Options) -> (Option<Document>, Vec<Diagnostic>) {
    match madl::load_with(file, options) {
        Err(e) => (None, vec![e.0]),
        Ok(doc) => {
            let found = madl::validate(&doc).diagnostics;
            (Some(doc), found)
        }
    }
}

fn validate(files: &[PathBuf], json: bool, options: &Options) -> Result<(), ()> {
    let mut failed = false;
    for file in files {
        let (_, found) = diagnose(file, options);
        for d in &found {
            if json {
                let line = serde_json::json!({ "file": file, "code": d.code, "severity": d.severity, "path": d.path, "message": d.message });
                println!("{line}");
            } else {
                eprintln!("{}: {d}", file.display());
            }
        }
        let errors = found.iter().filter(|d| d.severity == madl::Severity::Error).count();
        failed |= errors > 0;
        if !json && errors == 0 {
            eprintln!("{}: valid", file.display());
        }
    }
    if failed { Err(()) } else { Ok(()) }
}

/// The document, if it has no errors; otherwise the errors are printed.
fn checked(file: &Path, options: &Options) -> Result<Document, ()> {
    let (doc, found) = diagnose(file, options);
    for d in &found {
        eprintln!("{}: {d}", file.display());
    }
    let ok = !found.iter().any(|d| d.severity == madl::Severity::Error);
    match doc {
        Some(doc) if ok => Ok(doc),
        _ => Err(()),
    }
}

fn print_document(doc: &Document, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(doc).expect("documents serialise"));
    } else {
        print!("{}", serde_yaml_ng::to_string(doc).expect("documents serialise"));
    }
}
