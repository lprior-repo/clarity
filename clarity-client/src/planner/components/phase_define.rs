//! Phase Define Component
//!
//! Design phase (Right of Diamond) - Requirements definition and design.
//! Contains use case list, priority toggles, and technical context editor.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::components::{ListEditor, SectionLabel, SectionLevel, TextArea};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{Persona, UseCase, UseCasePriority};
use dioxus::prelude::*;
use rpds::Vector;
use uuid::Uuid;

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
  let use_cases = state.read().use_cases.clone();
  let personas = state.read().personas.clone();

  let add_use_case = move |_: dioxus::events::MouseEvent| {
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

  // Convert use_cases vector to a Vec for iteration
  let use_cases_vec: Vec<UseCase> = use_cases.iter().map(|u| u.as_ref().clone()).collect();

  rsx! {
      div { class: "use-case-list",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Use Cases".to_string(),
          }

          div { class: "use-case-actions",
              button {
                  class: "btn btn-primary",
                  onclick: add_use_case,
                  "Add Use Case"
              }
          }

          div { class: "use-case-items",
              if use_cases_vec.is_empty() {
                  div { class: "empty-state",
                      p { "No use cases yet. Click 'Add Use Case' to create your first one." }
                  }
              } else {
                  for use_case in use_cases_vec {
                      UseCaseCard {
                          key: "{use_case.id}",
                          use_case: use_case.clone(),
                          personas: personas.clone(),
                          state: state,
                      }
                  }
              }
          }
      }
  }
}

