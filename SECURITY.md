# Security

## Reporting a vulnerability

Please report security problems privately through GitHub's
[private vulnerability reporting](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language/security/advisories/new),
not in public issues. You will get an acknowledgement within a few days.

In scope: the reference implementation (for example a `$ref` that escapes the document's folder,
or input that makes the loader hang or crash) and the specification (for example a rule that lets
a document loosen a restriction it should only tighten).

## Running MADL files from elsewhere

A MADL document can contain commands (checks, guards) and instructions for agents. Treat a
document from someone else like a script: runners must show its commands, tools, network access
and restrictions and ask before the first run (spec: conformance and security). Secrets are never
written into documents; they are references the runner resolves.
