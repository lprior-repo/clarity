//! Phase Deliver Component
//!
//! Delivery phase (Left of Diamond) - Validation and handoff.
//! Contains validation view, dependency graph view, and handoff view.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::components::{SectionLabel, SectionLevel};
use crate::planner::state::{PlannerState, SelectedEntity};
use crate::planner::types::{DiamondPhase, GraphHealth, ValidationCheck, ValidationSeverity};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Phase deliver component
///
/// Delivery phase (Left of Diamond) with validation and handoff.
#[component]
pub fn PhaseDeliver(
  mut state: Signal<PlannerState>,
  mut selected_entity: Signal<Option<SelectedEntity>>,
) -> Element {
  let mut active_tab = use_signal(|| DeliverTab::Validation);

  rsx! {
      div { class: "phase-deliver",
          SectionLabel {
              level: SectionLevel::Phase,
              label: "Delivery Phase".to_string(),
          }

          div { class: "deliver-tabs",
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DeliverTab::Validation { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DeliverTab::Validation); },
                  "Validation"
              }
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DeliverTab::Graph { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DeliverTab::Graph); },
                  "Dependencies"
              }
              button {
                  class: format!("tab-button {}", if *active_tab.read() == DeliverTab::Handoff { "active" } else { "" }),
                  onclick: move |_| { active_tab.set(DeliverTab::Handoff); },
                  "Handoff"
              }
          }

          div { class: "deliver-content",
              {match &*active_tab.read() {
                  DeliverTab::Validation => rsx! {
                      ValidationView {
                          state: state
                      }
                  },
                  DeliverTab::Graph => rsx! {
                      DependencyGraphView {
                          state: state
                      }
                  },
                  DeliverTab::Handoff => rsx! {
                      HandoffView {
                          state: state
                      }
                  },
              }}
          }
      }
  }
}

/// Delivery tabs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeliverTab {
  Validation,
  Graph,
  Handoff,
}

/// Validation view component
///
/// Shows validation checks and their status.
#[component]
fn ValidationView(state: Signal<PlannerState>) -> Element {
  let validation_checks = generate_validation_checks(&state.read());

  let critical_count = validation_checks
    .iter()
    .filter(|c| c.severity == ValidationSeverity::Critical && !c.passed)
    .count();
  let error_count = validation_checks
    .iter()
    .filter(|c| c.severity == ValidationSeverity::Error && !c.passed)
    .count();
  let warning_count = validation_checks
    .iter()
    .filter(|c| c.severity == ValidationSeverity::Warning && !c.passed)
    .count();
  let passed_count = validation_checks.iter().filter(|c| c.passed).count();

  let can_deliver = critical_count == 0 && error_count == 0;

  rsx! {
      div { class: "validation-view",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Validation Checks".to_string(),
          }

          div { class: "validation-summary",
              div { class: format!("summary-card {}", if critical_count > 0 { "has-critical" } else { "" }),
                  div { class: "summary-value", "{critical_count}" }
                  div { class: "summary-label", "Critical" }
              }

              div { class: format!("summary-card {}", if error_count > 0 { "has-error" } else { "" }),
                  div { class: "summary-value", "{error_count}" }
                  div { class: "summary-label", "Errors" }
              }

              div { class: format!("summary-card {}", if warning_count > 0 { "has-warning" } else { "" }),
                  div { class: "summary-value", "{warning_count}" }
                  div { class: "summary-label", "Warnings" }
              }

              div { class: "summary-card success",
                  div { class: "summary-value", "{passed_count}" }
                  div { class: "summary-label", "Passed" }
              }
          }

          {if can_deliver {
              rsx! {
                  div { class: "validation-status success",
                      h3 { "Ready for Delivery" }
                      p { "All critical and error checks have passed. You can proceed to handoff." }
                  }
              }
          } else {
              rsx! {
                  div { class: "validation-status blocked",
                      h3 { "Not Ready for Delivery" }
                      p { "Please address all critical and error issues before proceeding to handoff." }
                  }
              }
          }}

          div { class: "validation-items",
              {if validation_checks.is_empty() {
                  rsx! {
                      div { class: "empty-state",
                          p { "No validation checks available." }
                      }
                  }
              } else {
                  rsx! {
                      for check in validation_checks.iter() {
                          ValidationCheckItem {
                              key: "{check.id}",
                              check: check.clone()
                          }
                      }
                  }
              }}
          }
      }
  }
}

