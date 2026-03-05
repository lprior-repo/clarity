#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unwrap_or_default)]
#![allow(clippy::implicit_clone)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::question_mark)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::useless_vec)]
#![forbid(unsafe_code)]

//! Effects module for dependency tracing and causal relationship analysis.
//!
//! This module parses causal language from solutions and builds dependency graphs
//! to trace how different outcomes relate to and depend on each other.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for effects analysis
#[derive(Debug, Error)]
pub enum EffectsError {
  #[error("circular dependency detected: {0} -> {1}")]
  CircularDependency(String, String),

  #[error("maximum dependency depth exceeded: {0}")]
  MaxDepthExceeded(usize),

  #[error("invalid causal language pattern: {0}")]
  InvalidPattern(String),
}

/// Represents a single causal effect with its confidence level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Effect {
  /// The trigger action or state
  pub trigger: String,
  /// The resulting outcome
  pub outcome: String,
  /// Confidence level (0.0 to 1.0)
  #[serde(default = "default_confidence")]
  pub confidence: f64,
  /// Indirect effects that follow from this outcome
  #[serde(default)]
  pub indirect_effects: Vec<String>,
}

fn default_confidence() -> f64 {
  0.5
}

/// Dependency graph node for visualization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyNode {
  pub id: String,
  pub label: String,
  #[serde(default)]
  pub is_root: bool,
  #[serde(default)]
  pub is_leaf: bool,
}

/// Dependency graph edge for visualization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyEdge {
  pub from: String,
  pub to: String,
  #[serde(default = "default_confidence")]
  pub confidence: f64,
  #[serde(default)]
  pub indirect: bool,
}

/// Complete output of effects analysis with dependency graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectsOutput {
  /// All extracted effects
  pub effects: Vec<Effect>,
  /// Dependency graph: node -> list of dependent nodes
  #[serde(default)]
  pub dependency_graph: HashMap<String, Vec<String>>,
  /// Nodes for visualization
  #[serde(default)]
  pub nodes: Vec<DependencyNode>,
  /// Edges for visualization
  #[serde(default)]
  pub edges: Vec<DependencyEdge>,
  /// Any warnings or issues detected
  #[serde(default)]
  pub warnings: Vec<String>,
}

impl EffectsOutput {
  /// Create a new empty EffectsOutput
  pub fn new() -> Self {
    Self {
      effects: Vec::new(),
      dependency_graph: HashMap::new(),
      nodes: Vec::new(),
      edges: Vec::new(),
      warnings: Vec::new(),
    }
  }

  /// Add an effect to the output
  pub fn add_effect(&mut self, effect: Effect) {
    // Add trigger->outcome edge to dependency graph
    self
      .dependency_graph
      .entry(effect.trigger.clone())
      .or_insert_with(Vec::new)
      .push(effect.outcome.clone());

    // Ensure outcome node exists in graph
    self
      .dependency_graph
      .entry(effect.outcome.clone())
      .or_insert_with(Vec::new);

    self.effects.push(effect);
  }

  /// Build visualization structures from the dependency graph
  pub fn build_visualization(&mut self) {
    let all_nodes: HashSet<&String> = self.dependency_graph.keys().collect();
    let has_dependents: HashSet<&String> = self.dependency_graph.values().flatten().collect();

    // Build nodes
    self.nodes = all_nodes
      .iter()
      .map(|id| DependencyNode {
        id: id.to_string(),
        label: id.to_string(),
        is_root: !has_dependents.contains(id),
        is_leaf: self
          .dependency_graph
          .get(*id)
          .map_or(true, |deps| deps.is_empty()),
      })
      .collect();

    // Build edges
    self.edges = self
      .dependency_graph
      .iter()
      .flat_map(|(from, to_list)| {
        to_list.iter().map(|to| DependencyEdge {
          from: from.clone(),
          to: to.clone(),
          confidence: 0.5,
          indirect: false,
        })
      })
      .collect();
  }
}

impl Default for EffectsOutput {
  fn default() -> Self {
    Self::new()
  }
}

