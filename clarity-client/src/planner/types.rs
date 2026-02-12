//! Planner domain types
//!
//! Pure domain types for the Diamond design methodology planning system.
//! All types are immutable, serializable, and follow zero-panic patterns.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Maximum collection size to prevent unbounded state growth
pub const MAX_COLLECTION_SIZE: usize = 10_000;

/// Maximum dependency depth to prevent stack overflow
pub const MAX_DEPTH: usize = 1_000;

/// Epsilon for considering a task complete (floating-point tolerance)
pub const COMPLETED_EPSILON: f32 = 1e-6;

/// Minimum required elements for Discovery phase completion
pub const MIN_DISCOVERY_PERSONAS: usize = 1;
pub const MIN_DISCOVERY_SCENARIOS: usize = 1;

/// Diamond methodology phases
///
/// Represents the four phases of the Diamond design methodology:
/// - Top: Discovery and problem exploration
/// - Right: Design and solution definition
/// - Bottom: Development and implementation
/// - Left: Delivery and validation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiamondPhase {
  /// Discovery phase - explore the problem space
  Top,
  /// Design phase - define solutions
  Right,
  /// Development phase - build and iterate
  Bottom,
  /// Delivery phase - validate and deliver
  Left,
}

impl DiamondPhase {
  /// Check if phase is active (is the current phase)
  #[must_use]
  pub fn is_active(&self, current_phase: DiamondPhase) -> bool {
    use DiamondPhase::*;
    matches!(
      (self, current_phase),
      (Top, Top) | (Right, Right) | (Bottom, Bottom) | (Left, Left)
    )
  }

  /// Check if phase is complete (all phases before current are complete)
  #[must_use]
  pub fn is_complete(&self, current_phase: DiamondPhase) -> bool {
    use DiamondPhase::*;
    match (self, current_phase) {
      // Current phase is always considered complete for rendering purposes
      (phase, current) if *phase == current => true,
      // All phases before the current one are complete
      (Top, _) => true, // Discovery is always complete if we've moved past it
      (Right, Bottom | Left) => true,
      (Bottom, Left) => true,
      (Left, _) => false, // Delivery is never "complete" as it's the final phase
      _ => false,
    }
  }

  /// Check if phase should be rendered (is active OR is complete)
  #[must_use]
  pub fn should_render(&self, current_phase: DiamondPhase) -> bool {
    self.is_active(current_phase) || self.is_complete(current_phase)
  }

  /// Get phase order for validation
  #[must_use]
  pub const fn order(&self) -> usize {
    use DiamondPhase::*;
    match self {
      Top => 0,
      Right => 1,
      Bottom => 2,
      Left => 3,
    }
  }

  /// Check if phase can be retreated to
  #[must_use]
  pub const fn can_retreat_to(&self) -> bool {
    !matches!(self, DiamondPhase::Top)
  }

  /// Get all phases that should be rendered for a given current phase
  #[must_use]
  pub fn get_rendered_phases(current_phase: DiamondPhase) -> Vec<DiamondPhase> {
    use DiamondPhase::*;
    match current_phase {
      Top => vec![Top],                       // Only Discovery renders
      Right => vec![Top, Right],              // Discovery and Design
      Bottom => vec![Top, Right, Bottom],     // All but Delivery
      Left => vec![Top, Right, Bottom, Left], // All phases
    }
  }
}

impl Default for DiamondPhase {
  fn default() -> Self {
    Self::Top
  }
}

impl fmt::Display for DiamondPhase {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Top => write!(f, "Discovery"),
      Self::Right => write!(f, "Design"),
      Self::Bottom => write!(f, "Development"),
      Self::Left => write!(f, "Delivery"),
    }
  }
}

/// Product thesis statement
///
/// Captures the core value proposition and problem understanding.
/// A well-formed thesis answers: What problem are we solving, for whom, and why?
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductThesis {
  /// Unique identifier
  pub id: Uuid,
  /// Thesis title
  pub title: String,
  /// Problem statement
  pub problem: String,
  /// Target audience
  pub audience: String,
  /// Proposed solution
  pub solution: String,
  /// Value proposition
  pub value_proposition: String,
  /// Success metrics
  pub success_metrics: Vec<String>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last modification timestamp
  pub updated_at: DateTime<Utc>,
}

impl ProductThesis {
  /// Create a new product thesis
  #[must_use]
  pub fn new(
    title: String,
    problem: String,
    audience: String,
    solution: String,
    value_proposition: String,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      title,
      problem,
      audience,
      solution,
      value_proposition,
      success_metrics: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a success metric
  #[must_use]
  pub fn with_success_metric(mut self, metric: String) -> Self {
    self.success_metrics.push(metric);
    self.updated_at = Utc::now();
    self
  }
}

impl Default for ProductThesis {
  fn default() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      title: String::new(),
      problem: String::new(),
      audience: String::new(),
      solution: String::new(),
      value_proposition: String::new(),
      success_metrics: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }
}

/// User persona definition
///
/// Represents a specific user archetype with goals, pain points, and behaviors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
  /// Unique identifier
  pub id: Uuid,
  /// Persona name/title
  pub name: String,
  /// Role or job title
  pub role: String,
  /// Description
  pub description: String,
  /// Goals and motivations
  pub goals: Vec<String>,
  /// Pain points and frustrations
  pub pain_points: Vec<String>,
  /// Behaviors and patterns
  pub behaviors: Vec<String>,
  /// Skill level (beginner, intermediate, expert)
  pub skill_level: String,
  /// Quote representing the persona
  pub quote: Option<String>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
}

impl Persona {
  /// Create a new persona
  #[must_use]
  pub fn new(name: String, role: String, description: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      role,
      description,
      goals: Vec::new(),
      pain_points: Vec::new(),
      behaviors: Vec::new(),
      skill_level: String::new(),
      quote: None,
      created_at: Utc::now(),
    }
  }

  /// Add a goal
  #[must_use]
  pub fn with_goal(mut self, goal: String) -> Self {
    self.goals.push(goal);
    self
  }

  /// Add a pain point
  #[must_use]
  pub fn with_pain_point(mut self, pain_point: String) -> Self {
    self.pain_points.push(pain_point);
    self
  }

  /// Add a behavior
  #[must_use]
  pub fn with_behavior(mut self, behavior: String) -> Self {
    self.behaviors.push(behavior);
    self
  }

  /// Set skill level
  #[must_use]
  pub fn with_skill_level(mut self, skill_level: String) -> Self {
    self.skill_level = skill_level;
    self
  }

  /// Set quote
  #[must_use]
  pub fn with_quote(mut self, quote: String) -> Self {
    self.quote = Some(quote);
    self
  }
}

