//! Bead Templates - Work item generation from interview sessions (WP26)
//!
//! This module provides bead (work item) generation functionality:
//! - `BeadTemplate` - JSONL-compatible bead record for output
//! - `BeadTemplateStats` - Statistics about generated beads
//! - Profile-specific bead generators
//! - 16-section enhanced CUE template
//! - JSONL output format
//!
//! ## Design Principles
//!
//! - Zero panics: All fallible operations return `Result<T, E>`
//! - Pure functions: Core logic is deterministic and side-effect free
//! - Type safety: Leverage Rust's type system for compile-time guarantees

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::intent::interview::types::{Answer, InterviewSession, Profile};

/// Error type for bead template operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BeadError {
  /// Bead title is empty
  #[error("empty bead title")]
  EmptyTitle,
  /// Bead description is empty
  #[error("empty bead description")]
  EmptyDescription,
  /// Priority value is invalid (must be 1-5)
  #[error("invalid priority: {0}")]
  InvalidPriority(u8),
  /// Profile type is missing
  #[error("missing profile type")]
  MissingProfileType,
  /// JSON serialization failed
  #[error("JSON serialization failed: {0}")]
  JsonError(String),
}

/// Bead template record for JSONL output
///
/// Represents a single work item (bead) that can be serialized to JSONL format
/// for import into issue trackers or project management tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadTemplate {
  /// Human-readable title for the work item
  pub title: String,
  /// Detailed description of the work
  pub description: String,
  /// Profile type (api, cli, event, data, workflow, ui)
  pub profile_type: String,
  /// Priority level (1-5, where 1 is highest)
  pub priority: u8,
  /// Issue type (feature, bug, task, spike)
  pub issue_type: String,
  /// Labels/tags for categorization
  pub labels: Vec<String>,
  /// AI hints for implementation guidance
  pub ai_hints: String,
  /// Acceptance criteria for completion
  pub acceptance_criteria: Vec<String>,
  /// Dependencies on other beads
  pub dependencies: Vec<String>,
}

impl Default for BeadTemplate {
  fn default() -> Self {
    Self {
      title: String::new(),
      description: String::new(),
      profile_type: String::new(),
      priority: 3,
      issue_type: "task".to_string(),
      labels: Vec::new(),
      ai_hints: String::new(),
      acceptance_criteria: Vec::new(),
      dependencies: Vec::new(),
    }
  }
}

impl BeadTemplate {
  /// Create a new bead template with validation
  ///
  /// # Errors
  /// - `BeadError::EmptyTitle` if title is empty or whitespace
  /// - `BeadError::EmptyDescription` if description is empty or whitespace
  /// - `BeadError::InvalidPriority` if priority is not 1-5
  /// - `BeadError::MissingProfileType` if `profile_type` is empty
  pub fn new(
    title: String,
    description: String,
    profile_type: String,
    priority: u8,
  ) -> Result<Self, BeadError> {
    if title.trim().is_empty() {
      return Err(BeadError::EmptyTitle);
    }
    if description.trim().is_empty() {
      return Err(BeadError::EmptyDescription);
    }
    if !(1..=5).contains(&priority) {
      return Err(BeadError::InvalidPriority(priority));
    }
    if profile_type.trim().is_empty() {
      return Err(BeadError::MissingProfileType);
    }

    Ok(Self {
      title,
      description,
      profile_type,
      priority,
      ..Self::default()
    })
  }

  /// Builder method to add a label
  #[must_use]
  pub fn with_label(mut self, label: String) -> Self {
    if !self.labels.contains(&label) {
      self.labels.push(label);
    }
    self
  }

  /// Builder method to set issue type
  #[must_use]
  pub fn with_issue_type(mut self, issue_type: String) -> Self {
    self.issue_type = issue_type;
    self
  }

  /// Builder method to set AI hints
  #[must_use]
  pub fn with_ai_hints(mut self, hints: String) -> Self {
    self.ai_hints = hints;
    self
  }

  /// Builder method to add acceptance criterion
  #[must_use]
  pub fn with_acceptance_criterion(mut self, criterion: String) -> Self {
    if !criterion.trim().is_empty() {
      self.acceptance_criteria.push(criterion);
    }
    self
  }

  /// Builder method to add dependency
  #[must_use]
  pub fn with_dependency(mut self, dependency: String) -> Self {
    if !dependency.trim().is_empty() && !self.dependencies.contains(&dependency) {
      self.dependencies.push(dependency);
    }
    self
  }
}

/// Bead template statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BeadTemplateStats {
  /// Total number of beads
  pub total_beads: usize,
  /// Count by priority level
  pub by_priority: HashMap<u8, usize>,
  /// Count by issue type
  pub by_type: HashMap<String, usize>,
  /// Count by profile type
  pub by_profile: HashMap<String, usize>,
}

impl BeadTemplateStats {
  /// Create empty stats
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Compute stats from a slice of beads
  #[must_use]
  pub fn from_beads(beads: &[BeadTemplate]) -> Self {
    let mut stats = Self::new();

    for bead in beads {
      stats.total_beads += 1;
      *stats.by_priority.entry(bead.priority).or_insert(0) += 1;
      *stats.by_type.entry(bead.issue_type.clone()).or_insert(0) += 1;
      *stats
        .by_profile
        .entry(bead.profile_type.clone())
        .or_insert(0) += 1;
    }

    stats
  }
}

/// Generate beads from an interview session.
///
/// Creates work items based on:
/// - Answers provided during the interview
/// - Features and behaviors identified
/// - Category-based prioritization
/// - Verification criteria for acceptance
///
/// # Errors
/// Returns `BeadError` if bead creation fails validation.
pub fn generate_beads_from_session(
  session: &InterviewSession,
) -> Result<Vec<BeadTemplate>, BeadError> {
  let profile_str = session.profile.as_str().to_string();
  let mut beads = Vec::new();

  // Generate beads from answers
  for answer in &session.answers {
    let bead = create_bead_from_answer(answer, &profile_str)?;
    beads.push(bead);
  }

  // Generate profile-specific beads
  let profile_beads = generate_profile_beads(session)?;
  beads.extend(profile_beads);

  Ok(beads)
}

