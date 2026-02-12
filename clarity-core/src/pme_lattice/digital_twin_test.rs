#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![cfg(test)]

use super::digital_twin::*;
use chrono::Utc;

fn create_valid_scenario() -> ScenarioStep {
  ScenarioStep {
    action: "user_clicks_button".to_string(),
    expected_outcome: "button_responds".to_string(),
    timestamp: Utc::now(),
  }
}

fn create_valid_load_pattern() -> LoadPattern {
  LoadPattern::Constant {
    requests_per_second: 100,
  }
}

#[test]
fn test_scenario_test_creation_with_valid_steps() {
  let steps = vec![create_valid_scenario()];
  let result = ScenarioTest::new("login_flow".to_string(), steps);
  assert!(result.is_ok());
  let test = result.expect("valid scenario");
  assert_eq!(test.name, "login_flow");
  assert_eq!(test.steps.len(), 1);
}

#[test]
fn test_scenario_test_rejects_empty_name() {
  let steps = vec![create_valid_scenario()];
  let result = ScenarioTest::new("".to_string(), steps);
  assert!(matches!(result, Err(DigitalTwinError::EmptyScenarioName)));
}

#[test]
fn test_scenario_test_rejects_empty_steps() {
  let result = ScenarioTest::new("test".to_string(), vec![]);
  assert!(matches!(result, Err(DigitalTwinError::EmptySteps)));
}

#[test]
fn test_load_simulation_creation() {
  let pattern = create_valid_load_pattern();
  let result = LoadSimulation::new("api_stress_test".to_string(), pattern, 60);
  assert!(result.is_ok());
  let sim = result.expect("valid simulation");
  assert_eq!(sim.name, "api_stress_test");
  assert_eq!(sim.duration_seconds, 60);
}

#[test]
fn test_load_simulation_rejects_zero_duration() {
  let pattern = create_valid_load_pattern();
  let result = LoadSimulation::new("test".to_string(), pattern, 0);
  assert!(matches!(result, Err(DigitalTwinError::InvalidDuration)));
}

#[test]
fn test_load_simulation_rejects_empty_name() {
  let pattern = create_valid_load_pattern();
  let result = LoadSimulation::new("".to_string(), pattern, 10);
  assert!(matches!(result, Err(DigitalTwinError::EmptySimulationName)));
}

#[test]
fn test_metric_dashboard_creation() {
  let result = MetricDashboard::new("production_metrics".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_metric_dashboard_rejects_empty_name() {
  let result = MetricDashboard::new("".to_string());
  assert!(matches!(result, Err(DigitalTwinError::EmptyDashboardName)));
}

#[test]
fn test_metric_dashboard_record_metric() {
  let mut dashboard = MetricDashboard::new("test".to_string()).expect("valid dashboard");
  let result = dashboard.record_metric("response_time_ms".to_string(), 150.5);
  assert!(result.is_ok());
  assert_eq!(dashboard.metrics.len(), 1);
}

#[test]
fn test_digital_twin_manager_creation() {
  let result = DigitalTwinManager::new("production_twin".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_digital_twin_manager_rejects_empty_name() {
  let result = DigitalTwinManager::new("".to_string());
  assert!(matches!(result, Err(DigitalTwinError::EmptyTwinName)));
}

#[test]
fn test_digital_twin_manager_add_scenario() {
  let mut manager = DigitalTwinManager::new("test".to_string()).expect("valid manager");
  let scenario =
    ScenarioTest::new("login".to_string(), vec![create_valid_scenario()]).expect("valid scenario");
  manager.add_scenario(scenario);
  assert_eq!(manager.scenarios.len(), 1);
}

#[test]
fn test_digital_twin_manager_run_simulation() {
  let manager = DigitalTwinManager::new("test".to_string()).expect("valid manager");
  let pattern = LoadPattern::Constant {
    requests_per_second: 10,
  };
  let sim = LoadSimulation::new("quick_test".to_string(), pattern, 1).expect("valid sim");
  let report = manager.run_simulation(sim);
  assert!(report.success);
}

#[test]
fn test_spike_load_pattern() {
  let pattern = LoadPattern::Spike {
    baseline_rps: 10,
    peak_rps: 100,
    spike_duration_seconds: 5,
  };
  let result = LoadSimulation::new("spike_test".to_string(), pattern, 10);
  assert!(result.is_ok());
}

#[test]
fn test_gradual_ramp_pattern() {
  let pattern = LoadPattern::GradualRamp {
    start_rps: 10,
    end_rps: 100,
  };
  let result = LoadSimulation::new("ramp_test".to_string(), pattern, 60);
  assert!(result.is_ok());
}

#[test]
fn test_scenario_result_tracking() {
  let test =
    ScenarioTest::new("login".to_string(), vec![create_valid_scenario()]).expect("valid test");
  let result = test.run();
  assert!(result.is_ok());
  let report = result.expect("scenario report");
  assert!(report.passed);
  assert_eq!(report.steps_completed, 1);
}