/// Generate validation checks from current state
fn generate_validation_checks(state: &PlannerState) -> Vec<ValidationCheck> {
  let mut checks = Vec::new();

  // Check if thesis exists
  match &state.thesis {
    Some(thesis) => {
      if thesis.title.is_empty() || thesis.problem.is_empty() {
        checks.push(
          ValidationCheck::new(
            "Thesis Completeness".to_string(),
            "Product thesis must have a title and problem statement".to_string(),
            ValidationSeverity::Error,
          )
          .with_message("Thesis is incomplete".to_string()),
        );
      } else {
        checks.push(
          ValidationCheck::new(
            "Thesis Completeness".to_string(),
            "Product thesis must have a title and problem statement".to_string(),
            ValidationSeverity::Info,
          )
          .with_passed(true),
        );
      }
    }
    None => {
      checks.push(
        ValidationCheck::new(
          "Thesis Exists".to_string(),
          "A product thesis is required".to_string(),
          ValidationSeverity::Critical,
        )
        .with_message("No thesis defined".to_string()),
      );
    }
  }

  // Check if personas exist
  if state.personas.is_empty() {
    checks.push(
      ValidationCheck::new(
        "Personas Defined".to_string(),
        "At least one persona should be defined".to_string(),
        ValidationSeverity::Warning,
      )
      .with_message("No personas defined".to_string()),
    );
  } else {
    checks.push(
      ValidationCheck::new(
        "Personas Defined".to_string(),
        "At least one persona should be defined".to_string(),
        ValidationSeverity::Info,
      )
      .with_passed(true),
    );
  }

  // Check if use cases exist
  if state.use_cases.is_empty() {
    checks.push(
      ValidationCheck::new(
        "Use Cases Defined".to_string(),
        "At least one use case should be defined".to_string(),
        ValidationSeverity::Warning,
      )
      .with_message("No use cases defined".to_string()),
    );
  } else {
    checks.push(
      ValidationCheck::new(
        "Use Cases Defined".to_string(),
        "At least one use case should be defined".to_string(),
        ValidationSeverity::Info,
      )
      .with_passed(true),
    );
  }

  // Check if tasks exist
  if state.tasks.is_empty() {
    checks.push(
      ValidationCheck::new(
        "Tasks Defined".to_string(),
        "At least one task should be defined".to_string(),
        ValidationSeverity::Error,
      )
      .with_message("No tasks defined".to_string()),
    );
  } else {
    checks.push(
      ValidationCheck::new(
        "Tasks Defined".to_string(),
        "Tasks have been defined".to_string(),
        ValidationSeverity::Info,
      )
      .with_passed(true),
    );
  }

  checks
}

/// Validation check item component
///
/// Individual validation check with status indicator.
#[component]
fn ValidationCheckItem(check: ValidationCheck) -> Element {
  let status_class = if check.passed {
    "status-passed"
  } else {
    match check.severity {
      ValidationSeverity::Critical => "status-critical",
      ValidationSeverity::Error => "status-error",
      ValidationSeverity::Warning => "status-warning",
      ValidationSeverity::Info => "status-info",
    }
  };

  let severity_class = format!("severity-{:?}", check.severity).to_lowercase();

  rsx! {
      div {
          class: format!("validation-check-item {} {}", status_class, severity_class),
          div { class: "check-indicator",
              {if check.passed {
                  rsx! { "✓" }
              } else {
                  rsx! { "!" }
              }}
          }

          div { class: "check-content",
              div { class: "check-header",
                  h4 { class: "check-name", "{check.name}" }
                  span { class: "check-severity", "{check.severity}" }
              }

              p { class: "check-description", "{check.description}" }

              {match &check.message {
                  Some(msg) if !check.passed => rsx! {
                      p { class: "check-message", "{msg}" }
                  },
                  _ => rsx! {}
              }}
          }
      }
  }
}