/// Create a bead from an interview answer.
fn create_bead_from_answer(answer: &Answer, profile_type: &str) -> Result<BeadTemplate, BeadError> {
  let title = format!("Implement: {}", answer.question_text);
  let description = format!(
    "Based on answer: {}\n\nResponse: {}",
    answer.question_text, answer.response
  );

  // Determine priority from category (default to 3 if not determinable)
  let priority = 3;

  // Determine issue type from response content
  let issue_type = determine_issue_type(&answer.response);

  // Generate acceptance criteria from extracted fields
  let acceptance_criteria: Vec<String> = answer
    .extracted
    .iter()
    .map(|(key, value)| format!("{}: {}", key.replace('_', " "), value))
    .collect();

  let mut bead = BeadTemplate::new(title, description, profile_type.to_string(), priority)?
    .with_issue_type(issue_type)
    .with_label(format!("round-{}", answer.round))
    .with_label(format!("perspective-{:?}", answer.perspective).to_lowercase());

  // Add acceptance criteria
  for criterion in acceptance_criteria {
    bead = bead.with_acceptance_criterion(criterion);
  }

  // Add AI hints based on confidence
  if answer.confidence < 0.7 {
    bead = bead.with_ai_hints("Low confidence answer - may need clarification".to_string());
  }

  Ok(bead)
}

/// Determine issue type from response content.
fn determine_issue_type(response: &str) -> String {
  let lower = response.to_lowercase();

  if lower.contains("fix")
    || lower.contains("bug")
    || lower.contains("error")
    || lower.contains("issue")
  {
    "bug".to_string()
  } else if lower.contains("investigate") || lower.contains("research") || lower.contains("spike") {
    "spike".to_string()
  } else if lower.contains("feature") || lower.contains("new") || lower.contains("add") {
    "feature".to_string()
  } else {
    "task".to_string()
  }
}

/// Generate beads specific to profile type.
///
/// Creates profile-specific work items for:
/// - API: auth, endpoints, error handling
/// - CLI: command parsing, exit codes
/// - Event: event types, payloads
/// - Data: models, queries
/// - Workflow: steps, transitions
/// - UI: flows, states
///
/// # Errors
/// Returns `BeadError` if bead creation fails validation.
pub fn generate_profile_beads(session: &InterviewSession) -> Result<Vec<BeadTemplate>, BeadError> {
  let profile_str = session.profile.as_str().to_string();

  let beads = match session.profile {
    Profile::Api => generate_api_profile_beads(&profile_str, session),
    Profile::Cli => generate_cli_profile_beads(&profile_str, session),
    Profile::Event => generate_event_profile_beads(&profile_str, session),
    Profile::Data => generate_data_profile_beads(&profile_str, session),
    Profile::Workflow => generate_workflow_profile_beads(&profile_str, session),
    Profile::Ui => generate_ui_profile_beads(&profile_str, session),
  };

  Ok(beads)
}

/// Generate API profile beads.
fn generate_api_profile_beads(profile_type: &str, session: &InterviewSession) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // Authentication bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement API Authentication".to_string(),
        "Set up authentication mechanism for API endpoints. Include token validation, refresh logic, and security best practices.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("api".to_string())
                .with_label("auth".to_string())
                .with_label("security".to_string())
                .with_acceptance_criterion("Authentication middleware validates tokens".to_string())
                .with_acceptance_criterion("Invalid tokens return 401 Unauthorized".to_string())
                .with_acceptance_criterion("Token refresh endpoint works correctly".to_string()),
        );
    }

  // API Endpoints bead
  if let Ok(bead) = BeadTemplate::new(
        "Define API Endpoints".to_string(),
        "Design and implement all API endpoints based on the specification. Ensure consistent response formats and proper HTTP status codes.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("api".to_string())
                .with_label("endpoints".to_string())
                .with_acceptance_criterion("All endpoints return correct HTTP status codes".to_string())
                .with_acceptance_criterion("Response format is consistent across endpoints".to_string()),
        );
    }

  // Error Handling bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement API Error Handling".to_string(),
        "Create comprehensive error handling with proper error codes, messages, and documentation. Include both client and server error scenarios.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("api".to_string())
                .with_label("error-handling".to_string())
                .with_acceptance_criterion("4xx errors return client-friendly messages".to_string())
                .with_acceptance_criterion("5xx errors are logged with stack traces".to_string())
                .with_acceptance_criterion("Error responses follow RFC 7807 format".to_string()),
        );
    }

  // Add dependency: endpoints depend on auth
  if beads.len() >= 2 {
    beads[1] = beads[1]
      .clone()
      .with_dependency("Implement API Authentication".to_string());
  }

  // Add extracted fields as additional beads
  for answer in &session.answers {
    for (key, value) in &answer.extracted {
      if key == "base_url" || key == "auth_method" {
        continue; // Already covered above
      }

      if let Ok(bead) = BeadTemplate::new(
        format!("Configure API {}", key.replace('_', " ")),
        format!("Set up {} for the API: {}", key.replace('_', " "), value),
        profile_type.to_string(),
        3,
      ) {
        beads.push(
          bead
            .with_issue_type("task".to_string())
            .with_label("api".to_string())
            .with_label("config".to_string()),
        );
      }
    }
  }

  beads
}

