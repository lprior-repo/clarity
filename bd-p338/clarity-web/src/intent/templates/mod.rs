//! Templates Submodule
//!
//! Spec template generation including:
//! - Profile-specific templates
//! - Boilerplate generation
//! - Template filling from session data

pub mod spec_templates;

// Re-export key types for convenience
pub use spec_templates::{fill_template, generate_spec_template, SpecTemplateError};
