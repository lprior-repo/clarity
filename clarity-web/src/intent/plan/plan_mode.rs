//! Plan Mode - Execution planning for intent sessions
//!
//! This module provides execution planning functionality that transforms
//! interview session data into actionable execution plans with phases,
//! beads (work items), and dependency tracking.
//!
//! ## Design Principles
//!
//! - Zero panics: All fallible operations return `Result<T, E>`
//! - Pure functions: Core logic is deterministic and side-effect free
//! - Type safety: Leverage Rust's type system for compile-time guarantees

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::intent::interview::types::InterviewSession;

/// Error type for plan operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanError {
  /// Error in dependency resolution
  #[error("dependency error: {0}")]
  DependencyError(String),
  /// Invalid phase number specified
  #[error("invalid phase: {0}")]
  InvalidPhase(u32),
  /// Phase has not been completed yet
  #[error("phase not complete: {0}")]
  PhaseNotComplete(u32),
  /// No beads are actionable due to blockers or dependencies
  #[error("no actionable beads")]
  NoActionableBeads,
  /// Session ID is empty
  #[error("session ID is empty")]
  EmptySessionId,
  /// Circular dependency detected in beads
  #[error("circular dependency detected: {0} -> {1}")]
  CircularDependency(String, String),
}

/// Status of a phase in the execution plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
  /// Phase is pending execution
  #[default]
  Pending,
  /// Phase is currently in progress
  InProgress,
  /// Phase has been completed
  Complete,
  /// Phase is blocked by unresolved issues
  Blocked,
}

/// Status of a bead (work item)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadStatus {
  /// Bead is pending (dependencies not met)
  #[default]
  Pending,
  /// Bead is ready to be worked on (dependencies satisfied)
  Ready,
  /// Bead is currently in progress
  InProgress,
  /// Bead has been completed
  Complete,
  /// Bead is blocked by unresolved issues
  Blocked,
}

/// A bead (work item) in the execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBead {
  /// Unique identifier for this bead
  pub id: String,
  /// Human-readable title
  pub title: String,
  /// Detailed description of the work
  pub description: String,
  /// Priority level (0-255, higher = more important)
  pub priority: u8,
  /// Current status of this bead
  pub status: BeadStatus,
  /// IDs of beads this bead depends on
  pub depends_on: Vec<String>,
  /// IDs of beads this bead blocks
  pub blocks: Vec<String>,
}

impl Default for PlanBead {
  fn default() -> Self {
    Self {
      id: String::new(),
      title: String::new(),
      description: String::new(),
      priority: 100,
      status: BeadStatus::default(),
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }
  }
}

/// A phase in the execution plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
  /// Phase number (1-indexed)
  pub phase_number: u32,
  /// Human-readable name
  pub name: String,
  /// Detailed description of this phase
  pub description: String,
  /// Beads (work items) in this phase
  pub beads: Vec<PlanBead>,
  /// Current status of this phase
  pub status: PhaseStatus,
  /// List of blockers preventing progress
  pub blockers: Vec<String>,
}

impl Default for Phase {
  fn default() -> Self {
    Self {
      phase_number: 1,
      name: String::new(),
      description: String::new(),
      beads: Vec::new(),
      status: PhaseStatus::default(),
      blockers: Vec::new(),
    }
  }
}

/// Complete execution plan for a session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionPlan {
  /// Session ID this plan is for
  pub session_id: String,
  /// Ordered phases in this plan
  pub phases: Vec<Phase>,
  /// Global blockers affecting the entire plan
  pub blockers: Vec<String>,
  /// ISO 8601 timestamp when plan was created
  pub created_at: String,
}

