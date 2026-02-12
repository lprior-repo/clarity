//! Mental lattice audit summary widget for PME Discover.

#![allow(clippy::disallowed_methods)]

use crate::pme::state::PmeDiscoverSignals;
use crate::shared::mental_lattice::{
  ConflictReport, ConflictSeverity, Constraint, ContractClause, ContractLayer, ContractReport,
  ContractSeverity, DesignSignal, DimensionScore, EQIAssessment, GapReport, InterviewMatrix,
  InterviewPerspective, QualityDimension,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LatticeAuditSummary {
  contract_ok: bool,
  quality_weak_dimensions: usize,
  interview_unanswered: usize,
  gap_count: usize,
  critical_conflicts: usize,
}

#[component]
pub fn LatticeAuditSummaryCard(signals: Signal<PmeDiscoverSignals>) -> Element {
  let snapshot = signals.read();
  let summary = evaluate_lattice(&snapshot);

  rsx! {
    div { class: "lattice-audit-summary",
      div { class: "summary-stat",
        span { class: "stat-value", if summary.contract_ok { "OK" } else { "NO" } }
        span { class: "stat-label", "Contract" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{summary.quality_weak_dimensions}" }
        span { class: "stat-label", "Weak EQI" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{summary.interview_unanswered}" }
        span { class: "stat-label", "Interview Gaps" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{summary.gap_count}" }
        span { class: "stat-label", "Security/Product Gaps" }
      }
      div { class: "summary-stat",
        span { class: "stat-value", "{summary.critical_conflicts}" }
        span { class: "stat-label", "Critical Conflicts" }
      }
    }
  }
}

fn evaluate_lattice(signals: &PmeDiscoverSignals) -> LatticeAuditSummary {
  let hypotheses_count = signals.hypothesis_count();
  let interviews_count = signals.interview_count();
  let blocking_holes = signals.blocking_plot_hole_count();

  let contract_report = build_contract_report(hypotheses_count, blocking_holes);
  let quality_assessment =
    build_quality_assessment(hypotheses_count, interviews_count, blocking_holes);
  let interview_matrix = build_interview_matrix(hypotheses_count, interviews_count);
  let gap_report = build_gap_report(hypotheses_count, interviews_count, blocking_holes);
  let conflict_report = build_conflict_report(hypotheses_count, interviews_count, blocking_holes);

  let interview_unanswered = interview_matrix.as_ref().map_or(0, |matrix| {
    ((1.0 - matrix.completion_ratio()) * 25.0).round() as usize
  });

  let critical_conflicts = conflict_report.as_ref().map_or(0, |report| {
    report
      .conflicts
      .iter()
      .filter(|conflict| conflict.severity == ConflictSeverity::Critical)
      .count()
  });

  LatticeAuditSummary {
    contract_ok: contract_report
      .as_ref()
      .is_some_and(|report| report.validate().is_ok()),
    quality_weak_dimensions: quality_assessment
      .as_ref()
      .map_or(0, |assessment| assessment.weak_dimensions().len()),
    interview_unanswered,
    gap_count: gap_report.as_ref().map_or(0, |report| report.gaps.len()),
    critical_conflicts,
  }
}

fn build_contract_report(hypotheses_count: usize, blocking_holes: usize) -> Option<ContractReport> {
  let report = ContractReport::new("discover-phase".to_string()).ok()?;

  let preconditions = ContractClause::new(
    ContractLayer::Precondition,
    "At least one hypothesis is formulated".to_string(),
    ContractSeverity::Critical,
    hypotheses_count > 0,
  )
  .ok()?;

  let postconditions = ContractClause::new(
    ContractLayer::Postcondition,
    "No blocking plot holes remain".to_string(),
    ContractSeverity::Major,
    blocking_holes == 0,
  )
  .ok()?;

  let invariants = ContractClause::new(
    ContractLayer::Invariant,
    "Discovery state remains measurable".to_string(),
    ContractSeverity::Major,
    true,
  )
  .ok()?;

  Some(
    report
      .with_clause(preconditions.with_confidence(if hypotheses_count > 0 { 0.9 } else { 0.2 }))
      .with_clause(postconditions.with_confidence(if blocking_holes == 0 { 0.9 } else { 0.3 }))
      .with_clause(invariants.with_confidence(0.9)),
  )
}

fn build_quality_assessment(
  hypotheses_count: usize,
  interviews_count: usize,
  blocking_holes: usize,
) -> Option<EQIAssessment> {
  let base = EQIAssessment::new("discover-phase".to_string()).ok()?;

  let completeness = DimensionScore::new(
    QualityDimension::Completeness,
    if hypotheses_count > 0 { 0.8 } else { 0.2 },
    "Hypothesis coverage in current discover state".to_string(),
  )
  .ok()?;
  let consistency = DimensionScore::new(
    QualityDimension::Consistency,
    0.7,
    "State transitions are consistently represented".to_string(),
  )
  .ok()?;
  let testability = DimensionScore::new(
    QualityDimension::Testability,
    if interviews_count >= 2 { 0.8 } else { 0.5 },
    "Interview and hypothesis metrics are observable".to_string(),
  )
  .ok()?;
  let clarity = DimensionScore::new(
    QualityDimension::Clarity,
    0.75,
    "Discover panel exposes explicit progress signals".to_string(),
  )
  .ok()?;
  let security = DimensionScore::new(
    QualityDimension::Security,
    if blocking_holes == 0 { 0.8 } else { 0.4 },
    "Blocking plot holes are treated as hard risk".to_string(),
  )
  .ok()?;

  Some(
    base
      .with_score(completeness)
      .with_score(consistency)
      .with_score(testability)
      .with_score(clarity)
      .with_score(security),
  )
}

fn build_interview_matrix(
  hypotheses_count: usize,
  interviews_count: usize,
) -> Option<InterviewMatrix> {
  let matrix = InterviewMatrix::new("discover-phase".to_string()).ok()?;

  let answered_per_perspective = if hypotheses_count > 0 && interviews_count > 0 {
    2
  } else if interviews_count > 0 {
    1
  } else {
    0
  };

  InterviewPerspective::all()
    .iter()
    .copied()
    .try_fold(matrix, |current, perspective| {
      (0..answered_per_perspective).try_fold(current, |acc, idx| {
        acc.answer_question(
          perspective,
          idx,
          format!("Evidence for {perspective} question {idx}"),
          true,
        )
      })
    })
    .ok()
}

fn build_gap_report(
  hypotheses_count: usize,
  interviews_count: usize,
  blocking_holes: usize,
) -> Option<GapReport> {
  let signals = [
    DesignSignal::new("authz".to_string(), true).ok()?,
    DesignSignal::new("input_validation".to_string(), interviews_count > 0).ok()?,
    DesignSignal::new("secure_defaults".to_string(), true).ok()?,
    DesignSignal::new("dependency_review".to_string(), hypotheses_count > 0).ok()?,
    DesignSignal::new("user_outcome".to_string(), hypotheses_count > 0).ok()?,
    DesignSignal::new("failure_modes".to_string(), blocking_holes == 0).ok()?,
    DesignSignal::new("dependency_map".to_string(), true).ok()?,
    DesignSignal::new("value_metric".to_string(), interviews_count > 0).ok()?,
  ];

  GapReport::detect("discover-phase".to_string(), &signals).ok()
}

fn build_conflict_report(
  hypotheses_count: usize,
  interviews_count: usize,
  blocking_holes: usize,
) -> Option<ConflictReport> {
  let constraints = vec![
    Constraint::new(
      "scope".to_string(),
      if hypotheses_count > 0 {
        "small"
      } else {
        "enterprise"
      }
      .to_string(),
    )
    .ok()?,
    Constraint::new(
      "scope".to_string(),
      if interviews_count > 1 {
        "small"
      } else {
        "enterprise"
      }
      .to_string(),
    )
    .ok()?,
    Constraint::new("consistency".to_string(), "true".to_string()).ok()?,
    Constraint::new("availability".to_string(), "true".to_string()).ok()?,
    Constraint::new(
      "partition_tolerance".to_string(),
      if blocking_holes == 0 { "false" } else { "true" }.to_string(),
    )
    .ok()?,
  ];

  let frame =
    crate::shared::mental_lattice::DecisionFrame::new("discover-phase".to_string(), constraints)
      .ok()?;
  ConflictReport::detect(&frame).ok()
}