/// North Star scenario - ideal user journey
///
/// Describes the ideal experience from the user's perspective.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NorthStarScenario {
  /// Unique identifier
  pub id: Uuid,
  /// Scenario title
  pub title: String,
  /// Narrative description
  pub narrative: String,
  /// Step-by-step journey
  pub steps: Vec<String>,
  /// Expected outcomes
  pub outcomes: Vec<String>,
  /// Related persona ID
  pub persona_id: Option<Uuid>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
}

impl NorthStarScenario {
  /// Create a new north star scenario
  #[must_use]
  pub fn new(title: String, narrative: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      title,
      narrative,
      steps: Vec::new(),
      outcomes: Vec::new(),
      persona_id: None,
      created_at: Utc::now(),
    }
  }

  /// Add a step
  #[must_use]
  pub fn with_step(mut self, step: String) -> Self {
    self.steps.push(step);
    self
  }

  /// Add an outcome
  #[must_use]
  pub fn with_outcome(mut self, outcome: String) -> Self {
    self.outcomes.push(outcome);
    self
  }

  /// Set persona
  #[must_use]
  pub fn with_persona(mut self, persona_id: Uuid) -> Self {
    self.persona_id = Some(persona_id);
    self
  }
}

/// Use case priority
///
/// Indicates the importance and urgency of implementing a use case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UseCasePriority {
  /// Must have - critical for MVP
  Critical,
  /// Should have - important but not blocking
  High,
  /// Could have - nice to have
  Medium,
  /// Won't have now - future consideration
  Low,
}

impl Default for UseCasePriority {
  fn default() -> Self {
    Self::Medium
  }
}

impl fmt::Display for UseCasePriority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Critical => write!(f, "Critical"),
      Self::High => write!(f, "High"),
      Self::Medium => write!(f, "Medium"),
      Self::Low => write!(f, "Low"),
    }
  }
}

/// Use case definition
///
/// Describes a specific user interaction and expected behavior.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCase {
  /// Unique identifier
  pub id: Uuid,
  /// Use case title
  pub title: String,
  /// Description
  pub description: String,
  /// User trigger/action
  pub trigger: String,
  /// Preconditions
  pub preconditions: Vec<String>,
  /// Main flow steps
  pub main_flow: Vec<String>,
  /// Alternative flows
  pub alternative_flows: Vec<String>,
  /// Post-conditions
  pub postconditions: Vec<String>,
  /// Priority level
  pub priority: UseCasePriority,
  /// Related persona ID
  pub persona_id: Option<Uuid>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
}

impl UseCase {
  /// Create a new use case
  #[must_use]
  pub fn new(title: String, description: String, trigger: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      title,
      description,
      trigger,
      preconditions: Vec::new(),
      main_flow: Vec::new(),
      alternative_flows: Vec::new(),
      postconditions: Vec::new(),
      priority: UseCasePriority::default(),
      persona_id: None,
      created_at: Utc::now(),
    }
  }

  /// Add a precondition
  #[must_use]
  pub fn with_precondition(mut self, precondition: String) -> Self {
    self.preconditions.push(precondition);
    self
  }

  /// Add a main flow step
  #[must_use]
  pub fn with_main_flow_step(mut self, step: String) -> Self {
    self.main_flow.push(step);
    self
  }

  /// Add an alternative flow
  #[must_use]
  pub fn with_alternative_flow(mut self, flow: String) -> Self {
    self.alternative_flows.push(flow);
    self
  }

  /// Add a postcondition
  #[must_use]
  pub fn with_postcondition(mut self, postcondition: String) -> Self {
    self.postconditions.push(postcondition);
    self
  }

  /// Set priority
  #[must_use]
  pub fn with_priority(mut self, priority: UseCasePriority) -> Self {
    self.priority = priority;
    self
  }

  /// Set persona
  #[must_use]
  pub fn with_persona(mut self, persona_id: Uuid) -> Self {
    self.persona_id = Some(persona_id);
    self
  }
}

/// Task type classification
///
/// Categorizes tasks by their nature and purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
  /// User research task
  Research,
  /// Design task
  Design,
  /// Development task
  Development,
  /// Testing task
  Testing,
  /// Documentation task
  Documentation,
  /// Planning task
  Planning,
  /// Review task
  Review,
  /// Infrastructure task
  Infrastructure,
  /// Other task type
  Other,
}

impl Default for TaskType {
  fn default() -> Self {
    Self::Other
  }
}

impl fmt::Display for TaskType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Research => write!(f, "Research"),
      Self::Design => write!(f, "Design"),
      Self::Development => write!(f, "Development"),
      Self::Testing => write!(f, "Testing"),
      Self::Documentation => write!(f, "Documentation"),
      Self::Planning => write!(f, "Planning"),
      Self::Review => write!(f, "Review"),
      Self::Infrastructure => write!(f, "Infrastructure"),
      Self::Other => write!(f, "Other"),
    }
  }
}

impl std::str::FromStr for TaskType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Research" => Ok(Self::Research),
      "Design" => Ok(Self::Design),
      "Development" => Ok(Self::Development),
      "Testing" => Ok(Self::Testing),
      "Documentation" => Ok(Self::Documentation),
      "Planning" => Ok(Self::Planning),
      "Review" => Ok(Self::Review),
      "Infrastructure" => Ok(Self::Infrastructure),
      "Other" => Ok(Self::Other),
      _ => Err(format!("Unknown TaskType: {s}")),
    }
  }
}

/// Task priority
///
/// Indicates the urgency and importance of a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
  /// Immediate attention required
  Urgent,
  /// High priority
  High,
  /// Normal priority
  Normal,
  /// Low priority
  Low,
}

impl Default for TaskPriority {
  fn default() -> Self {
    Self::Normal
  }
}

