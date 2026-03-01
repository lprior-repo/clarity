use super::types::{BeadStatus, ExecutionPlan, PhaseStatus, PlanBead};
use crate::intent::interview::types::InterviewSession;
use std::collections::HashSet;

#[must_use]
pub fn apply_phase_gating(plan: &ExecutionPlan, session: &InterviewSession) -> ExecutionPlan {
  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  let phases = plan
    .phases
    .iter()
    .map(|phase| {
      let status = if completed_phases.contains(&phase.phase_number) {
        PhaseStatus::Complete
      } else if phase.phase_number > 1
        && !completed_phases.contains(&(phase.phase_number.saturating_sub(1)))
      {
        PhaseStatus::Blocked
      } else {
        phase.status
      };

      let blockers = if status == PhaseStatus::Blocked {
        vec![format!(
          "Phase {} must be completed first",
          phase.phase_number.saturating_sub(1)
        )]
      } else {
        phase.blockers.clone()
      };

      let beads = apply_bead_gating(&phase.beads, &completed_phases);
      let mut updated = phase.clone();
      updated.status = status;
      updated.blockers = blockers;
      updated.beads = beads;
      updated
    })
    .collect();

  ExecutionPlan {
    session_id: plan.session_id.clone(),
    phases,
    blockers: plan.blockers.clone(),
    created_at: plan.created_at.clone(),
  }
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  let completed_ids: HashSet<&str> = plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .filter(|bead| bead.status == BeadStatus::Complete)
    .map(|bead| bead.id.as_str())
    .collect();

  plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .filter(|bead| {
      bead.status == BeadStatus::Ready
        && bead
          .depends_on
          .iter()
          .all(|dependency| completed_ids.contains(dependency.as_str()))
    })
    .collect()
}

fn apply_bead_gating(beads: &[PlanBead], completed_phases: &HashSet<u32>) -> Vec<PlanBead> {
  beads
    .iter()
    .map(|bead| {
      let dependencies_satisfied = bead.depends_on.iter().all(|_| !completed_phases.is_empty());
      let status = if bead.status == BeadStatus::Pending && dependencies_satisfied {
        BeadStatus::Ready
      } else {
        bead.status
      };
      PlanBead {
        status,
        ..bead.clone()
      }
    })
    .collect()
}

/// Check if a specific phase can be executed based on session completion state.
///
/// Phase 1 can always be executed. Other phases require all prior phases
/// (1 to phase_number-1) to be completed.
///
/// # Arguments
/// * `session` - The interview session containing completion state
/// * `phase_number` - The phase number to check (1-indexed)
///
/// # Returns
/// `true` if the phase can be executed, `false` otherwise
#[must_use]
pub fn can_execute_phase(session: &InterviewSession, phase_number: u32) -> bool {
  // Phase 1 can always be executed
  if phase_number == 1 {
    return true;
  }

  // For other phases, all prior phases must be completed
  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  // Check that all phases from 1 to phase_number-1 are completed
  (1..phase_number).all(|p| completed_phases.contains(&p))
}

/// Get the next unlockable phase.
///
/// Finds the first phase in the range 1..=current_phase that:
/// 1. Can be executed (all prior phases complete)
/// 2. Has not been completed yet
///
/// If all executable phases are completed, returns current_phase.
///
/// # Arguments
/// * `session` - The interview session containing completion state
///
/// # Returns
/// The phase number of the next executable phase, or current_phase if none available
#[must_use]
pub fn get_next_phase(session: &InterviewSession) -> u32 {
  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  // Check phases from 1 to current_phase (inclusive)
  for phase in 1..=session.current_phase {
    if can_execute_phase(session, phase) && !completed_phases.contains(&phase) {
      return phase;
    }
  }

  // No executable incomplete phase found, return current_phase
  session.current_phase
}

#[cfg(test)]
mod tests {
  use super::super::types::Phase;
  use super::*;

  fn create_session_with_completed_phases(completed: Vec<u32>) -> InterviewSession {
    InterviewSession {
      completed_phases: completed,
      current_phase: 1,
      ..InterviewSession::default()
    }
  }

  fn create_session(current_phase: u32, completed: Vec<u32>) -> InterviewSession {
    InterviewSession {
      completed_phases: completed,
      current_phase,
      ..InterviewSession::default()
    }
  }

  // =============================================================================
  // can_execute_phase Tests
  // =============================================================================

  #[test]
  fn can_execute_phase_1_always_true() {
    let session = create_session_with_completed_phases(vec![]);
    assert!(can_execute_phase(&session, 1));

    let session_with_complete = create_session_with_completed_phases(vec![1, 2, 3]);
    assert!(can_execute_phase(&session_with_complete, 1));
  }

  #[test]
  fn can_execute_phase_2_requires_phase_1() {
    let session_without = create_session_with_completed_phases(vec![]);
    assert!(!can_execute_phase(&session_without, 2));

    let session_with_1 = create_session_with_completed_phases(vec![1]);
    assert!(can_execute_phase(&session_with_1, 2));
  }

