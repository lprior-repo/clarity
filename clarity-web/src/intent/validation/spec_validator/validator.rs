use super::{
  BehaviorPriority, DependencyGraph, SpecValidationError, ValidationResult, ValidationWarning,
};
use crate::intent::types::{Behavior, Feature, Spec};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpecValidator {
  check_duplicates: bool,
  check_cycles: bool,
  check_required_fields: bool,
}

impl SpecValidator {
  #[must_use]
  pub fn new() -> Self {
    Self {
      check_duplicates: true,
      check_cycles: true,
      check_required_fields: true,
    }
  }

  #[must_use]
  pub fn without_duplicate_checking(mut self) -> Self {
    self.check_duplicates = false;
    self
  }

  #[must_use]
  pub fn without_cycle_checking(mut self) -> Self {
    self.check_cycles = false;
    self
  }

  #[must_use]
  pub fn without_required_field_checking(mut self) -> Self {
    self.check_required_fields = false;
    self
  }

  #[must_use]
  pub fn validate(&self, spec: &Spec) -> ValidationResult {
    let mut result = ValidationResult::new();

    if self.check_required_fields {
      self.validate_required_fields(spec, &mut result);
    }
    self.validate_unique_features(spec, &mut result);
    for feature in &spec.features {
      self.validate_feature(feature, &mut result);
    }
    if self.check_duplicates {
      self.detect_duplicate_behaviors(spec, &mut result);
    }
    if self.check_cycles {
      self.detect_circular_dependencies(spec, &mut result);
    }

    result
  }

  fn validate_required_fields(&self, spec: &Spec, result: &mut ValidationResult) {
    if spec.name.trim().is_empty() {
      result.add_error(SpecValidationError::MissingRequiredField(
        "spec.name".to_string(),
      ));
    }
  }

  fn validate_unique_features(&self, spec: &Spec, result: &mut ValidationResult) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (index, feature) in spec.features.iter().enumerate() {
      if let Some(&first_index) = seen.get(feature.name.as_str()) {
        result.add_error(SpecValidationError::DuplicateBehavior {
          behavior_a: format!(
            "features[{first_index}].{}",
            spec.features[first_index].name
          ),
          behavior_b: format!("features[{index}].{}", feature.name),
          description: format!("Duplicate feature name: {}", feature.name),
          impact: "Features must have unique names for proper identification".to_string(),
        });
      } else {
        seen.insert(&feature.name, index);
      }
    }
  }

  fn validate_feature(&self, feature: &Feature, result: &mut ValidationResult) {
    if feature.name.trim().is_empty() {
      result.add_error(SpecValidationError::MissingRequiredField(format!(
        "feature '{}' .name",
        feature.name
      )));
    }

    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (index, behavior) in feature.behaviors.iter().enumerate() {
      if let Some(&first_index) = seen.get(behavior.name.as_str()) {
        result.add_error(SpecValidationError::DuplicateBehavior {
          behavior_a: format!(
            "{}.behaviors[{first_index}].{}",
            feature.name, feature.behaviors[first_index].name
          ),
          behavior_b: format!("{}.behaviors[{index}].{}", feature.name, behavior.name),
          description: format!("Duplicate behavior name: {}", behavior.name),
          impact: "Behaviors within a feature must have unique names".to_string(),
        });
      } else {
        seen.insert(&behavior.name, index);
      }
    }
  }

  fn detect_duplicate_behaviors(&self, spec: &Spec, result: &mut ValidationResult) {
    let all_behaviors: Vec<(String, &Behavior)> = spec
      .features
      .iter()
      .flat_map(|feature| {
        feature
          .behaviors
          .iter()
          .map(move |behavior| (format!("{}.{}", feature.name, behavior.name), behavior))
      })
      .collect();

    let by_description: HashMap<&str, Vec<&str>> = all_behaviors
      .iter()
      .filter(|(_, behavior)| !behavior.description.is_empty())
      .fold(HashMap::new(), |mut acc, (path, behavior)| {
        acc
          .entry(behavior.description.as_str())
          .or_default()
          .push(path.as_str());
        acc
      });

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

  fn detect_circular_dependencies(&self, spec: &Spec, result: &mut ValidationResult) {
    if let Some(path) = self.build_dependency_graph(spec).detect_cycles() {
      if path.len() >= 2 {
        result.add_error(SpecValidationError::CircularDependencyPath { path });
      }
    }

    if let Some(path) = self.build_feature_dependency_graph(spec).detect_cycles() {
      if path.len() >= 2 {
        result.add_error(SpecValidationError::CircularDependencyPath { path });
      }
    }
  }

  fn build_dependency_graph(&self, spec: &Spec) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    let behavior_paths: HashMap<String, String> = spec
      .features
      .iter()
      .flat_map(|feature| {
        feature.behaviors.iter().map(move |behavior| {
          (
            behavior.name.clone(),
            format!("{}.{}", feature.name, behavior.name),
          )
        })
      })
      .collect();

    for path in behavior_paths.values() {
      graph.add_node(path.clone());
    }

    for feature in &spec.features {
      for behavior in &feature.behaviors {
        let from_path = format!("{}.{}", feature.name, behavior.name);
        for precondition in &behavior.preconditions {
          if let Some(to_path) = behavior_paths.get(precondition) {
            graph.add_edge(from_path.clone(), to_path.clone());
          }
        }
      }
    }

    graph
  }

  pub fn build_feature_dependency_graph(&self, spec: &Spec) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    for feature in &spec.features {
      graph.add_node(feature.name.clone());
    }
    for feature in &spec.features {
      for dependency in &feature.depends_on {
        graph.add_edge(feature.name.clone(), dependency.clone());
      }
    }

    graph
  }

  #[must_use]
  pub fn sort_behaviors_by_priority(&self, spec: &Spec) -> Vec<(String, BehaviorPriority)> {
    let behavior_paths: HashMap<String, String> = spec
      .features
      .iter()
      .flat_map(|feature| {
        feature.behaviors.iter().map(move |behavior| {
          (
            behavior.name.clone(),
            format!("{}.{}", feature.name, behavior.name),
          )
        })
      })
      .collect();

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

    let mut prioritized: Vec<(String, BehaviorPriority)> = spec
      .features
      .iter()
      .flat_map(|feature| {
        feature.behaviors.iter().map(|behavior| {
          let path = format!("{}.{}", feature.name, behavior.name);
          let dependent_count_for_path = match dependent_count.get(&path) {
            Some(value) => *value,
            None => 0,
          };
          let precondition_count_for_path = match precondition_count.get(&path) {
            Some(value) => *value,
            None => 0,
          };
          let priority = BehaviorPriority {
            path: path.clone(),
            dependent_count: dependent_count_for_path,
            precondition_count: precondition_count_for_path,
          };
          (path, priority)
        })
      })
      .collect();

    prioritized.sort_by(|left, right| {
      right
        .1
        .dependent_count
        .cmp(&left.1.dependent_count)
        .then(left.1.precondition_count.cmp(&right.1.precondition_count))
    });
    prioritized
  }
}

impl Default for SpecValidator {
  fn default() -> Self {
    Self::new()
  }
}
