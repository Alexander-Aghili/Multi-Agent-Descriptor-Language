# MADL core: design

Status: draft for review · 2026-10-14

## 1. Purpose

The Multi-Agent Descriptor Language (MADL) is a vendor-neutral file format that describes a
multi-agent system: its agents, how they are grouped, who may talk to whom and how, the shape of
the work (topology and workflow), the settings each agent runs with, and the restrictions it runs
under. One MADL document can be validated, drawn, compared, shared and run by any runtime that
implements it.

MADL describes a system. It does not define a wire protocol (A2A and MCP do that, and MADL refers
to them by name), and v0.1 does not define a run record or trace format.

### Where it sits

| Layer | Examples | MADL's relation |
|---|---|---|
| Agent ↔ tool protocol | MCP | Referenced: an agent's tools may be MCP servers |
| Agent ↔ agent protocol | A2A, runtime-native messaging | Referenced: a connection names its protocol |
| Single agent / flow spec | Open Agent Spec | Interop: import/export |
| Runtimes and frameworks | Sculptured, CAMEL, CrewAI, LangGraph, AutoGen, OpenAI Agents SDK | Targets: an adapter builds the runtime's objects from a MADL document |
| **System descriptor** | **MADL** | The whole system, its topology, communication, settings cascade and policy |

### Goals

1. Everything Sculptured's team format (`sculptured.team/v1`) expresses today can be written in
   MADL without loss, apart from things that are Sculptured's own (they go in an extension).
2. Global settings at the root, overridable per group and per agent, with a stated rule for whether
   each setting reaches that agent's subagents.
3. Communication is explicit: protocols, channels (mediums), direction, and allow/deny rules.
4. Restrictions are monotone: a group, agent or subagent can tighten an inherited restriction,
   never loosen it.
5. The effective settings of every agent are computable by a stated algorithm, so two
   implementations always agree (`madl resolve`).
6. The document is drawable: stable ids and an optional layout section.

### Non-goals (v0.1)

