//! WP23: Plan Next - Determine the next action in an interview session
//!
//! This module provides functionality to determine what action should be taken
//! next in an interview session based on the current state and execution plan.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::interview::types::{Gap, InterviewSession, InterviewStage};
use crate::intent::plan::types::{ExecutionPlan, PlanBead};
use serde::{Deserialize, Serialize};

/// Types of actions that can be recommended
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ActionType {
  /// Answer a pending question
  #[default]
  AnswerQuestion,
  /// Resolve a gap in requirements
  ResolveGap,
  /// Resolve a conflict between requirements
  ResolveConflict,
  /// Complete the current phase
  CompletePhase,
  /// Review and approve the execution plan
  ReviewPlan,
}

impl ActionType {
  /// Convert action type to string representation
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

  /// Get human-readable description
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

/// Next action recommendation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NextAction {
  /// Type of action to take
  pub action_type: ActionType,
  /// ID of the target entity (question, gap, conflict, phase, or plan)
  pub target_id: String,
  /// Human-readable description of the action
  pub description: String,
  /// Reason why this action is recommended
  pub reason: String,
  /// Priority of this action (lower = higher priority)
  #[serde(default)]
  pub priority: u32,
}

impl NextAction {
  /// Create a new next action
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

  /// Builder method to set priority
  #[must_use]
  pub const fn with_priority(mut self, priority: u32) -> Self {
    self.priority = priority;
    self
  }
}

/// Get the next recommended action for a session
///
/// Priority order:
/// 1. Resolve blocking gaps
/// 2. Resolve unresolved conflicts
/// 3. Answer questions for current round
/// 4. Complete current phase
/// 5. Review plan (if in validation stage)
///
/// # Arguments
/// * `session` - The interview session
/// * `plan` - The execution plan (optional, can be empty)
///
/// # Returns
/// The next recommended action, or None if session is complete
#[must_use]
pub fn get_next_action(session: &InterviewSession, plan: &ExecutionPlan) -> Option<NextAction> {
  // If session is complete, no action needed
  if session.stage == InterviewStage::Complete {
    return None;
  }

  // If session is paused, recommend resuming (represented as answer question with special ID)
  if session.stage == InterviewStage::Paused {
    return Some(NextAction::new(
      ActionType::AnswerQuestion,
      "resume".to_string(),
      "Resume the paused interview session".to_string(),
      "Session is currently paused".to_string(),
    ));
  }

  // Priority 1: Check for blocking gaps
  if let Some(action) = get_blocking_gap_action(session) {
    return Some(action);
  }

  // Priority 2: Check for unresolved conflicts
  if let Some(action) = get_conflict_action(session) {
    return Some(action);
  }

  // Priority 3: Check for pending questions in current round
  if let Some(action) = get_question_action(session) {
    return Some(action);
  }

  // Priority 4: Check for phase completion
  if let Some(action) = get_phase_completion_action(session, plan) {
    return Some(action);
  }

  // Priority 5: In validation stage, recommend reviewing plan
  if session.stage == InterviewStage::Validation {
    return Some(NextAction::new(
      ActionType::ReviewPlan,
      "plan".to_string(),
      "Review and approve the execution plan".to_string(),
      "Session is in validation stage".to_string(),
    ));
  }

  None
}

/// Get action for blocking gaps
fn get_blocking_gap_action(session: &InterviewSession) -> Option<NextAction> {
  session
    .gaps
    .iter()
    .filter(|g| g.blocking && !g.resolved)
    .min_by_key(|g| g.round)
    .map(|gap| {
      NextAction::new(
        ActionType::ResolveGap,
        gap.id.clone(),
        format!("Resolve gap: {}", gap.description),
        format!(
          "Blocking gap in field '{}' must be resolved to proceed",
          gap.field
        ),
      )
      .with_priority(1)
    })
}

