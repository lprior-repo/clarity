//! Documents Submodule
//!
//! Document generation including:
//! - Vision document
//! - Ready document
//! - Acceptance test synthesis
//! - Spec construction

pub mod acceptance_synthesizer;
pub mod ready;
pub mod spec_builder;
pub mod vision;

pub use acceptance_synthesizer::*;
pub use ready::*;
pub use spec_builder::*;
pub use vision::*;
