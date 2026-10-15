# Settings

## Settings blocks

The same shape appears at the root (`defaults`), on a group (`settings`), on an agent
(`settings`), and for what an agent spawns (`subagents.settings`):

```yaml
settings:
  preferences:            # overridable choices: the nearest value wins
    runtime: claude-code
    model: anthropic/claude-sonnet-5-5
    protocol: a2a@1.2
    resume: resume-if-possible
    language: en
    questions: { timeout: 30m, on_timeout: continue }
    options: { effort: high }
  restrictions:           # limits: they only tighten
    tools: { allow: [fs, search], deny: [shell] }
    files: { read: ["."], write: ["./out"] }
    network: { egress: [pypi.org, github.com] }
    budget: { tokens: 2000000, cost_usd: 20, wall_clock: 2h, scope: shared }
    spawn: { allowed: true, max_children: 4, max_depth: 2 }
    approvals: { required_for: [shell, network, write-outside-workspace] }
    communication: { mode: closed, deny: [ { from: cad, to: customer } ] }
  applies_to: { preferences: self, restrictions: descendants }
```

Every member is optional. An absent value is inherited. A document with no settings at all has no
limits: MADL sets no limit the author did not set.

### Preferences

| Member | |
|---|---|
| `runtime` | See [Agents](document.md#agents) |
| `model` | `provider/model`, or `{ id, fallback: [ … ] }` tried in order |
| `protocol` | The default [protocol](communication.md#protocols) |
| `resume` | `resume-if-possible`, `relaunch-fresh`, `checkpoint-only` |
| `language` | A BCP 47 tag |
| `questions` | `timeout` (a duration) and `on_timeout`: `continue`, `wait`, `stop` |
| `options` | Runtime-specific options, merged key by key |

Durations are written `90s`, `30m`, `2h`, `1d` or combined, as in `1h30m` (`bad-duration`).

### Restrictions

| Member | Meaning | Combines down the chain by | Loosening |
|---|---|---|---|
| `tools.allow` | Only these tools (absent: every tool the agent is given) | intersection | adding a tool an outer block does not allow, or one it denies |
| `tools.deny` | Never these tools | union | (cannot happen) |
| `files.read`, `files.write` | Paths (and everything under them) the agent may read or write | intersection of path sets | a path outside every outer path |
| `network.egress` | `deny`, a list of hosts, or `allow` (ordered tightest first) | the tighter; host lists intersect | a looser rule, or a host an outer list lacks |
| `budget.tokens`, `cost_usd`, `wall_clock`, `turns`, `working_time` | [Limits](#limits) | minimum (hard and soft values each) | a larger hard value |
| `budget.scope` | `shared`: the agent and what it spawns draw from one budget; `own`: each has its own | `shared` wins | `own` under `shared` |
| `spawn.allowed` | Whether the agent may start subagents | AND | `true` under `false` |
| `spawn.max_children`, `max_depth` | How many, and how many levels below the agent | minimum | a larger value |
| `approvals.required_for` | Actions that need a person's approval first | union | (cannot happen) |
| `communication` | [Mode and bans](communication.md#open-and-closed-communication) | `closed` wins; bans add up | `open` under `closed` |

Tools are named by tool id (`fs`), `<tool id>/<tool name>` for one MCP tool, or
`<runtime>:<name>` for a runtime's own tool. Effective allowed tools are the allow list minus the
deny list.

### Limits

A limit is a number or duration (its hard value) or `{ soft, hard }`. At the soft value a runtime
SHOULD start no new work for the agent and ask it to finish with a summary; at the hard value it MUST
stop the agent and what it started. Without `soft` a runtime may choose one. `soft` above `hard` is
`budget-soft-above-hard`. `turns` counts the agent's turns in one run; `working_time` is time spent
in turns. A budget for the whole run is [`system.budget`](document.md#run-budget).

A block that tries to loosen an inherited restriction is an error (`restriction-loosened`), reported
at the block's value. Resolution keeps the tighter value regardless. A runner meeting a loosening
at run time MUST keep the tighter value and SHOULD record that it did.

## Reach

`applies_to` sets how far a block's preferences and restrictions reach:

| Value | Reaches |
|---|---|
| `self` | The agents the block covers, not what they spawn |
| `subagents` | Only what those agents spawn, and what that spawns |
| `descendants` | The agents and everything they spawn |

Defaults: preferences `self`; restrictions `descendants`. So a restriction cannot be escaped by
spawning, and a preference does not leak into helpers unless the author says so.

## Subagents

```yaml
agents:
  - id: coordinator
    subagents:
      template:                     # what a spawned agent starts as
        instructions: You are a focused helper for one subtask.
        runtime: claude-code
        model: anthropic/claude-haiku-5-5
        tools: [ … ]
      settings:                     # settings for spawned agents only
        restrictions: { tools: { allow: [search] } }
```

A spawned agent runs on its parent's runtime unless its template or settings name another.

## Resolution

The effective settings of agent *A* are computed by applying these blocks in order, each only
where its reach covers *A* itself (`self` or `descendants`):

1. `defaults`
2. the root `communication`, as a restriction
3. *A*'s groups, each from its outermost enclosing group inwards; for groups that are not nested,
   in the order *A* lists them; a group applies once
4. *A*'s own `settings`
5. *A*'s shorthand (`runtime`, `model`, `resume`)

The effective settings of what *A* spawns are computed by applying the same blocks, each only
where its reach covers subagents (`subagents` or `descendants`), then *A*'s `subagents.settings`
and the template's `runtime` and `model`. Then: if no runtime is set, *A*'s runtime is used, and
`spawn.max_depth` is one less (at 0, `spawn.allowed` is `false`).

Preferences merge as follows: a scalar or list replaces the inherited value; `options` merges key
by key, recursively. Restrictions combine as in the table above.

A Resolver reports, for every effective value, the JSON Pointer of the block value that set it
last. `conformance/resolve/` pins this output.
