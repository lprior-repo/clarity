#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod boundary;
mod domain;
mod service;
mod store;

pub use domain::{BeadFeedback, BeadRecord, BeadStatus, FeedbackError};
pub use service::{
    collect_feedback,
    collect_feedback_with_reviewer,
    get_bead_feedback_history,
    update_bead_status,
};
