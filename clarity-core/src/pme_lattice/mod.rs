//! PME Lattice - Mental Lattice Framework shared modules
//!
//! First Principles frameworks for product-market engineering:
//! - Inversion: Avoid stupidity by thinking backward from failure
//! - North Star Scenario: Character + Simulation framework with plot hole detection
//! - Persona Forge: Realistic user personas to prevent Straw Man users
//! - Second-order thinking: Consider consequences of consequences
//! - Investment discipline: Resource allocation principles
//! - Design by contract: Pre/post conditions and invariants

pub mod inversion;
pub mod north_star;
pub mod persona_forge;
pub mod thesis_generator;

#[cfg(test)]
mod inversion_test;

#[cfg(test)]
mod north_star_test;

#[cfg(test)]
mod thesis_generator_test;

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
pub use thesis_generator::{ThesisAntithesisError, ThesisAntithesisGenerator};
