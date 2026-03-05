#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Conflict Detection module for requirements analysis.
//!
//! This module identifies contradictions and conflicts between requirements
//! to ensure consistency and feasibility.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Domain errors for conflict detection
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ConflictError {
  #[error("requirements list is empty")]
  EmptyRequirements,

  #[error("requirement text is empty at index {0}")]
  EmptyRequirement(usize),

  #[error("conflict resolution failed: {0}")]
  ResolutionFailed(String),
}

/// Types of conflicts between requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
  /// Direct logical contradiction
  Contradiction,
  /// Mutually exclusive requirements
  MutualExclusion,
  /// Resource contention
  ResourceConflict,
  /// Priority conflict
  PriorityConflict,
  /// Temporal conflict (timing incompatibility)
  TemporalConflict,
  /// Scope conflict
  ScopeConflict,
  /// Dependency conflict
  DependencyConflict,
}

impl ConflictType {
  /// Get all conflict types
  #[must_use]
  pub const fn all() -> [Self; 7] {
    [
      Self::Contradiction,
      Self::MutualExclusion,
      Self::ResourceConflict,
      Self::PriorityConflict,
      Self::TemporalConflict,
      Self::ScopeConflict,
      Self::DependencyConflict,
    ]
  }

  /// Get label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Contradiction => "Contradiction",
      Self::MutualExclusion => "Mutual Exclusion",
      Self::ResourceConflict => "Resource Conflict",
      Self::PriorityConflict => "Priority Conflict",
      Self::TemporalConflict => "Temporal Conflict",
      Self::ScopeConflict => "Scope Conflict",
      Self::DependencyConflict => "Dependency Conflict",
    }
  }

  /// Get description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Contradiction => "Requirements directly contradict each other",
      Self::MutualExclusion => "Requirements cannot both be satisfied",
      Self::ResourceConflict => "Requirements compete for same resources",
      Self::PriorityConflict => "Requirements have conflicting priorities",
      Self::TemporalConflict => "Requirements have incompatible timing",
      Self::ScopeConflict => "Requirements have overlapping/competing scopes",
      Self::DependencyConflict => "Requirements have conflicting dependencies",
    }
  }
}

/// Severity of a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ConflictSeverity {
  /// Minor inconsistency
  Low,
  /// Moderate conflict requiring resolution
  Medium,
  /// Significant conflict blocking implementation
  High,
  /// Critical conflict making requirements infeasible
  Critical,
}

impl ConflictSeverity {
  /// Convert to numeric score
  #[must_use]
  pub const fn score(&self) -> u8 {
    match self {
      Self::Low => 15,
      Self::Medium => 35,
      Self::High => 65,
      Self::Critical => 100,
    }
  }

  /// Get suggested resolution priority
  #[must_use]
  pub const fn resolution_priority(&self) -> &'static str {
    match self {
      Self::Low => "Address in next iteration",
      Self::Medium => "Resolve before finalization",
      Self::High => "Must resolve before implementation",
      Self::Critical => "Blocking - immediate resolution required",
    }
  }
}

/// A detected conflict between requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
  /// Unique identifier
  pub id: String,
  /// Conflict type
  pub conflict_type: ConflictType,
  /// Severity level
  pub severity: ConflictSeverity,
  /// First requirement involved
  pub requirement_a: String,
  /// Second requirement involved
  pub requirement_b: String,
  /// Description of the conflict
  pub description: String,
  /// Evidence/reasoning for the conflict
  pub evidence: String,
  /// Suggested resolutions
  pub resolutions: Vec<ConflictResolution>,
}

impl Conflict {
  /// Create a new conflict
  #[must_use]
  pub fn new(
    id: String,
    conflict_type: ConflictType,
    severity: ConflictSeverity,
    requirement_a: String,
    requirement_b: String,
    description: String,
  ) -> Self {
    Self {
      id,
      conflict_type,
      severity,
      requirement_a,
      requirement_b,
      description,
      evidence: String::new(),
      resolutions: Vec::new(),
    }
  }

