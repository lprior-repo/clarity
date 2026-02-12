//! North Star Scenario Builder - Discover Phase Framework
//!
//! Character + Simulation framework with plot hole detection for the PME Discover phase.
//! Detects: Discovery mechanism missing, Edge case unhandled, Timeline inconsistent.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_collect)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// CHARACTER ERROR
// ============================================================================

/// Errors for character creation
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CharacterError {
  #[error("character name cannot be empty")]
  EmptyName,

  #[error("character description cannot be empty")]
  EmptyDescription,
}

// ============================================================================
// CHARACTER
// ============================================================================

/// A character in a North Star scenario with goals, motivations, and constraints
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
  id: Uuid,
  name: String,
  description: String,
  goals: Vec<String>,
  motivations: Vec<String>,
  constraints: Vec<String>,
  created_at: DateTime<Utc>,
}

impl Character {
  pub fn new(name: &str, description: String) -> Result<Self, CharacterError> {
    if name.trim().is_empty() {
      return Err(CharacterError::EmptyName);
    }
    if description.trim().is_empty() {
      return Err(CharacterError::EmptyDescription);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description,
      goals: Vec::new(),
      motivations: Vec::new(),
      constraints: Vec::new(),
      created_at: Utc::now(),
    })
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  #[must_use]
  pub fn description(&self) -> &str {
    &self.description
  }

  #[must_use]
  pub fn goals(&self) -> &[String] {
    &self.goals
  }

  #[must_use]
  pub fn motivations(&self) -> &[String] {
    &self.motivations
  }

  #[must_use]
  pub fn constraints(&self) -> &[String] {
    &self.constraints
  }

  fn add_goal(&mut self, goal: String) {
    if !goal.trim().is_empty() && !self.goals.contains(&goal) {
      self.goals.push(goal);
    }
  }

  fn add_motivation(&mut self, motivation: String) {
    if !motivation.trim().is_empty() && !self.motivations.contains(&motivation) {
      self.motivations.push(motivation);
    }
  }

  fn add_constraint(&mut self, constraint: String) {
    if !constraint.trim().is_empty() && !self.constraints.contains(&constraint) {
      self.constraints.push(constraint);
    }
  }

  #[must_use]
  pub fn has_motivations(&self) -> bool {
    !self.motivations.is_empty()
  }

  #[must_use]
  pub fn has_goals(&self) -> bool {
    !self.goals.is_empty()
  }
}

// ============================================================================
// CHARACTER BUILDER
// ============================================================================

pub struct CharacterBuilder {
  name: String,
  description: String,
  goals: Vec<String>,
  motivations: Vec<String>,
  constraints: Vec<String>,
}

impl CharacterBuilder {
  pub fn new(name: &str, description: &str) -> Self {
    Self {
      name: name.to_string(),
      description: description.to_string(),
      goals: Vec::new(),
      motivations: Vec::new(),
      constraints: Vec::new(),
    }
  }

  #[must_use]
  pub fn with_goal(mut self, goal: &str) -> Self {
    self.goals.push(goal.to_string());
    self
  }

  #[must_use]
  pub fn with_motivation(mut self, motivation: &str) -> Self {
    self.motivations.push(motivation.to_string());
    self
  }

  #[must_use]
  pub fn with_constraint(mut self, constraint: &str) -> Self {
    self.constraints.push(constraint.to_string());
    self
  }

  pub fn build(self) -> Result<Character, CharacterError> {
    let mut character = Character::new(&self.name, self.description)?;

    for goal in self.goals {
      character.add_goal(goal);
    }
    for motivation in self.motivations {
      character.add_motivation(motivation);
    }
    for constraint in self.constraints {
      character.add_constraint(constraint);
    }

    Ok(character)
  }
}

// ============================================================================
// TIMELINE EVENT ERROR
// ============================================================================

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimelineEventError {
  #[error("event description cannot be empty")]
  EmptyDescription,

  #[error("end time cannot be before start time")]
  EndTimeBeforeStart,
}

