#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Product Management Engine - Discover Phase (Double Diamond Phase 1)
//!
//! This module implements the Discover phase components of the Product Management Engine,
//! providing tools for early-stage product discovery and validation.
//!
//! # Components
//!
//! - **Thesis Generator**: Generate product theses with required antitheses (null hypotheses)
//! - **Persona Forge**: Create realistic user personas with human limitations
//! - **North Star Scenario Builder**: Build and validate user journey scenarios
//! - **CDI Logger**: Track Customer Discovery Interviews with signal strength

pub mod thesis_generator;
pub mod persona_forge;
pub mod north_star;
pub mod cdi_logger;

pub use thesis_generator::{
    Antithesis, Thesis, ThesisAntithesisGenerator, ThesisError, ThesisOutput, ValidationStatus,
};
pub use persona_forge::{
    HumanLimitation, Persona, PersonaError, PersonaForge, PersonaOutput,
    ValidationResult,
};
pub use north_star::{
    Character, DiscoveryMechanism, EdgeCase, NorthStarBuilder, NorthStarError, NorthStarOutput,
    PlotHole, Scenario, SimulationResult, TimelineEvent,
};
pub use cdi_logger::{
    CdiEntry, CdiFunnel, CdiLogger, CdiSignal, InterviewOutcome, SignalStrength, SignalType,
};
