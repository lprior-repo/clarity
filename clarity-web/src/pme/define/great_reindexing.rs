#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Great Reindexing Engine - Converts time-based stories into graph-based requirements.
//!
//! This module transforms linear, time-based user stories into a graph structure
//! that captures relationships, dependencies, and Jobs to Be Done (JTBD).
//!
//! # Use Case Format
//!
//! Requirements are converted to the format: `[User] can [action] so that [motivation]`
//!
//! # Job to Be Done Identification
//!
//! The engine identifies Jobs to Be Done by analyzing user motivations and extracting
//! the underlying "job" the user is hiring the product to do.

use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Input Types
// ============================================================================

/// A raw story input to be reindexed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryInput {
  /// Unique identifier for this story
  pub id: String,
  /// The raw story text
  pub text: String,
  /// Optional source/context where this story came from
  pub source: Option<String>,
  /// Timestamp when the story was created (for time-based ordering)
  pub created_at: Option<String>,
}

impl StoryInput {
  /// Create a new story input.
  #[must_use]
  pub fn new(id: String, text: String) -> Self {
    Self {
      id,
      text,
      source: None,
      created_at: None,
    }
  }

  /// Add source context.
  #[must_use]
  pub fn with_source(mut self, source: String) -> Self {
    self.source = Some(source);
    self
  }

  /// Add timestamp.
  #[must_use]
  pub fn with_timestamp(mut self, timestamp: String) -> Self {
    self.created_at = Some(timestamp);
    self
  }
}

// ============================================================================
// User Story (Parsed)
// ============================================================================

/// A parsed user story in standard format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserStory {
  /// Original story ID
  pub id: String,
  /// The user/actor (who)
  pub user: String,
  /// The action (what)
  pub action: String,
  /// The motivation (why)
  pub motivation: String,
  /// Original text
  pub original_text: String,
}

impl UserStory {
  /// Create a new user story.
  #[must_use]
  pub fn new(
    id: String,
    user: String,
    action: String,
    motivation: String,
    original_text: String,
  ) -> Self {
    Self {
      id,
      user,
      action,
      motivation,
      original_text,
    }
  }

  /// Convert to use case format: `[User] can [action] so that [motivation]`
  #[must_use]
  pub fn to_use_case_format(&self) -> String {
    format!(
      "[{}] can [{}] so that [{}]",
      self.user, self.action, self.motivation
    )
  }
}

// ============================================================================
// Job to Be Done
// ============================================================================

/// A Job to Be Done identified from user motivations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobToBeDone {
  /// Unique identifier
  pub id: String,
  /// The job statement (what the user is hiring the product to do)
  pub job_statement: String,
  /// Source stories that led to this JTBD
  pub source_story_ids: Vec<String>,
  /// Related motivations
  pub motivations: Vec<String>,
  /// Priority (higher = more important)
  pub priority: u8,
}

impl JobToBeDone {
  /// Create a new Job to Be Done.
  #[must_use]
  pub fn new(id: String, job_statement: String) -> Self {
    Self {
      id,
      job_statement,
      source_story_ids: Vec::new(),
      motivations: Vec::new(),
      priority: 50,
    }
  }

  /// Add a source story.
  #[must_use]
  pub fn with_source_story(mut self, story_id: String) -> Self {
    self.source_story_ids.push(story_id);
    self
  }

  /// Add a motivation.
  #[must_use]
  pub fn with_motivation(mut self, motivation: String) -> Self {
    self.motivations.push(motivation);
    self
  }

  /// Set priority.
  #[must_use]
  pub fn with_priority(mut self, priority: u8) -> Self {
    self.priority = priority;
    self
  }
}

// ============================================================================
// Graph Types
// ============================================================================

/// A node in the requirement graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementNode {
  /// Unique identifier
  pub id: String,
  /// Node label/title
  pub label: String,
  /// Node type
  pub node_type: NodeType,
  /// Related user stories
  pub story_ids: Vec<String>,
}

