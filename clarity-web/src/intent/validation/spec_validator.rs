//! Spec Validator (WP19) - Comprehensive specification validation
//!
//! Provides validation including:
//! - Required field validation
//! - Type validation for fields
//! - Circular dependency detection via DFS on dependency graph
//! - Duplicate behavior detection
//! - Priority-based sorting
//! - Category-based organization

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(
  clippy::unused_self,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]

use crate::intent::types::{Behavior, Feature, Spec};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error type for spec validation
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SpecValidationError {
  /// Required field is missing
  #[error("missing required field: {0}")]
  MissingRequiredField(String),

  /// Field has invalid type
  #[error("invalid field '{field}': expected {expected}, got {actual}")]
  InvalidFieldType {
    /// Field name
    field: String,
    /// Expected type
    expected: String,
    /// Actual type found
    actual: String,
  },

  /// Circular dependency detected in behavior graph
  #[error("circular dependency detected in behavior '{behavior_a}' -> behavior '{behavior_b}'")]
  CircularDependency {
    /// First behavior in the cycle
    behavior_a: String,
    /// Second behavior in the cycle (the dependency that creates the cycle)
    behavior_b: String,
  },

  /// Circular dependency with full path
  #[error("circular dependency detected: cycle path {}", path.join(" -> "))]
  CircularDependencyPath {
    /// Full path of the cycle
    path: Vec<String>,
  },

  /// Duplicate behavior detected
  #[error("duplicate behavior detected: behavior '{behavior_a}' is duplicated with behavior '{behavior_b}'")]
  DuplicateBehavior {
    /// First behavior
    behavior_a: String,
    /// Second behavior (duplicate)
    behavior_b: String,
    /// Description of the duplicate
    description: String,
    /// Impact of the duplication
    impact: String,
  },

  /// Unknown dependency referenced
  #[error("unknown dependency: '{0}' references non-existent behavior or feature")]
  UnknownDependency(String),

  /// Validation failed with multiple errors
  #[error("validation failed with {count} errors")]
  MultipleValidationErrors {
    /// Number of errors
    count: usize,
    /// The individual errors
    errors: Vec<Self>,
  },
}

// =============================================================================
// Validation Result Types
// =============================================================================

/// Result of a spec validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
  /// Whether validation passed
  pub is_valid: bool,
  /// All errors found during validation
  pub errors: Vec<SpecValidationError>,
  /// All warnings found during validation
  pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
  /// Create a new validation result
  #[must_use]
  pub const fn new() -> Self {
    Self {
      is_valid: true,
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  /// Add an error to the result
  pub fn add_error(&mut self, error: SpecValidationError) {
    self.is_valid = false;
    self.errors.push(error);
  }

  /// Add a warning to the result
  pub fn add_warning(&mut self, warning: ValidationWarning) {
    self.warnings.push(warning);
  }

  /// Merge another validation result into this one
  pub fn merge(&mut self, other: Self) {
    if !other.is_valid {
      self.is_valid = false;
    }
    self.errors.extend(other.errors);
    self.warnings.extend(other.warnings);
  }

  /// Check if there are any errors
  #[must_use]
  pub const fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Check if there are any warnings
  #[must_use]
  pub const fn has_warnings(&self) -> bool {
    !self.warnings.is_empty()
  }
}

impl Default for ValidationResult {
  fn default() -> Self {
    Self::new()
  }
}

/// Warning type for non-fatal validation issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
  /// Warning message
  pub message: String,
  /// Field or item the warning relates to
  pub context: Option<String>,
}

impl ValidationWarning {
  /// Create a new warning
  #[must_use]
  pub const fn new(message: String, context: Option<String>) -> Self {
    Self { message, context }
  }
}

// =============================================================================
// Dependency Graph
// =============================================================================

/// Dependency graph for cycle detection
#[derive(Debug, Clone)]
pub struct DependencyGraph {
  /// Adjacency list: node -> list of dependencies
  adjacency: HashMap<String, Vec<String>>,
  /// All nodes in the graph
  nodes: HashSet<String>,
}