/// Get action for unresolved conflicts
fn get_conflict_action(session: &InterviewSession) -> Option<NextAction> {
  session
    .conflicts
    .iter()
    .filter(|c| c.chosen.is_none())
    .map(|conflict| {
      NextAction::new(
        ActionType::ResolveConflict,
        conflict.id.clone(),
        format!("Resolve conflict: {}", conflict.description),
        format!(
          "Conflict between '{}' and '{}' must be resolved",
          conflict.between.0, conflict.between.1
        ),
      )
      .with_priority(2)
    })
    .next()
}

/// Get action for pending questions
fn get_question_action(session: &InterviewSession) -> Option<NextAction> {
  // In validation stage, don't suggest new questions - review instead
  if session.stage == InterviewStage::Validation {
    return None;
  }

  // Check if there are answers for the current round
  let current_round = session.get_current_round();
  let current_round_answers: std::collections::HashSet<&str> = session
    .answers
    .iter()
    .filter(|a| a.round == current_round)
    .map(|a| a.question_id.as_str())
    .collect();

  // If we have answers in the current round but haven't completed it,
  // suggest continuing or completing
  if !current_round_answers.is_empty() {
    // Check if we can complete the round
    if session.can_proceed().is_ok() {
      return Some(
        NextAction::new(
          ActionType::AnswerQuestion,
          format!("round-{current_round}-complete"),
          format!("Complete round {current_round} or add more answers"),
          "You have answers ready; you can complete the round or add more details".to_string(),
        )
        .with_priority(3),
      );
    }
  }

  // If no answers yet in current round, suggest starting
  if current_round_answers.is_empty() {
    return Some(
      NextAction::new(
        ActionType::AnswerQuestion,
        format!("round-{current_round}-start"),
        format!("Start answering questions for round {current_round}"),
        format!("Round {current_round} has not started yet"),
      )
      .with_priority(3),
    );
  }

  None
}

/// Get action for phase completion
fn get_phase_completion_action(
  session: &InterviewSession,
  plan: &ExecutionPlan,
) -> Option<NextAction> {
  // In validation stage, skip phase completion - review plan instead
  if session.stage == InterviewStage::Validation {
    return None;
  }

  // Check if current phase is complete
  let current_phase = session.current_phase;

  // Get beads for current phase
  let phase_beads = plan.get_phase_beads(current_phase);

  // If there are beads in the plan for this phase
  if !phase_beads.is_empty() {
    let completed_count = phase_beads.iter().filter(|b| b.completed).count();

    // If all beads are completed, suggest completing the phase
    if completed_count == phase_beads.len() {
      return Some(
        NextAction::new(
          ActionType::CompletePhase,
          format!("phase-{current_phase}"),
          format!("Complete phase {current_phase} (all beads done)"),
          format!(
            "All {} work items in phase {} are complete",
            phase_beads.len(),
            current_phase
          ),
        )
        .with_priority(4),
      );
    }

    // If some beads are actionable, suggest working on them
    let actionable = plan.get_actionable_beads();
    let actionable_in_phase: Vec<&&PlanBead> = actionable
      .iter()
      .filter(|b| b.phase == current_phase)
      .collect();

    if !actionable_in_phase.is_empty() {
      let bead = actionable_in_phase[0];
      return Some(
        NextAction::new(
          ActionType::CompletePhase,
          bead.id.clone(),
          format!("Work on: {}", bead.title),
          format!(
            "{}/{} beads complete in phase {}",
            completed_count,
            phase_beads.len(),
            current_phase
          ),
        )
        .with_priority(4),
      );
    }
  }

  // If no beads in plan, check if session can proceed to next phase
  if session.can_proceed().is_ok() && !session.completed_phases.contains(&current_phase) {
    return Some(
      NextAction::new(
        ActionType::CompletePhase,
        format!("phase-{current_phase}"),
        format!("Complete phase {current_phase}"),
        "Phase requirements are satisfied".to_string(),
      )
      .with_priority(4),
    );
  }

  None
}

