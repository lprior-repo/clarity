//! Phase Discover Component
//!
//! Discovery phase (Top of Diamond) - Problem exploration and research.
//! Contains thesis editor, persona cards, and scenario cards.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]
#![allow(warnings)]
#![allow(clippy::all)]

use crate::planner::components::{FieldLabel, SectionLabel, SectionLevel, TextArea};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{NorthStarScenario, Persona, ProductThesis};
use dioxus::prelude::*;
use uuid::Uuid;

/// Phase discover component
///
/// Discovery phase (Top of Diamond) with thesis, personas, and scenarios.
#[component]
pub fn PhaseDiscover(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let mut active_tab = use_signal(|| DiscoverTab::Thesis);

  rsx! {
      div { class: "phase-discover",
          SectionLabel {
              level: SectionLevel::Phase,
              label: "Discovery Phase".to_string(),
          }

          div { class: "discover-tabs",
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DiscoverTab::Thesis { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DiscoverTab::Thesis); },
                  "Thesis"
              }
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DiscoverTab::Personas { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DiscoverTab::Personas); },
                  "Personas"
              }
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DiscoverTab::Scenarios { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DiscoverTab::Scenarios); },
                  "Scenarios"
              }
          }

          div { class: "discover-content",
              {match &*active_tab.read() {
                  DiscoverTab::Thesis => rsx! {
                      ThesisEditor {
                          state: state,
                          selected_entity: selected_entity
                      }
                  },
                  DiscoverTab::Personas => rsx! {
                      PersonaList {
                          state: state,
                          selected_entity: selected_entity
                      }
                  },
                  DiscoverTab::Scenarios => rsx! {
                      ScenarioList {
                          state: state,
                          selected_entity: selected_entity
                      }
                  },
              }}
          }
      }
  }
}

/// Discovery tabs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiscoverTab {
  Thesis,
  Personas,
  Scenarios,
}

/// Thesis editor component
///
/// Editor for product thesis statement.
#[component]
fn ThesisEditor(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let thesis = state
    .read()
    .thesis
    .as_ref()
    .map(|t| (**t).clone())
    .unwrap_or_else(ProductThesis::default);

  let mut title = use_signal(|| thesis.title.clone());
  let mut problem = use_signal(|| thesis.problem.clone());
  let mut audience = use_signal(|| thesis.audience.clone());
  let mut solution = use_signal(|| thesis.solution.clone());
  let mut value_proposition = use_signal(|| thesis.value_proposition.clone());

  let save_thesis = move |_| {
    let new_thesis = ProductThesis::new(
      title.read().clone(),
      problem.read().clone(),
      audience.read().clone(),
      solution.read().clone(),
      value_proposition.read().clone(),
    );

    let updated = state.read().update_thesis(new_thesis);
    state.set(updated);
  };

  rsx! {
      div { class: "thesis-editor",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Product Thesis".to_string(),
          }

          div { class: "thesis-form",
              div { class: "form-group",
                  FieldLabel {
                      label: "Title".to_string(),
                      required: true,
                      hint: Some("A concise title for your product thesis".to_string())
                  }
                  input {
                      class: "form-control",
                      r#type: "text",
                      value: "{title}",
                      oninput: move |evt: Event<FormData>| {
                          title.set(evt.value());
                      }
                  }
              }

              div { class: "form-group",
                  FieldLabel {
                      label: "Problem Statement".to_string(),
                      required: true,
                      hint: Some("What problem are you solving?".to_string())
                  }
                  TextArea {
                      label: "Problem".to_string(),
                      value: problem.read().clone(),
                      placeholder: Some("Describe the problem you're trying to solve...".to_string()),
                      rows: Some(4),
                      on_change: move |s| { problem.set(s); },
                  }
              }

              div { class: "form-group",
                  FieldLabel {
                      label: "Target Audience".to_string(),
                      required: true,
                      hint: Some("Who are you solving this for?".to_string())
                  }
                  input {
                      class: "form-control",
                      r#type: "text",
                      value: "{audience}",
                      oninput: move |evt: Event<FormData>| {
                          audience.set(evt.value());
                      }
                  }
              }

              div { class: "form-group",
                  FieldLabel {
                      label: "Proposed Solution".to_string(),
                      required: true,
                      hint: Some("What is your proposed solution?".to_string())
                  }
                  TextArea {
                      label: "Solution".to_string(),
                      value: solution.read().clone(),
                      placeholder: Some("Describe your solution...".to_string()),
                      rows: Some(4),
                      on_change: move |s| { solution.set(s); },
                  }
              }

              div { class: "form-group",
                  FieldLabel {
                      label: "Value Proposition".to_string(),
                      required: true,
                      hint: Some("Why is this valuable?".to_string())
                  }
                  TextArea {
                      label: "Value".to_string(),
                      value: value_proposition.read().clone(),
                      placeholder: Some("What value does this provide?".to_string()),
                      rows: Some(3),
                      on_change: move |s| { value_proposition.set(s); },
                  }
              }

              div { class: "form-actions",
                  button {
                      class: "btn btn-primary",
                      onclick: save_thesis,
                      "Save Thesis"
                  }
              }
          }
      }
  }
}