/// Causal language pattern matcher
#[derive(Debug, Clone)]
pub struct CausalPattern {
  /// The keyword to search for (e.g., "causes", "leads to")
  pub keyword: String,
  /// Whether this pattern indicates a positive (enables) or negative (blocks) relationship
  pub is_positive: bool,
  /// Default confidence for this pattern
  pub default_confidence: f64,
}

impl CausalPattern {
  /// Create a new causal pattern
  pub fn new(keyword: &str, is_positive: bool, default_confidence: f64) -> Self {
    Self {
      keyword: keyword.to_lowercase(),
      is_positive,
      default_confidence: default_confidence.clamp(0.0, 1.0),
    }
  }
}

/// Default causal patterns for English language
fn default_patterns() -> Vec<CausalPattern> {
  vec![
    CausalPattern::new("causes", true, 0.8),
    CausalPattern::new("leads to", true, 0.7),
    CausalPattern::new("enables", true, 0.9),
    CausalPattern::new("results in", true, 0.75),
    CausalPattern::new("produces", true, 0.7),
    CausalPattern::new("generates", true, 0.7),
    CausalPattern::new("creates", true, 0.7),
    CausalPattern::new("requires", true, 0.9),
    CausalPattern::new("depends on", true, 0.9),
    CausalPattern::new("needs", true, 0.8),
    CausalPattern::new("blocks", false, 0.8),
    CausalPattern::new("prevents", false, 0.9),
    CausalPattern::new("inhibits", false, 0.8),
    CausalPattern::new("stops", false, 0.9),
  ]
}

/// Parse causal relationships from solution text
///
/// # Arguments
/// * `solution` - The solution text to parse
///
/// # Returns
/// * `EffectsOutput` containing extracted effects and dependency graph
pub fn trace_effects(solution: &str) -> EffectsOutput {
  let patterns = default_patterns();
  trace_effects_with_patterns(solution, &patterns)
}

/// Parse causal relationships using custom patterns
///
/// # Arguments
/// * `solution` - The solution text to parse
/// * `patterns` - Custom causal patterns to use
///
/// # Returns
/// * `EffectsOutput` containing extracted effects and dependency graph
pub fn trace_effects_with_patterns(solution: &str, patterns: &[CausalPattern]) -> EffectsOutput {
  let mut output = EffectsOutput::new();
  let sentences: Vec<&str> = solution
    .split('.')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect();

  for sentence in sentences {
    if let Some(effect) = parse_sentence(sentence, patterns) {
      output.add_effect(effect);
    }
  }

  // Check for circular dependencies
  if let Err(e) = detect_cycles(&output.dependency_graph) {
    output.warnings.push(e.to_string());
  }

  // Build visualization structures
  output.build_visualization();

  output
}

/// Parse a single sentence for causal relationships
fn parse_sentence(sentence: &str, patterns: &[CausalPattern]) -> Option<Effect> {
  let lower = sentence.to_lowercase();

  for pattern in patterns {
    if let Some(pos) = lower.find(&pattern.keyword) {
      // Extract trigger (before the keyword)
      let trigger = sentence[..pos].trim().to_string();

      // Extract outcome (after the keyword)
      let outcome_start = pos + pattern.keyword.len();
      let outcome = sentence[outcome_start..].trim().to_string();

      // Only create effect if both trigger and outcome are non-empty
      if !trigger.is_empty() && !outcome.is_empty() {
        return Some(Effect {
          trigger: clean_text(&trigger),
          outcome: clean_text(&outcome),
          confidence: pattern.default_confidence,
          indirect_effects: Vec::new(),
        });
      }
    }
  }

  None
}

/// Clean and normalize text
fn clean_text(text: &str) -> String {
  text
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .trim()
    .to_string()
}

/// Detect circular dependencies in the graph
///
/// # Arguments
/// * `graph` - The dependency graph to check
///
/// # Returns
/// * `Ok(())` if no cycles detected
/// * `Err(EffectsError::CircularDependency)` if cycle found
pub fn detect_cycles(graph: &HashMap<String, Vec<String>>) -> Result<(), EffectsError> {
  let mut visited = HashSet::new();
  let mut rec_stack = HashSet::new();

  for node in graph.keys() {
    if !visited.contains(node) {
      if let Err(err) = dfs_visit(node, graph, &mut visited, &mut rec_stack, 0) {
        return Err(err);
      }
    }
  }

  Ok(())
}

