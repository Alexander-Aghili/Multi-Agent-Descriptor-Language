//! Fields that depend on a kind: steps, patterns and ban rules.
use crate::diagnostic::{Diagnostic, Report};
use crate::model::{Document, PatternKind, Step, StepKind};

pub fn check(doc: &Document, report: &mut Report) {
    for (i, step) in doc.steps.iter().enumerate() {
        step_fields(step, &format!("/steps/{i}"), report);
    }
    patterns(doc, report);
    let rules = doc.communication.iter().flat_map(|c| c.deny.iter().enumerate());
    for (i, rule) in rules {
        let pair = rule.from.is_some() && rule.to.is_some();
        let one_way = rule.from.is_some() || rule.to.is_some();
        if pair == rule.between.is_some() || (one_way && !pair) {
            report.push(Diagnostic::error(
                "deny-shape",
                format!("/communication/deny/{i}"),
                "a ban has either `from` and `to`, or `between`",
            ));
        }
    }
}

fn step_fields(step: &Step, at: &str, report: &mut Report) {
    let (needs, present): (&str, bool) = match step.kind {
        StepKind::Check => ("command", step.command.is_some()),
        StepKind::Human => ("prompt", step.prompt.is_some()),
        StepKind::FanOut => ("over", step.over.is_some()),
        StepKind::Merge | StepKind::FanIn => ("", true),
    };
    if !present {
        report.push(Diagnostic::error(
            "step-field",
            format!("{at}/{needs}"),
            format!("a `{}` step needs `{needs}`", kind_name(step.kind)),
        ));
    }
    stray_fields(step, at, report);
}

/// Fields set on a step whose kind does not use them.
fn stray_fields(step: &Step, at: &str, report: &mut Report) {
    let stray = [
        ("command", step.command.is_some(), StepKind::Check),
        ("prompt", step.prompt.is_some(), StepKind::Human),
        ("outcomes", !step.outcomes.is_empty(), StepKind::Human),
        ("into", step.into.is_some(), StepKind::Merge),
        ("over", step.over.is_some(), StepKind::FanOut),
        ("copies", step.copies.is_some(), StepKind::FanOut),
        ("join", step.join.is_some(), StepKind::FanIn),
    ];
    for (field, set, owner) in stray {
        if set && step.kind != owner {
            report.push(Diagnostic::error(
                "step-field",
                format!("{at}/{field}"),
                format!("`{field}` belongs to `{}` steps, not `{}`", kind_name(owner), kind_name(step.kind)),
            ));
        }
    }
}

fn kind_name(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Check => "check",
        StepKind::Human => "human",
        StepKind::Merge => "merge",
        StepKind::FanOut => "fan-out",
        StepKind::FanIn => "fan-in",
    }
}

fn patterns(doc: &Document, report: &mut Report) {
    let Some(t) = &doc.topology else { return };
    for (i, p) in t.patterns.iter().enumerate() {
        let at = format!("/topology/patterns/{i}");
        let problem = match p.pattern {
            PatternKind::Supervisor if p.supervisor.is_none() => Some("a `supervisor` pattern needs `supervisor`"),
            PatternKind::Auction if p.auctioneer.is_none() => Some("an `auction` pattern needs `auctioneer`"),
            PatternKind::Committee if p.source.is_none() => Some("a `committee` pattern needs `source`"),
            PatternKind::RolePlay if p.members.len() != 2 => Some("a `role-play` pattern has exactly two members"),
            PatternKind::Pipeline | PatternKind::Debate | PatternKind::Swarm if p.members.len() < 2 => {
                Some("this pattern needs at least two members")
            }
            _ if p.members.is_empty() => Some("a pattern needs members"),
            _ => None,
        };
        if let Some(message) = problem {
            report.push(Diagnostic::error("pattern-shape", at, message));
        }
    }
}