  /// Add evidence using builder pattern
  #[must_use]
  pub fn with_evidence(mut self, evidence: String) -> Self {
    self.evidence = evidence;
    self
  }

  /// Add resolution suggestion
  #[must_use]
  pub fn with_resolution(mut self, resolution: ConflictResolution) -> Self {
    self.resolutions.push(resolution);
    self
  }

  /// Check if conflict involves a specific requirement
  #[must_use]
  pub fn involves(&self, requirement: &str) -> bool {
    self.requirement_a == requirement || self.requirement_b == requirement
  }
}

/// A suggested resolution for a conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictResolution {
  /// Resolution description
  pub description: String,
  /// Resolution strategy
  pub strategy: ResolutionStrategy,
  /// Effort required (1-5)
  pub effort: u8,
  /// Whether this resolution preserves both requirements
  pub preserves_both: bool,
}

impl ConflictResolution {
  /// Create a new resolution
  #[must_use]
  pub fn new(description: String, strategy: ResolutionStrategy, effort: u8) -> Self {
    Self {
      description,
      strategy,
      effort: effort.min(5),
      preserves_both: false,
    }
  }

  /// Set preserves_both flag
  #[must_use]
  pub fn with_preserves_both(mut self, preserves: bool) -> Self {
    self.preserves_both = preserves;
    self
  }
}

/// Resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionStrategy {
  /// Remove one of the conflicting requirements
  Remove,
  /// Modify one or both requirements
  Modify,
  /// Add a condition to make requirements compatible
  Conditional,
  /// Prioritize one requirement over the other
  Prioritize,
  /// Find a compromise between requirements
  Compromise,
  /// Split requirements into phases
  Phase,
  /// Escalate for stakeholder decision
  Escalate,
}

impl ResolutionStrategy {
  /// Get label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Remove => "Remove",
      Self::Modify => "Modify",
      Self::Conditional => "Conditional",
      Self::Prioritize => "Prioritize",
      Self::Compromise => "Compromise",
      Self::Phase => "Phase",
      Self::Escalate => "Escalate",
    }
  }

  /// Get description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Remove => "Remove one conflicting requirement",
      Self::Modify => "Modify requirements to be compatible",
      Self::Conditional => "Add conditions to make compatible",
      Self::Prioritize => "Establish priority order",
      Self::Compromise => "Find middle ground",
      Self::Phase => "Implement in different phases",
      Self::Escalate => "Escalate for decision",
    }
  }
}

/// Complete conflict analysis result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictAnalysis {
  /// All detected conflicts
  pub conflicts: Vec<Conflict>,
  /// Requirements with conflicts
  pub conflicting_requirements: Vec<String>,
  /// Requirements without conflicts
  pub clean_requirements: Vec<String>,
  /// Conflict count by type
  pub conflicts_by_type: HashMap<ConflictType, usize>,
  /// Overall consistency score (0-100)
  pub consistency_score: u8,
  /// Summary message
  pub summary: String,
}

impl ConflictAnalysis {
  /// Create new conflict analysis
  #[must_use]
  pub fn new(conflicts: Vec<Conflict>, all_requirements: Vec<String>) -> Self {
    let conflicting_requirements: Vec<String> = conflicts
      .iter()
      .flat_map(|c| vec![c.requirement_a.clone(), c.requirement_b.clone()])
      .unique()
      .collect();

    let clean_requirements: Vec<String> = all_requirements
      .iter()
      .filter(|r| !conflicting_requirements.contains(r))
      .cloned()
      .collect();

    let conflicts_by_type = conflicts
      .iter()
      .map(|c| c.conflict_type)
      .counts()
      .into_iter()
      .collect();

    let consistency_score = calculate_consistency_score(&conflicts, all_requirements.len());
    let summary = generate_conflict_summary(&conflicts, consistency_score);

    Self {
      conflicts,
      conflicting_requirements,
      clean_requirements,
      conflicts_by_type,
      consistency_score,
      summary,
    }
  }

