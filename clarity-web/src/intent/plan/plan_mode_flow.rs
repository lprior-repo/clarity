use crate::intent::interview::types::InterviewSession;
use crate::intent::plan::plan_mode_types::{BeadStatus, ExecutionPlan, PhaseStatus, PlanBead};
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
