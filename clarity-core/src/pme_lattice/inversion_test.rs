#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::Utc;
use uuid::Uuid;

use super::inversion::{
  CognitiveBias, InversionAnalysis, InversionCategory, InversionError, InversionQuestion,
  StupidityCheck,
};

#[test]
fn cognitive_bias_display_confirmation_bias() {
  assert_eq!(
    CognitiveBias::ConfirmationBias.to_string(),
    "Confirmation Bias"
  );
}

#[test]
fn cognitive_bias_display_survivorship_bias() {
  assert_eq!(
    CognitiveBias::SurvivorshipBias.to_string(),
    "Survivorship Bias"
  );
}

#[test]
fn cognitive_bias_display_sunk_cost_fallacy() {
  assert_eq!(
    CognitiveBias::SunkCostFallacy.to_string(),
    "Sunk Cost Fallacy"
  );
}

#[test]
fn cognitive_bias_display_availability_heuristic() {
  assert_eq!(
    CognitiveBias::AvailabilityHeuristic.to_string(),
    "Availability Heuristic"
  );
}

#[test]
fn cognitive_bias_display_anchoring_bias() {
  assert_eq!(CognitiveBias::AnchoringBias.to_string(), "Anchoring Bias");
}

#[test]
fn cognitive_bias_display_optimism_bias() {
  assert_eq!(CognitiveBias::OptimismBias.to_string(), "Optimism Bias");
}

#[test]
fn cognitive_bias_display_bandwagon_effect() {
  assert_eq!(
    CognitiveBias::BandwagonEffect.to_string(),
    "Bandwagon Effect"
  );
}

#[test]
fn cognitive_bias_display_dunning_kruger() {
  assert_eq!(
    CognitiveBias::DunningKruger.to_string(),
    "Dunning-Kruger Effect"
  );
}

#[test]
fn inversion_category_display_market_failure() {
  assert_eq!(
    InversionCategory::MarketFailure.to_string(),
    "Market Failure"
  );
}

#[test]
fn inversion_category_display_product_failure() {
  assert_eq!(
    InversionCategory::ProductFailure.to_string(),
    "Product Failure"
  );
}

#[test]
fn inversion_category_display_team_failure() {
  assert_eq!(InversionCategory::TeamFailure.to_string(), "Team Failure");
}

#[test]
fn inversion_category_display_execution_failure() {
  assert_eq!(
    InversionCategory::ExecutionFailure.to_string(),
    "Execution Failure"
  );
}

#[test]
fn inversion_category_display_competition_failure() {
  assert_eq!(
    InversionCategory::CompetitionFailure.to_string(),
    "Competition Failure"
  );
}

#[test]
fn inversion_question_new_requires_non_empty_question() {
  let result = InversionQuestion::new(
    InversionCategory::MarketFailure,
    String::new(),
    "Users abandon the product".to_string(),
  );
  assert!(matches!(result, Err(InversionError::EmptyField { .. })));
}

#[test]
fn inversion_question_new_requires_non_empty_scenario() {
  let result = InversionQuestion::new(
    InversionCategory::MarketFailure,
    "Why would users leave?".to_string(),
    String::new(),
  );
  assert!(matches!(result, Err(InversionError::EmptyField { .. })));
}

#[test]
fn inversion_question_new_succeeds_with_valid_input() {
  let result = InversionQuestion::new(
    InversionCategory::MarketFailure,
    "Why would users abandon us?".to_string(),
    "Users leave because X".to_string(),
  );
  if let Ok(q) = result {
    assert_eq!(q.question, "Why would users abandon us?");
    assert_eq!(q.negative_scenario, "Users leave because X");
    assert!(q.prevention_strategy.is_none());
  } else {
    panic!("Expected Ok, got Err");
  }
}

#[test]
fn inversion_question_with_prevention_strategy() {
  let result = InversionQuestion::new(
    InversionCategory::ProductFailure,
    "What could break?".to_string(),
    "Feature fails to load".to_string(),
  );
  if let Ok(q) = result {
    let with_strategy = q.with_prevention_strategy("Add comprehensive error handling".to_string());
    assert!(with_strategy.prevention_strategy.is_some());
    assert_eq!(
      with_strategy.prevention_strategy,
      Some("Add comprehensive error handling".to_string())
    );
  }
}

#[test]
fn stupidity_check_new_creates_unanswered_check() {
  let check = StupidityCheck::new(
    CognitiveBias::ConfirmationBias,
    "Are you seeking only confirming evidence?".to_string(),
  );
  assert!(check.passed.is_none());
  assert!(check.evidence.is_none());
}

#[test]
fn stupidity_check_pass_sets_passed_true() {
  let check = StupidityCheck::new(
    CognitiveBias::OptimismBias,
    "Have you considered worst case?".to_string(),
  );
  let passed = check.pass();
  assert_eq!(passed.passed, Some(true));
}

