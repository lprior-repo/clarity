use super::domain::FeedbackError;

pub(super) fn parse_bead_id(input: &str) -> Result<String, FeedbackError> {
  if input.trim().is_empty() {
    Err(FeedbackError::EmptyBeadId)
  } else {
    Ok(input.to_string())
  }
}

pub(super) fn parse_notes(input: &str) -> Result<String, FeedbackError> {
  if input.trim().is_empty() {
    Err(FeedbackError::EmptyFeedback)
  } else {
    Ok(input.to_string())
  }
}