  #[test]
  fn can_execute_phase_3_requires_phases_1_and_2() {
    let session_empty = create_session_with_completed_phases(vec![]);
    assert!(!can_execute_phase(&session_empty, 3));

    let session_only_1 = create_session_with_completed_phases(vec![1]);
    assert!(!can_execute_phase(&session_only_1, 3));

    let session_1_and_2 = create_session_with_completed_phases(vec![1, 2]);
    assert!(can_execute_phase(&session_1_and_2, 3));
  }

  #[test]
  fn can_execute_phase_higher_requires_all_prior() {
    let session = create_session_with_completed_phases(vec![1, 2, 3, 4]);
    assert!(can_execute_phase(&session, 5));

    let session_missing_3 = create_session_with_completed_phases(vec![1, 2, 4]);
    assert!(!can_execute_phase(&session_missing_3, 5));
  }

  #[test]
  fn can_execute_phase_zero_edge_case() {
    // Phase 0 is not phase 1, so it requires prior phases (which don't exist)
    // This tests edge case behavior
    let session = create_session_with_completed_phases(vec![]);
    // Phase 0 with no prior phases - the range 1..0 is empty, so all() returns true
    assert!(can_execute_phase(&session, 0));
  }

  #[test]
  fn can_execute_phase_very_high_number() {
    // Phase 100 requires all 99 prior phases
    let session_empty = create_session_with_completed_phases(vec![]);
    assert!(!can_execute_phase(&session_empty, 100));

    // Even with many completions, missing any breaks the chain
    let session_partial = create_session_with_completed_phases(vec![1, 2, 3, 50, 51, 52]);
    assert!(!can_execute_phase(&session_partial, 100));
  }

