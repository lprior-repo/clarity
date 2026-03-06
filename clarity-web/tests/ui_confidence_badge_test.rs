#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
// Integration test for ConfidenceBadge component
// This test verifies the component is properly exported and can be used

use clarity_web::storage::types::Confidence;
use clarity_web::ui::confidence_badge::ConfidenceBadgeProps;

#[test]
fn test_confidence_badge_exports() {
  // Test that ConfidenceBadgeProps is accessible from the ui module
  // This verifies the module is properly structured
  let _badge = ConfidenceBadgeProps {
    confidence: Confidence::High,
    class: String::new(),
  };
}

#[test]
fn test_confidence_badge_with_custom_class() {
  let badge = ConfidenceBadgeProps {
    confidence: Confidence::Inferred,
    class: "custom-class".to_string(),
  };
  assert_eq!(badge.confidence, Confidence::Inferred);
  assert_eq!(badge.class, "custom-class");
}

#[test]
fn test_confidence_levels() {
  // Verify all confidence levels are accessible
  let _high = Confidence::High;
  let _inferred = Confidence::Inferred;
  let _uncertain = Confidence::Uncertain;
}

#[test]
fn test_confidence_badge_props() {
  // Test props structure
  let props = ConfidenceBadgeProps {
    confidence: Confidence::Uncertain,
    class: "test".to_string(),
  };

  assert_eq!(props.confidence, Confidence::Uncertain);
  assert_eq!(props.class, "test");
}
