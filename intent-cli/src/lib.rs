//! Intent CLI library
//!
//! This module provides the core functionality for the Intent CLI tool.

pub mod intent;

// Re-export key modules for convenience
pub use intent::{init_prompt, ui};