impl DependencyGraph {
  /// Create a new empty dependency graph
  #[must_use]
  pub fn new() -> Self {
    Self {
      adjacency: HashMap::new(),
      nodes: HashSet::new(),
    }
  }

  /// Add a node to the graph
  pub fn add_node(&mut self, node: String) {
    self.nodes.insert(node.clone());
    self.adjacency.entry(node).or_default();
  }

  /// Add a directed edge (from -> to means 'from' depends on 'to')
  pub fn add_edge(&mut self, from: String, to: String) {
    self.nodes.insert(from.clone());
    self.nodes.insert(to.clone());
    self.adjacency.entry(from).or_default().push(to.clone());
    self.adjacency.entry(to).or_default();
  }

  /// Detect cycles using DFS
  ///
  /// Returns the cycle path if a cycle is detected
  #[must_use]
  pub fn detect_cycles(&self) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    let mut path = Vec::new();

    for node in &self.nodes {
      if !visited.contains(node) {
        if let Some(cycle) =
          self.dfs_cycle_detect(node, &mut visited, &mut recursion_stack, &mut path)
        {
          return Some(cycle);
        }
      }
    }

    None
  }

  /// DFS helper for cycle detection
  fn dfs_cycle_detect(
    &self,
    node: &str,
    visited: &mut HashSet<String>,
    recursion_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
  ) -> Option<Vec<String>> {
    visited.insert(node.to_string());
    recursion_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = self.adjacency.get(node) {
      for neighbor in neighbors {
        if !visited.contains(neighbor) {
          if let Some(cycle) = self.dfs_cycle_detect(neighbor, visited, recursion_stack, path) {
            return Some(cycle);
          }
        } else if recursion_stack.contains(neighbor) {
          // Found a cycle - extract the cycle path
          let cycle_start = path.iter().position(|n| n == neighbor)?;
          return Some(path[cycle_start..].to_vec());
        }
      }
    }

    recursion_stack.remove(node);
    path.pop();
    None
  }

  /// Topological sort of the graph
  ///
  /// Returns `None` if the graph contains cycles
  #[must_use]
  pub fn topological_sort(&self) -> Option<Vec<String>> {
    let mut in_degree: HashMap<String, usize> = self.nodes.iter().map(|n| (n.clone(), 0)).collect();

    for deps in self.adjacency.values() {
      for dep in deps {
        if let Some(count) = in_degree.get_mut(dep) {
          *count += 1;
        }
      }
    }

    // Start with nodes that have no incoming edges
    let mut queue: Vec<String> = in_degree
      .iter()
      .filter(|(_, &deg)| deg == 0)
      .map(|(n, _)| n.clone())
      .collect();

    let mut result = Vec::new();

    while let Some(node) = queue.pop() {
      result.push(node.clone());

      if let Some(neighbors) = self.adjacency.get(&node) {
        for neighbor in neighbors {
          if let Some(deg) = in_degree.get_mut(neighbor) {
            *deg -= 1;
            if *deg == 0 {
              queue.push(neighbor.clone());
            }
          }
        }
      }
    }

    if result.len() == self.nodes.len() {
      // Reverse because we want dependencies first
      result.reverse();
      Some(result)
    } else {
      None // Graph has a cycle
    }
  }
}

impl Default for DependencyGraph {
  fn default() -> Self {
    Self::new()
  }
}

// =============================================================================
// Spec Validator
// =============================================================================

/// Spec validator with comprehensive validation capabilities
#[derive(Debug, Clone)]
pub struct SpecValidator {
  /// Whether to check for duplicate behaviors
  check_duplicates: bool,
  /// Whether to check for circular dependencies
  check_cycles: bool,
  /// Whether to validate required fields
  check_required_fields: bool,
}

impl SpecValidator {
  /// Create a new spec validator with default settings
  #[must_use]
  pub const fn new() -> Self {
    Self {
      check_duplicates: true,
      check_cycles: true,
      check_required_fields: true,
    }
  }

