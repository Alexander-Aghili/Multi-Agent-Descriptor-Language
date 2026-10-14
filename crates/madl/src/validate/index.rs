//! Every id in a document, with what it names and where it is.
use crate::model::Document;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Agent,
    Group,
    Step,
    Input,
    Channel,
    Connection,
    Pattern,
    Schedule,
}

impl Kind {
    pub fn noun(self) -> &'static str {
        match self {
            Kind::Agent => "an agent",
            Kind::Group => "a group",
            Kind::Step => "a step",
            Kind::Input => "an input",
            Kind::Channel => "a channel",
            Kind::Connection => "a connection",
            Kind::Pattern => "a pattern",
            Kind::Schedule => "a schedule",
        }
    }
}

/// Ids are unique across the whole document; the first use of an id wins here.
pub struct Index {
    pub ids: BTreeMap<String, (Kind, String)>,
    /// Later uses of an id already taken: (id, pointer, first pointer).
    pub duplicates: Vec<(String, String, String)>,
}

impl Index {
    pub fn build(doc: &Document) -> Self {
        let mut index = Index { ids: BTreeMap::new(), duplicates: Vec::new() };
        for (kind, list, ids) in lists(doc) {
            for (i, id) in ids.into_iter().enumerate() {
                if let Some(id) = id {
                    index.add(kind, id, format!("/{list}/{i}/id"));
                }
            }
        }
        index
    }

    fn add(&mut self, kind: Kind, id: &str, at: String) {
        match self.ids.get(id) {
            Some((_, first)) => self.duplicates.push((id.to_string(), at, first.clone())),
            None => {
                self.ids.insert(id.to_string(), (kind, at));
            }
        }
    }

    pub fn kind(&self, id: &str) -> Option<Kind> {
        self.ids.get(id).map(|(k, _)| *k)
    }
}

type IdList<'a> = (Kind, &'static str, Vec<Option<&'a String>>);

/// Every list whose entries have document-wide ids.
fn lists(doc: &Document) -> [IdList<'_>; 8] {
    fn ids(v: Vec<&String>) -> Vec<Option<&String>> {
        v.into_iter().map(Some).collect()
    }
    [
        (Kind::Agent, "agents", ids(doc.agents.iter().map(|a| &a.id).collect())),
        (Kind::Group, "groups", ids(doc.groups.iter().map(|g| &g.id).collect())),
        (Kind::Step, "steps", ids(doc.steps.iter().map(|s| &s.id).collect())),
        (Kind::Input, "inputs", ids(doc.inputs.iter().map(|i| &i.id).collect())),
        (Kind::Channel, "channels", ids(doc.channels.iter().map(|c| &c.id).collect())),
        (Kind::Connection, "connections", doc.connections.iter().map(|c| c.id.as_ref()).collect()),
        (Kind::Pattern, "topology/patterns", patterns(doc)),
        (Kind::Schedule, "policies/schedules", schedules(doc)),
    ]
}

fn patterns(doc: &Document) -> Vec<Option<&String>> {
    doc.topology.iter().flat_map(|t| t.patterns.iter().map(|p| p.id.as_ref())).collect()
}

fn schedules(doc: &Document) -> Vec<Option<&String>> {
    doc.policies.iter().flat_map(|p| p.schedules.iter().map(|s| Some(&s.id))).collect()
}
