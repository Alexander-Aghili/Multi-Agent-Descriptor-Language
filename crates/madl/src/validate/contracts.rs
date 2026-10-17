//! Data contracts and authority (spec: Agents, data access and authority): two agents that may write
//! the same paths, or that both own a decision, are warned about. Either may be intended; the
//! author should know.
use crate::diagnostic::{Diagnostic, Report};
use crate::model::Document;

/// A write path as a comparable prefix: `./a/b/`, `a/b/**` and `a/b` all become `a/b`.
fn prefix(path: &str) -> String {
    let p = path.trim().trim_start_matches("./");
    let p = p.strip_suffix("/**").or_else(|| p.strip_suffix("/*")).unwrap_or(p);
    p.trim_end_matches('/').to_string()
}

/// True when one path covers the other.
fn overlaps(a: &str, b: &str) -> bool {
    let (a, b) = (prefix(a), prefix(b));
    a == b || a.is_empty() || b.is_empty() || b.starts_with(&format!("{a}/")) || a.starts_with(&format!("{b}/"))
}

/// An authority's `scope` names a group, or `system`.
fn scopes(doc: &Document, report: &mut Report) {
    for (i, agent) in doc.agents.iter().enumerate() {
        if let Some(scope) = agent.authority.as_ref().and_then(|a| a.scope.as_deref())
            && scope != "system"
            && !doc.groups.iter().any(|g| g.id == scope)
        {
            report.push(Diagnostic::error("unknown-reference", format!("/agents/{i}/authority/scope"), format!("`{scope}` is not a group (or `system`)")));
        }
    }
}

pub fn check(doc: &Document, report: &mut Report) {
    scopes(doc, report);
    for (i, agent) in doc.agents.iter().enumerate() {
        for earlier in doc.agents.iter().take(i) {
            if let (Some(mine), Some(theirs)) = (&agent.data_access, &earlier.data_access) {
                for (k, w) in mine.writes.iter().enumerate() {
                    if let Some(other) = theirs.writes.iter().find(|o| overlaps(w, o)) {
                        report.push(Diagnostic::warning(
                            "write-overlap",
                            format!("/agents/{i}/data_access/writes/{k}"),
                            format!("`{w}` overlaps `{other}`, which `{}` also writes", earlier.id),
                        ));
                    }
                }
            }
            if let (Some(mine), Some(theirs)) = (&agent.authority, &earlier.authority) {
                for (k, d) in mine.domains.iter().enumerate() {
                    if theirs.domains.iter().any(|o| o.trim().eq_ignore_ascii_case(d.trim())) {
                        report.push(Diagnostic::warning(
                            "authority-overlap",
                            format!("/agents/{i}/authority/domains/{k}"),
                            format!("`{d}` is also decided by `{}`", earlier.id),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::overlaps;

    #[test]
    fn folders_cover_what_is_inside_them_and_nothing_else() {
        assert!(overlaps("cad/wings/", "cad/wings/spar.step"));
        assert!(overlaps("./cad/wings/**", "cad/wings"));
        assert!(!overlaps("cad/wings/", "cad/wingsuit/"));
        assert!(!overlaps("cad/wings/", "assembly/"));
    }
}
