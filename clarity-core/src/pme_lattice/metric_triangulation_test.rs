#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use super::metric_triangulation::*;

fn create_valid_kpi() -> KPIMetric {
  KPIMetric {
    name: "Monthly Recurring Revenue".to_string(),
    metric_type: KPIMetricType::Profit,
    target_value: 100000.0,
    current_value: 75000.0,
  }
}

fn create_valid_adoption() -> AdoptionMetric {
  AdoptionMetric {
    name: "Daily Active Users".to_string(),
    active_count: 5000,
    total_registered: 10000,
    period_days: 1,
  }
}

fn create_valid_value() -> ValueMetric {
  ValueMetric {
    name: "Time Saved Per User".to_string(),
    metric_type: ValueMetricType::TimeSaved,
    unit: "minutes".to_string(),
    value: 30.0,
  }
}

#[test]
fn test_kpi_metric_is_healthy() {
  let kpi = KPIMetric {
    name: "Test".to_string(),
    metric_type: KPIMetricType::Profit,
    target_value: 100.0,
    current_value: 80.0,
  };
  assert!(kpi.is_healthy());
}

#[test]
fn test_kpi_metric_is_unhealthy() {
  let kpi = KPIMetric {
    name: "Test".to_string(),
    metric_type: KPIMetricType::Profit,
    target_value: 100.0,
    current_value: 40.0,
  };
  assert!(!kpi.is_healthy());
}

#[test]
fn test_adoption_metric_rejects_vanity() {
  let vanity = AdoptionMetric {
    name: "Total Registered Users".to_string(),
    active_count: 100,
    total_registered: 10000,
    period_days: 1,
  };
  assert!(vanity.is_vanity_metric());
}

#[test]
fn test_adoption_metric_accepts_real() {
  let real = AdoptionMetric {
    name: "DAU".to_string(),
    active_count: 5000,
    total_registered: 10000,
    period_days: 1,
  };
  assert!(!real.is_vanity_metric());
}

#[test]
fn test_value_metric_creation() {
  let metric = create_valid_value();
  assert_eq!(metric.name, "Time Saved Per User");
  assert_eq!(metric.value, 30.0);
}

#[test]
fn test_metric_triangulation_creation() {
  let result = MetricTriangulation::new("User Engagement".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_metric_triangulation_rejects_empty_name() {
  let result = MetricTriangulation::new("".to_string());
  assert!(matches!(result, Err(TriangulationError::EmptyName)));
}

#[test]
fn test_metric_triangulation_add_kpi() {
  let mut tri = MetricTriangulation::new("Test".to_string()).expect("valid");
  let kpi = create_valid_kpi();
  let result = tri.add_kpi(kpi);
  assert!(result.is_ok());
  assert_eq!(tri.kpis.len(), 1);
}

#[test]
fn test_metric_triangulation_add_adoption() {
  let mut tri = MetricTriangulation::new("Test".to_string()).expect("valid");
  let adoption = create_valid_adoption();
  let result = tri.add_adoption_metric(adoption);
  assert!(result.is_ok());
  assert_eq!(tri.adoption_metrics.len(), 1);
}

#[test]
fn test_metric_triangulation_add_value() {
  let mut tri = MetricTriangulation::new("Test".to_string()).expect("valid");
  let value = create_valid_value();
  let result = tri.add_value_metric(value);
  assert!(result.is_ok());
  assert_eq!(tri.value_metrics.len(), 1);
}

#[test]
fn test_triangulation_result_validates_all_three_pillars() {
  let mut tri = MetricTriangulation::new("Complete".to_string()).expect("valid");
  tri.add_kpi(create_valid_kpi()).expect("kpi");
  tri
    .add_adoption_metric(create_valid_adoption())
    .expect("adoption");
  tri.add_value_metric(create_valid_value()).expect("value");

  let report = tri.validate();
  assert!(report.all_pillars_present());
}

#[test]
fn test_triangulation_result_warns_missing_pillars() {
  let mut tri = MetricTriangulation::new("Incomplete".to_string()).expect("valid");
  tri.add_kpi(create_valid_kpi()).expect("kpi");

  let report = tri.validate();
  assert!(!report.all_pillars_present());
  assert!(report.warnings.iter().any(|w| w.contains("missing")));
}

#[test]
fn test_triangulation_detects_vanity_metrics() {
  let mut tri = MetricTriangulation::new("Vanity".to_string()).expect("valid");
  let vanity = AdoptionMetric {
    name: "Total Users".to_string(),
    active_count: 10,
    total_registered: 10000,
    period_days: 1,
  };
  tri.add_adoption_metric(vanity).expect("added");

  let report = tri.validate();
  assert!(report.warnings.iter().any(|w| w.contains("Vanity")));
}

#[test]
fn test_kpi_metric_type_variants() {
  assert!(matches!(KPIMetricType::Profit, KPIMetricType::Profit));
  assert!(matches!(KPIMetricType::Revenue, KPIMetricType::Revenue));
  assert!(matches!(KPIMetricType::Cost, KPIMetricType::Cost));
  assert!(matches!(
    KPIMetricType::Custom("NPS".to_string()),
    KPIMetricType::Custom(_)
  ));
}

#[test]
fn test_value_metric_type_variants() {
  assert!(matches!(
    ValueMetricType::TimeSaved,
    ValueMetricType::TimeSaved
  ));
  assert!(matches!(
    ValueMetricType::ErrorsAverted,
    ValueMetricType::ErrorsAverted
  ));
  assert!(matches!(
    ValueMetricType::InsightsGenerated,
    ValueMetricType::InsightsGenerated
  ));
}
