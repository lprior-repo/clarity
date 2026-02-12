#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use super::support_flywheel::*;
use chrono::Utc;

fn create_valid_friction_entry() -> FrictionEntry {
  FrictionEntry {
    feature: "Login Flow".to_string(),
    description: "Confusing error message".to_string(),
    emotional_state: EmotionalState::Frustrated,
    timestamp: Utc::now(),
  }
}

fn create_valid_support_ticket() -> SupportTicket {
  SupportTicket::new(
    "TKT-001".to_string(),
    "Cannot reset password".to_string(),
    "User reports password reset not working".to_string(),
    TicketSeverity::Medium,
  )
  .expect("valid ticket")
}

#[test]
fn test_emotional_state_variants() {
  assert!(matches!(
    EmotionalState::Frustrated,
    EmotionalState::Frustrated
  ));
  assert!(matches!(EmotionalState::Confused, EmotionalState::Confused));
  assert!(matches!(EmotionalState::Neutral, EmotionalState::Neutral));
  assert!(matches!(EmotionalState::Pleased, EmotionalState::Pleased));
}

#[test]
fn test_friction_log_creation() {
  let result = FrictionLog::new("Dogfooding Sprint 1".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_friction_log_rejects_empty_name() {
  let result = FrictionLog::new("".to_string());
  assert!(matches!(result, Err(SupportFlywheelError::EmptyLogName)));
}

#[test]
fn test_friction_log_add_entry() {
  let mut log = FrictionLog::new("Test".to_string()).expect("valid log");
  let entry = create_valid_friction_entry();
  let result = log.add_entry(entry);
  assert!(result.is_ok());
  assert_eq!(log.entries.len(), 1);
}

#[test]
fn test_friction_log_rejects_empty_feature() {
  let mut log = FrictionLog::new("Test".to_string()).expect("valid log");
  let entry = FrictionEntry {
    feature: "".to_string(),
    description: "Test".to_string(),
    emotional_state: EmotionalState::Neutral,
    timestamp: Utc::now(),
  };
  let result = log.add_entry(entry);
  assert!(matches!(
    result,
    Err(SupportFlywheelError::EmptyFeatureName)
  ));
}

#[test]
fn test_friction_log_analyze_by_emotion() {
  let mut log = FrictionLog::new("Test".to_string()).expect("valid log");
  log
    .add_entry(FrictionEntry {
      feature: "A".to_string(),
      description: "Test".to_string(),
      emotional_state: EmotionalState::Frustrated,
      timestamp: Utc::now(),
    })
    .expect("entry");
  log
    .add_entry(FrictionEntry {
      feature: "B".to_string(),
      description: "Test".to_string(),
      emotional_state: EmotionalState::Frustrated,
      timestamp: Utc::now(),
    })
    .expect("entry");
  log
    .add_entry(FrictionEntry {
      feature: "C".to_string(),
      description: "Test".to_string(),
      emotional_state: EmotionalState::Pleased,
      timestamp: Utc::now(),
    })
    .expect("entry");

  let analysis = log.analyze_emotions();
  assert_eq!(analysis.get(&EmotionalState::Frustrated), Some(&2));
  assert_eq!(analysis.get(&EmotionalState::Pleased), Some(&1));
}

#[test]
fn test_support_ticket_creation() {
  let ticket = create_valid_support_ticket();
  assert_eq!(ticket.id, "TKT-001");
  assert!(matches!(ticket.status, TicketStatus::Open));
}

#[test]
fn test_support_ticket_rejects_empty_id() {
  let result = SupportTicket::new(
    "".to_string(),
    "Title".to_string(),
    "Description".to_string(),
    TicketSeverity::Medium,
  );
  assert!(matches!(result, Err(SupportFlywheelError::EmptyTicketId)));
}

#[test]
fn test_support_ticket_rejects_empty_title() {
  let result = SupportTicket::new(
    "TKT-001".to_string(),
    "".to_string(),
    "Description".to_string(),
    TicketSeverity::Medium,
  );
  assert!(matches!(
    result,
    Err(SupportFlywheelError::EmptyTicketTitle)
  ));
}

#[test]
fn test_use_case_link_creation() {
  let result = UseCaseLink::new("TKT-001".to_string(), "Password Reset".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_use_case_link_rejects_empty_ticket() {
  let result = UseCaseLink::new("".to_string(), "Use Case".to_string());
  assert!(matches!(result, Err(SupportFlywheelError::EmptyTicketId)));
}

#[test]
fn test_use_case_link_rejects_empty_use_case() {
  let result = UseCaseLink::new("TKT-001".to_string(), "".to_string());
  assert!(matches!(
    result,
    Err(SupportFlywheelError::EmptyUseCaseName)
  ));
}

#[test]
fn test_support_flywheel_creation() {
  let result = SupportFlywheel::new("Product Feedback Loop".to_string());
  assert!(result.is_ok());
}

#[test]
fn test_support_flywheel_rejects_empty_name() {
  let result = SupportFlywheel::new("".to_string());
  assert!(matches!(
    result,
    Err(SupportFlywheelError::EmptyFlywheelName)
  ));
}

#[test]
fn test_support_flywheel_add_friction_log() {
  let mut flywheel = SupportFlywheel::new("Test".to_string()).expect("valid");
  let log = FrictionLog::new("Sprint 1".to_string()).expect("valid log");
  flywheel.add_friction_log(log);
  assert_eq!(flywheel.friction_logs.len(), 1);
}

#[test]
fn test_support_flywheel_add_ticket() {
  let mut flywheel = SupportFlywheel::new("Test".to_string()).expect("valid");
  let ticket = create_valid_support_ticket();
  flywheel.add_ticket(ticket);
  assert_eq!(flywheel.tickets.len(), 1);
}

#[test]
fn test_support_flywheel_link_ticket_to_use_case() {
  let mut flywheel = SupportFlywheel::new("Test".to_string()).expect("valid");
  flywheel.add_ticket(create_valid_support_ticket());

  let result = flywheel.link_ticket_to_use_case("TKT-001", "Password Management");
  assert!(result.is_ok());
  assert_eq!(flywheel.use_case_links.len(), 1);
}

#[test]
fn test_support_flywheel_link_nonexistent_ticket_fails() {
  let mut flywheel = SupportFlywheel::new("Test".to_string()).expect("valid");
  let result = flywheel.link_ticket_to_use_case("NONEXISTENT", "Use Case");
  assert!(matches!(result, Err(SupportFlywheelError::TicketNotFound)));
}

#[test]
fn test_support_flywheel_generate_product_insights() {
  let mut flywheel = SupportFlywheel::new("Test".to_string()).expect("valid");

  let mut log = FrictionLog::new("Sprint 1".to_string()).expect("log");
  log.add_entry(create_valid_friction_entry()).expect("entry");
  flywheel.add_friction_log(log);
  flywheel.add_ticket(create_valid_support_ticket());
  flywheel
    .link_ticket_to_use_case("TKT-001", "Auth Flow")
    .expect("link");

  let insights = flywheel.generate_insights();
  assert!(!insights.friction_count.is_empty());
  assert!(!insights.use_case_coverage.is_empty());
}

#[test]
fn test_ticket_severity_variants() {
  assert!(matches!(TicketSeverity::Low, TicketSeverity::Low));
  assert!(matches!(TicketSeverity::Medium, TicketSeverity::Medium));
  assert!(matches!(TicketSeverity::High, TicketSeverity::High));
  assert!(matches!(TicketSeverity::Critical, TicketSeverity::Critical));
}

#[test]
fn test_ticket_status_variants() {
  assert!(matches!(TicketStatus::Open, TicketStatus::Open));
  assert!(matches!(TicketStatus::InProgress, TicketStatus::InProgress));
  assert!(matches!(TicketStatus::Resolved, TicketStatus::Resolved));
  assert!(matches!(TicketStatus::Closed, TicketStatus::Closed));
}
