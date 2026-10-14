# Communication

## Protocols

A protocol is named `native` (the runtime's own messaging, such as Claude Code's subagent tool),
`a2a`, `mcp` (one agent exposed to another as a tool), or `x-<vendor>-<name>`, each optionally
followed by `@<version>`: `a2a@1.2`, `mcp@2025-06-18` (`unknown-protocol`). The default comes from
`preferences.protocol`; a channel or connection MAY override it.

## Channels

A channel is where messages or artifacts travel:

```yaml
channels:
  - { id: review-chat, medium: direct, protocol: a2a@1.2 }
  - { id: shared, medium: filesystem, path: ./cad, access: { cad: read-write, reviewer: read } }
  - { id: findings, medium: bus, topic: findings }
  - { id: lead-desk, medium: human, reach: { slack: "#cad-review" } }
```

Mediums are `direct`, `filesystem`, `shared-memory`, `blackboard`, `bus`, `git`, `tracker`, `human`,
or `x-<vendor>-<name>` (`unknown-medium`). `access` gives agents or groups `read`, `write` or
`read-write` rights; without it, a channel's members are the agents whose connections use it.

## Connections

A connection is a directed, committed line of communication. Different pairs of agents may talk in
different ways, so every detail belongs to the connection:

```yaml
connections:
  - id: cad-to-review
    from: cad
    to: reviewer
    kind: handoff
    direction: one-way        # or both
    via: shared               # a channel
    protocol: a2a@1.2         # overrides the channel's
    initiates: from           # from | to | either
    mode: async               # async | sync (the sender waits for a reply)
    carries: [ { artifact: model }, { message: summary } ]
    max_rounds: 3             # for loops through this connection
    when: "review.outcome == 'changes'"
```

`from` and `to` name agents, groups or steps (`unknown-reference`, `wrong-kind`); a group means
every member. A connection from a node to itself is an error (`self-connection`).

| Kind | Meaning |
|---|---|
| `handoff` | Work passes to `to`, which starts on it |
| `report` | `from` sends results or status to `to` |
| `watch` | `from` may read `to`'s activity; it sends nothing |
| `delegate` | `from` may assign work to `to` while the system runs, choosing among its delegates |
| `ask` | `from` may consult `to` and wait for an answer; ownership does not move |
| `review` | `to` returns approve or changes-requested; changes go back to `from` |
| `approve` | `to` (usually a person) must approve before `from` continues |

`direction: both` is the same connection in both directions.

## Open and closed communication

```yaml
communication:
  mode: closed
  deny:
    - { from: reviewer, to: planner }                  # one direction
    - { between: [billing, triage], kinds: [handoff] } # both directions, one kind
    - { from: "*", to: auditor, kinds: [delegate] }    # anyone
```

1. In `closed` mode an agent MAY communicate only along its declared connections, in their
   direction, with their kind and details.
2. In `open` mode undeclared communication is allowed, with the default protocol over a direct
   channel.
3. A ban always wins. A ban names agents or groups (`*` is anyone) as `from` and `to`, or as a pair
   in `between` (`deny-shape`); `kinds` limits it to those kinds. A declared connection that a ban
   matches is an error (`connection-denied`).
4. Mode and bans MAY also be set as a [restriction](settings.md#restrictions) on groups and agents:
   an agent can go from open to closed and add bans, never the reverse.
5. A subagent MAY communicate with the agent that started it; anything else follows the rules
   above with the subagent's effective settings.

Runners MUST enforce the mode and bans they accept, or refuse the document (see
[Conformance](conformance.md#runner)).
