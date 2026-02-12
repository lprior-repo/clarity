//! Tests for quality_dimensions module - EQI Framework
//!
//! Test quality doesn't matter - we test source code quality.

#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]

use super::quality_dimensions::{
  EQIAssessment, ImprovementAction, QualityDimension, QualityDimensionError, QualityMetric,
};

// ============================================================================
// QUALITY DIMENSION DISPLAY TESTS
// ============================================================================

#[test]
fn quality_dimension_display_completeness() {
  assert_eq!(QualityDimension::Completeness.to_string(), "Completeness");
}

#[test]
fn quality_dimension_display_consistency() {
  assert_eq!(QualityDimension::Consistency.to_string(), "Consistency");
}

#[test]
fn quality_dimension_display_testability() {
  assert_eq!(QualityDimension::Testability.to_string(), "Testability");
}

#[test]
fn quality_dimension_display_clarity() {
  assert_eq!(QualityDimension::Clarity.to_string(), "Clarity");
}

#[test]
fn quality_dimension_display_security() {
  assert_eq!(QualityDimension::Security.to_string(), "Security");
}

#[test]
fn quality_dimension_display_performance() {
  assert_eq!(QualityDimension::Performance.to_string(), "Performance");
}

#[test]
fn quality_dimension_display_maintainability() {
  assert_eq!(
    QualityDimension::Maintainability.to_string(),
    "Maintainability"
  );
}

// ============================================================================
// QUALITY DIMENSION SERIALIZATION TESTS
// ============================================================================

#[test]
fn quality_dimension_serialization() {
  let dimension = QualityDimension::Completeness;
  let json = serde_json::to_string(&dimension);
  assert!(json.is_ok());
  assert!(json.ok().map_or(false, |j| j.contains("completeness")));
}

#[test]
fn quality_dimension_deserialization() {
  let json = "\"completeness\"";
  let parsed: Result<QualityDimension, _> = serde_json::from_str(json);
  assert!(parsed.is_ok());
  assert_eq!(parsed.ok(), Some(QualityDimension::Completeness));
}

// ============================================================================
// QUALITY METRIC TESTS
// ============================================================================

#[test]
fn quality_metric_new_succeeds_with_valid_score() {
  let result = QualityMetric::new(
    QualityDimension::Completeness,
    0.75,
    "All requirements covered".to_string(),
  );
  assert!(result.is_ok());
  if let Ok(metric) = result {
    assert_eq!(metric.dimension, QualityDimension::Completeness);
    assert!((metric.score - 0.75).abs() < f32::EPSILON);
    assert_eq!(metric.evidence, "All requirements covered");
  }
}

#[test]
fn quality_metric_new_rejects_negative_score() {
  let result = QualityMetric::new(
    QualityDimension::Security,
    -0.1,
    "Invalid score".to_string(),
  );
  assert!(matches!(
    result,
    Err(QualityDimensionError::InvalidScore { .. })
  ));
}

#[test]
fn quality_metric_new_rejects_score_above_one() {
  let result = QualityMetric::new(
    QualityDimension::Performance,
    1.5,
    "Invalid score".to_string(),
  );
  assert!(matches!(
    result,
    Err(QualityDimensionError::InvalidScore { .. })
  ));
}

#[test]
fn quality_metric_new_rejects_empty_evidence() {
  let result = QualityMetric::new(QualityDimension::Clarity, 0.8, String::new());
  assert!(matches!(result, Err(QualityDimensionError::EmptyEvidence)));
}

#[test]
fn quality_metric_new_clamps_score_to_valid_range() {
  // Test boundary values
  let zero = QualityMetric::new(QualityDimension::Completeness, 0.0, "Valid".to_string());
  let one = QualityMetric::new(QualityDimension::Completeness, 1.0, "Valid".to_string());

  assert!(zero.is_ok());
  assert!(one.is_ok());
}

#[test]
fn quality_metric_is_weak_returns_true_for_low_score() {
  let metric = QualityMetric::new(
    QualityDimension::Security,
    0.4,
    "Some vulnerabilities found".to_string(),
  );
  if let Ok(m) = metric {
    assert!(m.is_weak());
  }
}

#[test]
fn quality_metric_is_weak_returns_false_for_high_score() {
  let metric = QualityMetric::new(
    QualityDimension::Security,
    0.8,
    "No vulnerabilities".to_string(),
  );
  if let Ok(m) = metric {
    assert!(!m.is_weak());
  }
}

#[test]
fn quality_metric_is_strong_returns_true_for_high_score() {
  let metric = QualityMetric::new(
    QualityDimension::Testability,
    0.9,
    "Excellent test coverage".to_string(),
  );
  if let Ok(m) = metric {
    assert!(m.is_strong());
  }
}

