#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

mod logic;

use crate::intent::interview::types::{Gap, InterviewSession};
use crate::intent::plan::types::{ExecutionPlan, PlanBead};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ActionType {
  #[default]
  AnswerQuestion,
  ResolveGap,
  ResolveConflict,
  CompletePhase,
  ReviewPlan,
}

impl ActionType {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::AnswerQuestion => "answer_question",
      Self::ResolveGap => "resolve_gap",
      Self::ResolveConflict => "resolve_conflict",
      Self::CompletePhase => "complete_phase",
      Self::ReviewPlan => "review_plan",
    }
  }

  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::AnswerQuestion => "Answer pending interview question",
      Self::ResolveGap => "Resolve missing requirement",
      Self::ResolveConflict => "Resolve conflicting requirements",
      Self::CompletePhase => "Complete the current phase",
      Self::ReviewPlan => "Review and approve execution plan",
    }
  }
}

/// Action enum with data-carrying variants for specific operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Action {
  /// Resolve a gap with a specific resolution.
  ResolveGap {
    /// The ID of the gap to resolve.
    gap_id: String,
    /// The resolution text for the gap.
    resolution: String,
  },
  /// Answer a pending interview question.
  AnswerQuestion {
    /// The ID of the question to answer.
    question_id: String,
    /// The answer text.
    answer: String,
  },
  /// Resolve a conflict between requirements.
  ResolveConflict {
    /// The ID of the conflict to resolve.
    conflict_id: String,
    /// The resolution text for the conflict.
    resolution: String,
  },
  /// Complete a phase of the execution plan.
  CompletePhase {
    /// The phase number to complete.
    phase: u32,
  },
  /// Review and approve the execution plan.
  ReviewPlan {
    /// Whether the plan is approved.
    approved: bool,
    /// Optional feedback on the plan.
    feedback: Option<String>,
  },
}

impl Action {
  /// Create a new `ResolveGap` action.
  #[must_use]
  pub const fn resolve_gap(gap_id: String, resolution: String) -> Self {
    Self::ResolveGap { gap_id, resolution }
  }

  /// Create a new `AnswerQuestion` action.
  #[must_use]
  pub const fn answer_question(question_id: String, answer: String) -> Self {
    Self::AnswerQuestion {
      question_id,
      answer,
    }
  }

  /// Create a new `ResolveConflict` action.
  #[must_use]
  pub const fn resolve_conflict(conflict_id: String, resolution: String) -> Self {
    Self::ResolveConflict {
      conflict_id,
      resolution,
    }
  }

  /// Create a new `CompletePhase` action.
  #[must_use]
  pub const fn complete_phase(phase: u32) -> Self {
    Self::CompletePhase { phase }
  }

  /// Create a new `ReviewPlan` action.
  #[must_use]
  pub const fn review_plan(approved: bool, feedback: Option<String>) -> Self {
    Self::ReviewPlan { approved, feedback }
  }

  /// Get the action type for this action.
  #[must_use]
  pub const fn action_type(&self) -> ActionType {
    match self {
      Self::ResolveGap { .. } => ActionType::ResolveGap,
      Self::AnswerQuestion { .. } => ActionType::AnswerQuestion,
      Self::ResolveConflict { .. } => ActionType::ResolveConflict,
      Self::CompletePhase { .. } => ActionType::CompletePhase,
      Self::ReviewPlan { .. } => ActionType::ReviewPlan,
    }
  }

