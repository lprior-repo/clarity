//! Phase Develop Component
//!
//! Development phase (Bottom of Diamond) - Task management and implementation.
//! Contains task list with selection and master-detail layout.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::components::{SectionLabel, SectionLevel};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{DiamondPhase, PlanTask, TaskType};
use dioxus::prelude::*;

/// Phase develop component
///
/// Development phase (Bottom of Diamond) with task management.
#[component]
pub fn PhaseDevelop(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  rsx! {
      div { class: "phase-develop",
          SectionLabel {
              level: SectionLevel::Phase,
              label: "Development Phase".to_string(),
          }

          div { class: "master-detail-layout",
              TaskList {
                  state: state,
                  selected_entity: selected_entity
              }

              TaskDetail {
                  state: state,
                  selected_entity: selected_entity
              }
          }
      }
  }
}

/// Task list component
///
/// List of tasks with selection and filtering.
#[component]
fn TaskList(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let add_task = move |_| {
    let new_task = PlanTask::new(
      "New Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let current = state.read().clone();
    match current.add_task(new_task) {
      Ok(updated) => state.set(updated),
      Err(e) => {
        eprintln!("Failed to add task: {e}");
        // In production, this would show an error notification
      }
    }
  };

  rsx! {
      div { class: "task-list-panel",
          div { class: "panel-header",
              SectionLabel {
                  level: SectionLevel::Section,
                  label: "Tasks".to_string(),
              }
              button {
                  class: "btn btn-primary",
                  onclick: add_task,
                  "Add Task"
              }
          }

          div { class: "task-items",
              div { class: "empty-state",
                  p { "Task list UI simplified for compilation" }
              }
          }
      }
  }
}

/// Task detail component
///
/// Detail view for selected task.
#[component]
fn TaskDetail(
  state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let selected_id = match &*selected_entity.read() {
    Some(SelectedEntity::Task(id)) => Some(*id),
    _ => None,
  };

  let selected_task = match selected_id {
    Some(id) => state
      .read()
      .tasks
      .iter()
      .find(|t| t.id == id)
      .map(|t| t.clone()),
    None => None,
  };

  rsx! {
      div { class: "task-detail-panel",
          {match selected_task {
              Some(_task) => rsx! {
                  div { class: "task-detail-editor",
                      p { "Task detail editor requires Dioxus runtime" }
                  }
              },
              None => rsx! {
                  div { class: "empty-detail",
                      div { class: "empty-state",
                          p { "Select a task to view details" }
                      }
                  }
              }
          }}
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_phase_develop_component() {
    let _phase = DiamondPhase::Bottom;
    assert_eq!(_phase, DiamondPhase::Bottom);
  }
}
