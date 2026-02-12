//! Mental Lattice Framework
//!
//! Foundational mental models for product thinking and decision-making.
//! These modules provide structured frameworks for avoiding cognitive biases
//! and making better product decisions.

pub mod characters;
pub mod conflict_detection;
pub mod design_by_contract;
pub mod gap_detection;
pub mod interview_5x5;
pub mod inversion;
pub mod invest;
pub mod quality_dimensions;
pub mod scenario_builder;
pub mod scenarios;
pub mod second_order;

// Export characters module items with prefix to avoid conflicts
pub use characters::{Character as CharactersCharacter, Motivation as CharactersMotivation};

// Export design_by_contract items
pub use design_by_contract::{
  ContractClause, ContractLayer, ContractReport, ContractSeverity, DesignByContractError,
};

// Export conflict_detection items
pub use conflict_detection::{
  Conflict, ConflictDetectionError, ConflictReport, ConflictSeverity, ConflictType, Constraint,
  DecisionFrame,
};

// Export gap_detection items
pub use gap_detection::{
  DesignSignal, Gap, GapDetectionError, GapKind, GapReport, GapSeverity, OwaspCategory,
  ProductAntiPattern,
};

// Export interview_5x5 items
pub use interview_5x5::{
  InterviewMatrix, InterviewMatrixError, InterviewPerspective, InterviewQuestion, SignalStrength,
  QUESTIONS_PER_PERSPECTIVE,
};

// Export inversion with MIN/MAX_SCORE prefixed
pub use inversion::{
  BiasDetection, BiasSeverity, CognitiveBias, InversionReview, MAX_SCORE as INVERSION_MAX_SCORE,
  MIN_SCORE as INVERSION_MIN_SCORE,
};

// Export invest with MIN/MAX_SCORE prefixed
pub use invest::{
  BehaviorSpec, CriterionScore, InvestCriterion, InvestReview, ScoreLevel,
  MAX_SCORE as INVEST_MAX_SCORE, MIN_SCORE as INVEST_MIN_SCORE,
};

// Export quality_dimensions with MIN/MAX_SCORE prefixed
pub use quality_dimensions::{
  DimensionScore, EQIAssessment, QualityDimension, QualityDimensionsError,
  MAX_SCORE as QUALITY_MAX_SCORE, MIN_SCORE as QUALITY_MIN_SCORE,
};

// Export scenario_builder with Character/Motivation prefixed
pub use scenario_builder::{
  Character as ScenarioBuilderCharacter, DetectedPlotHole, Motivation as ScenarioBuilderMotivation,
  NorthStarScenarioBuilder, ScenarioPlotHoleKind, ScenarioStep,
};

// Export second_order items
pub use second_order::*;
