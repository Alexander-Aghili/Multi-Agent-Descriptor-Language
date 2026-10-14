//! Lexical path sets: a path covers itself and everything under it.

/// `a/b/../c` → `["a", "c"]`; `.` → `[]`.
fn parts(path: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            p => out.push(p),
        }
    }
    out
}

/// Whether `path` is `outer` or inside it.
pub fn within(path: &str, outer: &str) -> bool {
    if path.starts_with('/') != outer.starts_with('/') {
        return false;
    }
    let (p, o) = (parts(path), parts(outer));
    p.len() >= o.len() && p[..o.len()] == o[..]
}

/// The paths both sets cover.
pub fn intersect(a: &[String], b: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let keep = |p: &String, other: &[String]| other.iter().any(|q| within(p, q));
    for p in b.iter().filter(|p| keep(p, a)).chain(a.iter().filter(|p| keep(p, b))) {
        if !out.contains(p) {
            out.push(p.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_covers_what_is_under_it() {
        assert!(within("./out/report.md", "."));
        assert!(within("out/a", "./out"));
        assert!(!within("outside", "out"));
        assert!(!within("/etc", "."));
    }

    #[test]
    fn intersection_keeps_the_narrower_paths() {
        let a = vec![".".to_string()];
        let b = vec!["./out".to_string()];
        assert_eq!(intersect(&a, &b), vec!["./out".to_string()]);
    }
}
