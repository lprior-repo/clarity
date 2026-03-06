#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::plan::plan_mode::types::{
  BeadStatus, ExecutionPlan, Phase, PhaseStatus, PlanBead,
};
use std::fmt::Write as _;

#[must_use]
pub fn format_plan_human(plan: &ExecutionPlan) -> String {
  let mut output = String::new();
  let _ = writeln!(output, "Execution Plan: {}", plan.session_id);
  let _ = writeln!(output, "Created: {}", plan.created_at);

  if !plan.blockers.is_empty() {
    output.push_str("\nBlockers:\n");
    for blocker in &plan.blockers {
      let _ = writeln!(output, "  - {blocker}");
    }
  }

  output.push_str("\nPhases:\n");
  for phase in &plan.phases {
    output.push_str(&format_phase_human(phase));
  }

  output
}

fn format_phase_human(phase: &Phase) -> String {
  let mut output = String::new();
  let _ = writeln!(
    output,
    "\n  Phase {}: {} [{}]",
    phase.phase_number,
    phase.name,
    phase_status_to_string(phase.status)
  );

  if !phase.description.is_empty() {
    let _ = writeln!(output, "    {}", phase.description);
  }

  if !phase.blockers.is_empty() {
    output.push_str("    Blockers:\n");
    for blocker in &phase.blockers {
      let _ = writeln!(output, "      - {blocker}");
    }
  }

  output.push_str("    Beads:\n");
  for bead in &phase.beads {
    output.push_str(&format_bead_human(bead));
  }

  output
}

fn format_bead_human(bead: &PlanBead) -> String {
  let status_str = bead_status_to_string(bead.status);
  let priority_label = effort_to_label(bead.priority);

  let mut output = String::new();
  let _ = writeln!(
    output,
    "      - [{}] {} ({})",
    status_str, bead.id, priority_label
  );

  if !bead.title.is_empty() {
    let _ = writeln!(output, "        {}", bead.title);
  }

  if !bead.depends_on.is_empty() {
    let _ = writeln!(output, "        Depends on: {}", bead.depends_on.join(", "));
  }

  output
}

#[must_use]
pub fn format_plan_json(plan: &ExecutionPlan) -> String {
  serde_json::to_string_pretty(plan).unwrap_or_else(|_| "{}".to_string())
}

#[must_use]
pub fn format_plan_ai(plan: &ExecutionPlan) -> String {
  let mut output = String::new();

  output.push_str("# Execution Plan\n\n");
  let _ = writeln!(output, "Session: {}", plan.session_id);
  let _ = writeln!(output, "Created: {}\n", plan.created_at);

  for phase in &plan.phases {
    output.push_str(&format_phase_ai(phase));
  }

  output
}

fn format_phase_ai(phase: &Phase) -> String {
  let mut output = String::new();

  let _ = writeln!(
    output,
    "## Phase {}: {} ({:?})",
    phase.phase_number, phase.name, phase.status
  );

  if !phase.description.is_empty() {
    let _ = writeln!(output, "Description: {}", phase.description);
  }

  if !phase.blockers.is_empty() {
    let _ = writeln!(output, "Blockers: {:?}", phase.blockers);
  }

  output.push_str("\nBeads:\n");
  for bead in &phase.beads {
    output.push_str(&format_bead_ai(bead));
  }

  output.push('\n');
  output
}

fn format_bead_ai(bead: &PlanBead) -> String {
  format!(
    "- {}: {:?} (priority: {}, depends: {:?}, blocks: {:?})\n  {}\n",
    bead.id, bead.status, bead.priority, bead.depends_on, bead.blocks, bead.title
  )
}

#[must_use]
pub fn bead_status_to_string(status: BeadStatus) -> String {
  match status {
    BeadStatus::Pending => "PENDING".to_string(),
    BeadStatus::Ready => "READY".to_string(),
    BeadStatus::InProgress => "IN_PROGRESS".to_string(),
    BeadStatus::Complete => "COMPLETE".to_string(),
    BeadStatus::Blocked => "BLOCKED".to_string(),
  }
}

