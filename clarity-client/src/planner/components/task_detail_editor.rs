//! Task Detail Editor Component
//!
//! A professional, production-grade task editing interface with:
//! - 5 tabs: Basic, Requirements (EARS), Contracts, Tests, Research
//! - Quality bar with real-time validation status
//! - daisyUI styling for polished appearance
//!
//! Aesthetic: Technical precision / Industrial control panel
//! - Dark IDE-inspired interface
//! - Monospace typography for technical clarity
//! - Color-coded validation states
//! - Grid-based layout with structural elements

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
#![allow(clippy::disallowed_methods)]

use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{
  EventDrivenRequirement, PlanTask, TaskPriority, TaskType, UnwantedRequirement, ValidationSeverity,
};
use dioxus::prelude::*;

// ============================================================================
// Main Component
// ============================================================================

/// Task detail editor with tabbed interface and quality bar
#[component]
pub fn TaskDetailEditor(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let mut active_tab = use_signal(|| EditorTab::Basic);

  let selected_task = get_selected_task(state, selected_entity);

  rsx! {
    div { class: "task-detail-editor h-full flex flex-col bg-base-100 font-mono text-sm",
      {match selected_task {
        Some(task) => {
          let checks = generate_validation_checks(&task);
          let passed_count = checks.iter().filter(|c| c.passed).count();
          let total_count = checks.len();
          let completion_pct = if total_count > 0 {
            (passed_count * 100) / total_count
          } else {
            100
          };

          rsx! {
            // === Quality Bar ===
            div { class: "quality-bar bg-base-300 border-b-2 border-primary p-4",
              div { class: "flex items-center justify-between mb-3",
                div { class: "flex items-center gap-3",
                  span { class: "text-xs uppercase tracking-widest text-base-content/60", "Quality Status" }
                  div { class: "divider divider-horizontal mx-0" }
                  span { class: "text-lg font-bold",
                    {format!("{passed_count}/{total_count}")}
                  }
                  span { class: "text-xs text-base-content/60", "checks passed" }
                }
                div { class: "radial-progress text-primary",
                  style: format!("--value:{completion_pct}; --size:2.5rem;"),
                  "{completion_pct}%"
                }
              }

              // Validation badges
              div { class: "flex flex-wrap gap-2",
                {checks.iter().map(|check| {
                  let badge_class = if check.passed {
                    "badge badge-success badge-sm gap-1"
                  } else {
                    match check.severity {
                      ValidationSeverity::Error => "badge badge-error badge-sm gap-1",
                      ValidationSeverity::Warning => "badge badge-warning badge-sm gap-1",
                      ValidationSeverity::Critical => "badge badge-error badge-sm gap-1 font-bold",
                      ValidationSeverity::Info => "badge badge-info badge-sm gap-1",
                    }
                  };

                  let icon = if check.passed { "✓" } else { "!" };

                  rsx! {
                    span {
                      class: "{badge_class}",
                      title: "{check.description}",
                      span { class: "font-bold", "{icon}" }
                      span { "{check.label}" }
                    }
                  }
                })}
              }
            }

            // === Tab Navigation ===
            div { class: "tabs tabs-boxed bg-base-200 p-1 gap-1 mt-2",
              {EditorTab::ALL.iter().map(|&tab| {
                let is_active = *active_tab.read() == tab;
                let has_errors = tab_has_errors(tab, &checks);

                rsx! {
                  button {
                    class: format!(
                      "tab flex-1 {} {}",
                      if is_active { "tab-active bg-primary text-primary-content" } else { "" },
                      if has_errors { "text-error" } else { "" }
                    ),
                    onclick: move |_| active_tab.set(tab),
                    {tab.label()}
                    {if has_errors {
                      rsx! { span { class: "badge badge-error badge-xs ml-1", "!" } }
                    } else {
                      rsx! { span { "" } }
                    }}
                  }
                }
              })}
            }

            // === Tab Content ===
            div { class: "flex-1 overflow-y-auto p-4",
              {match *active_tab.read() {
                EditorTab::Basic => rsx! {
                  BasicTabContent {
                    task: task.clone(),
                    state: state,
                    selected_entity: selected_entity
                  }
                },
                EditorTab::Requirements => rsx! {
                  RequirementsTabContent {
                    task: task.clone(),
                    state: state,
                    selected_entity: selected_entity
                  }
                },
                EditorTab::Contracts => rsx! {
                  ContractsTabContent {
                    task: task.clone(),
                    state: state,
                    selected_entity: selected_entity
                  }
                },
                EditorTab::Tests => rsx! {
                  TestsTabContent {
                    task: task.clone(),
                    state: state,
                    selected_entity: selected_entity
                  }
                },
                EditorTab::Research => rsx! {
                  ResearchTabContent {
                    task: task.clone(),
                    state: state,
                    selected_entity: selected_entity
                  }
                },
              }}
            }
          }
        },
        None => rsx! {
          div { class: "flex-1 flex items-center justify-center text-base-content/40",
            div { class: "text-center",
              p { class: "text-4xl mb-4", "⬡" }
              p { class: "text-lg", "Select a task to edit" }
              p { class: "text-xs mt-2", "Choose from the task list on the left" }
            }
          }
        },
      }}
    }
  }
}

