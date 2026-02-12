//! Conflict Detection Module - First Principle: Constraint Satisfaction Framework
//!
//! Detects and analyzes conflicts between requirements using patterns from
//! distributed systems (CAP theorem), project management (scope/speed tradeoffs),
//! and resource allocation theory.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// CONFLICT TYPE ENUM
// ============================================================================

/// Types of conflicts that can occur between requirements
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
  /// Scope expanding while timeline shrinking (scope creep paradox)
  ScopeParadox,
  /// CAP theorem: cannot have consistency, availability, and partition tolerance
  CapTheorem,
  /// Multiple requirements competing for limited resources
  ResourceContention,
  /// Lower priority items blocking higher priority ones
  PriorityInversion,
  /// Circular or incompatible dependencies
  DependencyConflict,
}

impl fmt::Display for ConflictType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ScopeParadox => write!(f, "Scope Paradox"),
      Self::CapTheorem => write!(f, "CAP Theorem"),
      Self::ResourceContention => write!(f, "Resource Contention"),
      Self::PriorityInversion => write!(f, "Priority Inversion"),
      Self::DependencyConflict => write!(f, "Dependency Conflict"),
    }
  }
}

// ============================================================================
// SEVERITY TYPE
// ============================================================================

/// Severity level for conflicts (0.0 = low, 1.0 = critical)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Severity(f32);

impl Severity {
  /// Create a new severity from a f32 value
  ///
  /// # Errors
  /// Returns `ConflictError::InvalidSeverity` if value is not in range [0.0, 1.0]
  pub fn try_from(value: f32) -> Result<Self, ConflictError> {
    if (0.0..=1.0).contains(&value) {
      Ok(Self(value))
    } else {
      Err(ConflictError::InvalidSeverity { value })
    }
  }

  /// Get the numeric severity value
  #[must_use]
  pub const fn value(&self) -> f32 {
    self.0
  }

  /// Check if severity is low (< 0.3)
  #[must_use]
  pub fn is_low(&self) -> bool {
    self.0 < 0.3
  }

  /// Check if severity is medium (0.3 - 0.6)
  #[must_use]
  pub fn is_medium(&self) -> bool {
    (0.3..0.6).contains(&self.0)
  }

  /// Check if severity is high (0.6 - 0.9)
  #[must_use]
  pub fn is_high(&self) -> bool {
    (0.6..0.9).contains(&self.0)
  }

  /// Check if severity is critical (>= 0.9)
  #[must_use]
  pub fn is_critical(&self) -> bool {
    self.0 >= 0.9
  }
}

// ============================================================================
// CONFLICT STRUCT
// ============================================================================

/// A detected conflict between requirements
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
  /// Unique identifier for this conflict
  pub id: Uuid,
  /// Type of conflict detected
  pub conflict_type: ConflictType,
  /// Human-readable description of the conflict
  pub description: String,
  /// Severity of the conflict (0.0 - 1.0)
  pub severity: Severity,
  /// Optional hint for resolving this conflict
  pub resolution_hint: Option<String>,
  /// When this conflict was detected
  pub detected_at: DateTime<Utc>,
}

impl Conflict {
  /// Create a new conflict
  ///
  /// # Errors
  /// Returns `ConflictError::EmptyField` if description is empty or whitespace-only
  /// Returns `ConflictError::InvalidSeverity` if severity is not in [0.0, 1.0]
  pub fn new(
    conflict_type: ConflictType,
    description: String,
    severity: f32,
  ) -> Result<Self, ConflictError> {
    if description.trim().is_empty() {
      return Err(ConflictError::EmptyField {
        field: "description".to_string(),
      });
    }

    let validated_severity = Severity::try_from(severity)?;

    Ok(Self {
      id: Uuid::new_v4(),
      conflict_type,
      description,
      severity: validated_severity,
      resolution_hint: None,
      detected_at: Utc::now(),
    })
  }

  /// Add a resolution hint
  #[must_use]
  pub fn with_resolution_hint(mut self, hint: String) -> Self {
    self.resolution_hint = Some(hint);
    self
  }
}

// ============================================================================
// REQUIREMENT STRUCT
// ============================================================================

