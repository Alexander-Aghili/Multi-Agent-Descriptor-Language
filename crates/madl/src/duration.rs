//! Durations as written in documents: `90s`, `30m`, `2h`, `1d`, `1h30m`.

/// Seconds, or `None` when the text is not a duration.
pub fn seconds(text: &str) -> Option<u64> {
    let mut total: u64 = 0;
    let mut number = String::new();
    let mut seen_unit = false;
    for c in text.trim().chars() {
        if c.is_ascii_digit() {
            number.push(c);
            continue;
        }
        let unit = match c {
            's' => 1,
            'm' => 60,
            'h' => 3_600,
            'd' => 86_400,
            _ => return None,
        };
        let n: u64 = number.parse().ok()?;
        total = total.checked_add(n.checked_mul(unit)?)?;
        number.clear();
        seen_unit = true;
    }
    (number.is_empty() && seen_unit).then_some(total)
}

#[cfg(test)]
mod tests {
    use super::seconds;

    #[test]
    fn reads_units_and_compounds() {
        assert_eq!(seconds("90s"), Some(90));
        assert_eq!(seconds("1h30m"), Some(5_400));
        assert_eq!(seconds("2d"), Some(172_800));
    }

    #[test]
    fn rejects_bare_numbers_and_unknown_units() {
        assert_eq!(seconds("30"), None);
        assert_eq!(seconds("5w"), None);
        assert_eq!(seconds(""), None);
    }
}