/// Determine the next phase number based on the execution plan
///
/// # Arguments
/// * `plan` - The execution plan
///
/// # Returns
/// The next phase number, or None if all phases are complete
#[must_use]
pub fn determine_next_phase(plan: &ExecutionPlan) -> Option<u32> {
  if plan.beads.is_empty() {
    return None;
  }

  // Find incomplete phases
  let mut phase_numbers: Vec<u32> = plan
    .beads
    .iter()
    .filter(|b| !b.completed)
    .map(|b| b.phase)
    .collect();

  if phase_numbers.is_empty() {
    return None;
  }

  // Sort and deduplicate
  phase_numbers.sort_unstable();
  phase_numbers.dedup();

  // Return the lowest incomplete phase
  phase_numbers.into_iter().next()
}

/// Get all beads that are actionable (dependencies satisfied, not completed)
///
/// # Arguments
/// * `plan` - The execution plan
///
/// # Returns
/// Vector of references to actionable beads, sorted by priority
#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  let mut actionable = plan.get_actionable_beads();
  actionable.sort_by_key(|b| (b.phase, b.priority));
  actionable
}

/// Get all blocking gaps for a session
///
/// # Arguments
/// * `session` - The interview session
///
/// # Returns
/// Vector of references to blocking, unresolved gaps
#[must_use]
pub fn get_blocking_gaps(session: &InterviewSession) -> Vec<&Gap> {
  session.get_blocking_gaps()
}

