//! Utilities Submodule
//!
//! Shared utility functions including:
//! - Case-insensitive string matching
//! - JSON array navigation
//! - Standard input handling

pub mod array_indexing;
pub mod case_insensitive;
pub mod stdin;

pub use array_indexing::{
  navigate_path, parse_path_component, split_path, ArrayIndexError, ArraySpec,
};
pub use case_insensitive::*;
pub use stdin::*;
