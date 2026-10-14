//! Steps, topology patterns and loops.
use super::CopyMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A node that is not an agent. Which fields apply depends on `kind`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Step {
    pub id: String,
    pub kind: StepKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// `check`: the shell command; exit 0 means passed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout_s: Option<u32>,
    /// `human`: what the person decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub prompt: Option<String>,
    /// `human`: the choices the person has.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub outcomes: Vec<String>,
    /// `merge`: the branch to merge into. Default: the project's current branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub into: Option<String>,
    /// `fan-out`: what to split into items (CEL subset), e.g. `inputs.parts`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub over: Option<String>,
    /// `fan-out`: how items' workspaces are made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub copies: Option<CopyMode>,
    /// `fan-in`: wait for all items, or continue on the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub join: Option<Join>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum StepKind {
    Check,
    Human,
    Merge,
    FanOut,
    FanIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Join {
    All,
    Any,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Topology {
    /// Shorthand that expands into connections (and, for `blackboard`, a channel).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Pattern>>", optional)]
    pub patterns: Vec<Pattern>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub loops: Option<Loops>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Loops {
    /// How often one piece of work may go round a loop, unless a connection on it says less.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_rounds: Option<u32>,
}

/// A named shape. Which roles apply depends on `pattern`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Pattern {
    /// Default: `<pattern>-<position>`, counting from 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    pub pattern: PatternKind,
    pub members: Vec<String>,
    /// `supervisor`: the agent that assigns work and receives reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub supervisor: Option<String>,
    /// `auction`: the agent that runs the auction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auctioneer: Option<String>,
    /// `committee`: the agent that hands the work to every member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<String>,
    /// `committee`: where members report. Default: the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub collector: Option<String>,
    /// `debate`: the agent members report to when the debate ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub judge: Option<String>,
    /// `supervisor`: `dynamic` (default) chooses at run time; `static` hands work to every member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub routing: Option<Routing>,
    /// `debate`, `role-play`, `swarm`: the round limit put on the expanded connections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_rounds: Option<u32>,
    /// The channel the expanded connections use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub protocol: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum PatternKind {
    Pipeline,
    Supervisor,
    Committee,
    Debate,
    RolePlay,
    Auction,
    Swarm,
    Blackboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Routing {
    Dynamic,
    Static,
}
