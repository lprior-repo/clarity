#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![cfg(test)]

use super::traffic_lights::*;

fn create_valid_affordance() -> Affordance {
  Affordance {
    name: "Submit Button".to_string(),
    action: "click".to_string(),
    strength: TrafficLight::Green,
    description: "Submit form data".to_string(),
  }
}

fn create_dangerous_affordance() -> Affordance {
  Affordance {
    name: "Delete Account".to_string(),
    action: "click".to_string(),
    strength: TrafficLight::Red,
    description: "Permanently delete user account".to_string(),
  }
}

#[test]
fn test_traffic_light_variants() {
  assert!(matches!(TrafficLight::Green, TrafficLight::Green));
  assert!(matches!(TrafficLight::Yellow, TrafficLight::Yellow));
  assert!(matches!(TrafficLight::Red, TrafficLight::Red));
}

#[test]
fn test_traffic_light_safety_ordering() {
  assert!(TrafficLight::Green.is_safer_than(&TrafficLight::Yellow));
  assert!(TrafficLight::Yellow.is_safer_than(&TrafficLight::Red));
  assert!(!TrafficLight::Red.is_safer_than(&TrafficLight::Green));
}

#[test]
fn test_signifier_creation() {
  let result = Signifier::new(
    "Submit Button".to_string(),
    SignifierType::Visual,
    "Green button".to_string(),
  );
  assert!(result.is_ok());
}

#[test]
fn test_signifier_rejects_empty_name() {
  let result = Signifier::new(
    "".to_string(),
    SignifierType::Visual,
    "Description".to_string(),
  );
  assert!(matches!(result, Err(TrafficLightError::EmptySignifierName)));
}

#[test]
fn test_signifier_type_variants() {
  assert!(matches!(SignifierType::Visual, SignifierType::Visual));
  assert!(matches!(SignifierType::Auditory, SignifierType::Auditory));
  assert!(matches!(SignifierType::Haptic, SignifierType::Haptic));
  assert!(matches!(SignifierType::Textual, SignifierType::Textual));
}

#[test]
fn test_affordance_is_safe() {
  let safe = create_valid_affordance();
  assert!(safe.is_safe());

  let dangerous = create_dangerous_affordance();
  assert!(!dangerous.is_safe());
}

#[test]
fn test_malfunctioning_traffic_light_detection() {
  let safe_action = Affordance {
    name: "Safe Action".to_string(),
    action: "triple_confirm_click".to_string(),
    strength: TrafficLight::Green,
    description: "Safe".to_string(),
  };
  let dangerous_action = Affordance {
    name: "Dangerous Action".to_string(),
    action: "click".to_string(),
    strength: TrafficLight::Red,
    description: "Dangerous".to_string(),
  };

  let malfunction = MalfunctioningTrafficLight {
    dangerous_affordance: dangerous_action,
    safe_alternative: safe_action,
    reason: "Delete is easier than save".to_string(),
  };

  assert!(malfunction.is_malfunction());
}

#[test]
fn test_traffic_light_audit_creation() {
  let result = TrafficLightAudit::new("User Dashboard Audit".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_traffic_light_audit_rejects_empty_name() {
  let result = TrafficLightAudit::new("".to_string());
  assert!(matches!(result, Err(TrafficLightError::EmptyAuditName)));
}

#[test]
fn test_traffic_light_audit_add_affordance() {
  let mut audit = TrafficLightAudit::new("Test".to_string()).expect("valid");
  let affordance = create_valid_affordance();
  audit.add_affordance(affordance);
  assert_eq!(audit.affordances.len(), 1);
}

#[test]
fn test_traffic_light_audit_detect_malfunctions() {
  let mut audit = TrafficLightAudit::new("Test".to_string()).expect("valid");

  audit.add_affordance(Affordance {
    name: "Delete".to_string(),
    action: "single_click".to_string(),
    strength: TrafficLight::Red,
    description: "Delete all data".to_string(),
  });

  audit.add_affordance(Affordance {
    name: "Save".to_string(),
    action: "triple_confirm_click".to_string(),
    strength: TrafficLight::Green,
    description: "Save changes".to_string(),
  });

  let malfunctions = audit.detect_malfunctions();
  assert!(!malfunctions.is_empty());
}

#[test]
fn test_traffic_light_audit_clean_interface() {
  let mut audit = TrafficLightAudit::new("Test".to_string()).expect("valid");

  audit.add_affordance(Affordance {
    name: "Save".to_string(),
    action: "single_click".to_string(),
    strength: TrafficLight::Green,
    description: "Save".to_string(),
  });

  audit.add_affordance(Affordance {
    name: "Delete".to_string(),
    action: "confirm_click".to_string(),
    strength: TrafficLight::Red,
    description: "Delete".to_string(),
  });

  let malfunctions = audit.detect_malfunctions();
  assert!(malfunctions.is_empty());
}

#[test]
fn test_traffic_light_audit_generate_report() {
  let mut audit = TrafficLightAudit::new("Test".to_string()).expect("valid");
  audit.add_affordance(create_valid_affordance());

  let report = audit.generate_report();
  assert!(report.contains("Traffic Light Audit"));
  assert!(report.contains("Green"));
}

#[test]
fn test_affordance_strength_classification() {
  let green = Affordance {
    name: "Green".to_string(),
    action: "test".to_string(),
    strength: TrafficLight::Green,
    description: "Safe".to_string(),
  };
  assert!(green.is_safe());

  let yellow = Affordance {
    name: "Yellow".to_string(),
    action: "test".to_string(),
    strength: TrafficLight::Yellow,
    description: "Caution".to_string(),
  };
  assert!(!yellow.is_safe());
  assert!(yellow.requires_caution());

  let red = Affordance {
    name: "Red".to_string(),
    action: "test".to_string(),
    strength: TrafficLight::Red,
    description: "Danger".to_string(),
  };
  assert!(!red.is_safe());
  assert!(!red.requires_caution());
  assert!(red.is_dangerous());
}