  /// Get a human-readable description of the action.
  #[must_use]
  pub fn description(&self) -> String {
    match self {
      Self::ResolveGap { gap_id, resolution } => {
        format!("Resolve gap '{gap_id}' with: {resolution}")
      }
      Self::AnswerQuestion {
        question_id,
        answer,
      } => format!("Answer question '{question_id}' with: {answer}"),
      Self::ResolveConflict {
        conflict_id,
        resolution,
      } => format!("Resolve conflict '{conflict_id}' with: {resolution}"),
      Self::CompletePhase { phase } => format!("Complete phase {phase}"),
      Self::ReviewPlan { approved, feedback } => {
        let status = if *approved { "approve" } else { "reject" };
        match feedback {
          Some(fb) => format!("Review plan: {status} ({fb})"),
          None => format!("Review plan: {status}"),
        }
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NextAction {
  pub action_type: ActionType,
  pub target_id: String,
  pub description: String,
  pub reason: String,
  #[serde(default)]
  pub priority: u32,
}

impl NextAction {
  #[must_use]
  pub const fn new(
    action_type: ActionType,
    target_id: String,
    description: String,
    reason: String,
  ) -> Self {
    Self {
      action_type,
      target_id,
      description,
      reason,
      priority: 0,
    }
  }

  #[must_use]
  pub fn with_priority(self, priority: u32) -> Self {
    Self { priority, ..self }
  }
}

/// Context information for the next action recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionContext {
  /// Current interview stage
  pub stage: String,
  /// Current phase number
  pub current_phase: u32,
  /// Number of completed phases
  pub completed_phases: Vec<u32>,
  /// Number of open gaps
  pub open_gaps: usize,
  /// Number of pending conflicts
  pub pending_conflicts: usize,
  /// Whether the session can proceed
  pub can_proceed: bool,
  /// Additional context metadata
  #[serde(default)]
  pub metadata: HashMap<String, String>,
}

impl ActionContext {
  /// Create a new `ActionContext` from an `InterviewSession`.
  #[must_use]
  pub fn from_session(session: &InterviewSession) -> Self {
    Self {
      stage: format!("{:?}", session.stage),
      current_phase: session.current_phase,
      completed_phases: session.completed_phases.clone(),
      open_gaps: session.gaps.iter().filter(|g| !g.is_resolved()).count(),
      pending_conflicts: session
        .conflicts
        .iter()
        .filter(|c| !c.is_resolved())
        .count(),
      can_proceed: session.can_proceed().is_ok(),
      metadata: HashMap::new(),
    }
  }

  /// Add metadata to the context.
  #[must_use]
  pub fn with_metadata(self, key: String, value: String) -> Self {
    let mut metadata = self.metadata;
    metadata.insert(key, value);
    Self { metadata, ..self }
  }
}

/// A suggestion for alternative actions the user could take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionSuggestion {
  /// Suggested action type
  pub action_type: ActionType,
  /// Brief description of the suggestion
  pub description: String,
  /// Why this might be relevant
  pub rationale: String,
}

impl ActionSuggestion {
  /// Create a new `ActionSuggestion`.
  #[must_use]
  pub const fn new(action_type: ActionType, description: String, rationale: String) -> Self {
    Self {
      action_type,
      description,
      rationale,
    }
  }
}

/// Complete JSON output for `plan_next_command` functionality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanNextJsonOutput {
  /// The recommended next action, if any
  pub next_action: Option<NextAction>,
  /// Context about the current state
  pub context: ActionContext,
  /// Alternative suggestions
  pub suggestions: Vec<ActionSuggestion>,
  /// Actionable beads available
  pub actionable_beads: Vec<String>,
  /// Next phase to execute, if applicable
  pub next_phase: Option<u32>,
}

impl PlanNextJsonOutput {
  /// Create a `PlanNextJsonOutput` from session and plan.
  #[must_use]
  pub fn from_session_and_plan(session: &InterviewSession, plan: &ExecutionPlan) -> Self {
    let next_action = get_next_action(session, plan);
    let context = ActionContext::from_session(session);
    let suggestions = generate_suggestions(session, plan);
    let actionable_beads = get_actionable_beads(plan)
      .into_iter()
      .map(|bead| bead.id.clone())
      .collect();
    let next_phase = determine_next_phase(plan);

    Self {
      next_action,
      context,
      suggestions,
      actionable_beads,
      next_phase,
    }
  }

  /// Serialize to JSON string.
  ///
  /// # Errors
  /// Returns an error if serialization fails.
  pub fn to_json(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(self)
  }

  /// Serialize to compact JSON string.
  ///
  /// # Errors
  /// Returns an error if serialization fails.
  pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string(self)
  }
}