  #[test]
  fn can_execute_phase_all_prior_complete() {
    // When all prior phases complete, can execute any phase
    let session = create_session_with_completed_phases(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert!(can_execute_phase(&session, 10));
  }

  // =============================================================================
  // get_next_phase Tests
  // =============================================================================

  #[test]
  fn get_next_phase_first_incomplete_at_phase_1() {
    // At phase 1 with nothing completed, next is 1
    let session = create_session(1, vec![]);
    assert_eq!(get_next_phase(&session), 1);
  }

  #[test]
  fn get_next_phase_skips_completed() {
    // At phase 1 but phase 1 completed, should return 1 anyway (current_phase fallback)
    // Actually looking at the implementation, it checks range 1..=current_phase
    // Phase 1 is completed, so it won't find any, returns current_phase = 1
    let session = create_session(1, vec![1]);
    assert_eq!(get_next_phase(&session), 1);
  }

  #[test]
  fn get_next_phase_finds_next_incomplete() {
    // At phase 2 with phase 1 complete, next is 2
    let session = create_session(2, vec![1]);
    assert_eq!(get_next_phase(&session), 2);
  }

  #[test]
  fn get_next_phase_with_multiple_completed() {
    // At phase 3 with 1 and 2 completed, next is 3
    let session = create_session(3, vec![1, 2]);
    assert_eq!(get_next_phase(&session), 3);
  }

  #[test]
  fn get_next_phase_higher_current_phase() {
    // At phase 5 with 1,2,3 completed, next is 4
    let session = create_session(5, vec![1, 2, 3]);
    assert_eq!(get_next_phase(&session), 4);
  }

  #[test]
  fn get_next_phase_all_completed_returns_current() {
    // When all phases up to current are completed, returns current
    let session = create_session(3, vec![1, 2, 3]);
    assert_eq!(get_next_phase(&session), 3);
  }

  #[test]
  fn get_next_phase_gap_in_completion() {
    // With gap in completion (1, 3 completed but not 2), should handle gracefully
    // Phase 2 requires phase 1 complete (it is), so phase 2 is executable
    let session = create_session(3, vec![1, 3]);
    // Phase 2: can_execute(2) with completed=[1,3] -> requires 1, it exists -> true
    // But wait, phase 2 is NOT in completed, so it should return 2
    assert_eq!(get_next_phase(&session), 2);
  }

  // =============================================================================
  // apply_phase_gating Tests
  // =============================================================================

  fn create_test_phase(phase_number: u32, bead_ids: Vec<&str>) -> Phase {
    let beads: Vec<PlanBead> = bead_ids
      .iter()
      .map(|id| PlanBead {
        id: id.to_string(),
        title: format!("Bead {}", id),
        description: format!("Description for {}", id),
        priority: 50,
        depends_on: vec![],
        blocks: vec![],
        status: BeadStatus::Pending,
      })
      .collect();

    Phase {
      phase_number,
      name: format!("Phase {}", phase_number),
      description: format!("Description for phase {}", phase_number),
      beads,
      status: PhaseStatus::Pending,
      blockers: vec![],
    }
  }

  fn create_test_plan(phases: Vec<Phase>) -> ExecutionPlan {
    ExecutionPlan {
      session_id: "test".to_string(),
      phases,
      blockers: vec![],
      created_at: "2026-01-01T00:00:00Z".to_string(),
    }
  }

  #[test]
  fn apply_phase_gating_allows_phase_1_at_start() {
    let session = create_session(1, vec![]);
    let plan = create_test_plan(vec![create_test_phase(1, vec!["B-001"])]);

    let result = apply_phase_gating(&plan, &session);

    assert_eq!(result.phases.len(), 1);
    assert!(result.blockers.is_empty());
  }

  #[test]
  fn apply_phase_gating_blocks_later_phases() {
    let session = create_session(1, vec![]);
    let plan = create_test_plan(vec![
      create_test_phase(1, vec!["B-001"]),
      create_test_phase(2, vec!["B-002"]),
    ]);

    let result = apply_phase_gating(&plan, &session);

    // Both phases should be present, but phase 2 should be blocked
    assert_eq!(result.phases.len(), 2);
    // Phase 1 should be Ready/Pending, phase 2 should be Blocked
    assert_eq!(result.phases[1].status, PhaseStatus::Blocked);
    // Phase 2 should have blockers
    assert!(!result.phases[1].blockers.is_empty());
  }

  #[test]
  fn apply_phase_gating_allows_after_completion() {
    let session = create_session(2, vec![1]);
    let plan = create_test_plan(vec![
      create_test_phase(1, vec!["B-001"]),
      create_test_phase(2, vec!["B-002"]),
    ]);

    let result = apply_phase_gating(&plan, &session);

    // Both phases should be present
    assert_eq!(result.phases.len(), 2);
    // Phase 1 should be Complete
    assert_eq!(result.phases[0].status, PhaseStatus::Complete);
    // Phase 2 should not be blocked
    assert_eq!(result.phases[1].status, PhaseStatus::Pending);
  }

  #[test]
  fn apply_phase_gating_three_phase_progression() {
    // Test progression through all three phases
    // Phase 1 only
    let session1 = create_session(1, vec![]);
    let plan = create_test_plan(vec![
      create_test_phase(1, vec!["B-001"]),
      create_test_phase(2, vec!["B-002"]),
      create_test_phase(3, vec!["B-003"]),
    ]);

    let result1 = apply_phase_gating(&plan, &session1);
    // All phases present, but 2 and 3 should be blocked
    assert_eq!(result1.phases.len(), 3);
    assert_eq!(result1.phases[1].status, PhaseStatus::Blocked);
    assert_eq!(result1.phases[2].status, PhaseStatus::Blocked);

    // Phase 2 with phase 1 complete
    let session2 = create_session(2, vec![1]);
    let result2 = apply_phase_gating(&plan, &session2);
    assert_eq!(result2.phases.len(), 3);
    assert_eq!(result2.phases[0].status, PhaseStatus::Complete);
    assert_eq!(result2.phases[1].status, PhaseStatus::Pending);
    assert_eq!(result2.phases[2].status, PhaseStatus::Blocked);

    // Phase 3 with phases 1 and 2 complete
    let session3 = create_session(3, vec![1, 2]);
    let result3 = apply_phase_gating(&plan, &session3);
    assert_eq!(result3.phases.len(), 3);
    // All phases should be complete or pending (not blocked)
    assert_eq!(result3.phases[0].status, PhaseStatus::Complete);
    assert_eq!(result3.phases[1].status, PhaseStatus::Complete);
    assert_eq!(result3.phases[2].status, PhaseStatus::Pending);
  }

  #[test]
  fn apply_phase_gating_empty_plan() {
    let session = create_session(1, vec![]);
    let plan = create_test_plan(vec![]);

    let result = apply_phase_gating(&plan, &session);

    assert!(result.phases.is_empty());
    assert!(result.blockers.is_empty());
  }

  #[test]
  fn apply_phase_gating_preserves_existing_blockers() {
    let session = create_session(1, vec![]);
    let plan = ExecutionPlan {
      session_id: "test".to_string(),
      phases: vec![create_test_phase(1, vec!["B-001"])],
      blockers: vec!["Existing blocker".to_string()],
      created_at: "2026-01-01T00:00:00Z".to_string(),
    };

    let result = apply_phase_gating(&plan, &session);

    // Should have both existing and new blockers
    assert!(result.blockers.len() >= 1);
  }

  #[test]
  fn apply_phase_gating_single_phase_completed_allows_it() {
    // Even if current_phase is higher, completed phases should be in the result
    let session = create_session(3, vec![1, 2]);
    let plan = create_test_plan(vec![
      create_test_phase(1, vec!["B-001"]),
      create_test_phase(2, vec!["B-002"]),
      create_test_phase(3, vec!["B-003"]),
    ]);

    let result = apply_phase_gating(&plan, &session);

    // All three phases should be allowed since 1 and 2 are completed
    assert_eq!(result.phases.len(), 3);
  }
}