/// Types of nodes in the requirement graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
  /// A user/actor
  User,
  /// An action/capability
  Action,
  /// A motivation/outcome
  Motivation,
  /// A Job to Be Done
  JobToBeDone,
  /// A requirement
  Requirement,
}

impl RequirementNode {
  /// Create a new requirement node.
  #[must_use]
  pub fn new(id: String, label: String, node_type: NodeType) -> Self {
    Self {
      id,
      label,
      node_type,
      story_ids: Vec::new(),
    }
  }

  /// Add a related story.
  #[must_use]
  pub fn with_story(mut self, story_id: String) -> Self {
    self.story_ids.push(story_id);
    self
  }
}

/// An edge connecting nodes in the requirement graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementEdge {
  /// Source node ID
  pub from: String,
  /// Target node ID
  pub to: String,
  /// Edge type/relationship
  pub relationship: EdgeRelationship,
}

/// Types of relationships between nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeRelationship {
  /// User performs action
  Performs,
  /// Action achieves motivation
  Achieves,
  /// Item depends on another
  DependsOn,
  /// Item relates to a Job to Be Done
  SupportsJob,
  /// Items are similar/equivalent
  SimilarTo,
}

impl RequirementEdge {
  /// Create a new edge.
  #[must_use]
  pub fn new(from: String, to: String, relationship: EdgeRelationship) -> Self {
    Self {
      from,
      to,
      relationship,
    }
  }
}

/// A graph-based requirement structure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementGraph {
  /// All nodes in the graph
  pub nodes: Vec<RequirementNode>,
  /// All edges in the graph
  pub edges: Vec<RequirementEdge>,
  /// Jobs to Be Done identified
  pub jobs_to_be_done: Vec<JobToBeDone>,
}

impl RequirementGraph {
  /// Create an empty graph.
  #[must_use]
  pub fn new() -> Self {
    Self {
      nodes: Vec::new(),
      edges: Vec::new(),
      jobs_to_be_done: Vec::new(),
    }
  }

  /// Add a node to the graph.
  #[must_use]
  pub fn with_node(mut self, node: RequirementNode) -> Self {
    self.nodes.push(node);
    self
  }

  /// Add an edge to the graph.
  #[must_use]
  pub fn with_edge(mut self, edge: RequirementEdge) -> Self {
    self.edges.push(edge);
    self
  }

  /// Add a Job to Be Done.
  #[must_use]
  pub fn with_job(mut self, job: JobToBeDone) -> Self {
    self.jobs_to_be_done.push(job);
    self
  }

  /// Find a node by ID.
  #[must_use]
  pub fn find_node(&self, id: &str) -> Option<&RequirementNode> {
    self.nodes.iter().find(|n| n.id == id)
  }

  /// Get nodes by type.
  #[must_use]
  pub fn nodes_by_type(&self, node_type: NodeType) -> Vec<&RequirementNode> {
    self
      .nodes
      .iter()
      .filter(|n| n.node_type == node_type)
      .collect()
  }

  /// Get edges from a node.
  #[must_use]
  pub fn edges_from(&self, node_id: &str) -> Vec<&RequirementEdge> {
    self.edges.iter().filter(|e| e.from == node_id).collect()
  }

  /// Get edges to a node.
  #[must_use]
  pub fn edges_to(&self, node_id: &str) -> Vec<&RequirementEdge> {
    self.edges.iter().filter(|e| e.to == node_id).collect()
  }

  /// Count nodes by type.
  #[must_use]
  pub fn count_nodes_by_type(&self) -> HashMap<NodeType, usize> {
    self.nodes.iter().fold(HashMap::new(), |mut acc, node| {
      *acc.entry(node.node_type).or_insert(0) += 1;
      acc
    })
  }

