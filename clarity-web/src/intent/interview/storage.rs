//! Interview storage - JSONL persistence for interview sessions.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod diff;
mod error;
mod history;
mod jsonl;
mod models;

pub use diff::{diff_sessions, diff_snapshots, format_diff};
pub use error::StorageError;
pub use history::{append_to_history, create_snapshot, list_session_history};
pub use jsonl::{
  append_session_to_jsonl, get_session_from_jsonl, list_sessions_from_jsonl, session_to_jsonl_line,
};
pub use models::{AnswerChangeType, AnswerDiff, SessionDiff, SessionSnapshot};

#[cfg(test)]
mod tests;