Wire protocols; run records and traces (a later companion spec, from Sculptured's `session-v1`);
agent discovery or registries; document signing (security considerations reserve the space);
arbitrary code (runtime-specific callables live in extensions).

## 2. Decisions

| # | Decision |
|---|---|
| 1 | MADL is the vendor-neutral successor of `sculptured.team/v1`; Sculptured gets a converter, then reads and writes MADL natively |
| 2 | v0.1 scope: the system descriptor only |
| 3 | Authored as YAML or JSON; JSON is canonical; no custom syntax |
| 4 | Source of truth: the Rust crate's types → generated `schema/<revision>/schema.json` (JSON Schema 2020-12, via `schemars`) and TypeScript types (via `ts-rs`), as Sculptured does today. The published `schema.json` is the language-neutral definition other languages generate from |
| 5 | Flat document (compose-style), not `apiVersion/kind/metadata/spec` |
| 6 | Dated spec revisions (`2026-11-01`); `draft` while unreleased |
| 7 | Settings split into **preferences** (override) and **restrictions** (only tighten) |
| 8 | By default restrictions reach all descendants; preferences reach only the agent itself |
| 9 | Communication defaults to **closed**: only declared connections are allowed |
| 10 | Topology patterns are shorthand that expand to the explicit graph |
| 11 | One graph: agents and non-agent steps (check, human, merge, fan-out, fan-in) in the same graph |
| 12 | `layout` is an optional section of the standard |
| 13 | Open Agent Spec: interop only |
| 14 | Apache-2.0 for code, CC-BY-4.0 for spec text |
| 15 | Hosted under the author's GitHub account; moved to an org before public announcement |
| 16 | Name: Multi-Agent Descriptor Language (MADL) |
| 17 | Rust only for now: the `madl` crate and the `madl` CLI. Python and TypeScript packages and the CAMEL adapter come later, generated from `schema.json` |
| 18 | When Python and TypeScript packages come, they call the Rust core (PyO3, wasm) rather than reimplement it, so semantics cannot drift |

## 3. Document

```yaml
madl: "draft"                       # spec revision
system:
  id: cad-review                    # [a-z0-9][a-z0-9-]*, unique per document
  name: CAD review team
  description: Designs a bracket, checks it, and reviews it before release.
defaults:   { ... }                 # global settings (§5)
inputs:     [ ... ]                 # what the system is given (§4.4)
groups:     [ ... ]                 # §4.2
agents:     [ ... ]                 # §4.1
steps:      [ ... ]                 # non-agent nodes (§7)
channels:   [ ... ]                 # mediums (§6.2)
connections: [ ... ]                # directed communication (§6.3)
communication: { ... }              # mode and deny rules (§6.4)
topology:   { ... }                 # patterns, loops, spawning (§7)
policies:   { ... }                 # failure, escalation, schedules (§8)
layout:     { ... }                 # optional drawing (§9)
extensions: { ... }                 # reverse-DNS namespaced (§10)
```

A document may be split: any object may be `{ $ref: "./path.yaml" }` or a JSON Pointer into
another file; instruction text may be `{ $ref: "./prompts/reviewer.md" }`. Resolution is
relative to the referring file and must stay inside the document's folder unless the reader
opts in.

## 4. Participants

### 4.1 Agents

```yaml
agents:
  - id: cad
    name: CAD engineer
    instructions: { $ref: ./prompts/cad.md }   # or a string
    runtime: claude-code        # claude-code | codex | camel | crewai | langgraph | openai-agents
                                # | remote-a2a | human | x-<vendor>-<name>
    endpoint: null              # remote-a2a: the agent's A2A URL or Agent Card URL
    model: anthropic/claude-opus-5-5           # provider/model; or { id, fallback: [...] }
    groups: [design]            # membership; an agent may be in no group
    tools:                      # what the agent is given (§5.3 restricts what it may use)
      - { id: fs, mcp: { command: "npx", args: ["@modelcontextprotocol/server-filesystem", "."] } }
      - { id: cad-kernel, mcp: { url: "https://…" } }
    skills: [ { $ref: ./skills/step-export/SKILL.md } ]
    context: [ ./requirements.md ]             # files given to the agent
    workspace: { folder: ./cad, copy: own }    # own = its own git copy; shared = the project folder
    placement: { machine: { os: macos } }      # where it runs (selector)
    outputs: [ { id: model, path: ./cad/bracket.step } ]
    checks:  [ { command: "make verify", after: each-turn, timeout_s: 600 } ]
    resume: resume-if-possible  # | relaunch-fresh | checkpoint-only
    settings: { ... }           # §5
    subagents: { ... }          # §5.4
```

`runtime: human` makes a person a participant: they can receive handoffs, answer questions and
approve. Approvals and questions are connections to a human (§6.3).

### 4.2 Groups

```yaml
groups:
  - id: design
    name: Design
    purpose: Produce a manufacturable bracket.   # passed to every member and to agents connected to the group
    parent: null                # groups nest (CAMEL's nested Workforce)
    settings: { ... }           # §5; applies to members
```

A group holds only the agents that list it. Agents outside every group are valid.

### 4.3 Identity

Ids are stable and unique across agents, groups, steps, channels, connections and inputs in one
document. Tools rely on ids to keep diagrams and history attached to the same component across
revisions; renaming changes `name`, never `id`.

### 4.4 Inputs

```yaml
inputs:
  - id: brief
    text: { $ref: ./brief.md }
    to: [planner]               # default: the starting agents (§7.3)
```

An agent connected to an input receives it after its own instructions when the system starts;
an agent further along receives it with its first handoff.

## 5. Settings and inheritance

### 5.1 Shape

Every settings block, at the root (`defaults`), on a group, on an agent, or in a `subagents`
block, has the same shape:

```yaml
settings:
  preferences:            # override: the nearest value wins
    model: anthropic/claude-sonnet-5-5
    protocol: native
    language: en
    instructions_append: "Write in plain English."
    resume: resume-if-possible
  restrictions:           # tighten only (§5.3)
    tools:   { deny: [web-fetch] }        # or { allow: [fs, cad-kernel] }: only these
    communication: { mode: closed, deny: [ { from: cad, to: customer } ] }   # §6.4
    files:   { read: ["."], write: ["./out"] }
    network: { egress: [ "pypi.org", "github.com" ] }
    budget:  { tokens: 2000000, cost_usd: 20, wall_clock: 2h, scope: shared }
    spawn:   { allowed: true, max_children: 4, max_depth: 2 }
    approvals: { required_for: [shell, network, write-outside-workspace] }
    questions: { timeout: 30m, on_timeout: continue }
  applies_to:             # optional per-category override of §5.2's defaults
    preferences: self
    restrictions: descendants
```

Every field is optional. Absent means inherit; a document with no settings at all is valid and
has no limits, because MADL sets no limits the author did not set.

### 5.2 Reach

`applies_to` takes `self` (this agent only), `subagents` (only the agents this agent spawns,
and theirs), or `descendants` (this agent and everything it spawns).

| Category | Default reach |
|---|---|
| preferences | `self` |
| restrictions | `descendants` |

Group and root settings apply to every agent the block covers, with the same reach rules
applied to each of those agents.

### 5.3 Restrictions combine by tightening

| Restriction | Combination down the chain | Loosening is |
|---|---|---|
| `tools.allow` (only these tools; absent = every tool the agent is given) | allow lists intersect | an error to add an entry |
| `tools.deny` (never these tools) | deny lists union | an error to remove an entry |
| both present | allowed = allow list minus deny list | — |
| `communication.mode` | `open` < `closed` | an error to go from closed to open |
| `communication.deny` | union | an error to remove an entry |
| `files.read`, `files.write` | path sets intersect | an error |
| `network.egress` | `deny` < host list < `allow`; host lists intersect | an error |
| `budget.*` numbers | minimum | an error |
| `budget.scope` | `shared`: the agent and its descendants draw from one budget; `own`: each agent has its own | — |
| `spawn.allowed` | AND | an error |
| `spawn.max_*` | minimum | an error |
| `approvals.required_for` | union | an error to remove an entry |

Tools are named by tool id (`fs`), by MCP server and tool (`fs/write_file`), or by a runtime's
built-in name (`claude-code:Bash`). A validator reports a static loosening as an error with the
path of the inherited value; a runtime clamps a loosening it meets at run time and records it.

### 5.4 Subagents

```yaml
agents:
  - id: planner
    settings: { ... }            # the planner itself
    subagents:
      template:                  # what a spawned agent starts as (CAMEL's new_worker_agent)
        runtime: claude-code
        instructions: "You are a focused helper for one subtask."
      settings:                  # settings for spawned agents only
        preferences: { model: anthropic/claude-haiku-5-5 }
        restrictions: { tools: { allow: [fs] } }
```

### 5.5 Resolution algorithm

For agent A, effective settings are computed in this order, each step applying only the blocks
whose reach covers A:

1. `defaults`
2. A's groups, outermost to innermost; for an agent in several unrelated groups, in the order
   A lists them
3. A's own `settings`
4. If A was spawned by P: P's effective restrictions with reach `subagents` or `descendants`,
   then P's `subagents.settings`

Preferences: a scalar replaces, an object merges key by key, a list replaces.
Restrictions: combined by §5.3 at every step.

`madl resolve <file>` prints the effective settings of every agent, with the source of each
value. Conformance fixtures (§11) pin the output.

## 6. Communication

### 6.1 Protocols

A protocol is named with an optional version: `native` (the runtime's own messaging, e.g. Claude
Code's subagent tool), `a2a@1.2`, `mcp@2025-06-18` (one agent exposed to another as a tool),
or `x-<vendor>-<name>`. `defaults.settings.preferences.protocol` sets the default; a channel or
connection may override it.

### 6.2 Channels (mediums)

```yaml
channels:
  - id: review-chat
    medium: direct            # direct | filesystem | shared-memory | blackboard | bus | git
                              # | tracker | human | x-<vendor>-<name>
    protocol: a2a@1.2
  - id: shared
    medium: filesystem
    path: ./shared
    access: { cad: read-write, reviewer: read }
  - id: findings
    medium: bus
    topic: findings
  - id: approvals
    medium: human
    via: { slack: "#cad-review" }   # or email, console, the runtime's own UI
```

A channel is where messages or artifacts travel. A channel with `access` gives members those
rights; without it, members are the agents whose connections use it.

### 6.3 Connections

A connection is a directed, committed line of communication. Different pairs of agents may talk
in different ways, so every detail belongs to the connection:

```yaml
connections:
  - id: c-cad-review
    from: cad
    to: reviewer              # an agent, group, step or input id
    kind: handoff             # handoff | report | watch | delegate | ask | review | approve
    direction: one-way        # one-way | both (both = a pair of connections with the same details)
    via: review-chat          # channel id; default: a direct channel with the default protocol
    protocol: a2a@1.2         # overrides the channel's
    initiates: from           # who may open a conversation: from | to | either
    mode: async               # async | sync (request/response)
    carries:                  # what passes, by artifact or message type
      - artifact: model
      - message: summary
    max_rounds: 3             # for loops through this connection (§7.4)
    when: "status == 'ready'" # optional condition; expression language in §7.5
```

| Kind | Meaning | Sculptured v1 |
|---|---|---|
| `handoff` | Work passes to `to`, which starts on it | `hands-work-to` |
| `report` | `from` sends results or status to `to` | `reports-to` |
| `watch` | `from` may read `to`'s activity; sends nothing | `watches` |
| `delegate` | `from` may assign work to `to` at run time, choosing among its delegates | dispatcher |
| `ask` | `from` may consult `to` and wait for an answer; ownership does not move | — |
| `review` | `to` returns approve or changes-requested; changes go back to `from` | review panel |
| `approve` | `to` (usually a human) must approve before `from` continues | human stage |

A connection to a group reaches every member of the group.

### 6.4 Allowing and banning communication

```yaml
communication:
  mode: closed                # closed: only declared connections are allowed
                              # open: any agent may message any other, except as denied
  deny:
    - { from: cad, to: customer }              # one direction
    - { between: [planner, reviewer] }         # both directions
    - { from: "group:design", to: "group:sales", kinds: [handoff, delegate] }
```

Rules:

1. In `closed` mode an agent may communicate only along its declared connections, in their
   direction, with their kind, protocol and channel.
2. In `open` mode undeclared communication is allowed, using the default protocol and a direct
   channel.
3. `deny` always wins, in both modes; a declared connection that a deny rule matches is a
   validation error.
4. Groups and agents may set `communication` in their settings as a restriction: an agent can go
   from open to closed and add deny rules, never the reverse.
5. Subagents may communicate with their parent by default; other communication follows the
   same rules as for any agent, with the subagent's effective settings.

## 7. Topology and workflow

### 7.1 Steps

Non-agent nodes share the graph with agents:

```yaml
steps:
  - { id: verify,   kind: check,  command: "make verify", timeout_s: 600 }
  - { id: sign-off, kind: human,  prompt: "Release this revision?", outcomes: [release, revise] }
  - { id: land,     kind: merge,  into: main }
  - { id: each-part, kind: fan-out, over: "inputs.parts", copies: per-item }
  - { id: all-parts, kind: fan-in,  join: all }       # all | any
```

Sculptured's workflow stages map directly: `agent` stages are agents; `check`, `human`, `merge`
are steps; dispatcher and collector are `delegate` and `report` connections; review panels are
several `review` connections into one fan-in.

### 7.2 Patterns

Patterns are shorthand. A reader expands them into agents' connections before validation, and
`madl expand` prints the result.

```yaml
topology:
  patterns:
    - pattern: supervisor     # pipeline | supervisor | committee | debate | role-play
                              # | auction | swarm | blackboard
      supervisor: planner
      members: [cad, reviewer]
      routing: dynamic        # dynamic: the supervisor chooses at run time (delegate)
                              # static: fixed handoffs
```

| Pattern | Expands to |
|---|---|
| pipeline | `handoff` from each member to the next |
| supervisor | `delegate` from the supervisor to each member, `report` back |
| committee | `handoff` from a source to every member, `report` to a fan-in |
| debate | `review` both ways among members for `rounds`, then a judge |
| role-play | two members, `ask` both ways, `max_rounds` (CAMEL `RolePlaying`, `chat_turn_limit`) |
| auction | `delegate` from an auctioneer; members bid; one is chosen |
| swarm | `open` communication among members |
| blackboard | a `blackboard` channel all members read and write |

### 7.3 Starting points

A starting agent is one connected from an input, or with no incoming `handoff` or `delegate`.
A document with no starting agent is an error.

### 7.4 Loops

A cycle of `handoff` or `review` connections is allowed when it can be reached from a starting
agent. One piece of work goes round at most `max_rounds` times (the smallest on the cycle;
default from `topology.loops.max_rounds`; if neither is set, the runtime's own limit). A cycle
that no starting agent reaches is an error: the work has no starting point.

### 7.5 Conditions

`when` and `fan-out.over` use a small expression language (CEL subset): field access,
comparison, `&&`, `||`, `!`, `in`. Unknown fields are false, never errors.

## 8. Policies

```yaml
policies:
  on_failure:                 # CAMEL Workforce recovery: retry, replan, decompose
    strategy: retry           # retry | replan | decompose | escalate | stop
    max_attempts: 2
    then: escalate
  escalation:
    - { from: cad, after_rounds: 3, to: senior-cad }
  schedules:
    - { id: nightly, agent: reviewer, every: 24h, message: "Review today's changes." }
  guards:
    - { command: "make health", every: 10m, timeout_s: 600 }   # pauses new work while failing
```

## 9. Layout

```yaml
layout:
  units: px
  nodes:  { cad: { x: 120, y: 80 }, verify: { x: 360, y: 80 } }
  groups: { design: { x: 80, y: 40, w: 520, h: 260 } }
  inputs: { brief: { x: -160, y: 80 } }
```

Layout has no meaning for execution. A reader may ignore it; a writer must preserve the entries
of components it did not change.

## 10. Extensions

```yaml
extensions:
  com.usesculptured:
    decision_log: [ ... ]
  ai.camel:
    workforce: { share_memory: true, use_structured_output_handler: true }
```

Keys are reverse-DNS names. Readers ignore namespaces they don't know and keep them when
writing. A component may carry its own `extensions` map too.

## 11. Conformance

| Level | Must |
|---|---|
| Reader | Parse YAML and JSON, resolve `$ref`, validate against the schema and the rules in §§5–7 |
| Resolver | Reader + expand patterns and compute effective settings exactly as the fixtures say |
| Runner | Resolver + run the system and enforce every restriction it accepts. A runner that cannot enforce a restriction must refuse the document or ask the person to accept the gap, naming it |
| Renderer | Reader + draw the graph, using `layout` when present |

`conformance/` holds `valid/`, `invalid/` (each with the expected error code and path) and
`resolve/` (document → expected effective settings and expanded graph). Every implementation
runs the same fixtures in CI.

## 12. Implementations

```
crates/madl/          the types (serde), validate, resolve, expand, graph, convert
crates/madl-cli/      the `madl` binary
schema/<revision>/schema.json    generated from the crate's types; a test fails if it is stale
                                 (regenerate with UPDATE_SCHEMAS=1)
bindings/typescript/  generated TypeScript types (ts-rs), used by Sculptured's web board
```

CLI: `madl validate`, `madl resolve`, `madl expand`, `madl graph` (Mermaid and SVG),
`madl convert --from sculptured-team-v1 --to madl` (Open Agent Spec and CrewAI later),
`madl export a2a-cards`.

Sculptured depends on the `madl` crate (a git dependency until it is published to crates.io).

Later, not in v0.1: a Python package (pydantic models generated from `schema.json`, calling the
Rust core through PyO3 wheels), a TypeScript package (types from `ts-rs`, the core through
wasm), and the CAMEL adapter in Python.

## 13. Interop mappings

### Sculptured `team/v1` → MADL

| team/v1 | MADL |
|---|---|
| `TeamDraft.name`, `id` | `system.name`, `system.id` |
| `agents[].kind` (`claude`, `codex`) | `runtime: claude-code`, `codex` |
| `agents[].job` | `instructions` |
| `agents[].folder` | `workspace.folder` |
| `agents[].advanced.model`, group `models`, team `models` | `preferences.model` at agent, group, root |
| `advanced.permission` (`unrestricted`/`restricted`) | `restrictions.approvals` and `tools` |
| `advanced.token_limit {tokens, include_subagents}` | `restrictions.budget {tokens, scope: shared|own}` |
| `advanced.check` | `checks[] {after: each-turn}` |
| `advanced.resume` | `resume` |
| `groups[]` (`label`, `x/y/w/h`) | `groups[] {name}`; box in `layout.groups` |
| `connections[]` kinds | `handoff`, `report`, `watch` |
| `input {text, to, x, y}` | `inputs[]`; position in `layout.inputs` |
| `limits {max_agents, max_child_levels}` | `restrictions.spawn` |
| `schedules[]` | `policies.schedules` |
| `workflow.stages[]` | agents + `steps` + connections (§7.1) |
| `workflow.guard` | `policies.guards` |
| `workflow.copies` (`shared`/`per-item`) | `workspace.copy`, `fan-out.copies` |
| stage `escalate` | `policies.escalation` |
| `settings.own_copies`, `posture`, `question_timeout_min` | `workspace.copy`, `restrictions.approvals`, `restrictions.questions` |
| `decision_log` | `extensions.com.usesculptured` |
| `x`, `y` | `layout.nodes` |

### CAMEL Workforce → MADL

| CAMEL | MADL |
|---|---|
| `Workforce` | a group, with a `supervisor` pattern, `routing: dynamic` |
| nested `Workforce` | a child group (`parent`) |
| `coordinator_agent` | the supervisor agent (`delegate` connections) |
| `task_agent` (decomposition) | an agent with `delegate` to the coordinator, or `policies.on_failure.strategy: decompose` |
| `SingleAgentWorker` (`ChatAgent` + tools + system prompt) | an agent: `runtime: camel`, `tools`, `instructions` |
| `RolePlayingWorker` (`assistant_role_name`, `user_role_name`, `chat_turn_limit`) | a `role-play` pattern with `max_rounds` |
| `new_worker_agent` | `subagents.template` |
| `share_memory` | a `shared-memory` channel among the workers |
| `HumanToolkit` | a `human` participant with `ask` / `approve` connections |
| recovery: Retry / Replan / Decompose | `policies.on_failure` |
| `task_timeout_seconds` | `restrictions.budget.wall_clock` |
| `ModelFactory` platform + type | `model: provider/model` |
| toolkits, MCP toolkits | `tools` |

What CAMEL defines in Python code (custom toolkits, callables) is referenced from
`extensions.ai.camel` by import path; the adapter warns where a round trip is lossy.

### A2A

`madl export a2a-cards` writes one Agent Card per agent with `remote-a2a` or an A2A connection:
name, description from instructions' first paragraph, skills, and the endpoint.

## 14. Security considerations

1. A MADL document can contain commands (checks, guards) and instructions. Opening one from
   elsewhere is like opening a script: runners must show its commands, tools, network access
   and restrictions, and ask before the first run.
2. Secrets are references (`{ secret: NAME }`, resolved by the runner), never values.
3. `$ref` may not leave the document's folder or fetch over the network without the reader's
   opt-in.
4. A runner that cannot enforce a restriction says so (§11); it never silently ignores one.
5. Signing documents is reserved for a later revision (`signatures` is a reserved top-level key).

## 15. Repository

```
README.md  LICENSE (Apache-2.0)  LICENSE-SPEC (CC-BY-4.0)  CHANGELOG.md
GOVERNANCE.md  CONTRIBUTING.md  SECURITY.md  CODE_OF_CONDUCT.md
spec/draft/            index, document, participants, settings, communication, topology,
                       policies, layout, extensions, conformance, security, interop
schema/draft/          schema.json (generated)
examples/              pipeline, supervisor, debate, role-play, coding team, CAD review,
                       textbook team, human approval, open swarm with bans
conformance/           valid/ invalid/ resolve/
crates/madl/   crates/madl-cli/   bindings/typescript/
docs/                  site sources; design/ (this document)
meps/                  MADL Enhancement Proposals: 0001-core.md is this design once accepted
.github/workflows/     schema generation check, example validation, conformance per language,
                       markdown lint, link check
```

## 16. Testing

- A test regenerates `schema.json` and the TypeScript types and fails if the checked-in copies differ.
- Every file in `examples/` validates; every file in `conformance/invalid/` fails with its
  expected code and path.
- `conformance/resolve/` fixtures run against the crate (and every later implementation).
- Round trip: Sculptured's team fixtures convert to MADL and back without loss (extensions
  included).
- No model is called in any test.

## 17. Milestones

1. Spec text, the crate's types, generated `schema.json`, examples, conformance fixtures.
2. Validate, resolve, expand, graph; the `madl` CLI.
3. Sculptured: team/v1 ↔ MADL converter, then Sculptured reads and writes MADL natively.
4. Docs site, then the first dated revision.
5. Later: Python and TypeScript packages, the CAMEL adapter.

## 18. Changes made while building the reference implementation

The [spec text](../../spec/draft/index.md) governs where it differs from this design. These were
settled while implementing it:

1. **Plain ids everywhere.** Ids are unique across the whole document, so bans and connections name
   groups by id (`design`, not `group:design`); `*` in a ban means anyone.
2. **Inputs connect only through `inputs[].to`.** An input id in a connection is an error.
3. **Shorthand.** `runtime`, `model` and `resume` on an agent are shorthand for its preferences;
   setting both is an error.
4. **Preferences** are `runtime`, `model`, `protocol`, `resume`, `language`, `questions` and
   `options`. `instructions_append` was dropped: appending would contradict "the nearest value
   wins".
5. **Tools** use `allow` and `deny` lists with no `mode`: `allow` absent means every tool given.
6. **A channel's people** are reached through `reach` (`{ slack: "#…" }`), so `via` only ever
   names a channel.
7. **Layout** keeps agents, steps and inputs in one `nodes` map.
8. **Steps** are one object with `kind`; which fields each kind needs is a validation rule.
9. **Subagents run on their parent's runtime** unless the template or settings name another, and
   are one `max_depth` level further down.
10. **Patterns** mark the connections they produce with `pattern: <id>`; a connection the author
    already declared is kept, not repeated.
11. **The reference implementation does not enable `serde_json`'s `preserve_order`**, so a host
    that depends on it keeps its own JSON key order.