  /// Find connected components.
  #[must_use]
  pub fn connected_components(&self) -> Vec<HashSet<String>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    // Build adjacency list
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &self.edges {
      adjacency.entry(&edge.from).or_default().push(&edge.to);
      adjacency.entry(&edge.to).or_default().push(&edge.from);
    }

    for node in &self.nodes {
      if !visited.contains(&node.id) {
        let mut component = HashSet::new();
        Self::dfs(&node.id, &adjacency, &mut visited, &mut component);
        components.push(component);
      }
    }

    components
  }

  /// Depth-first search helper.
  fn dfs(
    node_id: &str,
    adjacency: &HashMap<&str, Vec<&str>>,
    visited: &mut HashSet<String>,
    component: &mut HashSet<String>,
  ) {
    visited.insert(node_id.to_string());
    component.insert(node_id.to_string());

    if let Some(neighbors) = adjacency.get(node_id) {
      for neighbor in neighbors {
        if !visited.contains(*neighbor) {
          Self::dfs(neighbor, adjacency, visited, component);
        }
      }
    }
  }
}

impl Default for RequirementGraph {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// Graph Requirement (Final Output)
// ============================================================================

/// A finalized graph-based requirement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRequirement {
  /// Unique identifier
  pub id: String,
  /// Use case format: `[User] can [action] so that [motivation]`
  pub use_case: String,
  /// Related Job to Be Done (if any)
  pub job_to_be_done: Option<String>,
  /// Dependencies (other requirement IDs)
  pub dependencies: Vec<String>,
  /// Source story IDs
  pub source_stories: Vec<String>,
  /// Priority score (0-100)
  pub priority: u8,
}

impl GraphRequirement {
  /// Create a new graph requirement.
  #[must_use]
  pub fn new(id: String, use_case: String) -> Self {
    Self {
      id,
      use_case,
      job_to_be_done: None,
      dependencies: Vec::new(),
      source_stories: Vec::new(),
      priority: 50,
    }
  }

  /// Add Job to Be Done reference.
  #[must_use]
  pub fn with_jtbd(mut self, jtbd: String) -> Self {
    self.job_to_be_done = Some(jtbd);
    self
  }

  /// Add dependency.
  #[must_use]
  pub fn with_dependency(mut self, dep_id: String) -> Self {
    self.dependencies.push(dep_id);
    self
  }

  /// Add source story.
  #[must_use]
  pub fn with_source_story(mut self, story_id: String) -> Self {
    self.source_stories.push(story_id);
    self
  }

  /// Set priority.
  #[must_use]
  pub fn with_priority(mut self, priority: u8) -> Self {
    self.priority = priority;
    self
  }
}

// ============================================================================
// Output
// ============================================================================

/// Output from the Great Reindexing Engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReindexingOutput {
  /// Parsed user stories
  pub user_stories: Vec<UserStory>,
  /// The requirement graph
  pub graph: RequirementGraph,
  /// Finalized graph-based requirements
  pub requirements: Vec<GraphRequirement>,
  /// Identified Jobs to Be Done
  pub jobs_to_be_done: Vec<JobToBeDone>,
  /// Statistics
  pub stats: ReindexingStats,
}

/// Statistics from the reindexing process.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReindexingStats {
  /// Total input stories
  pub total_input_stories: usize,
  /// Successfully parsed stories
  pub parsed_stories: usize,
  /// Graph nodes created
  pub graph_nodes: usize,
  /// Graph edges created
  pub graph_edges: usize,
  /// Jobs to Be Done identified
  pub jtbd_count: usize,
  /// Connected components in graph
  pub connected_components: usize,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the Great Reindexing Engine.
#[derive(Debug, Error)]
pub enum ReindexingError {
  /// Failed to parse a story
  #[error("Failed to parse story '{id}': {reason}")]
  ParseFailed {
    /// Story ID
    id: String,
    /// Reason for failure
    reason: String,
  },

  /// Empty input
  #[error("No stories provided for reindexing")]
  EmptyInput,

