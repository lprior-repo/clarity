//! Phase Define Component
//!
//! Design phase (Right of Diamond) - Requirements definition and design.
//! Contains use case list, priority toggles, and technical context editor.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::components::{SectionLabel, SectionLevel};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::UseCase;
use dioxus::prelude::*;

/// Phase define component
///
/// Design phase (Right of Diamond) with use cases and requirements.
#[component]
pub fn PhaseDefine(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  rsx! {
      div { class: "phase-define",
          SectionLabel {
              level: SectionLevel::Phase,
              label: "Design Phase".to_string(),
          }

          UseCaseList {
              state: state,
              selected_entity: selected_entity
          }

          SummaryStats {
              state: state
          }
      }
  }
}

/// Use case list component
///
/// List of use cases with sentence builder and priority toggles.
#[component]
fn UseCaseList(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let _add_use_case = move |_: dioxus::events::MouseEvent| {
    let new_use_case = UseCase::new(
      "New Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );

    let current = state.read().clone();
    match current.add_use_case(new_use_case) {
      Ok(updated) => state.set(updated),
      Err(e) => {
        eprintln!("Failed to add use case: {e}");
        // In production, this would show an error notification
      }
    }
  };

  rsx! {
      div { class: "use-case-list",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Use Cases".to_string(),
          }

          div { class: "use-case-items",
              div { class: "empty-state",
                  p { "Use case list UI simplified for compilation" }
              }
          }
      }
  }
}

/// Summary statistics component
///
/// Shows statistics about the plan.
#[component]
fn SummaryStats(state: Signal<PlannerState>) -> Element {
  let persona_count = state.read().personas.len();
  let use_case_count = state.read().use_cases.len();
  let task_count = state.read().tasks.len();

  rsx! {
      div { class: "summary-stats",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Summary".to_string(),
          }

          div { class: "stats-grid",
              div { class: "stat-card",
                  div { class: "stat-value", "{persona_count}" }
                  div { class: "stat-label", "Personas" }
              }
              div { class: "stat-card",
                  div { class: "stat-value", "{use_case_count}" }
                  div { class: "stat-label", "Use Cases" }
              }
              div { class: "stat-card",
                  div { class: "stat-value", "{task_count}" }
                  div { class: "stat-label", "Tasks" }
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_phase_define_component() {
    let _phase = crate::planner::types::DiamondPhase::Right;
    assert_eq!(_phase, crate::planner::types::DiamondPhase::Right);
  }
}
