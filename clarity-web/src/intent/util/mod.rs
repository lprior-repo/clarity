//! Utilities Submodule
//!
//! Shared utility functions including:
//! - Case-insensitive string matching
//! - JSON array navigation
//! - Standard input handling

pub mod array_indexing;
pub mod case_insensitive;

pub use array_indexing::{ArrayIndexError, ArraySpec, navigate_path, parse_path_component, split_path};
pub use case_insensitive::*;