/// A requirement that can conflict with other requirements
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
  /// Unique identifier for this requirement
  pub id: String,
  /// Description of the requirement
  pub description: String,
  /// Tags for categorization and conflict detection
  pub tags: Vec<String>,
  /// Priority level (lower = higher priority)
  pub priority: Option<u32>,
  /// IDs of requirements this one depends on
  pub dependencies: Vec<String>,
}

impl Requirement {
  /// Create a new requirement
  ///
  /// # Errors
  /// Returns `ConflictError::EmptyField` if id or description is empty
  pub fn new(id: String, description: String, tags: Vec<String>) -> Result<Self, ConflictError> {
    if id.trim().is_empty() {
      return Err(ConflictError::EmptyField {
        field: "id".to_string(),
      });
    }
    if description.trim().is_empty() {
      return Err(ConflictError::EmptyField {
        field: "description".to_string(),
      });
    }

    Ok(Self {
      id,
      description,
      tags,
      priority: None,
      dependencies: Vec::new(),
    })
  }

  /// Add tags to the requirement
  #[must_use]
  pub fn with_tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  /// Set the priority level
  #[must_use]
  pub fn with_priority(mut self, priority: u32) -> Self {
    self.priority = Some(priority);
    self
  }

  /// Set the dependencies
  #[must_use]
  pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
    self.dependencies = dependencies;
    self
  }
}

// ============================================================================
// CONFLICT ANALYSIS RESULT
// ============================================================================

/// Result of analyzing requirements for conflicts
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConflictAnalysis {
  /// Unique identifier for this analysis
  pub id: Uuid,
  /// All conflicts found during analysis
  pub conflicts_found: Vec<Conflict>,
  /// Overall risk score (0.0 - 1.0)
  pub risk_score: f32,
  /// When this analysis was performed
  pub analyzed_at: DateTime<Utc>,
}

impl ConflictAnalysis {
  /// Create a new empty analysis
  #[must_use]
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      conflicts_found: Vec::new(),
      risk_score: 0.0,
      analyzed_at: Utc::now(),
    }
  }

  /// Calculate risk score based on conflicts
  ///
  /// Returns a value between 0.0 (low risk) and 1.0 (high risk)
  #[must_use]
  pub fn calculate_risk_score(&self) -> f32 {
    if self.conflicts_found.is_empty() {
      return 0.0;
    }

    let total_severity: f32 = self
      .conflicts_found
      .iter()
      .map(|c| c.severity.value())
      .sum();

    let average_severity = total_severity / self.conflicts_found.len() as f32;
    let count_penalty = (self.conflicts_found.len() as f32 * 0.05).min(0.3);

    average_severity.mul_add(0.7, count_penalty).clamp(0.0, 1.0)
  }

  /// Check if there are any critical conflicts (severity >= 0.9)
  #[must_use]
  pub fn has_critical_conflicts(&self) -> bool {
    self
      .conflicts_found
      .iter()
      .any(|c| c.severity.is_critical())
  }

  /// Get conflicts by type
  #[must_use]
  pub fn conflicts_by_type(&self, conflict_type: ConflictType) -> Vec<&Conflict> {
    self
      .conflicts_found
      .iter()
      .filter(|c| c.conflict_type == conflict_type)
      .collect()
  }

  /// Get the count of conflicts by type
  #[must_use]
  pub fn count_by_type(&self, conflict_type: ConflictType) -> usize {
    self.conflicts_by_type(conflict_type).len()
  }
}

impl Default for ConflictAnalysis {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// CONFLICT DETECTOR BUILDER
// ============================================================================

/// Builder for creating a `ConflictDetector`
#[derive(Debug, Default)]
pub struct ConflictDetectorBuilder {
  requirements: Vec<Requirement>,
}

impl ConflictDetectorBuilder {
  /// Create a new builder
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a requirement to the detector
  #[must_use]
  pub fn with_requirement(mut self, requirement: Requirement) -> Self {
    self.requirements.push(requirement);
    self
  }

  /// Add multiple requirements to the detector
  #[must_use]
  pub fn with_requirements(mut self, requirements: Vec<Requirement>) -> Self {
    self.requirements.extend(requirements);
    self
  }

