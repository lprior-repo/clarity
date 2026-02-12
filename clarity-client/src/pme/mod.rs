//! PME (Product-Market Engineer) Module
//!
//! Scientific rigor tools for the Double Diamond methodology.
//! Enforces evidence-based decision making throughout all phases.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod components;
pub mod discover_panel;
pub mod state;
pub mod types;

pub use components::{
  HypothesisEditor, HypothesisSummary, InterviewLogger, InterviewSummary, PersonaForge,
  PersonaSummary, ScenarioSummary, ScenarioValidator,
};
pub use discover_panel::{DiscoverTab, PmeDiscoverPanel, PmeDiscoverPanelProps};
pub use state::{PmeDiscoverProvider, PmeDiscoverSignals};
pub use types::*;