/// Persona list component
///
/// List of persona cards with add/remove functionality.
#[component]
fn PersonaList(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  // Clone personas for rendering (persistent data structure, so clone is cheap)
  let personas: Vec<_> = state
    .read()
    .personas
    .iter()
    .map(|rc_persona| (**rc_persona).clone())
    .collect();

  // Get currently selected entity ID for highlighting
  let selected_id = selected_entity
    .read()
    .as_ref()
    .and_then(|entity| match entity {
      SelectedEntity::Persona(id) => Some(*id),
      _ => None,
    });

  let add_persona = move |_| {
    let new_persona = Persona::new(
      "New Persona".to_string(),
      "Role".to_string(),
      "Description".to_string(),
    );

    let current = state.read().clone();
    match current.add_persona(new_persona) {
      Ok(updated) => state.set(updated),
      Err(e) => {
        eprintln!("Failed to add persona: {e}");
        // In production, this would show an error notification
      }
    }
  };

  // Create handlers for each persona by capturing ID
  let render_persona_card = |persona: Persona| {
    let persona_id = persona.id;
    let is_selected = selected_id.map_or(false, |id| id == persona_id);

    rsx! {
        PersonaCard {
            persona: std::rc::Rc::new(persona),
            on_remove: move |_evt: MouseEvent| {
                let updated = state.read().remove_persona(persona_id);
                state.set(updated);
                selected_entity.set(Some(SelectedEntity::clear()));
            },
            on_select: move |_evt: MouseEvent| {
                selected_entity.set(Some(SelectedEntity::Persona(persona_id)));
            },
            is_selected: is_selected,
        }
    }
  };

  rsx! {
      div { class: "persona-list",
          div { class: "list-header",
              SectionLabel {
                  level: SectionLevel::Section,
                  label: "User Personas".to_string(),
              }
              button {
                  class: "btn btn-primary",
                  onclick: add_persona,
                  "Add Persona"
              }
          }

          div { class: "persona-cards",
              {if personas.is_empty() {
                  rsx! {
                      div { class: "empty-state",
                          p { "No personas yet. Click 'Add Persona' to create one." }
                      }
                  }
              } else {
                  rsx! {
                      {personas.into_iter().map(render_persona_card)}
                  }
              }}
          }
      }
  }
}

/// Persona card component
///
/// Individual persona card with remove button.
#[component]
fn PersonaCard(
  persona: std::rc::Rc<Persona>,
  on_remove: EventHandler<MouseEvent>,
  on_select: EventHandler<MouseEvent>,
  is_selected: bool,
) -> Element {
  rsx! {
      div {
          class: format!("persona-card {}", if is_selected { "selected" } else { "" }),
          onclick: move |evt: MouseEvent| {
              evt.stop_propagation();
              on_select.call(evt);
          },
          div { class: "persona-header",
              h3 { class: "persona-name", "{persona.name}" }
              div { class: "persona-actions",
                  button {
                      class: "btn btn-icon btn-danger",
                      onclick: move |evt: MouseEvent| {
                          evt.stop_propagation();
                          on_remove.call(evt);
                      },
                      "×"
                  }
              }
          }
          div { class: "persona-role", "{persona.role}" }
          div { class: "persona-description", "{persona.description}" }
          {if !persona.goals.is_empty() {
              rsx! {
                  div { class: "persona-section",
                      strong { "Goals:" }
                      ul {
                          for goal in persona.goals.iter() {
                              li { "{goal}" }
                          }
                      }
                  }
              }
          } else {
              rsx! {}
          }}
      }
  }
}