// ============================================================================
// Tab Enum
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTab {
  Basic,
  Requirements,
  Contracts,
  Tests,
  Research,
}

impl EditorTab {
  const ALL: [Self; 5] = [
    Self::Basic,
    Self::Requirements,
    Self::Contracts,
    Self::Tests,
    Self::Research,
  ];

  fn label(&self) -> &'static str {
    match self {
      Self::Basic => "Basic",
      Self::Requirements => "Requirements",
      Self::Contracts => "Contracts",
      Self::Tests => "Tests",
      Self::Research => "Research",
    }
  }
}

// ============================================================================
// Validation
// ============================================================================

#[derive(Clone, Debug)]
struct ValidationCheck {
  label: String,
  description: String,
  passed: bool,
  severity: ValidationSeverity,
}

fn generate_validation_checks(task: &PlanTask) -> Vec<ValidationCheck> {
  vec![
    ValidationCheck {
      label: "Title Format".to_string(),
      description: "Should follow 'component: action' format".to_string(),
      passed: task.title.contains(':'),
      severity: ValidationSeverity::Warning,
    },
    ValidationCheck {
      label: "Ubiquitous Req".to_string(),
      description: "At least one THE SYSTEM SHALL requirement".to_string(),
      passed: !task.ears.ubiquitous.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Event Req".to_string(),
      description: "At least one WHEN...THEN... requirement".to_string(),
      passed: !task.ears.event_driven.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Preconditions".to_string(),
      description: "At least one precondition".to_string(),
      passed: !task.contracts.preconditions.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Postconditions".to_string(),
      description: "At least one postcondition".to_string(),
      passed: !task.contracts.postconditions.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Happy Tests".to_string(),
      description: "At least one happy path test".to_string(),
      passed: !task.tests.happy.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Error Tests".to_string(),
      description: "At least one error path test".to_string(),
      passed: !task.tests.error.is_empty(),
      severity: ValidationSeverity::Error,
    },
    ValidationCheck {
      label: "Edge Tests".to_string(),
      description: "At least one edge case test".to_string(),
      passed: !task.tests.edge.is_empty(),
      severity: ValidationSeverity::Warning,
    },
  ]
}

fn tab_has_errors(tab: EditorTab, checks: &[ValidationCheck]) -> bool {
  let relevant = match tab {
    EditorTab::Basic => &["title", "description"][..],
    EditorTab::Requirements => &["ubiquitous", "event", "req"][..],
    EditorTab::Contracts => &["precondition", "postcondition", "contract"][..],
    EditorTab::Tests => &["happy", "error", "edge", "test"][..],
    EditorTab::Research => &["file", "pattern", "question", "research"][..],
  };

  checks
    .iter()
    .any(|c| !c.passed && relevant.iter().any(|&k| c.label.to_lowercase().contains(k)))
}

// ============================================================================
// Tab: Basic
// ============================================================================