/// Generate CLI profile beads.
fn generate_cli_profile_beads(profile_type: &str, session: &InterviewSession) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // Command Parsing bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement CLI Command Parsing".to_string(),
        "Set up command-line argument parsing with support for subcommands, flags, and options. Include help text generation.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("cli".to_string())
                .with_label("parsing".to_string())
                .with_acceptance_criterion("Parses all documented flags and options".to_string())
                .with_acceptance_criterion("Generates help text with --help flag".to_string())
                .with_acceptance_criterion("Invalid arguments return non-zero exit code".to_string()),
        );
    }

  // Exit Codes bead
  if let Ok(bead) = BeadTemplate::new(
        "Define CLI Exit Codes".to_string(),
        "Implement standardized exit codes for different scenarios. Document exit codes in help text and README.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("cli".to_string())
                .with_label("exit-codes".to_string())
                .with_acceptance_criterion("Exit code 0 for success".to_string())
                .with_acceptance_criterion("Exit code 1 for general errors".to_string())
                .with_acceptance_criterion("Exit code 2 for argument errors".to_string()),
        );
    }

  // Help System bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement CLI Help System".to_string(),
        "Create comprehensive help system with examples, usage patterns, and detailed flag descriptions.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("cli".to_string())
                .with_label("help".to_string())
                .with_acceptance_criterion("Help text includes usage examples".to_string())
                .with_acceptance_criterion("Each flag has description and default value".to_string()),
        );
    }

  // Add extracted fields as additional beads
  for answer in &session.answers {
    for (key, value) in &answer.extracted {
      if key == "command_name" || key == "exit_codes" {
        continue; // Already covered above
      }

      if let Ok(bead) = BeadTemplate::new(
        format!("Configure CLI {}", key.replace('_', " ")),
        format!("Set up {} for the CLI: {}", key.replace('_', " "), value),
        profile_type.to_string(),
        3,
      ) {
        beads.push(
          bead
            .with_issue_type("task".to_string())
            .with_label("cli".to_string())
            .with_label("config".to_string()),
        );
      }
    }
  }

  beads
}

/// Generate Event profile beads.
fn generate_event_profile_beads(
  profile_type: &str,
  _session: &InterviewSession,
) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // Event Type Definition bead
  if let Ok(bead) = BeadTemplate::new(
        "Define Event Types".to_string(),
        "Create type definitions for all event types. Include versioning, schema validation, and backward compatibility considerations.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("event".to_string())
                .with_label("schema".to_string())
                .with_acceptance_criterion("All event types have unique identifiers".to_string())
                .with_acceptance_criterion("Schema validation rejects invalid events".to_string())
                .with_acceptance_criterion("Event versioning is supported".to_string()),
        );
    }

  // Event Payload bead
  if let Ok(bead) = BeadTemplate::new(
        "Define Event Payloads".to_string(),
        "Design payload structures for each event type. Include required fields, optional fields, and data validation.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("event".to_string())
                .with_label("payload".to_string())
                .with_acceptance_criterion("Payloads are serializable to JSON".to_string())
                .with_acceptance_criterion("Required fields are validated on produce".to_string()),
        );
    }

  // Event Trigger bead
  if let Ok(bead) = BeadTemplate::new(
    "Implement Event Triggers".to_string(),
    "Set up event production triggers for each event type. Include error handling and retry logic."
      .to_string(),
    profile_type.to_string(),
    2,
  ) {
    beads.push(
      bead
        .with_issue_type("feature".to_string())
        .with_label("event".to_string())
        .with_label("trigger".to_string())
        .with_acceptance_criterion("Events are produced on defined triggers".to_string())
        .with_acceptance_criterion("Failed publishes are logged and retried".to_string()),
    );
  }

  beads
}

/// Generate Data profile beads.
fn generate_data_profile_beads(
  profile_type: &str,
  _session: &InterviewSession,
) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // Data Model bead
  if let Ok(bead) = BeadTemplate::new(
        "Design Data Model".to_string(),
        "Create the data model with entities, relationships, and constraints. Include indexes and performance considerations.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("data".to_string())
                .with_label("model".to_string())
                .with_acceptance_criterion("All entities have primary keys".to_string())
                .with_acceptance_criterion("Relationships are properly defined".to_string())
                .with_acceptance_criterion("Indexes exist for frequently queried fields".to_string()),
        );
    }

  // Data Queries bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement Data Queries".to_string(),
        "Create query functions for all access patterns. Include pagination, filtering, and sorting support.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("data".to_string())
                .with_label("queries".to_string())
                .with_acceptance_criterion("Queries support pagination".to_string())
                .with_acceptance_criterion("Queries support filtering by key fields".to_string())
                .with_acceptance_criterion("Query performance meets SLA".to_string()),
        );
    }

  // Data Retention bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement Data Retention Policy".to_string(),
        "Set up data retention and archival policies. Include cleanup jobs and compliance requirements.".to_string(),
        profile_type.to_string(),
        3,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("data".to_string())
                .with_label("retention".to_string())
                .with_acceptance_criterion("Old data is archived per policy".to_string())
                .with_acceptance_criterion("Cleanup jobs run on schedule".to_string()),
        );
    }

  beads
}

/// Generate Workflow profile beads.
fn generate_workflow_profile_beads(
  profile_type: &str,
  _session: &InterviewSession,
) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // Workflow Steps bead
  if let Ok(bead) = BeadTemplate::new(
        "Define Workflow Steps".to_string(),
        "Define all workflow steps with inputs, outputs, and dependencies. Include step-level error handling.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("workflow".to_string())
                .with_label("steps".to_string())
                .with_acceptance_criterion("Each step has defined inputs and outputs".to_string())
                .with_acceptance_criterion("Step dependencies are explicit".to_string())
                .with_acceptance_criterion("Step failures are handled gracefully".to_string()),
        );
    }

  // Workflow Transitions bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement Workflow Transitions".to_string(),
        "Create state transition logic with validation guards and side effects. Include rollback support.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("workflow".to_string())
                .with_label("transitions".to_string())
                .with_acceptance_criterion("Invalid transitions are rejected".to_string())
                .with_acceptance_criterion("Transitions are idempotent".to_string())
                .with_acceptance_criterion("Rollback is supported for critical transitions".to_string()),
        );
    }

  // Error Recovery bead
  if let Ok(bead) = BeadTemplate::new(
        "Implement Workflow Error Recovery".to_string(),
        "Add error recovery mechanisms including retry logic, compensation actions, and dead letter queues.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("workflow".to_string())
                .with_label("error-recovery".to_string())
                .with_acceptance_criterion("Failed workflows can be retried".to_string())
                .with_acceptance_criterion("Compensation actions run on failure".to_string())
                .with_acceptance_criterion("Unrecoverable errors go to dead letter queue".to_string()),
        );
    }

  beads
}