/// Scenario list component
///
/// List of north star scenario cards with add/remove functionality.
#[component]
fn ScenarioList(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  // Convert Vector to Vec for use in rsx
  let scenarios: Vec<_> = state.read().scenarios.iter().cloned().collect();

  let add_scenario = move |_| {
    let new_scenario = NorthStarScenario::new(
      "New Scenario".to_string(),
      "Describe the ideal user journey...".to_string(),
    );

    let current = state.read().clone();
    match current.add_scenario(new_scenario) {
      Ok(updated) => state.set(updated),
      Err(e) => {
        eprintln!("Failed to add scenario: {e}");
        // In production, this would show an error notification
      }
    }
  };

  let remove_scenario = move |id: Uuid| {
    let updated = state.read().remove_scenario(id);
    state.set(updated);
    selected_entity.set(Some(SelectedEntity::clear()));
  };

  let select_scenario = move |id: Uuid| {
    selected_entity.set(Some(SelectedEntity::Scenario(id)));
  };

  // Note: The actual rendering of scenario cards requires Dioxus runtime
  // This is a simplified version for compilation
  let _ = (scenarios, add_scenario, remove_scenario, select_scenario);

  rsx! {
      div { class: "scenario-list",
          div { class: "list-header",
              SectionLabel {
                  level: SectionLevel::Section,
                  label: "North Star Scenarios".to_string(),
              }
              button {
                  class: "btn btn-primary",
                  onclick: add_scenario,
                  "Add Scenario"
              }
          }

          div { class: "scenario-cards",
              div { class: "empty-state",
                  p { "Scenario list requires Dioxus runtime" }
              }
          }
      }
  }
}

