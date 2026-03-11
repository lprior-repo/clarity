#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Tests for planning_coach component utilities

use clarity_web::components::planning_coach::truncate;

#[test]
fn test_truncate_multi_byte_utf8_preserves_char_boundary() {
  // Japanese chars are 3 bytes each: 日(3) 本(3) 語(3)
  let input = "日本語テスト";
  // Truncating at byte 5 would slice mid-character
  let result = truncate(input, 5);
  // Must end on valid char boundary
  assert!(result.is_char_boundary(result.len()));
  // Should truncate to "日" (3 bytes), not panic
  assert_eq!(result, "日");
}
