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

#[cfg(test)]
mod tests {
  use super::*;

  fn create_session_with_completed_phases(completed: Vec<u32>) -> InterviewSession {
    InterviewSession {
      completed_phases: completed,
      current_phase: 1,
      ..InterviewSession::default()
    }
  }

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
}
