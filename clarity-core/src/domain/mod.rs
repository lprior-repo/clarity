#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Core types with validation and business rules
//!
//! This module provides:
//! - Strongly typed IDs (BeadId) to prevent mixing up identifiers
//! - Validated types (BeadPriority) that enforce constraints at construction
//! - Business logic for status transitions and permissions
//! - Comprehensive tests for all validation rules

pub mod bead_operations;
pub mod core;
pub mod models;
pub mod repository;
pub mod types;

pub use bead_operations::{BeadError, BeadOperations, BeadState, Statistics};
pub use core::{
  add_bead, change_bead_status, close_bead, create_bead, filter_high_priority, filter_non_blocked,
  generate_bead_report, process_bead_pipeline, update_bead, update_bead_priority,
  validate_status_transition, with_beads, DomainError, DomainState, DomainStats,
};
pub use models::{Bead, ModelError, NewBead};
pub use repository::{BeadRepository, BeadSearchFilters, BeadSearchResult, BeadStatistics};
pub use types::{BeadId, BeadPriority, BeadStatus, BeadType, UserId};
