//! br Show Integration Module
//!
//! This module provides functionality to integrate with the `br` command line tool
//! for displaying issue details directly in the Clarity desktop application.
//!
//! # Implementation Notes
//!
//! - Uses `tokio::process::Command` for async command execution
//! - Returns structured data using thiserror for domain errors
//! - Follows functional core, imperative shell pattern
//! - Zero unwrap principle enforced

mod bead_bd_2zk;
mod core;
mod models;
mod show_bead;

pub use bead_bd_2zk::{bd_2zk_exists, get_bd_2zk_bead, Bd2zkShowPage};
pub use core::{fetch_br_issue, get_issue_ids, issue_exists};
pub use models::{BrIssue, BrShowError};
pub use show_bead::{BrShowPage, BrShowProps};
