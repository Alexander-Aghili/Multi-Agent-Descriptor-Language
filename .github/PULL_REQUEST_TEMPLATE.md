## What this changes

## Checklist

- [ ] `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` pass
- [ ] A change to the language has an accepted MEP and updates `spec/draft/`, the types (with `UPDATE_SCHEMAS=1 cargo test`), `conformance/` and `CHANGELOG.md` together
- [ ] New rules have an invalid fixture with the expected code and path
