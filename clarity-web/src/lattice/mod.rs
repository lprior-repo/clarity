#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Public API exports - used by library consumers
#![allow(unused_imports)]

//! Lattice pattern implementations for requirements analysis.
//!
//! This module contains various pattern recognition and parsing modules
//! for requirements engineering patterns.

mod compact;
mod coverage;
pub mod ears;
pub mod effects;
pub mod inversion;
mod premortem;
pub mod quality;
mod quality_example;

pub use compact::{
  clean_text, compact_artifacts, CompactAnswer, CompactError, CompactOutput, CompactSummary, Phase,
};
pub use coverage::{
  analyze_coverage, Component, CoverageError, CoverageGap, CoverageOutput, CoveredComponent, Task,
  UseCase,
};
pub use ears::*;
pub use effects::{
  detect_cycles, trace_effects, trace_effects_with_patterns, CausalPattern, DependencyEdge,
  DependencyNode, Effect, EffectsError, EffectsOutput,
};
pub use inversion::{
  apply_counterexample, apply_edge_case, apply_negation, apply_reversal, extract_assumptions,
  generate_challenges, invert, ChallengePattern, InversionChallenge, InversionError,
  InversionOutput, Severity,
};
pub use premortem::{
  generate_premortem, FailureCategory, FailureScenario, Likelihood, PremortemOutput,
};
pub use quality::*;

/// Mental Lattice invariants for quality assurance
///
/// These represent KIRK (Keep Invariants Regular and Known) patterns:
/// - Completeness: All required fields present
/// - Consistency: No contradictory requirements
/// - Testability: Acceptance criteria present
/// - Clarity: Minimal jargon and complexity
/// - Security: Auth/encryption/validation considered
pub type LatticeInvariants = quality::QualityScore;
