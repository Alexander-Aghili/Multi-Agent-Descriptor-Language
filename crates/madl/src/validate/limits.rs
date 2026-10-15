//! Limits (MEP-0002): every soft value at or below its hard value, durations that parse, plan
//! shares between 0 and 100 percent, and room for at least one working agent.
use crate::diagnostic::{Diagnostic, Report};
use crate::duration;
use crate::model::{Budget, Limit, PlanShare, RunBudget};

fn soft_above_hard(at: &str, report: &mut Report) {
    report.push(Diagnostic::error("budget-soft-above-hard", at, "the soft value is above the hard value"));
}

fn number<T: Copy + Into<f64>>(l: &Option<Limit<T>>, at: &str, report: &mut Report) {
    if l.as_ref().is_some_and(|l| l.soft().is_some_and(|s| (*s).into() > (*l.hard()).into())) {
        soft_above_hard(at, report);
    }
}

fn tokens(l: &Option<Limit<u64>>, at: &str, report: &mut Report) {
    if l.as_ref().is_some_and(|l| l.soft().is_some_and(|s| s > l.hard())) {
        soft_above_hard(at, report);
    }
}

/// Both values parse as durations, and soft is not the longer.
fn time(l: &Option<Limit<String>>, at: &str, report: &mut Report) {
    let Some(l) = l else { return };
    let hard = duration::seconds(l.hard());
    super::names::duration_of(Some(l.hard()), at, report);
    if let Some(s) = l.soft() {
        super::names::duration_of(Some(s), at, report);
        if duration::seconds(s).zip(hard).is_some_and(|(s, h)| s > h) {
            soft_above_hard(at, report);
        }
    }
}

pub fn budget(b: &Budget, at: &str, report: &mut Report) {
    tokens(&b.tokens, &format!("{at}/tokens"), report);
    tokens(&b.turns, &format!("{at}/turns"), report);
    time(&b.wall_clock, &format!("{at}/wall_clock"), report);
    time(&b.working_time, &format!("{at}/working_time"), report);
}

fn plan(p: &PlanShare, at: &str, report: &mut Report) {
    let ok = |v: f64| v.is_finite() && v > 0.0 && v <= 100.0;
    if !ok(p.hard) || p.soft.is_some_and(|s| !ok(s)) {
        report.push(Diagnostic::error(
            "budget-percent-out-of-range",
            at,
            "a plan share is a percent above 0 and at most 100",
        ));
    } else if p.soft.is_some_and(|s| s > p.hard) {
        soft_above_hard(at, report);
    }
}

pub fn run_budget(b: &RunBudget, at: &str, report: &mut Report) {
    for (i, p) in b.plan.iter().enumerate() {
        plan(p, &format!("{at}/plan/{i}"), report);
    }
    tokens(&b.tokens, &format!("{at}/tokens"), report);
    number(&b.cost_usd, &format!("{at}/cost_usd"), report);
    time(&b.wall_clock, &format!("{at}/wall_clock"), report);
    if b.max_working == Some(0) {
        report.push(Diagnostic::error(
            "budget-no-room",
            format!("{at}/max_working"),
            "at least one agent must be able to work",
        ));
    }
}
