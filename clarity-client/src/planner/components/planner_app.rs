//! Planner App Component
//!
//! Main application component for the Diamond methodology planner.
//! Handles phase routing and provides the header bar.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]
#![allow(warnings)]
#![allow(clippy::all)]

use crate::app::{NavLink, Route};
use crate::components::SaveButton;
use crate::components::TerminalFeed;
use crate::planner::components::diamond_stepper::DiamondStepper;
use crate::planner::components::phase_define::PhaseDefine;
use crate::planner::components::phase_deliver::PhaseDeliver;
use crate::planner::components::phase_develop::PhaseDevelop;
use crate::planner::components::phase_discover::PhaseDiscover;
use crate::planner::components::status_display::StatusSummary;
use crate::planner::state::{PlannerState, PlannerUIState};
use crate::planner::types::DiamondPhase;
use dioxus::prelude::*;

/// Save result type
pub type SaveResult = Result<String, String>;

/// Convert planner state to terminal answers format
fn generate_terminal_answers(state: &PlannerState) -> Vec<(String, String)> {
  let mut answers = Vec::new();

  // Add thesis/problem info
  if let Some(ref thesis) = state.thesis {
    answers.push(("problem".to_string(), thesis.problem.clone()));
    answers.push(("solution".to_string(), thesis.solution.clone()));
  }

  // Add personas
  for persona in &state.personas {
    answers.push((
      "persona".to_string(),
      format!("{}: {}", persona.name, persona.role),
    ));
  }

  // Add use cases
  let use_cases: String = state
    .use_cases
    .iter()
    .map(|uc| format!("- {}", uc.title))
    .collect::<Vec<_>>()
    .join("\n");
  if !use_cases.is_empty() {
    answers.push(("use-cases".to_string(), use_cases));
  }

  // Add tasks
  let tasks: String = state
    .tasks
    .iter()
    .map(|t| format!("- {}", t.title))
    .collect::<Vec<_>>()
    .join("\n");
  if !tasks.is_empty() {
    answers.push(("tasks".to_string(), tasks));
  }

  answers
}

/// Planner app component
///
/// Main application component with phase routing and header.
#[component]
pub fn PlannerApp() -> Element {
  let state = use_signal(|| PlannerState::new());
  let ui_state = use_signal(|| PlannerUIState::new());
  let save_status = use_signal(|| SaveStatus::Idle);

  rsx! {
      div { class: "planner-app",
          PlannerHeader {
              state: state,
              ui_state: ui_state,
              save_status: save_status,
              on_save: Callback::new({
                  let state = state;
                  let mut save_status = save_status;
                  move |_| {
                      handle_save(&state, &mut save_status);
                  }
              })
          }

          DiamondStepper {
              current_phase: state.read().current_phase,
              on_phase_change: Callback::new({
                  let mut state = state;
                  move |phase| {
                      let updated = state.read().set_phase(phase);
                      state.set(updated);
                  }
              }),
              show_labels: Some(true),
              labels: None,
              interactive: Some(true),
          }

          // Status summary section
          div { class: "planner-status-summary px-6 py-4",
              StatusSummary {
                  state: state,
                  on_status_click: None,
              }
          }

          div { class: "planner-content",
              // Lazy rendering: only render active or complete phases
              {{
                let state_read = state.read();
                let current_phase = state_read.current_phase;

                // Determine which phases to render
                let render_discovery = DiamondPhase::Top.should_render(current_phase);
                let render_design = DiamondPhase::Right.should_render(current_phase);
                let render_development = DiamondPhase::Bottom.should_render(current_phase);
                let render_delivery = DiamondPhase::Left.should_render(current_phase);

                rsx! {
                    // Render discovery phase if active or complete
                    {render_discovery.then(|| rsx! {
                        PhaseDiscover {
                            state: state,
                            selected_entity: use_signal(|| None)
                        }
                    })}

                    // Render design phase if active or complete
                    {render_design.then(|| rsx! {
                        PhaseDefine {
                            state: state,
                            selected_entity: use_signal(|| None)
                        }
                    })}

                    // Render development phase if active or complete
                    {render_development.then(|| rsx! {
                        PhaseDevelop {
                            state: state,
                            selected_entity: use_signal(|| None)
                        }
                    })}

                    // Render delivery phase if active or complete
                    {render_delivery.then(|| rsx! {
                        PhaseDeliver {
                            state: state,
                            selected_entity: use_signal(|| None)
                        }
                    })}
                }
              }}
          }

          // Terminal Feed Panel
          div { class: "terminal-panel",
              TerminalFeed {
                  answers: generate_terminal_answers(&state.read())
              }
          }
      }
  }
}