// ============================================================================
// TIMELINE EVENT
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEvent {
  id: Uuid,
  description: String,
  start_time: DateTime<Utc>,
  end_time: DateTime<Utc>,
}

impl TimelineEvent {
  pub fn new(
    description: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
  ) -> Result<Self, TimelineEventError> {
    if description.trim().is_empty() {
      return Err(TimelineEventError::EmptyDescription);
    }
    if end_time < start_time {
      return Err(TimelineEventError::EndTimeBeforeStart);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      description,
      start_time,
      end_time,
    })
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn description(&self) -> &str {
    &self.description
  }

  #[must_use]
  pub const fn start_time(&self) -> DateTime<Utc> {
    self.start_time
  }

  #[must_use]
  pub const fn end_time(&self) -> DateTime<Utc> {
    self.end_time
  }

  #[must_use]
  pub fn duration(&self) -> chrono::Duration {
    self.end_time - self.start_time
  }

  fn overlaps(&self, other: &Self) -> bool {
    self.start_time < other.end_time && other.start_time < self.end_time
  }
}

// ============================================================================
// EDGE CASE TYPE
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeCaseType {
  BoundaryCondition,
  EmptyInput,
  InvalidState,
  ResourceExhaustion,
  ConcurrentAccess,
  UnknownValue,
}

impl fmt::Display for EdgeCaseType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BoundaryCondition => write!(f, "Boundary Condition"),
      Self::EmptyInput => write!(f, "Empty Input"),
      Self::InvalidState => write!(f, "Invalid State"),
      Self::ResourceExhaustion => write!(f, "Resource Exhaustion"),
      Self::ConcurrentAccess => write!(f, "Concurrent Access"),
      Self::UnknownValue => write!(f, "Unknown Value"),
    }
  }
}

// ============================================================================
// PLOT HOLE TYPE
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotHoleType {
  DiscoveryMechanismMissing,
  EdgeCaseUnhandled(EdgeCaseType),
  TimelineInconsistent,
  MissingCharacterMotivation,
  LogicalContradiction,
}

impl fmt::Display for PlotHoleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::DiscoveryMechanismMissing => write!(f, "Discovery Mechanism Missing"),
      Self::EdgeCaseUnhandled(edge) => write!(f, "Edge Case Unhandled: {}", edge),
      Self::TimelineInconsistent => write!(f, "Timeline Inconsistent"),
      Self::MissingCharacterMotivation => write!(f, "Missing Character Motivation"),
      Self::LogicalContradiction => write!(f, "Logical Contradiction"),
    }
  }
}

impl PlotHoleType {
  fn default_severity(&self) -> f32 {
    match self {
      Self::TimelineInconsistent => 0.9,
      Self::LogicalContradiction => 0.85,
      Self::DiscoveryMechanismMissing => 0.6,
      Self::EdgeCaseUnhandled(_) => 0.5,
      Self::MissingCharacterMotivation => 0.4,
    }
  }
}

// ============================================================================
// PLOT HOLE
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotHole {
  id: Uuid,
  hole_type: PlotHoleType,
  description: String,
  severity: f32,
  detected_at: DateTime<Utc>,
}

impl PlotHole {
  pub fn new(hole_type: PlotHoleType, description: &str) -> Self {
    Self {
      id: Uuid::new_v4(),
      severity: hole_type.default_severity(),
      hole_type,
      description: description.to_string(),
      detected_at: Utc::now(),
    }
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn hole_type(&self) -> &PlotHoleType {
    &self.hole_type
  }

  #[must_use]
  pub fn description(&self) -> &str {
    &self.description
  }

  #[must_use]
  pub fn severity(&self) -> f32 {
    self.severity
  }

  #[must_use]
  pub fn with_severity(mut self, severity: f32) -> Self {
    self.severity = severity.clamp(0.0, 1.0);
    self
  }
}

// ============================================================================
// SIMULATION ERROR
// ============================================================================

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {
  #[error("simulation requires at least one character")]
  NoCharacters,

  #[error("simulation requires at least one timeline event")]
  NoTimeline,
}

// ============================================================================
// SIMULATION RESULT
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
  id: Uuid,
  plot_holes: Vec<PlotHole>,
  is_consistent: bool,
  confidence_score: f32,
  created_at: DateTime<Utc>,
}

