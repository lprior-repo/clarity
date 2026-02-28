use super::{EmailError, FormatError};

// ============================================================================
// EMAIL FORMAT CONSTANTS (RFC 5321 / RFC 5322)
// ============================================================================

/// Maximum length of the local part (before @) in an email address.
///
/// RFC 5321 specifies that the maximum total length of a reverse-path
/// (including the local-part) is 256 characters, but the local-part
/// itself has a maximum of 64 characters.
const MAX_LOCAL_PART_LENGTH: usize = 64;

/// Maximum length of the domain part (after @) in an email address.
///
/// RFC 5321 specifies a maximum domain length of 255 characters.
const MAX_DOMAIN_LENGTH: usize = 255;

/// Maximum length of a single domain label (segment between dots).
///
/// RFC 1035 specifies that each label in a domain name can be at most
/// 63 octets (characters).
const MAX_DOMAIN_LABEL_LENGTH: usize = 63;

/// Special characters allowed in the local part of an email address.
///
/// RFC 5322 allows these characters in the local part (with some restrictions
/// on dots and quoted strings).
const ALLOWED_LOCAL_SPECIAL: &[char] = &[
  '!', '#', '$', '%', '&', '\'', '*', '+', '-', '/', '=', '?', '^', '_', '`', '{', '|', '}', '~',
  '.', '"',
];

pub fn validate_email(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(EmailError::Empty.into());
  }

  match input.chars().filter(|&c| c == '@').count() {
    0 => return Err(EmailError::MissingAtSign.into()),
    1 => {}
    _ => return Err(EmailError::MultipleAtSigns.into()),
  }

  let (local, domain) = input.split_once('@').ok_or(EmailError::MissingAtSign)?;
  validate_email_local(local)?;
  validate_email_domain(domain)?;
  Ok(())
}

fn validate_email_local(local: &str) -> Result<(), EmailError> {
  if local.is_empty() {
    return Err(EmailError::EmptyLocalPart);
  }
  if local.len() > MAX_LOCAL_PART_LENGTH {
    return Err(EmailError::LocalPartTooLong);
  }
  if local.starts_with('.') {
    return Err(EmailError::LocalStartsWithDot);
  }
  if local.ends_with('.') {
    return Err(EmailError::LocalEndsWithDot);
  }
  if local.contains("..") {
    return Err(EmailError::ConsecutiveDots);
  }

  for ch in local.chars() {
    let is_valid = ch.is_ascii_alphanumeric() || ALLOWED_LOCAL_SPECIAL.contains(&ch);
    if !is_valid {
      return Err(EmailError::InvalidLocalChar(ch));
    }
  }

  Ok(())
}

fn validate_email_domain(domain: &str) -> Result<(), EmailError> {
  if domain.is_empty() {
    return Err(EmailError::EmptyDomain);
  }
  if domain.len() > MAX_DOMAIN_LENGTH {
    return Err(EmailError::DomainTooLong);
  }
  if domain.starts_with('.') {
    return Err(EmailError::DomainStartsWithDot);
  }
  if domain.ends_with('.') {
    return Err(EmailError::DomainEndsWithDot);
  }
  if !domain.contains('.') {
    return Err(EmailError::MissingTld);
  }
  if domain.contains("..") {
    return Err(EmailError::ConsecutiveDots);
  }

  for label in domain.split('.') {
    validate_domain_label(label)?;
  }

  Ok(())
}

fn validate_domain_label(label: &str) -> Result<(), EmailError> {
  if label.is_empty() {
    return Err(EmailError::ConsecutiveDots);
  }
  if label.len() > MAX_DOMAIN_LABEL_LENGTH {
    return Err(EmailError::DomainLabelTooLong);
  }
  if label.starts_with('-') {
    return Err(EmailError::DomainLabelStartsWithHyphen);
  }
  if label.ends_with('-') {
    return Err(EmailError::DomainLabelEndsWithHyphen);
  }

  for ch in label.chars() {
    if !ch.is_ascii_alphanumeric() && ch != '-' {
      return Err(EmailError::InvalidDomainChar(ch));
    }
  }

  Ok(())
}
