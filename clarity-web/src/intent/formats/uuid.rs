use super::helpers::is_valid_hex_char;
use super::{FormatError, UuidError};

pub fn validate_uuid(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(UuidError::Empty.into());
  }
  if input.len() != 36 {
    return Err(UuidError::WrongLength(input.len()).into());
  }

  let hyphen_positions = [8, 13, 18, 23];
  let chars: Vec<char> = input.chars().collect();

  for position in hyphen_positions {
    if chars.get(position) != Some(&'-') {
      return Err(UuidError::MissingHyphens.into());
    }
  }

  for (index, ch) in chars.iter().enumerate() {
    if hyphen_positions.contains(&index) {
      continue;
    }
    if !is_valid_hex_char(*ch) {
      return Err(UuidError::InvalidChar(index, *ch).into());
    }
  }

  let version_char = chars
    .get(14)
    .copied()
    .ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(version_char, '1' | '2' | '3' | '4' | '5') {
    return Err(UuidError::InvalidVersion(version_char).into());
  }

  let variant_char = chars
    .get(19)
    .copied()
    .ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(variant_char.to_ascii_lowercase(), '8' | '9' | 'a' | 'b') {
    return Err(UuidError::InvalidVariant(variant_char).into());
  }

  Ok(())
}