impl SimulationResult {
  fn new(plot_holes: Vec<PlotHole>) -> Self {
    let is_consistent = plot_holes.is_empty();
    let confidence_score = if is_consistent { 1.0 } else { 0.7 };

    Self {
      id: Uuid::new_v4(),
      plot_holes,
      is_consistent,
      confidence_score,
      created_at: Utc::now(),
    }
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn plot_holes(&self) -> &[PlotHole] {
    &self.plot_holes
  }

  #[must_use]
  pub const fn is_consistent(&self) -> bool {
    self.is_consistent
  }

  #[must_use]
  pub const fn confidence_score(&self) -> f32 {
    self.confidence_score
  }

  #[must_use]
  pub fn critical_holes(&self) -> Vec<&PlotHole> {
    self
      .plot_holes
      .iter()
      .filter(|h| h.severity() >= 0.8)
      .collect()
  }
}

// ============================================================================
// SIMULATION
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Simulation {
  characters: Vec<Character>,
  timeline: Vec<TimelineEvent>,
}

impl Simulation {
  pub fn new(
    characters: Vec<Character>,
    timeline: Vec<TimelineEvent>,
  ) -> Result<Self, SimulationError> {
    if characters.is_empty() {
      return Err(SimulationError::NoCharacters);
    }
    if timeline.is_empty() {
      return Err(SimulationError::NoTimeline);
    }

    Ok(Self {
      characters,
      timeline,
    })
  }

  pub fn run(&self) -> Result<SimulationResult, SimulationError> {
    let mut plot_holes = Vec::new();

    self.detect_timeline_issues(&mut plot_holes);
    self.detect_character_issues(&mut plot_holes);
    self.detect_edge_cases(&mut plot_holes);

    Ok(SimulationResult::new(plot_holes))
  }

  fn detect_timeline_issues(&self, plot_holes: &mut Vec<PlotHole>) {
    let sorted_timeline: Vec<&TimelineEvent> = self
      .timeline
      .iter()
      .filter(|e| e.start_time() > e.end_time())
      .collect();

    if !sorted_timeline.is_empty() {
      plot_holes.push(PlotHole::new(
        PlotHoleType::TimelineInconsistent,
        "Timeline events have invalid time ordering",
      ));
    }

    for window in self.timeline.windows(2) {
      if window[0].overlaps(&window[1]) {
        plot_holes.push(PlotHole::new(
          PlotHoleType::TimelineInconsistent,
          &format!(
            "Events '{}' and '{}' have overlapping time ranges",
            window[0].description(),
            window[1].description()
          ),
        ));
      }
    }
  }

  fn detect_character_issues(&self, plot_holes: &mut Vec<PlotHole>) {
    for character in &self.characters {
      if !character.has_motivations() {
        plot_holes.push(PlotHole::new(
          PlotHoleType::EdgeCaseUnhandled(EdgeCaseType::EmptyInput),
          &format!("Character '{}' lacks motivations", character.name()),
        ));
      }
    }
  }