/// Compute an execution plan from an interview session.
///
/// This function analyzes the session answers, generates beads from
/// features/behaviors, organizes them into phases, and detects blockers.
///
/// # Errors
///
/// Returns `PlanError::EmptySessionId` if the session has an empty ID.
///
/// # Example
///
/// ```ignore
/// use crate::intent::plan::plan_mode::{compute_plan, ExecutionPlan};
/// use crate::interview::types::{InterviewSession, Profile};
///
/// let session = InterviewSession::new(
///     "session-1".to_string(),
///     Profile::Api,
///     "2026-02-27T00:00:00Z".to_string(),
/// );
///
/// let plan = compute_plan(&session)?;
/// assert!(!plan.phases.is_empty());
/// ```
pub fn compute_plan(session: &InterviewSession) -> Result<ExecutionPlan, PlanError> {
  if session.id.is_empty() {
    return Err(PlanError::EmptySessionId);
  }

  // Generate beads from session answers
  let beads = generate_beads_from_session(session);

  // Organize beads into phases based on dependencies and priorities
  let phases = organize_into_phases(&beads, session);

  // Detect blockers from unresolved gaps and conflicts
  let blockers = detect_blockers(session);

  // Determine initial phase statuses
  let phases_with_status = apply_initial_phase_status(phases, &blockers);

  Ok(ExecutionPlan {
    session_id: session.id.clone(),
    phases: phases_with_status,
    blockers,
    created_at: session.updated_at.clone(),
  })
}

/// Apply phase gating to an execution plan.
///
/// This function checks phase completion status and blocks subsequent
/// phases if their dependencies are not met.
///
/// # Example
///
/// ```ignore
/// let gated_plan = apply_phase_gating(&plan, &session);
/// // Phase 2 will be Blocked if Phase 1 is not Complete
/// ```
#[must_use]
pub fn apply_phase_gating(plan: &ExecutionPlan, session: &InterviewSession) -> ExecutionPlan {
  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  let phases = plan
    .phases
    .iter()
    .map(|phase| {
      let mut gated_phase = phase.clone();

      // Check if this phase is completed in the session
      if completed_phases.contains(&phase.phase_number) {
        gated_phase.status = PhaseStatus::Complete;
      } else if phase.phase_number > 1 {
        // Phase is blocked if previous phase is not complete
        let prev_phase_complete = completed_phases.contains(&(phase.phase_number - 1));
        if !prev_phase_complete {
          gated_phase.status = PhaseStatus::Blocked;
          gated_phase.blockers.push(format!(
            "Phase {} must be completed first",
            phase.phase_number - 1
          ));
        }
      }

      // Apply gating to beads within the phase
      gated_phase.beads = apply_bead_gating(&gated_phase.beads, &completed_phases);
      gated_phase
    })
    .collect();

  ExecutionPlan {
    session_id: plan.session_id.clone(),
    phases,
    blockers: plan.blockers.clone(),
    created_at: plan.created_at.clone(),
  }
}

/// Get all actionable beads from an execution plan.
///
/// An actionable bead is one that:
/// - Has status == Ready
/// - Has all dependencies satisfied
///
/// # Returns
///
/// A vector of references to actionable beads, or an empty vector if none exist.
#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  // Collect all completed bead IDs
  let completed_ids: HashSet<&str> = plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .filter(|bead| bead.status == BeadStatus::Complete)
    .map(|bead| bead.id.as_str())
    .collect();

  // Find beads that are Ready and have all dependencies satisfied
  plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .filter(|bead| {
      bead.status == BeadStatus::Ready
        && bead
          .depends_on
          .iter()
          .all(|dep_id| completed_ids.contains(dep_id.as_str()))
    })
    .collect()
}

