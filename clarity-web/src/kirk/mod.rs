#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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

pub mod types;

pub use types::{
    ContractVersion, EarsRequirement, EarsSection, Invariant, KirkContract, KirkContractError,
    Postcondition, Precondition, TypeRegistry, TypeSchema,
};