/// Use case card component
///
/// Displays a single use case with all its editable fields.
#[component]
fn UseCaseCard(
  use_case: UseCase,
  personas: Vector<std::rc::Rc<Persona>>,
  mut state: Signal<PlannerState>,
) -> Element {
  let use_case_id = use_case.id;

  let priority_class = match use_case.priority {
    UseCasePriority::Critical => "priority-critical",
    UseCasePriority::High => "priority-high",
    UseCasePriority::Medium => "priority-medium",
    UseCasePriority::Low => "priority-low",
  };

  let personas_vec: Vec<Persona> = personas.iter().map(|p| p.as_ref().clone()).collect();

  let on_remove = move |_| {
    let current = state.read().clone();
    let updated = current.remove_use_case(use_case_id);
    state.set(updated);
  };

  rsx! {
      div { class: format!("use-case-card {}", priority_class),
          div { class: "use-case-header",
              input {
                  class: "use-case-title-input",
                  value: "{use_case.title}",
                  placeholder: "Use case title...",
                  oninput: move |e: Event<dioxus::prelude::FormData>| {
                      let title = e.value();
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.title = title;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  }
              }

              div { class: "use-case-priority",
                  button {
                      class: format!("priority-btn {}", if use_case.priority == UseCasePriority::Critical { "active" } else { "" }),
                      onclick: move |_| {
                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(uc) = uc_result {
                              let updated = current.update_use_case(use_case_id, uc.with_priority(UseCasePriority::Critical));
                              state.set(updated);
                          }
                      },
                      title: "Critical priority",
                      "Critical"
                  }
                  button {
                      class: format!("priority-btn {}", if use_case.priority == UseCasePriority::High { "active" } else { "" }),
                      onclick: move |_| {
                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(uc) = uc_result {
                              let updated = current.update_use_case(use_case_id, uc.with_priority(UseCasePriority::High));
                              state.set(updated);
                          }
                      },
                      title: "High priority",
                      "High"
                  }
                  button {
                      class: format!("priority-btn {}", if use_case.priority == UseCasePriority::Medium { "active" } else { "" }),
                      onclick: move |_| {
                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(uc) = uc_result {
                              let updated = current.update_use_case(use_case_id, uc.with_priority(UseCasePriority::Medium));
                              state.set(updated);
                          }
                      },
                      title: "Medium priority",
                      "Medium"
                  }
                  button {
                      class: format!("priority-btn {}", if use_case.priority == UseCasePriority::Low { "active" } else { "" }),
                      onclick: move |_| {
                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(uc) = uc_result {
                              let updated = current.update_use_case(use_case_id, uc.with_priority(UseCasePriority::Low));
                              state.set(updated);
                          }
                      },
                      title: "Low priority",
                      "Low"
                  }
              }

              button {
                  class: "btn btn-icon btn-danger",
                  onclick: on_remove,
                  title: "Remove use case",
                  "×"
              }
          }

          div { class: "use-case-body",
              TextArea {
                  label: "Description".to_string(),
                  value: use_case.description.clone(),
                  on_change: move |desc: String| {
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.description = desc;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  },
                  placeholder: Some("Describe what this use case accomplishes...".to_string()),
                  hint: Some("Be specific about the user's goal".to_string()),
                  required: Some(true),
                  rows: Some(3),
                  max_length: Some(500),
              }

              div { class: "use-case-field",
                  label { class: "field-label", "Trigger" }
                  input {
                      class: "use-case-trigger-input",
                      value: "{use_case.trigger}",
                      placeholder: "e.g., User clicks 'Save' button",
                      oninput: move |e: Event<dioxus::prelude::FormData>| {
                          let trigger = e.value();
                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(mut uc) = uc_result {
                              uc.trigger = trigger;
                              let updated = current.update_use_case(use_case_id, uc);
                              state.set(updated);
                          }
                      }
                  }
              }

              div { class: "use-case-persona-section",
                  label { class: "field-label", "Associated Persona" }
                  select {
                      class: "use-case-persona-select",
                      onchange: move |e: Event<dioxus::prelude::FormData>| {
                          let value = e.value();
                          let persona_id = if value.is_empty() {
                              None
                          } else if let Ok(id) = Uuid::parse_str(&value) {
                              Some(id)
                          } else {
                              None
                          };

                          let current = state.read().clone();
                          let uc_result = current.use_cases.iter()
                              .find(|u| u.id == use_case_id)
                              .map(|u| u.as_ref().clone());

                          if let Some(mut uc) = uc_result {
                              uc.persona_id = persona_id;
                              let updated = current.update_use_case(use_case_id, uc);
                              state.set(updated);
                          }
                      },
                      option {
                          value: "",
                          "None"
                      }
                      for persona in personas_vec.iter() {
                          option {
                              value: format!("{}", persona.id),
                              selected: use_case.persona_id.map_or(false, |id| id == persona.id),
                              "{persona.name} - {persona.role}"
                          }
                      }
                  }
              }

              ListEditor {
                  label: "Preconditions".to_string(),
                  hint: Some("What must be true before this use case can start?".to_string()),
                  items: use_case.preconditions.clone(),
                  on_change: move |items: Vec<String>| {
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.preconditions = items;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  },
                  placeholder: Some("e.g., User is logged in".to_string()),
              }

              ListEditor {
                  label: "Main Flow".to_string(),
                  hint: Some("Step-by-step description of the primary path".to_string()),
                  items: use_case.main_flow.clone(),
                  on_change: move |items: Vec<String>| {
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.main_flow = items;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  },
                  placeholder: Some("e.g., User enters credentials".to_string()),
                  required: Some(true),
              }

              ListEditor {
                  label: "Alternative Flows".to_string(),
                  hint: Some("Alternative paths and error handling".to_string()),
                  items: use_case.alternative_flows.clone(),
                  on_change: move |items: Vec<String>| {
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.alternative_flows = items;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  },
                  placeholder: Some("e.g., If login fails, show error".to_string()),
              }

              ListEditor {
                  label: "Postconditions".to_string(),
                  hint: Some("What must be true after this use case completes?".to_string()),
                  items: use_case.postconditions.clone(),
                  on_change: move |items: Vec<String>| {
                      let current = state.read().clone();
                      let uc_result = current.use_cases.iter()
                          .find(|u| u.id == use_case_id)
                          .map(|u| u.as_ref().clone());

                      if let Some(mut uc) = uc_result {
                          uc.postconditions = items;
                          let updated = current.update_use_case(use_case_id, uc);
                          state.set(updated);
                      }
                  },
                  placeholder: Some("e.g., User is redirected to dashboard".to_string()),
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
  use super::*;
  use crate::planner::state::PlannerState;
  use crate::planner::types::{UseCase, UseCasePriority};
  use uuid::Uuid;

  /// Helper to create a test use case with valid defaults
  fn create_test_use_case(title: &str) -> UseCase {
    UseCase::new(
      title.to_string(),
      "Test description".to_string(),
      "Test trigger".to_string(),
    )
  }

  //
  // Bounded Context: Phase Define (Design Phase)
  // Aggregate Root: PlannerState
  // Entity: UseCase
  //

  #[test]
  fn hostile_attack_given_empty_state_when_add_use_case_then_use_case_appears_in_list() {
    // Given: Empty planner state
    let state = PlannerState::new();
    assert_eq!(state.use_cases.len(), 0, "State should start empty");

    // When: Add use case
    let use_case = create_test_use_case("First Use Case");
    let result = state.add_use_case(use_case);

    // Then: Use case appears in list
    assert!(
      result.is_ok(),
      "Adding use case to empty state should succeed"
    );
    let updated_state = result.unwrap();
    assert_eq!(
      updated_state.use_cases.len(),
      1,
      "State should contain exactly one use case"
    );
    assert_eq!(
      updated_state.use_cases.get(0).map(|uc| &uc.title),
      Some(&"First Use Case".to_string()),
      "Added use case should have correct title"
    );
  }

  #[test]
  fn hostile_attack_given_use_case_exists_when_update_priority_then_priority_changes() {
    // Given: State with use case
    let state = PlannerState::new();
    let use_case = create_test_use_case("Priority Test");
    let state = state.add_use_case(use_case).unwrap();
    let use_case_id = state.use_cases.get(0).map(|uc| uc.id).unwrap();

    // When: Update priority to Critical
    let mut uc = state
      .use_cases
      .iter()
      .find(|u| u.id == use_case_id)
      .map(|u| u.as_ref().clone())
      .unwrap();
    uc = uc.with_priority(UseCasePriority::Critical);
    let updated_state = state.update_use_case(use_case_id, uc);

    // Then: Priority changes
    let updated_uc = updated_state
      .use_cases
      .iter()
      .find(|u| u.id == use_case_id)
      .map(|u| u.as_ref());
    assert_eq!(
      updated_uc.map(|u| u.priority),
      Some(UseCasePriority::Critical),
      "Priority should be updated to Critical"
    );
  }

  #[test]
  fn hostile_attack_given_use_case_exists_when_remove_use_case_then_use_case_removed_from_list() {
    // Given: State with use case
    let state = PlannerState::new();
    let use_case = create_test_use_case("Removable");
    let state = state.add_use_case(use_case).unwrap();
    let use_case_id = state.use_cases.get(0).map(|uc| uc.id).unwrap();
    assert_eq!(state.use_cases.len(), 1, "Should have one use case");

    // When: Remove use case
    let updated_state = state.remove_use_case(use_case_id);

    // Then: Use case removed from list
    assert_eq!(
      updated_state.use_cases.len(),
      0,
      "Use case should be removed"
    );
    assert!(
      updated_state.use_cases.iter().all(|u| u.id != use_case_id),
      "Removed use case ID should not exist in state"
    );
  }

  #[test]
  fn hostile_attack_given_max_use_cases_reached_when_add_use_case_then_returns_error() {
    // Given: State at maximum capacity
    let mut state = PlannerState::new();
    const MAX_SIZE: usize = 10_000;

    // Create use cases up to MAX - 1
    for i in 0..MAX_SIZE {
      let uc = create_test_use_case(&format!("Use Case {}", i));
      state = state.add_use_case(uc).unwrap();
    }
    assert_eq!(
      state.use_cases.len(),
      MAX_SIZE,
      "State should be at maximum capacity"
    );

    // When: Try to add one more use case
    let extra_uc = create_test_use_case("Extra Use Case");
    let result = state.add_use_case(extra_uc);

    // Then: Returns error
    assert!(result.is_err(), "Adding beyond maximum should return error");
    assert_eq!(
      result.unwrap_err(),
      crate::planner::types::StateError::CollectionTooLarge,
      "Should return CollectionTooLarge error"
    );
  }

  #[test]
  fn hostile_attack_given_duplicate_id_when_add_use_case_then_returns_error() {
    // Given: State with existing use case
    let state = PlannerState::new();
    let mut use_case = create_test_use_case("Original");
    let specific_id = Uuid::new_v4();
    use_case.id = specific_id;
    let state = state.add_use_case(use_case).unwrap();

    // When: Try to add use case with duplicate ID
    let mut duplicate_uc = create_test_use_case("Duplicate");
    duplicate_uc.id = specific_id; // Same ID
    let result = state.add_use_case(duplicate_uc);

    // Then: Returns error
    assert!(result.is_err(), "Adding duplicate ID should return error");
    assert_eq!(
      result.unwrap_err(),
      crate::planner::types::StateError::DuplicateId("use_case".to_string()),
      "Should return DuplicateId error"
    );
  }

  #[test]
  fn hostile_attack_given_multiple_use_cases_when_remove_one_then_others_preserved() {
    // Given: State with multiple use cases
    let state = PlannerState::new();
    let uc1 = create_test_use_case("Keep 1");
    let uc2 = create_test_use_case("Remove");
    let uc3 = create_test_use_case("Keep 2");
    let state = state.add_use_case(uc1).unwrap();
    let state = state.add_use_case(uc2).unwrap();
    let state = state.add_use_case(uc3).unwrap();
    let remove_id = state
      .use_cases
      .iter()
      .find(|u| &u.title == "Remove")
      .map(|u| u.id)
      .unwrap();

    // When: Remove middle use case
    let updated_state = state.remove_use_case(remove_id);

    // Then: Others preserved
    assert_eq!(
      updated_state.use_cases.len(),
      2,
      "Should have two use cases remaining"
    );
    assert!(
      updated_state.use_cases.iter().any(|u| &u.title == "Keep 1"),
      "First use case should be preserved"
    );
    assert!(
      updated_state.use_cases.iter().any(|u| &u.title == "Keep 2"),
      "Third use case should be preserved"
    );
    assert!(
      !updated_state.use_cases.iter().any(|u| &u.title == "Remove"),
      "Removed use case should not exist"
    );
  }

  #[test]
  fn hostile_attack_given_empty_title_when_create_use_case_then_still_valid() {
    // Given: Empty title
    let empty_title = "";

    // When: Create use case with empty title
    let use_case = UseCase::new(
      empty_title.to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );

    // Then: Use case is created (validation is UI-level, not domain-level)
    assert_eq!(use_case.title, "", "Empty title is allowed at domain level");
    assert!(!use_case.id.is_nil(), "Should have valid ID");
  }

  #[test]
  fn hostile_attack_given_nonexistent_id_when_remove_use_case_then_state_unchanged() {
    // Given: Empty state
    let state = PlannerState::new();
    let fake_id = Uuid::new_v4();

    // When: Try to remove non-existent use case
    let updated_state = state.remove_use_case(fake_id);

    // Then: State unchanged
    assert_eq!(
      updated_state.use_cases.len(),
      0,
      "State should remain empty"
    );
    assert_eq!(updated_state, state, "State should be unchanged");
  }

  #[test]
  fn hostile_attack_given_nonexistent_id_when_update_use_case_then_state_unchanged() {
    // Given: Empty state
    let state = PlannerState::new();
    let fake_id = Uuid::new_v4();
    let mut uc = create_test_use_case("Fake Update");

    // When: Try to update non-existent use case
    uc.title = "Updated".to_string();
    let updated_state = state.update_use_case(fake_id, uc);

    // Then: State unchanged
    assert_eq!(
      updated_state.use_cases.len(),
      0,
      "State should remain empty"
    );
  }

  #[test]
  fn hostile_attack_given_use_case_when_cycle_priority_then_all_priorities_work() {
    // Given: Use case with Medium priority
    let state = PlannerState::new();
    let mut uc = create_test_use_case("Priority Cycle");
    uc = uc.with_priority(UseCasePriority::Medium);
    let state = state.add_use_case(uc).unwrap();
    let use_case_id = state.use_cases.get(0).map(|uc| uc.id).unwrap();

    // When: Cycle through all priorities
    let priorities = [
      UseCasePriority::Critical,
      UseCasePriority::High,
      UseCasePriority::Low,
      UseCasePriority::Medium,
    ];

    for priority in priorities {
      let mut uc = state
        .use_cases
        .iter()
        .find(|u| u.id == use_case_id)
        .map(|u| u.as_ref().clone())
        .unwrap();
      uc = uc.with_priority(priority);
      let updated_state = state.update_use_case(use_case_id, uc);

      // Then: Each priority update works
      let updated_uc = updated_state
        .use_cases
        .iter()
        .find(|u| u.id == use_case_id)
        .map(|u| u.as_ref());
      assert_eq!(
        updated_uc.map(|u| u.priority),
        Some(priority),
        "Priority should be updated to {:?}",
        priority
      );
    }
  }

  #[test]
  fn hostile_attack_given_empty_state_when_add_multiple_use_cases_then_all_added() {
    // Given: Empty state
    let state = PlannerState::new();

    // When: Add multiple use cases
    let uc1 = create_test_use_case("First");
    let uc2 = create_test_use_case("Second");
    let uc3 = create_test_use_case("Third");
    let state = state.add_use_case(uc1).unwrap();
    let state = state.add_use_case(uc2).unwrap();
    let state = state.add_use_case(uc3).unwrap();

    // Then: All use cases added
    assert_eq!(state.use_cases.len(), 3, "Should have three use cases");
    assert!(
      state.use_cases.iter().any(|u| &u.title == "First"),
      "First use case should exist"
    );
    assert!(
      state.use_cases.iter().any(|u| &u.title == "Second"),
      "Second use case should exist"
    );
    assert!(
      state.use_cases.iter().any(|u| &u.title == "Third"),
      "Third use case should exist"
    );
  }

  #[test]
  fn hostile_attack_given_immutable_state_when_add_use_case_then_original_unchanged() {
    // Given: Original state
    let original_state = PlannerState::new();
    assert_eq!(original_state.use_cases.len(), 0);

    // When: Add use case
    let uc = create_test_use_case("Immutable Test");
    let _new_state = original_state.add_use_case(uc).unwrap();

    // Then: Original state unchanged
    assert_eq!(
      original_state.use_cases.len(),
      0,
      "Original state should be unchanged"
    );
  }
}
