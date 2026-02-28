//! Interview Submodule
//!
//! Core interview session management including:
//! - Session creation and lifecycle
//! - Gap detection (profile-specific required fields)
//! - Conflict detection (CAP theorem, anonymous+audit, perspective conflicts)
//! - JSONL storage and session diffing
//! - Question loading and management
//! - Contract validation
//! - Answer extraction (field extraction from free-text responses)

pub mod answer_extraction;
pub mod interview_contract;
pub mod interview_questions;
pub mod question_loader;
pub mod question_types;
pub mod storage;
pub mod types;

// Re-export types
pub use answer_extraction::*;
pub use interview_contract::*;
pub use interview_questions::*;
pub use question_loader::*;
pub use question_types::*;
pub use storage::*;
pub use types::*;
