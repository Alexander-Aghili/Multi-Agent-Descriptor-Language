//! Effective settings: what each agent, and each agent's subagents, actually run with.
//!
//! Blocks apply in order (see [`chain::for_agent`]); a block's preferences reach `self` by
//! default and its restrictions `descendants`. Preferences override, restrictions only tighten.
mod chain;
mod paths;
mod prefs;
mod tighten;

pub use chain::ancestors;

use crate::diagnostic::Diagnostic;
use crate::model::{Document, Preferences, Reach, Restrictions};
use chain::{Block, Origin};
use serde::Serialize;
use std::collections::BTreeMap;

/// Settings after every block that reaches them is applied.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Effective {
    pub preferences: Preferences,
    pub restrictions: Restrictions,
    /// Dotted field → the JSON Pointer of the block value that set it last.
    pub sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentSettings {
    pub id: String,
    #[serde(flatten)]
    pub settings: Effective,
    /// What an agent this one spawns would run with.
    pub subagents: Effective,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Resolved {
    pub agents: Vec<AgentSettings>,
    /// Blocks that tried to loosen an inherited restriction (code `restriction-loosened`).
    #[serde(skip)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Collects sources and loosening reports while one block is applied.
pub struct Ctx<'a> {
    origin: &'a Origin,
    sources: &'a mut BTreeMap<String, String>,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl Ctx<'_> {
    fn source(&mut self, field: &str) {
        self.sources.insert(field.to_string(), self.origin.pointer(field));
    }

    fn loosened(&mut self, field: &str, what: &str) {
        let at = self.origin.pointer(field);
        let d =
            Diagnostic::error("restriction-loosened", at, format!("`{field}` {what}; restrictions can only tighten"));
        if !self.diagnostics.contains(&d) {
            self.diagnostics.push(d);
        }
    }
}

/// Effective settings for every agent, in document order.
pub fn resolve(doc: &Document) -> Resolved {
    let mut out = Resolved::default();
    for (i, agent) in doc.agents.iter().enumerate() {
        let (settings, subagents) = agent_settings(doc, i, &mut out.diagnostics);
        out.agents.push(AgentSettings { id: agent.id.clone(), settings, subagents });
    }
    out
}

/// The agent's own settings, and those an agent it spawns would get.
fn agent_settings(doc: &Document, i: usize, diagnostics: &mut Vec<Diagnostic>) -> (Effective, Effective) {
    let blocks = chain::for_agent(doc, i);
    let settings = apply(&blocks, &[Reach::Itself, Reach::Descendants], diagnostics);
    let passes = |r: Reach| matches!(r, Reach::Subagents | Reach::Descendants);
    let reaching: Vec<Block> =
        blocks.into_iter().filter(|b| passes(b.preference_reach()) || passes(b.restriction_reach())).collect();
    let inherited = apply(&reaching, &[Reach::Subagents, Reach::Descendants], diagnostics);
    let mut subagents = apply_into(inherited, &chain::for_subagents(doc, i), diagnostics);
    same_runtime(&settings, &mut subagents);
    one_level_down(&mut subagents);
    (settings, subagents)
}

fn apply(blocks: &[Block], reaches: &[Reach], diagnostics: &mut Vec<Diagnostic>) -> Effective {
    let mut e = Effective::default();
    for block in blocks {
        let mut ctx = Ctx { origin: &block.origin, sources: &mut e.sources, diagnostics };
        if let Some(p) = block.settings.preferences.as_ref().filter(|_| reaches.contains(&block.preference_reach())) {
            prefs::merge(&mut e.preferences, p, &mut ctx);
        }
        if let Some(r) = block.settings.restrictions.as_ref().filter(|_| reaches.contains(&block.restriction_reach())) {
            tighten::apply(&mut e.restrictions, r, &mut ctx);
        }
    }
    e
}

/// Apply blocks meant only for subagents: all of each block applies.
fn apply_into(mut e: Effective, blocks: &[Block], diagnostics: &mut Vec<Diagnostic>) -> Effective {
    for block in blocks {
        let mut ctx = Ctx { origin: &block.origin, sources: &mut e.sources, diagnostics };
        if let Some(p) = &block.settings.preferences {
            prefs::merge(&mut e.preferences, p, &mut ctx);
        }
        if let Some(r) = &block.settings.restrictions {
            tighten::apply(&mut e.restrictions, r, &mut ctx);
        }
    }
    e
}

/// A spawned agent runs on its parent's runtime unless something says otherwise.
fn same_runtime(parent: &Effective, sub: &mut Effective) {
    if sub.preferences.runtime.is_none() {
        if let Some(r) = &parent.preferences.runtime {
            sub.preferences.runtime = Some(r.clone());
            if let Some(src) = parent.sources.get("preferences.runtime") {
                sub.sources.insert("preferences.runtime".into(), src.clone());
            }
        }
    }
}

/// A spawned agent is one level further down: it may spawn one level less.
fn one_level_down(e: &mut Effective) {
    if let Some(spawn) = e.restrictions.spawn.as_mut() {
        if let Some(depth) = spawn.max_depth {
            spawn.max_depth = Some(depth.saturating_sub(1));
            if depth <= 1 {
                spawn.allowed = Some(false);
            }
        }
    }
}
