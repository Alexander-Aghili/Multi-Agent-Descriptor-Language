//! `validate`: everything a Reader must check beyond the schema (spec §11).
mod flow;
mod index;
mod limits;
mod names;
mod refs;
mod shape;

pub use flow::bans;
pub use names::{MEDIUMS, RUNTIMES, is_protocol};

use crate::diagnostic::Report;
use crate::model::Document;

/// Check a loaded document. Patterns are expanded first, so graph rules see every connection.
pub fn validate(doc: &Document) -> Report {
    let mut report = Report::default();
    let index = index::Index::build(doc);
    refs::check(doc, &index, &mut report);
    names::check(doc, &mut report);
    if let Some(b) = &doc.system.budget {
        limits::run_budget(b, "/system/budget", &mut report);
    }
    shape::check(doc, &mut report);
    if !report.is_ok() {
        return report;
    }
    let expanded = crate::expand::expand(doc);
    let resolved = crate::resolve::resolve(&expanded);
    for d in &resolved.diagnostics {
        report.push(d.clone());
    }
    flow::check(&expanded, &resolved, &mut report);
    report
}
