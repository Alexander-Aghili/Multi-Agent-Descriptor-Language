# Governance

MADL is maintained by its maintainers (currently the repository owner) with the aim of moving to a
neutral home, such as a foundation or a multi-member organisation, before the first dated
revision.

## Decisions

- **Editorial changes** (wording, examples, fixtures that pin existing behaviour) need one
  maintainer's approval.
- **Language changes** go through a MADL Enhancement Proposal (MEP) in `meps/`. A MEP is accepted
  by maintainer consensus after a public comment period of at least 14 days on its issue.
- Implementers' experience counts: a MEP that cannot be implemented in the reference crate and pass
  the conformance fixtures is not accepted.

## Revisions

- `draft` changes as MEPs are accepted.
- A dated revision (for example `2027-01-15`) is a frozen copy of the draft: its spec text, schema
  and fixtures never change afterwards, apart from errata that do not change meaning.
- Readers state which revisions they support; documents name the revision they were written for in
  `madl`.

## Compatibility

Within a revision, a document that is valid stays valid. Across revisions, changes that make valid
documents invalid or change their meaning are listed in `CHANGELOG.md` with a migration note.
