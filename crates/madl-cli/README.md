# madl-cli

The `madl` command for [MADL](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language) documents.

```sh
cargo install --git https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language madl-cli

madl validate team.madl.yaml        # exit 1 on errors; --json for JSON lines
madl resolve  team.madl.yaml        # effective settings per agent, with sources
madl expand   team.madl.yaml        # $refs resolved, patterns expanded (--json for JSON)
madl graph    team.madl.yaml        # Mermaid flowchart
madl schema                         # the JSON Schema
```

`--allow-outside` lets `$ref` reach files outside the document's folder.

License: Apache-2.0.