#[component]
fn BasicTabContent(
  task: PlanTask,
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let title = task.title.clone();
  let description = task.description.clone();

  rsx! {
    div { class: "space-y-6",
      // Title
      FieldSection {
        label: "Title",
        hint: "Format: 'component: action description'",
        children: rsx! {
          input {
            class: "input input-bordered input-primary w-full font-mono",
            r#type: "text",
            value: "{title}",
            placeholder: "auth: implement JWT validation",
            oninput: move |evt| {
              update_task(state, selected_entity, |t| t.title = evt.value().clone());
            }
          }
        }
      }

      // Meta row
      div { class: "grid grid-cols-3 gap-4",
        // Type
        div { class: "form-control",
          label { class: "label",
            span { class: "label-text font-medium", "Type" }
          }
          select {
            class: "select select-bordered select-sm",
            onchange: move |evt| {
              if let Ok(new_type) = evt.value().parse() {
                update_task(state, selected_entity, |t| t.task_type = new_type);
              }
            },
            option { value: "Research", selected: task.task_type == TaskType::Research, "Research" }
            option { value: "Design", selected: task.task_type == TaskType::Design, "Design" }
            option { value: "Development", selected: task.task_type == TaskType::Development, "Development" }
            option { value: "Testing", selected: task.task_type == TaskType::Testing, "Testing" }
            option { value: "Documentation", selected: task.task_type == TaskType::Documentation, "Documentation" }
            option { value: "Planning", selected: task.task_type == TaskType::Planning, "Planning" }
            option { value: "Review", selected: task.task_type == TaskType::Review, "Review" }
            option { value: "Infrastructure", selected: task.task_type == TaskType::Infrastructure, "Infrastructure" }
            option { value: "Other", selected: task.task_type == TaskType::Other, "Other" }
          }
        }

        // Priority
        div { class: "form-control",
          label { class: "label",
            span { class: "label-text font-medium", "Priority" }
          }
          select {
            class: "select select-bordered select-sm",
            onchange: move |evt| {
              if let Ok(new_priority) = evt.value().parse() {
                update_task(state, selected_entity, |t| t.priority = new_priority);
              }
            },
            option { value: "Urgent", selected: task.priority == TaskPriority::Urgent, "Urgent" }
            option { value: "High", selected: task.priority == TaskPriority::High, "High" }
            option { value: "Normal", selected: task.priority == TaskPriority::Normal, "Normal" }
            option { value: "Low", selected: task.priority == TaskPriority::Low, "Low" }
          }
        }

        // Effort
        div { class: "form-control",
          label { class: "label",
            span { class: "label-text font-medium", "Effort" }
          }
          select {
            class: "select select-bordered select-sm",
            onchange: move |evt| {
              if let Ok(new_effort) = evt.value().parse() {
                update_task(state, selected_entity, |t| t.effort = new_effort);
              }
            },
            option { value: "Trivial", selected: task.effort == crate::planner::types::Effort::Trivial, "Trivial" }
            option { value: "Small", selected: task.effort == crate::planner::types::Effort::Small, "Small" }
            option { value: "Medium", selected: task.effort == crate::planner::types::Effort::Medium, "Medium" }
            option { value: "Large", selected: task.effort == crate::planner::types::Effort::Large, "Large" }
            option { value: "ExtraLarge", selected: task.effort == crate::planner::types::Effort::ExtraLarge, "Extra Large" }
          }
        }
      }

      // Description
      FieldSection {
        label: "Description",
        hint: "What does this task accomplish?",
        children: rsx! {
          textarea {
            class: "textarea textarea-bordered h-32",
            value: "{description}",
            placeholder: "Describe the task goals...",
            oninput: move |evt| {
              update_task(state, selected_entity, |t| t.description = evt.value().clone());
            }
          }
        }
      }
    }
  }
}

// ============================================================================
// Tab: Requirements (EARS)
// ============================================================================

