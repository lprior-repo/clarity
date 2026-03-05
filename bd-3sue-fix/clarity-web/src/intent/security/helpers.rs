#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::constants::{CONTROL_CHAR_MAX, SHELL_METACHARACTERS};
use super::types::{MetacharCategory, PathEncodingType, RegexVulnerability};

pub(super) fn is_shell_metachar(ch: char) -> bool {
  SHELL_METACHARACTERS.contains(&ch)
}

pub(super) const fn classify_metachar(ch: char) -> Option<MetacharCategory> {
  match ch {
    ';' | '|' | '&' => Some(MetacharCategory::CommandSeparator),
    '$' | '`' => Some(MetacharCategory::VariableExpansion),
    '(' | ')' | '{' | '}' | '[' | ']' => Some(MetacharCategory::Grouping),
    '<' | '>' => Some(MetacharCategory::Redirection),
    '\\' | '!' | '*' | '?' | '"' | '\'' => Some(MetacharCategory::EscapeQuote),
    _ => None,
  }
}

pub(super) fn is_control_character(ch: char) -> bool {
  (ch as u32) < u32::from(CONTROL_CHAR_MAX)
}

pub(super) fn contains_encoded_traversal(input: &str) -> Option<PathEncodingType> {
  let lower = input.to_lowercase();
  let double_encoded_patterns = ["%252e", "%255c", "%250a", "%250d", "%252f"];
  let single_encoded_patterns = ["%2e", "%2f", "%5c", "%00", "%0a", "%0d"];

  double_encoded_patterns
    .iter()
    .find(|pattern| lower.contains(**pattern))
    .map(|_| PathEncodingType::DoubleEncoded)
    .or_else(|| {
      single_encoded_patterns
        .iter()
        .find(|pattern| lower.contains(**pattern))
        .map(|_| PathEncodingType::SingleEncoded)
    })
}

pub(super) fn detect_redos_patterns(pattern: &str) -> Option<RegexVulnerability> {
  let exp_patterns = ["(.*)*", "(.+)+", "(.?)?", "(.+)*", "(.*)+"];
  let nested_patterns = [")+)+", "*)*", "]+]+", ")+*", "*)+", "?)+", ")+?"];

  if exp_patterns.iter().any(|exp| pattern.contains(exp)) {
    Some(RegexVulnerability::ExponentialBacktracking)
  } else if pattern.contains(".*.*") || pattern.contains(".+.+") || pattern.contains(".?.?") {
    Some(RegexVulnerability::OverlappingWildcards)
  } else if nested_patterns
    .iter()
    .any(|nested| pattern.contains(nested))
  {
    Some(RegexVulnerability::NestedQuantifiers)
  } else {
    None
  }
}

pub(super) const fn is_valid_session_id_char(ch: char) -> bool {
  matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

#[cfg(test)]
mod tests {
  use super::{
    classify_metachar, contains_encoded_traversal, is_valid_session_id_char, MetacharCategory,
    PathEncodingType,
  };

  #[test]
  fn test_classify_metachar() {
    assert_eq!(
      classify_metachar(';'),
      Some(MetacharCategory::CommandSeparator)
    );
    assert_eq!(
      classify_metachar('$'),
      Some(MetacharCategory::VariableExpansion)
    );
    assert_eq!(classify_metachar('('), Some(MetacharCategory::Grouping));
    assert_eq!(classify_metachar('<'), Some(MetacharCategory::Redirection));
    assert_eq!(classify_metachar('*'), Some(MetacharCategory::EscapeQuote));
  }

  #[test]
  fn test_is_valid_session_id_char() {
    assert!(is_valid_session_id_char('a'));
    assert!(is_valid_session_id_char('Z'));
    assert!(is_valid_session_id_char('0'));
    assert!(is_valid_session_id_char('9'));
    assert!(is_valid_session_id_char('-'));
    assert!(is_valid_session_id_char('_'));
    assert!(!is_valid_session_id_char(' '));
    assert!(!is_valid_session_id_char('.'));
    assert!(!is_valid_session_id_char('@'));
    assert!(!is_valid_session_id_char('/'));
  }

  #[test]
  fn test_contains_encoded_traversal_single() {
    assert_eq!(
      contains_encoded_traversal("%2e%2e"),
      Some(PathEncodingType::SingleEncoded)
    );
    assert_eq!(
      contains_encoded_traversal("%2E%2E"),
      Some(PathEncodingType::SingleEncoded)
    );
    assert_eq!(
      contains_encoded_traversal("%5c"),
      Some(PathEncodingType::SingleEncoded)
    );
  }

  #[test]
  fn test_contains_encoded_traversal_double() {
    assert_eq!(
      contains_encoded_traversal("%252e"),
      Some(PathEncodingType::DoubleEncoded)
    );
    assert_eq!(
      contains_encoded_traversal("%255c"),
      Some(PathEncodingType::DoubleEncoded)
    );
  }

  #[test]
  fn test_contains_encoded_traversal_none() {
    assert_eq!(contains_encoded_traversal("normal/path"), None);
    assert_eq!(contains_encoded_traversal("file.txt"), None);
  }
}
