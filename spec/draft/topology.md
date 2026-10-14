# Topology

## The graph

Agents and steps are the nodes; connections are the edges, with groups standing for their members
and `direction: both` counted in both directions. Work moves along `handoff`, `delegate` and
`review` connections.

## Patterns

Patterns are shorthand for common shapes. A reader expands them into connections (and, for
`blackboard`, a channel) before applying the graph rules; `madl expand` prints the result.

```yaml
topology:
  patterns:
    - { id: assign, pattern: supervisor, supervisor: coordinator, members: [searcher, writer], routing: dynamic }
    - { pattern: debate, members: [for-rust, for-go], judge: judge, max_rounds: 3 }
  loops: { max_rounds: 3 }
```

| Pattern | Roles | Expands to |
|---|---|---|
| `pipeline` | `members` (2+) | `handoff` from each member to the next |
| `supervisor` | `supervisor`, `members` | `delegate` (`routing: dynamic`, the default) or `handoff` (`static`) to each member; `report` back |
| `auction` | `auctioneer`, `members` | `delegate` to each member; `report` back |
| `committee` | `source`, `members`, optional `collector` | `handoff` from the source to each member; `report` to the collector (default: the source) |
| `debate` | `members` (2+), optional `judge` | `review` both ways between every two members with `max_rounds`; `report` to the judge |
| `role-play` | exactly two `members` | `ask` both ways with `max_rounds` |
| `swarm` | `members` (2+) | `handoff` both ways between every two members with `max_rounds` |
| `blackboard` | `members` | a `blackboard` channel the members read and write (`via` names an existing one) |

A pattern missing a role it needs is an error (`pattern-shape`). Expanded connections take the
pattern's `via` and `protocol`, carry `pattern: <pattern id>`, and have ids
`<pattern id>-<from>-<kind>-<to>`. A connection the author already declared with the same `from`,
`to` and `kind` is kept, and not repeated.

## Starting points

The starting agents are the agents an input names, and every agent that no `handoff` or `delegate`
reaches. People (`runtime: human`) start only when an input names them. A document with agents
other than people, but no starting agent, is an error (`no-starting-point`).

## Loops

A cycle of `handoff`, `delegate` or `review` connections is allowed when work can reach it from a
starting agent (along `handoff`, `delegate`, `review` or `report`). One piece of work goes round at
most the smallest `max_rounds` on the cycle, else `topology.loops.max_rounds`, else the runner's
own limit. A cycle that no work can reach has no way in, and is an error (`loop-without-start`).

A step that no work reaches is a warning (`unreachable`).

## Conditions

`when` on a connection and `over` on a fan-out step use a small expression language, a subset of
[CEL](https://cel.dev): field access (`review.outcome`), literals, `==`, `!=`, `<`, `<=`, `>`,
`>=`, `&&`, `||`, `!` and `in`. A field that is not there is false, never an error. What fields
exist (`outcome`, an agent's outputs, an input's members) is up to the runner, which SHOULD
document them.

## Policies

```yaml
policies:
  on_failure: { strategy: retry, max_attempts: 2, then: escalate }
  escalation: [ { from: cad, after_rounds: 3, to: senior-cad } ]
  schedules: [ { id: nightly, agent: reviewer, every: 24h, message: "Review today's changes." } ]
  guards: [ { command: "make health", every: 10m, timeout_s: 600 } ]
```

- `on_failure.strategy` and `then`: `retry`, `replan` (plan the remaining work again),
  `decompose` (split the failed work into smaller pieces), `escalate`, `stop`.
- `escalation` hands work from an agent or step to a stronger agent after a number of rounds.
- `schedules` send an agent a message at an interval.
- `guards` are commands that pause new work while they fail.
