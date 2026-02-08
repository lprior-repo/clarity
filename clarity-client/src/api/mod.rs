//! API client module for Clarity
//!
//! This module provides HTTP client functionality for communicating
//! with the Clarity backend server.

pub mod client;
pub mod types;

pub use client::*;
pub use types::*;