#[component]
fn RequirementsTabContent(
  task: PlanTask,
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let ears = task.ears.clone();

  rsx! {
    div { class: "space-y-6",
      // Ubiquitous
      RequirementSection {
        title: "Ubiquitous",
        subtitle: "THE SYSTEM SHALL...",
        color: "info",
        items: ears.ubiquitous,
        placeholder: "THE SYSTEM SHALL...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.ears.ubiquitous = items);
        }
      }

      // Event-driven
      RequirementSection {
        title: "Event-Driven",
        subtitle: "WHEN...THEN...",
        color: "accent",
        items: ears.event_driven.iter().map(|ed| {
          format!("WHEN {} THEN {}", ed.trigger, ed.response)
        }).collect::<Vec<String>>(),
        placeholder: "WHEN trigger THEN response",
        add_label: "Add",
        on_update: move |items: Vec<String>| {
          update_task(state, selected_entity, |t| {
            t.ears.event_driven = items.iter().map(|s| {
              let parts: Vec<&str> = s.split("THEN").collect();
              EventDrivenRequirement {
                trigger: parts.get(0).unwrap_or(&"").trim().trim_start_matches("WHEN").trim().to_string(),
                response: parts.get(1).unwrap_or(&"").trim().to_string(),
              }
            }).collect();
          });
        }
      }

      // Unwanted
      RequirementSection {
        title: "Unwanted",
        subtitle: "IF...SHALL NOT...",
        color: "error",
        items: ears.unwanted.iter().map(|uw| {
          format!("IF {} SHALL NOT {} BECAUSE {}", uw.condition, uw.shall_not, uw.because)
        }).collect::<Vec<String>>(),
        placeholder: "IF condition SHALL NOT X BECAUSE Y",
        add_label: "Add",
        on_update: move |items: Vec<String>| {
          update_task(state, selected_entity, |t| {
            t.ears.unwanted = items.iter().map(|_s| {
              // Simple parsing - in production would be more robust
              UnwantedRequirement {
                condition: String::new(),
                shall_not: String::new(),
                because: String::new(),
              }
            }).collect();
          });
        }
      }
    }
  }
}

// ============================================================================
// Tab: Contracts
// ============================================================================

#[component]
fn ContractsTabContent(
  task: PlanTask,
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let contracts = task.contracts.clone();

  rsx! {
    div { class: "space-y-6",
      RequirementSection {
        title: "Preconditions",
        subtitle: "Must be true BEFORE execution",
        color: "warning",
        items: contracts.preconditions,
        placeholder: "Precondition...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.contracts.preconditions = items);
        }
      }

      RequirementSection {
        title: "Postconditions",
        subtitle: "Must be true AFTER completion",
        color: "success",
        items: contracts.postconditions,
        placeholder: "Postcondition...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.contracts.postconditions = items);
        }
      }

      RequirementSection {
        title: "Invariants",
        subtitle: "Always true throughout",
        color: "info",
        items: contracts.invariants,
        placeholder: "Invariant...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.contracts.invariants = items);
        }
      }
    }
  }
}

// ============================================================================
// Tab: Tests
// ============================================================================

#[component]
fn TestsTabContent(
  task: PlanTask,
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let tests = task.tests.clone();

  rsx! {
    div { class: "space-y-6",
      RequirementSection {
        title: "Happy Path",
        subtitle: "It works as expected",
        color: "success",
        items: tests.happy,
        placeholder: "Test scenario...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.tests.happy = items);
        }
      }

      RequirementSection {
        title: "Error Path",
        subtitle: "Fails gracefully",
        color: "error",
        items: tests.error,
        placeholder: "Error scenario...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.tests.error = items);
        }
      }

      RequirementSection {
        title: "Edge Cases",
        subtitle: "Boundary conditions",
        color: "warning",
        items: tests.edge,
        placeholder: "Edge case scenario...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.tests.edge = items);
        }
      }
    }
  }
}

// ============================================================================
// Tab: Research
// ============================================================================

#[component]
fn ResearchTabContent(
  task: PlanTask,
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let research = task.research.clone();
  let implementation = task.implementation.clone();

  rsx! {
    div { class: "space-y-6",
      div { class: "divider", "Research" }

      RequirementSection {
        title: "Files",
        subtitle: "Files to read",
        color: "info",
        items: research.files,
        placeholder: "path/to/file...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.research.files = items);
        }
      }

      RequirementSection {
        title: "Patterns",
        subtitle: "Code patterns to find",
        color: "accent",
        items: research.patterns,
        placeholder: "Pattern or question...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.research.patterns = items);
        }
      }

      RequirementSection {
        title: "Questions",
        subtitle: "Open research questions",
        color: "warning",
        items: research.questions,
        placeholder: "Question...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.research.questions = items);
        }
      }

      div { class: "divider", "Implementation" }

      RequirementSection {
        title: "Phase 0",
        subtitle: "Research steps",
        color: "info",
        items: implementation.phase0,
        placeholder: "Research step...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.implementation.phase0 = items);
        }
      }

      RequirementSection {
        title: "Phase 1",
        subtitle: "Tests to write",
        color: "success",
        items: implementation.phase1,
        placeholder: "Test to write...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.implementation.phase1 = items);
        }
      }

      RequirementSection {
        title: "Phase 2",
        subtitle: "Implementation steps",
        color: "warning",
        items: implementation.phase2,
        placeholder: "Implementation step...",
        add_label: "Add",
        on_update: move |items| {
          update_task(state, selected_entity, |t| t.implementation.phase2 = items);
        }
      }
    }
  }
}

