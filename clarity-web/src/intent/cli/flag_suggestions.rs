//! Flag suggestion utilities for unknown flags
//!
//! Provides "did you mean" suggestions using Levenshtein distance.
//!
//! ## Example
//! ```ignore
//! let suggestion = suggest_flag("--verbsoe", 2);
//! // Returns: "--verbose"
//! ```
//!
//! Ported from intent-cli/src/intent/flag_suggestions.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;

/// Default maximum edit distance for suggestions
/// Flags within this distance will be suggested as corrections
const DEFAULT_MAX_DISTANCE: usize = 2;

/// All known boolean flags
const BOOL_FLAGS: &[&str] = &[
  "help",
  "json",
  "verbose",
  "quiet",
  "yes",
  "draft",
  "confirm",
  "dry-run",
  "execute",
  "no-config",
  "parallel",
  "continue-on-error",
];

/// All known value flags
const VALUE_FLAGS: &[&str] = &[
  "only",
  "profile",
  "resume",
  "answer",
  "bead-id",
  "status",
  "reason",
  "session",
  "format",
  "notes",
  "strategy",
  "output",
  "out",
  "name",
  "target",
  "feature",
  "export-answers-template",
  "vision",
  "dir",
];

/// Get all known flag names (without -- prefix)
#[must_use]
pub fn get_all_known_flags() -> Vec<&'static str> {
  BOOL_FLAGS
    .iter()
    .chain(VALUE_FLAGS.iter())
    .copied()
    .collect()
}

/// Calculate Levenshtein distance between two strings
/// Uses Wagner-Fisher algorithm with dynamic programming.
///
/// The distance is the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into the other.
///
/// ## Performance
/// Time complexity: O(m*n) where m and n are the lengths of the strings
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
  let a_chars: Vec<char> = a.chars().collect();
  let b_chars: Vec<char> = b.chars().collect();

  let a_len = a_chars.len();
  let b_len = b_chars.len();

  // Handle empty string cases
  if a_len == 0 {
    return b_len;
  }
  if b_len == 0 {
    return a_len;
  }

  // Use two rows for space efficiency
  let mut prev_row: Vec<usize> = (0..=b_len).collect();
  let mut curr_row: Vec<usize> = vec![0; b_len + 1];

  for (i, a_char) in a_chars.iter().enumerate() {
    curr_row[0] = i + 1;

    for (j, b_char) in b_chars.iter().enumerate() {
      let cost = if a_char == b_char { 0 } else { 1 };

      curr_row[j + 1] = (prev_row[j + 1] + 1) // deletion
        .min(curr_row[j] + 1) // insertion
        .min(prev_row[j] + cost); // substitution
    }

    std::mem::swap(&mut prev_row, &mut curr_row);
  }

  prev_row[b_len]
}

/// Find flags within edit distance threshold
#[must_use]
pub fn find_similar_flags(input: &str, max_distance: usize) -> Vec<&'static str> {
  get_all_known_flags()
    .into_iter()
    .filter(|flag| levenshtein(input, flag) <= max_distance)
    .collect()
}

/// Find the closest matching flag for an unknown flag
/// Returns the suggestion with "--" prefix, or empty string if no good match
#[must_use]
pub fn suggest_flag(unknown_flag: &str, max_distance: usize) -> String {
  // Check if it starts with --
  if !unknown_flag.starts_with("--") {
    return String::new();
  }

  let flag_name = &unknown_flag[2..];

  // First check if it's an exact match (no suggestion needed)
  if get_all_known_flags().contains(&flag_name) {
    return String::new();
  }

  // Find similar flags
  let similar = find_similar_flags(flag_name, max_distance);

  if similar.is_empty() {
    return String::new();
  }

  // Find the closest match by sorting by distance
  similar
    .into_iter()
    .min_by_key(|flag| levenshtein(flag_name, flag))
    .map(|s| format!("--{s}"))
    .unwrap_or_default()
}

/// Format error message with suggestion
#[must_use]
pub fn format_suggestion(unknown_flag: &str, suggestion: &str) -> String {
  if suggestion.is_empty() {
    format!("Unknown flag: {unknown_flag}")
  } else {
    format!("Unknown flag '{unknown_flag}'. Did you mean '{suggestion}'?")
  }
}

/// Check if an argument is a flag
fn is_flag(arg: &str) -> bool {
  arg.starts_with("--")
}

/// Extract flag name from argument (removes -- prefix)
fn extract_flag_name(arg: &str) -> &str {
  if arg.starts_with("--") {
    &arg[2..]
  } else {
    ""
  }
}