/// Generate UI profile beads.
fn generate_ui_profile_beads(profile_type: &str, _session: &InterviewSession) -> Vec<BeadTemplate> {
  let mut beads = Vec::new();

  // User Flows bead
  if let Ok(bead) = BeadTemplate::new(
        "Define User Flows".to_string(),
        "Map out all user flows with entry points, decision points, and outcomes. Include flow diagrams in documentation.".to_string(),
        profile_type.to_string(),
        1,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("ui".to_string())
                .with_label("flows".to_string())
                .with_acceptance_criterion("All user flows are documented".to_string())
                .with_acceptance_criterion("Happy path is clearly defined".to_string())
                .with_acceptance_criterion("Error paths are included".to_string()),
        );
    }

  // UI States bead
  if let Ok(bead) = BeadTemplate::new(
        "Define UI States".to_string(),
        "Define all UI states including loading, error, empty, and success states. Include state transition logic.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("ui".to_string())
                .with_label("state".to_string())
                .with_acceptance_criterion("Loading states show progress indicators".to_string())
                .with_acceptance_criterion("Error states show actionable messages".to_string())
                .with_acceptance_criterion("Empty states guide users to action".to_string()),
        );
    }

  // UI Components bead
  if let Ok(bead) = BeadTemplate::new(
        "Build UI Components".to_string(),
        "Create reusable UI components following design system guidelines. Include accessibility support.".to_string(),
        profile_type.to_string(),
        2,
    ) {
        beads.push(
            bead.with_issue_type("feature".to_string())
                .with_label("ui".to_string())
                .with_label("components".to_string())
                .with_acceptance_criterion("Components follow design system".to_string())
                .with_acceptance_criterion("Components are accessible (WCAG 2.1 AA)".to_string())
                .with_acceptance_criterion("Components have storybook documentation".to_string()),
        );
    }

  beads
}

/// Serialize beads to JSONL format.
///
/// Each bead is serialized as a JSON object on a single line,
/// with lines separated by newlines.
///
/// # Errors
/// Returns `BeadError::JsonError` if serialization fails.
pub fn beads_to_jsonl(beads: &[BeadTemplate]) -> Result<String, BeadError> {
  beads
    .iter()
    .map(|bead| serde_json::to_string(bead).map_err(|e| BeadError::JsonError(e.to_string())))
    .collect::<Result<Vec<String>, BeadError>>()
    .map(|lines| lines.join("\n"))
}

/// Helper to write metadata sections
fn write_metadata_sections(output: &mut String, beads: &[BeadTemplate]) {
  use std::fmt::Write;
  // Header
  let _ = writeln!(output, "// Enhanced CUE Template for Work Items");
  let _ = writeln!(output, "// Generated by clarity-web intent beads module\n");
  let _ = writeln!(output, "package beads\n");

  // Section 1: Metadata
  let _ = writeln!(output, "// Section 1: Metadata");
  let _ = writeln!(output, "metadata: {{");
  let _ = writeln!(output, "    total_beads: {}", beads.len());
  let _ = writeln!(output, "    format_version: \"1.0\"");
  let _ = writeln!(output, "    generator: \"clarity-web-intent-beads\"");
  let _ = writeln!(output, "}}\n");

  // Section 2: Profile Types
  let _ = writeln!(output, "// Section 2: Profile Types");
  let _ = writeln!(output, "profile_types: {{");
  let profiles: std::collections::HashSet<&str> =
    beads.iter().map(|b| b.profile_type.as_str()).collect();
  for profile in profiles {
    let count = beads.iter().filter(|b| b.profile_type == profile).count();
    let _ = writeln!(output, "    {profile}: {count}");
  }
  let _ = writeln!(output, "}}\n");

  // Section 3: Issue Types
  let _ = writeln!(output, "// Section 3: Issue Types");
  let _ = writeln!(output, "issue_types: {{");
  let types: std::collections::HashSet<&str> =
    beads.iter().map(|b| b.issue_type.as_str()).collect();
  for issue_type in types {
    let count = beads.iter().filter(|b| b.issue_type == issue_type).count();
    let _ = writeln!(output, "    {issue_type}: {count}");
  }
  let _ = writeln!(output, "}}\n");

  // Section 4: Priority Distribution
  let _ = writeln!(output, "// Section 4: Priority Distribution");
  let _ = writeln!(output, "priority_distribution: {{");
  for priority in 1..=5 {
    let count = beads.iter().filter(|b| b.priority == priority).count();
    let _ = writeln!(output, "    p{priority}: {count}");
  }
  let _ = writeln!(output, "}}\n");

  // Section 5: Labels Index
  let _ = writeln!(output, "// Section 5: Labels Index");
  let _ = writeln!(output, "labels_index: {{");
  let mut all_labels: std::collections::HashSet<&str> = std::collections::HashSet::new();
  for bead in beads {
    for label in &bead.labels {
      all_labels.insert(label.as_str());
    }
  }
  for label in all_labels {
    let _ = writeln!(output, "    \"{label}\": true");
  }
  let _ = writeln!(output, "}}\n");

  // Section 6: Dependencies Graph
  let _ = writeln!(output, "// Section 6: Dependencies Graph");
  let _ = writeln!(output, "dependencies_graph: {{");
  for bead in beads {
    if !bead.dependencies.is_empty() {
      let _ = writeln!(output, "    \"{}\": [", escape_cue_string(&bead.title));
      for dep in &bead.dependencies {
        let _ = writeln!(output, "        \"{}\",", escape_cue_string(dep));
      }
      let _ = writeln!(output, "    ]");
    }
  }
  let _ = writeln!(output, "}}\n");
}

