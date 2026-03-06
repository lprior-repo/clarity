#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Public API exports - used by library consumers
#![allow(unused_imports)]

//! KIRK (Keep Invariants Regular and Known) contract types.
//!
//! This module provides type definitions for design-by-contract specifications,
//! enabling formal preconditions, postconditions, and invariants to be attached
//! to system components.
//!
//! # Overview
//!
//! KIRK contracts provide:
//! - **Preconditions**: Requirements that must hold before an operation
//! - **Postconditions**: Guarantees that hold after an operation completes
//! - **Invariants**: Properties that must always hold true
//!
//! # EARS Integration
//!
//! Contract requirements are expressed using EARS (Easy Approach to Requirements Syntax):
//! - **Ubiquitous**: Always applicable requirements
//! - **Event-driven**: Triggered by specific events
//! - **Unwanted**: Behaviors that must NOT occur
//!
//! # Progressive Discover Integration
//!
//! The `progressive_discover` module provides types for the Progressive Discover phase,
//! including the 16-section KIRK contract structure and validation types.

pub mod progressive_discover;
#[cfg(not(target_arch = "wasm32"))]
pub mod terminal_integration;
pub mod types;

pub use progressive_discover::{
  AntithesisValidation, EarsExtraction, EarsPattern, ExtractedEarsRequirement,
  HolePunchingValidation, KirkContract16, KirkSection, VorpValidation,
};
#[cfg(not(target_arch = "wasm32"))]
pub use terminal_integration::{
  ConnectionState, ConnectionStatus, OpenCodeTerminalClient, ProcessedTranscript, TerminalClient,
  TerminalConfig, TerminalError, TranscriptProcessor,
};
pub use types::{
  ContractVersion, EarsRequirement, EarsSection, Invariant, KirkContract, KirkContractError,
  Postcondition, Precondition, TypeRegistry, TypeSchema,
};