impl fmt::Display for TaskPriority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Urgent => write!(f, "Urgent"),
      Self::High => write!(f, "High"),
      Self::Normal => write!(f, "Normal"),
      Self::Low => write!(f, "Low"),
    }
  }
}

impl std::str::FromStr for TaskPriority {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Urgent" => Ok(Self::Urgent),
      "High" => Ok(Self::High),
      "Normal" => Ok(Self::Normal),
      "Low" => Ok(Self::Low),
      _ => Err(format!("Unknown TaskPriority: {s}")),
    }
  }
}

/// Effort estimation
///
/// Represents the estimated effort for a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
  /// Very small task (< 1 hour)
  Trivial,
  /// Small task (1-2 hours)
  Small,
  /// Medium task (half day)
  Medium,
  /// Large task (full day)
  Large,
  /// Very large task (multiple days)
  ExtraLarge,
}

impl Default for Effort {
  fn default() -> Self {
    Self::Medium
  }
}

impl fmt::Display for Effort {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Trivial => write!(f, "Trivial (< 1h)"),
      Self::Small => write!(f, "Small (1-2h)"),
      Self::Medium => write!(f, "Medium (½ day)"),
      Self::Large => write!(f, "Large (1 day)"),
      Self::ExtraLarge => write!(f, "Extra Large (multi-day)"),
    }
  }
}

impl std::str::FromStr for Effort {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Trivial" => Ok(Self::Trivial),
      "Small" => Ok(Self::Small),
      "Medium" => Ok(Self::Medium),
      "Large" => Ok(Self::Large),
      "ExtraLarge" => Ok(Self::ExtraLarge),
      _ => Err(format!("Unknown Effort: {s}")),
    }
  }
}

/// EARS requirement value
///
/// Based on the Easy Approach to Requirements Syntax (EARS).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarsValue {
  /// Universal requirement: "The system SHALL..."
  Universal {
    /// The requirement statement
    statement: String,
  },
  /// Existential requirement: "WHERE <condition> the system SHALL..."
  Existential {
    /// Condition that triggers the requirement
    condition: String,
    /// The requirement statement
    statement: String,
  },
  /// Event-driven requirement: "WHEN <trigger> the system SHALL..."
  EventDriven {
    /// Trigger event
    trigger: String,
    /// The requirement statement
    statement: String,
  },
  /// Optional behavior: "WHERE <condition> the system MAY..."
  Optional {
    /// Condition for optional behavior
    condition: String,
    /// The optional behavior statement
    statement: String,
  },
  /// State-driven requirement: "WHILE <state> the system SHALL..."
  StateDriven {
    /// The state condition
    state: String,
    /// The requirement statement
    statement: String,
  },
  /// Complex requirement with multiple conditions
  Complex {
    /// The requirement statement
    statement: String,
    /// Additional notes or context
    notes: String,
  },
}

impl fmt::Display for EarsValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Universal { statement } => write!(f, "The system shall {statement}"),
      Self::Existential {
        condition,
        statement,
      } => {
        write!(f, "Where {condition}, the system shall {statement}")
      }
      Self::EventDriven { trigger, statement } => {
        write!(f, "When {trigger}, the system shall {statement}")
      }
      Self::Optional {
        condition,
        statement,
      } => {
        write!(f, "Where {condition}, the system may {statement}")
      }
      Self::StateDriven { state, statement } => {
        write!(f, "While {state}, the system shall {statement}")
      }
      Self::Complex { statement, notes } => {
        write!(f, "{statement} ({notes})")
      }
    }
  }
}

/// EARS requirements collection
///
/// Organized requirements using the Easy Approach to Requirements Syntax.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarsRequirements {
  /// Unique identifier
  pub id: Uuid,
  /// Universal requirements
  pub universal: Vec<EarsValue>,
  /// Existential requirements
  pub existential: Vec<EarsValue>,
  /// Event-driven requirements
  pub event_driven: Vec<EarsValue>,
  /// Optional behaviors
  pub optional: Vec<EarsValue>,
  /// State-driven requirements
  pub state_driven: Vec<EarsValue>,
  /// Complex requirements
  pub complex: Vec<EarsValue>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl EarsRequirements {
  /// Create a new EARS requirements collection
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      universal: Vec::new(),
      existential: Vec::new(),
      event_driven: Vec::new(),
      optional: Vec::new(),
      state_driven: Vec::new(),
      complex: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a universal requirement
  #[must_use]
  pub fn with_universal(mut self, statement: String) -> Self {
    self.universal.push(EarsValue::Universal { statement });
    self.updated_at = Utc::now();
    self
  }

  /// Add an existential requirement
  #[must_use]
  pub fn with_existential(mut self, condition: String, statement: String) -> Self {
    self.existential.push(EarsValue::Existential {
      condition,
      statement,
    });
    self.updated_at = Utc::now();
    self
  }

  /// Add an event-driven requirement
  #[must_use]
  pub fn with_event_driven(mut self, trigger: String, statement: String) -> Self {
    self
      .event_driven
      .push(EarsValue::EventDriven { trigger, statement });
    self.updated_at = Utc::now();
    self
  }

  /// Add an optional behavior
  #[must_use]
  pub fn with_optional(mut self, condition: String, statement: String) -> Self {
    self.optional.push(EarsValue::Optional {
      condition,
      statement,
    });
    self.updated_at = Utc::now();
    self
  }

  /// Add a state-driven requirement
  #[must_use]
  pub fn with_state_driven(mut self, state: String, statement: String) -> Self {
    self
      .state_driven
      .push(EarsValue::StateDriven { state, statement });
    self.updated_at = Utc::now();
    self
  }

  /// Add a complex requirement
  #[must_use]
  pub fn with_complex(mut self, statement: String, notes: String) -> Self {
    self.complex.push(EarsValue::Complex { statement, notes });
    self.updated_at = Utc::now();
    self
  }

  /// Get all requirements
  #[must_use]
  pub fn all_requirements(&self) -> Vec<EarsValue> {
    let mut all = Vec::new();
    all.extend(self.universal.iter().cloned());
    all.extend(self.existential.iter().cloned());
    all.extend(self.event_driven.iter().cloned());
    all.extend(self.optional.iter().cloned());
    all.extend(self.state_driven.iter().cloned());
    all.extend(self.complex.iter().cloned());
    all
  }
}