/// Generate suggestions for alternative actions.
fn generate_suggestions(session: &InterviewSession, plan: &ExecutionPlan) -> Vec<ActionSuggestion> {
  let mut suggestions = Vec::new();

  // Suggest reviewing plan if in validation stage
  if session.stage == crate::intent::interview::types::InterviewStage::Validation {
    suggestions.push(ActionSuggestion::new(
      ActionType::ReviewPlan,
      "Review the execution plan".to_string(),
      "Session is in validation stage - plan review recommended".to_string(),
    ));
  }

  // Suggest resolving gaps if any exist
  let open_gaps: Vec<_> = session.gaps.iter().filter(|g| !g.is_resolved()).collect();
  if !open_gaps.is_empty() {
    suggestions.push(ActionSuggestion::new(
      ActionType::ResolveGap,
      format!("Resolve {} open gap(s)", open_gaps.len()),
      "Unresolved gaps may block progress".to_string(),
    ));
  }

  // Suggest resolving conflicts if any exist
  let pending_conflicts: Vec<_> = session
    .conflicts
    .iter()
    .filter(|c| !c.is_resolved())
    .collect();
  if !pending_conflicts.is_empty() {
    suggestions.push(ActionSuggestion::new(
      ActionType::ResolveConflict,
      format!("Resolve {} pending conflict(s)", pending_conflicts.len()),
      "Conflicts need resolution before proceeding".to_string(),
    ));
  }

  // Suggest phase completion if actionable beads exist
  let actionable = plan.get_actionable_beads();
  if !actionable.is_empty() {
    suggestions.push(ActionSuggestion::new(
      ActionType::CompletePhase,
      format!("Work on {} actionable item(s)", actionable.len()),
      "Items are ready for execution".to_string(),
    ));
  }

  suggestions
}

/// Format the plan next output as a JSON string.
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn format_next_action_json(
  session: &InterviewSession,
  plan: &ExecutionPlan,
) -> Result<String, serde_json::Error> {
  let output = PlanNextJsonOutput::from_session_and_plan(session, plan);
  output.to_json()
}

/// Format the plan next output as a compact JSON string.
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn format_next_action_json_compact(
  session: &InterviewSession,
  plan: &ExecutionPlan,
) -> Result<String, serde_json::Error> {
  let output = PlanNextJsonOutput::from_session_and_plan(session, plan);
  output.to_json_compact()
}

#[must_use]
pub fn get_next_action(session: &InterviewSession, plan: &ExecutionPlan) -> Option<NextAction> {
  logic::get_next_action(session, plan)
}

#[must_use]
pub fn determine_next_phase(plan: &ExecutionPlan) -> Option<u32> {
  logic::determine_next_phase(plan)
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  logic::get_actionable_beads(plan)
}

#[must_use]
pub fn get_blocking_gaps(session: &InterviewSession) -> Vec<&Gap> {
  session.get_blocking_gaps()
}