  /// Build the conflict detector
  #[must_use]
  pub fn build(self) -> ConflictDetector {
    ConflictDetector {
      requirements: self.requirements,
    }
  }
}

// ============================================================================
// CONFLICT DETECTOR
// ============================================================================

/// Detects conflicts between requirements
#[derive(Debug, Clone, Default)]
pub struct ConflictDetector {
  requirements: Vec<Requirement>,
}

impl ConflictDetector {
  /// Create a new empty detector
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Get the requirements being analyzed
  #[must_use]
  pub fn requirements(&self) -> &[Requirement] {
    &self.requirements
  }

  /// Analyze all requirements for conflicts
  ///
  /// # Errors
  /// Currently never returns an error, but returns Result for API consistency
  /// and future extensibility
  pub fn analyze(&self) -> Result<ConflictAnalysis, ConflictError> {
    let mut analysis = ConflictAnalysis::new();

    // Detect each type of conflict
    self.detect_scope_paradoxes(&mut analysis);
    self.detect_cap_theorem_conflicts(&mut analysis);
    self.detect_resource_contention(&mut analysis);
    self.detect_priority_inversions(&mut analysis);
    self.detect_dependency_conflicts(&mut analysis);

    // Calculate final risk score
    analysis.risk_score = analysis.calculate_risk_score();

    Ok(analysis)
  }

  /// Detect scope paradoxes: requirements that expand scope while demanding speed
  fn detect_scope_paradoxes(&self, analysis: &mut ConflictAnalysis) {
    let scope_keywords = ["comprehensive", "complete", "full", "extensive", "all"];
    let speed_keywords = ["minimal", "fast", "quick", "rapid", "immediate", "sprint"];

    let has_scope_expansion = self.requirements.iter().any(|r| {
      scope_keywords
        .iter()
        .any(|kw| r.description.to_lowercase().contains(kw))
    });

    let has_speed_demand = self.requirements.iter().any(|r| {
      speed_keywords
        .iter()
        .any(|kw| r.description.to_lowercase().contains(kw))
    });

    if has_scope_expansion && has_speed_demand {
      if let Ok(conflict) = Conflict::new(
        ConflictType::ScopeParadox,
        "Requirements demand comprehensive scope while requesting minimal timeline".to_string(),
        0.6,
      )
      .map(|c| c.with_resolution_hint("Consider phased delivery or reduced scope".to_string()))
      {
        analysis.conflicts_found.push(conflict);
      }
    }
  }

  /// Detect CAP theorem conflicts in distributed system requirements
  fn detect_cap_theorem_conflicts(&self, analysis: &mut ConflictAnalysis) {
    let has_consistency = self.requirements.iter().any(|r| {
      r.description.to_lowercase().contains("consistency")
        || r.tags.iter().any(|t| t == "consistency")
    });

    let has_availability = self.requirements.iter().any(|r| {
      r.description.to_lowercase().contains("availability")
        || r.description.to_lowercase().contains("uptime")
        || r.tags.iter().any(|t| t == "availability")
    });

    let has_partition_tolerance = self.requirements.iter().any(|r| {
      r.description.to_lowercase().contains("partition")
        || r.description.to_lowercase().contains("network")
        || r.tags.iter().any(|t| t.contains("partition"))
    });

    if has_consistency && has_availability && has_partition_tolerance {
      if let Ok(conflict) = Conflict::new(
        ConflictType::CapTheorem,
        "Distributed system requirements demand all three CAP properties which is impossible"
          .to_string(),
        0.85,
      )
      .map(|c| {
        c.with_resolution_hint(
          "Choose two of: Consistency, Availability, Partition Tolerance".to_string(),
        )
      }) {
        analysis.conflicts_found.push(conflict);
      }
    }
  }

