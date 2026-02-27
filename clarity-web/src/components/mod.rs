#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(ambiguous_glob_reexports)]

// Re-export discover components
pub mod discover;

// Re-export quality components
pub mod quality;

// Individual component modules
pub mod artifact_panel;
pub mod graph_visualizer;
pub mod planning_coach;
pub mod state_machine;

// Re-exports
pub use artifact_panel::ArtifactPanel;
pub use discover::*;
pub use graph_visualizer::GraphVisualizer;
pub use planning_coach::PlanningCoach;
pub use quality::*;
pub use state_machine::StateMachine;
