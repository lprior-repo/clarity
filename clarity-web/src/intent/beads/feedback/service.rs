use super::boundary::{parse_bead_id, parse_notes};
use super::domain::{
  current_timestamp, transition_record, BeadFeedback, BeadRecord, BeadStatus, FeedbackError,
};
use super::store::{read_feedback_history, store_feedback};

/// Collects feedback for a bead.
///
/// # Errors
///
/// Returns `FeedbackError` if the `bead_id` or `notes` validation fails.
pub fn collect_feedback(
  bead_id: &str,
  status: BeadStatus,
  notes: &str,
) -> Result<BeadFeedback, FeedbackError> {
  collect_feedback_with_reviewer(bead_id, status, notes, None, false)
}

/// Collects feedback for a bead with reviewer information.
///
/// # Errors
///
/// Returns `FeedbackError` if the `bead_id` or `notes` validation fails.
pub fn collect_feedback_with_reviewer(
  bead_id: &str,
  status: BeadStatus,
  notes: &str,
  reviewer: Option<String>,
  approved: bool,
) -> Result<BeadFeedback, FeedbackError> {
  let parsed_bead_id = parse_bead_id(bead_id)?;
  let parsed_notes = parse_notes(notes)?;
  let feedback = BeadFeedback {
    bead_id: parsed_bead_id,
    status,
    notes: parsed_notes,
    timestamp: current_timestamp(),
    reviewer,
    approved,
  };
  store_feedback(&feedback)?;
  Ok(feedback)
}

/// Updates the status of a bead based on feedback.
///
/// # Errors
///
/// Returns `FeedbackError` if the status transition is invalid.
pub fn update_bead_status(
  bead: &mut BeadRecord,
  feedback: &BeadFeedback,
) -> Result<(), FeedbackError> {
  transition_record(bead, feedback).map(|next| {
    *bead = next;
  })
}

/// Returns the feedback history for a bead in insertion order.
///
/// # Errors
///
/// Returns `FeedbackError::Blocked` when the shared feedback store cannot be read.
pub fn get_bead_feedback_history(bead_id: &str) -> Result<Vec<BeadFeedback>, FeedbackError> {
  read_feedback_history(bead_id)
}