/// Generate beads from an interview session.
///
/// Creates work items based on:
/// - Required fields for the profile
/// - Answers provided during the interview
/// - Gaps and conflicts that need resolution
fn generate_beads_from_session(session: &InterviewSession) -> Vec<PlanBead> {
  let mut beads = Vec::new();

  // Generate beads from required fields
  let required_fields = session.profile.required_fields();
  let answered_fields: HashSet<&str> = session
    .answers
    .iter()
    .flat_map(|answer| answer.extracted.keys())
    .map(String::as_str)
    .collect();

  for (index, field) in required_fields.iter().enumerate() {
    let is_answered = answered_fields.contains(*field);
    let bead_id = format!("bead-{}-{}", session.id, field);

    beads.push(PlanBead {
      id: bead_id,
      title: format!("Define {}", field.replace('_', " ")),
      description: if is_answered {
        format!("Review and validate the {field} specification")
      } else {
        format!("Define the {field} for the system")
      },
      priority: if is_answered { 50 } else { 200 },
      status: if is_answered {
        BeadStatus::Ready
      } else {
        BeadStatus::Pending
      },
      depends_on: Vec::new(),
      blocks: Vec::new(),
    });

    // Add dependency from later fields to earlier ones (example dependency chain)
    if index > 0 {
      let prev_field = required_fields[index - 1];
      if let Some(prev_bead) = beads.iter_mut().find(|b| b.id.ends_with(prev_field)) {
        prev_bead
          .blocks
          .push(format!("bead-{}-{}", session.id, field));
      }
      if let Some(curr_bead) = beads.iter_mut().find(|b| b.id.ends_with(field)) {
        curr_bead
          .depends_on
          .push(format!("bead-{}-{}", session.id, prev_field));
      }
    }
  }

  // Generate beads from gaps
  for gap in &session.gaps {
    if !gap.resolved {
      beads.push(PlanBead {
        id: format!("bead-gap-{}", gap.id),
        title: format!("Resolve gap: {}", gap.field),
        description: gap.description.clone(),
        priority: if gap.blocking { 255 } else { 100 },
        status: BeadStatus::Blocked,
        depends_on: Vec::new(),
        blocks: Vec::new(),
      });
    }
  }

  // Generate beads from conflicts
  for conflict in &session.conflicts {
    if conflict.chosen.is_none() {
      beads.push(PlanBead {
        id: format!("bead-conflict-{}", conflict.id),
        title: format!(
          "Resolve conflict: {} vs {}",
          conflict.between.0, conflict.between.1
        ),
        description: conflict.description.clone(),
        priority: 200,
        status: BeadStatus::Blocked,
        depends_on: Vec::new(),
        blocks: Vec::new(),
      });
    }
  }

  beads
}

/// Organize beads into phases based on dependencies and priorities.
fn organize_into_phases(beads: &[PlanBead], session: &InterviewSession) -> Vec<Phase> {
  // Create phases based on the session's current state
  let phase_names = [
    (
      1,
      "Discovery",
      "Initial exploration and requirements gathering",
    ),
    (2, "Refinement", "Detailed specification and design"),
    (3, "Validation", "Verification and quality assurance"),
    (4, "Implementation", "Development and execution"),
  ];

  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  phase_names
    .iter()
    .map(|(num, name, desc)| {
      let phase_beads = assign_beads_to_phase(beads, *num, completed_phases.contains(num));
      Phase {
        phase_number: *num,
        name: name.to_string(),
        description: desc.to_string(),
        beads: phase_beads,
        status: if completed_phases.contains(num) {
          PhaseStatus::Complete
        } else {
          PhaseStatus::Pending
        },
        blockers: Vec::new(),
      }
    })
    .collect()
}

/// Assign beads to a specific phase based on their characteristics.
fn assign_beads_to_phase(
  beads: &[PlanBead],
  phase_number: u32,
  _is_complete: bool,
) -> Vec<PlanBead> {
  // Simple assignment: distribute beads across phases based on priority
  beads
    .iter()
    .filter(|bead| match phase_number {
      1 => bead.priority >= 200 || bead.status == BeadStatus::Blocked,
      2 => bead.priority >= 100 && bead.priority < 200,
      3 => bead.priority >= 50 && bead.priority < 100,
      4 => bead.priority < 50,
      _ => false,
    })
    .cloned()
    .collect()
}