impl Default for EarsRequirements {
  fn default() -> Self {
    Self::new()
  }
}

/// Contract definitions
///
/// External contracts, APIs, and service dependencies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contracts {
  /// Unique identifier
  pub id: Uuid,
  /// API contracts
  pub api_contracts: Vec<ApiContract>,
  /// Service dependencies
  pub service_dependencies: Vec<ServiceDependency>,
  /// Data contracts
  pub data_contracts: Vec<DataContract>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl Contracts {
  /// Create a new contracts collection
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      api_contracts: Vec::new(),
      service_dependencies: Vec::new(),
      data_contracts: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add an API contract
  #[must_use]
  pub fn with_api_contract(mut self, contract: ApiContract) -> Self {
    self.api_contracts.push(contract);
    self.updated_at = Utc::now();
    self
  }

  /// Add a service dependency
  #[must_use]
  pub fn with_service_dependency(mut self, dependency: ServiceDependency) -> Self {
    self.service_dependencies.push(dependency);
    self.updated_at = Utc::now();
    self
  }

  /// Add a data contract
  #[must_use]
  pub fn with_data_contract(mut self, contract: DataContract) -> Self {
    self.data_contracts.push(contract);
    self.updated_at = Utc::now();
    self
  }
}

impl Default for Contracts {
  fn default() -> Self {
    Self::new()
  }
}

/// API contract definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiContract {
  /// Contract name
  pub name: String,
  /// Endpoint URL
  pub endpoint: String,
  /// HTTP method
  pub method: String,
  /// Request schema
  pub request_schema: Option<String>,
  /// Response schema
  pub response_schema: Option<String>,
  /// Authentication type
  pub auth_type: Option<String>,
}

/// Service dependency definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDependency {
  /// Service name
  pub name: String,
  /// Service type
  pub service_type: String,
  /// Version
  pub version: Option<String>,
  /// Criticality
  pub criticality: String,
}

/// Data contract definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataContract {
  /// Contract name
  pub name: String,
  /// Data format
  pub format: String,
  /// Schema
  pub schema: Option<String>,
  /// Validation rules
  pub validation_rules: Vec<String>,
}

/// Test definitions
///
/// Test cases and testing strategy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tests {
  /// Unique identifier
  pub id: Uuid,
  /// Unit test cases
  pub unit_tests: Vec<TestCase>,
  /// Integration test cases
  pub integration_tests: Vec<TestCase>,
  /// End-to-end test cases
  pub e2e_tests: Vec<TestCase>,
  /// Performance test cases
  pub performance_tests: Vec<TestCase>,
  /// Security test cases
  pub security_tests: Vec<TestCase>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl Tests {
  /// Create a new tests collection
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      unit_tests: Vec::new(),
      integration_tests: Vec::new(),
      e2e_tests: Vec::new(),
      performance_tests: Vec::new(),
      security_tests: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a unit test
  #[must_use]
  pub fn with_unit_test(mut self, test: TestCase) -> Self {
    self.unit_tests.push(test);
    self.updated_at = Utc::now();
    self
  }

  /// Add an integration test
  #[must_use]
  pub fn with_integration_test(mut self, test: TestCase) -> Self {
    self.integration_tests.push(test);
    self.updated_at = Utc::now();
    self
  }

  /// Add an E2E test
  #[must_use]
  pub fn with_e2e_test(mut self, test: TestCase) -> Self {
    self.e2e_tests.push(test);
    self.updated_at = Utc::now();
    self
  }

  /// Add a performance test
  #[must_use]
  pub fn with_performance_test(mut self, test: TestCase) -> Self {
    self.performance_tests.push(test);
    self.updated_at = Utc::now();
    self
  }

  /// Add a security test
  #[must_use]
  pub fn with_security_test(mut self, test: TestCase) -> Self {
    self.security_tests.push(test);
    self.updated_at = Utc::now();
    self
  }
}

impl Default for Tests {
  fn default() -> Self {
    Self::new()
  }
}

/// Test case definition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCase {
  /// Test name
  pub name: String,
  /// Description
  pub description: String,
  /// Test steps
  pub steps: Vec<String>,
  /// Expected result
  pub expected_result: String,
  /// Priority
  pub priority: TaskPriority,
}

/// Research findings
///
/// Research tasks, findings, and insights.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Research {
  /// Unique identifier
  pub id: Uuid,
  /// Research questions
  pub questions: Vec<String>,
  /// Methods to be used
  pub methods: Vec<String>,
  /// Participants or sources
  pub participants: Vec<String>,
  /// Key findings
  pub findings: Vec<ResearchFinding>,
  /// Insights and recommendations
  pub insights: Vec<String>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl Research {
  /// Create a new research collection
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      questions: Vec::new(),
      methods: Vec::new(),
      participants: Vec::new(),
      findings: Vec::new(),
      insights: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a research question
  #[must_use]
  pub fn with_question(mut self, question: String) -> Self {
    self.questions.push(question);
    self.updated_at = Utc::now();
    self
  }

  /// Add a research method
  #[must_use]
  pub fn with_method(mut self, method: String) -> Self {
    self.methods.push(method);
    self.updated_at = Utc::now();
    self
  }

  /// Add a participant
  #[must_use]
  pub fn with_participant(mut self, participant: String) -> Self {
    self.participants.push(participant);
    self.updated_at = Utc::now();
    self
  }

  /// Add a finding
  #[must_use]
  pub fn with_finding(mut self, finding: ResearchFinding) -> Self {
    self.findings.push(finding);
    self.updated_at = Utc::now();
    self
  }

  /// Add an insight
  #[must_use]
  pub fn with_insight(mut self, insight: String) -> Self {
    self.insights.push(insight);
    self.updated_at = Utc::now();
    self
  }
}

impl Default for Research {
  fn default() -> Self {
    Self::new()
  }
}

/// Research finding
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchFinding {
  /// Finding title
  pub title: String,
  /// Description
  pub description: String,
  /// Evidence or quotes
  pub evidence: Vec<String>,
  /// Confidence level (low, medium, high)
  pub confidence: String,
}