#[test]
fn quality_metric_is_strong_returns_false_for_low_score() {
  let metric = QualityMetric::new(
    QualityDimension::Testability,
    0.5,
    "Moderate coverage".to_string(),
  );
  if let Ok(m) = metric {
    assert!(!m.is_strong());
  }
}

#[test]
fn quality_metric_serialization() {
  let metric = QualityMetric::new(
    QualityDimension::Completeness,
    0.85,
    "Good coverage".to_string(),
  );
  if let Ok(m) = metric {
    let json = serde_json::to_string(&m);
    assert!(json.is_ok());
  }
}

// ============================================================================
// EQI ASSESSMENT BUILDER TESTS
// ============================================================================

#[test]
fn eqi_assessment_new_creates_empty_assessment() {
  let assessment = EQIAssessment::new();
  assert!(assessment.metrics.is_empty());
  assert!((assessment.overall_score - 0.0).abs() < f32::EPSILON);
  assert!(assessment.recommendations.is_empty());
}

#[test]
fn eqi_assessment_builder_with_metric() {
  let metric = QualityMetric::new(
    QualityDimension::Completeness,
    0.8,
    "Complete features".to_string(),
  );
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new().with_metric(m);
    assert_eq!(assessment.metrics.len(), 1);
  }
}

#[test]
fn eqi_assessment_builder_with_recommendation() {
  let assessment = EQIAssessment::new()
    .with_recommendation("Improve test coverage".to_string())
    .with_recommendation("Add security audit".to_string());

  assert_eq!(assessment.recommendations.len(), 2);
}

#[test]
fn eqi_assessment_builder_with_overall_score() {
  let assessment = EQIAssessment::new().with_overall_score(0.75);
  assert!((assessment.overall_score - 0.75).abs() < f32::EPSILON);
}

#[test]
fn eqi_assessment_builder_chaining() {
  let metric = QualityMetric::new(
    QualityDimension::Security,
    0.9,
    "Strong security posture".to_string(),
  );
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new()
      .with_metric(m)
      .with_recommendation("Maintain security practices".to_string())
      .with_overall_score(0.9);

    assert_eq!(assessment.metrics.len(), 1);
    assert_eq!(assessment.recommendations.len(), 1);
    assert!((assessment.overall_score - 0.9).abs() < f32::EPSILON);
  }
}

// ============================================================================
// EQI ASSESSMENT CALCULATE OVERALL SCORE TESTS
// ============================================================================

#[test]
fn eqi_assessment_calculate_overall_score_empty_returns_zero() {
  let assessment = EQIAssessment::new();
  let score = assessment.calculate_overall_score();
  assert!((score - 0.0).abs() < f32::EPSILON);
}

#[test]
fn eqi_assessment_calculate_overall_score_single_metric() {
  let metric = QualityMetric::new(QualityDimension::Completeness, 0.8, "Good".to_string());
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new().with_metric(m);
    let score = assessment.calculate_overall_score();
    assert!((score - 0.8).abs() < f32::EPSILON);
  }
}

#[test]
fn eqi_assessment_calculate_overall_score_multiple_metrics() {
  let m1 = QualityMetric::new(QualityDimension::Completeness, 0.8, "Good".to_string());
  let m2 = QualityMetric::new(QualityDimension::Security, 0.6, "Fair".to_string());
  let m3 = QualityMetric::new(QualityDimension::Performance, 0.7, "Good".to_string());

  if let (Ok(metric1), Ok(metric2), Ok(metric3)) = (m1, m2, m3) {
    let assessment = EQIAssessment::new()
      .with_metric(metric1)
      .with_metric(metric2)
      .with_metric(metric3);

    let score = assessment.calculate_overall_score();
    // Average: (0.8 + 0.6 + 0.7) / 3 = 0.7
    assert!((score - 0.7).abs() < f32::EPSILON);
  }
}

#[test]
fn eqi_assessment_calculate_overall_score_returns_correct_value() {
  let metric = QualityMetric::new(QualityDimension::Testability, 0.9, "Excellent".to_string());
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new().with_metric(m);
    let score = assessment.calculate_overall_score();
    // Pure function returns calculated value
    assert!((score - 0.9).abs() < f32::EPSILON);
  }
}

// ============================================================================
// EQI ASSESSMENT GET WEAK DIMENSIONS TESTS
// ============================================================================

#[test]
fn eqi_assessment_get_weak_dimensions_empty_returns_empty() {
  let assessment = EQIAssessment::new();
  let weak = assessment.get_weak_dimensions();
  assert!(weak.is_empty());
}