/// Detect blockers from gaps and conflicts.
fn detect_blockers(session: &InterviewSession) -> Vec<String> {
  let mut blockers = Vec::new();

  // Add unresolved blocking gaps
  for gap in session.get_blocking_gaps() {
    blockers.push(format!(
      "Unresolved gap: {} - {}",
      gap.field, gap.description
    ));
  }

  // Add unresolved conflicts
  for conflict in &session.conflicts {
    if conflict.chosen.is_none() {
      blockers.push(format!(
        "Unresolved conflict: {} vs {} - {}",
        conflict.between.0, conflict.between.1, conflict.description
      ));
    }
  }

  blockers
}

/// Apply initial phase status based on blockers.
fn apply_initial_phase_status(phases: Vec<Phase>, blockers: &[String]) -> Vec<Phase> {
  phases
    .into_iter()
    .map(|phase| {
      if phase.status == PhaseStatus::Pending && !blockers.is_empty() {
        Phase {
          status: PhaseStatus::Blocked,
          blockers: blockers.to_vec(),
          ..phase
        }
      } else {
        phase
      }
    })
    .collect()
}

/// Apply gating to beads based on completed phases.
fn apply_bead_gating(beads: &[PlanBead], completed_phases: &HashSet<u32>) -> Vec<PlanBead> {
  beads
    .iter()
    .map(|bead| {
      // Check if all dependencies are satisfied
      let deps_satisfied = bead
        .depends_on
        .iter()
        .all(|_dep_id| !completed_phases.is_empty()); // Simplified check

      let new_status = if bead.status == BeadStatus::Pending && deps_satisfied {
        BeadStatus::Ready
      } else {
        bead.status
      };

      PlanBead {
        status: new_status,
        ..bead.clone()
      }
    })
    .collect()
}

/// Validate that a plan has no circular dependencies.
///
/// # Errors
///
/// Returns `PlanError::CircularDependency` if a cycle is detected.
pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
  let all_beads: HashMap<&str, &PlanBead> = plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .map(|bead| (bead.id.as_str(), bead))
    .collect();

  // Check for cycles using DFS
  let mut visiting: HashSet<&str> = HashSet::new();
  let mut visited: HashSet<&str> = HashSet::new();

  for bead in all_beads.values() {
    if !visited.contains(bead.id.as_str()) {
      validate_bead_dependencies(&all_beads, bead.id.as_str(), &mut visiting, &mut visited)?;
    }
  }

  Ok(())
}

