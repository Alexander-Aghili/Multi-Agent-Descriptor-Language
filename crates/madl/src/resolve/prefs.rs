//! Preferences: the nearest value wins; `options` merges key by key.
use super::Ctx;
use crate::model::Preferences;
use serde_json::{Map, Value};

/// Apply `new` over `acc`, recording each value's source.
pub fn merge(acc: &mut Preferences, new: &Preferences, ctx: &mut Ctx) {
    let mut a = to_map(acc);
    for (key, value) in to_map(new) {
        let field = format!("preferences.{key}");
        if key == "options" {
            let slot = a.entry(key).or_insert_with(|| Value::Object(Map::new()));
            deep(slot, value, &field, ctx);
        } else {
            ctx.source(&field);
            a.insert(key, value);
        }
    }
    *acc = serde_json::from_value(Value::Object(a)).expect("merged preferences keep their shape");
}

fn deep(slot: &mut Value, value: Value, field: &str, ctx: &mut Ctx) {
    match (slot, value) {
        (Value::Object(into), Value::Object(from)) => {
            for (k, v) in from {
                let inner = format!("{field}.{k}");
                let entry = into.entry(k).or_insert(Value::Null);
                if entry.is_object() && v.is_object() {
                    deep(entry, v, &inner, ctx);
                } else {
                    ctx.source(&inner);
                    *entry = v;
                }
            }
        }
        (slot, value) => {
            ctx.source(field);
            *slot = value;
        }
    }
}

fn to_map(p: &Preferences) -> Map<String, Value> {
    match serde_json::to_value(p) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}
