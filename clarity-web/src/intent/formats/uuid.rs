use super::helpers::is_valid_hex_char;
use super::{FormatError, UuidError};

// ============================================================================
// UUID FORMAT CONSTANTS
// ============================================================================

/// Expected total length of a UUID string including hyphens.
///
/// Standard UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// 8 + 1 + 4 + 1 + 4 + 1 + 4 + 1 + 12 = 36 characters
const UUID_LENGTH: usize = 36;

/// Character positions where hyphens must appear in a valid UUID.
///
/// Positions are zero-indexed:
/// - Position 8: After the first 8 hex digits (`time_low`)
/// - Position 13: After the next 4 hex digits (`time_mid`)
/// - Position 18: After the next 4 hex digits (`time_hi_and_version`)
/// - Position 23: After the next 4 hex digits (`clock_seq_hi_and_reserved` + `clock_seq_low`)
const HYPHEN_POSITIONS: [usize; 4] = [8, 13, 18, 23];

/// Position of the version character in a UUID (within the third group).
///
/// The version digit appears at index 14, immediately after the second hyphen.
/// Valid versions: 1 (time-based), 2 (DCE security), 3 (MD5 hash), 4 (random), 5 (SHA-1 hash)
const VERSION_CHAR_POSITION: usize = 14;

/// Position of the variant character in a UUID (within the fourth group).
///
/// The variant digit appears at index 19, immediately after the third hyphen.
/// Valid variant indicators: 8, 9, a, b (RFC 4122 variant)
const VARIANT_CHAR_POSITION: usize = 19;

/// Validates a canonical hyphenated UUID string.
///
/// # Errors
/// Returns `FormatError::Uuid` when length, hyphen positions, hex digits, version, or variant are invalid.
pub fn validate_uuid(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(UuidError::Empty.into());
  }
  if input.len() != UUID_LENGTH {
    return Err(UuidError::WrongLength(input.len()).into());
  }

  let chars: Vec<char> = input.chars().collect();

  for position in HYPHEN_POSITIONS {
    if chars.get(position) != Some(&'-') {
      return Err(UuidError::MissingHyphens.into());
    }
  }

  for (index, ch) in chars.iter().enumerate() {
    if HYPHEN_POSITIONS.contains(&index) {
      continue;
    }
    if !is_valid_hex_char(*ch) {
      return Err(UuidError::InvalidChar(index, *ch).into());
    }
  }

  let version_char = chars
    .get(VERSION_CHAR_POSITION)
    .copied()
    .ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(version_char, '1' | '2' | '3' | '4' | '5') {
    return Err(UuidError::InvalidVersion(version_char).into());
  }

  let variant_char = chars
    .get(VARIANT_CHAR_POSITION)
    .copied()
    .ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(variant_char.to_ascii_lowercase(), '8' | '9' | 'a' | 'b') {
    return Err(UuidError::InvalidVariant(variant_char).into());
  }

  Ok(())
}