#[test]
fn stupidity_check_fail_sets_passed_false() {
  let check = StupidityCheck::new(
    CognitiveBias::SunkCostFallacy,
    "Are you continuing due to past investment?".to_string(),
  );
  let failed = check.fail();
  assert_eq!(failed.passed, Some(false));
}

#[test]
fn stupidity_check_with_evidence() {
  let check = StupidityCheck::new(
    CognitiveBias::AvailabilityHeuristic,
    "Are you overweighting recent events?".to_string(),
  );
  let with_evidence =
    check.with_evidence("Reviewed 6 months of data, not just last week".to_string());
  assert!(with_evidence.evidence.is_some());
  assert_eq!(
    with_evidence.evidence,
    Some("Reviewed 6 months of data, not just last week".to_string())
  );
}

#[test]
fn inversion_analysis_new_creates_empty_analysis() {
  let analysis = InversionAnalysis::new();
  assert!(analysis.biases_detected.is_empty());
  assert!(analysis.checks.is_empty());
  assert!(analysis.inversion_questions.is_empty());
  assert!(analysis.failure_modes_identified.is_empty());
  assert!(analysis.prevention_strategies.is_empty());
}

#[test]
fn inversion_analysis_with_bias_detected() {
  let analysis = InversionAnalysis::new()
    .with_bias(CognitiveBias::ConfirmationBias)
    .with_bias(CognitiveBias::OptimismBias);

  assert_eq!(analysis.biases_detected.len(), 2);
  assert!(analysis
    .biases_detected
    .contains(&CognitiveBias::ConfirmationBias));
  assert!(analysis
    .biases_detected
    .contains(&CognitiveBias::OptimismBias));
}

#[test]
fn inversion_analysis_with_check() {
  let check = StupidityCheck::new(
    CognitiveBias::AnchoringBias,
    "First price too influential?".to_string(),
  );
  let analysis = InversionAnalysis::new().with_check(check);

  assert_eq!(analysis.checks.len(), 1);
}

#[test]
fn inversion_analysis_with_inversion_question() {
  let question = InversionQuestion::new(
    InversionCategory::MarketFailure,
    "Why would market reject this?".to_string(),
    "No product-market fit".to_string(),
  );

  if let Ok(q) = question {
    let analysis = InversionAnalysis::new().with_question(q);
    assert_eq!(analysis.inversion_questions.len(), 1);
  }
}

#[test]
fn inversion_analysis_with_failure_mode() {
  let analysis = InversionAnalysis::new()
    .with_failure_mode("Users don't understand value proposition".to_string())
    .with_failure_mode("Onboarding is too complex".to_string());

  assert_eq!(analysis.failure_modes_identified.len(), 2);
}

#[test]
fn inversion_analysis_with_prevention_strategy() {
  let analysis = InversionAnalysis::new()
    .with_prevention("Simplify onboarding to 3 steps".to_string())
    .with_prevention("A/B test value messaging".to_string());

  assert_eq!(analysis.prevention_strategies.len(), 2);
}

#[test]
fn inversion_analysis_with_scenario() {
  let scenario_id = Uuid::new_v4();
  let analysis = InversionAnalysis::new().with_scenario(scenario_id);

  assert_eq!(analysis.scenario_id, Some(scenario_id));
}

#[test]
fn inversion_analysis_all_checks_passed_when_empty() {
  let analysis = InversionAnalysis::new();
  assert!(analysis.all_checks_passed());
}

#[test]
fn inversion_analysis_all_checks_passed_when_all_true() {
  let check1 = StupidityCheck::new(CognitiveBias::ConfirmationBias, "Check 1".to_string()).pass();

  let check2 = StupidityCheck::new(CognitiveBias::OptimismBias, "Check 2".to_string()).pass();

  let analysis = InversionAnalysis::new()
    .with_check(check1)
    .with_check(check2);

  assert!(analysis.all_checks_passed());
}

#[test]
fn inversion_analysis_all_checks_passed_returns_false_when_any_failed() {
  let check1 = StupidityCheck::new(CognitiveBias::ConfirmationBias, "Check 1".to_string()).pass();

  let check2 = StupidityCheck::new(CognitiveBias::OptimismBias, "Check 2".to_string()).fail();

  let analysis = InversionAnalysis::new()
    .with_check(check1)
    .with_check(check2);

  assert!(!analysis.all_checks_passed());
}

#[test]
fn inversion_analysis_all_checks_passed_returns_false_when_unanswered() {
  let check1 = StupidityCheck::new(CognitiveBias::ConfirmationBias, "Check 1".to_string()).pass();

  let check2 = StupidityCheck::new(CognitiveBias::OptimismBias, "Check 2".to_string());

  let analysis = InversionAnalysis::new()
    .with_check(check1)
    .with_check(check2);

  assert!(!analysis.all_checks_passed());
}

#[test]
fn inversion_analysis_has_blocking_issues_when_unanswered_checks() {
  let check = StupidityCheck::new(CognitiveBias::OptimismBias, "Unanswered check".to_string());

  let analysis = InversionAnalysis::new().with_check(check);
  assert!(analysis.has_blocking_issues());
}