/// Helper to write individual bead records
fn write_bead_records(output: &mut String, beads: &[BeadTemplate]) -> Result<(), BeadError> {
  use std::fmt::Write;
  let _ = writeln!(output, "// Sections 7-16: Bead Records");
  let _ = writeln!(output, "beads: [");

  for (index, bead) in beads.iter().enumerate() {
    if bead.title.trim().is_empty() {
      return Err(BeadError::EmptyTitle);
    }

    let _ = writeln!(output, "    {{");
    let _ = writeln!(output, "        // Section 7: Record {}", index + 1);
    let _ = writeln!(output, "        id: \"bead-{}\"", index + 1);
    let _ = writeln!(
      output,
      "        title: \"{}\"",
      escape_cue_string(&bead.title)
    );
    let _ = writeln!(
      output,
      "        description: \"{}\"",
      escape_cue_string(&bead.description)
    );

    let _ = writeln!(output, "        // Section 8: Classification");
    let _ = writeln!(output, "        profile_type: \"{}\"", bead.profile_type);
    let _ = writeln!(output, "        issue_type: \"{}\"", bead.issue_type);
    let _ = writeln!(output, "        priority: {}", bead.priority);

    let _ = writeln!(output, "        // Section 9: Labels");
    let _ = writeln!(output, "        labels: [");
    for label in &bead.labels {
      let _ = writeln!(output, "            \"{}\",", escape_cue_string(label));
    }
    let _ = writeln!(output, "        ]");

    let _ = writeln!(output, "        // Section 10: AI Hints");
    let _ = writeln!(
      output,
      "        ai_hints: \"{}\"",
      escape_cue_string(&bead.ai_hints)
    );

    let _ = writeln!(output, "        // Section 11: Acceptance Criteria");
    let _ = writeln!(output, "        acceptance_criteria: [");
    for criterion in &bead.acceptance_criteria {
      let _ = writeln!(output, "            \"{}\",", escape_cue_string(criterion));
    }
    let _ = writeln!(output, "        ]");

    let _ = writeln!(output, "        // Section 12: Dependencies");
    let _ = writeln!(output, "        dependencies: [");
    for dep in &bead.dependencies {
      let _ = writeln!(output, "            \"{}\",", escape_cue_string(dep));
    }
    let _ = writeln!(output, "        ]");

    let _ = writeln!(output, "        // Section 13: Effort Estimate");
    let _ = writeln!(
      output,
      "        effort_estimate: {}",
      estimate_effort_from_priority(bead.priority)
    );

    let _ = writeln!(output, "        // Section 14: Risk Assessment");
    let _ = writeln!(
      output,
      "        risk_level: \"{}\"",
      assess_risk_level(bead)
    );

    let _ = writeln!(output, "        // Section 15: Implementation Notes");
    let _ = writeln!(output, "        implementation_notes: \"\"");

    let _ = writeln!(output, "        // Section 16: Verification Status");
    let _ = writeln!(output, "        verification_status: \"pending\"");

    let _ = writeln!(output, "    }}");
  }

  let _ = writeln!(output, "]");
  Ok(())
}

/// Generate enhanced CUE format with 16-section template.
///
/// Creates a CUE configuration document with all bead metadata
/// organized into 16 standard sections for maximum interoperability.
///
/// # Errors
/// Returns `BeadError::EmptyTitle` if any bead has an empty title.
pub fn beads_to_enhanced_cue(beads: &[BeadTemplate]) -> Result<String, BeadError> {
  if beads.is_empty() {
    return Ok(String::new());
  }

  let mut output = String::new();
  write_metadata_sections(&mut output, beads);
  write_bead_records(&mut output, beads)?;

  Ok(output)
}

/// Escape a string for CUE format.
fn escape_cue_string(s: &str) -> String {
  s.replace('\\', "\\\\")
    .replace('"', "\\\"")
    .replace('\n', "\\n")
    .replace('\r', "\\r")
    .replace('\t', "\\t")
}

/// Estimate effort from priority.
const fn estimate_effort_from_priority(priority: u8) -> u8 {
  match priority {
    1 => 5, // High priority = more effort
    2 => 4,
    3 => 3,
    4 => 2,
    _ => 1, // Low priority = less effort
  }
}

