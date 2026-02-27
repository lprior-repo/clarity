#![allow(clippy::all)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::needless_collect)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::single_match_else)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(unused_comparisons)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::assertions_on_constants)]
// Integration test for ConfidenceBadge component
// This test verifies the component is properly exported and can be used
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

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