/// Dependency graph view component
///
/// Shows graph health metrics and dependency visualization.
#[component]
fn DependencyGraphView(state: Signal<PlannerState>) -> Element {
  let tasks: Vec<_> = state.read().tasks.iter().cloned().collect();
  let graph_health = calculate_graph_health(&tasks);

  rsx! {
      div { class: "dependency-graph-view",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Dependency Graph".to_string(),
          }

          div { class: "graph-health",
              div { class: "health-card",
                  h3 { "Graph Health Score" }
                  div {
                      class: format!("health-score {}", if graph_health.health_score >= 0.8 { "good" } else if graph_health.health_score >= 0.5 { "fair" } else { "poor" }),
                      "{format_health_score(graph_health.health_score)}"
                  }
              }

              div { class: "health-metrics",
                  div { class: "metric",
                      span { class: "metric-label", "Nodes" }
                      span { class: "metric-value", "{graph_health.node_count}" }
                  }

                  div { class: "metric",
                      span { class: "metric-label", "Edges" }
                      span { class: "metric-value", "{graph_health.edge_count}" }
                  }

                  div { class: "metric",
                      span { class: "metric-label", "Complexity" }
                      span { class: "metric-value", "{format_complexity(graph_health.complexity)}" }
                  }

                  {if graph_health.orphaned_nodes > 0 {
                      rsx! {
                          div { class: "metric warning",
                              span { class: "metric-label", "Orphaned" }
                              span { class: "metric-value", "{graph_health.orphaned_nodes}" }
                          }
                      }
                  } else {
                      rsx! {}
                  }}

                  {if graph_health.disconnected_components > 1 {
                      rsx! {
                          div { class: "metric warning",
                              span { class: "metric-label", "Components" }
                              span { class: "metric-value", "{graph_health.disconnected_components}" }
                          }
                      }
                  } else {
                      rsx! {}
                  }}
              }
          }

          div { class: "graph-visualization",
              div { class: "graph-placeholder",
                  p { "Graph visualization placeholder" }
                  p { class: "hint", "Dependency graph visualization will be rendered here" }
              }
          }
      }
  }
}

/// Calculate graph health from current state
fn calculate_graph_health(tasks: &[std::rc::Rc<crate::planner::types::PlanTask>]) -> GraphHealth {
  use crate::planner::validation::get_graph_health;
  let plan_tasks: Vec<crate::planner::types::PlanTask> =
    tasks.iter().map(|t| (**t).clone()).collect();
  get_graph_health(&plan_tasks)
}

/// Format health score as percentage
fn format_health_score(score: f32) -> String {
  format!("{:.0}%", score * 100.0)
}

/// Format complexity score
fn format_complexity(complexity: f32) -> String {
  format!("{:.1}", complexity)
}

/// Export format options
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
  Json,
  Markdown,
  Beads,
}

impl ExportFormat {
  /// Get all export formats
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[Self::Json, Self::Markdown, Self::Beads]
  }

  /// Get file extension for the format
  #[must_use]
  pub const fn extension(&self) -> &str {
    match self {
      Self::Json => "json",
      Self::Markdown => "md",
      Self::Beads => "beads",
    }
  }
}

impl fmt::Display for ExportFormat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Json => write!(f, "JSON"),
      Self::Markdown => write!(f, "Markdown"),
      Self::Beads => write!(f, "Beads"),
    }
  }
}

/// Export result type
pub type ExportResult = Result<String, String>;

/// Export the plan state to the specified format
///
/// # Errors
/// Returns error message if export fails
#[must_use]
pub fn export_plan(state: &PlannerState, format: ExportFormat) -> ExportResult {
  match format {
    ExportFormat::Json => export_to_json(state),
    ExportFormat::Markdown => export_to_markdown(state),
    ExportFormat::Beads => export_to_beads(state),
  }
}

/// Serializable representation of planner state for export
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SerializablePlannerState {
  pub current_phase: DiamondPhase,
  pub project_name: String,
  pub notes: String,
  pub tags: Vec<String>,
  pub thesis: Option<crate::planner::types::ProductThesis>,
  pub personas: Vec<crate::planner::types::Persona>,
  pub scenarios: Vec<crate::planner::types::NorthStarScenario>,
  pub use_cases: Vec<crate::planner::types::UseCase>,
  pub tasks: Vec<crate::planner::types::PlanTask>,
}

impl From<&PlannerState> for SerializablePlannerState {
  fn from(state: &PlannerState) -> Self {
    Self {
      current_phase: state.current_phase,
      project_name: state.context.project_name.clone(),
      notes: state.context.notes.clone(),
      tags: state.context.tags.iter().cloned().collect(),
      thesis: state.thesis.as_ref().map(|t| (**t).clone()),
      personas: state.personas.iter().map(|p| (**p).clone()).collect(),
      scenarios: state.scenarios.iter().map(|s| (**s).clone()).collect(),
      use_cases: state.use_cases.iter().map(|u| (**u).clone()).collect(),
      tasks: state.tasks.iter().map(|t| (**t).clone()).collect(),
    }
  }
}

/// Export plan to JSON format with sanitization
fn export_to_json(state: &PlannerState) -> ExportResult {
  // Convert to serializable format
  let serializable = SerializablePlannerState::from(state);

  // Validate export size before serialization
  let json_result = serde_json::to_string_pretty(&serializable)
    .map_err(|e| format!("JSON serialization failed: {e}"))?;

  // Security: Limit export size
  const MAX_EXPORT_SIZE: usize = 10_000_000; // 10MB limit
  if json_result.len() > MAX_EXPORT_SIZE {
    return Err(format!(
      "Export too large: {} bytes exceeds maximum of {} bytes",
      json_result.len(),
      MAX_EXPORT_SIZE
    ));
  }

  Ok(json_result)
}

