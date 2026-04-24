//! Answer Extraction Helpers
//!
//! This module provides internal helper functions used by the extraction logic.
//! These functions are not part of the public API.

/// Extracts the first number-like sequence from a string.
///
/// This function scans through the input character by character to find
/// and extract a numeric sequence. It handles:
/// - Optional sign prefix (+ or -) before any digits
/// - Decimal point after at least one digit
/// - Stops at the first non-numeric character after digits start
///
/// # Algorithm
///
/// 1. Scan characters, looking for optional sign or digits
/// 2. Once a digit is found, mark as "started"
/// 3. Allow one decimal point after start
/// 4. Stop at first non-digit character (after start)
///
/// # Returns
///
/// - The extracted number string if found
/// - The original string if no number sequence found (for error reporting)
pub fn extract_number_sequence(s: &str) -> String {
  let mut result = String::new();
  // Track state during scanning
  let mut seen_decimal = false; // Have we seen a decimal point?
  let mut seen_sign = false; // Have we seen a sign (+/-)?
  let mut started = false; // Have we started collecting digits?

  for ch in s.chars() {
    match ch {
      // Sign is only valid at the start, before any digits
      '-' | '+' if !started && !seen_sign => {
        result.push(ch);
        seen_sign = true;
      }
      // Decimal point is only valid after at least one digit
      '.' if started && !seen_decimal => {
        result.push(ch);
        seen_decimal = true;
      }
      // Digits are always valid after sign or on their own
      '0'..='9' => {
        result.push(ch);
        started = true;
      }
      // Any other character after we've started means end of number
      _ if started => {
        break;
      }
      // Ignore other characters before number starts (e.g., leading text)
      _ => {}
    }
  }

  // If we didn't extract anything, return original for error message
  if result.is_empty() {
    s.to_string()
  } else {
    result
  }
}

/// Extracts a URL pattern from text by finding HTTP/HTTPS protocol.
///
/// This function searches for `http://` or `https://` (case-insensitive)
/// and extracts the URL up to the first delimiter character.
///
/// # Algorithm
///
/// 1. Find the start of the URL by looking for http:// or https://
/// 2. Scan forward until hitting a delimiter (whitespace, comma, bracket)
/// 3. Strip trailing period if present (likely sentence punctuation)
///
/// # Returns
///
/// - `Some(url)` if a URL pattern is found
/// - `None` if no HTTP/HTTPS URL is found
pub fn extract_url_pattern(s: &str) -> Option<String> {
  // Case-insensitive search for protocol marker
  let lower = s.to_lowercase();
  let start = lower.find("http://").or_else(|| lower.find("https://"))?;

  // Extract from the original string (preserving case in URL)
  let rest = &s[start..];

  // Find the end of the URL - stop at common delimiters
  let end = rest
    .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ']' || c == '}')
    .map_or(rest.len(), |v| v);

  let url = &rest[..end];

  // Strip trailing period - it's likely sentence punctuation, not part of URL
  // (URLs can technically end with a period, but this is rare in practice)
  let url = url.strip_suffix('.').map_or(url, |s| s);

  Some(url.to_string())
}

/// Extracts an email pattern from text by finding the @ symbol.
///
/// This function locates an email address by finding the @ symbol and
/// expanding outward to capture the local and domain parts.
///
/// # Algorithm
///
/// 1. Find the @ symbol position
/// 2. Scan backwards from @ to find the start (stop at whitespace or delimiters)
/// 3. Scan forwards from @ to find the end (stop at whitespace or delimiters)
/// 4. Strip trailing period if present (likely sentence punctuation)
///
/// # Returns
///
/// - `Some(email)` if an @ symbol is found and text surrounds it
/// - `None` if no @ symbol is present
pub fn extract_email_pattern(s: &str) -> Option<&str> {
  // Find the @ symbol - required for any email
  let at_pos = s.find('@')?;

  // Scan backwards from @ to find the start of the local part
  // Stop at whitespace or common delimiters that precede emails
  let start = s[..at_pos]
    .rfind(|c: char| c.is_whitespace() || c == '<' || c == '(' || c == '[')
    .map_or(0, |pos| pos + 1);

  // Scan forwards from @ to find the end of the domain part
  // Stop at whitespace or common delimiters that follow emails
  let rest = &s[at_pos..];
  let end_offset = rest
    .find(|c: char| c.is_whitespace() || c == '>' || c == ')' || c == ']' || c == ',')
    .map_or(rest.len(), |v| v);

  let email = &s[start..at_pos + end_offset];

  // Strip trailing period if present (likely end of sentence, not part of email)
  let email = email.strip_suffix('.').map_or(email, |s| s);

  Some(email)
}