/// Depth-limited DFS visit for cycle detection
fn dfs_visit(
  node: &str,
  graph: &HashMap<String, Vec<String>>,
  visited: &mut HashSet<String>,
  rec_stack: &mut HashSet<String>,
  depth: usize,
) -> Result<(), EffectsError> {
  const MAX_DEPTH: usize = 5;

  if depth > MAX_DEPTH {
    return Err(EffectsError::MaxDepthExceeded(depth));
  }

  visited.insert(node.to_string());
  rec_stack.insert(node.to_string());

  if let Some(neighbors) = graph.get(node) {
    for neighbor in neighbors {
      if !visited.contains(neighbor) {
        dfs_visit(neighbor, graph, visited, rec_stack, depth + 1)?;
      } else if rec_stack.contains(neighbor) {
        return Err(EffectsError::CircularDependency(
          node.to_string(),
          neighbor.to_string(),
        ));
      }
    }
  }

  rec_stack.remove(node);
  Ok(())
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  #![allow(clippy::float_cmp)]

  use super::*;

  #[test]
  fn test_extract_simple_causal_relationship() {
    let solution = "Increasing exercise causes weight loss. Better diet causes better health.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 2);
    assert_eq!(output.effects[0].trigger, "Increasing exercise");
    assert_eq!(output.effects[0].outcome, "weight loss");
    assert!(output.effects[0].confidence > 0.5);
    assert_eq!(output.effects[1].trigger, "Better diet");
    assert_eq!(output.effects[1].outcome, "better health");
  }

  #[test]
  fn test_extract_leads_to_pattern() {
    let solution = "Practice leads to mastery. Study leads to understanding.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 2);
    assert_eq!(output.effects[0].trigger, "Practice");
    assert_eq!(output.effects[0].outcome, "mastery");
  }

  #[test]
  fn test_extract_enables_pattern() {
    let solution = "Education enables career growth. Training enables opportunities.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 2);
    assert_eq!(output.effects[0].trigger, "Education");
    assert_eq!(output.effects[0].outcome, "career growth");
    // "enables" should have higher confidence
    assert!(output.effects[0].confidence > 0.8);
    assert_eq!(output.effects[1].trigger, "Training");
    assert_eq!(output.effects[1].outcome, "opportunities");
  }

  #[test]
  fn test_extract_negative_causality() {
    let solution = "Poor planning prevents success. Procrastination blocks progress.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 2);
    assert_eq!(output.effects[0].trigger, "Poor planning");
    assert_eq!(output.effects[0].outcome, "success");
    assert_eq!(output.effects[1].trigger, "Procrastination");
    assert_eq!(output.effects[1].outcome, "progress");
  }

  #[test]
  fn test_dependency_graph_construction() {
    let solution = "A causes B. B causes C. C causes D.";
    let output = trace_effects(solution);

    assert_eq!(output.dependency_graph.len(), 4);
    assert_eq!(
      output.dependency_graph.get("A"),
      Some(&vec!["B".to_string()])
    );
    assert_eq!(
      output.dependency_graph.get("B"),
      Some(&vec!["C".to_string()])
    );
    assert_eq!(
      output.dependency_graph.get("C"),
      Some(&vec!["D".to_string()])
    );
    assert_eq!(output.dependency_graph.get("D"), Some(&vec![]));
  }

  #[test]
  fn test_detect_circular_dependencies() {
    let mut graph = HashMap::new();
    graph.insert("A".to_string(), vec!["B".to_string()]);
    graph.insert("B".to_string(), vec!["C".to_string()]);
    graph.insert("C".to_string(), vec!["A".to_string()]);

    let result = detect_cycles(&graph);
    assert!(result.is_err());
    match result {
      Err(EffectsError::CircularDependency(from, to)) => {
        // The exact nodes in the cycle may vary depending on traversal order
        assert!(vec!["A", "B", "C"].contains(&from.as_str()));
        assert!(vec!["A", "B", "C"].contains(&to.as_str()));
        assert_ne!(from, to); // They should be different nodes
      }
      _ => panic!("Expected CircularDependency error"),
    }
  }

  #[test]
  fn test_no_circular_dependencies() {
    let mut graph = HashMap::new();
    graph.insert("A".to_string(), vec!["B".to_string()]);
    graph.insert("B".to_string(), vec!["C".to_string()]);
    graph.insert("C".to_string(), vec![]);

    let result = detect_cycles(&graph);
    assert!(result.is_ok());
  }

  #[test]
  fn test_max_depth_limit() {
    // Create a chain within MAX_DEPTH to verify it passes
    // A -> B -> C -> D -> E (5 nodes = 4 edges, depth 0-4)
    let mut graph = HashMap::new();
    let nodes = ["A", "B", "C", "D", "E"];

    for window in nodes.windows(2) {
      graph.insert(window[0].to_string(), vec![window[1].to_string()]);
    }
    graph.insert("E".to_string(), vec![]);

    // This chain is within depth limits, so it should pass
    let result = detect_cycles(&graph);
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_solution() {
    let solution = "";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 0);
    assert_eq!(output.dependency_graph.len(), 0);
    assert_eq!(output.nodes.len(), 0);
    assert_eq!(output.edges.len(), 0);
  }

  #[test]
  fn test_solution_without_causal_language() {
    let solution = "This is a regular sentence. Nothing causal here.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 0);
    assert_eq!(output.dependency_graph.len(), 0);
  }

  #[test]
  fn test_text_cleaning() {
    let result = clean_text("  too    many   spaces  ");
    assert_eq!(result, "too many spaces");
  }

  #[test]
  fn test_custom_patterns() {
    let patterns = vec![
      CausalPattern::new("triggers", true, 0.9),
      CausalPattern::new("yields", true, 0.85),
    ];

    let solution = "Action A triggers Result B. Process C yields Outcome D.";
    let output = trace_effects_with_patterns(solution, &patterns);

    assert_eq!(output.effects.len(), 2);
    assert_eq!(output.effects[0].trigger, "Action A");
    assert_eq!(output.effects[0].outcome, "Result B");
    assert_eq!(output.effects[0].confidence, 0.9);
  }

  #[test]
  fn test_visualization_structure() {
    let solution = "Root causes Branch. Branch causes Leaf.";
    let mut output = trace_effects(solution);

    output.build_visualization();

    // Should have 3 nodes
    assert_eq!(output.nodes.len(), 3);

    // Should have 2 edges
    assert_eq!(output.edges.len(), 2);

    // Root node identification
    let root_node = output.nodes.iter().find(|n| n.id == "Root");
    assert!(root_node.is_some());
    assert!(root_node.unwrap().is_root);

    // Leaf node identification
    let leaf_node = output.nodes.iter().find(|n| n.id == "Leaf");
    assert!(leaf_node.is_some());
    assert!(leaf_node.unwrap().is_leaf);
  }

  #[test]
  fn test_serialization() {
    let solution = "A causes B. B causes C.";
    let output = trace_effects(solution);

    let json = serde_json::to_string(&output).expect("Failed to serialize");
    assert!(json.contains("A"));
    assert!(json.contains("B"));
    assert!(json.contains("C"));

    // Deserialize and verify
    let deserialized: EffectsOutput = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.effects.len(), 2);
    assert_eq!(deserialized.dependency_graph.len(), 3);
  }

  #[test]
  fn test_complex_dependency_chain() {
    let solution = "Research causes discovery. Discovery leads to innovation. Innovation enables progress. Progress results in growth.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 4);

    // Verify the chain is correctly built
    let graph = &output.dependency_graph;
    assert!(graph
      .get("Research")
      .map_or(false, |deps| deps.iter().any(|d| d.contains("discovery"))));
  }

  #[test]
  fn test_warning_on_cycle_detection() {
    let solution = "A causes B. B causes C. C causes A.";
    let output = trace_effects(solution);

    assert!(!output.warnings.is_empty());
    assert!(output
      .warnings
      .iter()
      .any(|w| w.contains("circular dependency")));
  }

  #[test]
  fn test_multiple_sentences_same_pattern() {
    let solution = "Heat causes expansion. Pressure causes compression. Force causes motion.";
    let output = trace_effects(solution);

    assert_eq!(output.effects.len(), 3);

    // All should use "causes" pattern
    for effect in &output.effects {
      assert_eq!(effect.confidence, 0.8);
    }
  }
}
