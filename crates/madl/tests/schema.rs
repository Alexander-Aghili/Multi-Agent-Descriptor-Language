//! The checked-in schema matches the types. Regenerate with `UPDATE_SCHEMAS=1 cargo test`.
use std::path::Path;

#[test]
fn the_checked_in_schema_matches_the_types() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema/draft/schema.json");
    let text = madl::schema::json();
    if std::env::var_os("UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &text).unwrap();
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(on_disk == text, "schema/draft/schema.json is stale: run with UPDATE_SCHEMAS=1");
}
