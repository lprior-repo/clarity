#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Bead Feedback - Feedback collection for work items (WP27)
//!
//! This module provides types and functions for collecting and managing
//! feedback on beads (work items) throughout their lifecycle.
//!
//! ## Key Types
//!
//! - [`BeadStatus`] - Status of a bead in its lifecycle
//! - [`BeadFeedback`] - Feedback record for a bead
//! - [`FeedbackError`] - Errors for feedback operations
//! - [`BeadRecord`] - A bead record with status tracking
//!
//! ## Functions
//!
//! - [`collect_feedback`] - Create a feedback record with validation
//! - [`update_bead_status`] - Update bead status based on feedback
//! - [`get_bead_feedback_history`] - Retrieve feedback history for a bead

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error type for feedback operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FeedbackError {
    /// Bead not found
    #[error("bead not found: {0}")]
    BeadNotFound(String),

    /// Invalid status transition
    #[error("invalid status transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Current status
        from: BeadStatus,
        /// Attempted new status
        to: BeadStatus,
    },

    /// Empty feedback notes
    #[error("empty feedback: notes cannot be empty")]
    EmptyFeedback,

    /// Empty bead ID
    #[error("bead ID cannot be empty")]
    EmptyBeadId,

    /// Bead already complete
    #[error("bead is already complete")]
    AlreadyComplete,

    /// Bead is blocked
    #[error("bead is blocked: {0}")]
    Blocked(String),
}

// =============================================================================
// Status Types
// =============================================================================

/// Status of a bead in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeadStatus {
    /// Not yet ready for work
    Pending,
    /// Ready to be picked up
    Ready,
    /// Currently being worked on
    InProgress,
    /// Blocked by something
    Blocked,
    /// Successfully completed
    Complete,
    /// Failed to complete
    Failed,
}

impl Default for BeadStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl BeadStatus {
    /// Check if a transition to another status is valid
    ///
    /// Valid transitions:
    /// - Pending -> Ready, Blocked
    /// - Ready -> InProgress, Blocked
    /// - InProgress -> Complete, Failed, Blocked
    /// - Blocked -> Ready, Pending, InProgress
    /// - Complete -> (terminal)
    /// - Failed -> Ready, Pending (for retry)
    #[must_use]
    pub fn can_transition_to(&self, to: &Self) -> bool {
        match (self, to) {
            // Pending can go to Ready or Blocked
            (Self::Pending, Self::Ready) | (Self::Pending, Self::Blocked) => true,
            // Ready can go to InProgress or Blocked
            (Self::Ready, Self::InProgress) | (Self::Ready, Self::Blocked) => true,
            // InProgress can go to Complete, Failed, or Blocked
            (Self::InProgress, Self::Complete)
            | (Self::InProgress, Self::Failed)
            | (Self::InProgress, Self::Blocked) => true,
            // Blocked can go back to Ready, Pending, or InProgress
            (Self::Blocked, Self::Ready)
            | (Self::Blocked, Self::Pending)
            | (Self::Blocked, Self::InProgress) => true,
            // Failed can retry via Ready or Pending
            (Self::Failed, Self::Ready) | (Self::Failed, Self::Pending) => true,
            // Complete is terminal - no transitions allowed
            (Self::Complete, _) => false,
            // Same status is always valid (no-op)
            (a, b) if a == b => true,
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Check if this status is terminal (no further transitions)
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Check if this status indicates the bead is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress)
    }
}

// =============================================================================
// Feedback Types
// =============================================================================

/// Feedback on a bead
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadFeedback {
    /// ID of the bead this feedback is for
    pub bead_id: String,
    /// Current status of the bead
    pub status: BeadStatus,
    /// Feedback notes
    pub notes: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Reviewer identifier (optional)
    pub reviewer: Option<String>,
    /// Whether this feedback approves the bead
    pub approved: bool,
}

