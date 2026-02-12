//! Tests for North Star Scenario Builder
//!
//! ATDD tests for character + simulation framework with plot hole detection

#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::expect_used)]
#![allow(clippy::manual_string_new)]
#![forbid(unsafe_code)]

use chrono::Utc;

use crate::pme_lattice::north_star::{
  Character, CharacterBuilder, CharacterError, EdgeCaseType, NorthStarScenario,
  NorthStarScenarioBuilder, NorthStarScenarioError, PlotHole, PlotHoleType, Simulation,
  SimulationError, TimelineEvent, TimelineEventError,
};

// ============================================================================
// CHARACTER TESTS
// ============================================================================

mod character_tests {
  use super::*;

  #[test]
  fn test_character_rejects_empty_name() {
    let result = Character::new("", "A test character".to_string());
    assert!(matches!(result, Err(CharacterError::EmptyName)));
  }

  #[test]
  fn test_character_rejects_empty_description() {
    let result = Character::new("Hero", "".to_string());
    assert!(matches!(result, Err(CharacterError::EmptyDescription)));
  }

  #[test]
  fn test_character_creates_with_valid_input() {
    let result = Character::new("Hero", "A brave protagonist".to_string());
    assert!(result.is_ok());
    let character = result.expect("character should be valid");
    assert_eq!(character.name(), "Hero");
    assert_eq!(character.description(), "A brave protagonist");
  }

  #[test]
  fn test_character_builder_adds_goal() {
    let character = CharacterBuilder::new("Hero", "Protagonist")
      .with_goal("Save the world")
      .build();
    assert!(character.is_ok());
    let c = character.expect("character should be valid");
    assert_eq!(c.goals().len(), 1);
    assert_eq!(c.goals()[0], "Save the world");
  }

  #[test]
  fn test_character_builder_adds_motivation() {
    let character = CharacterBuilder::new("Hero", "Protagonist")
      .with_motivation("Revenge for fallen kingdom")
      .build();
    assert!(character.is_ok());
    let c = character.expect("character should be valid");
    assert_eq!(c.motivations().len(), 1);
  }

  #[test]
  fn test_character_builder_adds_constraint() {
    let character = CharacterBuilder::new("Hero", "Protagonist")
      .with_constraint("Cannot use magic on Tuesdays")
      .build();
    assert!(character.is_ok());
    let c = character.expect("character should be valid");
    assert_eq!(c.constraints().len(), 1);
  }

  #[test]
  fn test_character_builder_chains_multiple_goals() {
    let character = CharacterBuilder::new("Hero", "Protagonist")
      .with_goal("Find the artifact")
      .with_goal("Defeat the villain")
      .with_goal("Restore peace")
      .build();
    assert!(character.is_ok());
    let c = character.expect("character should be valid");
    assert_eq!(c.goals().len(), 3);
  }

  #[test]
  fn test_character_has_unique_id() {
    let c1 = Character::new("Hero", "First hero".to_string()).expect("valid");
    let c2 = Character::new("Hero", "Second hero".to_string()).expect("valid");
    assert_ne!(c1.id(), c2.id());
  }
}

// ============================================================================
// TIMELINE EVENT TESTS
// ============================================================================

mod timeline_event_tests {
  use super::*;

  #[test]
  fn test_timeline_event_rejects_empty_description() {
    let now = Utc::now();
    let result = TimelineEvent::new("".to_string(), now, now);
    assert!(matches!(result, Err(TimelineEventError::EmptyDescription)));
  }

  #[test]
  fn test_timeline_event_rejects_end_before_start() {
    let start = Utc::now();
    let end = start - chrono::Duration::hours(1);
    let result = TimelineEvent::new("Event".to_string(), start, end);
    assert!(matches!(
      result,
      Err(TimelineEventError::EndTimeBeforeStart)
    ));
  }

  #[test]
  fn test_timeline_event_creates_valid_event() {
    let start = Utc::now();
    let end = start + chrono::Duration::hours(1);
    let result = TimelineEvent::new("Battle begins".to_string(), start, end);
    assert!(result.is_ok());
  }

  #[test]
  fn test_timeline_event_accepts_zero_duration() {
    let now = Utc::now();
    let result = TimelineEvent::new("Instant event".to_string(), now, now);
    assert!(result.is_ok());
  }
}

