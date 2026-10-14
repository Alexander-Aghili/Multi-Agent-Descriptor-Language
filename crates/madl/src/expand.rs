//! Topology patterns expand into connections (and, for `blackboard`, a channel). A connection the
//! author already declared with the same `from`, `to` and `kind` is kept and not repeated.
use crate::model::*;

/// The pattern's id: its own, or `<pattern>-<position>` counting from 1.
pub fn pattern_id(p: &Pattern, index: usize) -> String {
    p.id.clone().unwrap_or_else(|| format!("{}-{}", pattern_name(p.pattern), index + 1))
}

pub fn pattern_name(kind: PatternKind) -> &'static str {
    match kind {
        PatternKind::Pipeline => "pipeline",
        PatternKind::Supervisor => "supervisor",
        PatternKind::Committee => "committee",
        PatternKind::Debate => "debate",
        PatternKind::RolePlay => "role-play",
        PatternKind::Auction => "auction",
        PatternKind::Swarm => "swarm",
        PatternKind::Blackboard => "blackboard",
    }
}

/// A copy of the document with every pattern expanded. Patterns stay, as a record of intent.
pub fn expand(doc: &Document) -> Document {
    let mut out = doc.clone();
    let patterns = doc.topology.iter().flat_map(|t| t.patterns.iter().enumerate());
    for (i, p) in patterns {
        let id = pattern_id(p, i);
        for (from, kind, to) in edges(p) {
            add(&mut out, connection(p, &id, from, kind, to));
        }
        if p.pattern == PatternKind::Blackboard {
            board(&mut out, p, &id);
        }
    }
    out
}

fn edges(p: &Pattern) -> Vec<(String, ConnectionKind, String)> {
    use ConnectionKind::*;
    let m = &p.members;
    match p.pattern {
        PatternKind::Pipeline => m.windows(2).map(|w| (w[0].clone(), Handoff, w[1].clone())).collect(),
        PatternKind::Supervisor => {
            let kind = if p.routing == Some(Routing::Static) { Handoff } else { Delegate };
            p.supervisor.as_ref().map(|s| hub(m, s, kind)).unwrap_or_default()
        }
        PatternKind::Auction => p.auctioneer.as_ref().map(|s| hub(m, s, Delegate)).unwrap_or_default(),
        PatternKind::Committee => committee(p),
        PatternKind::Debate => debate(p),
        PatternKind::RolePlay => pairs(m, Ask),
        PatternKind::Swarm => pairs(m, Handoff),
        PatternKind::Blackboard => Vec::new(),
    }
}

/// `head` sends work to every other member with `kind`; each reports back.
fn hub(members: &[String], head: &String, kind: ConnectionKind) -> Vec<(String, ConnectionKind, String)> {
    let others = members.iter().filter(|x| *x != head);
    others.flat_map(|x| [(head.clone(), kind, x.clone()), (x.clone(), ConnectionKind::Report, head.clone())]).collect()
}

fn committee(p: &Pattern) -> Vec<(String, ConnectionKind, String)> {
    let Some(source) = &p.source else {
        return Vec::new();
    };
    let collector = p.collector.as_ref().unwrap_or(source);
    let members = p.members.iter().filter(|m| *m != source);
    members
        .flat_map(|m| {
            [
                (source.clone(), ConnectionKind::Handoff, m.clone()),
                (m.clone(), ConnectionKind::Report, collector.clone()),
            ]
        })
        .collect()
}

fn debate(p: &Pattern) -> Vec<(String, ConnectionKind, String)> {
    let mut out = pairs(&p.members, ConnectionKind::Review);
    if let Some(judge) = &p.judge {
        out.extend(
            p.members.iter().filter(|m| *m != judge).map(|m| (m.clone(), ConnectionKind::Report, judge.clone())),
        );
    }
    out
}

/// Both directions between every two members.
fn pairs(m: &[String], kind: ConnectionKind) -> Vec<(String, ConnectionKind, String)> {
    let mut out = Vec::new();
    for (i, a) in m.iter().enumerate() {
        for b in &m[i + 1..] {
            out.push((a.clone(), kind, b.clone()));
            out.push((b.clone(), kind, a.clone()));
        }
    }
    out
}

fn connection(p: &Pattern, pattern: &str, from: String, kind: ConnectionKind, to: String) -> Connection {
    let looped = matches!(p.pattern, PatternKind::Debate | PatternKind::RolePlay | PatternKind::Swarm);
    Connection {
        id: Some(format!("{pattern}-{from}-{}-{to}", kind_name(kind))),
        from,
        to,
        kind,
        direction: None,
        via: p.via.clone(),
        protocol: p.protocol.clone(),
        initiates: None,
        mode: None,
        carries: Vec::new(),
        max_rounds: if looped && kind != ConnectionKind::Report { p.max_rounds } else { None },
        when: None,
        pattern: Some(pattern.to_string()),
        extensions: Default::default(),
    }
}

fn add(doc: &mut Document, c: Connection) {
    let exists = doc.connections.iter().any(|d| d.from == c.from && d.to == c.to && d.kind == c.kind);
    if !exists {
        doc.connections.push(c);
    }
}

fn board(doc: &mut Document, p: &Pattern, pattern: &str) {
    let id = p.via.clone().unwrap_or_else(|| format!("{pattern}-board"));
    if doc.channels.iter().any(|c| c.id == id) {
        return;
    }
    let access = p.members.iter().map(|m| (m.clone(), Access::ReadWrite)).collect();
    doc.channels.push(Channel {
        id,
        medium: "blackboard".into(),
        protocol: p.protocol.clone(),
        access,
        ..Default::default()
    });
}

pub fn kind_name(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::Handoff => "handoff",
        ConnectionKind::Report => "report",
        ConnectionKind::Watch => "watch",
        ConnectionKind::Delegate => "delegate",
        ConnectionKind::Ask => "ask",
        ConnectionKind::Review => "review",
        ConnectionKind::Approve => "approve",
    }
}
