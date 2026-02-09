//! Bead Components Module
//!
//! This module contains all components for managing beads in the UI.
//! All components use server functions instead of manual HTTP calls.

pub mod detail;
pub mod form;
pub mod list;

pub use detail::BeadDetailPage;
pub use form::BeadFormPage;
pub use list::BeadListPage;