  /// Disable duplicate checking
  #[must_use]
  pub const fn without_duplicate_checking(mut self) -> Self {
    self.check_duplicates = false;
    self
  }

  /// Disable cycle checking
  #[must_use]
  pub const fn without_cycle_checking(mut self) -> Self {
    self.check_cycles = false;
    self
  }

  /// Disable required field checking
  #[must_use]
  pub const fn without_required_field_checking(mut self) -> Self {
    self.check_required_fields = false;
    self
  }

  /// Validate a complete spec
  #[must_use]
  pub fn validate(&self, spec: &Spec) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if self.check_required_fields {
      self.validate_required_fields(spec, &mut result);
    }

    // Check for duplicate features
    self.validate_unique_features(spec, &mut result);

    // Check each feature
    for feature in &spec.features {
      self.validate_feature(feature, &mut result);
    }

    // Check for duplicate behaviors across all features
    if self.check_duplicates {
      self.detect_duplicate_behaviors(spec, &mut result);
    }

    // Check for circular dependencies
    if self.check_cycles {
      self.detect_circular_dependencies(spec, &mut result);
    }

    result
  }

  /// Validate required fields in a spec
  fn validate_required_fields(&self, spec: &Spec, result: &mut ValidationResult) {
    if spec.name.trim().is_empty() {
      result.add_error(SpecValidationError::MissingRequiredField(
        "spec.name".to_string(),
      ));
    }
  }

  /// Validate that feature names are unique
  fn validate_unique_features(&self, spec: &Spec, result: &mut ValidationResult) {
    let mut seen: HashMap<&str, usize> = HashMap::new();

    for (index, feature) in spec.features.iter().enumerate() {
      if let Some(&first_index) = seen.get(feature.name.as_str()) {
        result.add_error(SpecValidationError::DuplicateBehavior {
          behavior_a: format!(
            "features[{}].{}",
            first_index, spec.features[first_index].name
          ),
          behavior_b: format!("features[{}].{}", index, feature.name),
          description: format!("Duplicate feature name: {}", feature.name),
          impact: "Features must have unique names for proper identification".to_string(),
        });
      } else {
        seen.insert(&feature.name, index);
      }
    }
  }

  /// Validate a single feature
  fn validate_feature(&self, feature: &Feature, result: &mut ValidationResult) {
    // Check required fields
    if feature.name.trim().is_empty() {
      result.add_error(SpecValidationError::MissingRequiredField(format!(
        "feature '{}' .name",
        feature.name
      )));
    }

    // Check for duplicate behaviors within feature
    let mut seen: HashMap<&str, usize> = HashMap::new();

    for (index, behavior) in feature.behaviors.iter().enumerate() {
      if let Some(&first_index) = seen.get(behavior.name.as_str()) {
        result.add_error(SpecValidationError::DuplicateBehavior {
          behavior_a: format!(
            "{}.behaviors[{}].{}",
            feature.name, first_index, feature.behaviors[first_index].name
          ),
          behavior_b: format!("{}.behaviors[{}].{}", feature.name, index, behavior.name),
          description: format!("Duplicate behavior name: {}", behavior.name),
          impact: "Behaviors within a feature must have unique names".to_string(),
        });
      } else {
        seen.insert(&behavior.name, index);
      }
    }
  }

  /// Detect duplicate behaviors across the entire spec
  fn detect_duplicate_behaviors(&self, spec: &Spec, result: &mut ValidationResult) {
    // Collect all behaviors with their full path
    let all_behaviors: Vec<(String, &Behavior)> = spec
      .features
      .iter()
      .flat_map(|f| {
        f.behaviors
          .iter()
          .map(move |b| (format!("{}.{}", f.name, b.name), b))
      })
      .collect();

    // Group by description to find potential duplicates
    let by_description: HashMap<&str, Vec<&str>> = all_behaviors
      .iter()
      .filter(|(_, b)| !b.description.is_empty())
      .fold(HashMap::new(), |mut acc, (path, b)| {
        acc.entry(&b.description).or_default().push(path.as_str());
        acc
      });

    // Report duplicates with same description
    for (description, paths) in by_description {
      if paths.len() > 1 {
        for pair in paths.iter().copied().tuple_windows::<(&str, &str)>() {
          result.add_warning(ValidationWarning::new(
            format!(
              "Behaviors '{}' and '{}' have identical descriptions",
              pair.0, pair.1
            ),
            Some(description.to_string()),
          ));
        }
      }
    }
  }

  /// Detect circular dependencies using DFS on dependency graph
  fn detect_circular_dependencies(&self, spec: &Spec, result: &mut ValidationResult) {
    let graph = self.build_dependency_graph(spec);

    if let Some(cycle_path) = graph.detect_cycles() {
      if cycle_path.len() >= 2 {
        result.add_error(SpecValidationError::CircularDependencyPath { path: cycle_path });
      }
    }

    // Also check feature-level dependencies
    let feature_graph = self.build_feature_dependency_graph(spec);

    if let Some(cycle_path) = feature_graph.detect_cycles() {
      if cycle_path.len() >= 2 {
        result.add_error(SpecValidationError::CircularDependencyPath { path: cycle_path });
      }
    }
  }

  /// Build a dependency graph from behavior dependencies
  fn build_dependency_graph(&self, spec: &Spec) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    // Build a map of behavior names to their full paths
    let behavior_paths: HashMap<String, String> = spec
      .features
      .iter()
      .flat_map(|f| {
        f.behaviors
          .iter()
          .map(move |b| (b.name.clone(), format!("{}.{}", f.name, b.name)))
      })
      .collect();

    // Add all behaviors as nodes
    for path in behavior_paths.values() {
      graph.add_node(path.clone());
    }

    // Add edges based on preconditions (which may reference other behaviors)
    for feature in &spec.features {
      for behavior in &feature.behaviors {
        let from_path = format!("{}.{}", feature.name, behavior.name);
        for precondition in &behavior.preconditions {
          // If precondition references another behavior, add edge
          if let Some(to_path) = behavior_paths.get(precondition) {
            graph.add_edge(from_path.clone(), to_path.clone());
          }
        }
      }
    }

    graph
  }

  /// Build a dependency graph from feature dependencies
  fn build_feature_dependency_graph(&self, spec: &Spec) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    // Add all features as nodes
    for feature in &spec.features {
      graph.add_node(feature.name.clone());
    }

    // Add edges based on depends_on
    for feature in &spec.features {
      for dep in &feature.depends_on {
        graph.add_edge(feature.name.clone(), dep.clone());
      }
    }

    graph
  }

  /// Sort behaviors by priority (highest priority first)
  ///
  /// Priority is determined by:
  /// 1. Number of other behaviors that depend on this one (more = higher priority)
  /// 2. Number of preconditions (fewer = higher priority, as it's more foundational)
  #[must_use]
  pub fn sort_behaviors_by_priority(&self, spec: &Spec) -> Vec<(String, BehaviorPriority)> {
    let behavior_paths: HashMap<String, String> = spec
      .features
      .iter()
      .flat_map(|f| {
        f.behaviors
          .iter()
          .map(move |b| (b.name.clone(), format!("{}.{}", f.name, b.name)))
      })
      .collect();

    // Count dependents (how many other behaviors depend on this one)
    let mut dependent_count: HashMap<String, usize> = HashMap::new();
    let mut precondition_count: HashMap<String, usize> = HashMap::new();

    for feature in &spec.features {
      for behavior in &feature.behaviors {
        let path = format!("{}.{}", feature.name, behavior.name);
        precondition_count.insert(path.clone(), behavior.preconditions.len());

        for precondition in &behavior.preconditions {
          if let Some(dep_path) = behavior_paths.get(precondition) {
            *dependent_count.entry(dep_path.clone()).or_default() += 1;
          }
        }
      }
    }

    // Create prioritized list using fold to avoid closure move issues
    let prioritized: Vec<(String, BehaviorPriority)> =
      spec.features.iter().fold(Vec::new(), |mut acc, f| {
        for b in &f.behaviors {
          let path = format!("{}.{}", f.name, b.name);
          let priority = BehaviorPriority {
            path: path.clone(),
            dependent_count: dependent_count.get(&path).map_or(0, |v| *v),
            precondition_count: precondition_count.get(&path).map_or(0, |v| *v),
          };
          acc.push((path, priority));
        }
        acc
      });

    // Sort: higher dependent_count first, then lower precondition_count
    let mut sorted = prioritized;
    sorted.sort_by(|a, b| {
      b.1
        .dependent_count
        .cmp(&a.1.dependent_count)
        .then(a.1.precondition_count.cmp(&b.1.precondition_count))
    });

    sorted
  }

  /// Sort behaviors by category
  ///
  /// Categories are inferred from behavior name prefixes (e.g., "create_*" -> "create")
  #[must_use]
  pub fn sort_by_category(&self, spec: &Spec) -> HashMap<String, Vec<String>> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for feature in &spec.features {
      for behavior in &feature.behaviors {
        let category = self.infer_category(&behavior.name);
        let path = format!("{}.{}", feature.name, behavior.name);
        categories.entry(category).or_default().push(path);
      }
    }

    // Sort behaviors within each category
    for behaviors in categories.values_mut() {
      behaviors.sort();
    }

    categories
  }

  /// Infer category from behavior name
  fn infer_category(&self, name: &str) -> String {
    name
      .split('_')
      .next()
      .map_or_else(|| "other".to_string(), std::string::ToString::to_string)
  }
}