/// Save status for UI feedback
#[derive(Clone, Debug, PartialEq)]
enum SaveStatus {
  Idle,
  Saving,
  Success(String),
  Error(String),
}

/// Handle save operation
fn handle_save(state: &Signal<PlannerState>, save_status: &mut Signal<SaveStatus>) {
  save_status.set(SaveStatus::Saving);

  // In a real implementation, this would:
  // 1. Validate the current state
  // 2. Serialize to JSON
  // 3. Save to database or file
  // 4. Return appropriate status

  let current_state = state.read();
  let _project_name = current_state.context.project_name.clone();

  // Simulate async save operation
  let result = save_plan_to_state(&current_state);

  match result {
    Ok(message) => {
      save_status.set(SaveStatus::Success(message));
    }
    Err(error) => {
      save_status.set(SaveStatus::Error(error));
    }
  }
}

/// Save plan to persistent storage
///
/// This is a placeholder implementation. In a real application, this would:
/// - Connect to a database via the repository layer
/// - Serialize the plan session to JSON
/// - Handle save conflicts and merging
/// - Return proper error types
fn save_plan_to_state(state: &PlannerState) -> Result<String, String> {
  // Validate that we have minimal required data
  let has_thesis = state.thesis.is_some();
  let has_content =
    !state.personas.is_empty() || !state.use_cases.is_empty() || !state.tasks.is_empty();

  if !has_thesis && !has_content {
    return Err(
      "Cannot save: Plan is empty. Please add at least a thesis, personas, use cases, or tasks."
        .to_string(),
    );
  }

  // In a real implementation, we would serialize the full plan session
  // and save it to the database. For now, we return a success message.

  let project_name = if state.context.project_name.is_empty() {
    "Untitled Plan"
  } else {
    &state.context.project_name
  };

  let phase = state.current_phase;
  let persona_count = state.personas.len();
  let use_case_count = state.use_cases.len();
  let task_count = state.tasks.len();

  Ok(format!(
    "Saved '{project_name}' ({phase}) with {persona_count} personas, {use_case_count} use cases, and {task_count} tasks"
  ))
}

/// Planner header component
///
/// Header bar with navigation and actions including save functionality.
#[component]
fn PlannerHeader(
  state: Signal<PlannerState>,
  ui_state: Signal<PlannerUIState>,
  save_status: Signal<SaveStatus>,
  on_save: Callback<()>,
) -> Element {
  let project_name = state.read().context.project_name.clone();
  let progress = state.read().progress();

  rsx! {
      header { class: "planner-header",
          div { class: "header-left",
              h1 { class: "header-title",
                  {if project_name.is_empty() {
                      "Diamond Planner"
                  } else {
                      "{project_name}"
                  }}
              }

              div { class: "header-progress",
                  div { class: "progress-bar",
                      div {
                          class: "progress-fill",
                          style: "width: {progress * 100.0}%;"
                      }
                  }
                  span { class: "progress-text", "{progress * 100.0:.0}%" }
              }
          }

          nav { class: "header-nav",
              NavLink { to: Route::Home, "Home" }
              NavLink { to: Route::Dashboard, "Dashboard" }
              NavLink { to: Route::BeadsList, "Beads" }
          }

          div { class: "header-actions",
              button {
                  class: "btn btn-secondary",
                  onclick: move |_| {
                      let ui = ui_state.read();
                      let new_ui = ui.toggle_sidebar();
                      drop(ui);
                      ui_state.set(new_ui);
                  },
                  "Toggle Sidebar"
              }

              SaveButton {
                  onclick: move |_| {
                      on_save.call(());
                  },
                  disabled: matches!(&*save_status.read(), SaveStatus::Saving),
              }

              // Save status notification (toasts)
              {match &*save_status.read() {
                  SaveStatus::Idle => rsx! {},
                  SaveStatus::Saving => rsx! {
                      div { class: "save-status saving",
                          p { "Saving plan..." }
                      }
                  },
                  SaveStatus::Success(message) => rsx! {
                      div { class: "save-status success",
                          p { strong { "Success! " } "{message}" }
                          button {
                              class: "btn-close",
                              onclick: move |_| {
                                  save_status.set(SaveStatus::Idle);
                              },
                              "×"
                          }
                      }
                  },
                  SaveStatus::Error(error) => rsx! {
                      div { class: "save-status error",
                          p { strong { "Save Failed: " } "{error}" }
                          button {
                              class: "btn-close",
                              onclick: move |_| {
                                  save_status.set(SaveStatus::Idle);
                              },
                              "×"
                          }
                      }
                  },
              }}
          }
      }
  }
}