/// Validate all flags in the argument list
/// Returns `Ok(())` if all flags are valid, `Err(message)` if unknown flag found
///
/// # Errors
/// Returns an error message string if an unknown flag is found.
pub fn validate_flags(args: &[String]) -> Result<(), String> {
  // Find all flags in the args
  let flags: Vec<&str> = args
    .iter()
    .filter(|arg| is_flag(arg))
    .map(|arg| extract_flag_name(arg))
    .filter(|name| !name.is_empty())
    .collect();

  // Find first unknown flag
  for unknown_flag in flags {
    if !get_all_known_flags().contains(&unknown_flag) {
      let suggestion = suggest_flag(&format!("--{unknown_flag}"), DEFAULT_MAX_DISTANCE);
      return Err(format_suggestion(&format!("--{unknown_flag}"), &suggestion));
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_all_known_flags_includes_common_flags() {
    let flags = get_all_known_flags();
    assert!(flags.contains(&"help"));
    assert!(flags.contains(&"verbose"));
    assert!(flags.contains(&"profile"));
    assert!(flags.contains(&"format"));
  }

  #[test]
  fn test_levenshtein_identical_strings() {
    assert_eq!(levenshtein("hello", "hello"), 0);
    assert_eq!(levenshtein("", ""), 0);
  }

  #[test]
  fn test_levenshtein_empty_string() {
    assert_eq!(levenshtein("hello", ""), 5);
    assert_eq!(levenshtein("", "world"), 5);
  }

  #[test]
  fn test_levenshtein_single_substitution() {
    assert_eq!(levenshtein("cat", "bat"), 1);
    assert_eq!(levenshtein("hello", "hallo"), 1);
  }

  #[test]
  fn test_levenshtein_single_insertion() {
    assert_eq!(levenshtein("cat", "cats"), 1);
    assert_eq!(levenshtein("hello", "hell"), 1);
  }

  #[test]
  fn test_levenshtein_single_deletion() {
    assert_eq!(levenshtein("cats", "cat"), 1);
    assert_eq!(levenshtein("hello", "helo"), 1);
  }

  #[test]
  fn test_levenshtein_multiple_edits() {
    assert_eq!(levenshtein("kitten", "sitting"), 3);
    assert_eq!(levenshtein("saturday", "sunday"), 3);
  }

  #[test]
  fn test_levenshtein_case_sensitive() {
    assert_eq!(levenshtein("Hello", "hello"), 1);
  }

  #[test]
  fn test_find_similar_flags_exact_match() {
    let similar = find_similar_flags("verbose", 0);
    assert!(similar.contains(&"verbose"));
  }

  #[test]
  fn test_find_similar_flags_typo() {
    let similar = find_similar_flags("verbsoe", 2);
    assert!(similar.contains(&"verbose"));
  }

  #[test]
  fn test_find_similar_flags_too_far() {
    let similar = find_similar_flags("completelydifferent", 2);
    assert!(similar.is_empty());
  }

  #[test]
  fn test_suggest_flag_exact_match() {
    let suggestion = suggest_flag("--verbose", 2);
    assert!(suggestion.is_empty()); // No suggestion for exact match
  }

  #[test]
  fn test_suggest_flag_typo() {
    let suggestion = suggest_flag("--verbsoe", 2);
    assert_eq!(suggestion, "--verbose");
  }

  #[test]
  fn test_suggest_flag_no_prefix() {
    let suggestion = suggest_flag("verbose", 2);
    assert!(suggestion.is_empty()); // No suggestion without -- prefix
  }

  #[test]
  fn test_suggest_flag_unknown_too_far() {
    let suggestion = suggest_flag("--xyzzy", 2);
    assert!(suggestion.is_empty()); // No suggestion if too far
  }

  #[test]
  fn test_format_suggestion_with_suggestion() {
    let msg = format_suggestion("--verbsoe", "--verbose");
    assert!(msg.contains("--verbsoe"));
    assert!(msg.contains("--verbose"));
    assert!(msg.contains("Did you mean"));
  }

  #[test]
  fn test_format_suggestion_without_suggestion() {
    let msg = format_suggestion("--unknown", "");
    assert!(msg.contains("Unknown flag"));
    assert!(msg.contains("--unknown"));
    assert!(!msg.contains("Did you mean"));
  }

  #[test]
  fn test_is_flag() {
    assert!(is_flag("--verbose"));
    assert!(is_flag("--help"));
    assert!(!is_flag("verbose"));
    assert!(!is_flag("-v"));
  }

  #[test]
  fn test_extract_flag_name() {
    assert_eq!(extract_flag_name("--verbose"), "verbose");
    assert_eq!(extract_flag_name("verbose"), "");
  }

  #[test]
  fn test_validate_flags_all_valid() {
    let args = vec![
      "--verbose".to_string(),
      "--format".to_string(),
      "json".to_string(),
    ];
    assert!(validate_flags(&args).is_ok());
  }

  #[test]
  fn test_validate_flags_unknown_flag() {
    let args = vec!["--verbsoe".to_string()];
    let result = validate_flags(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("--verbsoe"));
  }

  #[test]
  fn test_validate_flags_no_flags() {
    let args = vec!["file.cue".to_string()];
    assert!(validate_flags(&args).is_ok());
  }

  #[test]
  fn test_validate_flags_empty_args() {
    let args: Vec<String> = Vec::new();
    assert!(validate_flags(&args).is_ok());
  }
}