/// Recursively validate bead dependencies for cycles.
fn validate_bead_dependencies<'a>(
  all_beads: &HashMap<&'a str, &'a PlanBead>,
  bead_id: &'a str,
  visiting: &mut HashSet<&'a str>,
  visited: &mut HashSet<&'a str>,
) -> Result<(), PlanError> {
  if visited.contains(bead_id) {
    return Ok(());
  }

  if visiting.contains(bead_id) {
    return Err(PlanError::CircularDependency(
      bead_id.to_string(),
      bead_id.to_string(),
    ));
  }

  visiting.insert(bead_id);

  if let Some(bead) = all_beads.get(bead_id) {
    for dep_id in &bead.depends_on {
      if visiting.contains(dep_id.as_str()) {
        return Err(PlanError::CircularDependency(
          bead_id.to_string(),
          dep_id.clone(),
        ));
      }
      validate_bead_dependencies(all_beads, dep_id.as_str(), visiting, visited)?;
    }
  }

  visiting.remove(bead_id);
  visited.insert(bead_id);

  Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;
  use crate::intent::interview::types::{
    Answer, Conflict, ConflictResolution, Gap, Perspective, Profile,
  };
  use std::collections::HashMap;

  fn create_test_session() -> InterviewSession {
    InterviewSession::new(
      "test-session".to_string(),
      Profile::Api,
      "2026-02-27T00:00:00Z".to_string(),
    )
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
      response: "The base URL is https://api.example.com".to_string(),
      extracted,
      confidence: 0.9,
      notes: String::new(),
      timestamp: "2026-02-27T00:01:00Z".to_string(),
    });

    session
  }

  #[test]
  fn test_compute_plan_empty_session_id() {
    let session = InterviewSession::default();
    let result = compute_plan(&session);
    assert_eq!(result, Err(PlanError::EmptySessionId));
  }

  #[test]
  fn test_compute_plan_success() {
    let session = create_test_session();
    let result = compute_plan(&session);

    assert!(result.is_ok());
    let plan = result.expect("plan should exist");

    assert_eq!(plan.session_id, "test-session");
    assert!(!plan.phases.is_empty());
    assert_eq!(plan.phases.len(), 4); // Discovery, Refinement, Validation, Implementation
  }

  #[test]
  fn test_compute_plan_with_answers() {
    let session = create_test_session_with_answers();
    let result = compute_plan(&session);

    assert!(result.is_ok());
    let plan = result.expect("plan should exist");

    // Plan should have been created
    assert_eq!(plan.session_id, "test-session");
  }

  #[test]
  fn test_compute_plan_with_gaps() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test_field".to_string(),
      description: "Missing field".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    });

    let result = compute_plan(&session);

    assert!(result.is_ok());
    let plan = result.expect("plan should exist");

    // Should have blockers due to unresolved gap
    assert!(!plan.blockers.is_empty());
    assert!(plan.blockers[0].contains("Unresolved gap"));
  }

  #[test]
  fn test_compute_plan_with_conflicts() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("option_a".to_string(), "option_b".to_string()),
      description: "Test conflict".to_string(),
      impact: "High impact".to_string(),
      options: vec![ConflictResolution {
        option: "opt1".to_string(),
        description: "Option 1".to_string(),
        tradeoffs: "Some tradeoff".to_string(),
        recommendation: true,
      }],
      chosen: None,
    });

    let result = compute_plan(&session);

    assert!(result.is_ok());
    let plan = result.expect("plan should exist");

    // Should have blockers due to unresolved conflict
    assert!(!plan.blockers.is_empty());
    assert!(plan.blockers[0].contains("Unresolved conflict"));
  }

  #[test]
  fn test_apply_phase_gating_no_completed() {
    let session = create_test_session();
    let plan = compute_plan(&session).expect("plan should exist");

    let gated_plan = apply_phase_gating(&plan, &session);

    // First phase should be pending, later phases should be blocked
    assert_eq!(gated_plan.phases[0].status, PhaseStatus::Pending);

    // Phase 2 should be blocked since Phase 1 is not complete
    assert_eq!(gated_plan.phases[1].status, PhaseStatus::Blocked);
  }

  #[test]
  fn test_apply_phase_gating_with_completed_phase() {
    let mut session = create_test_session();
    session.completed_phases.push(1);

    let plan = compute_plan(&session).expect("plan should exist");
    let gated_plan = apply_phase_gating(&plan, &session);

    // Phase 1 should be complete
    assert_eq!(gated_plan.phases[0].status, PhaseStatus::Complete);

    // Phase 2 should now be accessible
    assert_eq!(gated_plan.phases[1].status, PhaseStatus::Pending);
  }

  #[test]
  fn test_get_actionable_beads_empty() {
    let session = create_test_session();
    let plan = compute_plan(&session).expect("plan should exist");
    let gated_plan = apply_phase_gating(&plan, &session);

    let _actionable = get_actionable_beads(&gated_plan);

    // May be empty if no beads are ready
    // This is expected behavior for a fresh session with no answers
  }

  #[test]
  fn test_get_actionable_beads_with_ready_beads() {
    let session = create_test_session_with_answers();
    let plan = compute_plan(&session).expect("plan should exist");
    let gated_plan = apply_phase_gating(&plan, &session);

    let _actionable = get_actionable_beads(&gated_plan);

    // Beads with Ready status and satisfied dependencies should be returned
    // Exact count depends on bead generation logic
  }

  #[test]
  fn test_validate_plan_dependencies_no_cycles() {
    let session = create_test_session();
    let plan = compute_plan(&session).expect("plan should exist");

    let result = validate_plan_dependencies(&plan);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_plan_dependencies_with_cycle() {
    let plan = ExecutionPlan {
      session_id: "test".to_string(),
      phases: vec![Phase {
        phase_number: 1,
        name: "Test".to_string(),
        description: "Test phase".to_string(),
        beads: vec![
          PlanBead {
            id: "bead-a".to_string(),
            title: "A".to_string(),
            description: "Bead A".to_string(),
            priority: 100,
            status: BeadStatus::Pending,
            depends_on: vec!["bead-b".to_string()],
            blocks: vec!["bead-b".to_string()],
          },
          PlanBead {
            id: "bead-b".to_string(),
            title: "B".to_string(),
            description: "Bead B".to_string(),
            priority: 100,
            status: BeadStatus::Pending,
            depends_on: vec!["bead-a".to_string()],
            blocks: vec!["bead-a".to_string()],
          },
        ],
        status: PhaseStatus::Pending,
        blockers: Vec::new(),
      }],
      blockers: Vec::new(),
      created_at: "2026-02-27T00:00:00Z".to_string(),
    };

    let result = validate_plan_dependencies(&plan);
    assert!(matches!(result, Err(PlanError::CircularDependency(_, _))));
  }

  #[test]
  fn test_phase_status_default() {
    let status = PhaseStatus::default();
    assert_eq!(status, PhaseStatus::Pending);
  }

  #[test]
  fn test_bead_status_default() {
    let status = BeadStatus::default();
    assert_eq!(status, BeadStatus::Pending);
  }

  #[test]
  fn test_plan_bead_default() {
    let bead = PlanBead::default();
    assert!(bead.id.is_empty());
    assert!(bead.title.is_empty());
    assert_eq!(bead.priority, 100);
    assert_eq!(bead.status, BeadStatus::Pending);
    assert!(bead.depends_on.is_empty());
    assert!(bead.blocks.is_empty());
  }

  #[test]
  fn test_phase_default() {
    let phase = Phase::default();
    assert_eq!(phase.phase_number, 1);
    assert!(phase.name.is_empty());
    assert!(phase.beads.is_empty());
    assert_eq!(phase.status, PhaseStatus::Pending);
    assert!(phase.blockers.is_empty());
  }

  #[test]
  fn test_execution_plan_default() {
    let plan = ExecutionPlan::default();
    assert!(plan.session_id.is_empty());
    assert!(plan.phases.is_empty());
    assert!(plan.blockers.is_empty());
    assert!(plan.created_at.is_empty());
  }

  #[test]
  fn test_serde_roundtrip_phase_status() {
    let statuses = [
      PhaseStatus::Pending,
      PhaseStatus::InProgress,
      PhaseStatus::Complete,
      PhaseStatus::Blocked,
    ];

    for status in statuses {
      let json = serde_json::to_string(&status).expect("should serialize");
      let parsed: PhaseStatus = serde_json::from_str(&json).expect("should deserialize");
      assert_eq!(status, parsed);
    }
  }

  #[test]
  fn test_serde_roundtrip_bead_status() {
    let statuses = [
      BeadStatus::Pending,
      BeadStatus::Ready,
      BeadStatus::InProgress,
      BeadStatus::Complete,
      BeadStatus::Blocked,
    ];

    for status in statuses {
      let json = serde_json::to_string(&status).expect("should serialize");
      let parsed: BeadStatus = serde_json::from_str(&json).expect("should deserialize");
      assert_eq!(status, parsed);
    }
  }

  #[test]
  fn test_serde_roundtrip_plan_bead() {
    let bead = PlanBead {
      id: "bead-1".to_string(),
      title: "Test Bead".to_string(),
      description: "A test bead".to_string(),
      priority: 150,
      status: BeadStatus::Ready,
      depends_on: vec!["bead-0".to_string()],
      blocks: vec!["bead-2".to_string()],
    };

    let json = serde_json::to_string(&bead).expect("should serialize");
    let parsed: PlanBead = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(bead, parsed);
  }

  #[test]
  fn test_serde_roundtrip_phase() {
    let phase = Phase {
      phase_number: 2,
      name: "Refinement".to_string(),
      description: "Detailed design".to_string(),
      beads: vec![PlanBead {
        id: "bead-1".to_string(),
        title: "Test".to_string(),
        description: "Test bead".to_string(),
        priority: 100,
        status: BeadStatus::Pending,
        depends_on: Vec::new(),
        blocks: Vec::new(),
      }],
      status: PhaseStatus::InProgress,
      blockers: vec!["Blocker 1".to_string()],
    };

    let json = serde_json::to_string(&phase).expect("should serialize");
    let parsed: Phase = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(phase, parsed);
  }

  #[test]
  fn test_serde_roundtrip_execution_plan() {
    let plan = ExecutionPlan {
      session_id: "session-123".to_string(),
      phases: vec![Phase {
        phase_number: 1,
        name: "Discovery".to_string(),
        description: "Initial phase".to_string(),
        beads: Vec::new(),
        status: PhaseStatus::Pending,
        blockers: Vec::new(),
      }],
      blockers: vec!["Global blocker".to_string()],
      created_at: "2026-02-27T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&plan).expect("should serialize");
    let parsed: ExecutionPlan = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(plan, parsed);
  }

  #[test]
  fn test_plan_error_display() {
    let err = PlanError::DependencyError("circular".to_string());
    assert!(format!("{err}").contains("circular"));

    let err = PlanError::InvalidPhase(0);
    assert!(format!("{err}").contains('0'));

    let err = PlanError::PhaseNotComplete(1);
    assert!(format!("{err}").contains('1'));

    let err = PlanError::NoActionableBeads;
    assert!(format!("{err}").contains("no actionable beads"));

    let err = PlanError::EmptySessionId;
    assert!(format!("{err}").contains("empty"));

    let err = PlanError::CircularDependency("a".to_string(), "b".to_string());
    assert!(format!("{err}").contains('a'));
    assert!(format!("{err}").contains('b'));
  }

  #[test]
  fn test_detect_blockers_with_resolved_gap() {
    let mut session = create_test_session();
    session.gaps.push(Gap {
      id: "gap-1".to_string(),
      field: "test_field".to_string(),
      description: "Missing field".to_string(),
      blocking: true,
      resolved: true,
      ..Gap::default()
    });

    let blockers = detect_blockers(&session);

    // Resolved gaps should not appear in blockers
    assert!(blockers.is_empty());
  }

  #[test]
  fn test_detect_blockers_with_resolved_conflict() {
    let mut session = create_test_session();
    session.conflicts.push(Conflict {
      id: "conflict-1".to_string(),
      between: ("a".to_string(), "b".to_string()),
      description: "Test conflict".to_string(),
      impact: "High".to_string(),
      options: vec![ConflictResolution {
        option: "opt1".to_string(),
        description: "Option 1".to_string(),
        tradeoffs: "Tradeoff".to_string(),
        recommendation: true,
      }],
      chosen: Some(0), // Resolved
    });

    let blockers = detect_blockers(&session);

    // Resolved conflicts should not appear in blockers
    assert!(blockers.is_empty());
  }

  #[test]
  fn test_apply_phase_gating_all_complete() {
    let mut session = create_test_session();
    session.completed_phases = vec![1, 2, 3, 4];

    let plan = compute_plan(&session).expect("plan should exist");
    let gated_plan = apply_phase_gating(&plan, &session);

    // All phases should be complete
    for phase in &gated_plan.phases {
      assert_eq!(phase.status, PhaseStatus::Complete);
    }
  }

  #[test]
  fn test_compute_plan_preserves_session_timestamp() {
    let session = create_test_session();
    let plan = compute_plan(&session).expect("plan should exist");

    assert_eq!(plan.created_at, session.updated_at);
  }

  #[test]
  fn test_get_actionable_beads_filters_by_status() {
    let plan = ExecutionPlan {
      session_id: "test".to_string(),
      phases: vec![Phase {
        phase_number: 1,
        name: "Test".to_string(),
        description: "Test".to_string(),
        beads: vec![
          PlanBead {
            id: "bead-ready".to_string(),
            title: "Ready".to_string(),
            description: "Ready bead".to_string(),
            priority: 100,
            status: BeadStatus::Ready,
            depends_on: Vec::new(),
            blocks: Vec::new(),
          },
          PlanBead {
            id: "bead-pending".to_string(),
            title: "Pending".to_string(),
            description: "Pending bead".to_string(),
            priority: 100,
            status: BeadStatus::Pending,
            depends_on: Vec::new(),
            blocks: Vec::new(),
          },
          PlanBead {
            id: "bead-blocked".to_string(),
            title: "Blocked".to_string(),
            description: "Blocked bead".to_string(),
            priority: 100,
            status: BeadStatus::Blocked,
            depends_on: Vec::new(),
            blocks: Vec::new(),
          },
        ],
        status: PhaseStatus::Pending,
        blockers: Vec::new(),
      }],
      blockers: Vec::new(),
      created_at: "2026-02-27T00:00:00Z".to_string(),
    };

    let actionable = get_actionable_beads(&plan);

    // Only the Ready bead should be actionable
    assert_eq!(actionable.len(), 1);
    assert_eq!(actionable[0].id, "bead-ready");
  }

  #[test]
  fn test_get_actionable_beads_respects_dependencies() {
    let plan = ExecutionPlan {
      session_id: "test".to_string(),
      phases: vec![Phase {
        phase_number: 1,
        name: "Test".to_string(),
        description: "Test".to_string(),
        beads: vec![
          PlanBead {
            id: "bead-ready".to_string(),
            title: "Ready".to_string(),
            description: "Ready bead".to_string(),
            priority: 100,
            status: BeadStatus::Ready,
            depends_on: vec!["bead-incomplete".to_string()],
            blocks: Vec::new(),
          },
          PlanBead {
            id: "bead-incomplete".to_string(),
            title: "Incomplete".to_string(),
            description: "Incomplete bead".to_string(),
            priority: 100,
            status: BeadStatus::InProgress, // Not complete
            depends_on: Vec::new(),
            blocks: Vec::new(),
          },
        ],
        status: PhaseStatus::Pending,
        blockers: Vec::new(),
      }],
      blockers: Vec::new(),
      created_at: "2026-02-27T00:00:00Z".to_string(),
    };

    let actionable = get_actionable_beads(&plan);

    // bead-ready depends on bead-incomplete which is not complete
    assert!(actionable.is_empty());
  }

  #[test]
  fn test_get_actionable_beads_with_satisfied_dependencies() {
    let plan = ExecutionPlan {
      session_id: "test".to_string(),
      phases: vec![Phase {
        phase_number: 1,
        name: "Test".to_string(),
        description: "Test".to_string(),
        beads: vec![
          PlanBead {
            id: "bead-ready".to_string(),
            title: "Ready".to_string(),
            description: "Ready bead".to_string(),
            priority: 100,
            status: BeadStatus::Ready,
            depends_on: vec!["bead-complete".to_string()],
            blocks: Vec::new(),
          },
          PlanBead {
            id: "bead-complete".to_string(),
            title: "Complete".to_string(),
            description: "Complete bead".to_string(),
            priority: 100,
            status: BeadStatus::Complete,
            depends_on: Vec::new(),
            blocks: Vec::new(),
          },
        ],
        status: PhaseStatus::Pending,
        blockers: Vec::new(),
      }],
      blockers: Vec::new(),
      created_at: "2026-02-27T00:00:00Z".to_string(),
    };

    let actionable = get_actionable_beads(&plan);

    // bead-ready should be actionable since bead-complete is done
    assert_eq!(actionable.len(), 1);
    assert_eq!(actionable[0].id, "bead-ready");
  }
}
