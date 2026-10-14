//! Checks on the expanded graph: bans, starting points and loops.
use crate::diagnostic::{Diagnostic, Report};
use crate::graph::{self, Edge};
use crate::model::{DenyRule, Document};
use crate::resolve::Resolved;
use std::collections::{BTreeMap, BTreeSet};

pub fn check(doc: &Document, resolved: &Resolved, report: &mut Report) {
    denied(doc, resolved, report);
    let has_workers = doc.agents.iter().any(|a| a.runtime.as_deref() != Some("human"));
    let starts = graph::starting_agents(doc);
    if has_workers && starts.is_empty() {
        report.push(Diagnostic::error(
            "no-starting-point",
            "/agents",
            "no agent starts the work: give the system an input, or leave one agent that nothing hands work to",
        ));
    }
    loops(doc, &starts, report);
    let reached = graph::reachable(doc, &starts);
    for (i, s) in doc.steps.iter().enumerate() {
        if !reached.contains(&s.id) {
            report.push(Diagnostic::warning(
                "unreachable",
                format!("/steps/{i}"),
                format!("no work reaches step `{}`", s.id),
            ));
        }
    }
}

fn matches(doc: &Document, pattern: &str, agent: &str) -> bool {
    pattern == "*" || pattern == agent || graph::members(doc, pattern).iter().any(|m| m == agent)
}

/// Whether `rule` bans communication `from` → `to` of this kind.
pub fn bans(doc: &Document, rule: &DenyRule, e: &Edge) -> bool {
    if !rule.kinds.is_empty() && !rule.kinds.contains(&e.kind) {
        return false;
    }
    match (&rule.from, &rule.to, &rule.between) {
        (Some(f), Some(t), _) => matches(doc, f, &e.from) && matches(doc, t, &e.to),
        (_, _, Some([a, b])) => {
            (matches(doc, a, &e.from) && matches(doc, b, &e.to)) || (matches(doc, b, &e.from) && matches(doc, a, &e.to))
        }
        _ => false,
    }
}

/// A declared connection that a ban matches: the ban wins, and the document is wrong.
fn denied(doc: &Document, resolved: &Resolved, report: &mut Report) {
    let deny_of = |c: &Option<crate::model::Communication>| c.as_ref().map(|c| c.deny.clone()).unwrap_or_default();
    let effective: BTreeMap<&str, Vec<DenyRule>> =
        resolved.agents.iter().map(|a| (a.id.as_str(), deny_of(&a.settings.restrictions.communication))).collect();
    let root = deny_of(&doc.communication);
    let mut flagged = BTreeSet::new();
    for e in graph::edges(doc) {
        let own = effective.get(e.from.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        if root.iter().chain(own).any(|r| bans(doc, r, &e)) && flagged.insert(e.connection) {
            let c = &doc.connections[e.connection];
            let why = format!("`{}` → `{}` is banned by a `deny` rule; remove the connection or the ban", c.from, c.to);
            report.push(Diagnostic::error("connection-denied", format!("/connections/{}", e.connection), why));
        }
    }
}

/// A loop of work no starting agent reaches has no way in.
fn loops(doc: &Document, starts: &BTreeSet<String>, report: &mut Report) {
    let reached = graph::reachable(doc, starts);
    let work: Vec<Edge> = graph::edges(doc).into_iter().filter(|e| graph::carries_work(e.kind)).collect();
    for ring in cycles(&work) {
        if ring.iter().any(|n| reached.contains(n)) {
            continue;
        }
        let path = ring.join(" → ");
        let first = work.iter().find(|e| e.from == ring[0] && ring.contains(&e.to)).map_or(0, |e| e.connection);
        report.push(Diagnostic::error(
            "loop-without-start",
            format!("/connections/{first}"),
            format!("the loop {path} → {} has no way in: connect an agent outside it, or give it an input", ring[0]),
        ));
    }
}

/// Strongly connected groups of more than one node (Tarjan's algorithm, iterative enough for
/// documents of any realistic size).
fn cycles(edges: &[Edge]) -> Vec<Vec<String>> {
    let mut next: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in edges {
        next.entry(e.from.as_str()).or_default().push(e.to.as_str());
        next.entry(e.to.as_str()).or_default();
    }
    let mut t = Tarjan {
        next: &next,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        stack: Vec::new(),
        on: BTreeSet::new(),
        out: Vec::new(),
        counter: 0,
    };
    let nodes: Vec<&str> = next.keys().copied().collect();
    for n in nodes {
        if !t.index.contains_key(n) {
            t.visit(n);
        }
    }
    t.out
}

struct Tarjan<'a> {
    next: &'a BTreeMap<&'a str, Vec<&'a str>>,
    index: BTreeMap<&'a str, usize>,
    low: BTreeMap<&'a str, usize>,
    stack: Vec<&'a str>,
    on: BTreeSet<&'a str>,
    out: Vec<Vec<String>>,
    counter: usize,
}

impl<'a> Tarjan<'a> {
    fn visit(&mut self, n: &'a str) {
        self.index.insert(n, self.counter);
        self.low.insert(n, self.counter);
        self.counter += 1;
        self.stack.push(n);
        self.on.insert(n);
        for &m in self.next.get(n).into_iter().flatten() {
            if !self.index.contains_key(m) {
                self.visit(m);
                let low = self.low[n].min(self.low[m]);
                self.low.insert(n, low);
            } else if self.on.contains(m) {
                let low = self.low[n].min(self.index[m]);
                self.low.insert(n, low);
            }
        }
        if self.low[n] == self.index[n] {
            self.pop_component(n);
        }
    }

    fn pop_component(&mut self, root: &'a str) {
        let mut ring = Vec::new();
        while let Some(m) = self.stack.pop() {
            self.on.remove(m);
            ring.push(m.to_string());
            if m == root {
                break;
            }
        }
        if ring.len() > 1 {
            ring.reverse();
            self.out.push(ring);
        }
    }
}
