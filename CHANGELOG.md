# Changelog

## draft (unreleased)

The first public draft.

- Document: system, defaults, inputs, groups (nesting), agents, steps, channels, connections,
  communication, topology, policies, layout, extensions; `$ref` across files.
- Settings: preferences override, restrictions only tighten; `applies_to` reach (`self`,
  `subagents`, `descendants`); subagent templates and settings; a stated resolution order.
- Communication: protocols (`native`, `a2a@…`, `mcp@…`, `x-…`), channels with mediums and access,
  connections with kind, direction, initiator, mode, contents, round limits and conditions; open or
  closed mode; bans that always win.
- Topology: patterns that expand into connections; starting points; loops allowed when work can
  reach them.
- Reference implementation (`madl` crate and CLI), JSON Schema, TypeScript bindings, examples and
  conformance fixtures.
