use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanError {
  #[error("dependency error: {0}")]
  DependencyError(String),
  #[error("invalid phase: {0}")]
  InvalidPhase(u32),
  #[error("phase not complete: {0}")]
  PhaseNotComplete(u32),
  #[error("no actionable beads")]
  NoActionableBeads,
  #[error("session ID is empty")]
  EmptySessionId,
  #[error("circular dependency detected: {0} -> {1}")]
  CircularDependency(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
  #[default]
  Pending,
  InProgress,
  Complete,
  Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadStatus {
  #[default]
  Pending,
  Ready,
  InProgress,
  Complete,
  Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBead {
  pub id: String,
  pub title: String,
  pub description: String,
  pub priority: u8,
  pub status: BeadStatus,
  pub depends_on: Vec<String>,
  pub blocks: Vec<String>,
}

impl Default for PlanBead {
  fn default() -> Self {
    Self {
      id: String::new(),
      title: String::new(),
      description: String::new(),
      priority: 100,
      status: BeadStatus::default(),
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
  pub phase_number: u32,
  pub name: String,
  pub description: String,
  pub beads: Vec<PlanBead>,
  pub status: PhaseStatus,
  pub blockers: Vec<String>,
}

impl Default for Phase {
  fn default() -> Self {
    Self {
      phase_number: 1,
      name: String::new(),
      description: String::new(),
      beads: Vec::new(),
      status: PhaseStatus::default(),
      blockers: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
  pub session_id: String,
  pub phases: Vec<Phase>,
  pub blockers: Vec<String>,
  pub created_at: String,
}

impl Default for ExecutionPlan {
  fn default() -> Self {
    Self {
      session_id: String::new(),
      phases: Vec::new(),
      blockers: Vec::new(),
      created_at: String::new(),
    }
  }
}
