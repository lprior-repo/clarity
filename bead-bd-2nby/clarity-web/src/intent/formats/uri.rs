use super::{FormatError, UriError};

pub fn validate_uri(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(UriError::Empty.into());
  }

  let Some(separator_idx) = input.find("://") else {
    return Err(UriError::MissingSchemeSeparator.into());
  };

  let scheme = &input[..separator_idx];
  validate_uri_scheme(scheme)?;

  let authority_start = separator_idx + 3;
  if authority_start >= input.len() {
    return Err(UriError::MissingAuthority.into());
  }

  let authority_and_path = &input[authority_start..];
  let authority_end = authority_and_path
    .find('/')
    .map_or(authority_and_path.len(), |index| index);
  let authority = &authority_and_path[..authority_end];

  if authority.is_empty() {
    return Err(UriError::EmptyAuthority.into());
  }

  Ok(())
}

fn validate_uri_scheme(scheme: &str) -> Result<(), UriError> {
  if scheme.is_empty() {
    return Err(UriError::EmptyScheme);
  }

  let chars: Vec<char> = scheme.chars().collect();
  let first_char = chars.first().copied().ok_or(UriError::EmptyScheme)?;
  if !first_char.is_ascii_alphabetic() {
    return Err(UriError::SchemeMustStartWithLetter);
  }

  for ch in chars.iter().skip(1) {
    let is_valid = ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.');
    if !is_valid {
      return Err(UriError::InvalidSchemeChar(*ch));
    }
  }

  Ok(())
}