/// Assess risk level based on bead characteristics.
const fn assess_risk_level(bead: &BeadTemplate) -> &'static str {
  if bead.priority <= 2 && bead.dependencies.len() > 2 {
    "high"
  } else if bead.priority <= 2 || bead.dependencies.len() > 1 {
    "medium"
  } else {
    "low"
  }
}

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
  use crate::intent::interview::types::{InterviewStage, Perspective};
  use std::collections::HashMap;

  fn create_test_session() -> InterviewSession {
    let mut session = InterviewSession::new(
      "test-session".to_string(),
      Profile::Api,
      "2026-02-27T00:00:00Z".to_string(),
    );
    session.stage = InterviewStage::Refinement;
    session
  }

  fn create_test_session_with_answers() -> InterviewSession {
    let mut session = create_test_session();

    let mut extracted = HashMap::new();
    extracted.insert(
      "base_url".to_string(),
      "https://api.example.com".to_string(),
    );
    extracted.insert("auth_method".to_string(), "Bearer".to_string());

    session.answers.push(Answer {
      question_id: "q1".to_string(),
      question_text: "What is the API base URL?".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "The base URL is https://api.example.com with Bearer authentication".to_string(),
      extracted,
      confidence: 0.9,
      notes: String::new(),
      timestamp: "2026-02-27T00:01:00Z".to_string(),
    });

    session
  }

  #[test]
  fn test_bead_error_display() {
    let err = BeadError::EmptyTitle;
    assert!(format!("{err}").contains("empty"));

    let err = BeadError::EmptyDescription;
    assert!(format!("{err}").contains("empty"));

    let err = BeadError::InvalidPriority(10);
    assert!(format!("{err}").contains("10"));

    let err = BeadError::MissingProfileType;
    assert!(format!("{err}").contains("profile"));

    let err = BeadError::JsonError("test error".to_string());
    assert!(format!("{err}").contains("test error"));
  }

  #[test]
  fn test_bead_template_new_valid() {
    let bead = BeadTemplate::new(
      "Test Bead".to_string(),
      "Test description".to_string(),
      "api".to_string(),
      3,
    );

    assert!(bead.is_ok());
    let bead = bead.expect("valid bead");
    assert_eq!(bead.title, "Test Bead");
    assert_eq!(bead.description, "Test description");
    assert_eq!(bead.profile_type, "api");
    assert_eq!(bead.priority, 3);
    assert_eq!(bead.issue_type, "task");
    assert!(bead.labels.is_empty());
  }

  #[test]
  fn test_bead_template_new_empty_title() {
    let result = BeadTemplate::new(
      String::new(),
      "Test description".to_string(),
      "api".to_string(),
      3,
    );
    assert!(matches!(result, Err(BeadError::EmptyTitle)));
  }

  #[test]
  fn test_bead_template_new_whitespace_title() {
    let result = BeadTemplate::new(
      "   ".to_string(),
      "Test description".to_string(),
      "api".to_string(),
      3,
    );
    assert!(matches!(result, Err(BeadError::EmptyTitle)));
  }

  #[test]
  fn test_bead_template_new_empty_description() {
    let result = BeadTemplate::new("Test Bead".to_string(), String::new(), "api".to_string(), 3);
    assert!(matches!(result, Err(BeadError::EmptyDescription)));
  }

  #[test]
  fn test_bead_template_new_invalid_priority_low() {
    let result = BeadTemplate::new(
      "Test Bead".to_string(),
      "Test description".to_string(),
      "api".to_string(),
      0,
    );
    assert!(matches!(result, Err(BeadError::InvalidPriority(0))));
  }

  #[test]
  fn test_bead_template_new_invalid_priority_high() {
    let result = BeadTemplate::new(
      "Test Bead".to_string(),
      "Test description".to_string(),
      "api".to_string(),
      6,
    );
    assert!(matches!(result, Err(BeadError::InvalidPriority(6))));
  }

  #[test]
  fn test_bead_template_new_empty_profile() {
    let result = BeadTemplate::new(
      "Test Bead".to_string(),
      "Test description".to_string(),
      String::new(),
      3,
    );
    assert!(matches!(result, Err(BeadError::MissingProfileType)));
  }

  #[test]
  fn test_bead_template_builder_methods() {
    let bead = BeadTemplate::new(
      "Test Bead".to_string(),
      "Test description".to_string(),
      "api".to_string(),
      2,
    )
    .expect("valid bead")
    .with_label("api".to_string())
    .with_label("auth".to_string())
    .with_issue_type("feature".to_string())
    .with_ai_hints("Use JWT tokens".to_string())
    .with_acceptance_criterion("Token validation works".to_string())
    .with_acceptance_criterion("Refresh works".to_string())
    .with_dependency("Setup auth server".to_string());

    assert_eq!(bead.title, "Test Bead");
    assert_eq!(bead.labels, vec!["api", "auth"]);
    assert_eq!(bead.issue_type, "feature");
    assert_eq!(bead.ai_hints, "Use JWT tokens");
    assert_eq!(bead.acceptance_criteria.len(), 2);
    assert_eq!(bead.dependencies, vec!["Setup auth server"]);
  }

  #[test]
  fn test_bead_template_duplicate_labels() {
    let bead = BeadTemplate::new("Test".to_string(), "Desc".to_string(), "api".to_string(), 3)
      .expect("valid")
      .with_label("api".to_string())
      .with_label("api".to_string())
      .with_label("test".to_string());

    assert_eq!(bead.labels, vec!["api", "test"]);
  }

  #[test]
  fn test_bead_template_duplicate_dependencies() {
    let bead = BeadTemplate::new("Test".to_string(), "Desc".to_string(), "api".to_string(), 3)
      .expect("valid")
      .with_dependency("dep1".to_string())
      .with_dependency("dep1".to_string())
      .with_dependency("dep2".to_string());

    assert_eq!(bead.dependencies, vec!["dep1", "dep2"]);
  }

  #[test]
  fn test_bead_template_empty_acceptance_criterion() {
    let bead = BeadTemplate::new("Test".to_string(), "Desc".to_string(), "api".to_string(), 3)
      .expect("valid")
      .with_acceptance_criterion(String::new())
      .with_acceptance_criterion("   ".to_string())
      .with_acceptance_criterion("Valid criterion".to_string());

    assert_eq!(bead.acceptance_criteria.len(), 1);
  }

  #[test]
  fn test_bead_template_stats_new() {
    let stats = BeadTemplateStats::new();
    assert_eq!(stats.total_beads, 0);
    assert!(stats.by_priority.is_empty());
    assert!(stats.by_type.is_empty());
    assert!(stats.by_profile.is_empty());
  }

  #[test]
  fn test_bead_template_stats_from_beads_empty() {
    let stats = BeadTemplateStats::from_beads(&[]);
    assert_eq!(stats.total_beads, 0);
  }

  #[test]
  fn test_bead_template_stats_from_beads() {
    let beads = vec![
      BeadTemplate {
        title: "Bead 1".to_string(),
        description: "Desc 1".to_string(),
        profile_type: "api".to_string(),
        priority: 1,
        issue_type: "feature".to_string(),
        labels: vec!["api".to_string()],
        ai_hints: String::new(),
        acceptance_criteria: vec![],
        dependencies: vec![],
      },
      BeadTemplate {
        title: "Bead 2".to_string(),
        description: "Desc 2".to_string(),
        profile_type: "api".to_string(),
        priority: 1,
        issue_type: "bug".to_string(),
        labels: vec!["api".to_string()],
        ai_hints: String::new(),
        acceptance_criteria: vec![],
        dependencies: vec![],
      },
      BeadTemplate {
        title: "Bead 3".to_string(),
        description: "Desc 3".to_string(),
        profile_type: "cli".to_string(),
        priority: 3,
        issue_type: "feature".to_string(),
        labels: vec!["cli".to_string()],
        ai_hints: String::new(),
        acceptance_criteria: vec![],
        dependencies: vec![],
      },
    ];

    let stats = BeadTemplateStats::from_beads(&beads);

    assert_eq!(stats.total_beads, 3);
    assert_eq!(*stats.by_priority.get(&1).unwrap_or(&0), 2);
    assert_eq!(*stats.by_priority.get(&3).unwrap_or(&0), 1);
    assert_eq!(*stats.by_type.get("feature").unwrap_or(&0), 2);
    assert_eq!(*stats.by_type.get("bug").unwrap_or(&0), 1);
    assert_eq!(*stats.by_profile.get("api").unwrap_or(&0), 2);
    assert_eq!(*stats.by_profile.get("cli").unwrap_or(&0), 1);
  }

  #[test]
  fn test_determine_issue_type_bug() {
    assert_eq!(determine_issue_type("fix the bug"), "bug");
    assert_eq!(determine_issue_type("error handling"), "bug");
    assert_eq!(determine_issue_type("this is an issue"), "bug");
  }

  #[test]
  fn test_determine_issue_type_spike() {
    assert_eq!(determine_issue_type("investigate the problem"), "spike");
    assert_eq!(determine_issue_type("research new approach"), "spike");
    assert_eq!(determine_issue_type("spike for performance"), "spike");
  }

  #[test]
  fn test_determine_issue_type_feature() {
    assert_eq!(determine_issue_type("add new feature"), "feature");
    assert_eq!(determine_issue_type("new functionality"), "feature");
  }

  #[test]
  fn test_determine_issue_type_task() {
    assert_eq!(determine_issue_type("update the config"), "task");
    assert_eq!(determine_issue_type("refactor code"), "task");
  }

  #[test]
  fn test_generate_beads_from_session_empty() {
    let session = create_test_session();
    let result = generate_beads_from_session(&session);

    assert!(result.is_ok());
    let beads = result.expect("beads");
    // Should still generate profile-specific beads
    assert!(!beads.is_empty());
  }

  #[test]
  fn test_generate_beads_from_session_with_answers() {
    let session = create_test_session_with_answers();
    let result = generate_beads_from_session(&session);

    assert!(result.is_ok());
    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    // Should contain at least one bead from the answer
    let answer_beads: Vec<&BeadTemplate> = beads
      .iter()
      .filter(|b| b.title.contains("Implement:"))
      .collect();
    assert!(!answer_beads.is_empty());
  }

  #[test]
  fn test_generate_profile_beads_api() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Api,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    // Should contain API-specific beads
    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("API")));
    assert!(titles
      .iter()
      .any(|t| t.contains("Authentication") || t.contains("Auth")));
  }

  #[test]
  fn test_generate_profile_beads_cli() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Cli,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    // Should contain CLI-specific beads
    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("CLI")));
  }

  #[test]
  fn test_generate_profile_beads_event() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Event,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("Event")));
  }

  #[test]
  fn test_generate_profile_beads_data() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Data,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("Data")));
  }

  #[test]
  fn test_generate_profile_beads_workflow() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Workflow,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("Workflow")));
  }

  #[test]
  fn test_generate_profile_beads_ui() {
    let session = InterviewSession::new(
      "test".to_string(),
      Profile::Ui,
      "2026-02-27T00:00:00Z".to_string(),
    );

    let result = generate_profile_beads(&session);
    assert!(result.is_ok());

    let beads = result.expect("beads");
    assert!(!beads.is_empty());

    let titles: Vec<&str> = beads.iter().map(|b| b.title.as_str()).collect();
    assert!(titles
      .iter()
      .any(|t| t.contains("UI") || t.contains("User")));
  }

  #[test]
  fn test_beads_to_jsonl_empty() {
    let result = beads_to_jsonl(&[]);
    assert!(result.is_ok());
    assert!(result.expect("jsonl").is_empty());
  }

  #[test]
  fn test_beads_to_jsonl_single() {
    let beads = vec![BeadTemplate {
      title: "Test Bead".to_string(),
      description: "Test description".to_string(),
      profile_type: "api".to_string(),
      priority: 3,
      issue_type: "task".to_string(),
      labels: vec!["api".to_string()],
      ai_hints: String::new(),
      acceptance_criteria: vec!["Criterion 1".to_string()],
      dependencies: vec![],
    }];

    let result = beads_to_jsonl(&beads);
    assert!(result.is_ok());

    let jsonl = result.expect("jsonl");
    assert!(jsonl.contains("\"title\":\"Test Bead\""));
    assert!(jsonl.contains("\"priority\":3"));
    assert!(!jsonl.contains('\n')); // Single line
  }

  #[test]
  fn test_beads_to_jsonl_multiple() {
    let beads = vec![
      BeadTemplate {
        title: "Bead 1".to_string(),
        description: "Desc 1".to_string(),
        profile_type: "api".to_string(),
        priority: 1,
        issue_type: "feature".to_string(),
        labels: vec![],
        ai_hints: String::new(),
        acceptance_criteria: vec![],
        dependencies: vec![],
      },
      BeadTemplate {
        title: "Bead 2".to_string(),
        description: "Desc 2".to_string(),
        profile_type: "cli".to_string(),
        priority: 2,
        issue_type: "bug".to_string(),
        labels: vec![],
        ai_hints: String::new(),
        acceptance_criteria: vec![],
        dependencies: vec![],
      },
    ];

    let result = beads_to_jsonl(&beads);
    assert!(result.is_ok());

    let jsonl = result.expect("jsonl");
    let lines: Vec<&str> = jsonl.split('\n').collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Bead 1"));
    assert!(lines[1].contains("Bead 2"));
  }

  #[test]
  fn test_beads_to_enhanced_cue_empty() {
    let result = beads_to_enhanced_cue(&[]);
    assert!(result.is_ok());
    assert!(result.expect("cue").is_empty());
  }

  #[test]
  fn test_beads_to_enhanced_cue_single() {
    let beads = vec![BeadTemplate {
      title: "Test Bead".to_string(),
      description: "Test description".to_string(),
      profile_type: "api".to_string(),
      priority: 1,
      issue_type: "feature".to_string(),
      labels: vec!["api".to_string(), "auth".to_string()],
      ai_hints: "Use JWT".to_string(),
      acceptance_criteria: vec!["Must work".to_string()],
      dependencies: vec!["Setup server".to_string()],
    }];

    let result = beads_to_enhanced_cue(&beads);
    assert!(result.is_ok());

    let cue = result.expect("cue");
    assert!(cue.contains("package beads"));
    assert!(cue.contains("metadata:"));
    assert!(cue.contains("profile_types:"));
    assert!(cue.contains("issue_types:"));
    assert!(cue.contains("priority_distribution:"));
    assert!(cue.contains("labels_index:"));
    assert!(cue.contains("dependencies_graph:"));
    assert!(cue.contains("beads:"));
    assert!(cue.contains("Test Bead"));
    assert!(cue.contains("Section 1:"));
    assert!(cue.contains("Section 16:"));
  }

  #[test]
  fn test_beads_to_enhanced_cue_empty_title() {
    let beads = vec![BeadTemplate {
      title: String::new(),
      description: "Test".to_string(),
      profile_type: "api".to_string(),
      priority: 3,
      issue_type: "task".to_string(),
      labels: vec![],
      ai_hints: String::new(),
      acceptance_criteria: vec![],
      dependencies: vec![],
    }];

    let result = beads_to_enhanced_cue(&beads);
    assert!(matches!(result, Err(BeadError::EmptyTitle)));
  }

  #[test]
  fn test_beads_to_enhanced_cue_escapes_special_chars() {
    let beads = vec![BeadTemplate {
      title: "Test \"quoted\" bead".to_string(),
      description: "Line 1\nLine 2".to_string(),
      profile_type: "api".to_string(),
      priority: 3,
      issue_type: "task".to_string(),
      labels: vec!["test\\label".to_string()],
      ai_hints: String::new(),
      acceptance_criteria: vec![],
      dependencies: vec![],
    }];

    let result = beads_to_enhanced_cue(&beads);
    assert!(result.is_ok());

    let cue = result.expect("cue");
    assert!(cue.contains("\\\"quoted\\\""));
    assert!(cue.contains("\\n"));
    assert!(cue.contains("test\\\\label"));
  }

  #[test]
  fn test_estimate_effort_from_priority() {
    assert_eq!(estimate_effort_from_priority(1), 5);
    assert_eq!(estimate_effort_from_priority(2), 4);
    assert_eq!(estimate_effort_from_priority(3), 3);
    assert_eq!(estimate_effort_from_priority(4), 2);
    assert_eq!(estimate_effort_from_priority(5), 1);
    assert_eq!(estimate_effort_from_priority(0), 3); // Invalid defaults to 3
    assert_eq!(estimate_effort_from_priority(10), 3); // Invalid defaults to 3
  }

  #[test]
  fn test_assess_risk_level_high() {
    let bead = BeadTemplate {
      title: "Test".to_string(),
      description: "Test".to_string(),
      profile_type: "api".to_string(),
      priority: 1,
      issue_type: "task".to_string(),
      labels: vec![],
      ai_hints: String::new(),
      acceptance_criteria: vec![],
      dependencies: vec!["dep1".to_string(), "dep2".to_string(), "dep3".to_string()],
    };

    assert_eq!(assess_risk_level(&bead), "high");
  }

  #[test]
  fn test_assess_risk_level_medium() {
    let bead = BeadTemplate {
      title: "Test".to_string(),
      description: "Test".to_string(),
      profile_type: "api".to_string(),
      priority: 2,
      issue_type: "task".to_string(),
      labels: vec![],
      ai_hints: String::new(),
      acceptance_criteria: vec![],
      dependencies: vec!["dep1".to_string()],
    };

    assert_eq!(assess_risk_level(&bead), "medium");
  }

  #[test]
  fn test_assess_risk_level_low() {
    let bead = BeadTemplate {
      title: "Test".to_string(),
      description: "Test".to_string(),
      profile_type: "api".to_string(),
      priority: 5,
      issue_type: "task".to_string(),
      labels: vec![],
      ai_hints: String::new(),
      acceptance_criteria: vec![],
      dependencies: vec![],
    };

    assert_eq!(assess_risk_level(&bead), "low");
  }

  #[test]
  fn test_bead_template_serde_roundtrip() {
    let bead = BeadTemplate {
      title: "Test Bead".to_string(),
      description: "Test description".to_string(),
      profile_type: "api".to_string(),
      priority: 2,
      issue_type: "feature".to_string(),
      labels: vec!["api".to_string(), "auth".to_string()],
      ai_hints: "Use JWT".to_string(),
      acceptance_criteria: vec!["Must work".to_string(), "Must be fast".to_string()],
      dependencies: vec!["Setup server".to_string()],
    };

    let json = serde_json::to_string(&bead).expect("should serialize");
    let parsed: BeadTemplate = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(bead, parsed);
  }

  #[test]
  fn test_bead_template_stats_serde_roundtrip() {
    let mut stats = BeadTemplateStats::new();
    stats.total_beads = 10;
    stats.by_priority.insert(1, 3);
    stats.by_priority.insert(2, 7);
    stats.by_type.insert("feature".to_string(), 6);
    stats.by_type.insert("bug".to_string(), 4);
    stats.by_profile.insert("api".to_string(), 10);

    let json = serde_json::to_string(&stats).expect("should serialize");
    let parsed: BeadTemplateStats = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(stats, parsed);
  }

  #[test]
  fn test_create_bead_from_answer_low_confidence() {
    let answer = Answer {
      question_id: "q1".to_string(),
      question_text: "Test question?".to_string(),
      perspective: Perspective::Developer,
      round: 1,
      response: "Test response".to_string(),
      extracted: HashMap::new(),
      confidence: 0.5, // Low confidence
      notes: String::new(),
      timestamp: "2026-02-27T00:00:00Z".to_string(),
    };

    let result = create_bead_from_answer(&answer, "api");
    assert!(result.is_ok());

    let bead = result.expect("bead");
    assert!(bead.ai_hints.contains("Low confidence"));
  }

  #[test]
  fn test_escape_cue_string() {
    assert_eq!(escape_cue_string("hello"), "hello");
    assert_eq!(escape_cue_string("hello\"world"), "hello\\\"world");
    assert_eq!(escape_cue_string("line1\nline2"), "line1\\nline2");
    assert_eq!(escape_cue_string("tab\there"), "tab\\there");
    assert_eq!(escape_cue_string("back\\slash"), "back\\\\slash");
  }

  #[test]
  fn test_generate_all_profile_beads() {
    let profiles = [
      Profile::Api,
      Profile::Cli,
      Profile::Event,
      Profile::Data,
      Profile::Workflow,
      Profile::Ui,
    ];

    for profile in profiles {
      let session = InterviewSession::new(
        "test".to_string(),
        profile,
        "2026-02-27T00:00:00Z".to_string(),
      );

      let result = generate_profile_beads(&session);
      assert!(result.is_ok(), "Profile {profile:?} should generate beads");

      let beads = result.expect("beads");
      assert!(
        !beads.is_empty(),
        "Profile {profile:?} should have non-empty beads"
      );

      // All beads should have the correct profile type
      for bead in &beads {
        assert_eq!(bead.profile_type, profile.as_str());
      }
    }
  }
}