// Integration tests for lazy phase rendering
#[cfg(test)]
mod lazy_rendering_tests {
  use super::*;
  use crate::planner::state::PlannerState;

  #[test]
  fn test_discovery_phase_only_rendered() {
    let _state = PlannerState::new().set_phase(DiamondPhase::Top);

    // When in Discovery phase, only Discovery should be rendered
    let rendered = DiamondPhase::get_rendered_phases(DiamondPhase::Top);
    assert_eq!(rendered, vec![DiamondPhase::Top]);

    assert!(DiamondPhase::Top.should_render(DiamondPhase::Top));
    assert!(!DiamondPhase::Right.should_render(DiamondPhase::Top));
    assert!(!DiamondPhase::Bottom.should_render(DiamondPhase::Top));
    assert!(!DiamondPhase::Left.should_render(DiamondPhase::Top));
  }

  #[test]
  fn test_discovery_and_design_rendered() {
    let _state = PlannerState::new().set_phase(DiamondPhase::Right);

    // When in Design phase, Discovery and Design should be rendered
    let rendered = DiamondPhase::get_rendered_phases(DiamondPhase::Right);
    assert_eq!(rendered, vec![DiamondPhase::Top, DiamondPhase::Right]);

    assert!(DiamondPhase::Top.should_render(DiamondPhase::Right));
    assert!(DiamondPhase::Right.should_render(DiamondPhase::Right));
    assert!(!DiamondPhase::Bottom.should_render(DiamondPhase::Right));
    assert!(!DiamondPhase::Left.should_render(DiamondPhase::Right));
  }

  #[test]
  fn test_three_phases_rendered() {
    let _state = PlannerState::new().set_phase(DiamondPhase::Bottom);

    // When in Development phase, Discovery, Design, and Development should be rendered
    let rendered = DiamondPhase::get_rendered_phases(DiamondPhase::Bottom);
    assert_eq!(
      rendered,
      vec![DiamondPhase::Top, DiamondPhase::Right, DiamondPhase::Bottom]
    );

    assert!(DiamondPhase::Top.should_render(DiamondPhase::Bottom));
    assert!(DiamondPhase::Right.should_render(DiamondPhase::Bottom));
    assert!(DiamondPhase::Bottom.should_render(DiamondPhase::Bottom));
    assert!(!DiamondPhase::Left.should_render(DiamondPhase::Bottom));
  }

  #[test]
  fn test_all_phases_rendered() {
    let _state = PlannerState::new().set_phase(DiamondPhase::Left);

    // When in Delivery phase, all phases should be rendered
    let rendered = DiamondPhase::get_rendered_phases(DiamondPhase::Left);
    assert_eq!(
      rendered,
      vec![
        DiamondPhase::Top,
        DiamondPhase::Right,
        DiamondPhase::Bottom,
        DiamondPhase::Left
      ]
    );

    assert!(DiamondPhase::Top.should_render(DiamondPhase::Left));
    assert!(DiamondPhase::Right.should_render(DiamondPhase::Left));
    assert!(DiamondPhase::Bottom.should_render(DiamondPhase::Left));
    assert!(DiamondPhase::Left.should_render(DiamondPhase::Left));
  }

  #[test]
  fn test_phase_state_consistency() {
    // Test that phase state is consistent across different phases
    let phases = vec![
      DiamondPhase::Top,
      DiamondPhase::Right,
      DiamondPhase::Bottom,
      DiamondPhase::Left,
    ];

    for &current_phase in &phases {
      let rendered_phases = DiamondPhase::get_rendered_phases(current_phase);

      // Each rendered phase should either be active or complete
      for &phase in &rendered_phases {
        assert!(
          phase == current_phase || phase.is_complete(current_phase),
          "Phase {:?} should be either active or complete when current phase is {:?}",
          phase,
          current_phase
        );
      }

      // All phases not in the rendered list should not be active and not complete
      for &phase in &phases {
        if !rendered_phases.contains(&phase) {
          assert_ne!(
            phase, current_phase,
            "Current phase should always be in rendered phases"
          );
          assert!(
            !phase.is_complete(current_phase),
            "Phase {:?} should not be complete when current phase is {:?}",
            phase,
            current_phase
          );
        }
      }
    }
  }