/// Sanitize text for markdown export (escape dangerous characters)
fn sanitize_markdown_text(text: &str) -> String {
  const MAX_FIELD_LENGTH: usize = 10_000;

  // Truncate if too long
  let truncated = if text.len() > MAX_FIELD_LENGTH {
    &text[..MAX_FIELD_LENGTH]
  } else {
    text
  };

  // Remove control characters (except tab, newline, carriage return)
  let sanitized: String = truncated
    .chars()
    .filter(|c| {
      *c == '\t' || *c == '\n' || *c == '\r' || (*c as u32) >= 32 || *c == '\x0b' || *c == '\x0c'
    })
    .collect();

  // Escape markdown special characters to prevent injection
  sanitized
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('&', "&amp;")
}

/// Export plan to Markdown format with sanitization
fn export_to_markdown(state: &PlannerState) -> ExportResult {
  let mut output = String::new();

  // Title
  let project_name = if state.context.project_name.is_empty() {
    "Untitled Plan".to_string()
  } else {
    sanitize_markdown_text(&state.context.project_name)
  };

  output.push_str(&format!("# {}\n\n", project_name));

  // Phase indicator
  output.push_str(&format!("**Current Phase:** {}\n\n", state.current_phase));

  // Thesis section
  if let Some(thesis) = &state.thesis {
    output.push_str("## Product Thesis\n\n");
    output.push_str(&format!(
      "**Title:** {}\n\n",
      sanitize_markdown_text(&thesis.title)
    ));
    output.push_str(&format!(
      "**Problem:** {}\n\n",
      sanitize_markdown_text(&thesis.problem)
    ));
    output.push_str(&format!(
      "**Audience:** {}\n\n",
      sanitize_markdown_text(&thesis.audience)
    ));
    output.push_str(&format!(
      "**Solution:** {}\n\n",
      sanitize_markdown_text(&thesis.solution)
    ));
    output.push_str(&format!(
      "**Value Proposition:** {}\n\n",
      sanitize_markdown_text(&thesis.value_proposition)
    ));
  }

  // Personas section
  if !state.personas.is_empty() {
    output.push_str("## Personas\n\n");
    for persona in state.personas.iter() {
      output.push_str(&format!(
        "### {}\n\n",
        sanitize_markdown_text(&persona.name)
      ));
      output.push_str(&format!(
        "**Role:** {}\n\n",
        sanitize_markdown_text(&persona.role)
      ));
      output.push_str(&format!(
        "{}\n\n",
        sanitize_markdown_text(&persona.description)
      ));
    }
  }

  // Use cases section
  if !state.use_cases.is_empty() {
    output.push_str("## Use Cases\n\n");
    for use_case in state.use_cases.iter() {
      output.push_str(&format!(
        "### {}\n\n",
        sanitize_markdown_text(&use_case.title)
      ));
      output.push_str(&format!(
        "{}\n\n",
        sanitize_markdown_text(&use_case.description)
      ));
    }
  }

  // Tasks section
  if !state.tasks.is_empty() {
    output.push_str("## Tasks\n\n");
    for task in state.tasks.iter() {
      let status = if task.completion >= 1.0 { "✓" } else { " " };
      output.push_str(&format!(
        "- [{status}] **{}** ({})\n",
        sanitize_markdown_text(&task.title),
        task.task_type
      ));
    }
    output.push('\n');
  }

  // Validate final output size
  const MAX_MARKDOWN_SIZE: usize = 10_000_000; // 10MB
  if output.len() > MAX_MARKDOWN_SIZE {
    return Err(format!(
      "Markdown export too large: {} bytes exceeds maximum of {} bytes",
      output.len(),
      MAX_MARKDOWN_SIZE
    ));
  }

  Ok(output)
}