/// Implementation plan
///
/// Implementation details and technical specifications.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Implementation {
  /// Unique identifier
  pub id: Uuid,
  /// Technical architecture
  pub architecture: Vec<String>,
  /// Technology stack
  pub tech_stack: Vec<String>,
  /// Implementation phases
  pub phases: Vec<ImplementationPhase>,
  /// Dependencies
  pub dependencies: Vec<String>,
  /// Risk mitigation
  pub risks: Vec<Risk>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl Implementation {
  /// Create a new implementation plan
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      architecture: Vec::new(),
      tech_stack: Vec::new(),
      phases: Vec::new(),
      dependencies: Vec::new(),
      risks: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add an architecture note
  #[must_use]
  pub fn with_architecture(mut self, arch: String) -> Self {
    self.architecture.push(arch);
    self.updated_at = Utc::now();
    self
  }

  /// Add a technology
  #[must_use]
  pub fn with_tech(mut self, tech: String) -> Self {
    self.tech_stack.push(tech);
    self.updated_at = Utc::now();
    self
  }

  /// Add a phase
  #[must_use]
  pub fn with_phase(mut self, phase: ImplementationPhase) -> Self {
    self.phases.push(phase);
    self.updated_at = Utc::now();
    self
  }

  /// Add a dependency
  #[must_use]
  pub fn with_dependency(mut self, dep: String) -> Self {
    self.dependencies.push(dep);
    self.updated_at = Utc::now();
    self
  }

  /// Add a risk
  #[must_use]
  pub fn with_risk(mut self, risk: Risk) -> Self {
    self.risks.push(risk);
    self.updated_at = Utc::now();
    self
  }
}

impl Default for Implementation {
  fn default() -> Self {
    Self::new()
  }
}

/// Implementation phase
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationPhase {
  /// Phase name
  pub name: String,
  /// Description
  pub description: String,
  /// Estimated duration
  pub duration: String,
  /// Deliverables
  pub deliverables: Vec<String>,
}

/// Risk assessment
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risk {
  /// Risk description
  pub description: String,
  /// Impact level (low, medium, high, critical)
  pub impact: String,
  /// Probability (low, medium, high)
  pub probability: String,
  /// Mitigation strategy
  pub mitigation: String,
}

/// Task detail - EARS requirements
///
/// Simplified EARS requirements for task-level use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEarsRequirements {
  /// Ubiquitous requirements (THE SYSTEM SHALL...)
  pub ubiquitous: Vec<String>,
  /// Event-driven requirements (WHEN trigger THEN response)
  pub event_driven: Vec<EventDrivenRequirement>,
  /// Unwanted behaviors (IF condition SHALL NOT...)
  pub unwanted: Vec<UnwantedRequirement>,
}

impl Default for TaskEarsRequirements {
  fn default() -> Self {
    Self {
      ubiquitous: Vec::new(),
      event_driven: Vec::new(),
      unwanted: Vec::new(),
    }
  }
}

/// Event-driven requirement (WHEN trigger THEN response)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDrivenRequirement {
  /// Trigger event
  pub trigger: String,
  /// Response (THE SYSTEM SHALL...)
  pub response: String,
}

/// Unwanted requirement (IF condition SHALL NOT...)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnwantedRequirement {
  /// Condition
  pub condition: String,
  /// What shall not happen
  pub shall_not: String,
  /// Reason (BECAUSE...)
  pub because: String,
}

/// Task detail - Contracts
///
/// Design-by-contract invariants for the task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskContracts {
  /// Preconditions (must be true BEFORE execution)
  pub preconditions: Vec<String>,
  /// Postconditions (must be true AFTER completion)
  pub postconditions: Vec<String>,
  /// Invariants (always true throughout)
  pub invariants: Vec<String>,
}

impl Default for TaskContracts {
  fn default() -> Self {
    Self {
      preconditions: Vec::new(),
      postconditions: Vec::new(),
      invariants: Vec::new(),
    }
  }
}

/// Task detail - Tests
///
/// Test scenarios for the task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTests {
  /// Happy path tests (it works)
  pub happy: Vec<String>,
  /// Error path tests (it fails gracefully)
  pub error: Vec<String>,
  /// Edge case tests (boundary conditions)
  pub edge: Vec<String>,
}

impl Default for TaskTests {
  fn default() -> Self {
    Self {
      happy: Vec::new(),
      error: Vec::new(),
      edge: Vec::new(),
    }
  }
}

/// Task detail - Research
///
/// Research findings and questions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskResearch {
  /// Files to read
  pub files: Vec<String>,
  /// Patterns to find
  pub patterns: Vec<String>,
  /// Open questions
  pub questions: Vec<String>,
}

impl Default for TaskResearch {
  fn default() -> Self {
    Self {
      files: Vec::new(),
      patterns: Vec::new(),
      questions: Vec::new(),
    }
  }
}

/// Task detail - Implementation
///
/// Implementation phases for the task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskImplementation {
  /// Phase 0: Research steps
  pub phase0: Vec<String>,
  /// Phase 1: Tests to write
  pub phase1: Vec<String>,
  /// Phase 2: Implementation steps
  pub phase2: Vec<String>,
}

impl Default for TaskImplementation {
  fn default() -> Self {
    Self {
      phase0: Vec::new(),
      phase1: Vec::new(),
      phase2: Vec::new(),
    }
  }
}

/// Plan task
///
/// Individual task within a plan session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanTask {
  /// Unique identifier
  pub id: Uuid,
  /// Task title
  pub title: String,
  /// Description
  pub description: String,
  /// Task type
  pub task_type: TaskType,
  /// Priority
  pub priority: TaskPriority,
  /// Estimated effort
  pub effort: Effort,
  /// Associated diamond phase
  pub phase: DiamondPhase,
  /// Completion status (0.0 to 1.0)
  pub completion: f32,
  /// Dependencies (task IDs)
  pub dependencies: Vec<Uuid>,
  /// Tags
  pub tags: Vec<String>,
  /// EARS requirements
  pub ears: TaskEarsRequirements,
  /// Contracts
  pub contracts: TaskContracts,
  /// Tests
  pub tests: TaskTests,
  /// Research
  pub research: TaskResearch,
  /// Implementation phases
  pub implementation: TaskImplementation,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
  /// Due date
  pub due_date: Option<DateTime<Utc>>,
}

