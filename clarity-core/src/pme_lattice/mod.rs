//! PME Lattice - Mental Lattice Framework shared modules
//!
//! First Principles frameworks for product-market engineering:
//! - CDI Logger: Customer Data Insight collection and analysis
//! - Conflict Detection: Identify contradictions between requirements
//! - Design by Contract: Meyer's DbC with preconditions, postconditions, invariants
//! - Inversion: Avoid stupidity by thinking backward from failure
//! - North Star Scenario: Character + Simulation framework with plot hole detection
//! - Persona Forge: Realistic user personas to prevent Straw Man users
//! - Quality Dimensions: EQI Framework for engineering quality assessment
//! - Thesis Antithesis: Dialectical hypothesis testing for PME Discover
//! - Second-order thinking: Consider consequences of consequences
//! - Investment discipline: Resource allocation principles

pub mod cdi_logger;
pub mod conflict_detection;
pub mod design_by_contract;
pub mod inversion;
pub mod north_star;
pub mod persona_forge;
pub mod quality_dimensions;
pub mod thesis_antithesis;
pub mod thesis_generator;

#[cfg(test)]
mod cdi_logger_test;

#[cfg(test)]
mod conflict_detection_test;

#[cfg(test)]
mod design_by_contract_test;

#[cfg(test)]
mod inversion_test;

#[cfg(test)]
mod north_star_test;

#[cfg(test)]
mod quality_dimensions_test;

#[cfg(test)]
mod thesis_antithesis_test;

#[cfg(test)]
mod thesis_generator_test;

pub use cdi_logger::{
  calculate_aggregate_strength, AggregateStrengthError, CDIError, CDILogger, CustomerSignal,
  SignalSource, SignalStrength,
};
pub use conflict_detection::{
  Conflict, ConflictAnalysis, ConflictDetector, ConflictDetectorBuilder, ConflictError,
  ConflictType, Requirement, Severity,
};
pub use design_by_contract::{
  Contract, ContractMeta, ContractViolation, Invariant, InvariantMeta, InvariantSeverity,
  Postcondition, PostconditionMeta, Precondition, PreconditionMeta,
};
pub use inversion::{
  CognitiveBias, InversionAnalysis, InversionCategory, InversionError, InversionQuestion,
  StupidityCheck,
};
pub use north_star::{
  Character, CharacterBuilder, CharacterError, EdgeCaseType, NorthStarScenario,
  NorthStarScenarioBuilder, NorthStarScenarioError, PlotHole, PlotHoleType, Simulation,
  SimulationError, SimulationResult, TimelineEvent, TimelineEventError,
};
pub use persona_forge::{
  AuthorityLevel, Demographics, EducationLevel, HumanLimitations, Means, Persona, PersonaError,
  SkillLevel,
};
pub use quality_dimensions::{
  EQIAssessment, ImprovementAction, QualityDimension, QualityDimensionError, QualityMetric,
};
pub use thesis_antithesis::{
  Antithesis, AntithesisBuilder, HypothesisPair, HypothesisPairBuilder, SynthesisStatus, Thesis,
  ThesisAntithesisError, ThesisAntithesisGenerator, ThesisBuilder,
};
pub use thesis_generator::{
  ThesisAntithesisError as LegacyThesisAntithesisError,
  ThesisAntithesisGenerator as LegacyThesisAntithesisGenerator,
};
