use crate::intent::interview::types::{InterviewSession, InterviewStage};
use crate::intent::plan::plan_next::{ActionType, NextAction};
use crate::intent::plan::types::{ExecutionPlan, PlanBead};
use std::collections::HashSet;

#[must_use]
pub fn get_next_action(session: &InterviewSession, plan: &ExecutionPlan) -> Option<NextAction> {
  if session.stage == InterviewStage::Complete {
    return None;
  }

  if session.stage == InterviewStage::Paused {
    return Some(NextAction::new(
      ActionType::AnswerQuestion,
      "resume".to_string(),
      "Resume the paused interview session".to_string(),
      "Session is currently paused".to_string(),
    ));
  }

  get_blocking_gap_action(session)
    .or_else(|| get_conflict_action(session))
    .or_else(|| get_question_action(session))
    .or_else(|| get_phase_completion_action(session, plan))
    .or_else(|| {
      (session.stage == InterviewStage::Validation).then_some(NextAction::new(
        ActionType::ReviewPlan,
        "plan".to_string(),
        "Review and approve the execution plan".to_string(),
        "Session is in validation stage".to_string(),
      ))
    })
}

fn get_blocking_gap_action(session: &InterviewSession) -> Option<NextAction> {
  session
    .gaps
    .iter()
    .filter(|gap| gap.blocking && !gap.resolved)
    .min_by_key(|gap| gap.round)
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

fn get_conflict_action(session: &InterviewSession) -> Option<NextAction> {
  session
    .conflicts
    .iter()
    .find(|conflict| conflict.chosen.is_none())
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
}

fn get_question_action(session: &InterviewSession) -> Option<NextAction> {
  if session.stage == InterviewStage::Validation {
    return None;
  }

  let current_round = session.get_current_round();
  let current_round_answers: HashSet<&str> = session
    .answers
    .iter()
    .filter(|answer| answer.round == current_round)
    .map(|answer| answer.question_id.as_str())
    .collect();

  if !current_round_answers.is_empty() && session.can_proceed().is_ok() {
    return Some(
      NextAction::new(
        ActionType::AnswerQuestion,
        format!("round-{}-complete", current_round),
        format!("Complete round {} or add more answers", current_round),
        "You have answers ready; you can complete the round or add more details".to_string(),
      )
      .with_priority(3),
    );
  }

  if current_round_answers.is_empty() {
    return Some(
      NextAction::new(
        ActionType::AnswerQuestion,
        format!("round-{}-start", current_round),
        format!("Start answering questions for round {}", current_round),
        format!("Round {} has not started yet", current_round),
      )
      .with_priority(3),
    );
  }

  None
}

fn get_phase_completion_action(
  session: &InterviewSession,
  plan: &ExecutionPlan,
) -> Option<NextAction> {
  if session.stage == InterviewStage::Validation {
    return None;
  }

  let current_phase = session.current_phase;
  let phase_beads = plan.get_phase_beads(current_phase);

  if !phase_beads.is_empty() {
    let completed_count = phase_beads.iter().filter(|bead| bead.completed).count();
    if completed_count == phase_beads.len() {
      return Some(
        NextAction::new(
          ActionType::CompletePhase,
          format!("phase-{}", current_phase),
          format!("Complete phase {} (all beads done)", current_phase),
          format!(
            "All {} work items in phase {} are complete",
            phase_beads.len(),
            current_phase
          ),
        )
        .with_priority(4),
      );
    }

    let actionable = plan.get_actionable_beads();
    let first_in_phase = actionable
      .iter()
      .find(|bead| bead.phase == current_phase)
      .map(|bead| *bead);

    if let Some(bead) = first_in_phase {
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

  (session.can_proceed().is_ok() && !session.completed_phases.contains(&current_phase)).then_some(
    NextAction::new(
      ActionType::CompletePhase,
      format!("phase-{}", current_phase),
      format!("Complete phase {}", current_phase),
      "Phase requirements are satisfied".to_string(),
    )
    .with_priority(4),
  )
}

#[must_use]
pub fn determine_next_phase(plan: &ExecutionPlan) -> Option<u32> {
  (!plan.beads.is_empty())
    .then(|| {
      let mut phases = plan
        .beads
        .iter()
        .filter(|bead| !bead.completed)
        .map(|bead| bead.phase)
        .collect::<Vec<_>>();
      phases.sort_unstable();
      phases.dedup();
      phases.into_iter().next()
    })
    .flatten()
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  let mut actionable = plan.get_actionable_beads();
  actionable.sort_by_key(|bead| (bead.phase, bead.priority));
  actionable
}
