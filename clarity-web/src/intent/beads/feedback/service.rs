use super::boundary::{parse_bead_id, parse_notes};
use super::domain::{
  current_timestamp, transition_record, BeadFeedback, BeadRecord, BeadStatus, FeedbackError,
};
use super::store::{read_feedback_history, store_feedback};

pub fn collect_feedback(
  bead_id: &str,
  status: BeadStatus,
  notes: &str,
) -> Result<BeadFeedback, FeedbackError> {
  collect_feedback_with_reviewer(bead_id, status, notes, None, false)
}

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

pub fn update_bead_status(
  bead: &mut BeadRecord,
  feedback: &BeadFeedback,
) -> Result<(), FeedbackError> {
  transition_record(bead, feedback).map(|next| {
    *bead = next;
  })
}

#[must_use]
pub fn get_bead_feedback_history(bead_id: &str) -> Vec<BeadFeedback> {
  read_feedback_history(bead_id)
}