  /// Get conflicts by type
  #[must_use]
  pub fn get_conflicts_by_type(&self, conflict_type: ConflictType) -> Vec<&Conflict> {
    self
      .conflicts
      .iter()
      .filter(|c| c.conflict_type == conflict_type)
      .collect()
  }

  /// Get conflicts by severity
  #[must_use]
  pub fn get_conflicts_by_severity(&self, severity: ConflictSeverity) -> Vec<&Conflict> {
    self
      .conflicts
      .iter()
      .filter(|c| c.severity == severity)
      .collect()
  }

  /// Get critical conflicts
  #[must_use]
  pub fn critical_conflicts(&self) -> Vec<&Conflict> {
    self.get_conflicts_by_severity(ConflictSeverity::Critical)
  }

  /// Get high severity conflicts
  #[must_use]
  pub fn high_severity_conflicts(&self) -> Vec<&Conflict> {
    self.get_conflicts_by_severity(ConflictSeverity::High)
  }

  /// Check if any critical conflicts exist
  #[must_use]
  pub fn has_critical_conflicts(&self) -> bool {
    self
      .conflicts
      .iter()
      .any(|c| c.severity == ConflictSeverity::Critical)
  }

  /// Get conflicts involving a specific requirement
  #[must_use]
  pub fn get_conflicts_for_requirement(&self, requirement: &str) -> Vec<&Conflict> {
    self
      .conflicts
      .iter()
      .filter(|c| c.involves(requirement))
      .collect()
  }

  /// Get prioritized conflicts (sorted by severity)
  #[must_use]
  pub fn prioritized_conflicts(&self) -> Vec<&Conflict> {
    self
      .conflicts
      .iter()
      .sorted_by(|a, b| b.severity.cmp(&a.severity))
      .collect()
  }

  /// Check if requirements are conflict-free
  #[must_use]
  pub fn is_conflict_free(&self) -> bool {
    self.conflicts.is_empty()
  }
}

/// Calculate consistency score
fn calculate_consistency_score(conflicts: &[Conflict], total_requirements: usize) -> u8 {
  if total_requirements == 0 {
    return 100;
  }

  let total_impact: u32 = conflicts
    .iter()
    .map(|c| u32::from(c.severity.score()))
    .sum();

  // Score decreases based on conflict severity
  let penalty = total_impact / total_requirements as u32;
  100_u8.saturating_sub(penalty as u8)
}

/// Generate conflict summary
fn generate_conflict_summary(conflicts: &[Conflict], consistency_score: u8) -> String {
  let critical = conflicts
    .iter()
    .filter(|c| c.severity == ConflictSeverity::Critical)
    .count();
  let high = conflicts
    .iter()
    .filter(|c| c.severity == ConflictSeverity::High)
    .count();
  let total = conflicts.len();

  format!(
    "Consistency: {}% | {} conflicts total ({} critical, {} high)",
    consistency_score, total, critical, high
  )
}

/// Detect conflicts in requirements
///
/// # Arguments
/// * `requirements` - List of requirement texts to analyze
///
/// # Returns
/// Complete conflict analysis with detected conflicts
#[must_use]
pub fn detect_conflicts(requirements: &[&str]) -> ConflictAnalysis {
  let all_requirements: Vec<String> = requirements.iter().map(|s| s.to_string()).collect();
  let mut conflicts = Vec::new();
  let mut conflict_id = 0;

  // Check all pairs of requirements
  for (i, req_a) in requirements.iter().enumerate() {
    for req_b in requirements.iter().skip(i + 1) {
      if let Some(conflict) = check_requirement_pair(req_a, req_b, &mut conflict_id) {
        conflicts.push(conflict);
      }
    }
  }

  ConflictAnalysis::new(conflicts, all_requirements)
}

