//! Answer Parsers
//!
//! This module provides parsing functions for structured formats within
//! free-text responses, particularly list parsing.

/// Parses a numbered list from text.
///
/// This function extracts items from numbered list formats like:
/// - `1. item` (period separator)
/// - `1) item` (parenthesis separator)
/// - `1- item` (dash separator)
/// - `1: item` (colon separator)
///
/// # Algorithm
///
/// 1. Process each line independently
/// 2. Check if line starts with a digit
/// 3. Skip all consecutive digits (handles numbers >= 10)
/// 4. Skip the separator character
/// 5. Collect the remaining text as the list item
///
/// # Returns
///
/// A vector of list item strings. Empty if no numbered list pattern is found.
pub fn parse_numbered_list(s: &str) -> Vec<String> {
  let mut items = Vec::new();

  for line in s.lines() {
    let trimmed = line.trim();

    // Check if line starts with a digit (first digit of list number)
    if let Some(after_number) = trimmed
      .strip_prefix(|c: char| c.is_ascii_digit())
      .map(str::trim_start)
    {
      // Skip remaining digits for multi-digit numbers (10, 11, etc.)
      let after_number = after_number
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();

      // Strip the separator character (. or ) or - or :)
      // If no separator, use whatever remains (handles edge cases)
      let content = after_number
        .strip_prefix(['.', ')', '-', ':'])
        .map_or_else(|| after_number.trim(), str::trim);

      if !content.is_empty() {
        items.push(content.to_string());
      }
    }
  }

  items
}
