use crate::intent::interview::types::InterviewSession;
use super::types::{
  BeadStatus, ExecutionPlan, Phase, PhaseStatus, PlanBead, PlanError,
};
use std::collections::HashSet;

pub fn compute_plan(session: &InterviewSession) -> Result<ExecutionPlan, PlanError> {
  if session.id.is_empty() {
    return Err(PlanError::EmptySessionId);
  }

  let beads = generate_beads_from_session(session);
  let phases = organize_into_phases(&beads, session);
  let blockers = detect_blockers(session);
  let phases_with_status = apply_initial_phase_status(phases, &blockers);

  Ok(ExecutionPlan {
    session_id: session.id.clone(),
    phases: phases_with_status,
    blockers,
    created_at: session.updated_at.clone(),
  })
}

fn generate_beads_from_session(session: &InterviewSession) -> Vec<PlanBead> {
  let required_fields = session.profile.required_fields();
  let answered_fields: HashSet<&str> = session
    .answers
    .iter()
    .flat_map(|answer| answer.extracted.keys())
    .map(String::as_str)
    .collect();

  let field_beads = required_fields
    .iter()
    .map(|field| {
      let is_answered = answered_fields.contains(*field);
      PlanBead {
        id: format!("bead-{}-{}", session.id, field),
        title: format!("Define {}", field.replace('_', " ")),
        description: if is_answered {
          format!("Review and validate the {} specification", field)
        } else {
          format!("Define the {} for the system", field)
        },
        priority: if is_answered { 50 } else { 200 },
        status: if is_answered {
          BeadStatus::Ready
        } else {
          BeadStatus::Pending
        },
        depends_on: Vec::new(),
        blocks: Vec::new(),
      }
    })
    .collect::<Vec<_>>();

  let gap_beads = session
    .gaps
    .iter()
    .filter(|gap| !gap.resolved)
    .map(|gap| PlanBead {
      id: format!("bead-gap-{}", gap.id),
      title: format!("Resolve gap: {}", gap.field),
      description: gap.description.clone(),
      priority: if gap.blocking { 255 } else { 100 },
      status: BeadStatus::Blocked,
      depends_on: Vec::new(),
      blocks: Vec::new(),
    });

  let conflict_beads = session
    .conflicts
    .iter()
    .filter(|conflict| conflict.chosen.is_none())
    .map(|conflict| PlanBead {
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

  field_beads
    .into_iter()
    .chain(gap_beads)
    .chain(conflict_beads)
    .collect()
}

fn organize_into_phases(beads: &[PlanBead], session: &InterviewSession) -> Vec<Phase> {
  let phase_names = [
    (
      1_u32,
      "Discovery",
      "Initial exploration and requirements gathering",
    ),
    (2_u32, "Refinement", "Detailed specification and design"),
    (3_u32, "Validation", "Verification and quality assurance"),
    (4_u32, "Implementation", "Development and execution"),
  ];
  let completed_phases: HashSet<u32> = session.completed_phases.iter().copied().collect();

  phase_names
    .iter()
    .map(|(number, name, description)| Phase {
      phase_number: *number,
      name: (*name).to_string(),
      description: (*description).to_string(),
      beads: assign_beads_to_phase(beads, *number),
      status: if completed_phases.contains(number) {
        PhaseStatus::Complete
      } else {
        PhaseStatus::Pending
      },
      blockers: Vec::new(),
    })
    .collect()
}

fn assign_beads_to_phase(beads: &[PlanBead], phase_number: u32) -> Vec<PlanBead> {
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

fn detect_blockers(session: &InterviewSession) -> Vec<String> {
  let blocking_gaps = session.get_blocking_gaps();
  let unresolved_gaps = blocking_gaps
    .iter()
    .map(|gap| format!("Unresolved gap: {} - {}", gap.field, gap.description));

  let unresolved_conflicts = session
    .conflicts
    .iter()
    .filter(|conflict| conflict.chosen.is_none())
    .map(|conflict| {
      format!(
        "Unresolved conflict: {} vs {} - {}",
        conflict.between.0, conflict.between.1, conflict.description
      )
    });

  unresolved_gaps.chain(unresolved_conflicts).collect()
}

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