/// Check a pair of requirements for conflicts
fn check_requirement_pair(req_a: &str, req_b: &str, conflict_id: &mut usize) -> Option<Conflict> {
  let lower_a = req_a.to_lowercase();
  let lower_b = req_b.to_lowercase();

  // Check for direct contradictions
  if let Some(conflict) = check_contradiction(req_a, req_b, &lower_a, &lower_b, conflict_id) {
    return Some(conflict);
  }

  // Check for mutual exclusion
  if let Some(conflict) = check_mutual_exclusion(req_a, req_b, &lower_a, &lower_b, conflict_id) {
    return Some(conflict);
  }

  // Check for resource conflicts
  if let Some(conflict) = check_resource_conflict(req_a, req_b, &lower_a, &lower_b, conflict_id) {
    return Some(conflict);
  }

  // Check for priority conflicts
  if let Some(conflict) = check_priority_conflict(req_a, req_b, &lower_a, &lower_b, conflict_id) {
    return Some(conflict);
  }

  None
}

/// Check for direct contradictions
fn check_contradiction(
  req_a: &str,
  req_b: &str,
  lower_a: &str,
  lower_b: &str,
  conflict_id: &mut usize,
) -> Option<Conflict> {
  let contradictions = [
    ("must", "must not"),
    ("shall", "shall not"),
    ("always", "never"),
    ("enabled", "disabled"),
    ("required", "optional"),
    ("include", "exclude"),
    ("allow", "deny"),
    ("accept", "reject"),
  ];

  for (pos, neg) in &contradictions {
    if (lower_a.contains(pos) && lower_b.contains(neg))
      || (lower_a.contains(neg) && lower_b.contains(pos))
    {
      // Check if they refer to the same concept
      if shares_concept(lower_a, lower_b) {
        *conflict_id += 1;
        return Some(
          Conflict::new(
            format!("CONFLICT-{:03}", conflict_id),
            ConflictType::Contradiction,
            ConflictSeverity::Critical,
            req_a.to_string(),
            req_b.to_string(),
            format!("Direct contradiction: '{}' vs '{}'", pos, neg),
          )
          .with_evidence(format!(
            "One requires '{}' while other requires '{}'",
            pos, neg
          ))
          .with_resolution(ConflictResolution::new(
            "Choose one requirement or add condition".to_string(),
            ResolutionStrategy::Prioritize,
            2,
          ))
          .with_resolution(ConflictResolution::new(
            "Modify one requirement to be conditional".to_string(),
            ResolutionStrategy::Conditional,
            3,
          )),
        );
      }
    }
  }

  None
}

/// Check for mutual exclusion
fn check_mutual_exclusion(
  req_a: &str,
  req_b: &str,
  lower_a: &str,
  lower_b: &str,
  conflict_id: &mut usize,
) -> Option<Conflict> {
  let exclusive_pairs: [([&str; 3], [&str; 3]); 4] = [
    (
      ["real-time", "immediate", ""],
      ["batch", "deferred", "asynchronous"],
    ),
    (["online", "connected", ""], ["offline", "disconnected", ""]),
    (["public", "shared", ""], ["private", "restricted", ""]),
    (
      ["encrypted", "secure", ""],
      ["plaintext", "unencrypted", ""],
    ),
  ];

  for (group_a, group_b) in &exclusive_pairs {
    let a_in_first = group_a.iter().any(|k| !k.is_empty() && lower_a.contains(k));
    let b_in_first = group_a.iter().any(|k| !k.is_empty() && lower_b.contains(k));
    let a_in_second = group_b.iter().any(|k| !k.is_empty() && lower_a.contains(k));
    let b_in_second = group_b.iter().any(|k| !k.is_empty() && lower_b.contains(k));

    if (a_in_first && b_in_second) || (a_in_second && b_in_first) {
      *conflict_id += 1;
      return Some(
        Conflict::new(
          format!("CONFLICT-{:03}", conflict_id),
          ConflictType::MutualExclusion,
          ConflictSeverity::High,
          req_a.to_string(),
          req_b.to_string(),
          "Mutually exclusive requirements".to_string(),
        )
        .with_evidence("Requirements cannot both be satisfied".to_string())
        .with_resolution(ConflictResolution::new(
          "Implement as configurable options".to_string(),
          ResolutionStrategy::Conditional,
          4,
        )),
      );
    }
  }

  None
}