#[must_use]
pub fn phase_status_to_string(status: PhaseStatus) -> String {
  match status {
    PhaseStatus::Pending => "PENDING".to_string(),
    PhaseStatus::InProgress => "IN_PROGRESS".to_string(),
    PhaseStatus::Complete => "COMPLETE".to_string(),
    PhaseStatus::Blocked => "BLOCKED".to_string(),
  }
}

#[must_use]
pub fn effort_to_label(priority: u8) -> String {
  match priority {
    0 => "CRITICAL".to_string(),
    1 => "HIGH".to_string(),
    2 => "MEDIUM".to_string(),
    3 => "LOW".to_string(),
    _ => "BACKLOG".to_string(),
  }
}

#[must_use]
pub fn risk_to_string(risk: f64) -> String {
  if risk >= 0.8 {
    "HIGH".to_string()
  } else if risk >= 0.5 {
    "MEDIUM".to_string()
  } else {
    "LOW".to_string()
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;
  use crate::intent::plan::plan_mode::types::{BeadStatus, PhaseStatus};

  #[test]
  fn test_bead_status_to_string() {
    assert_eq!(bead_status_to_string(BeadStatus::Pending), "PENDING");
    assert_eq!(bead_status_to_string(BeadStatus::Ready), "READY");
    assert_eq!(bead_status_to_string(BeadStatus::InProgress), "IN_PROGRESS");
    assert_eq!(bead_status_to_string(BeadStatus::Complete), "COMPLETE");
    assert_eq!(bead_status_to_string(BeadStatus::Blocked), "BLOCKED");
  }

  #[test]
  fn test_phase_status_to_string() {
    assert_eq!(phase_status_to_string(PhaseStatus::Pending), "PENDING");
    assert_eq!(
      phase_status_to_string(PhaseStatus::InProgress),
      "IN_PROGRESS"
    );
    assert_eq!(phase_status_to_string(PhaseStatus::Complete), "COMPLETE");
    assert_eq!(phase_status_to_string(PhaseStatus::Blocked), "BLOCKED");
  }

  #[test]
  fn test_effort_to_label() {
    assert_eq!(effort_to_label(0), "CRITICAL");
    assert_eq!(effort_to_label(1), "HIGH");
    assert_eq!(effort_to_label(2), "MEDIUM");
    assert_eq!(effort_to_label(3), "LOW");
    assert_eq!(effort_to_label(10), "BACKLOG");
  }

  #[test]
  fn test_risk_to_string() {
    assert_eq!(risk_to_string(0.9), "HIGH");
    assert_eq!(risk_to_string(0.5), "MEDIUM");
    assert_eq!(risk_to_string(0.3), "LOW");
  }

  #[test]
  fn test_format_plan_json() {
    let plan = ExecutionPlan {
      session_id: "test-session".to_string(),
      phases: vec![],
      blockers: vec![],
      created_at: "2024-01-01".to_string(),
    };

    let json = format_plan_json(&plan);
    assert!(json.contains("test-session"));
  }

  #[test]
  fn test_format_plan_human() {
    let plan = ExecutionPlan {
      session_id: "test-session".to_string(),
      phases: vec![],
      blockers: vec![],
      created_at: "2024-01-01".to_string(),
    };

    let human = format_plan_human(&plan);
    assert!(human.contains("test-session"));
    assert!(human.contains("Phases:"));
  }

  #[test]
  fn test_format_plan_ai() {
    let plan = ExecutionPlan {
      session_id: "test-session".to_string(),
      phases: vec![],
      blockers: vec![],
      created_at: "2024-01-01".to_string(),
    };

    let ai = format_plan_ai(&plan);
    assert!(ai.contains("test-session"));
    assert!(ai.contains("# Execution Plan"));
  }
}
