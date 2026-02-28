use crate::intent::interview::types::{Answer, InterviewSession, InterviewStage};
use crate::intent::plan::plan_emit_beads::{check_existing_beads, EmissionMode, EmissionResult};
use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError, PlanPhase};
use std::collections::{HashMap, HashSet};

pub fn emit_beads(
  session: &InterviewSession,
  plan: &mut ExecutionPlan,
  mode: EmissionMode,
) -> Result<(Vec<PlanBead>, EmissionResult), PlanError> {
  validate_session(session)?;

  let existing_titles: Vec<String> = plan.beads.iter().map(|bead| bead.title.clone()).collect();
  let mut result = EmissionResult::new();

  let from_answers = generate_beads_from_answers(session, &existing_titles, &mut result);
  let from_gaps = generate_beads_from_gaps(session, &existing_titles, &mut result);
  let from_conflicts = generate_beads_from_conflicts(session, &existing_titles, &mut result);

  let emitted_beads = from_answers
    .into_iter()
    .chain(from_gaps)
    .chain(from_conflicts)
    .collect::<Vec<_>>();

  if mode.should_persist() {
    emitted_beads
      .iter()
      .for_each(|bead| match plan.add_bead(bead.clone()) {
        Ok(()) => {}
        Err(PlanError::DuplicateBeadId(_)) => result.add_skipped(1),
        Err(error) => result.add_error(format!("Failed to add bead '{}': {}", bead.id, error)),
      });
    update_plan_phases(plan);
  }

  result.emitted = emitted_beads.len();
  Ok((emitted_beads, result))
}

fn validate_session(session: &InterviewSession) -> Result<(), PlanError> {
  if session.id.trim().is_empty() {
    return Err(PlanError::EmptySessionId);
  }
  if session.stage == InterviewStage::Discovery {
    return Err(PlanError::InvalidPhaseNumber { phase_number: 0 });
  }
  Ok(())
}

fn generate_beads_from_answers(
  session: &InterviewSession,
  existing_titles: &[String],
  result: &mut EmissionResult,
) -> Vec<PlanBead> {
  group_answers_by_phase(session)
    .into_iter()
    .flat_map(|(phase, answers)| {
      let titles = answers
        .iter()
        .map(|answer| format!("Implement: {}", answer.question_text))
        .collect::<Vec<_>>();

      let new_titles = check_existing_beads(&titles, existing_titles);
      result.add_skipped(titles.len().saturating_sub(new_titles.len()));

      answers
        .into_iter()
        .filter_map(|answer| {
          let title = format!("Implement: {}", answer.question_text);
          if new_titles.contains(&title) {
            create_bead_from_answer(answer, phase).ok()
          } else {
            None
          }
        })
        .collect::<Vec<_>>()
    })
    .collect()
}

fn generate_beads_from_gaps(
  session: &InterviewSession,
  existing_titles: &[String],
  result: &mut EmissionResult,
) -> Vec<PlanBead> {
  session
    .gaps
    .iter()
    .filter(|gap| !gap.is_resolved())
    .filter_map(|gap| {
      let title = format!("Address gap: {}", gap.field);
      if check_existing_beads(std::slice::from_ref(&title), existing_titles).is_empty() {
        result.add_skipped(1);
        return None;
      }

      PlanBead::new(format!("gap-{}", gap.id), title, gap.round)
        .map(|bead| {
          bead
            .with_description(format!("Resolve gap: {} - {}", gap.field, gap.description))
            .with_effort(if gap.blocking { 3 } else { 1 })
            .with_tag("gap".to_string())
            .with_tag(if gap.blocking {
              "blocking".to_string()
            } else {
              "optional".to_string()
            })
        })
        .map_or_else(
          |error| {
            result.add_error(format!("Failed to create gap bead: {error}"));
            None
          },
          Some,
        )
    })
    .collect()
}

fn generate_beads_from_conflicts(
  session: &InterviewSession,
  existing_titles: &[String],
  result: &mut EmissionResult,
) -> Vec<PlanBead> {
  session
    .conflicts
    .iter()
    .filter(|conflict| !conflict.is_resolved())
    .filter_map(|conflict| {
      let title = format!(
        "Resolve conflict: {} vs {}",
        conflict.between.0, conflict.between.1
      );
      if check_existing_beads(std::slice::from_ref(&title), existing_titles).is_empty() {
        result.add_skipped(1);
        return None;
      }

      PlanBead::new(format!("conflict-{}", conflict.id), title, 1)
        .map(|bead| {
          bead
            .with_description(format!(
              "Resolve conflict between '{}' and '{}': {}",
              conflict.between.0, conflict.between.1, conflict.description
            ))
            .with_effort(2)
            .with_tag("conflict".to_string())
            .with_priority(1)
        })
        .map_or_else(
          |error| {
            result.add_error(format!("Failed to create conflict bead: {error}"));
            None
          },
          Some,
        )
    })
    .collect()
}

fn create_bead_from_answer(answer: &Answer, phase: u32) -> Result<PlanBead, PlanError> {
  PlanBead::new(
    format!("answer-{}-{}", answer.round, answer.question_id),
    format!("Implement: {}", answer.question_text),
    phase,
  )
  .map(|bead| {
    bead
      .with_description(format!(
        "Implementation task from answer: {}",
        answer.response
      ))
      .with_effort(estimate_effort_from_confidence(answer.confidence))
      .with_tag(format!("round-{}", answer.round))
  })
}

fn estimate_effort_from_confidence(confidence: f64) -> u32 {
  if confidence >= 0.9 {
    1
  } else if confidence >= 0.7 {
    2
  } else if confidence >= 0.5 {
    3
  } else {
    5
  }
}

fn group_answers_by_phase(session: &InterviewSession) -> Vec<(u32, Vec<&Answer>)> {
  let phase_map = session.answers.iter().fold(
    HashMap::<u32, Vec<&Answer>>::new(),
    |mut grouped, answer| {
      let phase = match answer.round {
        1 | 2 => 1,
        3 => 2,
        4 => 3,
        _ => answer.round.saturating_sub(1),
      };
      grouped.entry(phase).or_default().push(answer);
      grouped
    },
  );

  let mut phases = phase_map.into_iter().collect::<Vec<_>>();
  phases.sort_by_key(|(phase, _)| *phase);
  phases
}

fn update_plan_phases(plan: &mut ExecutionPlan) {
  let phase_numbers = plan
    .beads
    .iter()
    .map(|bead| bead.phase)
    .collect::<HashSet<_>>();

  let mut sorted = phase_numbers.into_iter().collect::<Vec<_>>();
  sorted.sort_unstable();

  plan.phases = sorted
    .into_iter()
    .map(|phase_number| {
      let bead_ids = plan
        .beads
        .iter()
        .filter(|bead| bead.phase == phase_number)
        .map(|bead| bead.id.clone())
        .collect::<Vec<_>>();

      let mut phase = PlanPhase::new(phase_number, format!("Phase {phase_number}"));
      bead_ids
        .into_iter()
        .for_each(|bead_id| phase.add_bead(bead_id));
      phase
    })
    .collect();
}
