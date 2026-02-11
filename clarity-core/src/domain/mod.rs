#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Core types with validation and business rules
//!
//! This module provides:
//! - Strongly typed IDs (UserId, BeadId) to prevent mixing up identifiers
//! - Validated types (Email, BeadPriority) that enforce constraints at construction
//! - Business logic for status transitions and permissions
//! - Comprehensive tests for all validation rules

pub mod bead_operations;
pub mod core;
pub mod models;
pub mod repository;
pub mod types;
pub mod user_service;

pub use bead_operations::{BeadError, BeadOperations, BeadState, Statistics};
pub use core::{
  add_bead, change_bead_status, close_bead, create_bead, filter_high_priority, filter_non_blocked,
  generate_bead_report, process_bead_pipeline, update_bead, update_bead_priority,
  validate_status_transition, with_beads, DomainError, DomainState, DomainStats,
};
pub use models::{Bead, User};
pub use repository::{
  BeadRepository, BeadSearchFilters, BeadSearchResult, BeadStatistics, UserRepository,
};
pub use types::{BeadId, BeadPriority, BeadStatus, BeadType, Email, UserId, UserRole};
pub use user_service::{
  add_user, create_user, delete_user, filter_admin_users, filter_users_after_date,
  generate_user_report, get_all_users, get_user_by_email, get_user_by_id, get_user_count,
  get_users_by_role, process_user_pipeline, update_user, update_user_email, update_user_password,
  update_user_role, user_exists_by_email, user_to_summary, with_users, UserError, UserResult,
  UserSummary,
};
