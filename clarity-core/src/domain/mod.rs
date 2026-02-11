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

pub mod models;
pub mod types;

pub use models::{Bead, User};
pub use types::{BeadId, BeadPriority, BeadStatus, BeadType, Email, UserId, UserRole};
