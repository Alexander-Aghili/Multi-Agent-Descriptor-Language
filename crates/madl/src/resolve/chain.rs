//! The settings blocks that cover an agent, in the order they apply.
use crate::model::{Agent, Communication, Document, Preferences, Reach, Restrictions, Settings};

/// Where a block sits, so a value can name its source and a loosening its location.
#[derive(Clone, Debug)]
pub enum Origin {
    /// A settings block at this JSON Pointer.
    Settings(String),
    /// The document's root `communication`.
    RootCommunication,
    /// An agent's (or template's) `runtime` / `model` / `resume` shorthand at this pointer.
    Shorthand(String),
}

impl Origin {
    /// The JSON Pointer of `field` (dotted, e.g. `restrictions.budget.tokens`) in this block.
    pub fn pointer(&self, field: &str) -> String {
        match self {
            Origin::Settings(at) => format!("{at}/{}", field.replace('.', "/")),
            Origin::RootCommunication => {
                let rest = field.strip_prefix("restrictions.communication").unwrap_or("");
                format!("/communication{}", rest.replace('.', "/"))
            }
            Origin::Shorthand(at) => format!("{at}/{}", field.rsplit('.').next().unwrap_or(field)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub origin: Origin,
    pub settings: Settings,
}

impl Block {
    pub fn preference_reach(&self) -> Reach {
        self.settings.applies_to.and_then(|a| a.preferences).unwrap_or(Reach::Itself)
    }

    pub fn restriction_reach(&self) -> Reach {
        self.settings.applies_to.and_then(|a| a.restrictions).unwrap_or(Reach::Descendants)
    }
}

/// Blocks covering agent `index`: defaults, root communication, its groups outermost first,
/// its own settings, its shorthand.
pub fn for_agent(doc: &Document, index: usize) -> Vec<Block> {
    let agent = &doc.agents[index];
    let mut blocks = Vec::new();
    if let Some(s) = &doc.defaults {
        blocks.push(Block { origin: Origin::Settings("/defaults".into()), settings: s.clone() });
    }
    blocks.extend(doc.communication.as_ref().map(root_communication));
    blocks.extend(group_blocks(doc, agent));
    if let Some(s) = &agent.settings {
        let origin = Origin::Settings(format!("/agents/{index}/settings"));
        blocks.push(Block { origin, settings: s.clone() });
    }
    blocks.extend(shorthand(agent, format!("/agents/{index}")));
    blocks
}

/// The root `communication` acts as a restriction on every agent.
fn root_communication(c: &Communication) -> Block {
    let restrictions = Restrictions { communication: Some(c.clone()), ..Default::default() };
    let settings = Settings { restrictions: Some(restrictions), ..Default::default() };
    Block { origin: Origin::RootCommunication, settings }
}

fn group_blocks(doc: &Document, agent: &Agent) -> Vec<Block> {
    let mut seen: Vec<usize> = Vec::new();
    for gid in &agent.groups {
        let mut chain = ancestors(doc, gid);
        chain.reverse();
        for g in chain {
            if !seen.contains(&g) {
                seen.push(g);
            }
        }
    }
    seen.into_iter()
        .filter_map(|g| {
            let settings = doc.groups[g].settings.clone()?;
            Some(Block { origin: Origin::Settings(format!("/groups/{g}/settings")), settings })
        })
        .collect()
}

/// The group with this id and its parents, innermost first. Stops at a cycle or unknown id.
pub fn ancestors(doc: &Document, id: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut next = Some(id.to_string());
    while let Some(gid) = next {
        let Some(i) = doc.groups.iter().position(|g| g.id == gid) else {
            break;
        };
        if out.contains(&i) {
            break;
        }
        out.push(i);
        next = doc.groups[i].parent.clone();
    }
    out
}

/// The agent's shorthand fields as a preferences block.
fn shorthand(agent: &Agent, at: String) -> Option<Block> {
    let preferences = Preferences {
        runtime: agent.runtime.clone(),
        model: agent.model.clone(),
        resume: agent.resume,
        ..Default::default()
    };
    (preferences != Preferences::default()).then(|| Block {
        origin: Origin::Shorthand(at),
        settings: Settings { preferences: Some(preferences), ..Default::default() },
    })
}

/// Blocks that only spawned agents get: the agent's `subagents.settings`, then its template.
pub fn for_subagents(doc: &Document, index: usize) -> Vec<Block> {
    let Some(sub) = &doc.agents[index].subagents else {
        return Vec::new();
    };
    let mut blocks = Vec::new();
    if let Some(s) = &sub.settings {
        let at = format!("/agents/{index}/subagents/settings");
        blocks.push(Block { origin: Origin::Settings(at), settings: s.clone() });
    }
    if let Some(t) = &sub.template {
        let preferences = Preferences { runtime: t.runtime.clone(), model: t.model.clone(), ..Default::default() };
        if preferences != Preferences::default() {
            let origin = Origin::Shorthand(format!("/agents/{index}/subagents/template"));
            let settings = Settings { preferences: Some(preferences), ..Default::default() };
            blocks.push(Block { origin, settings });
        }
    }
    blocks
}