/// Check for resource conflicts
fn check_resource_conflict(
  req_a: &str,
  req_b: &str,
  lower_a: &str,
  lower_b: &str,
  conflict_id: &mut usize,
) -> Option<Conflict> {
  // Check for competing resource demands
  let resource_patterns: [(&str, [&str; 3]); 4] = [
    ("memory", ["unlimited", "all available", "maximum"]),
    ("cpu", ["100%", "dedicated", "exclusive"]),
    ("storage", ["unlimited", "all", ""]),
    ("bandwidth", ["unlimited", "dedicated", ""]),
  ];

  for (resource, demands) in &resource_patterns {
    let a_wants_resource = lower_a.contains(resource);
    let b_wants_resource = lower_b.contains(resource);

    if a_wants_resource && b_wants_resource {
      let a_exclusive = demands.iter().any(|d| !d.is_empty() && lower_a.contains(d));
      let b_exclusive = demands.iter().any(|d| !d.is_empty() && lower_b.contains(d));

      if a_exclusive || b_exclusive {
        *conflict_id += 1;
        return Some(
          Conflict::new(
            format!("CONFLICT-{:03}", conflict_id),
            ConflictType::ResourceConflict,
            ConflictSeverity::High,
            req_a.to_string(),
            req_b.to_string(),
            format!("Resource conflict over {}", resource),
          )
          .with_evidence(format!(
            "Both requirements demand exclusive {} access",
            resource
          ))
          .with_resolution(ConflictResolution::new(
            "Define resource allocation policy".to_string(),
            ResolutionStrategy::Compromise,
            4,
          )),
        );
      }
    }
  }

  None
}

/// Check for priority conflicts
fn check_priority_conflict(
  req_a: &str,
  req_b: &str,
  lower_a: &str,
  lower_b: &str,
  conflict_id: &mut usize,
) -> Option<Conflict> {
  let priority_words = [
    "critical",
    "essential",
    "mandatory",
    "must have",
    "highest priority",
  ];

  let a_high_priority = priority_words.iter().any(|p| lower_a.contains(p));
  let b_high_priority = priority_words.iter().any(|p| lower_b.contains(p));

  if a_high_priority && b_high_priority && shares_concept(lower_a, lower_b) {
    *conflict_id += 1;
    return Some(
      Conflict::new(
        format!("CONFLICT-{:03}", conflict_id),
        ConflictType::PriorityConflict,
        ConflictSeverity::Medium,
        req_a.to_string(),
        req_b.to_string(),
        "Both requirements marked as high priority".to_string(),
      )
      .with_evidence("Cannot have multiple highest priorities".to_string())
      .with_resolution(ConflictResolution::new(
        "Establish clear priority ranking".to_string(),
        ResolutionStrategy::Prioritize,
        2,
      )),
    );
  }

  None
}

/// Check if two requirements share a common concept
fn shares_concept(lower_a: &str, lower_b: &str) -> bool {
  // Extract significant words (longer than 3 chars)
  let words_a: std::collections::HashSet<&str> =
    lower_a.split_whitespace().filter(|w| w.len() > 3).collect();

  let words_b: std::collections::HashSet<&str> =
    lower_b.split_whitespace().filter(|w| w.len() > 3).collect();

  // Check for overlapping concepts
  let common = words_a.intersection(&words_b).count();
  common >= 2
}

/// Quick conflict check between two requirements
///
/// # Arguments
/// * `req_a` - First requirement
/// * `req_b` - Second requirement
///
/// # Returns
/// True if a conflict exists
#[must_use]
pub fn has_conflict(req_a: &str, req_b: &str) -> bool {
  let mut conflict_id = 0;
  check_requirement_pair(req_a, req_b, &mut conflict_id).is_some()
}