  /// Invalid story format
  #[error("Invalid story format: {0}")]
  InvalidFormat(String),
}

// ============================================================================
// Engine Implementation
// ============================================================================

/// The Great Reindexing Engine.
///
/// Converts time-based user stories into a graph-based requirement structure.
pub struct GreatReindexingEngine;

impl GreatReindexingEngine {
  /// Run the reindexing process on a collection of stories.
  ///
  /// # Errors
  ///
  /// Returns an error if no stories are provided or if parsing fails completely.
  pub fn reindex(stories: &[StoryInput]) -> Result<ReindexingOutput, ReindexingError> {
    if stories.is_empty() {
      return Err(ReindexingError::EmptyInput);
    }

    // Step 1: Parse stories into user story format
    let user_stories = Self::parse_stories(stories);

    // Step 2: Build the requirement graph
    let graph = Self::build_graph(&user_stories);

    // Step 3: Identify Jobs to Be Done
    let jobs = Self::identify_jobs(&user_stories);

    // Step 4: Create graph-based requirements
    let requirements = Self::create_requirements(&user_stories, &jobs);

    // Step 5: Calculate statistics
    let stats = ReindexingStats {
      total_input_stories: stories.len(),
      parsed_stories: user_stories.len(),
      graph_nodes: graph.nodes.len(),
      graph_edges: graph.edges.len(),
      jtbd_count: jobs.len(),
      connected_components: graph.connected_components().len(),
    };

    Ok(ReindexingOutput {
      user_stories,
      graph,
      requirements,
      jobs_to_be_done: jobs,
      stats,
    })
  }

  /// Parse raw stories into structured user stories.
  fn parse_stories(stories: &[StoryInput]) -> Vec<UserStory> {
    stories
      .iter()
      .filter_map(|story| Self::parse_single_story(story))
      .collect()
  }

  /// Parse a single story into user story format.
  fn parse_single_story(story: &StoryInput) -> Option<UserStory> {
    let text = story.text.trim();

    // Try to parse as "As a X, I want Y, so that Z" format
    if let Some((user, action, motivation)) = Self::parse_as_a_format(text) {
      return Some(UserStory::new(
        story.id.clone(),
        user,
        action,
        motivation,
        story.text.clone(),
      ));
    }

    // Try to parse as "I want X so that Y" format
    if let Some((action, motivation)) = Self::parse_i_want_format(text) {
      return Some(UserStory::new(
        story.id.clone(),
        "User".to_string(),
        action,
        motivation,
        story.text.clone(),
      ));
    }

    // Try to parse as simple statement
    if !text.is_empty() {
      return Some(UserStory::new(
        story.id.clone(),
        "User".to_string(),
        text.to_string(),
        "Achieve goal".to_string(),
        story.text.clone(),
      ));
    }

    None
  }

  /// Parse "As a X, I want Y, so that Z" format.
  fn parse_as_a_format(text: &str) -> Option<(String, String, String)> {
    let lower = text.to_lowercase();

    // Find "as a" or "as an"
    let as_pos = if lower.contains("as a ") {
      lower.find("as a ")
    } else {
      lower.find("as an ")
    }?;

    // Find "I want" or "i want"
    let want_pos = lower.find("i want")?;

    // Find "so that"
    let so_that_pos = lower.find("so that")?;

    if want_pos > as_pos && so_that_pos > want_pos {
      let user_start = as_pos
        + if lower[as_pos..].starts_with("as an ") {
          6
        } else {
          5
        };
      let user = text[user_start..want_pos]
        .trim()
        .trim_end_matches(',')
        .to_string();
      let action = text[want_pos + 6..so_that_pos]
        .trim()
        .trim_end_matches(',')
        .to_string();
      let motivation = text[so_that_pos + 7..].trim().to_string();

      if !user.is_empty() && !action.is_empty() {
        return Some((user, action, motivation));
      }
    }

    None
  }

