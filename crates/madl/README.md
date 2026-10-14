# madl

The reference implementation of the [Multi-Agent Descriptor Language](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language):
document types, a loader (YAML, JSON, `$ref`), a validator with stable error codes, pattern
expansion, settings resolution and Mermaid output.

```rust
let doc = madl::load(std::path::Path::new("team.madl.yaml"))?;
let report = madl::validate(&doc);
for d in &report.diagnostics {
    eprintln!("{d}"); // error[restriction-loosened] /agents/1/settings/restrictions/budget/tokens: …
}
let expanded = madl::expand(&doc);
let settings = madl::resolve(&expanded);           // every agent's effective settings, with sources
let chart = madl::mermaid::flowchart(&expanded);   // a Mermaid flowchart
```

The types generate the published JSON Schema (`schemars`) and TypeScript types (`ts-rs`).

License: Apache-2.0.
