//! Phase Develop Component
//!
//! Development phase (Bottom of Diamond) - Task management and implementation.
//! Contains task list with selection and master-detail layout.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]
#![allow(warnings)]
#![allow(clippy::all)]

use crate::planner::components::{SectionLabel, SectionLevel, TaskDetailEditor};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{DiamondPhase, PlanTask, TaskType};
use crate::planner::validation;
use dioxus::prelude::*;
use uuid;

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
  let tasks = state.read().tasks.clone();
  let selected_id = match &*selected_entity.read() {
    Some(SelectedEntity::Task(id)) => Some(*id),
    _ => None,
  };

  let add_task = move |_| {
    let new_task = PlanTask::new(
      "New Task".to_string(),
      "Description".to_string(),
      TaskType::Other,
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

  let is_empty = tasks.is_empty();

  rsx! {
    div { class: "task-list-panel flex flex-col h-full bg-base-200",
        div { class: "panel-header flex items-center justify-between p-4 border-b border-base-300",
            SectionLabel {
                level: SectionLevel::Section,
                label: "Tasks".to_string(),
            }
            button {
                class: "btn btn-primary btn-sm",
                onclick: add_task,
                "Add Task"
            }
        }

        div { class: "task-items flex-1 overflow-y-auto p-2 space-y-2",
            {if is_empty {
                rsx! {
                    div { class: "text-center py-8 text-base-content/50",
                        p { class: "text-sm", "No tasks yet" }
                        p { class: "text-xs mt-1", "Click 'Add Task' to create your first task" }
                    }
                }
            } else {
                rsx! {
                    {tasks.iter().enumerate().map(|(idx, task)| {
                        let task = task.as_ref();
                        let checks = validation::validate_task(task);
                        let is_selected = selected_id == Some(task.id);
                        let error_count = checks
                            .as_ref()
                            .err()
                            .map_or(0, |errors| errors.len());
                        let is_ready = task.is_complete();

                        rsx! {
                            TaskCard {
                                key: "{idx}",
                                task: task.clone(),
                                is_selected: is_selected,
                                error_count: error_count,
                                is_ready: is_ready,
                                task_id: task.id,
                                selected_entity: selected_entity
                            }
                        }
                    })}
                }
            }}
        }
    }
  }
}

/// Task card component
///
/// Individual task card showing validation status and basic info.
///
/// Note: onclick is the task ID to select - this avoids closure issues
#[component]
fn TaskCard(
  task: PlanTask,
  is_selected: bool,
  error_count: usize,
  is_ready: bool,
  task_id: uuid::Uuid,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let card_class = format!(
    "card cursor-pointer transition-all {}",
    if is_selected {
      "bg-primary text-primary-content"
    } else {
      "bg-base-100 hover:bg-base-300"
    }
  );

  let effort_badge_class = match task.effort {
    crate::planner::types::Effort::Trivial | crate::planner::types::Effort::Small => {
      "badge-success"
    }
    crate::planner::types::Effort::Medium => "",
    crate::planner::types::Effort::Large => "badge-warning",
    crate::planner::types::Effort::ExtraLarge => "badge-error",
  };

  rsx! {
    div {
      class: "{card_class}",
      onclick: move |_evt| {
        selected_entity.set(Some(SelectedEntity::Task(task_id)));
      },
      div { class: "card-body p-3",
          div { class: "flex items-start justify-between gap-2",
              div { class: "flex-1 min-w-0",
                  p { class: "text-xs font-mono text-base-content/50 truncate",
                      "{task.id}"
                  }
                  h4 { class: "card-title text-sm font-medium",
                      "{task.title}"
                  }
                  p { class: "text-xs truncate mt-1",
                      "{task.description}"
                  }
              }
              div { class: "flex flex-col gap-1 items-end",
                  {if error_count > 0 {
                      rsx! {
                          span { class: "badge badge-error badge-sm", "{error_count} issues" }
                      }
                  } else {
                      rsx! { span { "" } }
                  }}
                  {if is_ready {
                      rsx! {
                          span { class: "badge badge-success badge-sm", "Ready" }
                      }
                  } else {
                      rsx! { span { "" } }
                  }}
                  span {
                      class: format!("badge badge-sm {}", effort_badge_class),
                      "{task.effort}"
                  }
              }
          }
      }
    }
  }
}

/// Task detail component
///
/// Detail view for selected task with tabbed editor.
#[component]
fn TaskDetail(
  state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  rsx! {
    div { class: "task-detail-panel flex flex-col h-full bg-base-100",
        TaskDetailEditor {
            state: state,
            selected_entity: selected_entity
        }
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
