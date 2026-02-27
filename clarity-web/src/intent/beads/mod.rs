//! Beads Module - Work Item Generation
//!
//! This module provides bead (work item) generation functionality:
//! - Profile-specific bead generators
//! - 16-section enhanced CUE template
//! - JSONL output
//! - Feedback collection
//!
//! ## Submodules
//!
//! - [`templates`] - Bead template generation from interview sessions (WP26)
//! - [`feedback`] - Feedback collection for work items (WP27)

pub mod feedback;
pub mod templates;

// Re-export key types for convenience
pub use feedback::{
    collect_feedback, collect_feedback_with_reviewer, get_bead_feedback_history, update_bead_status,
    BeadFeedback, BeadStatus, FeedbackError,
};
pub use feedback::BeadRecord as FeedbackBeadRecord;

// Re-export template types (WP26)
pub use templates::{
    beads_to_enhanced_cue, beads_to_jsonl, generate_beads_from_session, generate_profile_beads,
    BeadError, BeadTemplate, BeadTemplateStats,
};