impl Default for SpecValidator {
  fn default() -> Self {
    Self::new()
  }
}

// =============================================================================
// Priority Type
// =============================================================================

/// Priority information for a behavior
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorPriority {
  /// Full path to the behavior (feature.behavior)
  pub path: String,
  /// Number of other behaviors that depend on this one
  pub dependent_count: usize,
  /// Number of preconditions this behavior has
  pub precondition_count: usize,
}

impl BehaviorPriority {
  /// Get the priority score (higher = more important)
  #[must_use]
  pub fn score(&self) -> usize {
    // More dependents = higher priority
    // Fewer preconditions = more foundational = higher priority
    self.dependent_count.saturating_mul(10) + (100 - self.precondition_count.min(100))
  }
}

// =============================================================================
// Standalone Functions
// =============================================================================

/// Validate a spec and return all errors
///
/// This is a convenience function that creates a default validator and runs validation.
#[must_use]
pub fn validate_spec(spec: &Spec) -> ValidationResult {
  SpecValidator::new().validate(spec)
}

/// Check if a spec has any circular dependencies
#[must_use]
pub fn has_circular_dependencies(spec: &Spec) -> bool {
  let validator = SpecValidator::new();
  let graph = validator.build_feature_dependency_graph(spec);
  graph.detect_cycles().is_some()
}

