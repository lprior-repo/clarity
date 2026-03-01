#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Lattice pattern implementations for requirements analysis.
//!
//! This module contains various pattern recognition and parsing modules
//! for requirements engineering patterns.

mod compact;
pub mod conflict_detection;
mod coverage;
pub mod design_by_contract;
pub mod ears;
pub mod effects;
pub mod gap_detection;
pub mod interview_5x5;
pub mod inversion;
mod premortem;
pub mod quality;
pub mod quality_dimensions;
mod quality_example;

pub use compact::{
  clean_text, compact_artifacts, CompactAnswer, CompactError, CompactOutput, CompactSummary, Phase,
};
pub use conflict_detection::{
  detect_conflicts, get_conflict_type, has_conflict, Conflict, ConflictAnalysis, ConflictError,
  ConflictResolution, ConflictSeverity, ConflictType, ResolutionStrategy,
};
pub use coverage::{
  analyze_coverage as coverage_analyze, Component, CoverageError, CoverageGap, CoverageOutput,
  CoveredComponent, Task, UseCase,
};
pub use design_by_contract::{
  analyze_contracts, extract_contracts, Contract, ContractAnalysis, ContractError, ContractType,
  ContractViolation, ViolationSeverity,
};
pub use ears::{parse_requirement, parse_requirements, EarsError, EarsOutput, EarsRequirement};
pub use effects::{
  detect_cycles, trace_effects, trace_effects_with_patterns, CausalPattern, DependencyEdge,
  DependencyNode, Effect, EffectsError, EffectsOutput,
};
pub use gap_detection::{
  check_category_coverage, detect_gaps, generate_requirements_template, get_missing_areas,
  DetectedGap, GapAnalysis, GapCategory, GapError, GapSeverity,
};
pub use interview_5x5::{
  analyze_coverage as interview_analyze_coverage, Answer, Interview5x5, InterviewError,
  Perspective, PerspectiveCoverage, Question, QuestionType,
};
pub use inversion::{
  apply_counterexample, apply_edge_case, apply_negation, apply_reversal, extract_assumptions,
  generate_challenges, invert, ChallengePattern, InversionChallenge, InversionError,
  InversionOutput, Severity,
};
pub use premortem::{
  generate_premortem, FailureCategory, FailureScenario, Likelihood, PremortemOutput,
};
pub use quality::{
  calculate_quality, DimensionScore, QualityDimension, QualityError, QualityIssue, QualityScore,
};
pub use quality_dimensions::{
  analyze_dimensions, CoreDimension, DimensionAnalysis, DimensionCategory, DimensionIssue,
  DimensionScore as QDimensionalScore, IssueSeverity, QualityDimensionError,
};

/// Mental Lattice invariants for quality assurance
///
/// These represent KIRK (Keep Invariants Regular and Known) patterns:
/// - Completeness: All required fields present
/// - Consistency: No contradictory requirements
/// - Testability: Acceptance criteria present
/// - Clarity: Minimal jargon and complexity
/// - Security: Auth/encryption/validation considered
pub type LatticeInvariants = quality::QualityScore;
