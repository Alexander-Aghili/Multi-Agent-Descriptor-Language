# MEP 0003: Data access, authority, and acceptance checks

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

A document says which tools an agent may use and, with `restrictions.files`, which paths it may
touch at most. It does not say what the agent is *for*: which data it reads and produces, which
effects it has beyond its files, and which decisions are its to make. In a team that is where work
collides. Two agents rewrite the same files, or both decide the same question, and nothing in the
design shows it.

Outputs have the opposite gap. An output names a file, but not how anyone knows it is done, so
"done" is whatever an agent or a reviewer says.

These ideas come from Pact (github.com/wandercom/pact), which gives each component a contract with
`data_access` and `authority` before any implementation, and treats tests, not reviewers, as the
judge of done.

## Proposal

An agent may declare its data contract and the decisions it owns:

```yaml
agents:
  - id: wings
    data_access:
      reads: [manuals/, specs/wings.md]
      writes: [cad/wings/]
      side_effects: [none]
      rationale: Only the wing team changes wing geometry.
    authority:
      domains: [wing geometry]
      rationale: It owns the wing model.
    outputs:
      - id: wing-step
        path: cad/wings/wing.step
        acceptance: [ { command: python check_wing.py, timeout_s: 120 } ]
```

- **`data_access`**: paths are relative to the document. A trailing `/` or `/**` covers a folder.
  `side_effects` names effects beyond files in plain words. A runner SHOULD refuse a write outside
  `writes` where it can, and MUST tell the person where it cannot (for example, writes made by a
  shell command). `restrictions.files` remains the hard limit, and the two combine by tightening.
- **`authority`**: domains are plain words. A runner SHOULD tell every agent who decides what, and
  route a question within a domain to its owner.
- **`outputs[].acceptance`**: checks with the same shape as `checks`. The output is done when every
  one exits 0. A runner MUST NOT report an output as verified without running them.

New warnings: `write-overlap` (two agents may write the same path) and `authority-overlap` (two
agents own the same domain). Both are warnings because a shared folder or a shared decision can be
intended. The author should still know about it.

## Compatibility

All three fields are optional. Documents without them mean what they meant before.
