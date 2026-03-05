//! Interview storage - JSONL persistence for interview sessions.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

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
pub use models::{
  answer_to_version, AnswerChangeType, AnswerDiff, AnswerVersion, AnswerWithHistory, SessionDiff,
  SessionSnapshot, SessionWithHistories, SessionWithHistoriesError,
};

#[cfg(test)]
mod tests;