  /// Parse "I want X so that Y" format.
  fn parse_i_want_format(text: &str) -> Option<(String, String)> {
    let lower = text.to_lowercase();
    let want_pos = lower.find("i want")?;
    let so_that_pos = lower.find("so that")?;

    if so_that_pos > want_pos {
      let action = text[want_pos + 6..so_that_pos]
        .trim()
        .trim_end_matches(',')
        .to_string();
      let motivation = text[so_that_pos + 7..].trim().to_string();

      if !action.is_empty() {
        return Some((action, motivation));
      }
    }

    None
  }

  /// Build the requirement graph from user stories.
  fn build_graph(stories: &[UserStory]) -> RequirementGraph {
    let mut graph = RequirementGraph::new();
    let mut user_nodes: HashMap<String, String> = HashMap::new();
    let mut action_nodes: HashMap<String, String> = HashMap::new();

    for story in stories {
      // Create or find user node
      let user_node_id = match user_nodes.get(&story.user) {
        Some(id) => id.clone(),
        None => {
          let id = format!("user_{}", story.user.to_lowercase().replace(' ', "_"));
          user_nodes.insert(story.user.clone(), id.clone());
          graph = graph.with_node(
            RequirementNode::new(id.clone(), story.user.clone(), NodeType::User)
              .with_story(story.id.clone()),
          );
          id
        }
      };

      // Create action node
      let action_id = format!("action_{}", story.id);
      graph = graph.with_node(
        RequirementNode::new(action_id.clone(), story.action.clone(), NodeType::Action)
          .with_story(story.id.clone()),
      );
      action_nodes.insert(story.id.clone(), action_id.clone());

      // Create motivation node
      let motivation_id = format!("motivation_{}", story.id);
      graph = graph.with_node(
        RequirementNode::new(
          motivation_id.clone(),
          story.motivation.clone(),
          NodeType::Motivation,
        )
        .with_story(story.id.clone()),
      );

      // Create edges
      graph = graph
        .with_edge(RequirementEdge::new(
          user_node_id.clone(),
          action_id.clone(),
          EdgeRelationship::Performs,
        ))
        .with_edge(RequirementEdge::new(
          action_id.clone(),
          motivation_id,
          EdgeRelationship::Achieves,
        ));
    }

    // Find similar actions and create similarity edges
    // Collect edges first to avoid borrow issues
    let actions: Vec<_> = graph.nodes_by_type(NodeType::Action);
    let similarity_edges: Vec<RequirementEdge> = actions
      .iter()
      .enumerate()
      .flat_map(|(i, action1)| {
        actions
          .iter()
          .skip(i + 1)
          .filter(|action2| Self::are_similar(&action1.label, &action2.label))
          .map(|action2| {
            RequirementEdge::new(
              action1.id.clone(),
              action2.id.clone(),
              EdgeRelationship::SimilarTo,
            )
          })
          .collect::<Vec<_>>()
      })
      .collect();

    for edge in similarity_edges {
      graph = graph.with_edge(edge);
    }

    graph
  }

  /// Check if two action labels are similar.
  fn are_similar(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    // Check for significant word overlap
    let a_words: HashSet<_> = a_lower.split_whitespace().collect();
    let b_words: HashSet<_> = b_lower.split_whitespace().collect();

    let intersection_count = a_words.intersection(&b_words).count();
    let min_len = a_words.len().min(b_words.len());

    min_len > 0 && intersection_count * 2 >= min_len
  }

