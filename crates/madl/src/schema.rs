//! The JSON Schema of a MADL document (after `$ref` resolution).
use crate::model::Document;

/// The schema as pretty JSON, with a trailing newline.
pub fn json() -> String {
    let mut schema = schemars::schema_for!(Document);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/Alexander-Aghili/Multi-Agent-Descriptor-Language/main/schema/draft/schema.json".into());
    schema.insert("title".into(), "MADL document (draft)".into());
    let mut text = serde_json::to_string_pretty(&schema).expect("a schema serialises");
    text.push('\n');
    text
}