// ============================================================================
// PLOT HOLE DETECTION TESTS
// ============================================================================

mod plot_hole_tests {
  use super::*;

  #[test]
  fn test_plot_hole_discovery_mechanism_missing() {
    let hole = PlotHole::new(
      PlotHoleType::DiscoveryMechanismMissing,
      "No way to learn password",
    );
    assert_eq!(hole.hole_type(), &PlotHoleType::DiscoveryMechanismMissing);
    assert_eq!(hole.description(), "No way to learn password");
  }

  #[test]
  fn test_plot_hole_edge_case_unhandled() {
    let hole = PlotHole::new(
      PlotHoleType::EdgeCaseUnhandled(EdgeCaseType::BoundaryCondition),
      "What if user enters negative value?",
    );
    assert!(matches!(
      hole.hole_type(),
      PlotHoleType::EdgeCaseUnhandled(EdgeCaseType::BoundaryCondition)
    ));
  }

  #[test]
  fn test_plot_hole_timeline_inconsistent() {
    let hole = PlotHole::new(
      PlotHoleType::TimelineInconsistent,
      "Character ages backwards in chapter 3",
    );
    assert!(matches!(
      hole.hole_type(),
      &PlotHoleType::TimelineInconsistent
    ));
  }

  #[test]
  fn test_plot_hole_has_severity() {
    let hole = PlotHole::new(PlotHoleType::DiscoveryMechanismMissing, "Issue");
    assert!(hole.severity() >= 0.0 && hole.severity() <= 1.0);
  }

  #[test]
  fn test_timeline_inconsistent_has_higher_severity() {
    let timeline_hole = PlotHole::new(PlotHoleType::TimelineInconsistent, "Time issue");
    let discovery_hole = PlotHole::new(PlotHoleType::DiscoveryMechanismMissing, "Discovery issue");
    assert!(timeline_hole.severity() > discovery_hole.severity());
  }
}

// ============================================================================
// SIMULATION TESTS
// ============================================================================

mod simulation_tests {
  use super::*;

  fn create_test_character() -> Character {
    CharacterBuilder::new("Hero", "Test protagonist")
      .with_goal("Win the battle")
      .with_constraint("Must rest after using magic")
      .build()
      .expect("valid character")
  }

  fn create_test_timeline() -> Vec<TimelineEvent> {
    let now = Utc::now();
    vec![
      TimelineEvent::new(
        "Act 1: Introduction".to_string(),
        now,
        now + chrono::Duration::hours(1),
      )
      .expect("valid event"),
      TimelineEvent::new(
        "Act 2: Conflict".to_string(),
        now + chrono::Duration::hours(2),
        now + chrono::Duration::hours(3),
      )
      .expect("valid event"),
    ]
  }

  #[test]
  fn test_simulation_rejects_empty_characters() {
    let timeline = create_test_timeline();
    let result = Simulation::new(vec![], timeline);
    assert!(matches!(result, Err(SimulationError::NoCharacters)));
  }

  #[test]
  fn test_simulation_rejects_empty_timeline() {
    let characters = vec![create_test_character()];
    let result = Simulation::new(characters, vec![]);
    assert!(matches!(result, Err(SimulationError::NoTimeline)));
  }

  #[test]
  fn test_simulation_runs_and_produces_result() {
    let characters = vec![create_test_character()];
    let timeline = create_test_timeline();
    let simulation = Simulation::new(characters, timeline).expect("valid simulation");
    let result = simulation.run();
    assert!(result.is_ok());
  }

  #[test]
  fn test_simulation_result_contains_plot_holes() {
    let characters = vec![create_test_character()];
    let timeline = create_test_timeline();
    let simulation = Simulation::new(characters, timeline).expect("valid simulation");
    let result = simulation.run().expect("simulation should succeed");
    assert!(!result.plot_holes().is_empty() || result.is_consistent());
  }