/// State management errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
  #[error("collection size exceeds maximum of {MAX_COLLECTION_SIZE}")]
  CollectionTooLarge,

  #[error("duplicate ID detected: {0}")]
  DuplicateId(String),

  #[error("self-dependency detected: task {0} cannot depend on itself")]
  SelfDependency(Uuid),

  #[error("dependency depth exceeds maximum of {MAX_DEPTH}")]
  MaxDepthExceeded,

  #[error("cycle detected: {0}")]
  CycleDetected(String),

  #[error("phase transition not allowed: {current_phase} -> {next_phase}")]
  InvalidPhaseTransition {
    current_phase: DiamondPhase,
    next_phase: DiamondPhase,
  },

  #[error("phase completion requirements not met for {0}")]
  PhaseNotReady(DiamondPhase),
}

impl PlanTask {
  /// Create a new plan task
  #[must_use]
  pub fn new(title: String, description: String, task_type: TaskType, phase: DiamondPhase) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      title,
      description,
      task_type,
      priority: TaskPriority::default(),
      effort: Effort::default(),
      phase,
      completion: 0.0,
      dependencies: Vec::new(),
      tags: Vec::new(),
      ears: TaskEarsRequirements::default(),
      contracts: TaskContracts::default(),
      tests: TaskTests::default(),
      research: TaskResearch::default(),
      implementation: TaskImplementation::default(),
      created_at: now,
      updated_at: now,
      due_date: None,
    }
  }

  /// Set priority
  #[must_use]
  pub const fn with_priority(mut self, priority: TaskPriority) -> Self {
    self.priority = priority;
    self
  }

  /// Set effort
  #[must_use]
  pub const fn with_effort(mut self, effort: Effort) -> Self {
    self.effort = effort;
    self
  }

  /// Set completion
  #[must_use]
  pub const fn with_completion(mut self, completion: f32) -> Self {
    self.completion = completion;
    self
  }

  /// Add a dependency
  ///
  /// # Panics
  /// Panics if trying to add self as dependency (this is a type-level invariant violation)
  ///
  /// # Errors
  /// This method will never return a task with self-dependencies. If you attempt
  /// to add the task's own ID as a dependency, the dependency will be silently ignored.
  #[must_use]
  pub fn with_dependency(mut self, dep_id: Uuid) -> Self {
    // CRITICAL-003: Prevent self-dependencies at construction time
    // This is a type-level invariant - tasks can NEVER depend on themselves
    if dep_id != self.id {
      self.dependencies.push(dep_id);
    }
    self
  }

  /// Check if task is complete (with epsilon tolerance)
  #[must_use]
  pub const fn is_complete(&self) -> bool {
    self.completion >= 1.0 - COMPLETED_EPSILON
  }

  /// Add a tag
  #[must_use]
  pub fn with_tag(mut self, tag: String) -> Self {
    self.tags.push(tag);
    self
  }

  /// Set due date
  #[must_use]
  pub const fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
    self.due_date = Some(due_date);
    self
  }

  /// Set EARS requirements
  #[must_use]
  pub fn with_ears(mut self, ears: TaskEarsRequirements) -> Self {
    self.ears = ears;
    self
  }

  /// Set contracts
  #[must_use]
  pub fn with_contracts(mut self, contracts: TaskContracts) -> Self {
    self.contracts = contracts;
    self
  }

  /// Set tests
  #[must_use]
  pub fn with_tests(mut self, tests: TaskTests) -> Self {
    self.tests = tests;
    self
  }

  /// Set research
  #[must_use]
  pub fn with_research(mut self, research: TaskResearch) -> Self {
    self.research = research;
    self
  }

  /// Set implementation phases
  #[must_use]
  pub fn with_implementation(mut self, implementation: TaskImplementation) -> Self {
    self.implementation = implementation;
    self
  }
}

/// Plan session
///
/// A complete planning session using the Diamond methodology.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanSession {
  /// Unique identifier
  pub id: Uuid,
  /// Session title
  pub title: String,
  /// Product thesis
  pub thesis: ProductThesis,
  /// Personas
  pub personas: Vec<Persona>,
  /// North star scenarios
  pub north_star_scenarios: Vec<NorthStarScenario>,
  /// Use cases
  pub use_cases: Vec<UseCase>,
  /// Requirements
  pub requirements: EarsRequirements,
  /// Contracts
  pub contracts: Contracts,
  /// Tests
  pub tests: Tests,
  /// Research
  pub research: Research,
  /// Implementation plan
  pub implementation: Implementation,
  /// Tasks
  pub tasks: Vec<PlanTask>,
  /// Validation checks
  pub validation_checks: Vec<ValidationCheck>,
  /// Graph health metrics
  pub graph_health: GraphHealth,
  /// Current phase
  pub current_phase: DiamondPhase,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl PlanSession {
  /// Create a new plan session
  #[must_use]
  pub fn new(title: String, thesis: ProductThesis) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      title,
      thesis,
      personas: Vec::new(),
      north_star_scenarios: Vec::new(),
      use_cases: Vec::new(),
      requirements: EarsRequirements::new(),
      contracts: Contracts::new(),
      tests: Tests::new(),
      research: Research::new(),
      implementation: Implementation::new(),
      tasks: Vec::new(),
      validation_checks: Vec::new(),
      graph_health: GraphHealth::new(),
      current_phase: DiamondPhase::default(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a persona
  #[must_use]
  pub fn with_persona(mut self, persona: Persona) -> Self {
    self.personas.push(persona);
    self.updated_at = Utc::now();
    self
  }

  /// Add a north star scenario
  #[must_use]
  pub fn with_north_star_scenario(mut self, scenario: NorthStarScenario) -> Self {
    self.north_star_scenarios.push(scenario);
    self.updated_at = Utc::now();
    self
  }

  /// Add a use case
  #[must_use]
  pub fn with_use_case(mut self, use_case: UseCase) -> Self {
    self.use_cases.push(use_case);
    self.updated_at = Utc::now();
    self
  }

  /// Add a task
  #[must_use]
  pub fn with_task(mut self, task: PlanTask) -> Self {
    self.tasks.push(task);
    self.updated_at = Utc::now();
    self
  }

  /// Add a validation check
  #[must_use]
  pub fn with_validation_check(mut self, check: ValidationCheck) -> Self {
    self.validation_checks.push(check);
    self.updated_at = Utc::now();
    self
  }

  /// Set current phase
  #[must_use]
  pub const fn with_current_phase(mut self, phase: DiamondPhase) -> Self {
    self.current_phase = phase;
    self
  }
}

/// Validation check severity
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
  /// Information only
  Info,
  /// Warning - should be addressed
  Warning,
  /// Error - must be addressed
  Error,
  /// Critical - blocks progress
  Critical,
}

