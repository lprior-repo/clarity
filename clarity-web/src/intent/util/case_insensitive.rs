//! Case-insensitive string matching utilities
//!
//! Ported from intent-cli/src/intent/case_insensitive.gleam

/// Check if haystack contains needle (case-insensitive)
/// Both strings are lowercased once before comparison.
#[must_use]
pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Check if haystack contains any of the needles (case-insensitive)
/// The haystack is lowercased once, then checked against all needles.
#[must_use]
pub fn contains_any_ignore_case(haystack: &str, needles: &[&str]) -> bool {
    let lower_haystack = haystack.to_lowercase();
    needles
        .iter()
        .any(|needle| lower_haystack.contains(&needle.to_lowercase()))
}

/// Check if haystack contains all of the needles (case-insensitive)
/// The haystack is lowercased once, then checked against all needles.
#[must_use]
pub fn contains_all_ignore_case(haystack: &str, needles: &[&str]) -> bool {
    let lower_haystack = haystack.to_lowercase();
    needles
        .iter()
        .all(|needle| lower_haystack.contains(&needle.to_lowercase()))
}

/// Check if two strings are equal (case-insensitive)
#[must_use]
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Check if haystack starts with prefix (case-insensitive)
#[must_use]
pub fn starts_with_ignore_case(haystack: &str, prefix: &str) -> bool {
    haystack.to_lowercase().starts_with(&prefix.to_lowercase())
}

/// Check if haystack ends with suffix (case-insensitive)
#[must_use]
pub fn ends_with_ignore_case(haystack: &str, suffix: &str) -> bool {
    haystack.to_lowercase().ends_with(&suffix.to_lowercase())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ignore_case() {
        assert!(contains_ignore_case("Hello World", "hello"));
        assert!(contains_ignore_case("Hello World", "WORLD"));
        assert!(!contains_ignore_case("Hello World", "xyz"));
    }

    #[test]
    fn test_contains_any_ignore_case() {
        assert!(contains_any_ignore_case("Hello World", &["foo", "hello"]));
        assert!(contains_any_ignore_case("Hello World", &["WORLD", "bar"]));
        assert!(!contains_any_ignore_case("Hello World", &["foo", "bar"]));
    }

    #[test]
    fn test_contains_all_ignore_case() {
        assert!(contains_all_ignore_case("Hello World", &["hello", "world"]));
        assert!(!contains_all_ignore_case("Hello World", &["hello", "xyz"]));
    }

    #[test]
    fn test_equals_ignore_case() {
        assert!(equals_ignore_case("Hello", "hello"));
        assert!(equals_ignore_case("HELLO", "hello"));
        assert!(!equals_ignore_case("Hello", "world"));
    }

    #[test]
    fn test_starts_with_ignore_case() {
        assert!(starts_with_ignore_case("Hello World", "HELLO"));
        assert!(!starts_with_ignore_case("Hello World", "world"));
    }

    #[test]
    fn test_ends_with_ignore_case() {
        assert!(ends_with_ignore_case("Hello World", "WORLD"));
        assert!(!ends_with_ignore_case("Hello World", "hello"));
    }
}