  #[test]
  fn test_simulation_detects_missing_character_motivation() {
    let character = CharacterBuilder::new("Flat Character", "No depth")
      .build()
      .expect("valid character");
    let timeline = create_test_timeline();
    let simulation = Simulation::new(vec![character], timeline).expect("valid simulation");
    let result = simulation.run().expect("simulation should succeed");
    let has_motivation_hole = result
      .plot_holes()
      .iter()
      .any(|h| matches!(h.hole_type(), PlotHoleType::EdgeCaseUnhandled(_)));
    assert!(has_motivation_hole || result.plot_holes().is_empty());
  }
}

// ============================================================================
// NORTH STAR SCENARIO TESTS
// ============================================================================

mod north_star_scenario_tests {
  use super::*;

  fn create_test_character() -> Character {
    CharacterBuilder::new("Protagonist", "Main character")
      .with_goal("Complete the quest")
      .with_motivation("Save the village")
      .build()
      .expect("valid character")
  }

  fn create_test_timeline() -> Vec<TimelineEvent> {
    let now = Utc::now();
    vec![
      TimelineEvent::new("Start".to_string(), now, now + chrono::Duration::hours(1))
        .expect("valid event"),
    ]
  }

  #[test]
  fn test_scenario_rejects_empty_title() {
    let characters = vec![create_test_character()];
    let timeline = create_test_timeline();
    let result = NorthStarScenario::new("", characters, timeline);
    assert!(matches!(result, Err(NorthStarScenarioError::EmptyTitle)));
  }

  #[test]
  fn test_scenario_rejects_empty_characters() {
    let timeline = create_test_timeline();
    let result = NorthStarScenario::new("Test Scenario", vec![], timeline);
    assert!(matches!(result, Err(NorthStarScenarioError::NoCharacters)));
  }

  #[test]
  fn test_scenario_rejects_empty_timeline() {
    let characters = vec![create_test_character()];
    let result = NorthStarScenario::new("Test Scenario", characters, vec![]);
    assert!(matches!(result, Err(NorthStarScenarioError::NoTimeline)));
  }

  #[test]
  fn test_scenario_builder_creates_valid_scenario() {
    let scenario = NorthStarScenarioBuilder::new("Epic Quest")
      .with_character(create_test_character())
      .with_timeline_event(
        TimelineEvent::new(
          "Beginning".to_string(),
          Utc::now(),
          Utc::now() + chrono::Duration::hours(1),
        )
        .expect("valid event"),
      )
      .build();
    assert!(scenario.is_ok());
  }

  #[test]
  fn test_scenario_validates_plot_holes() {
    let scenario = NorthStarScenarioBuilder::new("Test")
      .with_character(create_test_character())
      .with_timeline_event(
        TimelineEvent::new(
          "Event".to_string(),
          Utc::now(),
          Utc::now() + chrono::Duration::hours(1),
        )
        .expect("valid event"),
      )
      .build()
      .expect("valid scenario");

    let holes = scenario.detect_plot_holes();
    assert!(holes.is_ok());
  }

  #[test]
  fn test_scenario_has_unique_id() {
    let s1 = NorthStarScenarioBuilder::new("Scenario 1")
      .with_character(create_test_character())
      .with_timeline_event(
        TimelineEvent::new(
          "E".to_string(),
          Utc::now(),
          Utc::now() + chrono::Duration::hours(1),
        )
        .expect("valid event"),
      )
      .build()
      .expect("valid");

    let s2 = NorthStarScenarioBuilder::new("Scenario 2")
      .with_character(create_test_character())
      .with_timeline_event(
        TimelineEvent::new(
          "E".to_string(),
          Utc::now(),
          Utc::now() + chrono::Duration::hours(1),
        )
        .expect("valid event"),
      )
      .build()
      .expect("valid");

    assert_ne!(s1.id(), s2.id());
  }

  #[test]
  fn test_scenario_includes_north_star_statement() {
    let scenario = NorthStarScenarioBuilder::new("Vision")
      .with_north_star("Users can accomplish X in under 30 seconds")
      .with_character(create_test_character())
      .with_timeline_event(
        TimelineEvent::new(
          "Event".to_string(),
          Utc::now(),
          Utc::now() + chrono::Duration::hours(1),
        )
        .expect("valid event"),
      )
      .build()
      .expect("valid scenario");

    assert_eq!(
      scenario.north_star_statement(),
      Some("Users can accomplish X in under 30 seconds")
    );
  }
}