/// Get conflict type between two requirements
///
/// # Arguments
/// * `req_a` - First requirement
/// * `req_b` - Second requirement
///
/// # Returns
/// Conflict type if a conflict exists
#[must_use]
pub fn get_conflict_type(req_a: &str, req_b: &str) -> Option<ConflictType> {
  let mut conflict_id = 0;
  check_requirement_pair(req_a, req_b, &mut conflict_id).map(|c| c.conflict_type)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_conflict_type_labels() {
    assert_eq!(ConflictType::Contradiction.label(), "Contradiction");
    assert_eq!(ConflictType::MutualExclusion.label(), "Mutual Exclusion");
  }

  #[test]
  fn test_conflict_type_descriptions() {
    for ct in ConflictType::all() {
      assert!(!ct.description().is_empty());
    }
  }

  #[test]
  fn test_conflict_severity_ordering() {
    assert!(ConflictSeverity::Critical > ConflictSeverity::High);
    assert!(ConflictSeverity::High > ConflictSeverity::Medium);
    assert!(ConflictSeverity::Medium > ConflictSeverity::Low);
  }

  #[test]
  fn test_conflict_severity_scores() {
    assert_eq!(ConflictSeverity::Low.score(), 15);
    assert_eq!(ConflictSeverity::Medium.score(), 35);
    assert_eq!(ConflictSeverity::High.score(), 65);
    assert_eq!(ConflictSeverity::Critical.score(), 100);
  }

  #[test]
  fn test_conflict_new() {
    let conflict = Conflict::new(
      "CONFLICT-001".to_string(),
      ConflictType::Contradiction,
      ConflictSeverity::Critical,
      "Req A".to_string(),
      "Req B".to_string(),
      "Test conflict".to_string(),
    );

    assert_eq!(conflict.id, "CONFLICT-001");
    assert!(conflict.resolutions.is_empty());
  }

  #[test]
  fn test_conflict_builder() {
    let conflict = Conflict::new(
      "CONFLICT-001".to_string(),
      ConflictType::Contradiction,
      ConflictSeverity::High,
      "Req A".to_string(),
      "Req B".to_string(),
      "Test".to_string(),
    )
    .with_evidence("Evidence".to_string())
    .with_resolution(ConflictResolution::new(
      "Fix it".to_string(),
      ResolutionStrategy::Modify,
      3,
    ));

    assert!(!conflict.evidence.is_empty());
    assert_eq!(conflict.resolutions.len(), 1);
  }

  #[test]
  fn test_conflict_involves() {
    let conflict = Conflict::new(
      "CONFLICT-001".to_string(),
      ConflictType::Contradiction,
      ConflictSeverity::High,
      "Req A".to_string(),
      "Req B".to_string(),
      "Test".to_string(),
    );

    assert!(conflict.involves("Req A"));
    assert!(conflict.involves("Req B"));
    assert!(!conflict.involves("Req C"));
  }

  #[test]
  fn test_conflict_resolution_new() {
    let resolution = ConflictResolution::new(
      "Test resolution".to_string(),
      ResolutionStrategy::Compromise,
      5,
    );

    assert_eq!(resolution.effort, 5);
    assert!(!resolution.preserves_both);
  }

  #[test]
  fn test_conflict_resolution_effort_capped() {
    let resolution = ConflictResolution::new("Test".to_string(), ResolutionStrategy::Modify, 10);

    assert_eq!(resolution.effort, 5); // Capped at 5
  }

  #[test]
  fn test_resolution_strategy_labels() {
    assert_eq!(ResolutionStrategy::Remove.label(), "Remove");
    assert_eq!(ResolutionStrategy::Compromise.label(), "Compromise");
  }

  #[test]
  fn test_detect_conflicts_empty() {
    let analysis = detect_conflicts(&[]);

    assert!(analysis.is_conflict_free());
    assert_eq!(analysis.consistency_score, 100);
  }

  #[test]
  fn test_detect_conflicts_no_conflicts() {
    let requirements = vec![
      "The system shall authenticate users.",
      "Data must be encrypted at rest.",
      "Response time shall be under 2 seconds.",
    ];

    let analysis = detect_conflicts(&requirements);

    assert!(analysis.is_conflict_free());
  }

  #[test]
  fn test_detect_conflicts_with_contradiction() {
    let requirements = vec![
      "The system must always be available.",
      "The system must never be available on weekends.",
    ];

    let analysis = detect_conflicts(&requirements);

    assert!(!analysis.is_conflict_free());
    assert!(analysis.has_critical_conflicts());
  }

  #[test]
  fn test_detect_conflicts_mutual_exclusion() {
    let requirements = vec![
      "Processing must be real-time.",
      "Processing shall be batch-based.",
    ];

    let analysis = detect_conflicts(&requirements);

    assert!(!analysis.is_conflict_free());
  }

  #[test]
  fn test_has_conflict_true() {
    let result = has_conflict("Must always enable", "Must never enable");
    assert!(result);
  }

  #[test]
  fn test_has_conflict_false() {
    let result = has_conflict("Must authenticate users", "Must encrypt data");
    assert!(!result);
  }

  #[test]
  fn test_get_conflict_type_contradiction() {
    let conflict_type = get_conflict_type("Must always process", "Must never process");

    assert_eq!(conflict_type, Some(ConflictType::Contradiction));
  }

  #[test]
  fn test_get_conflict_type_none() {
    let conflict_type = get_conflict_type("Authenticate users", "Encrypt data");

    assert_eq!(conflict_type, None);
  }

  #[test]
  fn test_conflict_analysis_get_by_type() {
    let conflicts = vec![
      Conflict::new(
        "C1".to_string(),
        ConflictType::Contradiction,
        ConflictSeverity::Critical,
        "A".to_string(),
        "B".to_string(),
        "Test".to_string(),
      ),
      Conflict::new(
        "C2".to_string(),
        ConflictType::MutualExclusion,
        ConflictSeverity::High,
        "C".to_string(),
        "D".to_string(),
        "Test".to_string(),
      ),
    ];

    let analysis = ConflictAnalysis::new(
      conflicts,
      vec!["A".into(), "B".into(), "C".into(), "D".into()],
    );

    let contradictions = analysis.get_conflicts_by_type(ConflictType::Contradiction);
    assert_eq!(contradictions.len(), 1);

    let exclusions = analysis.get_conflicts_by_type(ConflictType::MutualExclusion);
    assert_eq!(exclusions.len(), 1);
  }

  #[test]
  fn test_conflict_analysis_get_by_severity() {
    let conflicts = vec![
      Conflict::new(
        "C1".to_string(),
        ConflictType::Contradiction,
        ConflictSeverity::Critical,
        "A".to_string(),
        "B".to_string(),
        "Test".to_string(),
      ),
      Conflict::new(
        "C2".to_string(),
        ConflictType::MutualExclusion,
        ConflictSeverity::High,
        "C".to_string(),
        "D".to_string(),
        "Test".to_string(),
      ),
    ];

    let analysis = ConflictAnalysis::new(
      conflicts,
      vec!["A".into(), "B".into(), "C".into(), "D".into()],
    );

    let critical = analysis.get_conflicts_by_severity(ConflictSeverity::Critical);
    assert_eq!(critical.len(), 1);

    let high = analysis.get_conflicts_by_severity(ConflictSeverity::High);
    assert_eq!(high.len(), 1);
  }

  #[test]
  fn test_conflict_analysis_get_for_requirement() {
    let conflicts = vec![
      Conflict::new(
        "C1".to_string(),
        ConflictType::Contradiction,
        ConflictSeverity::Critical,
        "Req A".to_string(),
        "Req B".to_string(),
        "Test".to_string(),
      ),
      Conflict::new(
        "C2".to_string(),
        ConflictType::MutualExclusion,
        ConflictSeverity::High,
        "Req A".to_string(),
        "Req C".to_string(),
        "Test".to_string(),
      ),
    ];

    let analysis = ConflictAnalysis::new(
      conflicts,
      vec!["Req A".into(), "Req B".into(), "Req C".into()],
    );

    let conflicts_for_a = analysis.get_conflicts_for_requirement("Req A");
    assert_eq!(conflicts_for_a.len(), 2);

    let conflicts_for_b = analysis.get_conflicts_for_requirement("Req B");
    assert_eq!(conflicts_for_b.len(), 1);
  }

  #[test]
  fn test_conflict_analysis_prioritized() {
    let conflicts = vec![
      Conflict::new(
        "C1".to_string(),
        ConflictType::Contradiction,
        ConflictSeverity::Low,
        "A".to_string(),
        "B".to_string(),
        "Test".to_string(),
      ),
      Conflict::new(
        "C2".to_string(),
        ConflictType::Contradiction,
        ConflictSeverity::Critical,
        "C".to_string(),
        "D".to_string(),
        "Test".to_string(),
      ),
    ];

    let analysis = ConflictAnalysis::new(
      conflicts,
      vec!["A".into(), "B".into(), "C".into(), "D".into()],
    );

    let prioritized = analysis.prioritized_conflicts();

    assert_eq!(prioritized[0].severity, ConflictSeverity::Critical);
    assert_eq!(prioritized[1].severity, ConflictSeverity::Low);
  }

  #[test]
  fn test_conflict_analysis_conflicting_requirements() {
    let conflicts = vec![Conflict::new(
      "C1".to_string(),
      ConflictType::Contradiction,
      ConflictSeverity::Critical,
      "A".to_string(),
      "B".to_string(),
      "Test".to_string(),
    )];

    let analysis = ConflictAnalysis::new(conflicts, vec!["A".into(), "B".into(), "C".into()]);

    assert!(analysis.conflicting_requirements.contains(&"A".to_string()));
    assert!(analysis.conflicting_requirements.contains(&"B".to_string()));
    assert!(analysis.clean_requirements.contains(&"C".to_string()));
  }

  #[test]
  fn test_calculate_consistency_score() {
    let conflicts = vec![];

    let score = calculate_consistency_score(&conflicts, 10);
    assert_eq!(score, 100);

    let conflicts = vec![Conflict::new(
      "C1".to_string(),
      ConflictType::Contradiction,
      ConflictSeverity::Critical,
      "A".to_string(),
      "B".to_string(),
      "Test".to_string(),
    )];

    let score = calculate_consistency_score(&conflicts, 10);
    assert!(score < 100);
  }

  #[test]
  fn test_shares_concept_true() {
    let result = shares_concept(
      "user authentication required",
      "user authorization required",
    );
    assert!(result);
  }

  #[test]
  fn test_shares_concept_false() {
    let result = shares_concept("user authentication required", "data encryption mandatory");
    assert!(!result);
  }

  #[test]
  fn test_check_contradiction_detected() {
    let mut conflict_id = 0;
    let result = check_contradiction(
      "System must always be available",
      "System must never be available",
      "system must always be available",
      "system must never be available",
      &mut conflict_id,
    );

    assert!(result.is_some());
    let conflict = result.unwrap();
    assert_eq!(conflict.conflict_type, ConflictType::Contradiction);
    assert_eq!(conflict.severity, ConflictSeverity::Critical);
  }

  #[test]
  fn test_check_contradiction_none() {
    let mut conflict_id = 0;
    let result = check_contradiction(
      "System must authenticate users",
      "System must encrypt data",
      "system must authenticate users",
      "system must encrypt data",
      &mut conflict_id,
    );

    assert!(result.is_none());
  }

  #[test]
  fn test_check_resource_conflict_detected() {
    let mut conflict_id = 0;
    let result = check_resource_conflict(
      "System requires 100% cpu dedicated",
      "Background task needs maximum cpu",
      "system requires 100% cpu dedicated",
      "background task needs maximum cpu",
      &mut conflict_id,
    );

    assert!(result.is_some());
    let conflict = result.unwrap();
    assert_eq!(conflict.conflict_type, ConflictType::ResourceConflict);
  }
}