#[test]
fn eqi_assessment_get_weak_dimensions_filters_correctly() {
  let strong = QualityMetric::new(QualityDimension::Completeness, 0.9, "Strong".to_string());
  let weak = QualityMetric::new(QualityDimension::Security, 0.3, "Weak".to_string());
  let moderate = QualityMetric::new(QualityDimension::Performance, 0.6, "Moderate".to_string());

  if let (Ok(s), Ok(w), Ok(m)) = (strong, weak, moderate) {
    let assessment = EQIAssessment::new()
      .with_metric(s)
      .with_metric(w)
      .with_metric(m);

    let weak_dims = assessment.get_weak_dimensions();
    assert_eq!(weak_dims.len(), 1);
    assert!(weak_dims.contains(&QualityDimension::Security));
  }
}

#[test]
fn eqi_assessment_get_weak_dimensions_multiple_weak() {
  let w1 = QualityMetric::new(QualityDimension::Security, 0.3, "Weak".to_string());
  let w2 = QualityMetric::new(QualityDimension::Maintainability, 0.4, "Weak".to_string());

  if let (Ok(metric1), Ok(metric2)) = (w1, w2) {
    let assessment = EQIAssessment::new()
      .with_metric(metric1)
      .with_metric(metric2);

    let weak_dims = assessment.get_weak_dimensions();
    assert_eq!(weak_dims.len(), 2);
  }
}

// ============================================================================
// EQI ASSESSMENT GENERATE IMPROVEMENT PLAN TESTS
// ============================================================================

#[test]
fn eqi_assessment_generate_improvement_plan_empty_returns_empty() {
  let assessment = EQIAssessment::new();
  let plan = assessment.generate_improvement_plan();
  assert!(plan.is_empty());
}

#[test]
fn eqi_assessment_generate_improvement_plan_includes_weak_dimensions() {
  let weak_metric = QualityMetric::new(
    QualityDimension::Security,
    0.3,
    "Security issues found".to_string(),
  );

  if let Ok(m) = weak_metric {
    let assessment = EQIAssessment::new().with_metric(m);
    let plan = assessment.generate_improvement_plan();

    assert!(!plan.is_empty());
    // Plan should contain actions for Security dimension
    assert!(plan
      .iter()
      .any(|a| a.dimension == QualityDimension::Security));
  }
}

#[test]
fn eqi_assessment_generate_improvement_plan_prioritizes_weakest() {
  let very_weak = QualityMetric::new(
    QualityDimension::Security,
    0.2,
    "Critical issues".to_string(),
  );
  let weak = QualityMetric::new(
    QualityDimension::Maintainability,
    0.4,
    "Some issues".to_string(),
  );

  if let (Ok(m1), Ok(m2)) = (very_weak, weak) {
    let assessment = EQIAssessment::new().with_metric(m1).with_metric(m2);
    let plan = assessment.generate_improvement_plan();

    // Should be prioritized by score (lowest first)
    assert!(!plan.is_empty());
    if plan.len() >= 2 {
      assert!(plan[0].dimension == QualityDimension::Security);
    }
  }
}

#[test]
fn eqi_assessment_generate_improvement_plan_excludes_strong_dimensions() {
  let strong_metric = QualityMetric::new(
    QualityDimension::Completeness,
    0.95,
    "Excellent".to_string(),
  );

  if let Ok(m) = strong_metric {
    let assessment = EQIAssessment::new().with_metric(m);
    let plan = assessment.generate_improvement_plan();

    // Strong dimensions should not have improvement actions
    assert!(plan
      .iter()
      .all(|a| a.dimension != QualityDimension::Completeness));
  }
}

// ============================================================================
// IMPROVEMENT ACTION TESTS
// ============================================================================

#[test]
fn improvement_action_new() {
  let action = ImprovementAction::new(
    QualityDimension::Security,
    "Add authentication".to_string(),
    1,
  );
  assert_eq!(action.dimension, QualityDimension::Security);
  assert_eq!(action.description, "Add authentication");
  assert_eq!(action.priority, 1);
}

#[test]
fn improvement_action_serialization() {
  let action = ImprovementAction::new(
    QualityDimension::Performance,
    "Optimize queries".to_string(),
    2,
  );
  let json = serde_json::to_string(&action);
  assert!(json.is_ok());
}

// ============================================================================
// EQI ASSESSMENT ASSESS METHOD TESTS
// ============================================================================

#[test]
fn eqi_assessment_assess_with_empty_metrics() {
  let result = EQIAssessment::assess(Vec::new());
  assert!(result.is_ok());
  if let Ok(assessment) = result {
    assert!(assessment.metrics.is_empty());
    assert!((assessment.overall_score - 0.0).abs() < f32::EPSILON);
  }
}