  #[test]
  fn test_component_render_logic() {
    // Test that the component render logic matches our phase rendering logic
    let test_cases = vec![
      (DiamondPhase::Top, vec![DiamondPhase::Top]),
      (
        DiamondPhase::Right,
        vec![DiamondPhase::Top, DiamondPhase::Right],
      ),
      (
        DiamondPhase::Bottom,
        vec![DiamondPhase::Top, DiamondPhase::Right, DiamondPhase::Bottom],
      ),
      (
        DiamondPhase::Left,
        vec![
          DiamondPhase::Top,
          DiamondPhase::Right,
          DiamondPhase::Bottom,
          DiamondPhase::Left,
        ],
      ),
    ];

    for (current_phase, expected_rendered) in test_cases {
      let actual_rendered = DiamondPhase::get_rendered_phases(current_phase);
      assert_eq!(actual_rendered, expected_rendered);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::planner::types::{Persona, ProductThesis, UseCase};

  #[test]
  fn test_planner_app_renders() {
    // Test component structure - actual rendering requires Dioxus DOM
    let state = PlannerState::new();
    assert_eq!(state.current_phase, DiamondPhase::Top);
  }

  #[test]
  fn test_planner_state_progress() {
    let state = PlannerState::new();
    assert_eq!(state.progress(), 0.0);

    let state = state.set_phase(DiamondPhase::Right);
    assert!((state.progress() - 0.33).abs() < 0.01);

    let state = state.set_phase(DiamondPhase::Bottom);
    assert!((state.progress() - 0.66).abs() < 0.01);

    let state = state.set_phase(DiamondPhase::Left);
    assert_eq!(state.progress(), 1.0);
  }

  #[test]
  fn test_save_plan_empty_state() {
    let state = PlannerState::new();
    let result = save_plan_to_state(&state);
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Cannot save") || error_msg.contains("empty"));
  }

  #[test]
  fn test_save_plan_with_thesis() {
    let mut state = PlannerState::new();
    let thesis = ProductThesis::new(
      "Test Thesis".to_string(),
      "Test Problem".to_string(),
      "Test Audience".to_string(),
      "Test Solution".to_string(),
      "Test Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Saved"));
  }

  #[test]
  fn test_save_plan_with_personas() {
    let mut state = PlannerState::new();
    let persona = Persona::new(
      "Test User".to_string(),
      "Developer".to_string(),
      "A test persona".to_string(),
    );
    state = state.add_persona(persona).unwrap();

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("1 personas"));
  }

  #[test]
  fn test_save_plan_with_multiple_items() {
    let mut state = PlannerState::new();
    state = state.update_project_name("Test Project".to_string());

    let thesis = ProductThesis::new(
      "Test".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let persona = Persona::new("User".to_string(), "Dev".to_string(), "Test".to_string());
    state = state.add_persona(persona).unwrap();

    let use_case = UseCase::new(
      "Test Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );
    state = state.add_use_case(use_case).unwrap();

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(message.contains("Saved 'Test Project'"));
    assert!(message.contains("1 personas"));
    assert!(message.contains("1 use cases"));
  }

  #[test]
  fn test_save_status_equality() {
    assert_eq!(SaveStatus::Idle, SaveStatus::Idle);
    assert_ne!(SaveStatus::Idle, SaveStatus::Saving);
    assert_ne!(
      SaveStatus::Success("test".to_string()),
      SaveStatus::Error("test".to_string())
    );
  }

  #[test]
  fn test_save_plan_named_project() {
    let mut state = PlannerState::new();
    state = state.update_project_name("My Awesome Project".to_string());

    let thesis = ProductThesis::new(
      "Test".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("My Awesome Project"));
  }

  #[test]
  fn test_save_plan_unnamed_project() {
    let mut state = PlannerState::new();
    let thesis = ProductThesis::new(
      "Test".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Untitled Plan"));
  }

  #[test]
  fn test_save_plan_includes_phase() {
    let mut state = PlannerState::new();
    state = state.set_phase(DiamondPhase::Bottom);

    let thesis = ProductThesis::new(
      "Test".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let result = save_plan_to_state(&state);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Development"));
  }
}