impl Default for ValidationSeverity {
  fn default() -> Self {
    Self::Info
  }
}

impl fmt::Display for ValidationSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Info => write!(f, "Info"),
      Self::Warning => write!(f, "Warning"),
      Self::Error => write!(f, "Error"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

/// Validation check
///
/// Represents a validation rule or check on the plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationCheck {
  /// Unique identifier
  pub id: Uuid,
  /// Check name
  pub name: String,
  /// Description
  pub description: String,
  /// Severity level
  pub severity: ValidationSeverity,
  /// Whether the check passed
  pub passed: bool,
  /// Error or warning message
  pub message: Option<String>,
  /// Related entity ID (if applicable)
  pub entity_id: Option<Uuid>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
}

impl ValidationCheck {
  /// Create a new validation check
  #[must_use]
  pub fn new(name: String, description: String, severity: ValidationSeverity) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      severity,
      passed: false,
      message: None,
      entity_id: None,
      created_at: Utc::now(),
    }
  }

  /// Set pass status
  #[must_use]
  pub const fn with_passed(mut self, passed: bool) -> Self {
    self.passed = passed;
    self
  }

  /// Set message
  #[must_use]
  pub fn with_message(mut self, message: String) -> Self {
    self.message = Some(message);
    self
  }

  /// Set entity
  #[must_use]
  pub const fn with_entity(mut self, entity_id: Uuid) -> Self {
    self.entity_id = Some(entity_id);
    self
  }
}

/// Graph health metrics
///
/// Health indicators for the plan graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphHealth {
  /// Unique identifier
  pub id: Uuid,
  /// Total number of nodes
  pub node_count: usize,
  /// Total number of edges
  pub edge_count: usize,
  /// Number of disconnected components
  pub disconnected_components: usize,
  /// Maximum depth from root
  pub max_depth: usize,
  /// Number of orphaned nodes
  pub orphaned_nodes: usize,
  /// Cyclomatic complexity
  pub complexity: f32,
  /// Overall health score (0.0 to 1.0)
  pub health_score: f32,
  /// Last calculation timestamp
  pub calculated_at: DateTime<Utc>,
}

impl GraphHealth {
  /// Create a new graph health metrics
  #[must_use]
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      node_count: 0,
      edge_count: 0,
      disconnected_components: 0,
      max_depth: 0,
      orphaned_nodes: 0,
      complexity: 0.0,
      health_score: 1.0,
      calculated_at: Utc::now(),
    }
  }

  /// Calculate health score based on metrics
  #[must_use]
  pub fn calculate_score(&self) -> f32 {
    // Empty graph has perfect health
    if self.node_count == 0 {
      return 1.0;
    }

    // Base score starts at 1.0
    let mut score = 1.0_f32;

    // Penalize orphaned nodes
    score -= (self.orphaned_nodes as f32 / self.node_count as f32).min(0.3);

    // Penalize disconnected components
    score -= (self.disconnected_components as f32 * 0.1).min(0.2);

    // Penalize excessive complexity
    score -= (self.complexity / 100.0).min(0.2);

    score.max(0.0).min(1.0)
  }

  /// Update with calculated metrics
  #[must_use]
  pub fn with_metrics(
    mut self,
    node_count: usize,
    edge_count: usize,
    disconnected_components: usize,
    max_depth: usize,
    orphaned_nodes: usize,
    complexity: f32,
  ) -> Self {
    self.node_count = node_count;
    self.edge_count = edge_count;
    self.disconnected_components = disconnected_components;
    self.max_depth = max_depth;
    self.orphaned_nodes = orphaned_nodes;
    self.complexity = complexity;
    self.health_score = self.calculate_score();
    self.calculated_at = Utc::now();
    self
  }
}

impl Default for GraphHealth {
  fn default() -> Self {
    Self::new()
  }
}

/// Test helper for phase rendering
#[cfg(test)]
mod phase_rendering_tests {
  use super::*;

  #[test]
  fn test_phase_is_active() {
    // Phase is active only when it matches current phase
    assert!(DiamondPhase::Top.is_active(DiamondPhase::Top));
    assert!(DiamondPhase::Right.is_active(DiamondPhase::Right));
    assert!(DiamondPhase::Bottom.is_active(DiamondPhase::Bottom));
    assert!(DiamondPhase::Left.is_active(DiamondPhase::Left));

    // Phase is not active when it doesn't match current phase
    assert!(!DiamondPhase::Top.is_active(DiamondPhase::Right));
    assert!(!DiamondPhase::Right.is_active(DiamondPhase::Bottom));
  }

  #[test]
  fn test_phase_is_complete() {
    // Discovery phase (Top)
    assert!(DiamondPhase::Top.is_complete(DiamondPhase::Top)); // Current phase
    assert!(DiamondPhase::Top.is_complete(DiamondPhase::Right)); // Before Right
    assert!(DiamondPhase::Top.is_complete(DiamondPhase::Bottom)); // Before Bottom
    assert!(DiamondPhase::Top.is_complete(DiamondPhase::Left)); // Before Left

    // Design phase (Right)
    assert!(!DiamondPhase::Right.is_complete(DiamondPhase::Top)); // Not reached yet
    assert!(DiamondPhase::Right.is_complete(DiamondPhase::Right)); // Current phase
    assert!(DiamondPhase::Right.is_complete(DiamondPhase::Bottom)); // Before Bottom
    assert!(DiamondPhase::Right.is_complete(DiamondPhase::Left)); // Before Left

    // Development phase (Bottom)
    assert!(!DiamondPhase::Bottom.is_complete(DiamondPhase::Top)); // Not reached yet
    assert!(!DiamondPhase::Bottom.is_complete(DiamondPhase::Right)); // Not reached yet
    assert!(DiamondPhase::Bottom.is_complete(DiamondPhase::Bottom)); // Current phase
    assert!(DiamondPhase::Bottom.is_complete(DiamondPhase::Left)); // Before Left

    // Delivery phase (Left)
    assert!(!DiamondPhase::Left.is_complete(DiamondPhase::Top)); // Not reached yet
    assert!(!DiamondPhase::Left.is_complete(DiamondPhase::Right)); // Not reached yet
    assert!(!DiamondPhase::Left.is_complete(DiamondPhase::Bottom)); // Not reached yet
                                                                    // Note: Left is never "complete" as it's the final phase
  }

