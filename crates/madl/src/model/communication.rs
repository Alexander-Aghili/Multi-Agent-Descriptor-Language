//! Channels (mediums), connections (committed directional communication) and the rules that
//! allow or ban communication.
use super::Extensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// Where messages or artifacts travel.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Channel {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// `direct`, `filesystem`, `shared-memory`, `blackboard`, `bus`, `git`, `tracker`, `human`, or
    /// `x-<vendor>-<name>`.
    pub medium: String,
    /// Overrides the default protocol for traffic on this channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub protocol: Option<String>,
    /// `filesystem`: the folder, relative to the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub path: Option<String>,
    /// `bus`: the topic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub topic: Option<String>,
    /// Members and their rights. Absent: the agents whose connections use the channel.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, Access>>", optional)]
    pub access: BTreeMap<String, Access>,
    /// `human`: how people are reached, e.g. `{ slack: "#cad-review" }`, `{ email: "…" }`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, String>>", optional)]
    pub reach: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Access {
    Read,
    Write,
    ReadWrite,
}

/// A directed, committed line of communication, with every detail of how it happens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Connection {
    /// Default: `<from>-<kind>-<to>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    /// An agent, group or step id. A group means every member.
    pub from: String,
    pub to: String,
    pub kind: ConnectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub direction: Option<Direction>,
    /// The channel used. Default: a direct channel with the default protocol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub via: Option<String>,
    /// Overrides the channel's protocol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub protocol: Option<String>,
    /// Who may open a conversation. Default: `from`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiates: Option<Initiates>,
    /// `sync`: the sender waits for a reply. Default: `async`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mode: Option<SyncMode>,
    /// What passes along it, by artifact (an output id) or message type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Carried>>", optional)]
    pub carries: Vec<Carried>,
    /// How often one piece of work may pass along it in a loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_rounds: Option<u32>,
    /// A condition (CEL subset) that must hold for the connection to be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub when: Option<String>,
    /// Set by pattern expansion: the pattern this connection came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ConnectionKind {
    /// Work passes to `to`, which starts on it.
    Handoff,
    /// `from` sends results or status to `to`.
    Report,
    /// `from` may read `to`'s activity; it sends nothing.
    Watch,
    /// `from` may assign work to `to` at run time, choosing among its delegates.
    Delegate,
    /// `from` may consult `to` and wait for an answer; ownership does not move.
    Ask,
    /// `to` returns approve or changes-requested; changes go back to `from`.
    Review,
    /// `to` (usually a person) must approve before `from` continues.
    Approve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Direction {
    OneWay,
    /// The same connection in both directions.
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Initiates {
    From,
    To,
    Either,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum SyncMode {
    Async,
    Sync,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Carried {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub artifact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub message: Option<String>,
}

/// The communication mode and bans, at the root or as a restriction.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Communication {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mode: Option<CommunicationMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<DenyRule>>", optional)]
    pub deny: Vec<DenyRule>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CommunicationMode {
    /// Any agent may communicate with any other, except as denied.
    Open,
    /// Only along declared connections, in their direction, with their details.
    Closed,
}

/// A ban: `from` → `to` (one direction) or `between` (both). Ids may be agents or groups; `*`
/// matches anyone. `kinds` limits the ban to those kinds; empty bans every kind.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DenyRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub between: Option<[String; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<ConnectionKind>>", optional)]
    pub kinds: Vec<ConnectionKind>,
}
