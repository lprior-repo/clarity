//! Conflict detection for scope paradoxes and CAP-style trade-offs.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
  ScopeParadox,
  CapTheorem,
  ResourceConstraint,
  TimelineConstraint,
}

impl fmt::Display for ConflictType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ScopeParadox => write!(f, "Scope Paradox"),
      Self::CapTheorem => write!(f, "CAP Theorem Conflict"),
      Self::ResourceConstraint => write!(f, "Resource Constraint"),
      Self::TimelineConstraint => write!(f, "Timeline Constraint"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
  Low,
  Medium,
  High,
  Critical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constraint {
  pub key: String,
  pub value: String,
}

impl Constraint {
  pub fn new(key: String, value: String) -> Result<Self, ConflictDetectionError> {
    if key.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyField("key".to_string()));
    }
    if value.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyField("value".to_string()));
    }
    Ok(Self {
      key: key.trim().to_string(),
      value: value.trim().to_string(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionFrame {
  pub id: Uuid,
  pub subject: String,
  pub constraints: Vec<Constraint>,
  pub created_at: DateTime<Utc>,
}

impl DecisionFrame {
  pub fn new(
    subject: String,
    constraints: Vec<Constraint>,
  ) -> Result<Self, ConflictDetectionError> {
    if subject.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyField("subject".to_string()));
    }
    if constraints.is_empty() {
      return Err(ConflictDetectionError::NoConstraints);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      subject: subject.trim().to_string(),
      constraints,
      created_at: Utc::now(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
  pub id: Uuid,
  pub conflict_type: ConflictType,
  pub severity: ConflictSeverity,
  pub message: String,
  pub keys: Vec<String>,
  pub mitigation: String,
}

impl Conflict {
  fn new(
    conflict_type: ConflictType,
    severity: ConflictSeverity,
    message: String,
    keys: Vec<String>,
    mitigation: String,
  ) -> Result<Self, ConflictDetectionError> {
    if message.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyField("message".to_string()));
    }
    if mitigation.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyField("mitigation".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      conflict_type,
      severity,
      message,
      keys,
      mitigation,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictReport {
  pub frame_id: Uuid,
  pub subject: String,
  pub conflicts: Vec<Conflict>,
  pub generated_at: DateTime<Utc>,
}

impl ConflictReport {
  pub fn detect(frame: &DecisionFrame) -> Result<Self, ConflictDetectionError> {
    let contradictory = contradictory_value_conflicts(&frame.constraints)?;
    let cap = cap_theorem_conflict(&frame.constraints)?;
    let scope = scope_paradox_conflict(&frame.constraints)?;

    let conflicts: Vec<Conflict> = contradictory.into_iter().chain(cap).chain(scope).collect();

    Ok(Self {
      frame_id: frame.id,
      subject: frame.subject.clone(),
      conflicts,
      generated_at: Utc::now(),
    })
  }

  #[must_use]
  pub fn has_critical(&self) -> bool {
    self
      .conflicts
      .iter()
      .any(|conflict| conflict.severity == ConflictSeverity::Critical)
  }
}

fn contradictory_value_conflicts(
  constraints: &[Constraint],
) -> Result<Vec<Conflict>, ConflictDetectionError> {
  let keys: Vec<&str> = constraints
    .iter()
    .map(|constraint| constraint.key.as_str())
    .collect();

  keys
    .iter()
    .enumerate()
    .flat_map(|(left_idx, left_key)| {
      keys
        .iter()
        .enumerate()
        .filter_map(move |(right_idx, right_key)| {
          if left_idx >= right_idx || left_key != right_key {
            return None;
          }
          Some((left_idx, right_idx))
        })
    })
    .filter_map(|(left_idx, right_idx)| {
      let left = &constraints[left_idx];
      let right = &constraints[right_idx];
      (left.value != right.value).then(|| {
        Conflict::new(
          ConflictType::ResourceConstraint,
          ConflictSeverity::High,
          format!(
            "Constraint '{}' has conflicting values '{}' and '{}'",
            left.key, left.value, right.value
          ),
          vec![left.key.clone()],
          "Split requirement by context or choose one explicit value.".to_string(),
        )
      })
    })
    .collect()
}

fn cap_theorem_conflict(
  constraints: &[Constraint],
) -> Result<Vec<Conflict>, ConflictDetectionError> {
  let has_true = |key: &str| {
    constraints
      .iter()
      .any(|constraint| constraint.key == key && constraint.value.eq_ignore_ascii_case("true"))
  };

  if has_true("consistency") && has_true("availability") && has_true("partition_tolerance") {
    return Ok(vec![Conflict::new(
      ConflictType::CapTheorem,
      ConflictSeverity::Critical,
      "CAP conflict: consistency, availability, and partition_tolerance all required".to_string(),
      vec![
        "consistency".to_string(),
        "availability".to_string(),
        "partition_tolerance".to_string(),
      ],
      "Choose two CAP properties as primary and define graceful degradation.".to_string(),
    )?]);
  }

  Ok(Vec::new())
}

fn scope_paradox_conflict(
  constraints: &[Constraint],
) -> Result<Vec<Conflict>, ConflictDetectionError> {
  let has = |key: &str, value: &str| {
    constraints
      .iter()
      .any(|constraint| constraint.key == key && constraint.value.eq_ignore_ascii_case(value))
  };

  if has("scope", "small") && has("scope", "enterprise") {
    return Ok(vec![Conflict::new(
      ConflictType::ScopeParadox,
      ConflictSeverity::High,
      "Scope paradox: both 'small' and 'enterprise' requested".to_string(),
      vec!["scope".to_string()],
      "Split into phased delivery with small core and explicit enterprise extension.".to_string(),
    )?]);
  }

  Ok(Vec::new())
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ConflictDetectionError {
  #[error("field cannot be empty: {0}")]
  EmptyField(String),

  #[error("at least one constraint is required")]
  NoConstraints,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_cap_theorem_conflict() {
    let constraints_result = vec![
      Constraint::new("consistency".to_string(), "true".to_string()),
      Constraint::new("availability".to_string(), "true".to_string()),
      Constraint::new("partition_tolerance".to_string(), "true".to_string()),
    ];
    assert!(constraints_result.iter().all(Result::is_ok));
    let constraints: Vec<Constraint> = constraints_result
      .into_iter()
      .filter_map(Result::ok)
      .collect();

    let frame_result = DecisionFrame::new("distributed writes".to_string(), constraints);
    assert!(frame_result.is_ok());
    let frame = match frame_result {
      Ok(frame) => frame,
      Err(_) => return,
    };

    let report_result = ConflictReport::detect(&frame);
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    assert!(report.has_critical());
  }

  #[test]
  fn detects_scope_paradox() {
    let constraints_result = vec![
      Constraint::new("scope".to_string(), "small".to_string()),
      Constraint::new("scope".to_string(), "enterprise".to_string()),
    ];
    assert!(constraints_result.iter().all(Result::is_ok));
    let constraints: Vec<Constraint> = constraints_result
      .into_iter()
      .filter_map(Result::ok)
      .collect();

    let frame_result = DecisionFrame::new("planner v1".to_string(), constraints);
    assert!(frame_result.is_ok());
    let frame = match frame_result {
      Ok(frame) => frame,
      Err(_) => return,
    };

    let report_result = ConflictReport::detect(&frame);
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    assert!(report
      .conflicts
      .iter()
      .any(|conflict| conflict.conflict_type == ConflictType::ScopeParadox));
  }
}