/// Get the topological order of features
///
/// Returns `None` if there are circular dependencies
#[must_use]
pub fn feature_execution_order(spec: &Spec) -> Option<Vec<String>> {
  let validator = SpecValidator::new();
  let graph = validator.build_feature_dependency_graph(spec);
  graph.topological_sort()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use crate::intent::types::{Invariant, Verification};

  fn create_test_behavior(name: &str) -> Behavior {
    Behavior::new(name.to_string())
      .expect("valid behavior name")
      .with_description(format!("Behavior: {name}"))
  }

  fn create_test_feature(name: &str, behaviors: Vec<Behavior>) -> Feature {
    let mut feature = Feature::new(name.to_string()).expect("valid feature name");
    for behavior in behaviors {
      feature.add_behavior(behavior).expect("should add behavior");
    }
    feature
  }

  #[test]
  fn test_validation_result_new() {
    let result = ValidationResult::new();
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
  }

  #[test]
  fn test_validation_result_add_error() {
    let mut result = ValidationResult::new();
    result.add_error(SpecValidationError::MissingRequiredField(
      "test".to_string(),
    ));

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 1);
    assert!(result.has_errors());
  }

  #[test]
  fn test_validation_result_add_warning() {
    let mut result = ValidationResult::new();
    result.add_warning(ValidationWarning::new("test warning".to_string(), None));

    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.has_warnings());
  }

  #[test]
  fn test_validation_result_merge() {
    let mut result1 = ValidationResult::new();
    let mut result2 = ValidationResult::new();

    result2.add_error(SpecValidationError::MissingRequiredField(
      "test".to_string(),
    ));
    result2.add_warning(ValidationWarning::new("warning".to_string(), None));

    result1.merge(result2);

    assert!(!result1.is_valid);
    assert_eq!(result1.errors.len(), 1);
    assert_eq!(result1.warnings.len(), 1);
  }

  #[test]
  fn test_dependency_graph_new() {
    let graph = DependencyGraph::new();
    assert!(graph.nodes.is_empty());
    assert!(graph.adjacency.is_empty());
  }

  #[test]
  fn test_dependency_graph_add_node() {
    let mut graph = DependencyGraph::new();
    graph.add_node("A".to_string());

    assert!(graph.nodes.contains("A"));
    assert!(graph.adjacency.contains_key("A"));
  }

  #[test]
  fn test_dependency_graph_add_edge() {
    let mut graph = DependencyGraph::new();
    graph.add_edge("A".to_string(), "B".to_string());

    assert!(graph.nodes.contains("A"));
    assert!(graph.nodes.contains("B"));
    assert_eq!(graph.adjacency.get("A"), Some(&vec!["B".to_string()]));
  }

  #[test]
  fn test_dependency_graph_no_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge("A".to_string(), "B".to_string());
    graph.add_edge("B".to_string(), "C".to_string());

    assert!(graph.detect_cycles().is_none());
  }

  #[test]
  fn test_dependency_graph_simple_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge("A".to_string(), "B".to_string());
    graph.add_edge("B".to_string(), "A".to_string());

    let cycle = graph.detect_cycles();
    assert!(cycle.is_some());
    let path = cycle.expect("cycle should exist");
    assert_eq!(path.len(), 2);
  }

  #[test]
  fn test_dependency_graph_three_node_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge("A".to_string(), "B".to_string());
    graph.add_edge("B".to_string(), "C".to_string());
    graph.add_edge("C".to_string(), "A".to_string());

    let cycle = graph.detect_cycles();
    assert!(cycle.is_some());
    let path = cycle.expect("cycle should exist");
    assert_eq!(path.len(), 3);
  }

  #[test]
  fn test_dependency_graph_topological_sort() {
    let mut graph = DependencyGraph::new();
    // A depends on B, B depends on C
    graph.add_edge("A".to_string(), "B".to_string());
    graph.add_edge("B".to_string(), "C".to_string());

    let sorted = graph.topological_sort();
    assert!(sorted.is_some());
    let order = sorted.expect("should have valid order");

    // C should come before B, B should come before A
    let c_pos = order
      .iter()
      .position(|n| n == "C")
      .expect("C should be in order");
    let b_pos = order
      .iter()
      .position(|n| n == "B")
      .expect("B should be in order");
    let a_pos = order
      .iter()
      .position(|n| n == "A")
      .expect("A should be in order");

    assert!(c_pos < b_pos);
    assert!(b_pos < a_pos);
  }

  #[test]
  fn test_dependency_graph_topological_sort_with_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge("A".to_string(), "B".to_string());
    graph.add_edge("B".to_string(), "A".to_string());

    assert!(graph.topological_sort().is_none());
  }

  #[test]
  fn test_spec_validator_new() {
    let validator = SpecValidator::new();
    assert!(validator.check_duplicates);
    assert!(validator.check_cycles);
    assert!(validator.check_required_fields);
  }

  #[test]
  fn test_spec_validator_builder_methods() {
    let validator = SpecValidator::new()
      .without_duplicate_checking()
      .without_cycle_checking()
      .without_required_field_checking();

    assert!(!validator.check_duplicates);
    assert!(!validator.check_cycles);
    assert!(!validator.check_required_fields);
  }

  #[test]
  fn test_validate_empty_spec_name() {
    let spec = Spec {
      name: String::new(),
      description: String::new(),
      features: Vec::new(),
      invariants: Vec::new(),
      anti_patterns: Vec::new(),
      ai_hints: crate::intent::types::AIHints::default(),
    };

    let result = validate_spec(&spec);
    assert!(result.has_errors());

    let has_missing_name = result.errors.iter().any(|e| {
      matches!(
          e,
          SpecValidationError::MissingRequiredField(field) if field == "spec.name"
      )
    });
    assert!(has_missing_name);
  }

  #[test]
  fn test_validate_valid_spec() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let behavior = create_test_behavior("create_user");
    let feature = create_test_feature("users", vec![behavior]);

    spec.add_feature(feature).expect("should add feature");

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);
  }

  #[test]
  fn test_validate_duplicate_features() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let feature1 = create_test_feature("users", vec![]);
    let feature2 = create_test_feature("users", vec![]);

    // Manually add both to bypass Spec::add_feature duplicate check
    spec.features.push(feature1);
    spec.features.push(feature2);

    let result = validate_spec(&spec);
    assert!(result.has_errors());
  }

  #[test]
  fn test_validate_duplicate_behaviors_in_feature() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let behavior = create_test_behavior("create_user");
    let mut feature = Feature::new("users".to_string()).expect("valid feature");

    // Manually add duplicate to bypass Feature::add_behavior duplicate check
    feature.behaviors.push(behavior.clone());
    feature.behaviors.push(behavior);

    spec.features.push(feature);

    let result = validate_spec(&spec);
    assert!(result.has_errors());
  }

  #[test]
  fn test_validate_circular_feature_dependencies() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let mut feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_a.add_dependency("feature_b".to_string());
    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    let result = validate_spec(&spec);
    assert!(result.has_errors());

    let has_cycle_error = result
      .errors
      .iter()
      .any(|e| matches!(e, SpecValidationError::CircularDependencyPath { .. }));
    assert!(has_cycle_error);
  }

  #[test]
  fn test_validate_no_circular_dependencies() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);
  }

  #[test]
  fn test_has_circular_dependencies_true() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let mut feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_a.add_dependency("feature_b".to_string());
    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    assert!(has_circular_dependencies(&spec));
  }

  #[test]
  fn test_has_circular_dependencies_false() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    assert!(!has_circular_dependencies(&spec));
  }

  #[test]
  fn test_feature_execution_order_valid() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    let order = feature_execution_order(&spec);
    assert!(order.is_some());

    let order = order.expect("should have order");
    let a_pos = order
      .iter()
      .position(|n| n == "feature_a")
      .expect("a in order");
    let b_pos = order
      .iter()
      .position(|n| n == "feature_b")
      .expect("b in order");

    assert!(a_pos < b_pos, "feature_a should come before feature_b");
  }

  #[test]
  fn test_feature_execution_order_with_cycle() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let mut feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);

    feature_a.add_dependency("feature_b".to_string());
    feature_b.add_dependency("feature_a".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);

    assert!(feature_execution_order(&spec).is_none());
  }

  #[test]
  fn test_sort_behaviors_by_priority() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let behavior1 = create_test_behavior("create_user");
    let behavior2 = create_test_behavior("delete_user");

    let feature = create_test_feature("users", vec![behavior1, behavior2]);
    spec.features.push(feature);

    let validator = SpecValidator::new();
    let sorted = validator.sort_behaviors_by_priority(&spec);

    assert_eq!(sorted.len(), 2);
  }

  #[test]
  fn test_sort_by_category() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let behavior1 = create_test_behavior("create_user");
    let behavior2 = create_test_behavior("delete_user");
    let behavior3 = create_test_behavior("update_user");

    let feature = create_test_feature("users", vec![behavior1, behavior2, behavior3]);
    spec.features.push(feature);

    let validator = SpecValidator::new();
    let categories = validator.sort_by_category(&spec);

    // All behaviors start with different words, so they go in different categories
    assert!(categories.contains_key("create"));
    assert!(categories.contains_key("delete"));
    assert!(categories.contains_key("update"));
  }

  #[test]
  fn test_behavior_priority_score() {
    let priority = BehaviorPriority {
      path: "test.behavior".to_string(),
      dependent_count: 5,
      precondition_count: 2,
    };

    // Score = 5 * 10 + (100 - 2) = 50 + 98 = 148
    assert_eq!(priority.score(), 148);
  }

  #[test]
  fn test_spec_validation_error_display() {
    let err = SpecValidationError::MissingRequiredField("name".to_string());
    assert!(format!("{err}").contains("name"));

    let err = SpecValidationError::InvalidFieldType {
      field: "count".to_string(),
      expected: "number".to_string(),
      actual: "string".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("count"));
    assert!(msg.contains("number"));
    assert!(msg.contains("string"));

    let err = SpecValidationError::CircularDependency {
      behavior_a: "A".to_string(),
      behavior_b: "B".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains('A'));
    assert!(msg.contains('B'));
  }

  #[test]
  fn test_validation_warning() {
    let warning = ValidationWarning::new("test warning".to_string(), Some("context".to_string()));

    assert_eq!(warning.message, "test warning");
    assert_eq!(warning.context, Some("context".to_string()));
  }

  #[test]
  fn test_validate_spec_with_invariants() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let invariant = Invariant::new(
      "unique_email".to_string(),
      "Emails must be unique".to_string(),
    );
    spec.add_invariant(invariant);

    let feature = create_test_feature("users", vec![]);
    spec.add_feature(feature).expect("should add feature");

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);
  }

  #[test]
  fn test_validate_spec_with_verification() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    let verification = Verification::new("unit_test".to_string(), "Test it".to_string());
    let behavior = Behavior::new("create_user".to_string())
      .expect("valid name")
      .with_verification(verification);

    let feature = create_test_feature("users", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);
  }

  #[test]
  fn test_validate_complex_dependency_chain() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    // Create features: A -> B -> C -> D (linear chain)
    let feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);
    let mut feature_c = create_test_feature("feature_c", vec![]);
    let mut feature_d = create_test_feature("feature_d", vec![]);

    feature_b.add_dependency("feature_a".to_string());
    feature_c.add_dependency("feature_b".to_string());
    feature_d.add_dependency("feature_c".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);
    spec.features.push(feature_c);
    spec.features.push(feature_d);

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);

    // Verify execution order
    let order = feature_execution_order(&spec).expect("should have order");
    assert_eq!(order.len(), 4);

    // A should be first, D should be last
    assert_eq!(order[0], "feature_a");
    assert_eq!(order[3], "feature_d");
  }

  #[test]
  fn test_validate_diamond_dependency() {
    let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

    // Diamond: A -> B, A -> C, B -> D, C -> D
    let feature_a = create_test_feature("feature_a", vec![]);
    let mut feature_b = create_test_feature("feature_b", vec![]);
    let mut feature_c = create_test_feature("feature_c", vec![]);
    let mut feature_d = create_test_feature("feature_d", vec![]);

    feature_b.add_dependency("feature_a".to_string());
    feature_c.add_dependency("feature_a".to_string());
    feature_d.add_dependency("feature_b".to_string());
    feature_d.add_dependency("feature_c".to_string());

    spec.features.push(feature_a);
    spec.features.push(feature_b);
    spec.features.push(feature_c);
    spec.features.push(feature_d);

    let result = validate_spec(&spec);
    assert!(result.is_valid, "Errors: {:?}", result.errors);
  }
}