#[must_use]
pub fn can_proceed(session: &InterviewSession) -> bool {
  session.can_proceed().is_ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::interview::types::{Conflict, ConflictResolution, GapState, InterviewStage};
  use crate::intent::plan::types::PlanBead;

  fn create_test_session() -> InterviewSession {
    InterviewSession {
      id: "test-session".to_string(),
      stage: InterviewStage::Discovery,
      current_phase: 1,
      completed_phases: vec![],
      gaps: vec![],
      conflicts: vec![],
      ..InterviewSession::default()
    }
  }

  fn create_test_plan() -> ExecutionPlan {
    ExecutionPlan::new("test-session".to_string())
  }

  #[test]
  fn test_action_context_from_session() {
    let session = create_test_session();
    let context = ActionContext::from_session(&session);

    assert_eq!(context.stage, "Discovery");
    assert_eq!(context.current_phase, 1);
    assert!(context.completed_phases.is_empty());
    assert_eq!(context.open_gaps, 0);
    assert_eq!(context.pending_conflicts, 0);
    assert!(context.can_proceed);
  }

  #[test]
  fn test_action_context_with_gaps() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test_field".to_string(),
      description: "Test gap".to_string(),
      blocking: true,
      state: GapState::Open,
      ..Gap::default()
    });

    let context = ActionContext::from_session(&session);
    assert_eq!(context.open_gaps, 1);
    assert!(!context.can_proceed);
  }

  #[test]
  fn test_action_context_with_resolved_gaps() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test_field".to_string(),
      description: "Test gap".to_string(),
      blocking: true,
      state: GapState::Resolved {
        resolution: "resolved".to_string(),
      },
      ..Gap::default()
    });

    let context = ActionContext::from_session(&session);
    assert_eq!(context.open_gaps, 0);
  }

  #[test]
  fn test_action_context_with_conflicts() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "Test conflict".to_string(),
      options: vec![ConflictResolution {
        option: "Option A".to_string(),
        ..ConflictResolution::default()
      }],
      ..Conflict::default()
    });

    let context = ActionContext::from_session(&session);
    assert_eq!(context.pending_conflicts, 1);
  }

  #[test]
  fn test_action_context_with_metadata() {
    let session = create_test_session();
    let context =
      ActionContext::from_session(&session).with_metadata("key".to_string(), "value".to_string());

    assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
  }

  #[test]
  fn test_plan_next_json_output_creation() {
    let session = create_test_session();
    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output.next_action.is_some());
    assert_eq!(output.context.stage, "Discovery");
    assert!(output.suggestions.is_empty() || output.suggestions.len() <= 4);
    assert!(output.actionable_beads.is_empty());
    assert!(output.next_phase.is_none());
  }

  #[test]
  fn test_plan_next_json_output_serialization() {
    let session = create_test_session();
    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    let json = output.to_json();
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("next_action")));
    assert!(json_str.as_ref().map_or(false, |s| s.contains("context")));
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("suggestions")));
  }

  #[test]
  fn test_plan_next_json_output_compact_serialization() {
    let session = create_test_session();
    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    let json = output.to_json_compact();
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("next_action")));
    assert!(json_str.as_ref().map_or(false, |s| !s.contains('\n')));
  }

  #[test]
  fn test_format_next_action_json_function() {
    let session = create_test_session();
    let plan = create_test_plan();

    let json = format_next_action_json(&session, &plan);
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("next_action")));
    assert!(json_str.as_ref().map_or(false, |s| s.contains("context")));
  }

  #[test]
  fn test_format_next_action_json_compact_function() {
    let session = create_test_session();
    let plan = create_test_plan();

    let json = format_next_action_json_compact(&session, &plan);
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("next_action")));
    assert!(json_str.as_ref().map_or(false, |s| !s.contains('\n')));
  }

  #[test]
  fn test_json_output_with_blocking_gap() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "required_field".to_string(),
      description: "Missing required field".to_string(),
      blocking: true,
      state: GapState::Open,
      ..Gap::default()
    });

    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output.next_action.is_some());
    let action = output.next_action;
    assert_eq!(
      action.as_ref().map(|a| a.action_type),
      Some(ActionType::ResolveGap)
    );

    assert!(output
      .suggestions
      .iter()
      .any(|s| matches!(s.action_type, ActionType::ResolveGap)));
  }

  #[test]
  fn test_json_output_with_conflict() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("option_a".to_string(), "option_b".to_string()),
      description: "Conflicting requirements".to_string(),
      options: vec![
        ConflictResolution {
          option: "A".to_string(),
          ..ConflictResolution::default()
        },
        ConflictResolution {
          option: "B".to_string(),
          ..ConflictResolution::default()
        },
      ],
      ..Conflict::default()
    });

    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output
      .suggestions
      .iter()
      .any(|s| matches!(s.action_type, ActionType::ResolveConflict)));
  }

  #[test]
  fn test_json_output_with_actionable_beads() {
    let session = create_test_session();
    let mut plan = create_test_plan();

    let bead = match PlanBead::new("bead-1".to_string(), "Test bead".to_string(), 1) {
      Ok(b) => b.mark_ready(),
      Err(_) => return,
    };
    plan.beads.push(bead);

    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output.actionable_beads.contains(&"bead-1".to_string()));
    assert_eq!(output.next_phase, Some(1));
  }

  #[test]
  fn test_json_output_complete_session() {
    let mut session = create_test_session();
    session.stage = InterviewStage::Complete;

    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output.next_action.is_none());
    assert_eq!(output.context.stage, "Complete");
  }

  #[test]
  fn test_json_output_validation_stage() {
    let mut session = create_test_session();
    session.stage = InterviewStage::Validation;

    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    assert!(output
      .suggestions
      .iter()
      .any(|s| matches!(s.action_type, ActionType::ReviewPlan)));
  }

  #[test]
  fn test_json_round_trip() {
    let session = create_test_session();
    let plan = create_test_plan();
    let output = PlanNextJsonOutput::from_session_and_plan(&session, &plan);

    let json = output.to_json();
    assert!(json.is_ok());

    let json_str = json;
    let deserialized: Result<PlanNextJsonOutput, _> = match json_str.as_ref() {
      Ok(s) => serde_json::from_str(s),
      Err(_) => panic!("JSON serialization failed"),
    };

    assert!(deserialized.is_ok());
    if let Ok(parsed) = deserialized {
      assert_eq!(parsed.context.stage, "Discovery");
    }
  }

  #[test]
  fn test_action_suggestion_creation() {
    let suggestion = ActionSuggestion::new(
      ActionType::AnswerQuestion,
      "Answer the question".to_string(),
      "There are pending questions".to_string(),
    );

    assert_eq!(suggestion.action_type, ActionType::AnswerQuestion);
    assert_eq!(suggestion.description, "Answer the question");
    assert_eq!(suggestion.rationale, "There are pending questions");
  }

  #[test]
  fn test_next_action_serialization() {
    let action = NextAction::new(
      ActionType::ResolveGap,
      "gap-1".to_string(),
      "Resolve the gap".to_string(),
      "Blocking gap found".to_string(),
    )
    .with_priority(1);

    let json = serde_json::to_string(&action);
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("resolve_gap")));
    assert!(json_str.as_ref().map_or(false, |s| s.contains("gap-1")));
    assert!(json_str.as_ref().map_or(false, |s| s.contains("priority")));
  }

  #[test]
  fn test_action_enum_resolve_gap() {
    let action = Action::resolve_gap("gap-1".to_string(), "resolution text".to_string());
    assert_eq!(action.action_type(), ActionType::ResolveGap);
    assert!(action.description().contains("gap-1"));
    assert!(action.description().contains("resolution text"));
  }

  #[test]
  fn test_action_enum_answer_question() {
    let action = Action::answer_question("q-1".to_string(), "answer text".to_string());
    assert_eq!(action.action_type(), ActionType::AnswerQuestion);
    assert!(action.description().contains("q-1"));
    assert!(action.description().contains("answer text"));
  }

  #[test]
  fn test_action_enum_resolve_conflict() {
    let action = Action::resolve_conflict("c-1".to_string(), "chosen option".to_string());
    assert_eq!(action.action_type(), ActionType::ResolveConflict);
    assert!(action.description().contains("c-1"));
  }

  #[test]
  fn test_action_enum_complete_phase() {
    let action = Action::complete_phase(2);
    assert_eq!(action.action_type(), ActionType::CompletePhase);
    assert!(action.description().contains("phase 2"));
  }

  #[test]
  fn test_action_enum_review_plan() {
    let action = Action::review_plan(true, Some("looks good".to_string()));
    assert_eq!(action.action_type(), ActionType::ReviewPlan);
    assert!(action.description().contains("approve"));
    assert!(action.description().contains("looks good"));
  }

  #[test]
  fn test_action_enum_serialization() {
    let action = Action::resolve_gap("gap-1".to_string(), "fix".to_string());
    let json = serde_json::to_string(&action);
    assert!(json.is_ok());

    let json_str = json;
    assert!(json_str
      .as_ref()
      .map_or(false, |s| s.contains("resolve_gap")));
    assert!(json_str.as_ref().map_or(false, |s| s.contains("gap-1")));
  }
}
