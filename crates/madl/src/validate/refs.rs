//! Ids are well formed and unique, and every reference names something of the right kind.
use super::index::{Index, Kind};
use crate::diagnostic::{Diagnostic, Report};
use crate::model::Document;

pub fn check(doc: &Document, index: &Index, report: &mut Report) {
    if !crate::REVISIONS.contains(&doc.madl.as_str()) {
        let known = crate::REVISIONS.join(", ");
        report.push(Diagnostic::error(
            "unknown-revision",
            "/madl",
            format!("`{}` is not a revision this reader knows ({known})", doc.madl),
        ));
    }
    well_formed(&doc.system.id, "/system/id", report);
    for (id, (_, at)) in &index.ids {
        well_formed(id, at, report);
    }
    for (id, at, first) in &index.duplicates {
        report.push(Diagnostic::error("duplicate-id", at, format!("`{id}` is already used at {first}")));
    }
    let mut r = Refs { index, report };
    r.groups(doc);
    r.connections(doc);
    r.inputs_and_channels(doc);
    r.policies(doc);
    r.patterns(doc);
}

/// Letters, digits, `-` and `_`, starting with a letter or digit, at most 64 characters.
pub fn well_formed(id: &str, at: &str, report: &mut Report) {
    let mut chars = id.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_alphanumeric());
    let rest_ok = id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !first_ok || !rest_ok || id.len() > 64 {
        report.push(Diagnostic::error(
            "bad-id",
            at,
            format!("`{id}` is not a valid id: use letters, digits, `-` and `_`, starting with a letter or digit"),
        ));
    }
}

struct Refs<'a> {
    index: &'a Index,
    report: &'a mut Report,
}

impl Refs<'_> {
    /// `id` must name one of `kinds` (`*` too, when `star`).
    fn expect(&mut self, id: &str, at: String, kinds: &[Kind], star: bool) {
        if star && id == "*" {
            return;
        }
        let wanted: Vec<&str> = kinds.iter().map(|k| k.noun()).collect();
        match self.index.kind(id) {
            None => self.report.push(Diagnostic::error(
                "unknown-reference",
                at,
                format!("`{id}` is not defined; expected {}", wanted.join(" or ")),
            )),
            Some(k) if !kinds.contains(&k) => self.report.push(Diagnostic::error(
                "wrong-kind",
                at,
                format!("`{id}` is {}; expected {}", k.noun(), wanted.join(" or ")),
            )),
            Some(_) => {}
        }
    }

    fn groups(&mut self, doc: &Document) {
        for (i, a) in doc.agents.iter().enumerate() {
            for (j, g) in a.groups.iter().enumerate() {
                self.expect(g, format!("/agents/{i}/groups/{j}"), &[Kind::Group], false);
            }
        }
        for (i, g) in doc.groups.iter().enumerate() {
            let Some(parent) = &g.parent else { continue };
            self.expect(parent, format!("/groups/{i}/parent"), &[Kind::Group], false);
            let chain = crate::resolve::ancestors(doc, parent);
            if chain.contains(&i) {
                self.report.push(Diagnostic::error(
                    "group-cycle",
                    format!("/groups/{i}/parent"),
                    format!("group `{}` is inside itself", g.id),
                ));
            }
        }
    }

    fn connections(&mut self, doc: &Document) {
        let nodes = [Kind::Agent, Kind::Group, Kind::Step];
        for (i, c) in doc.connections.iter().enumerate() {
            self.expect(&c.from, format!("/connections/{i}/from"), &nodes, false);
            self.expect(&c.to, format!("/connections/{i}/to"), &nodes, false);
            if let Some(via) = &c.via {
                self.expect(via, format!("/connections/{i}/via"), &[Kind::Channel], false);
            }
            if c.from == c.to {
                self.report.push(Diagnostic::error(
                    "self-connection",
                    format!("/connections/{i}"),
                    format!("`{}` is connected to itself", c.from),
                ));
            }
        }
    }

    fn inputs_and_channels(&mut self, doc: &Document) {
        for (i, input) in doc.inputs.iter().enumerate() {
            for (j, to) in input.to.iter().enumerate() {
                self.expect(to, format!("/inputs/{i}/to/{j}"), &[Kind::Agent, Kind::Group, Kind::Step], false);
            }
        }
        for (i, ch) in doc.channels.iter().enumerate() {
            for member in ch.access.keys() {
                self.expect(member, format!("/channels/{i}/access/{member}"), &[Kind::Agent, Kind::Group], false);
            }
        }
        let rules = doc.communication.iter().flat_map(|c| c.deny.iter().enumerate());
        for (i, rule) in rules {
            let at = format!("/communication/deny/{i}");
            let ids = rule.from.iter().chain(rule.to.iter()).chain(rule.between.iter().flatten());
            for id in ids.cloned().collect::<Vec<_>>() {
                self.expect(&id, at.clone(), &[Kind::Agent, Kind::Group], true);
            }
        }
    }

    fn policies(&mut self, doc: &Document) {
        let Some(p) = &doc.policies else { return };
        for (i, e) in p.escalation.iter().enumerate() {
            self.expect(&e.from, format!("/policies/escalation/{i}/from"), &[Kind::Agent, Kind::Step], false);
            self.expect(&e.to, format!("/policies/escalation/{i}/to"), &[Kind::Agent], false);
        }
        for (i, s) in p.schedules.iter().enumerate() {
            self.expect(&s.agent, format!("/policies/schedules/{i}/agent"), &[Kind::Agent], false);
        }
    }

    fn patterns(&mut self, doc: &Document) {
        let Some(t) = &doc.topology else { return };
        for (i, pat) in t.patterns.iter().enumerate() {
            if let Some(via) = &pat.via {
                self.expect(via, format!("/topology/patterns/{i}/via"), &[Kind::Channel], false);
            }
            let roles = pat
                .members
                .iter()
                .chain(&pat.supervisor)
                .chain(&pat.auctioneer)
                .chain(&pat.source)
                .chain(&pat.collector)
                .chain(&pat.judge);
            for id in roles.cloned().collect::<Vec<_>>() {
                self.expect(&id, format!("/topology/patterns/{i}"), &[Kind::Agent], false);
            }
        }
    }
}
