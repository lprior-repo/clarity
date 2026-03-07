#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Bead Feedback facade for work-item review and lifecycle tracking.
//!
//! This module now acts as a thin public boundary over the split functional
//! implementation in `feedback/`, keeping one source of truth for validation,
//! state transitions, and store access.

#[path = "feedback/boundary.rs"]
mod boundary;
#[path = "feedback/domain.rs"]
mod domain;
#[path = "feedback/service.rs"]
mod service;
#[path = "feedback/store.rs"]
mod store;

pub use domain::{BeadFeedback, BeadRecord, BeadStatus, FeedbackError};
pub use service::{
  collect_feedback, collect_feedback_with_reviewer, get_bead_feedback_history, update_bead_status,
};

#[cfg(test)]
pub use store::clear_feedback_store;

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use chrono::DateTime;
  use serial_test::serial;

  #[test]
  fn test_bead_status_default() {
    assert_eq!(BeadStatus::default(), BeadStatus::Pending);
  }

  #[test]
  fn test_bead_status_is_terminal() {
    assert!(BeadStatus::Complete.is_terminal());
    assert!(!BeadStatus::Pending.is_terminal());
    assert!(!BeadStatus::InProgress.is_terminal());
    assert!(!BeadStatus::Failed.is_terminal());
  }

  #[test]
  fn test_bead_status_is_active() {
    assert!(BeadStatus::Ready.is_active());
    assert!(BeadStatus::InProgress.is_active());
    assert!(!BeadStatus::Pending.is_active());
    assert!(!BeadStatus::Complete.is_active());
    assert!(!BeadStatus::Failed.is_active());
    assert!(!BeadStatus::Blocked.is_active());
  }

  #[test]
  fn test_bead_status_valid_transitions_from_pending() {
    let status = BeadStatus::Pending;
    assert!(status.can_transition_to(&BeadStatus::Ready));
    assert!(status.can_transition_to(&BeadStatus::Blocked));
    assert!(status.can_transition_to(&BeadStatus::Pending));

    assert!(!status.can_transition_to(&BeadStatus::InProgress));
    assert!(!status.can_transition_to(&BeadStatus::Complete));
    assert!(!status.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn test_bead_status_valid_transitions_from_ready() {
    let status = BeadStatus::Ready;
    assert!(status.can_transition_to(&BeadStatus::InProgress));
    assert!(status.can_transition_to(&BeadStatus::Blocked));
    assert!(status.can_transition_to(&BeadStatus::Ready));

    assert!(!status.can_transition_to(&BeadStatus::Pending));
    assert!(!status.can_transition_to(&BeadStatus::Complete));
    assert!(!status.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn test_bead_status_valid_transitions_from_in_progress() {
    let status = BeadStatus::InProgress;
    assert!(status.can_transition_to(&BeadStatus::Complete));
    assert!(status.can_transition_to(&BeadStatus::Failed));
    assert!(status.can_transition_to(&BeadStatus::Blocked));
    assert!(status.can_transition_to(&BeadStatus::InProgress));

    assert!(!status.can_transition_to(&BeadStatus::Pending));
    assert!(!status.can_transition_to(&BeadStatus::Ready));
  }

  #[test]
  fn test_bead_status_valid_transitions_from_blocked() {
    let status = BeadStatus::Blocked;
    assert!(status.can_transition_to(&BeadStatus::Ready));
    assert!(status.can_transition_to(&BeadStatus::Pending));
    assert!(status.can_transition_to(&BeadStatus::InProgress));
    assert!(status.can_transition_to(&BeadStatus::Blocked));

    assert!(!status.can_transition_to(&BeadStatus::Complete));
    assert!(!status.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn test_bead_status_valid_transitions_from_failed() {
    let status = BeadStatus::Failed;
    assert!(status.can_transition_to(&BeadStatus::Ready));
    assert!(status.can_transition_to(&BeadStatus::Pending));
    assert!(status.can_transition_to(&BeadStatus::Failed));

    assert!(!status.can_transition_to(&BeadStatus::InProgress));
    assert!(!status.can_transition_to(&BeadStatus::Blocked));
    assert!(!status.can_transition_to(&BeadStatus::Complete));
  }

  #[test]
  fn test_bead_status_no_transitions_from_complete() {
    let status = BeadStatus::Complete;
    assert!(!status.can_transition_to(&BeadStatus::Pending));
    assert!(!status.can_transition_to(&BeadStatus::Ready));
    assert!(!status.can_transition_to(&BeadStatus::InProgress));
    assert!(!status.can_transition_to(&BeadStatus::Blocked));
    assert!(!status.can_transition_to(&BeadStatus::Failed));
    assert!(status.can_transition_to(&BeadStatus::Complete));
  }

  #[test]
  fn test_bead_feedback_new_valid() {
    let feedback = BeadFeedback::new(
      "bead-1".to_string(),
      BeadStatus::InProgress,
      "Starting work".to_string(),
      None,
      false,
    );
    assert!(feedback.is_ok());
    let fb = feedback.map_err(|_| ()).ok();
    if let Some(fb) = fb {
      assert_eq!(fb.bead_id, "bead-1");
      assert_eq!(fb.status, BeadStatus::InProgress);
      assert_eq!(fb.notes, "Starting work");
      assert!(fb.reviewer.is_none());
      assert!(!fb.approved);
    }
  }

  #[test]
  fn test_bead_feedback_new_with_reviewer() {
    let feedback = BeadFeedback::new(
      "bead-1".to_string(),
      BeadStatus::Complete,
      "Looks good".to_string(),
      Some("alice".to_string()),
      true,
    );
    assert!(feedback.is_ok());
    let fb = feedback.map_err(|_| ()).ok();
    if let Some(fb) = fb {
      assert_eq!(fb.reviewer, Some("alice".to_string()));
      assert!(fb.approved);
    }
  }

  #[test]
  fn test_bead_feedback_new_empty_bead_id() {
    let result = BeadFeedback::new(
      String::new(),
      BeadStatus::InProgress,
      "notes".to_string(),
      None,
      false,
    );
    assert!(matches!(result, Err(FeedbackError::EmptyBeadId)));
  }

  #[test]
  fn test_bead_feedback_new_whitespace_bead_id() {
    let result = BeadFeedback::new(
      "   ".to_string(),
      BeadStatus::InProgress,
      "notes".to_string(),
      None,
      false,
    );
    assert!(matches!(result, Err(FeedbackError::EmptyBeadId)));
  }

  #[test]
  fn test_bead_feedback_new_empty_notes() {
    let result = BeadFeedback::new(
      "bead-1".to_string(),
      BeadStatus::InProgress,
      String::new(),
      None,
      false,
    );
    assert!(matches!(result, Err(FeedbackError::EmptyFeedback)));
  }

  #[test]
  fn test_bead_feedback_new_whitespace_notes() {
    let result = BeadFeedback::new(
      "bead-1".to_string(),
      BeadStatus::InProgress,
      "   ".to_string(),
      None,
      false,
    );
    assert!(matches!(result, Err(FeedbackError::EmptyFeedback)));
  }

  #[test]
  fn test_bead_feedback_builder() {
    let feedback = BeadFeedback::new(
      "bead-1".to_string(),
      BeadStatus::Complete,
      "Done".to_string(),
      None,
      false,
    )
    .map_err(|_| ())
    .ok()
    .map(|fb| fb.with_reviewer("bob".to_string()).with_approved(true));

    if let Some(fb) = feedback {
      assert_eq!(fb.reviewer, Some("bob".to_string()));
      assert!(fb.approved);
    }
  }

  #[test]
  fn test_bead_record_new_valid() {
    let record = BeadRecord::new("bead-1".to_string(), "My Bead".to_string());
    assert!(record.is_ok());
    let record = record.map_err(|_| ()).ok();
    if let Some(r) = record {
      assert_eq!(r.id, "bead-1");
      assert_eq!(r.title, "My Bead");
      assert_eq!(r.status, BeadStatus::Pending);
      assert!(!r.approved);
    }
  }

  #[test]
  fn test_bead_record_new_empty_id() {
    let result = BeadRecord::new(String::new(), "Title".to_string());
    assert!(matches!(result, Err(FeedbackError::EmptyBeadId)));
  }

  #[test]
  fn test_bead_record_can_transition_to() {
    let record = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    if let Some(r) = record {
      assert!(r.can_transition_to(BeadStatus::Ready));
      assert!(!r.can_transition_to(BeadStatus::Complete));
    }
  }

  #[test]
  #[serial]
  fn test_collect_feedback_valid() {
    clear_feedback_store();
    let result = collect_feedback("test-bead-1", BeadStatus::Ready, "Ready to start");
    assert!(result.is_ok());

    let feedback = result.map_err(|_| ()).ok();
    if let Some(fb) = feedback {
      assert_eq!(fb.bead_id, "test-bead-1");
      assert_eq!(fb.status, BeadStatus::Ready);
      assert_eq!(fb.notes, "Ready to start");
      assert!(!fb.timestamp.is_empty());
    }
  }

  #[test]
  fn test_collect_feedback_empty_bead_id() {
    let result = collect_feedback("", BeadStatus::Ready, "notes");
    assert!(matches!(result, Err(FeedbackError::EmptyBeadId)));
  }

  #[test]
  fn test_collect_feedback_empty_notes() {
    let result = collect_feedback("bead-1", BeadStatus::Ready, "");
    assert!(matches!(result, Err(FeedbackError::EmptyFeedback)));
  }

  #[test]
  #[serial]
  fn test_collect_feedback_with_reviewer() {
    clear_feedback_store();
    let result = collect_feedback_with_reviewer(
      "test-bead-2",
      BeadStatus::Complete,
      "Approved",
      Some("alice".to_string()),
      true,
    );
    assert!(result.is_ok());

    let feedback = result.map_err(|_| ()).ok();
    if let Some(fb) = feedback {
      assert_eq!(fb.reviewer, Some("alice".to_string()));
      assert!(fb.approved);
    }
  }

  #[test]
  fn test_update_bead_status_valid_transition() {
    let mut bead = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::Ready,
      notes: "Ready".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
      reviewer: None,
      approved: false,
    };

    if let Some(b) = bead.as_mut() {
      let result = update_bead_status(b, &feedback);
      assert!(result.is_ok());
    }

    if let Some(b) = bead.as_ref() {
      assert_eq!(b.status, BeadStatus::Ready);
    }
  }

  #[test]
  fn test_update_bead_status_invalid_transition() {
    let mut bead = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::Complete,
      notes: "Done".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
      reviewer: None,
      approved: false,
    };

    if let Some(b) = bead.as_mut() {
      let result = update_bead_status(b, &feedback);
      assert!(matches!(
        result,
        Err(FeedbackError::InvalidTransition { .. })
      ));
    }
  }

  #[test]
  fn test_update_bead_status_already_complete() {
    let mut bead = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    if let Some(b) = bead.as_mut() {
      b.status = BeadStatus::Complete;
    }

    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::Ready,
      notes: "Reopen".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
      reviewer: None,
      approved: false,
    };

    if let Some(b) = bead.as_mut() {
      let result = update_bead_status(b, &feedback);
      assert!(matches!(result, Err(FeedbackError::AlreadyComplete)));
    }
  }

  #[test]
  fn test_update_bead_status_sets_approved() {
    let mut bead = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    if let Some(b) = bead.as_mut() {
      b.status = BeadStatus::Ready;
    }

    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::InProgress,
      notes: "Starting".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
      reviewer: None,
      approved: true,
    };

    if let Some(b) = bead.as_mut() {
      let result = update_bead_status(b, &feedback);
      assert!(result.is_ok());
    }

    if let Some(b) = bead.as_ref() {
      assert!(b.approved);
    }
  }

  #[test]
  fn test_update_bead_status_adds_to_history() {
    let mut bead = BeadRecord::new("bead-1".to_string(), "Test".to_string())
      .map_err(|_| ())
      .ok();
    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::Ready,
      notes: "Ready".to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
      reviewer: None,
      approved: false,
    };

    if let Some(b) = bead.as_mut() {
      let result = update_bead_status(b, &feedback);
      assert!(result.is_ok());
    }

    if let Some(b) = bead.as_ref() {
      let history = b.get_feedback();
      assert_eq!(history.len(), 1);
    }
  }

  #[test]
  #[serial]
  fn test_get_bead_feedback_history_empty() {
    clear_feedback_store();
    let history = get_bead_feedback_history("nonexistent-bead");
    assert!(matches!(history, Ok(items) if items.is_empty()));
  }

  #[test]
  #[serial]
  fn test_get_bead_feedback_history_multiple_entries() {
    clear_feedback_store();

    let _ = collect_feedback("history-test-bead", BeadStatus::Ready, "First feedback");
    let _ = collect_feedback(
      "history-test-bead",
      BeadStatus::InProgress,
      "Second feedback",
    );
    let _ = collect_feedback("history-test-bead", BeadStatus::Complete, "Third feedback");

    let history = get_bead_feedback_history("history-test-bead");
    assert!(matches!(
      history,
      Ok(items)
        if items.len() == 3
          && items[0].notes == "First feedback"
          && items[1].notes == "Second feedback"
          && items[2].notes == "Third feedback"
    ));
  }

  #[test]
  #[serial]
  fn test_get_bead_feedback_history_different_beads() {
    clear_feedback_store();

    let _ = collect_feedback("bead-a", BeadStatus::Ready, "A1");
    let _ = collect_feedback("bead-b", BeadStatus::Ready, "B1");
    let _ = collect_feedback("bead-a", BeadStatus::InProgress, "A2");

    let history_a = get_bead_feedback_history("bead-a");
    let history_b = get_bead_feedback_history("bead-b");

    assert!(matches!(history_a, Ok(items) if items.len() == 2));
    assert!(matches!(history_b, Ok(items) if items.len() == 1));
  }

  #[test]
  fn test_current_timestamp_format() {
    let feedback =
      collect_feedback_with_reviewer("bead-ts", BeadStatus::Ready, "Timestamp check", None, false)
        .map_err(|_| ())
        .ok();

    if let Some(feedback) = feedback {
      assert!(!feedback.timestamp.is_empty());
      let parsed = DateTime::parse_from_rfc3339(&feedback.timestamp);
      assert!(parsed.is_ok());
    }
  }

  #[test]
  fn test_bead_status_serde_roundtrip() {
    let statuses = [
      BeadStatus::Pending,
      BeadStatus::Ready,
      BeadStatus::InProgress,
      BeadStatus::Blocked,
      BeadStatus::Complete,
      BeadStatus::Failed,
    ];

    for status in statuses {
      let json = serde_json::to_string(&status).map_err(|_| ()).ok();
      if let Some(json) = json {
        let parsed: Option<BeadStatus> = serde_json::from_str(&json).map_err(|_| ()).ok();
        if let Some(p) = parsed {
          assert_eq!(status, p);
        }
      }
    }
  }

  #[test]
  fn test_bead_feedback_serde_roundtrip() {
    let feedback = BeadFeedback {
      bead_id: "bead-1".to_string(),
      status: BeadStatus::Complete,
      notes: "All done".to_string(),
      timestamp: "2024-01-15T10:30:00Z".to_string(),
      reviewer: Some("alice".to_string()),
      approved: true,
    };

    let json = serde_json::to_string(&feedback).map_err(|_| ()).ok();
    if let Some(json) = json {
      let parsed: Option<BeadFeedback> = serde_json::from_str(&json).map_err(|_| ()).ok();
      if let Some(p) = parsed {
        assert_eq!(feedback, p);
      }
    }
  }

  #[test]
  fn test_bead_record_serde_roundtrip() {
    let record = BeadRecord::new("bead-1".to_string(), "Test Bead".to_string())
      .map_err(|_| ())
      .ok();

    if let Some(r) = record {
      let json = serde_json::to_string(&r).map_err(|_| ()).ok();
      if let Some(json) = json {
        let parsed: Option<BeadRecord> = serde_json::from_str(&json).map_err(|_| ()).ok();
        if let Some(p) = parsed {
          assert_eq!(r.id, p.id);
          assert_eq!(r.title, p.title);
          assert_eq!(r.status, p.status);
          assert_eq!(r.approved, p.approved);
        }
      }
    }
  }

  #[test]
  fn test_feedback_error_display() {
    assert_eq!(
      FeedbackError::BeadNotFound("x".to_string()).to_string(),
      "bead not found: x"
    );

    let err = FeedbackError::InvalidTransition {
      from: BeadStatus::Pending,
      to: BeadStatus::Complete,
    };
    let msg = err.to_string();
    assert!(msg.contains("Pending"));
    assert!(msg.contains("Complete"));

    assert_eq!(
      FeedbackError::EmptyFeedback.to_string(),
      "empty feedback: notes cannot be empty"
    );
  }
}
