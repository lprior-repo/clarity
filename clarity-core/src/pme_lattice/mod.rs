//! PME Lattice - Mental Lattice Framework shared modules
//!
//! First Principles frameworks for product-market engineering:
//! - Inversion: Avoid stupidity by thinking backward from failure
//! - Second-order thinking: Consider consequences of consequences
//! - Investment discipline: Resource allocation principles
//! - Design by contract: Pre/post conditions and invariants

pub mod inversion;

pub use inversion::{
  CognitiveBias, InversionAnalysis, InversionCategory, InversionError, InversionQuestion,
  StupidityCheck,
};