/// Export plan to Beads format with sanitization
fn export_to_beads(state: &PlannerState) -> ExportResult {
  let mut output = String::new();

  output.push_str("# Beads Project Plan\n\n");

  // Metadata
  output.push_str("## Metadata\n\n");
  output.push_str(&format!(
    "project: {}\n",
    sanitize_markdown_text(&state.context.project_name)
  ));
  output.push_str(&format!("phase: {}\n", state.current_phase));
  output.push_str(&format!("created: {}\n\n", chrono::Utc::now().to_rfc3339()));

  // Thesis as bead
  if let Some(thesis) = &state.thesis {
    output.push_str("## Thesis\n\n");
    output.push_str(&format!(
      "title: {}\n",
      sanitize_markdown_text(&thesis.title)
    ));
    output.push_str(&format!(
      "problem: {}\n",
      sanitize_markdown_text(&thesis.problem)
    ));
    output.push_str(&format!(
      "audience: {}\n",
      sanitize_markdown_text(&thesis.audience)
    ));
    output.push_str(&format!(
      "solution: {}\n\n",
      sanitize_markdown_text(&thesis.solution)
    ));
  }

  // Personas as beads
  for persona in state.personas.iter() {
    output.push_str(&format!(
      "## Persona: {}\n",
      sanitize_markdown_text(&persona.name)
    ));
    output.push_str(&format!(
      "role: {}\n",
      sanitize_markdown_text(&persona.role)
    ));
    output.push_str(&format!(
      "description: {}\n\n",
      sanitize_markdown_text(&persona.description)
    ));
  }

  // Use cases as beads
  for use_case in state.use_cases.iter() {
    output.push_str(&format!(
      "## UseCase: {}\n",
      sanitize_markdown_text(&use_case.title)
    ));
    output.push_str(&format!("priority: {}\n", use_case.priority));
    output.push_str(&format!(
      "trigger: {}\n\n",
      sanitize_markdown_text(&use_case.trigger)
    ));
  }

  // Tasks as beads
  for task in state.tasks.iter() {
    output.push_str(&format!(
      "## Task: {}\n",
      sanitize_markdown_text(&task.title)
    ));
    output.push_str(&format!("type: {}\n", task.task_type));
    output.push_str(&format!("phase: {}\n", task.phase));
    output.push_str(&format!("completion: {}\n\n", task.completion));
  }

  // Validate final output size
  const MAX_BEADS_SIZE: usize = 10_000_000; // 10MB
  if output.len() > MAX_BEADS_SIZE {
    return Err(format!(
      "Beads export too large: {} bytes exceeds maximum of {} bytes",
      output.len(),
      MAX_BEADS_SIZE
    ));
  }

  Ok(output)
}

/// Copy text to clipboard with sanitization and size limits
///
/// # Errors
/// Returns error message if clipboard operation fails or input is invalid
///
/// Note: This is a synchronous wrapper for the async clipboard API.
/// In the actual UI context, use the platform-specific clipboard API directly.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
  // Security: Limit clipboard size to prevent DoS
  const MAX_CLIPBOARD_SIZE: usize = 1_000_000; // 1MB limit

  // Validate input size
  let text_len = text.len();
  if text_len > MAX_CLIPBOARD_SIZE {
    return Err(format!(
      "Clipboard content too large: {} bytes exceeds maximum of {} bytes",
      text_len, MAX_CLIPBOARD_SIZE
    ));
  }

  // Sanitize: Remove null bytes and control characters (except newline, tab, carriage return)
  let _sanitized: String = text
    .chars()
    .filter(|c| {
      *c == '\n' || *c == '\t' || *c == '\r' || (*c as u32) >= 32 || *c == '\x0b' || *c == '\x0c'
    })
    .collect();

  // In a real desktop context, this would use the platform clipboard API
  // For now, return an error to indicate this needs UI context
  Err(
    "Clipboard copy must be called from UI context. Use platform-specific clipboard API in the component."
      .to_string(),
  )
}

