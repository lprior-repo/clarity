//! Scenario primitive (Character + Simulation)
//!
//! From The Product-Minded Engineer:
//! "User Scenario" is core primitive of product thinking. A scenario is not merely
//! a story; it's a rigorous simulation used to elicit critical thinking.
//!
//! This module implements the Scenario primitive as specified in the Mental Lattice
//! Framework, integrated with the Product-Minded Engineer Double Diamond process.
//!
//! # Architecture
//!
//! The Scenario primitive consists of two main components:
//! - **Character**: A Persona with a Motivation (the "I Want" moment)
//! - **Simulation**: A plot - step-by-step sequence of actions with logical flow
//!
//! # Core Concepts
//!
//! ## Character = Persona + Motivation
//!
//! ### Persona
//!
//! Defines who is acting in the scenario:
//! - **Demographics**: Background characteristics (age, location, etc.)
//! - **Means**: Resources available to them (time, money, skills, tools)
//! - **Universal Limitations**: Cognitive constraints all humans share
//!     (lazy, distracted, risk-averse, impatient, forgetful)
//!
//! ### Motivation
//!
//! The "I Want" moment - Root Cause Analysis (RCA) of WHY the character
//! needs this particular feature or outcome at this specific moment.
//! Must be strong enough to compel action through friction.
//!
//! ## Simulation
//!
//! The plot - a sequence of steps where each:
//! - **Trigger**: What prompts the action
//! - **Action**: What the character does
//! - **Expected Outcome**: What should happen
//!
//! ### Plot Holes
//!
//! Inconsistencies in the narrative that make the scenario unrealistic:
//! - **Discovery Mechanism Missing**: How did user discover feature?
//! - **Edge Case Unhandled**: What if user clicks wrong button?
//! - **Timeline Inconsistent**: Sequence impossible
//!
//! # Design Principles
//!
//! 1. **Mathematical Proof**: Every step must follow logically from the last
//! 2. **No Hand-Waving**: All actions must be justifiable, not magic
//! 3. **Plot Hole Detection**: Automatically flag narrative inconsistencies
//! 4. **Universal Human Attributes**: Account for shared limitations
//!
//! # Dependencies
//!
//! - `uuid`: For generating unique IDs
//! - `serde`: For serialization
//! - `chrono`: For timestamp handling
//!
//! # Module Structure
//!
//! ```text,ignore
//! // Domain exports
//! pub use crate::planner_domain::character;
//! pub use crate::planner_domain::motivation;
//! pub use crate::planner_domain::scenarios::prelude::*;
//! ```
//!
//! # Core Types
//!
//! ## Scenario
//!
//! The complete scenario with character and simulation:
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use serde::{Serialize, Deserialize};
//!
//! /// North Star Scenario - ideal future state narrative
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! pub struct NorthStarScenario {
//!     /// Links to the Persona this scenario is about
//!     pub persona_id: Uuid,
//!
//!     /// The motivation - "I Want" moment with Root Cause Analysis
//!     pub motivation: Motivation,
//!
//!     /// The narrative steps - chronological sequence of actions
//!     pub narrative_steps: Vec<ScenarioStep>,
//!
//!     /// Detected plot holes - inconsistencies in the narrative
//!     pub plot_holes: Vec<PlotHole>,
//! }
//!
//! ## Scenario Step
//!
//! A single action in the simulation plot:
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use serde::{Serialize, Deserialize};
//!
//! /// Single step in the scenario simulation
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! pub struct ScenarioStep {
//!     /// What prompts the action
//!     pub trigger: String,
//!
//!     /// What the character does
//!     pub action: String,
//!
//!     /// What should happen as a result
//!     pub expected_outcome: String,
//! }
//!
//! ## Motivation
//!
//! The "I Want" moment - Root Cause Analysis:
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use serde::{Serialize, Deserialize};
//!
//! /// Character motivation - why they need something now
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! pub struct Motivation {
//!     /// Root cause analysis - WHY does character want this?
//!     pub root_cause: String,
//! }
//!
//! ## Plot Hole
//!
//! Inconsistency detected in the narrative:
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use serde::{Serialize, Deserialize};
//!
//! /// Plot hole - inconsistency in scenario narrative
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! pub enum PlotHole {
//!     /// User didn't specify how they discovered a feature
//!     DiscoveryMechanismMissing {
//!         question: String,
//!     },
//!
//!     /// Edge case wasn't handled in narrative
//!     EdgeCaseUnhandled {
//!         scenario: String,
//!     },
//!
//!     /// Timeline doesn't make logical sense
//!     TimelineInconsistent {
//!         issue: String,
//!     },
//! }
//! }
//!
//! # Constructor
//!
//! ## Functions
//!
//! ### Scenario Creation
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use super::*;
//!
//! /// Create a new North Star Scenario
//! #[must_use]
//! pub fn new_north_star_scenario(
//!     persona_id: Uuid,
//!     motivation: Motivation,
//!     narrative_steps: Vec<ScenarioStep>,
//! ) -> NorthStarScenario {
//!     let scenario = NorthStarScenario {
//!         persona_id,
//!         motivation,
//!         narrative_steps,
//!         plot_holes: Vec::new(),
//!     };
//!
//!     // Auto-detect plot holes
//!     scenario.plot_holes = detect_plot_holes(&scenario);
//!
//!     scenario
//! }
//! ```
//!
//! ### Plot Hole Detection
//!
//! ```rust,ignore
//! use super::*;
//!
//! /// Detect plot holes in a scenario narrative
//! #[must_use]
//! pub fn detect_plot_holes(scenario: &NorthStarScenario) -> Vec<PlotHole> {
//!     let mut holes = Vec::new();
//!
//!     // Check for discovery mechanism
//!     let has_discovery = scenario.narrative_steps.iter()
//!         .any(|step| step.action.to_lowercase().contains("discover")
//!             || step.action.to_lowercase().contains("learn about"));
//!
//!     if !has_discovery {
//!         holes.push(PlotHole::DiscoveryMechanismMissing {
//!             question: "How did user discover this feature?".into(),
//!         });
//!     }
//!
//!     // Check for edge cases (error handling)
//!     let has_error_handling = scenario.narrative_steps.iter()
//!         .any(|step| step.action.to_lowercase().contains("error"));
//!
//!     if !has_error_handling {
//!         holes.push(PlotHole::EdgeCaseUnhandled {
//!             scenario: "What happens when something goes wrong?".into(),
//!         });
//!     }
//!
//!     // Check for timeline consistency
//!     let steps_count = scenario.narrative_steps.len();
//!     for (i, step) in scenario.narrative_steps.iter().enumerate() {
//!         // After the first step, we should have both trigger and expected outcome
//!         if i > 0 {
//!             let has_trigger = !step.trigger.is_empty();
//!             let has_outcome = !step.expected_outcome.is_empty();
//!
//!             // Missing either trigger or expected outcome = hole
//!             if !has_trigger || !has_outcome {
//!                 holes.push(PlotHole::TimelineInconsistent {
//!                     issue: format!("Step {}: has trigger '{}' but no expected outcome",
//!                         i + 1, step.trigger, step.expected_outcome),
//!                 });
//!             }
//!         }
//!     }
//!
//!     holes
//! }
//! }
//! ```
//!
//! # Tests
//!
//! Comprehensive unit tests for the Scenario primitive:
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!
//!     #[test]
//!     fn test_new_scenario_creates_with_persona() {
//!         let persona_id = Uuid::new_v4();
//!         let motivation = Motivation {
//!             root_cause: "User needs to track tasks".into(),
//!         };
//!
//!         let scenario = new_north_star_scenario(persona_id, motivation);
//!
//!         assert_eq!(scenario.narrative_steps.len(), 0);
//!         assert_eq!(scenario.plot_holes.len(), 0);
//!     }
//!
//!     #[test]
//!     fn test_detect_plot_holes_on_complete_scenario() {
//!         let persona_id = Uuid::new_v4();
//!         let motivation = Motivation {
//!             root_cause: "Test".into(),
//!         };
//!
//!         let scenario = NorthStarScenario {
//!             persona_id,
//!             motivation,
//!             narrative_steps: vec![],
//!             plot_holes: vec![],
//!         };
//!
//!         let holes = detect_plot_holes(&scenario);
//!
//!         assert!(holes.is_empty(), "Complete scenario should have no plot holes");
//!     }
//!
//!     #[test]
//!     fn test_detects_missing_discovery_mechanism() {
//!         let persona_id = Uuid::new_v4();
//!         let motivation = Motivation {
//!             root_cause: "Test".into(),
//!         };
//!
//!         let scenario = new_north_star_scenario(persona_id, motivation);
//!
//!         let holes = detect_plot_holes(&scenario);
//!
//!         assert!(!holes.is_empty(), "Should detect missing discovery mechanism");
//!         assert!(holes.iter().any(|h| matches!(h,
//!             PlotHole::DiscoveryMechanismMissing(_))));
//!     }
//! }
//! ```
//!
//! # Re-exports
//!
//! Export the core types for use by other modules:
//!
//! ```rust,ignore
//! pub use crate::planner_domain::scenarios::prelude::*;
//! ```