#[test]
fn eqi_assessment_assess_with_valid_metrics() {
  let metric = QualityMetric::new(QualityDimension::Completeness, 0.8, "Good".to_string());
  if let Ok(m) = metric {
    let result = EQIAssessment::assess(vec![m]);
    assert!(result.is_ok());
    if let Ok(assessment) = result {
      assert_eq!(assessment.metrics.len(), 1);
      assert!((assessment.overall_score - 0.8).abs() < f32::EPSILON);
    }
  }
}

#[test]
fn eqi_assessment_assess_generates_recommendations() {
  let weak_metric = QualityMetric::new(
    QualityDimension::Security,
    0.3,
    "Security issues".to_string(),
  );
  if let Ok(m) = weak_metric {
    let result = EQIAssessment::assess(vec![m]);
    assert!(result.is_ok());
    if let Ok(assessment) = result {
      assert!(!assessment.recommendations.is_empty());
    }
  }
}

#[test]
fn eqi_assessment_assess_with_multiple_metrics() {
  let m1 = QualityMetric::new(QualityDimension::Completeness, 0.9, "Good".to_string());
  let m2 = QualityMetric::new(QualityDimension::Security, 0.5, "Fair".to_string());
  let m3 = QualityMetric::new(QualityDimension::Performance, 0.7, "Good".to_string());

  if let (Ok(metric1), Ok(metric2), Ok(metric3)) = (m1, m2, m3) {
    let result = EQIAssessment::assess(vec![metric1, metric2, metric3]);
    assert!(result.is_ok());
    if let Ok(assessment) = result {
      assert_eq!(assessment.metrics.len(), 3);
      // Overall score should be average
      let expected = (0.9 + 0.5 + 0.7) / 3.0;
      assert!((assessment.overall_score - expected).abs() < f32::EPSILON);
    }
  }
}

// ============================================================================
// EQI ASSESSMENT HELPERS TESTS
// ============================================================================

#[test]
fn eqi_assessment_get_metric_for_dimension() {
  let metric = QualityMetric::new(QualityDimension::Security, 0.8, "Good".to_string());
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new().with_metric(m);

    let found = assessment.get_metric_for_dimension(QualityDimension::Security);
    assert!(found.is_some());
    if let Some(m) = found {
      assert_eq!(m.dimension, QualityDimension::Security);
      assert!((m.score - 0.8).abs() < f32::EPSILON);
    }
  }
}

#[test]
fn eqi_assessment_get_metric_for_dimension_not_found() {
  let metric = QualityMetric::new(QualityDimension::Security, 0.8, "Good".to_string());
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new().with_metric(m);

    let not_found = assessment.get_metric_for_dimension(QualityDimension::Performance);
    assert!(not_found.is_none());
  }
}

#[test]
fn eqi_assessment_has_weak_dimensions() {
  let weak = QualityMetric::new(QualityDimension::Security, 0.3, "Weak".to_string());
  if let Ok(m) = weak {
    let assessment = EQIAssessment::new().with_metric(m);
    assert!(assessment.has_weak_dimensions());
  }
}

#[test]
fn eqi_assessment_no_weak_dimensions() {
  let strong = QualityMetric::new(QualityDimension::Security, 0.9, "Strong".to_string());
  if let Ok(m) = strong {
    let assessment = EQIAssessment::new().with_metric(m);
    assert!(!assessment.has_weak_dimensions());
  }
}

#[test]
fn eqi_assessment_serialization() {
  let metric = QualityMetric::new(QualityDimension::Completeness, 0.85, "Good".to_string());
  if let Ok(m) = metric {
    let assessment = EQIAssessment::new()
      .with_metric(m)
      .with_recommendation("Test".to_string());

    let json = serde_json::to_string(&assessment);
    assert!(json.is_ok());
  }
}

// ============================================================================
// ALL DIMENSIONS TEST
// ============================================================================

#[test]
fn quality_dimension_all_variants() {
  // Ensure all dimension variants are covered
  let dimensions = [
    QualityDimension::Completeness,
    QualityDimension::Consistency,
    QualityDimension::Testability,
    QualityDimension::Clarity,
    QualityDimension::Security,
    QualityDimension::Performance,
    QualityDimension::Maintainability,
  ];

  for dimension in dimensions {
    let display = dimension.to_string();
    assert!(!display.is_empty());

    let json = serde_json::to_string(&dimension);
    assert!(json.is_ok());
  }
}

#[test]
fn quality_metric_all_dimensions() {
  // Test creating metrics for all dimensions
  let dimensions = [
    QualityDimension::Completeness,
    QualityDimension::Consistency,
    QualityDimension::Testability,
    QualityDimension::Clarity,
    QualityDimension::Security,
    QualityDimension::Performance,
    QualityDimension::Maintainability,
  ];

  for dimension in dimensions {
    let result = QualityMetric::new(dimension, 0.75, format!("Evidence for {dimension}"));
    assert!(result.is_ok());
  }
}
