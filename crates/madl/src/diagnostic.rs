//! What a reader reports: a stable code, where in the document, and a sentence.
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Stable, kebab-case: conformance fixtures and tools match on it.
    pub code: &'static str,
    pub severity: Severity,
    /// A JSON Pointer into the document (after `$ref` resolution), e.g. `/agents/2/groups/0`.
    pub path: String,
    pub message: String,
}

impl Diagnostic {
    pub fn error(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code, severity: Severity::Error, path: path.into(), message: message.into() }
    }

    pub fn warning(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code, severity: Severity::Warning, path: path.into(), message: message.into() }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let at = if self.path.is_empty() { "/" } else { &self.path };
        write!(f, "{level}[{}] {at}: {}", self.code, self.message)
    }
}

/// Everything found in one document.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn push(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error)
    }

    /// No errors (warnings are allowed).
    pub fn is_ok(&self) -> bool {
        self.errors().next().is_none()
    }
}
