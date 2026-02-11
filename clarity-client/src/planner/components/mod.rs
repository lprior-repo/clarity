#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Planner UI components
//!
//! Reusable components for the Diamond methodology planner interface.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
// This is a framework limitation, not our code using unwrap.
#![allow(clippy::disallowed_methods)]

// Temporarily disabled - incomplete implementation with web_sys dependencies
// pub mod coach;
pub mod diamond_stepper;
pub mod field_label;
pub mod list_editor;
pub mod phase_define;
pub mod phase_deliver;
pub mod phase_develop;
pub mod phase_discover;
pub mod planner_app;
pub mod section_label;
pub mod status_display;
pub mod task_detail_editor;
pub mod text_area;

// pub use coach::{CoachBubble, InlineTerminal, PlanningCoach, TerminalCommand};
pub use diamond_stepper::{get_progress_width, DiamondStepper};
pub use field_label::{FieldHint, FieldLabel};
pub use list_editor::{validate_list_items, ListEditor};
pub use phase_define::PhaseDefine;
pub use phase_deliver::{export_plan, ExportFormat, ExportResult, PhaseDeliver};
pub use phase_develop::PhaseDevelop;
pub use phase_discover::PhaseDiscover;
pub use planner_app::{PlannerApp, SaveResult};
pub use section_label::{SectionLabel, SectionLevel};
pub use status_display::{
  format_status, get_color_from_status, StatusBadge, StatusBadgeSize, StatusCard, StatusIndicator,
  StatusProgressBar, StatusSummary,
};
pub use task_detail_editor::{EditorTab, TaskDetailEditor};
pub use text_area::TextArea;