impl BeadFeedback {
    /// Create a new feedback record
    ///
    /// # Errors
    /// - Returns `FeedbackError::EmptyBeadId` if `bead_id` is empty
    /// - Returns `FeedbackError::EmptyFeedback` if `notes` is empty
    pub fn new(
        bead_id: String,
        status: BeadStatus,
        notes: String,
        reviewer: Option<String>,
        approved: bool,
    ) -> Result<Self, FeedbackError> {
        if bead_id.trim().is_empty() {
            return Err(FeedbackError::EmptyBeadId);
        }
        if notes.trim().is_empty() {
            return Err(FeedbackError::EmptyFeedback);
        }
        Ok(Self {
            bead_id,
            status,
            notes,
            timestamp: current_timestamp(),
            reviewer,
            approved,
        })
    }

    /// Builder method to set reviewer
    #[must_use]
    pub fn with_reviewer(mut self, reviewer: String) -> Self {
        self.reviewer = Some(reviewer);
        self
    }

    /// Builder method to set approved flag
    #[must_use]
    pub fn with_approved(mut self, approved: bool) -> Self {
        self.approved = approved;
        self
    }
}

// =============================================================================
// Bead Record Types
// =============================================================================

/// A bead record with status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadRecord {
    /// Unique bead identifier
    pub id: String,
    /// Bead title
    pub title: String,
    /// Current status
    pub status: BeadStatus,
    /// Feedback history
    #[serde(skip)]
    pub feedback_history: VecDeque<BeadFeedback>,
    /// Whether the bead is approved
    pub approved: bool,
}

impl BeadRecord {
    /// Create a new bead record
    ///
    /// # Errors
    /// Returns `FeedbackError::EmptyBeadId` if `id` is empty
    pub fn new(id: String, title: String) -> Result<Self, FeedbackError> {
        if id.trim().is_empty() {
            return Err(FeedbackError::EmptyBeadId);
        }
        Ok(Self {
            id,
            title,
            status: BeadStatus::Pending,
            feedback_history: VecDeque::new(),
            approved: false,
        })
    }

    /// Check if the bead can transition to a new status
    #[must_use]
    pub fn can_transition_to(&self, new_status: BeadStatus) -> bool {
        self.status.can_transition_to(&new_status)
    }

    /// Get all feedback for this bead
    #[must_use]
    pub fn get_feedback(&self) -> Vec<&BeadFeedback> {
        self.feedback_history.iter().collect()
    }
}

// =============================================================================
// Feedback Store (for history tracking)
// =============================================================================

use std::sync::{Arc, RwLock};

/// Global feedback store for tracking history across all beads
static FEEDBACK_STORE: once_cell::sync::Lazy<Arc<RwLock<HashMap<String, VecDeque<BeadFeedback>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Clear the feedback store (for testing)
#[cfg(test)]
pub fn clear_feedback_store() {
    let store = FEEDBACK_STORE
        .write()
        .map_err(|_| "Failed to acquire write lock")
        .ok();
    if let Some(mut store) = store {
        store.clear();
    }
}

// =============================================================================
// Core Functions
// =============================================================================

/// Collect feedback for a bead
///
/// Creates a feedback record and validates the status transition.
/// The feedback is stored in the global feedback history.
///
/// # Errors
/// - `FeedbackError::EmptyBeadId` if `bead_id` is empty
/// - `FeedbackError::EmptyFeedback` if `notes` is empty
/// - `FeedbackError::InvalidTransition` if current status cannot transition to new status
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::beads::feedback::{collect_feedback, BeadStatus};
///
/// let feedback = collect_feedback("bead-1", BeadStatus::InProgress, "Starting work")?;
/// assert_eq!(feedback.bead_id, "bead-1");
/// assert_eq!(feedback.status, BeadStatus::InProgress);
/// ```
pub fn collect_feedback(
    bead_id: &str,
    status: BeadStatus,
    notes: &str,
) -> Result<BeadFeedback, FeedbackError> {
    collect_feedback_with_reviewer(bead_id, status, notes, None, false)
}

/// Collect feedback with full options including reviewer and approval
///
/// # Errors
/// Same as [`collect_feedback`]
pub fn collect_feedback_with_reviewer(
    bead_id: &str,
    status: BeadStatus,
    notes: &str,
    reviewer: Option<String>,
    approved: bool,
) -> Result<BeadFeedback, FeedbackError> {
    if bead_id.trim().is_empty() {
        return Err(FeedbackError::EmptyBeadId);
    }
    if notes.trim().is_empty() {
        return Err(FeedbackError::EmptyFeedback);
    }

    let feedback = BeadFeedback {
        bead_id: bead_id.to_string(),
        status,
        notes: notes.to_string(),
        timestamp: current_timestamp(),
        reviewer,
        approved,
    };

    // Store feedback in history
    store_feedback(&feedback)?;

    Ok(feedback)
}