/// Handoff view component
///
/// Shows handoff summary, export options, and command preview.
#[component]
fn HandoffView(state: Signal<PlannerState>) -> Element {
  let thesis = state.read().thesis.clone();
  let task_count = state.read().tasks.len();
  let use_case_count = state.read().use_cases.len();
  let persona_count = state.read().personas.len();

  let mut selected_format = use_signal(|| ExportFormat::Json);
  let mut export_status = use_signal(|| ExportStatus::Idle);
  let mut exported_content = use_signal(|| None::<String>);

  let get_export_command = move || {
    let project_name = state.read().context.project_name.clone();
    let format = selected_format.read();
    if project_name.is_empty() {
      format!("clarity plan export --format {}", format.extension())
    } else {
      format!(
        "clarity plan export --project \"{}\" --format {}",
        project_name,
        format.extension()
      )
    }
  };

  let handle_export = move |_| {
    export_status.set(ExportStatus::Exporting);

    let current_state = state.read();
    let format = *selected_format.read();

    let result = export_plan(&current_state, format);

    match result {
      Ok(content) => {
        // Validate content before setting Success state
        match ExportStatus::validate_success(content) {
          Ok(success_status) => {
            if let ExportStatus::Success { content, .. } = success_status {
              exported_content.set(Some(content.clone()));
              export_status.set(ExportStatus::Success {
                content,
                format: format.to_string(),
              });
            }
          }
          Err(error) => {
            export_status.set(ExportStatus::validate_error(error));
          }
        }
      }
      Err(error) => {
        export_status.set(ExportStatus::validate_error(error));
      }
    }
  };

  let copy_to_clipboard_action = move |_| {
    // Use state machine validation - can only copy from Success state
    if !export_status.read().can_copy() {
      return;
    }

    if let Some(content) = &*exported_content.read() {
      match copy_to_clipboard(content) {
        Ok(()) => {
          export_status.set(ExportStatus::Copied);
        }
        Err(error) => {
          export_status.set(ExportStatus::validate_error(error));
        }
      }
    }
  };

  let mut reset_export_status = move |_| {
    export_status.set(ExportStatus::Idle);
  };

  let can_export = thesis.is_some() || persona_count > 0 || use_case_count > 0 || task_count > 0;

  rsx! {
      div { class: "handoff-view",
          SectionLabel {
              level: SectionLevel::Section,
              label: "Project Handoff".to_string(),
          }

          div { class: "handoff-summary",
              h3 { "Project Summary" }

              {match thesis {
                  Some(t) => rsx! {
                      div { class: "summary-section",
                          h4 { "Product Thesis" }
                          div { class: "thesis-summary",
                              p { strong { "Title: " } "{t.title}" }
                              p { strong { "Problem: " } "{t.problem}" }
                              p { strong { "Audience: " } "{t.audience}" }
                          }
                      }
                  },
                  None => rsx! {
                      div { class: "summary-section empty",
                          p { "No product thesis defined" }
                      }
                  }
              }}

              div { class: "summary-section",
                  h4 { "Project Statistics" }
                  div { class: "stats-grid",
                      div { class: "stat-item",
                          span { class: "stat-label", "Personas:" }
                          span { class: "stat-value", "{persona_count}" }
                      }

                      div { class: "stat-item",
                          span { class: "stat-label", "Use Cases:" }
                          span { class: "stat-value", "{use_case_count}" }
                      }

                      div { class: "stat-item",
                          span { class: "stat-label", "Tasks:" }
                          span { class: "stat-value", "{task_count}" }
                      }
                  }
              }
          }

          // Export section
          div { class: "handoff-export",
              h4 { "Export Plan" }

              div { class: "export-options",
                  label { class: "export-format-label", "Format:" }
                  for format in ExportFormat::all() {
                      label { class: "export-format-option",
                          input {
                              r#type: "radio",
                              name: "export-format",
                              value: "{format:?}",
                              checked: *selected_format.read() == *format,
                              oninput: move |_| {
                                  selected_format.set(*format);
                                  reset_export_status(());
                              }
                          }
                          span { "{format}" }
                      }
                  }
              }

              div { class: "export-actions",
                  button {
                      class: format!("btn btn-primary {}",
                          if matches!(&*export_status.read(), ExportStatus::Exporting) {
                              "btn-loading"
                          } else {
                              ""
                          }
                      ),
                      onclick: handle_export,
                      disabled: !can_export || matches!(&*export_status.read(), ExportStatus::Exporting),
                      {match &*export_status.read() {
                          ExportStatus::Idle | ExportStatus::Copied => rsx! {
                              "Export Plan"
                          },
                          ExportStatus::Exporting => rsx! {
                              span { class: "spinner", "⏳" }
                              span { "Exporting..." }
                          },
                          ExportStatus::Success { .. } => rsx! {
                              span { class: "success-icon", "✓" }
                              span { "Exported!" }
                          },
                          ExportStatus::Error(_) => rsx! {
                              span { class: "error-icon", "✕" }
                              span { "Failed" }
                          },
                          ExportStatus::Cancelled => rsx! {
                              "Export Plan"
                          },
                      }}
                  }

                  {matches!(&*export_status.read(), ExportStatus::Success { .. }).then(||
                      rsx! {
                          button {
                              class: "btn btn-secondary",
                              onclick: copy_to_clipboard_action,
                              title: "Copy to clipboard",
                              "📋 Copy"
                          }
                      }
                  )}
              }

              // Export status notifications
              {match &*export_status.read() {
                  ExportStatus::Idle => rsx! {},
                  ExportStatus::Exporting => rsx! {
                      div { class: "export-status exporting",
                          p { "Exporting plan..." }
                      }
                  },
                  ExportStatus::Success { content, format } => rsx! {
                      div { class: "export-status success",
                          p { strong { "Export successful! " }
                              "Generated {format} ({content.len()} bytes)"
                          }
                          button {
                              class: "btn-close",
                              onclick: move |_| reset_export_status(()),
                              "×"
                          }
                      }
                  },
                  ExportStatus::Error(error) => rsx! {
                      div { class: "export-status error",
                          p { strong { "Export Failed: " } "{error}" }
                          button {
                              class: "btn-close",
                              onclick: move |_| reset_export_status(()),
                              "×"
                          }
                      }
                  },
                  ExportStatus::Copied => rsx! {
                      div { class: "export-status info",
                          p { "Copied to clipboard!" }
                          button {
                              class: "btn-close",
                              onclick: move |_| reset_export_status(()),
                              "×"
                          }
                      }
                  },
                  ExportStatus::Cancelled => rsx! {},
              }}
          }

          div { class: "handoff-command",
              h4 { "CLI Export Command" }
              div { class: "command-preview",
                  code {
                      class: "command",
                      "{get_export_command()}"
                  }
              }

              p { class: "command-hint",
                  "Run this command in your terminal to export the plan from the CLI"
              }
          }

          div { class: "handoff-notes",
              h4 { "Handoff Notes" }
              p { "This project plan can be exported and shared with your team." }
              p { "The export includes all phases: discovery, design, development, and delivery." }
              ul {
                  li { strong { "JSON: " } "Machine-readable format for programmatic access" }
                  li { strong { "Markdown: " } "Human-readable format for documentation" }
                  li { strong { "Beads: " } "Native format for Beads project management" }
              }
          }
      }
  }
}

