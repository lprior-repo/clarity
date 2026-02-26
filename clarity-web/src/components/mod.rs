#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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
pub use discover::{
    antithesis::AntithesisResponse, brutal_truths::*, extract_fields_button::*, extracting_progress::*,
    field_card::*, locked_phase::*, locked_summary::*, nonpersona_confirm::*, persona_confirm::*,
    preview_summary::*, problem_confirm::*, progressive_discover::*, prompt_textarea::*, quality_score::*,
    scenario_confirm::*, solution_confirm::*, state::*, straw_man::*, types::*, phases::*,
};
pub use graph_visualizer::GraphVisualizer;
pub use planning_coach::PlanningCoach;
// Re-export specific items from quality to avoid ambiguous glob re-exports
pub use quality::{QualityDimension, QualityScore, quality_issues};
pub use state_machine::StateMachine;