  fn detect_edge_cases(&self, plot_holes: &mut Vec<PlotHole>) {
    if self.timeline.len() == 1 {
      plot_holes.push(PlotHole::new(
        PlotHoleType::DiscoveryMechanismMissing,
        "Single event timeline may lack context for discovery",
      ));
    }
  }
}

// ============================================================================
// NORTH STAR SCENARIO ERROR
// ============================================================================

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NorthStarScenarioError {
  #[error("scenario title cannot be empty")]
  EmptyTitle,

  #[error("scenario requires at least one character")]
  NoCharacters,

  #[error("scenario requires at least one timeline event")]
  NoTimeline,
}

// ============================================================================
// NORTH STAR SCENARIO
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NorthStarScenario {
  id: Uuid,
  title: String,
  north_star_statement: Option<String>,
  characters: Vec<Character>,
  timeline: Vec<TimelineEvent>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl NorthStarScenario {
  pub fn new(
    title: &str,
    characters: Vec<Character>,
    timeline: Vec<TimelineEvent>,
  ) -> Result<Self, NorthStarScenarioError> {
    if title.trim().is_empty() {
      return Err(NorthStarScenarioError::EmptyTitle);
    }
    if characters.is_empty() {
      return Err(NorthStarScenarioError::NoCharacters);
    }
    if timeline.is_empty() {
      return Err(NorthStarScenarioError::NoTimeline);
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      title: title.to_string(),
      north_star_statement: None,
      characters,
      timeline,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn title(&self) -> &str {
    &self.title
  }

  #[must_use]
  pub fn north_star_statement(&self) -> Option<&str> {
    self.north_star_statement.as_deref()
  }

  #[must_use]
  pub fn characters(&self) -> &[Character] {
    &self.characters
  }

  #[must_use]
  pub fn timeline(&self) -> &[TimelineEvent] {
    &self.timeline
  }

  pub fn detect_plot_holes(&self) -> Result<Vec<PlotHole>, SimulationError> {
    let simulation = Simulation::new(self.characters.clone(), self.timeline.clone())?;
    let result = simulation.run()?;
    Ok(result.plot_holes().to_vec())
  }
}

// ============================================================================
// NORTH STAR SCENARIO BUILDER
// ============================================================================

pub struct NorthStarScenarioBuilder {
  title: String,
  north_star_statement: Option<String>,
  characters: Vec<Character>,
  timeline: Vec<TimelineEvent>,
}

impl NorthStarScenarioBuilder {
  pub fn new(title: &str) -> Self {
    Self {
      title: title.to_string(),
      north_star_statement: None,
      characters: Vec::new(),
      timeline: Vec::new(),
    }
  }

  #[must_use]
  pub fn with_north_star(mut self, statement: &str) -> Self {
    self.north_star_statement = Some(statement.to_string());
    self
  }

  #[must_use]
  pub fn with_character(mut self, character: Character) -> Self {
    self.characters.push(character);
    self
  }

  #[must_use]
  pub fn with_timeline_event(mut self, event: TimelineEvent) -> Self {
    self.timeline.push(event);
    self
  }

  pub fn build(self) -> Result<NorthStarScenario, NorthStarScenarioError> {
    let mut scenario = NorthStarScenario::new(&self.title, self.characters, self.timeline)?;

    if let Some(statement) = self.north_star_statement {
      scenario = scenario.with_north_star_statement(statement);
    }

    Ok(scenario)
  }
}

impl NorthStarScenario {
  #[must_use]
  pub fn with_north_star_statement(mut self, statement: String) -> Self {
    self.north_star_statement = Some(statement);
    self.updated_at = Utc::now();
    self
  }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_edge_case_types_have_display() {
    let types = [
      EdgeCaseType::BoundaryCondition,
      EdgeCaseType::EmptyInput,
      EdgeCaseType::InvalidState,
      EdgeCaseType::ResourceExhaustion,
      EdgeCaseType::ConcurrentAccess,
      EdgeCaseType::UnknownValue,
    ];

    for ect in types {
      let display = ect.to_string();
      assert!(!display.is_empty());
    }
  }

  #[test]
  fn test_plot_hole_types_have_display() {
    let types = [
      PlotHoleType::DiscoveryMechanismMissing,
      PlotHoleType::EdgeCaseUnhandled(EdgeCaseType::BoundaryCondition),
      PlotHoleType::TimelineInconsistent,
      PlotHoleType::MissingCharacterMotivation,
      PlotHoleType::LogicalContradiction,
    ];

    for pht in types {
      let display = pht.to_string();
      assert!(!display.is_empty());
    }
  }
}