/// Export status for UI feedback with validated state transitions
#[derive(Clone, Debug, PartialEq)]
enum ExportStatus {
  Idle,
  Exporting,
  Success { content: String, format: String },
  Error(String),
  Copied,
  Cancelled,
}

impl ExportStatus {
  /// Validate that content is non-empty before allowing Success state
  fn validate_success(content: String) -> Result<Self, String> {
    let sanitized_content = content.trim();
    if sanitized_content.is_empty() {
      return Err("Export content cannot be empty".to_string());
    }

    // Security: Validate content size
    const MAX_EXPORT_CONTENT_SIZE: usize = 10_000_000; // 10MB
    if sanitized_content.len() > MAX_EXPORT_CONTENT_SIZE {
      return Err(format!(
        "Export content exceeds maximum size of {} bytes",
        MAX_EXPORT_CONTENT_SIZE
      ));
    }

    Ok(Self::Success {
      content: sanitized_content.to_string(),
      format: String::new(), // Will be set by caller
    })
  }

  /// Validate error message is meaningful
  fn validate_error(error: String) -> Self {
    let sanitized_error = error.trim();
    let message = if sanitized_error.is_empty() {
      "An unknown error occurred during export".to_string()
    } else if sanitized_error.len() > 500 {
      // Truncate very long error messages
      sanitized_error.chars().take(500).collect::<String>() + "..."
    } else {
      sanitized_error.to_string()
    };

    Self::Error(message)
  }

  /// Check if Copied state can be reached from current state
  fn can_copy(&self) -> bool {
    matches!(self, Self::Success { .. })
  }

