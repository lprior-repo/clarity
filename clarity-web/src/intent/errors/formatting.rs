#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::{ContextualError, IntentError};
use crate::intent::plan::PlanError;
use std::fmt::Write as _;

/// Formats a `PlanError` for display with helpful messages and context.
#[must_use]
pub fn format_plan_error(error: &PlanError) -> String {
  let mut output = String::new();

  match error {
    PlanError::DependencyError(msg) => {
      let _ = writeln!(output, "Error: Dependency Error");
      let _ = writeln!(output, "  Message: {msg}");
      let _ = writeln!(
        output,
        "  Suggestion: Review bead dependencies to resolve the issue"
      );
    }
    PlanError::InvalidPhase(phase_number) => {
      let _ = writeln!(output, "Error: Invalid Phase");
      let _ = writeln!(output, "  Message: Invalid phase number: {phase_number}");
      let _ = writeln!(output, "  Context:");
      let _ = writeln!(output, "    phase_number: {phase_number}");
      let _ = writeln!(
        output,
        "  Suggestion: Use a valid phase number (typically starting from 1)"
      );
    }
    PlanError::PhaseNotComplete(phase_number) => {
      let _ = writeln!(output, "Error: Phase Not Complete");
      let _ = writeln!(
        output,
        "  Message: Phase {phase_number} is not yet complete"
      );
      let _ = writeln!(output, "  Context:");
      let _ = writeln!(output, "    phase_number: {phase_number}");
      let _ = writeln!(
        output,
        "  Suggestion: Complete all beads in the current phase before proceeding"
      );
    }
    PlanError::NoActionableBeads => {
      let _ = writeln!(output, "Error: No Actionable Beads");
      let _ = writeln!(output, "  Message: No beads available to execute");
      let _ = writeln!(
        output,
        "  Suggestion: Add beads to the plan or resolve any blockers"
      );
    }
    PlanError::EmptySessionId => {
      let _ = writeln!(output, "Error: Empty Session ID");
      let _ = writeln!(output, "  Message: Session ID cannot be empty");
      let _ = writeln!(output, "  Suggestion: Provide a valid session identifier");
    }
    PlanError::CircularDependency(from, to) => {
      let _ = writeln!(output, "Error: Circular Dependency");
      let _ = writeln!(
        output,
        "  Message: Circular dependency detected: {from} -> {to}"
      );
      let _ = writeln!(output, "  Context:");
      let _ = writeln!(output, "    from: {from}");
      let _ = writeln!(output, "    to: {to}");
      let _ = writeln!(
        output,
        "  Suggestion: Review bead dependencies to remove cycles"
      );
    }
    PlanError::InvalidPhaseTransition { from, to } => {
      let _ = writeln!(output, "Error: Invalid Phase Transition");
      let _ = writeln!(
        output,
        "  Message: Invalid phase status transition: {from:?} -> {to:?}"
      );
      let _ = writeln!(output, "  Context:");
      let _ = writeln!(output, "    from_state: {from:?}");
      let _ = writeln!(output, "    to_state: {to:?}");
      let _ = writeln!(output, "  Suggestion: Check valid phase status transitions");
    }
    PlanError::InvalidBeadTransition { from, to } => {
      let _ = writeln!(output, "Error: Invalid Bead Transition");
      let _ = writeln!(
        output,
        "  Message: Invalid bead status transition: {from:?} -> {to:?}"
      );
      let _ = writeln!(output, "  Context:");
      let _ = writeln!(output, "    from_state: {from:?}");
      let _ = writeln!(output, "    to_state: {to:?}");
      let _ = writeln!(output, "  Suggestion: Check valid bead status transitions");
    }
  }

  output.trim_end().to_string()
}

#[must_use]
pub fn format_error(error: &ContextualError) -> String {
  let mut output = String::new();

  let error_type = match &error.error {
    IntentError::JsonParse { .. } => "JSON Parse Error",
    IntentError::MissingField { .. } => "Missing Field",
    IntentError::InvalidType { .. } => "Type Error",
    IntentError::InvalidValue { .. } => "Value Error",
    IntentError::UnknownField { .. } => "Unknown Field",
    IntentError::ValidationFailed { .. } => "Validation Error",
    IntentError::Io { .. } => "IO Error",
    IntentError::FileNotFound { .. } => "File Not Found",
    IntentError::InvalidPath { .. } => "Invalid Path",
    IntentError::CircularDependency { .. } => "Circular Dependency",
    IntentError::ConstraintViolation { .. } => "Constraint Violation",
    IntentError::Configuration { .. } => "Configuration Error",
    IntentError::Internal { .. } => "Internal Error",
  };

  let _ = writeln!(output, "Error: {error_type}");
  let _ = writeln!(output, "  Message: {}", error.message);

  if let Some(location) = error.location_string() {
    let _ = writeln!(output, "  Location: {location}");
  }
  if let Some(json_path) = &error.json_path {
    let _ = writeln!(output, "  JSON Path: {json_path}");
  }

  if !error.suggestions.is_empty() {
    output.push_str("  Suggestions:\n");
    for suggestion in &error.suggestions {
      let _ = writeln!(
        output,
        "    - {} (edit distance: {})",
        suggestion.text, suggestion.distance
      );
    }
  }

  if !error.context.is_empty() {
    output.push_str("  Context:\n");
    for (key, value) in &error.context {
      let _ = writeln!(output, "    {key}: {value}");
    }
  }

  output.trim_end().to_string()
}