  /// Detect resource contention between requirements
  fn detect_resource_contention(&self, analysis: &mut ConflictAnalysis) {
    let resource_keywords = [
      "cpu",
      "memory",
      "real-time",
      "realtime",
      "batch",
      "intensive",
      "heavy",
    ];

    let resource_heavy: Vec<&Requirement> = self
      .requirements
      .iter()
      .filter(|r| {
        resource_keywords
          .iter()
          .any(|kw| r.description.to_lowercase().contains(kw))
          || r
            .tags
            .iter()
            .any(|t| resource_keywords.contains(&t.as_str()))
      })
      .collect();

    if resource_heavy.len() >= 2 {
      // Check for conflicting resource needs
      let has_realtime = resource_heavy.iter().any(|r| {
        r.description.to_lowercase().contains("real-time")
          || r.description.to_lowercase().contains("realtime")
      });
      let has_batch = resource_heavy
        .iter()
        .any(|r| r.description.to_lowercase().contains("batch"));

      if has_realtime && has_batch {
        if let Ok(conflict) = Conflict::new(
          ConflictType::ResourceContention,
          "Real-time and batch processing requirements compete for same resources".to_string(),
          0.5,
        )
        .map(|c| {
          c.with_resolution_hint(
            "Consider resource isolation, scheduling, or separate infrastructure".to_string(),
          )
        }) {
          analysis.conflicts_found.push(conflict);
        }
      }
    }
  }

  /// Detect priority inversions where lower priority blocks higher
  fn detect_priority_inversions(&self, analysis: &mut ConflictAnalysis) {
    let requirements_with_priority: Vec<&Requirement> = self
      .requirements
      .iter()
      .filter(|r| r.priority.is_some())
      .collect();

    for req in &requirements_with_priority {
      let req_priority = req.priority;
      for dep_id in &req.dependencies {
        // Find the dependency
        let dependency = self.requirements.iter().find(|r| &r.id == dep_id);

        if let (Some(dep), Some(req_prio)) = (dependency, req_priority) {
          if let Some(dep_prio) = dep.priority {
            // If dependency has lower priority (higher number) than the requirement
            if dep_prio > req_prio {
              if let Ok(conflict) = Conflict::new(
                ConflictType::PriorityInversion,
                format!(
                  "High-priority requirement '{}' blocked by lower-priority dependency '{}'",
                  req.id, dep.id
                ),
                0.7,
              )
              .map(|c| {
                c.with_resolution_hint(
                  "Elevate dependency priority or find alternative path".to_string(),
                )
              }) {
                analysis.conflicts_found.push(conflict);
              }
            }
          }
        }
      }
    }
  }

  /// Detect dependency conflicts including circular dependencies
  fn detect_dependency_conflicts(&self, analysis: &mut ConflictAnalysis) {
    // Check for circular dependencies using depth-first search
    for req in &self.requirements {
      if self.has_circular_dependency(&req.id, &req.id, 0) {
        if let Ok(conflict) = Conflict::new(
          ConflictType::DependencyConflict,
          format!(
            "Circular dependency detected involving requirement '{}'",
            req.id
          ),
          0.9,
        )
        .map(|c| {
          c.with_resolution_hint(
            "Break the circular dependency by extracting shared functionality".to_string(),
          )
        }) {
          analysis.conflicts_found.push(conflict);
        }
        break; // Only report once per cycle
      }
    }
  }

  /// Helper to detect circular dependencies
  fn has_circular_dependency(&self, start_id: &str, current_id: &str, depth: usize) -> bool {
    // Prevent infinite recursion
    if depth > 100 {
      return false;
    }

    let current_req = self.requirements.iter().find(|r| r.id == current_id);

    if let Some(req) = current_req {
      for dep_id in &req.dependencies {
        if dep_id == start_id && depth > 0 {
          return true;
        }
        if self.has_circular_dependency(start_id, dep_id, depth + 1) {
          return true;
        }
      }
    }

    false
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for the conflict detection module
#[derive(Debug, Error, PartialEq)]
pub enum ConflictError {
  /// A required field was empty
  #[error("required field is empty: {field}")]
  EmptyField { field: String },

  /// Invalid severity value
  #[error("invalid severity value: {value}. Must be between 0.0 and 1.0")]
  InvalidSeverity { value: f32 },

  /// Validation failed
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

// ============================================================================
// INTERNAL TESTS
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_conflict_types_have_display() {
    let types = [
      ConflictType::ScopeParadox,
      ConflictType::CapTheorem,
      ConflictType::ResourceContention,
      ConflictType::PriorityInversion,
      ConflictType::DependencyConflict,
    ];

    for conflict_type in types {
      let display = conflict_type.to_string();
      assert!(!display.is_empty());
    }
  }
}