  /// Check if export can be cancelled from current state
  fn can_cancel(&self) -> bool {
    matches!(self, Self::Exporting)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::planner::types::{Persona, ProductThesis};

  #[test]
  fn test_deliver_tab_equality() {
    assert_eq!(DeliverTab::Validation, DeliverTab::Validation);
    assert_ne!(DeliverTab::Validation, DeliverTab::Graph);
    assert_ne!(DeliverTab::Graph, DeliverTab::Handoff);
  }

  #[test]
  fn test_generate_validation_checks() {
    let state = PlannerState::new();
    let checks = generate_validation_checks(&state);

    // Should have checks for missing thesis, personas, use cases, and tasks
    assert!(checks.iter().any(|c| c.name == "Thesis Exists"));
    assert!(checks.iter().any(|c| c.name == "Personas Defined"));
  }

  #[test]
  fn test_calculate_graph_health() {
    let state = PlannerState::new();
    let tasks: Vec<std::rc::Rc<crate::planner::types::PlanTask>> =
      state.tasks.iter().cloned().collect();
    let health = calculate_graph_health(&tasks);

    assert_eq!(health.node_count, 0);
    assert_eq!(health.edge_count, 0);
  }

  #[test]
  fn test_format_health_score() {
    assert_eq!(format_health_score(0.5), "50%");
    assert_eq!(format_health_score(1.0), "100%");
    assert_eq!(format_health_score(0.75), "75%");
  }

  #[test]
  fn test_format_complexity() {
    assert_eq!(format_complexity(1.5), "1.5");
    assert_eq!(format_complexity(0.0), "0.0");
  }

  // Export format tests
  #[test]
  fn test_export_format_extensions() {
    assert_eq!(ExportFormat::Json.extension(), "json");
    assert_eq!(ExportFormat::Markdown.extension(), "md");
    assert_eq!(ExportFormat::Beads.extension(), "beads");
  }

  #[test]
  fn test_export_format_display() {
    assert_eq!(format!("{}", ExportFormat::Json), "JSON");
    assert_eq!(format!("{}", ExportFormat::Markdown), "Markdown");
    assert_eq!(format!("{}", ExportFormat::Beads), "Beads");
  }

  #[test]
  fn test_export_format_all() {
    let all = ExportFormat::all();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&ExportFormat::Json));
    assert!(all.contains(&ExportFormat::Markdown));
    assert!(all.contains(&ExportFormat::Beads));
  }

  #[test]
  fn test_export_to_json() {
    let state = PlannerState::new();
    let result = export_to_json(&state);
    assert!(result.is_ok());

    let json_str = result.unwrap();
    // Should be valid JSON
    assert!(json_str.starts_with('{'));
    assert!(json_str.contains("current_phase"));
  }

  #[test]
  fn test_export_to_markdown() {
    let mut state = PlannerState::new();
    state = state.update_project_name("Test Project".to_string());

    let thesis = ProductThesis::new(
      "Test".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let result = export_to_markdown(&state);
    assert!(result.is_ok());

    let md_str = result.unwrap();
    assert!(md_str.contains("# Test Project"));
    assert!(md_str.contains("## Product Thesis"));
    assert!(md_str.contains("**Title:** Test"));
  }

  #[test]
  fn test_export_to_beads() {
    let mut state = PlannerState::new();
    state = state.update_project_name("Test Project".to_string());

    let result = export_to_beads(&state);
    assert!(result.is_ok());

    let beads_str = result.unwrap();
    assert!(beads_str.contains("# Beads Project Plan"));
    assert!(beads_str.contains("project: Test Project"));
  }

  #[test]
  fn test_export_plan_json() {
    let state = PlannerState::new();
    let result = export_plan(&state, ExportFormat::Json);
    assert!(result.is_ok());
    assert!(result.unwrap().starts_with('{'));
  }

  #[test]
  fn test_export_plan_markdown() {
    let state = PlannerState::new();
    let result = export_plan(&state, ExportFormat::Markdown);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("# Untitled Plan"));
  }

  #[test]
  fn test_export_plan_beads() {
    let state = PlannerState::new();
    let result = export_plan(&state, ExportFormat::Beads);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("# Beads Project Plan"));
  }

  #[test]
  fn test_export_markdown_with_personas() {
    let mut state = PlannerState::new();
    let persona = Persona::new(
      "Test User".to_string(),
      "Developer".to_string(),
      "A test persona".to_string(),
    );
    state = state.add_persona(persona).unwrap();

    let result = export_to_markdown(&state);
    assert!(result.is_ok());
    let md = result.unwrap();
    assert!(md.contains("## Personas"));
    assert!(md.contains("### Test User"));
    assert!(md.contains("**Role:** Developer"));
  }

  #[test]
  fn test_export_status_equality() {
    assert_eq!(ExportStatus::Idle, ExportStatus::Idle);
    assert_ne!(ExportStatus::Idle, ExportStatus::Exporting);
    assert_ne!(
      ExportStatus::Success {
        content: "test".to_string(),
        format: "JSON".to_string()
      },
      ExportStatus::Error("error".to_string())
    );
  }

  #[test]
  fn test_export_status_validate_success() {
    // Valid content should succeed
    let result = ExportStatus::validate_success("valid content".to_string());
    assert!(result.is_ok());

    // Empty content should fail
    let result = ExportStatus::validate_success("   ".to_string());
    assert!(result.is_err());

    // Content that's too large should fail
    let large_content = "a".repeat(10_000_001);
    let result = ExportStatus::validate_success(large_content);
    assert!(result.is_err());
  }

  #[test]
  fn test_export_status_can_copy() {
    assert!(!ExportStatus::Idle.can_copy());
    assert!(!ExportStatus::Exporting.can_copy());
    assert!(ExportStatus::Success {
      content: "test".to_string(),
      format: "JSON".to_string()
    }
    .can_copy());
    assert!(!ExportStatus::Error("error".to_string()).can_copy());
    assert!(!ExportStatus::Copied.can_copy());
  }

  #[test]
  fn test_clipboard_size_limit() {
    // Content exceeding limit should always fail
    let large_content = "a".repeat(1_000_001);
    let result = copy_to_clipboard(&large_content);
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("too large") || error_msg.contains("desktop context"));
  }
}