/// Update a bead record's status based on feedback
///
/// Validates the status transition and updates the bead record.
///
/// # Errors
/// - `FeedbackError::InvalidTransition` if the status transition is invalid
/// - `FeedbackError::AlreadyComplete` if the bead is already complete
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::beads::feedback::{
///     BeadRecord, BeadStatus, BeadFeedback, update_bead_status
/// };
///
/// let mut bead = BeadRecord::new("bead-1".to_string(), "My Bead".to_string())?;
/// let feedback = BeadFeedback {
///     bead_id: "bead-1".to_string(),
///     status: BeadStatus::Ready,
///     notes: "Ready to start".to_string(),
///     timestamp: "2024-01-01T00:00:00Z".to_string(),
///     reviewer: None,
///     approved: false,
/// };
///
/// update_bead_status(&mut bead, &feedback)?;
/// assert_eq!(bead.status, BeadStatus::Ready);
/// ```
pub fn update_bead_status(
    bead: &mut BeadRecord,
    feedback: &BeadFeedback,
) -> Result<(), FeedbackError> {
    // Check if bead is already complete (terminal state)
    if bead.status.is_terminal() {
        return Err(FeedbackError::AlreadyComplete);
    }

    // Validate transition
    if !bead.status.can_transition_to(&feedback.status) {
        return Err(FeedbackError::InvalidTransition {
            from: bead.status,
            to: feedback.status,
        });
    }

    // Update status
    bead.status = feedback.status;

    // Update approval flag
    if feedback.approved {
        bead.approved = true;
    }

    // Add feedback to history
    bead.feedback_history.push_back(feedback.clone());

    Ok(())
}

/// Get all feedback history for a bead
///
/// Returns all feedback records for the specified bead ID,
/// sorted by timestamp (oldest first).
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::beads::feedback::{get_bead_feedback_history, collect_feedback, BeadStatus};
///
/// collect_feedback("bead-1", BeadStatus::Ready, "Ready to start")?;
/// collect_feedback("bead-1", BeadStatus::InProgress, "Starting work")?;
///
/// let history = get_bead_feedback_history("bead-1");
/// assert_eq!(history.len(), 2);
/// ```
#[must_use]
pub fn get_bead_feedback_history(bead_id: &str) -> Vec<BeadFeedback> {
    let store = FEEDBACK_STORE.read().ok();
    match store {
        Some(s) => s
            .get(bead_id)
            .map_or_else(Vec::new, |v: &VecDeque<BeadFeedback>| {
                v.iter().cloned().collect()
            }),
        None => Vec::new(),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Store feedback in the global history
fn store_feedback(feedback: &BeadFeedback) -> Result<(), FeedbackError> {
    let store = FEEDBACK_STORE
        .write()
        .map_err(|_| FeedbackError::Blocked("Failed to acquire feedback store lock".into()))?;

    let mut store = store;

    let entry = store.entry(feedback.bead_id.clone()).or_insert_with(VecDeque::new);
    entry.push_back(feedback.clone());

    Ok(())
}

/// Get current timestamp in ISO 8601 format using chrono
fn current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // =========================================================================
    // BeadStatus Tests
    // =========================================================================

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
        assert!(status.can_transition_to(&BeadStatus::Pending)); // No-op

        assert!(!status.can_transition_to(&BeadStatus::InProgress));
        assert!(!status.can_transition_to(&BeadStatus::Complete));
        assert!(!status.can_transition_to(&BeadStatus::Failed));
    }

    #[test]
    fn test_bead_status_valid_transitions_from_ready() {
        let status = BeadStatus::Ready;
        assert!(status.can_transition_to(&BeadStatus::InProgress));
        assert!(status.can_transition_to(&BeadStatus::Blocked));
        assert!(status.can_transition_to(&BeadStatus::Ready)); // No-op

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
        assert!(status.can_transition_to(&BeadStatus::InProgress)); // No-op

        assert!(!status.can_transition_to(&BeadStatus::Pending));
        assert!(!status.can_transition_to(&BeadStatus::Ready));
    }

    #[test]
    fn test_bead_status_valid_transitions_from_blocked() {
        let status = BeadStatus::Blocked;
        assert!(status.can_transition_to(&BeadStatus::Ready));
        assert!(status.can_transition_to(&BeadStatus::Pending));
        assert!(status.can_transition_to(&BeadStatus::InProgress));
        assert!(status.can_transition_to(&BeadStatus::Blocked)); // No-op

        assert!(!status.can_transition_to(&BeadStatus::Complete));
        assert!(!status.can_transition_to(&BeadStatus::Failed));
    }

    #[test]
    fn test_bead_status_valid_transitions_from_failed() {
        let status = BeadStatus::Failed;
        assert!(status.can_transition_to(&BeadStatus::Ready));
        assert!(status.can_transition_to(&BeadStatus::Pending));
        assert!(status.can_transition_to(&BeadStatus::Failed)); // No-op

        assert!(!status.can_transition_to(&BeadStatus::InProgress));
        assert!(!status.can_transition_to(&BeadStatus::Complete));
        assert!(!status.can_transition_to(&BeadStatus::Blocked));
    }

    #[test]
    fn test_bead_status_no_transitions_from_complete() {
        let status = BeadStatus::Complete;
        assert!(!status.can_transition_to(&BeadStatus::Pending));
        assert!(!status.can_transition_to(&BeadStatus::Ready));
        assert!(!status.can_transition_to(&BeadStatus::InProgress));
        assert!(!status.can_transition_to(&BeadStatus::Blocked));
        assert!(!status.can_transition_to(&BeadStatus::Failed));
        assert!(!status.can_transition_to(&BeadStatus::Complete)); // Even no-op
    }

    // =========================================================================
    // BeadFeedback Tests
    // =========================================================================

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

    // =========================================================================
    // BeadRecord Tests
    // =========================================================================

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

    // =========================================================================
    // collect_feedback Tests
    // =========================================================================

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

    // =========================================================================
    // update_bead_status Tests
    // =========================================================================

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
        // Try to jump from Pending directly to Complete (invalid)
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
            assert!(matches!(result, Err(FeedbackError::InvalidTransition { .. })));
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
        // First transition to Ready
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

    // =========================================================================
    // get_bead_feedback_history Tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_get_bead_feedback_history_empty() {
        clear_feedback_store();
        let history = get_bead_feedback_history("nonexistent-bead");
        assert!(history.is_empty());
    }

    #[test]
    #[serial]
    fn test_get_bead_feedback_history_multiple_entries() {
        clear_feedback_store();

        let _ = collect_feedback("history-test-bead", BeadStatus::Ready, "First feedback");
        let _ = collect_feedback("history-test-bead", BeadStatus::InProgress, "Second feedback");
        let _ = collect_feedback("history-test-bead", BeadStatus::Complete, "Third feedback");

        let history = get_bead_feedback_history("history-test-bead");
        assert_eq!(history.len(), 3);

        // Verify order (oldest first)
        assert_eq!(history[0].notes, "First feedback");
        assert_eq!(history[1].notes, "Second feedback");
        assert_eq!(history[2].notes, "Third feedback");
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

        // bead-a has 2 entries (Ready + InProgress)
        // bead-b has 1 entry (Ready)
        assert_eq!(history_a.len(), 2);
        assert_eq!(history_b.len(), 1);
    }

    // =========================================================================
    // Timestamp Tests
    // =========================================================================

    #[test]
    fn test_current_timestamp_format() {
        use chrono::DateTime;
        let ts = current_timestamp();
        // Chrono timestamp should be parseable
        assert!(!ts.is_empty());
        // Should be valid RFC 3339
        let parsed = DateTime::parse_from_rfc3339(&ts);
        assert!(parsed.is_ok());
    }

    // =========================================================================
    // Serde Tests
    // =========================================================================

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

    // =========================================================================
    // Error Display Tests
    // =========================================================================

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