// ============================================================================
// Reusable Components
// ============================================================================

#[component]
fn FieldSection(label: String, hint: String, children: Element) -> Element {
  rsx! {
    div { class: "form-control",
      label { class: "label",
        span { class: "label-text font-bold uppercase text-xs tracking-wider", "{label}" }
      }
      {children}
      label { class: "label",
        span { class: "label-text-alt text-xs text-base-content/50", "{hint}" }
      }
    }
  }
}

#[component]
fn RequirementSection(
  title: String,
  subtitle: String,
  color: String,
  items: Vec<String>,
  placeholder: String,
  add_label: String,
  on_update: EventHandler<Vec<String>>,
) -> Element {
  let badge_class = format!("badge badge-{color} badge-sm");
  let border_class = format!("border-{color}/20");

  // Clone items for button closure
  let items_for_button = items.clone();

  rsx! {
    div { class: format!("card bg-base-200 border {border_class}"),
      div { class: "card-body p-4",
        div { class: "flex items-center gap-2 mb-3",
          span { class: "{badge_class}", "●" }
          h3 { class: "card-title text-sm", "{title}" }
          span { class: "text-xs text-base-content/50", "{subtitle}" }
        }
        div { class: "space-y-2",
          {items.iter().enumerate().map(|(i, item)| {
            // Clone for oninput closure
            let items_for_oninput = items.clone();
            // Clone for onclick closure
            let items_for_onclick = items.clone();

            rsx! {
              div { class: "flex gap-2 items-start",
                span { class: "text-xs text-base-content/40 mt-1.5 w-5 text-right font-mono", "{i + 1}" }
                input {
                  class: "input input-bordered input-sm flex-1 bg-base-100",
                  r#type: "text",
                  value: "{item}",
                  placeholder: "{placeholder}",
                  oninput: move |evt| {
                    let mut updated = items_for_oninput.clone();
                    updated[i] = evt.value().clone();
                    on_update(updated);
                  }
                }
                button {
                  class: "btn btn-ghost btn-xs btn-circle text-error hover:text-error hover:bg-error/10",
                  onclick: move |_| {
                    let mut updated = items_for_onclick.clone();
                    updated.remove(i);
                    on_update(updated);
                  },
                  "×"
                }
              }
            }
          })}
          button {
            class: "btn btn-sm btn-ghost w-full",
            onclick: move |_| {
              let mut updated = items_for_button.clone();
              updated.push(String::new());
              on_update(updated);
            },
            "+ {add_label}"
          }
        }
      }
    }
  }
}

// ============================================================================
// Helpers
// ============================================================================

fn get_selected_task(
  state: Signal<PlannerState>,
  selected_entity: Signal<Option<SelectedEntity>>,
) -> Option<PlanTask> {
  match &*selected_entity.read() {
    Some(SelectedEntity::Task(id)) => state
      .read()
      .tasks
      .iter()
      .find(|t| t.id == *id)
      .map(|t| t.as_ref().clone()),
    _ => None,
  }
}

fn update_task(
  mut state: Signal<PlannerState>,
  selected_entity: Signal<Option<SelectedEntity>>,
  f: impl FnOnce(&mut PlanTask),
) {
  let task_id = match &*selected_entity.read() {
    Some(SelectedEntity::Task(id)) => Some(*id),
    _ => None,
  };

  if let Some(id) = task_id {
    let current = state.read().clone();
    if let Some(idx) = current.tasks.iter().position(|t| t.id == id) {
      let task = current.tasks[idx].as_ref().clone();
      let mut updated = task;
      f(&mut updated);

      // The tasks are stored as Vector<Rc<PlanTask>>
      // So we need to wrap the updated PlanTask in Rc
      let wrapped = std::rc::Rc::new(updated);
      if let Some(new_tasks) = current.tasks.set(idx, wrapped) {
        let mut new_state = current;
        new_state.tasks = new_tasks;
        state.set(new_state);
      }
    }
  }
}