  /// Identify Jobs to Be Done from user stories.
  fn identify_jobs(stories: &[UserStory]) -> Vec<JobToBeDone> {
    // Group stories by motivation similarity
    let mut motivation_groups: HashMap<String, Vec<&UserStory>> = HashMap::new();

    for story in stories {
      // Use the motivation as a key (simplified grouping)
      let key = story.motivation.to_lowercase();
      motivation_groups.entry(key).or_default().push(story);
    }

    // Create JTBD from each group
    motivation_groups
      .into_iter()
      .sorted_by(|a, b| b.1.len().cmp(&a.1.len()))
      .enumerate()
      .map(|(idx, (motivation, group_stories))| {
        let job_statement = Self::extract_job_statement(&motivation);
        let source_ids: Vec<String> = group_stories.iter().map(|s| s.id.clone()).collect();
        let motivations: Vec<String> = group_stories.iter().map(|s| s.motivation.clone()).collect();

        JobToBeDone {
          id: format!("jtbd_{}", idx),
          job_statement,
          source_story_ids: source_ids,
          motivations,
          priority: 100u8.saturating_sub(u8::try_from(idx * 10).unwrap_or(0)),
        }
      })
      .collect()
  }

  /// Extract a job statement from a motivation.
  fn extract_job_statement(motivation: &str) -> String {
    // Transform motivation into a job statement
    // "so that I can save time" -> "Save time"
    let lower = motivation.to_lowercase();

    let cleaned = lower
      .strip_prefix("i can ")
      .or_else(|| lower.strip_prefix("to "))
      .or_else(|| lower.strip_prefix("i am able to "))
      .unwrap_or(&lower);

    // Capitalize first letter
    let mut result = String::new();
    for (i, c) in cleaned.chars().enumerate() {
      if i == 0 {
        result.extend(c.to_uppercase());
      } else {
        result.push(c);
      }
    }

    result
  }

