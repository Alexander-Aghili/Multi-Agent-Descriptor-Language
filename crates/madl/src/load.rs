//! Reading a document: YAML or JSON, `$ref` resolution, then the typed model.
//!
//! A `$ref` is an object whose only key is `$ref`: `{ $ref: "./agents.yaml#/0" }`. A `.yaml`,
//! `.yml` or `.json` target is parsed (and its own `$ref`s resolved); any other file becomes its
//! text. A ref without a file part points into the same file. Refs may not leave the folder of
//! the document being loaded unless [`Options::allow_outside`] is set.
use crate::diagnostic::Diagnostic;
use crate::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Let `$ref` reach files outside the document's folder.
    pub allow_outside: bool,
}

/// Why a document could not be read. Each diagnostic has a stable code.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct LoadError(pub Diagnostic);

fn fail<T>(code: &'static str, path: &str, message: impl Into<String>) -> Result<T, LoadError> {
    Err(LoadError(Diagnostic::error(code, path, message)))
}

/// Read, resolve and type-check a document.
pub fn load(path: &Path) -> Result<Document, LoadError> {
    load_with(path, &Options::default())
}

pub fn load_with(path: &Path, options: &Options) -> Result<Document, LoadError> {
    let value = load_value(path, options)?;
    from_value(value)
}

/// Read and resolve a document without type-checking it.
pub fn load_value(path: &Path, options: &Options) -> Result<Value, LoadError> {
    let file = canonical(path, "")?;
    let root = file.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut resolver = Resolver { root, options: options.clone(), stack: Vec::new() };
    resolver.file(&file, "")
}

/// Parse text (YAML, which includes JSON) whose `$ref`s resolve against `base`.
pub fn from_str(text: &str, base: &Path, options: &Options) -> Result<Document, LoadError> {
    let value = parse(text, "<text>")?;
    let root = std::fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    let mut resolver = Resolver { root: root.clone(), options: options.clone(), stack: Vec::new() };
    let resolved = resolver.walk(value, &root.join("<text>"), &Value::Null, "")?;
    from_value(resolved)
}

/// Type-check a resolved value, reporting the exact path of the first mismatch.
pub fn from_value(value: Value) -> Result<Document, LoadError> {
    serde_path_to_error::deserialize(value).map_err(|e| {
        let pointer = pointer_of(e.path());
        LoadError(Diagnostic::error("shape", pointer, e.inner().to_string()))
    })
}

fn pointer_of(path: &serde_path_to_error::Path) -> String {
    use serde_path_to_error::Segment;
    let mut out = String::new();
    for segment in path.iter() {
        match segment {
            Segment::Seq { index } => out.push_str(&format!("/{index}")),
            Segment::Map { key } => out.push_str(&format!("/{}", escape(key))),
            Segment::Enum { .. } | Segment::Unknown => {}
        }
    }
    out
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn canonical(path: &Path, at: &str) -> Result<PathBuf, LoadError> {
    std::fs::canonicalize(path).or_else(|e| fail("ref-missing", at, format!("cannot open {}: {e}", path.display())))
}

fn parse(text: &str, name: &str) -> Result<Value, LoadError> {
    let mut yaml: serde_yaml_ng::Value = match serde_yaml_ng::from_str(text) {
        Ok(v) => v,
        Err(e) => return fail("parse", "", format!("{name}: {e}")),
    };
    if let Err(e) = yaml.apply_merge() {
        return fail("parse", "", format!("{name}: {e}"));
    }
    serde_json::to_value(yaml).or_else(|e| fail("parse", "", format!("{name}: {e}")))
}

fn is_structured(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("yaml" | "yml" | "json"))
}

struct Resolver {
    root: PathBuf,
    options: Options,
    /// The refs being resolved, outermost first: a repeat is a cycle.
    stack: Vec<String>,
}

impl Resolver {
    fn file(&mut self, file: &Path, at: &str) -> Result<Value, LoadError> {
        let text = std::fs::read_to_string(file)
            .or_else(|e| fail("ref-missing", at, format!("cannot read {}: {e}", file.display())))?;
        let value = parse(&text, &file.display().to_string())?;
        let whole = value.clone();
        self.walk(value, file, &whole, at)
    }

    fn walk(&mut self, value: Value, file: &Path, whole: &Value, at: &str) -> Result<Value, LoadError> {
        match value {
            Value::Object(map) if map.len() == 1 && map.get("$ref").is_some_and(Value::is_string) => {
                let target = map["$ref"].as_str().unwrap_or_default().to_string();
                self.reference(&target, file, whole, at)
            }
            Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    let inner = format!("{at}/{}", escape(&k));
                    out.insert(k, self.walk(v, file, whole, &inner)?);
                }
                Ok(Value::Object(out))
            }
            Value::Array(items) => {
                let walked =
                    items.into_iter().enumerate().map(|(i, v)| self.walk(v, file, whole, &format!("{at}/{i}")));
                walked.collect::<Result<Vec<_>, _>>().map(Value::Array)
            }
            other => Ok(other),
        }
    }

    fn reference(&mut self, target: &str, file: &Path, whole: &Value, at: &str) -> Result<Value, LoadError> {
        let (part, pointer) = target.split_once('#').unwrap_or((target, ""));
        let key = format!("{}#{pointer}", if part.is_empty() { file.display().to_string() } else { part.to_string() });
        if self.stack.contains(&key) {
            return fail("ref-cycle", at, format!("`{target}` refers back to itself"));
        }
        self.stack.push(key);
        let result = self.target(part, pointer, file, whole, at);
        self.stack.pop();
        result
    }

    fn target(&mut self, part: &str, pointer: &str, file: &Path, whole: &Value, at: &str) -> Result<Value, LoadError> {
        if part.is_empty() {
            let found = point(whole, pointer, at)?;
            return self.walk(found, file, whole, at);
        }
        let dir = file.parent().unwrap_or(Path::new("."));
        let path = canonical(&dir.join(part), at)?;
        if !self.options.allow_outside && !path.starts_with(&self.root) {
            return fail("ref-outside", at, format!("`{part}` is outside the document's folder"));
        }
        if !is_structured(&path) {
            return text_of(&path, part, pointer, at);
        }
        let loaded = self.file(&path, at)?;
        point(&loaded, pointer, at)
    }
}

/// A ref to a file that is not YAML or JSON: its text, which has no inside to point into.
fn text_of(path: &Path, part: &str, pointer: &str, at: &str) -> Result<Value, LoadError> {
    if !pointer.is_empty() {
        return fail("ref-pointer", at, format!("`{part}` is text; it has no `#{pointer}`"));
    }
    let text = std::fs::read_to_string(path)
        .or_else(|e| fail("ref-missing", at, format!("cannot read {}: {e}", path.display())))?;
    Ok(Value::String(text))
}

fn point(value: &Value, pointer: &str, at: &str) -> Result<Value, LoadError> {
    match value.pointer(pointer) {
        Some(found) => Ok(found.clone()),
        None => fail("ref-pointer", at, format!("nothing at `#{pointer}`")),
    }
}