/// Check if a session can proceed (no blocking issues)
///
/// # Arguments
/// * `session` - The interview session
///
/// # Returns
/// true if the session can proceed
#[must_use]
pub fn can_proceed(session: &InterviewSession) -> bool {
  session.can_proceed().is_ok()
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
  use crate::intent::interview::types::{Conflict, ConflictResolution, Gap, Profile};

  fn create_test_session() -> InterviewSession {
    InterviewSession::new(
      "test-session".to_string(),
      Profile::Api,
      "2026-02-27T00:00:00Z".to_string(),
    )
  }

  fn create_test_plan() -> ExecutionPlan {
    ExecutionPlan::new("test-session".to_string())
  }

  #[test]
  fn test_action_type_as_str() {
    assert_eq!(ActionType::AnswerQuestion.as_str(), "answer_question");
    assert_eq!(ActionType::ResolveGap.as_str(), "resolve_gap");
    assert_eq!(ActionType::ResolveConflict.as_str(), "resolve_conflict");
    assert_eq!(ActionType::CompletePhase.as_str(), "complete_phase");
    assert_eq!(ActionType::ReviewPlan.as_str(), "review_plan");
  }

  #[test]
  fn test_action_type_description() {
    assert!(!ActionType::AnswerQuestion.description().is_empty());
    assert!(!ActionType::ResolveGap.description().is_empty());
  }

  #[test]
  fn test_next_action_new() {
    let action = NextAction::new(
      ActionType::ResolveGap,
      "gap-1".to_string(),
      "Resolve the gap".to_string(),
      "Gap is blocking".to_string(),
    );

    assert_eq!(action.action_type, ActionType::ResolveGap);
    assert_eq!(action.target_id, "gap-1");
    assert_eq!(action.priority, 0);
  }

  #[test]
  fn test_next_action_with_priority() {
    let action = NextAction::new(
      ActionType::ResolveGap,
      "gap-1".to_string(),
      "Description".to_string(),
      "Reason".to_string(),
    )
    .with_priority(5);

    assert_eq!(action.priority, 5);
  }

  #[test]
  fn test_get_next_action_complete_session() {
    let mut session = create_test_session();
    session.stage = InterviewStage::Complete;
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_none());
  }

  #[test]
  fn test_get_next_action_paused_session() {
    let mut session = create_test_session();
    session.stage = InterviewStage::Paused;
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_some());
    let action = action.expect("has action");
    assert_eq!(action.action_type, ActionType::AnswerQuestion);
    assert_eq!(action.target_id, "resume");
  }

  #[test]
  fn test_get_next_action_blocking_gap() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "base_url".to_string(),
      description: "Missing base URL".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    });
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_some());
    let action = action.expect("has action");
    assert_eq!(action.action_type, ActionType::ResolveGap);
    assert_eq!(action.target_id, "gap-1");
  }

  #[test]
  fn test_get_next_action_resolved_gap() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "base_url".to_string(),
      description: "Missing base URL".to_string(),
      blocking: true,
      resolved: true, // Already resolved
      ..Gap::default()
    });
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    // Should not suggest resolving an already resolved gap
    if let Some(a) = &action {
      assert_ne!(a.action_type, ActionType::ResolveGap);
    }
  }

  #[test]
  fn test_get_next_action_conflict() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "CAP theorem conflict".to_string(),
      impact: "High".to_string(),
      options: vec![ConflictResolution {
        option: "opt1".to_string(),
        description: "Option 1".to_string(),
        tradeoffs: "Tradeoffs".to_string(),
        recommendation: true,
      }],
      chosen: None,
    });
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_some());
    let action = action.expect("has action");
    assert_eq!(action.action_type, ActionType::ResolveConflict);
    assert_eq!(action.target_id, "conflict-1");
  }

  #[test]
  fn test_get_next_action_resolved_conflict() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "Conflict".to_string(),
      impact: "High".to_string(),
      options: vec![ConflictResolution::default()],
      chosen: Some(0), // Already resolved
    });
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    if let Some(a) = &action {
      assert_ne!(a.action_type, ActionType::ResolveConflict);
    }
  }

  #[test]
  fn test_get_next_action_validation_stage() {
    let mut session = create_test_session();
    session.stage = InterviewStage::Validation;
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_some());
    let action = action.expect("has action");
    assert_eq!(action.action_type, ActionType::ReviewPlan);
  }

  #[test]
  fn test_get_next_action_new_session() {
    let session = create_test_session();
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan);
    assert!(action.is_some());
    // New session should suggest answering questions
    let action = action.expect("has action");
    assert_eq!(action.action_type, ActionType::AnswerQuestion);
  }

  #[test]
  fn test_determine_next_phase_empty_plan() {
    let plan = create_test_plan();
    assert!(determine_next_phase(&plan).is_none());
  }

  #[test]
  fn test_determine_next_phase_with_beads() {
    let mut plan = create_test_plan();
    plan
      .add_bead(PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"))
      .expect("add");
    plan
      .add_bead(PlanBead::new("b2".to_string(), "Second".to_string(), 2).expect("valid"))
      .expect("add");

    assert_eq!(determine_next_phase(&plan), Some(1));
  }

  #[test]
  fn test_determine_next_phase_phase1_complete() {
    let mut plan = create_test_plan();
    let mut bead1 = PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid");
    bead1.completed = true;
    plan.add_bead(bead1).expect("add");

    plan
      .add_bead(PlanBead::new("b2".to_string(), "Second".to_string(), 2).expect("valid"))
      .expect("add");

    assert_eq!(determine_next_phase(&plan), Some(2));
  }

  #[test]
  fn test_determine_next_phase_all_complete() {
    let mut plan = create_test_plan();
    let mut bead1 = PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid");
    bead1.completed = true;
    plan.add_bead(bead1).expect("add");

    let mut bead2 = PlanBead::new("b2".to_string(), "Second".to_string(), 2).expect("valid");
    bead2.completed = true;
    plan.add_bead(bead2).expect("add");

    assert!(determine_next_phase(&plan).is_none());
  }

  #[test]
  fn test_get_actionable_beads_empty_plan() {
    let plan = create_test_plan();
    let actionable = get_actionable_beads(&plan);
    assert!(actionable.is_empty());
  }

  #[test]
  fn test_get_actionable_beads_with_beads() {
    let mut plan = create_test_plan();
    plan
      .add_bead(PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"))
      .expect("add");
    plan
      .add_bead(
        PlanBead::new("b2".to_string(), "Second".to_string(), 1)
          .expect("valid")
          .with_dependency("b1".to_string()),
      )
      .expect("add");

    let actionable = get_actionable_beads(&plan);
    assert_eq!(actionable.len(), 1);
    assert_eq!(actionable[0].id, "b1");
  }

  #[test]
  fn test_get_actionable_beads_sorted_by_priority() {
    let mut plan = create_test_plan();
    plan
      .add_bead(
        PlanBead::new("b1".to_string(), "First".to_string(), 1)
          .expect("valid")
          .with_priority(5),
      )
      .expect("add");
    plan
      .add_bead(
        PlanBead::new("b2".to_string(), "Second".to_string(), 1)
          .expect("valid")
          .with_priority(1),
      )
      .expect("add");
    plan
      .add_bead(
        PlanBead::new("b3".to_string(), "Third".to_string(), 1)
          .expect("valid")
          .with_priority(3),
      )
      .expect("add");

    let actionable = get_actionable_beads(&plan);
    assert_eq!(actionable.len(), 3);
    // Should be sorted by priority
    assert_eq!(actionable[0].id, "b2"); // priority 1
    assert_eq!(actionable[1].id, "b3"); // priority 3
    assert_eq!(actionable[2].id, "b1"); // priority 5
  }

  #[test]
  fn test_can_proceed_no_gaps() {
    let session = create_test_session();
    assert!(can_proceed(&session));
  }

  #[test]
  fn test_can_proceed_with_blocking_gap() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    });

    assert!(!can_proceed(&session));
  }

  #[test]
  fn test_get_blocking_gaps_empty() {
    let session = create_test_session();
    let gaps = get_blocking_gaps(&session);
    assert!(gaps.is_empty());
  }

  #[test]
  fn test_get_blocking_gaps_with_gaps() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    });
    session.gaps.push(Gap {
      id: "gap-2".to_string(),
      field: "test2".to_string(),
      blocking: false, // Non-blocking
      resolved: false,
      ..Gap::default()
    });
    session.gaps.push(Gap {
      id: "gap-3".to_string(),
      field: "test3".to_string(),
      blocking: true,
      resolved: true, // Resolved
      ..Gap::default()
    });

    let gaps = get_blocking_gaps(&session);
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].id, "gap-1");
  }

  #[test]
  fn test_priority_ordering_gap_over_conflict() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    });
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "Conflict".to_string(),
      impact: "High".to_string(),
      options: vec![],
      chosen: None,
    });
    let plan = create_test_plan();

    let action = get_next_action(&session, &plan).expect("has action");
    // Gap should have priority over conflict
    assert_eq!(action.action_type, ActionType::ResolveGap);
  }

  #[test]
  fn test_serde_roundtrip_action_type() {
    let types = [
      ActionType::AnswerQuestion,
      ActionType::ResolveGap,
      ActionType::ResolveConflict,
      ActionType::CompletePhase,
      ActionType::ReviewPlan,
    ];

    for action_type in types {
      let json = serde_json::to_string(&action_type).expect("should serialize");
      let parsed: ActionType = serde_json::from_str(&json).expect("should deserialize");
      assert_eq!(action_type, parsed);
    }
  }

  #[test]
  fn test_serde_roundtrip_next_action() {
    let action = NextAction::new(
      ActionType::ResolveGap,
      "gap-1".to_string(),
      "Resolve the gap".to_string(),
      "Gap is blocking progress".to_string(),
    )
    .with_priority(3);

    let json = serde_json::to_string(&action).expect("should serialize");
    let parsed: NextAction = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(action, parsed);
  }
}