  #[test]
  fn test_phase_should_render() {
    use DiamondPhase::*;

    // Discovery phase (Top)
    assert!(Top.should_render(Top)); // Active
    assert!(Top.should_render(Right)); // Complete
    assert!(Top.should_render(Bottom)); // Complete
    assert!(Top.should_render(Left)); // Complete

    // Design phase (Right)
    assert!(!Right.should_render(Top)); // Not active, not complete
    assert!(Right.should_render(Right)); // Active
    assert!(Right.should_render(Bottom)); // Complete
    assert!(Right.should_render(Left)); // Complete

    // Development phase (Bottom)
    assert!(!Bottom.should_render(Top)); // Not active, not complete
    assert!(!Bottom.should_render(Right)); // Not active, not complete
    assert!(Bottom.should_render(Bottom)); // Active
    assert!(Bottom.should_render(Left)); // Complete

    // Delivery phase (Left)
    assert!(!Left.should_render(Top)); // Not active, not complete
    assert!(!Left.should_render(Right)); // Not active, not complete
    assert!(!Left.should_render(Bottom)); // Not active, not complete
    assert!(Left.should_render(Left)); // Active
  }

  #[test]
  fn test_get_rendered_phases() {
    use DiamondPhase::*;

    assert_eq!(DiamondPhase::get_rendered_phases(Top), vec![Top]);
    assert_eq!(DiamondPhase::get_rendered_phases(Right), vec![Top, Right]);
    assert_eq!(
      DiamondPhase::get_rendered_phases(Bottom),
      vec![Top, Right, Bottom]
    );
    assert_eq!(
      DiamondPhase::get_rendered_phases(Left),
      vec![Top, Right, Bottom, Left]
    );
  }

  #[test]
  fn test_phase_order() {
    assert_eq!(DiamondPhase::Top.order(), 0);
    assert_eq!(DiamondPhase::Right.order(), 1);
    assert_eq!(DiamondPhase::Bottom.order(), 2);
    assert_eq!(DiamondPhase::Left.order(), 3);
  }

  #[test]
  fn test_can_retreat_to() {
    assert!(!DiamondPhase::Top.can_retreat_to()); // Cannot retreat from start
    assert!(DiamondPhase::Right.can_retreat_to()); // Can retreat to Discovery
    assert!(DiamondPhase::Bottom.can_retreat_to()); // Can retreat to Design
    assert!(DiamondPhase::Left.can_retreat_to()); // Can retreat to Development
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // CRITICAL-003 HOSTILE TEST: with_dependency rejects self-dependency
  #[test]
  fn test_with_dependency_rejects_self_dependency() {
    let task = PlanTask::new(
      "Test Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_id = task.id;

    // Try to add self as dependency
    let task_with_self_dep = task.clone().with_dependency(task_id);

    // The dependency should be silently ignored (not added)
    assert!(
      !task_with_self_dep.dependencies.contains(&task_id),
      "Task should not contain itself as dependency"
    );
    assert!(
      task_with_self_dep.dependencies.is_empty(),
      "Dependencies should be empty when trying to add self"
    );
  }

  // CRITICAL-003 HOSTILE TEST: Multiple self-dependency attempts
  #[test]
  fn test_with_dependency_multiple_self_attempts() {
    let task = PlanTask::new(
      "Test Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_id = task.id;

    // Try multiple times to add self
    let task1 = task.clone().with_dependency(task_id);
    let task2 = task1.with_dependency(task_id);
    let task3 = task2.with_dependency(task_id);

    // All attempts should be silently ignored
    assert!(task3.dependencies.is_empty());
  }

  // CRITICAL-003 HOSTILE TEST: Self-dependency mixed with valid dependencies
  #[test]
  fn test_with_dependency_valid_and_self_mixed() {
    let task1 = PlanTask::new(
      "Task 1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task2 = PlanTask::new(
      "Task 2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    // Add valid dependency, then self, then valid
    let task1_with_deps = task1
      .clone()
      .with_dependency(task2.id)
      .with_dependency(task1.id) // Self
      .with_dependency(task2.id); // Duplicate of valid

    // Should have 2 valid dependencies (both task2), self ignored
    assert_eq!(task1_with_deps.dependencies.len(), 2);
    assert!(
      task1_with_deps
        .dependencies
        .iter()
        .all(|&id| id == task2.id),
      "All dependencies should be task2"
    );
    assert!(
      !task1_with_deps.dependencies.contains(&task1.id),
      "Should not contain self-dependency"
    );
  }

  // CRITICAL-003 HOSTILE TEST: Validation catches self-dependencies
  #[test]
  fn test_validate_task_catches_self_dependency() {
    use crate::planner::validation::{validate_task, ValidationError};

    let task = PlanTask::new(
      "Test Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    // Manually create task with self-dependency (bypassing with_dependency)
    let task_with_self = PlanTask {
      dependencies: vec![task.id],
      ..task
    };

    let result = validate_task(&task_with_self);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors
      .iter()
      .any(|e| matches!(e, ValidationError::SelfDependency(_))));
  }

  // CRITICAL-003 HOSTILE TEST: Clone of task with self-dependency
  #[test]
  fn test_task_clone_preserves_no_self_dependency() {
    let task1 = PlanTask::new(
      "Task 1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task2 = PlanTask::new(
      "Task 2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_with_deps = task1.with_dependency(task2.id);
    let cloned = task_with_deps.clone();

    // Try to add self to clone
    let cloned_with_self = cloned.clone().with_dependency(cloned.id);

    // Should still not have self-dependency
    assert!(!cloned_with_self.dependencies.contains(&cloned.id));
  }
}
