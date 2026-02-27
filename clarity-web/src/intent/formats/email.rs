use super::{EmailError, FormatError};

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
  if local.len() > 64 {
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
  if domain.len() > 255 {
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
  if label.len() > 63 {
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
