#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Product Management Engine (PME)
//!
//! This module implements the Product Management Engine, providing tools for
//! the Double Diamond product discovery and definition process.
//!
//! # Modules
//!
//! - **discover**: Double Diamond Phase 1 - Discover phase components
//!   - Thesis & Antithesis Generator: Product thesis with null hypothesis
//!   - Persona Forge: Realistic user personas with human limitations
//!   - North Star Scenario Builder: Character + Simulation framework
//!   - CDI Logger: Customer Discovery Interview tracking with signal strength
//! - **define**: Double Diamond Phase 2 - Define phase components
//! - **develop**: Double Diamond Phase 3 - Develop phase components
//!   - Error Taxonomy Engine: 5-category error classification
//!   - NFR Wizard: Non-Functional Requirements trade-off wizard

pub mod discover;
pub mod define;
pub mod error_taxonomy;
pub mod nfr_wizard;

// Re-export discover phase types
pub use discover::thesis_generator::{
    Antithesis, Thesis, ThesisAntithesisGenerator, ThesisError, ThesisOutput, ValidationStatus,
};
pub use discover::persona_forge::{
    HumanLimitation, Persona, PersonaError, PersonaForge, PersonaOutput,
    ValidationResult,
};
pub use discover::north_star::{
    Character, DiscoveryMechanism, EdgeCase, NorthStarBuilder, NorthStarError, NorthStarOutput,
    PlotHole, Scenario, SimulationResult, TimelineEvent,
};
pub use discover::cdi_logger::{
    CdiEntry, CdiFunnel, CdiLogger, CdiSignal, InterviewOutcome, SignalStrength, SignalType,
};

pub use define::brutal_truths::{
    BrutalTruth, BrutalTruthsOutput, BrutalTruthsPrioritizer, PrioritizedItem, PrioritizerError,
    VorpCalculator, VorpScore,
};
pub use define::great_reindexing::{
    GraphRequirement, GreatReindexingEngine, JobToBeDone, RequirementEdge, RequirementGraph,
    RequirementNode, ReindexingError, ReindexingOutput, StoryInput, UserStory,
};

// Re-export develop phase types
pub use error_taxonomy::{
    ClassifiedError, ErrorCategory, ErrorClassifier, ErrorContext, ErrorSummary,
    ErrorTaxonomyEngine, LogLevel, MessageStyle, Remediation, Responsibility, RoutingResult,
    RoutingStrategy, TaxonomyError,
};

pub use nfr_wizard::{
    ArchitectureDecision, ComparisonOperator, DecisionStatus, GateResult, NfrCategory, NfrProfile,
    NfrSummary, NfrWizard, NfrWizardError, PersonaType, Priority, QualityGate, TradeOffChoice,
    WizardState, create_default_gates,
};