  /// Create graph-based requirements from stories and jobs.
  fn create_requirements(stories: &[UserStory], jobs: &[JobToBeDone]) -> Vec<GraphRequirement> {
    stories
      .iter()
      .map(|story| {
        let use_case = story.to_use_case_format();
        let job_ref = jobs
          .iter()
          .find(|j| j.source_story_ids.contains(&story.id))
          .map(|j| j.id.clone());

        let mut req = GraphRequirement::new(format!("req_{}", story.id), use_case)
          .with_source_story(story.id.clone());

        if let Some(jtbd) = job_ref {
          req = req.with_jtbd(jtbd);
        }

        req
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_story(id: &str, text: &str) -> StoryInput {
    StoryInput::new(id.to_string(), text.to_string())
  }

  #[test]
  fn test_parse_as_a_format() {
    let text = "As a developer, I want to debug code, so that I can fix bugs faster";
    let result = GreatReindexingEngine::parse_as_a_format(text);

    let (user, action, motivation) = result.expect("Should parse valid format");
    assert_eq!(user, "developer");
    assert_eq!(action, "to debug code");
    assert_eq!(motivation, "I can fix bugs faster");
  }

  #[test]
  fn test_parse_as_an_format() {
    let text = "As an administrator, I want to manage users, so that I can control access";
    let result = GreatReindexingEngine::parse_as_a_format(text);

    let (user, action, _motivation) = result.expect("Should parse 'as an' format");
    assert_eq!(user, "administrator");
    assert_eq!(action, "to manage users");
  }

  #[test]
  fn test_parse_i_want_format() {
    let text = "I want to export data so that I can analyze it externally";
    let result = GreatReindexingEngine::parse_i_want_format(text);

    let (action, motivation) = result.expect("Should parse 'I want' format");
    assert_eq!(action, "to export data");
    assert_eq!(motivation, "I can analyze it externally");
  }

  #[test]
  fn test_reindex_empty_input() {
    let result = GreatReindexingEngine::reindex(&[]);
    assert!(result.is_err());
    assert!(matches!(result, Err(ReindexingError::EmptyInput)));
  }

  #[test]
  fn test_reindex_single_story() {
    let stories = vec![create_test_story(
      "s1",
      "As a user, I want to login, so that I can access my account",
    )];

    let output = GreatReindexingEngine::reindex(&stories).expect("Should succeed");

    assert_eq!(output.user_stories.len(), 1);
    assert!(!output.graph.nodes.is_empty());
    assert!(!output.requirements.is_empty());

    let req = &output.requirements[0];
    assert!(req.use_case.contains("[user]"));
    assert!(req.use_case.contains("[to login]"));
  }

  #[test]
  fn test_reindex_multiple_stories() {
    let stories = vec![
      create_test_story(
        "s1",
        "As a user, I want to login, so that I can access my account",
      ),
      create_test_story(
        "s2",
        "As a user, I want to logout, so that I can secure my account",
      ),
      create_test_story(
        "s3",
        "As an admin, I want to delete users, so that I can manage access",
      ),
    ];

    let output = GreatReindexingEngine::reindex(&stories).expect("Should succeed");

    assert_eq!(output.stats.total_input_stories, 3);
    assert_eq!(output.stats.parsed_stories, 3);
    assert!(output.stats.graph_nodes >= 6); // At least user nodes + action nodes
  }

  #[test]
  fn test_user_story_to_use_case_format() {
    let story = UserStory::new(
      "test".to_string(),
      "Developer".to_string(),
      "write tests".to_string(),
      "ensure quality".to_string(),
      "original".to_string(),
    );

    assert_eq!(
      story.to_use_case_format(),
      "[Developer] can [write tests] so that [ensure quality]"
    );
  }

  #[test]
  fn test_graph_connected_components() {
    let mut graph = RequirementGraph::new()
      .with_node(RequirementNode::new(
        "a".to_string(),
        "A".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "b".to_string(),
        "B".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "c".to_string(),
        "C".to_string(),
        NodeType::User,
      ))
      .with_edge(RequirementEdge::new(
        "a".to_string(),
        "b".to_string(),
        EdgeRelationship::DependsOn,
      ));

    // c is not connected
    let components = graph.connected_components();
    assert_eq!(components.len(), 2);

    // Connect c to a
    graph = graph.with_edge(RequirementEdge::new(
      "c".to_string(),
      "a".to_string(),
      EdgeRelationship::DependsOn,
    ));
    let components = graph.connected_components();
    assert_eq!(components.len(), 1);
  }

  #[test]
  fn test_graph_nodes_by_type() {
    let graph = RequirementGraph::new()
      .with_node(RequirementNode::new(
        "u1".to_string(),
        "User1".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "a1".to_string(),
        "Action1".to_string(),
        NodeType::Action,
      ))
      .with_node(RequirementNode::new(
        "u2".to_string(),
        "User2".to_string(),
        NodeType::User,
      ));

    let users = graph.nodes_by_type(NodeType::User);
    assert_eq!(users.len(), 2);

    let actions = graph.nodes_by_type(NodeType::Action);
    assert_eq!(actions.len(), 1);
  }

  #[test]
  fn test_jtbd_creation() {
    let job = JobToBeDone::new("j1".to_string(), "Save time".to_string())
      .with_source_story("s1".to_string())
      .with_motivation("be more productive".to_string())
      .with_priority(90);

    assert_eq!(job.id, "j1");
    assert_eq!(job.job_statement, "Save time");
    assert_eq!(job.source_story_ids.len(), 1);
    assert_eq!(job.motivations.len(), 1);
    assert_eq!(job.priority, 90);
  }

  #[test]
  fn test_graph_requirement_creation() {
    let req = GraphRequirement::new("r1".to_string(), "[User] can [do thing]".to_string())
      .with_jtbd("j1".to_string())
      .with_dependency("r0".to_string())
      .with_source_story("s1".to_string())
      .with_priority(75);

    assert_eq!(req.id, "r1");
    assert_eq!(req.job_to_be_done, Some("j1".to_string()));
    assert_eq!(req.dependencies.len(), 1);
    assert_eq!(req.source_stories.len(), 1);
    assert_eq!(req.priority, 75);
  }

  #[test]
  fn test_similarity_detection() {
    let stories = vec![
      create_test_story(
        "s1",
        "As a user, I want to create documents, so that I can write content",
      ),
      create_test_story(
        "s2",
        "As a user, I want to create new documents, so that I can write",
      ),
    ];

    let output = GreatReindexingEngine::reindex(&stories).expect("Should succeed");

    // Should detect similarity edge
    let similar_edges: Vec<_> = output
      .graph
      .edges
      .iter()
      .filter(|e| e.relationship == EdgeRelationship::SimilarTo)
      .collect();

    assert!(!similar_edges.is_empty(), "Should detect similar actions");
  }

  #[test]
  fn test_extract_job_statement() {
    assert_eq!(
      GreatReindexingEngine::extract_job_statement("i can save time"),
      "Save time"
    );
    assert_eq!(
      GreatReindexingEngine::extract_job_statement("to be more productive"),
      "Be more productive"
    );
    assert_eq!(
      GreatReindexingEngine::extract_job_statement("i am able to work faster"),
      "Work faster"
    );
  }

  #[test]
  fn test_stats_calculation() {
    let stories = vec![
      create_test_story(
        "s1",
        "As a user, I want to login, so that I can access my data",
      ),
      create_test_story(
        "s2",
        "As a user, I want to logout, so that I can stay secure",
      ),
    ];

    let output = GreatReindexingEngine::reindex(&stories).expect("Should succeed");

    assert_eq!(output.stats.total_input_stories, 2);
    assert_eq!(output.stats.parsed_stories, 2);
    assert!(output.stats.graph_nodes > 0);
    assert!(output.stats.graph_edges > 0);
  }

  #[test]
  fn test_fallback_parsing() {
    let stories = vec![create_test_story("s1", "Simple feature request")];

    let output = GreatReindexingEngine::reindex(&stories).expect("Should succeed");

    assert_eq!(output.user_stories.len(), 1);
    assert_eq!(output.user_stories[0].user, "User");
    assert_eq!(output.user_stories[0].action, "Simple feature request");
  }

  #[test]
  fn test_graph_edge_relationships() {
    let graph = RequirementGraph::new()
      .with_node(RequirementNode::new(
        "u1".to_string(),
        "User".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "a1".to_string(),
        "Action".to_string(),
        NodeType::Action,
      ))
      .with_node(RequirementNode::new(
        "m1".to_string(),
        "Motivation".to_string(),
        NodeType::Motivation,
      ))
      .with_edge(RequirementEdge::new(
        "u1".to_string(),
        "a1".to_string(),
        EdgeRelationship::Performs,
      ))
      .with_edge(RequirementEdge::new(
        "a1".to_string(),
        "m1".to_string(),
        EdgeRelationship::Achieves,
      ));

    let edges_from_u1 = graph.edges_from("u1");
    assert_eq!(edges_from_u1.len(), 1);
    assert_eq!(edges_from_u1[0].relationship, EdgeRelationship::Performs);

    let edges_to_m1 = graph.edges_to("m1");
    assert_eq!(edges_to_m1.len(), 1);
    assert_eq!(edges_to_m1[0].relationship, EdgeRelationship::Achieves);
  }

  #[test]
  fn test_count_nodes_by_type() {
    let graph = RequirementGraph::new()
      .with_node(RequirementNode::new(
        "u1".to_string(),
        "User1".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "u2".to_string(),
        "User2".to_string(),
        NodeType::User,
      ))
      .with_node(RequirementNode::new(
        "a1".to_string(),
        "Action".to_string(),
        NodeType::Action,
      ));

    let counts = graph.count_nodes_by_type();
    assert_eq!(*counts.get(&NodeType::User).unwrap_or(&0), 2);
    assert_eq!(*counts.get(&NodeType::Action).unwrap_or(&0), 1);
    assert_eq!(*counts.get(&NodeType::Motivation).unwrap_or(&0), 0);
  }
}