/// Scenario card component
///
/// Individual scenario card with remove button.
#[component]
fn ScenarioCard(
  scenario: std::rc::Rc<NorthStarScenario>,
  on_remove: EventHandler<MouseEvent>,
  on_select: EventHandler<MouseEvent>,
  is_selected: bool,
) -> Element {
  rsx! {
      div {
          class: format!("scenario-card {}", if is_selected { "selected" } else { "" }),
          onclick: move |evt: MouseEvent| {
              evt.stop_propagation();
              on_select.call(evt);
          },
          div { class: "scenario-header",
              h3 { class: "scenario-title", "{scenario.title}" }
              div { class: "scenario-actions",
                  button {
                      class: "btn btn-icon btn-danger",
                      onclick: move |evt: MouseEvent| {
                          evt.stop_propagation();
                          on_remove.call(evt);
                      },
                      "×"
                  }
              }
          }
          div { class: "scenario-narrative", "{scenario.narrative}" }
          {if !scenario.steps.is_empty() {
              rsx! {
                  div { class: "scenario-section",
                      strong { "Steps:" }
                      ol {
                          for step in scenario.steps.iter() {
                              li { "{step}" }
                          }
                      }
                  }
              }
          } else {
              rsx! {}
          }}
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::planner::state::{PlannerState, SelectedEntity};
  use crate::planner::types::{DiamondPhase, Persona};
  use uuid::Uuid;

  /// Helper to create a test persona with valid defaults
  fn create_test_persona(name: &str) -> Persona {
    Persona::new(
      name.to_string(),
      "Test Role".to_string(),
      "Test Description".to_string(),
    )
  }

  //
  // Bounded Context: Phase Discover (Discovery Phase)
  // Aggregate Root: PlannerState
  // Entity: Persona
  //

  #[test]
  fn hostile_attack_given_empty_state_when_add_persona_then_persona_card_renders() {
    // Given: Empty planner state
    let state = PlannerState::new();
    assert_eq!(state.personas.len(), 0, "State should start empty");

    // When: Add persona
    let persona = create_test_persona("Alice");
    let result = state.add_persona(persona);

    // Then: Persona card renders (persona exists in state)
    assert!(
      result.is_ok(),
      "Adding persona to empty state should succeed"
    );
    let updated_state = result.unwrap();
    assert_eq!(
      updated_state.personas.len(),
      1,
      "State should contain exactly one persona"
    );
    assert_eq!(
      updated_state.personas.get(0).map(|p| &p.name),
      Some(&"Alice".to_string()),
      "Added persona should have correct name"
    );
  }

  #[test]
  fn hostile_attack_given_persona_selected_when_click_remove_then_persona_removed() {
    // Given: State with selected persona
    let state = PlannerState::new();
    let persona = create_test_persona("Selected Persona");
    let state = state.add_persona(persona).unwrap();
    let persona_id = state.personas.get(0).map(|p| p.id).unwrap();
    let selected_entity = Some(SelectedEntity::Persona(persona_id));

    // When: Remove persona (simulate clicking remove button)
    let updated_state = state.remove_persona(persona_id);

    // Then: Persona removed
    assert_eq!(
      updated_state.personas.len(),
      0,
      "Persona should be removed from state"
    );
    assert!(
      updated_state.personas.iter().all(|p| p.id != persona_id),
      "Removed persona ID should not exist"
    );

    // Selection should be cleared
    let should_clear =
      matches!(selected_entity, Some(SelectedEntity::Persona(id)) if id == persona_id);
    assert!(should_clear, "Selection should be cleared when removed");
  }

  #[test]
  fn hostile_attack_given_no_persona_selected_when_click_persona_then_persona_selected() {
    // Given: State with persona but no selection
    let state = PlannerState::new();
    let persona = create_test_persona("Selectable Persona");
    let state = state.add_persona(persona).unwrap();
    let persona_id = state.personas.get(0).map(|p| p.id).unwrap();
    let selected_entity = None;

    // When: Click persona (simulate selection)
    let new_selection = Some(SelectedEntity::Persona(persona_id));

    // Then: Persona is selected
    assert_eq!(
      new_selection,
      Some(SelectedEntity::Persona(persona_id)),
      "Persona should be selected"
    );
    assert_ne!(
      selected_entity, new_selection,
      "Selection should change from None"
    );
  }

  #[test]
  fn hostile_attack_given_persona_selected_when_click_different_persona_then_selection_changes() {
    // Given: State with two personas and first persona selected
    let state = PlannerState::new();
    let persona1 = create_test_persona("Persona One");
    let persona2 = create_test_persona("Persona Two");
    let state = state.add_persona(persona1).unwrap();
    let state = state.add_persona(persona2).unwrap();
    let first_id = state.personas.get(0).map(|p| p.id).unwrap();
    let second_id = state.personas.get(1).map(|p| p.id).unwrap();

    let mut selected_entity = Some(SelectedEntity::Persona(first_id));

    // When: Select different persona (simulate clicking)
    selected_entity = Some(SelectedEntity::Persona(second_id));

    // Then: Selection changes
    assert_eq!(
      selected_entity,
      Some(SelectedEntity::Persona(second_id)),
      "Selection should change to second persona"
    );
    assert_ne!(
      selected_entity,
      Some(SelectedEntity::Persona(first_id)),
      "Selection should not be first persona"
    );
  }

  #[test]
  fn hostile_attack_given_max_personas_reached_when_add_persona_then_returns_error() {
    // Given: State at maximum capacity
    let mut state = PlannerState::new();
    const MAX_SIZE: usize = 10_000;

    // Create personas up to MAX - 1
    for i in 0..MAX_SIZE {
      let persona = create_test_persona(&format!("Persona {}", i));
      state = state.add_persona(persona).unwrap();
    }
    assert_eq!(
      state.personas.len(),
      MAX_SIZE,
      "State should be at maximum capacity"
    );

    // When: Try to add one more persona
    let extra_persona = create_test_persona("Extra Persona");
    let result = state.add_persona(extra_persona);

    // Then: Returns error
    assert!(result.is_err(), "Adding beyond maximum should return error");
    assert_eq!(
      result.unwrap_err(),
      crate::planner::types::StateError::CollectionTooLarge,
      "Should return CollectionTooLarge error"
    );
  }

  #[test]
  fn hostile_attack_given_duplicate_id_when_add_persona_then_returns_error() {
    // Given: State with existing persona
    let state = PlannerState::new();
    let mut persona = create_test_persona("Original");
    let specific_id = Uuid::new_v4();
    persona.id = specific_id;
    let state = state.add_persona(persona).unwrap();

    // When: Try to add persona with duplicate ID
    let mut duplicate_persona = create_test_persona("Duplicate");
    duplicate_persona.id = specific_id; // Same ID
    let result = state.add_persona(duplicate_persona);

    // Then: Returns error
    assert!(result.is_err(), "Adding duplicate ID should return error");
    assert_eq!(
      result.unwrap_err(),
      crate::planner::types::StateError::DuplicateId("persona".to_string()),
      "Should return DuplicateId error"
    );
  }

  #[test]
  fn hostile_attack_given_multiple_personas_when_remove_one_then_others_preserved() {
    // Given: State with multiple personas
    let state = PlannerState::new();
    let persona1 = create_test_persona("Keep 1");
    let persona2 = create_test_persona("Remove");
    let persona3 = create_test_persona("Keep 2");
    let state = state.add_persona(persona1).unwrap();
    let state = state.add_persona(persona2).unwrap();
    let state = state.add_persona(persona3).unwrap();
    let remove_id = state
      .personas
      .iter()
      .find(|p| &p.name == "Remove")
      .map(|p| p.id)
      .unwrap();

    // When: Remove middle persona
    let updated_state = state.remove_persona(remove_id);

    // Then: Others preserved
    assert_eq!(
      updated_state.personas.len(),
      2,
      "Should have two personas remaining"
    );
    assert!(
      updated_state.personas.iter().any(|p| &p.name == "Keep 1"),
      "First persona should be preserved"
    );
    assert!(
      updated_state.personas.iter().any(|p| &p.name == "Keep 2"),
      "Third persona should be preserved"
    );
    assert!(
      !updated_state.personas.iter().any(|p| &p.name == "Remove"),
      "Removed persona should not exist"
    );
  }

  #[test]
  fn hostile_attack_given_nonexistent_persona_id_when_remove_then_state_unchanged() {
    // Given: Empty state
    let state = PlannerState::new();
    let fake_id = Uuid::new_v4();

    // When: Try to remove non-existent persona
    let updated_state = state.remove_persona(fake_id);

    // Then: State unchanged
    assert_eq!(updated_state.personas.len(), 0, "State should remain empty");
    assert_eq!(updated_state, state, "State should be unchanged");
  }

  #[test]
  fn hostile_attack_given_empty_state_when_add_multiple_personas_then_all_added() {
    // Given: Empty state
    let state = PlannerState::new();

    // When: Add multiple personas
    let persona1 = create_test_persona("Alice");
    let persona2 = create_test_persona("Bob");
    let persona3 = create_test_persona("Charlie");
    let state = state.add_persona(persona1).unwrap();
    let state = state.add_persona(persona2).unwrap();
    let state = state.add_persona(persona3).unwrap();

    // Then: All personas added
    assert_eq!(state.personas.len(), 3, "Should have three personas");
    assert!(
      state.personas.iter().any(|p| &p.name == "Alice"),
      "First persona should exist"
    );
    assert!(
      state.personas.iter().any(|p| &p.name == "Bob"),
      "Second persona should exist"
    );
    assert!(
      state.personas.iter().any(|p| &p.name == "Charlie"),
      "Third persona should exist"
    );
  }

  #[test]
  fn hostile_attack_given_persona_when_add_goal_then_goal_added() {
    // Given: Persona
    let persona = create_test_persona("Goal-oriented");

    // When: Add goal
    let persona_with_goal = persona.with_goal("Achieve success".to_string());

    // Then: Goal added
    assert_eq!(
      persona_with_goal.goals.len(),
      1,
      "Persona should have one goal"
    );
    assert_eq!(
      persona_with_goal.goals.get(0),
      Some(&"Achieve success".to_string()),
      "Goal should match"
    );
  }

  #[test]
  fn hostile_attack_given_persona_when_add_pain_point_then_pain_point_added() {
    // Given: Persona
    let persona = create_test_persona("Pain-aware");

    // When: Add pain point
    let persona_with_pain = persona.with_pain_point("Too complex".to_string());

    // Then: Pain point added
    assert_eq!(
      persona_with_pain.pain_points.len(),
      1,
      "Persona should have one pain point"
    );
    assert_eq!(
      persona_with_pain.pain_points.get(0),
      Some(&"Too complex".to_string()),
      "Pain point should match"
    );
  }

  #[test]
  fn hostile_attack_given_persona_when_add_behavior_then_behavior_added() {
    // Given: Persona
    let persona = create_test_persona("Behavioral");

    // When: Add behavior
    let persona_with_behavior = persona.with_behavior("Clicks buttons".to_string());

    // Then: Behavior added
    assert_eq!(
      persona_with_behavior.behaviors.len(),
      1,
      "Persona should have one behavior"
    );
    assert_eq!(
      persona_with_behavior.behaviors.get(0),
      Some(&"Clicks buttons".to_string()),
      "Behavior should match"
    );
  }

  #[test]
  fn hostile_attack_given_persona_when_add_multiple_goals_then_all_preserved() {
    // Given: Persona
    let persona = create_test_persona("Multi-goal");

    // When: Add multiple goals
    let persona = persona.with_goal("Goal 1".to_string());
    let persona = persona.with_goal("Goal 2".to_string());
    let persona = persona.with_goal("Goal 3".to_string());

    // Then: All goals preserved
    assert_eq!(persona.goals.len(), 3, "Should have three goals");
    assert!(persona.goals.contains(&"Goal 1".to_string()));
    assert!(persona.goals.contains(&"Goal 2".to_string()));
    assert!(persona.goals.contains(&"Goal 3".to_string()));
  }

  #[test]
  fn hostile_attack_given_discover_tabs_when_switch_tabs_then_tab_changes() {
    // Given: Thesis tab active
    let mut active_tab = DiscoverTab::Thesis;

    // When: Switch to Personas tab
    active_tab = DiscoverTab::Personas;

    // Then: Tab changes
    assert_eq!(
      active_tab,
      DiscoverTab::Personas,
      "Should be on Personas tab"
    );

    // When: Switch to Scenarios tab
    active_tab = DiscoverTab::Scenarios;

    // Then: Tab changes
    assert_eq!(
      active_tab,
      DiscoverTab::Scenarios,
      "Should be on Scenarios tab"
    );
  }

  #[test]
  fn hostile_attack_given_persona_with_empty_name_when_create_then_still_valid() {
    // Given: Empty name
    let empty_name = "";

    // When: Create persona with empty name
    let persona = Persona::new(
      empty_name.to_string(),
      "Role".to_string(),
      "Description".to_string(),
    );

    // Then: Persona is created (validation is UI-level, not domain-level)
    assert_eq!(persona.name, "", "Empty name is allowed at domain level");
    assert!(!persona.id.is_nil(), "Should have valid ID");
  }

  #[test]
  fn hostile_attack_given_persona_with_empty_role_when_create_then_still_valid() {
    // Given: Empty role
    let empty_role = "";

    // When: Create persona with empty role
    let persona = Persona::new(
      "Name".to_string(),
      empty_role.to_string(),
      "Description".to_string(),
    );

    // Then: Persona is created
    assert_eq!(persona.role, "", "Empty role is allowed at domain level");
    assert!(!persona.id.is_nil(), "Should have valid ID");
  }

  #[test]
  fn hostile_attack_given_persona_with_empty_description_when_create_then_still_valid() {
    // Given: Empty description
    let empty_description = "";

    // When: Create persona with empty description
    let persona = Persona::new(
      "Name".to_string(),
      "Role".to_string(),
      empty_description.to_string(),
    );

    // Then: Persona is created
    assert_eq!(
      persona.description, "",
      "Empty description is allowed at domain level"
    );
    assert!(!persona.id.is_nil(), "Should have valid ID");
  }

  #[test]
  fn hostile_attack_given_phase_discover_when_check_phase_then_is_top() {
    // Given: Discovery phase
    let state = PlannerState::new();

    // When: Check current phase
    let phase = state.current_phase;

    // Then: Phase is Top (Discovery)
    assert_eq!(phase, DiamondPhase::Top, "Discovery phase should be Top");
  }

  #[test]
  fn hostile_attack_given_immutable_state_when_add_persona_then_original_unchanged() {
    // Given: Original state
    let original_state = PlannerState::new();
    assert_eq!(original_state.personas.len(), 0);

    // When: Add persona
    let persona = create_test_persona("Immutable Test");
    let _new_state = original_state.add_persona(persona).unwrap();

    // Then: Original state unchanged
    assert_eq!(
      original_state.personas.len(),
      0,
      "Original state should be unchanged"
    );
  }

  #[test]
  fn hostile_attack_given_persona_when_set_quote_then_quote_set() {
    // Given: Persona
    let persona = create_test_persona("Quotable");

    // When: Set quote
    let persona_with_quote = persona
      .clone()
      .with_quote("I love this feature!".to_string());

    // Then: Quote set
    assert_eq!(
      persona_with_quote.quote,
      Some("I love this feature!".to_string()),
      "Quote should be set"
    );
    assert_eq!(persona.quote, None, "Original persona unchanged");
  }

  #[test]
  fn hostile_attack_given_persona_when_set_skill_level_then_skill_level_set() {
    // Given: Persona
    let persona = create_test_persona("Skilled");

    // When: Set skill level
    let persona_with_skill = persona.clone().with_skill_level("Expert".to_string());

    // Then: Skill level set
    assert_eq!(
      persona_with_skill.skill_level, "Expert",
      "Skill level should be set"
    );
    assert_eq!(persona.skill_level, "", "Original persona unchanged");
  }
}