#[test]
fn inversion_analysis_has_blocking_issues_when_failed_checks() {
  let check =
    StupidityCheck::new(CognitiveBias::SunkCostFallacy, "Failed check".to_string()).fail();

  let analysis = InversionAnalysis::new().with_check(check);
  assert!(analysis.has_blocking_issues());
}

#[test]
fn inversion_analysis_no_blocking_issues_when_all_passed() {
  let check =
    StupidityCheck::new(CognitiveBias::ConfirmationBias, "Passed check".to_string()).pass();

  let analysis = InversionAnalysis::new().with_check(check);
  assert!(!analysis.has_blocking_issues());
}

#[test]
fn inversion_analysis_risk_score_zero_when_empty() {
  let analysis = InversionAnalysis::new();
  let score = analysis.calculate_risk_score();
  assert!((score - 0.0).abs() < f32::EPSILON);
}

#[test]
fn inversion_analysis_risk_score_increases_with_failed_checks() {
  let failed_check = StupidityCheck::new(CognitiveBias::OptimismBias, "Failed".to_string()).fail();

  let analysis = InversionAnalysis::new().with_check(failed_check);
  let score = analysis.calculate_risk_score();

  assert!(score > 0.0);
}

#[test]
fn inversion_analysis_risk_score_increases_with_biases() {
  let analysis = InversionAnalysis::new()
    .with_bias(CognitiveBias::ConfirmationBias)
    .with_bias(CognitiveBias::OptimismBias);

  let score = analysis.calculate_risk_score();
  assert!(score > 0.0);
}

#[test]
fn inversion_analysis_risk_score_reduces_with_passed_checks() {
  let analysis = InversionAnalysis::new()
    .with_bias(CognitiveBias::ConfirmationBias)
    .with_check(StupidityCheck::new(CognitiveBias::ConfirmationBias, "Check".to_string()).pass());

  let score = analysis.calculate_risk_score();
  assert!(score < 0.5);
}

#[test]
fn inversion_analysis_failed_checks_count() {
  let analysis = InversionAnalysis::new()
    .with_check(StupidityCheck::new(CognitiveBias::ConfirmationBias, "Pass".to_string()).pass())
    .with_check(StupidityCheck::new(CognitiveBias::OptimismBias, "Fail".to_string()).fail())
    .with_check(StupidityCheck::new(CognitiveBias::SunkCostFallacy, "Fail2".to_string()).fail());

  assert_eq!(analysis.failed_checks_count(), 2);
}

#[test]
fn inversion_analysis_passed_checks_count() {
  let analysis = InversionAnalysis::new()
    .with_check(StupidityCheck::new(CognitiveBias::ConfirmationBias, "Pass".to_string()).pass())
    .with_check(StupidityCheck::new(CognitiveBias::OptimismBias, "Fail".to_string()).fail())
    .with_check(StupidityCheck::new(CognitiveBias::SunkCostFallacy, "Pass2".to_string()).pass());

  assert_eq!(analysis.passed_checks_count(), 2);
}

#[test]
fn inversion_analysis_unanswered_checks_count() {
  let analysis = InversionAnalysis::new()
    .with_check(StupidityCheck::new(CognitiveBias::ConfirmationBias, "Pass".to_string()).pass())
    .with_check(StupidityCheck::new(
      CognitiveBias::OptimismBias,
      "Unanswered".to_string(),
    ))
    .with_check(StupidityCheck::new(
      CognitiveBias::SunkCostFallacy,
      "Unanswered2".to_string(),
    ));

  assert_eq!(analysis.unanswered_checks_count(), 2);
}

#[test]
fn inversion_question_serialization() {
  let question = InversionQuestion::new(
    InversionCategory::MarketFailure,
    "Test question".to_string(),
    "Test scenario".to_string(),
  );

  if let Ok(q) = question {
    if let Ok(json) = serde_json::to_string(&q) {
      let parsed: Result<InversionQuestion, _> = serde_json::from_str(&json);
      assert!(parsed.is_ok());
    }
  }
}

#[test]
fn cognitive_bias_serialization() {
  let bias = CognitiveBias::ConfirmationBias;
  let json = serde_json::to_string(&bias);
  assert!(json.is_ok());
  assert!(json.ok().map_or(false, |j| j.contains("confirmation_bias")));
}

#[test]
fn stupidity_check_serialization() {
  let check = StupidityCheck::new(CognitiveBias::OptimismBias, "Test check".to_string())
    .pass()
    .with_evidence("Test evidence".to_string());

  let json = serde_json::to_string(&check);
  assert!(json.is_ok());
}

#[test]
fn inversion_analysis_serialization() {
  let analysis = InversionAnalysis::new()
    .with_bias(CognitiveBias::ConfirmationBias)
    .with_failure_mode("Test failure".to_string());

  let json = serde_json::to_string(&analysis);
  assert!(json.is_ok());
}
