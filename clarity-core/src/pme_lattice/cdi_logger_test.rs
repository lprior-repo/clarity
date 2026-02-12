//! Tests for CDI (Customer Data Insight) Logger
//!
//! Tests follow TDD approach for bead bd-16qs.4

#![cfg(test)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use chrono::Utc;
use uuid::Uuid;

use super::cdi_logger::{
  calculate_aggregate_strength, AggregateStrengthError, CDIError, CDILogger, CustomerSignal,
  SignalSource, SignalStrength,
};

// ============================================================================
// SIGNAL STRENGTH TESTS
// ============================================================================

#[test]
fn signal_strength_strong_has_correct_value() {
  let strong = SignalStrength::Strong;
  assert!((strong.value() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn signal_strength_medium_has_correct_value() {
  let medium = SignalStrength::Medium;
  assert!((medium.value() - 0.6).abs() < f32::EPSILON);
}

#[test]
fn signal_strength_weak_has_correct_value() {
  let weak = SignalStrength::Weak;
  assert!((weak.value() - 0.3).abs() < f32::EPSILON);
}

#[test]
fn signal_strength_noise_has_correct_value() {
  let noise = SignalStrength::Noise;
  assert!((noise.value() - 0.1).abs() < f32::EPSILON);
}

#[test]
fn signal_strength_from_f32_strong_range() {
  let result = SignalStrength::from_f32(0.85);
  assert_eq!(result, SignalStrength::Strong);

  let result = SignalStrength::from_f32(1.0);
  assert_eq!(result, SignalStrength::Strong);
}

#[test]
fn signal_strength_from_f32_medium_range() {
  let result = SignalStrength::from_f32(0.5);
  assert_eq!(result, SignalStrength::Medium);

  let result = SignalStrength::from_f32(0.84);
  assert_eq!(result, SignalStrength::Medium);
}

#[test]
fn signal_strength_from_f32_weak_range() {
  let result = SignalStrength::from_f32(0.2);
  assert_eq!(result, SignalStrength::Weak);

  let result = SignalStrength::from_f32(0.49);
  assert_eq!(result, SignalStrength::Weak);
}

#[test]
fn signal_strength_from_f32_noise_range() {
  let result = SignalStrength::from_f32(0.0);
  assert_eq!(result, SignalStrength::Noise);

  let result = SignalStrength::from_f32(0.19);
  assert_eq!(result, SignalStrength::Noise);
}

#[test]
fn signal_strength_from_f32_clamps_high_values() {
  let result = SignalStrength::from_f32(1.5);
  assert_eq!(result, SignalStrength::Strong);
}

#[test]
fn signal_strength_from_f32_clamps_negative_values() {
  let result = SignalStrength::from_f32(-0.5);
  assert_eq!(result, SignalStrength::Noise);
}

#[test]
fn signal_strength_display_formats_correctly() {
  assert_eq!(format!("{}", SignalStrength::Strong), "Strong");
  assert_eq!(format!("{}", SignalStrength::Medium), "Medium");
  assert_eq!(format!("{}", SignalStrength::Weak), "Weak");
  assert_eq!(format!("{}", SignalStrength::Noise), "Noise");
}

// ============================================================================
// CUSTOMER SIGNAL TESTS
// ============================================================================

#[test]
fn customer_signal_new_creates_valid_signal() {
  let before = Utc::now();
  let signal = CustomerSignal::new(
    "Customers mention pricing frequently in interviews".to_string(),
    SignalStrength::Strong,
    SignalSource::Interview,
  )
  .expect("signal creation should succeed");
  let after = Utc::now();

  assert!(!signal.id.is_nil());
  assert_eq!(
    signal.insight,
    "Customers mention pricing frequently in interviews"
  );
  assert_eq!(signal.signal_strength, SignalStrength::Strong);
  assert_eq!(signal.source, SignalSource::Interview);
  assert!(signal.captured_at >= before);
  assert!(signal.captured_at <= after);
}

#[test]
fn customer_signal_new_rejects_empty_insight() {
  let result = CustomerSignal::new("".to_string(), SignalStrength::Medium, SignalSource::Survey);
  assert!(matches!(result, Err(CDIError::EmptyInsight)));

  let result = CustomerSignal::new(
    "   ".to_string(),
    SignalStrength::Medium,
    SignalSource::Survey,
  );
  assert!(matches!(result, Err(CDIError::EmptyInsight)));
}

#[test]
fn customer_signal_with_source_detail_adds_context() {
  let signal = CustomerSignal::new(
    "Users struggle with onboarding".to_string(),
    SignalStrength::Strong,
    SignalSource::Analytics,
  )
  .expect("signal creation should succeed");

  let signal_with_detail = signal.with_source_detail("Drop-off at step 3: 67%".to_string());
  assert_eq!(
    signal_with_detail.source_detail,
    Some("Drop-off at step 3: 67%".to_string())
  );
}

#[test]
fn customer_signal_with_tag_adds_category() {
  let signal = CustomerSignal::new(
    "Feature X is highly requested".to_string(),
    SignalStrength::Medium,
    SignalSource::SupportTicket,
  )
  .expect("signal creation should succeed");

  let signal_with_tag = signal.with_tag("feature-request".to_string());
  assert!(signal_with_tag
    .tags
    .contains(&"feature-request".to_string()));
}

#[test]
fn customer_signal_tags_are_unique() {
  let signal = CustomerSignal::new(
    "Test insight".to_string(),
    SignalStrength::Weak,
    SignalSource::SocialMedia,
  )
  .expect("signal creation should succeed")
  .with_tag("duplicate".to_string())
  .with_tag("duplicate".to_string());

  assert_eq!(signal.tags.len(), 1);
}

// ============================================================================
// CDI LOGGER TESTS
// ============================================================================

#[test]
fn cdi_logger_new_creates_empty_logger() {
  let logger = CDILogger::new();
  assert!(logger.is_empty());
  assert_eq!(logger.signal_count(), 0);
}

#[test]
fn cdi_logger_add_signal_increases_count() {
  let logger = CDILogger::new();

  let signal = CustomerSignal::new(
    "Test insight".to_string(),
    SignalStrength::Medium,
    SignalSource::Interview,
  )
  .expect("signal creation should succeed");

  let updated = logger.add_signal(signal);
  assert!(!updated.is_empty());
  assert_eq!(updated.signal_count(), 1);
}

#[test]
fn cdi_logger_is_immutably_persistent() {
  let logger = CDILogger::new();

  let signal1 = CustomerSignal::new(
    "First insight".to_string(),
    SignalStrength::Strong,
    SignalSource::Analytics,
  )
  .expect("signal1 creation should succeed");

  let signal2 = CustomerSignal::new(
    "Second insight".to_string(),
    SignalStrength::Weak,
    SignalSource::Survey,
  )
  .expect("signal2 creation should succeed");

  // Clone logger to preserve original for assertion
  let original = logger.clone();
  let updated_once = logger.add_signal(signal1);
  let updated_twice = updated_once.clone().add_signal(signal2);

  // Original logger is unchanged
  assert!(original.is_empty());
  // First update is unchanged
  assert_eq!(updated_once.signal_count(), 1);
  // Second update has both
  assert_eq!(updated_twice.signal_count(), 2);
}

#[test]
fn cdi_logger_get_strong_signals_filters_correctly() {
  let logger = CDILogger::new()
    .add_signal(
      CustomerSignal::new(
        "Strong 1".to_string(),
        SignalStrength::Strong,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Medium 1".to_string(),
        SignalStrength::Medium,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Strong 2".to_string(),
        SignalStrength::Strong,
        SignalSource::Survey,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Weak 1".to_string(),
        SignalStrength::Weak,
        SignalSource::Analytics,
      )
      .expect("signal creation should succeed"),
    );

  let strong_signals = logger.get_strong_signals();
  assert_eq!(strong_signals.len(), 2);
  assert!(strong_signals
    .iter()
    .all(|s| s.signal_strength == SignalStrength::Strong));
}

#[test]
fn cdi_logger_get_signals_by_source_filters_correctly() {
  let logger = CDILogger::new()
    .add_signal(
      CustomerSignal::new(
        "Interview 1".to_string(),
        SignalStrength::Strong,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Survey 1".to_string(),
        SignalStrength::Medium,
        SignalSource::Survey,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Interview 2".to_string(),
        SignalStrength::Weak,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    );

  let interview_signals = logger.get_signals_by_source(SignalSource::Interview);
  assert_eq!(interview_signals.len(), 2);

  let survey_signals = logger.get_signals_by_source(SignalSource::Survey);
  assert_eq!(survey_signals.len(), 1);

  let analytics_signals = logger.get_signals_by_source(SignalSource::Analytics);
  assert!(analytics_signals.is_empty());
}

#[test]
fn cdi_logger_get_signals_above_threshold_filters_correctly() {
  let logger = CDILogger::new()
    .add_signal(
      CustomerSignal::new(
        "Strong".to_string(),
        SignalStrength::Strong,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Medium".to_string(),
        SignalStrength::Medium,
        SignalSource::Survey,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Weak".to_string(),
        SignalStrength::Weak,
        SignalSource::Analytics,
      )
      .expect("signal creation should succeed"),
    )
    .add_signal(
      CustomerSignal::new(
        "Noise".to_string(),
        SignalStrength::Noise,
        SignalSource::SocialMedia,
      )
      .expect("signal creation should succeed"),
    );

  // Threshold at 0.5 should include Strong (1.0) and Medium (0.6)
  let above_threshold = logger.get_signals_above_threshold(0.5);
  assert_eq!(above_threshold.len(), 2);

  // Threshold at 0.7 should only include Strong (1.0)
  let high_threshold = logger.get_signals_above_threshold(0.7);
  assert_eq!(high_threshold.len(), 1);
}

#[test]
fn cdi_logger_get_signal_by_id_finds_signal() {
  let signal = CustomerSignal::new(
    "Test insight".to_string(),
    SignalStrength::Strong,
    SignalSource::Interview,
  )
  .expect("signal creation should succeed");

  let id = signal.id;
  let logger = CDILogger::new().add_signal(signal);

  let found = logger.get_signal_by_id(id);
  assert!(found.is_some());
  assert_eq!(found.map(|s| &s.insight), Some(&"Test insight".to_string()));

  let not_found = logger.get_signal_by_id(Uuid::nil());
  assert!(not_found.is_none());
}

// ============================================================================
// AGGREGATE STRENGTH TESTS
// ============================================================================

#[test]
fn calculate_aggregate_strength_empty_returns_error() {
  let logger = CDILogger::new();
  let result = calculate_aggregate_strength(&logger);

  assert!(matches!(result, Err(AggregateStrengthError::NoSignals)));
}

#[test]
fn calculate_aggregate_strength_single_signal() {
  let logger = CDILogger::new().add_signal(
    CustomerSignal::new(
      "Test".to_string(),
      SignalStrength::Strong,
      SignalSource::Interview,
    )
    .expect("signal creation should succeed"),
  );

  let result = calculate_aggregate_strength(&logger);
  assert!(result.is_ok());
  assert!((result.expect("should have value") - 1.0).abs() < f32::EPSILON);
}

#[test]
fn calculate_aggregate_strength_averages_multiple_signals() {
  let logger = CDILogger::new()
    .add_signal(
      CustomerSignal::new(
        "Strong".to_string(),
        SignalStrength::Strong,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    ) // 1.0
    .add_signal(
      CustomerSignal::new(
        "Medium".to_string(),
        SignalStrength::Medium,
        SignalSource::Survey,
      )
      .expect("signal creation should succeed"),
    ); // 0.6

  // Average: (1.0 + 0.6) / 2 = 0.8
  let result = calculate_aggregate_strength(&logger);
  assert!(result.is_ok());
  let aggregate = result.expect("should have value");
  assert!((aggregate - 0.8).abs() < f32::EPSILON);
}

#[test]
fn calculate_aggregate_strength_with_various_signals() {
  let logger = CDILogger::new()
    .add_signal(
      CustomerSignal::new(
        "S1".to_string(),
        SignalStrength::Strong,
        SignalSource::Interview,
      )
      .expect("signal creation should succeed"),
    ) // 1.0
    .add_signal(
      CustomerSignal::new(
        "S2".to_string(),
        SignalStrength::Medium,
        SignalSource::Survey,
      )
      .expect("signal creation should succeed"),
    ) // 0.6
    .add_signal(
      CustomerSignal::new(
        "S3".to_string(),
        SignalStrength::Weak,
        SignalSource::Analytics,
      )
      .expect("signal creation should succeed"),
    ) // 0.3
    .add_signal(
      CustomerSignal::new(
        "S4".to_string(),
        SignalStrength::Noise,
        SignalSource::SocialMedia,
      )
      .expect("signal creation should succeed"),
    ); // 0.1

  // Average: (1.0 + 0.6 + 0.3 + 0.1) / 4 = 0.5
  let result = calculate_aggregate_strength(&logger);
  assert!(result.is_ok());
  let aggregate = result.expect("should have value");
  assert!((aggregate - 0.5).abs() < f32::EPSILON);
}

// ============================================================================
// SIGNAL SOURCE TESTS
// ============================================================================

#[test]
fn signal_source_display_formats_correctly() {
  assert_eq!(format!("{}", SignalSource::Interview), "Interview");
  assert_eq!(format!("{}", SignalSource::Survey), "Survey");
  assert_eq!(format!("{}", SignalSource::Analytics), "Analytics");
  assert_eq!(format!("{}", SignalSource::SupportTicket), "Support Ticket");
  assert_eq!(format!("{}", SignalSource::SocialMedia), "Social Media");
  assert_eq!(format!("{}", SignalSource::SalesCall), "Sales Call");
  assert_eq!(format!("{}", SignalSource::Other), "Other");
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn signal_strength_serializes_correctly() {
  let strong = SignalStrength::Strong;
  let json = serde_json::to_string(&strong).expect("serialization should succeed");
  assert_eq!(json, r#""strong""#);
}

#[test]
fn signal_strength_deserializes_correctly() {
  let json = r#""medium""#;
  let strength: SignalStrength =
    serde_json::from_str(json).expect("deserialization should succeed");
  assert_eq!(strength, SignalStrength::Medium);
}

#[test]
fn customer_signal_serializes_correctly() {
  let signal = CustomerSignal::new(
    "Test insight".to_string(),
    SignalStrength::Strong,
    SignalSource::Interview,
  )
  .expect("signal creation should succeed");

  let json = serde_json::to_string(&signal).expect("serialization should succeed");
  assert!(json.contains(r#""insight":"Test insight""#));
  assert!(json.contains(r#""signal_strength":"strong""#));
  assert!(json.contains(r#""source":"interview""#));
}
