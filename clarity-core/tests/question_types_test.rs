//! Question Types Tests
//!
//! These tests verify the QuestionType enum and its variants.
//!
//! See docs/TESTING.md for testing standards.

use clarity_core::types::question::{QuestionType, QuestionTypeError};

// 1. test_question_type_should_create_text_question_successfully
#[test]
fn test_question_type_should_create_text_question_successfully() {
  let result = QuestionType::text("What is your name?", None);
  assert!(result.is_ok(), "Should create text question successfully");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "What is your name?");
  assert!(question.validate().is_ok(), "Validation should pass");

  // Test serialization
  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 2. test_question_type_should_create_multiple_choice_question
#[test]
fn test_question_type_should_create_multiple_choice_question() {
  let result = QuestionType::multiple_choice(
    "Choose one",
    vec!["A".to_string(), "B".to_string(), "C".to_string()],
    Some(0),
  );
  assert!(result.is_ok(), "Should create multiple choice question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Choose one");
  assert!(question.validate().is_ok(), "Validation should pass");

  // Test serialization
  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 3. test_question_type_should_validate_required_fields
#[test]
fn test_question_type_should_validate_required_fields() {
  // Test with empty prompt
  let result = QuestionType::text("", None);
  assert!(result.is_err(), "Should reject empty prompt");

  match result {
    Err(QuestionTypeError::MissingField { field }) => {
      assert_eq!(field, "prompt", "Error should indicate prompt is missing");
    }
    _ => panic!("Expected MissingField error for prompt"),
  }
}

// 4. test_question_type_should_reject_empty_multiple_choice_options
#[test]
fn test_question_type_should_reject_empty_multiple_choice_options() {
  let result = QuestionType::multiple_choice("Choose", vec![], None);
  assert!(result.is_err(), "Should reject empty options");

  match result {
    Err(QuestionTypeError::Validation { reason }) => {
      assert!(reason.contains("options"), "Error should mention options");
      assert!(
        reason.contains("empty"),
        "Error should indicate cannot be empty"
      );
    }
    _ => panic!("Expected Validation error for empty options"),
  }
}

// 5. test_question_type_should_reject_invalid_default_index
#[test]
fn test_question_type_should_reject_invalid_default_index() {
  let result =
    QuestionType::multiple_choice("Choose", vec!["A".to_string(), "B".to_string()], Some(5));
  assert!(result.is_err(), "Should reject out-of-bounds default index");

  match result {
    Err(QuestionTypeError::Validation { reason }) => {
      assert!(reason.contains("index"), "Error should mention index");
      assert!(
        reason.contains("out of bounds") || reason.contains("range"),
        "Error should indicate out of bounds"
      );
    }
    _ => panic!("Expected Validation error for invalid default index"),
  }
}

// 6. test_question_type_should_serialize_to_json
#[test]
fn test_question_type_should_serialize_to_json() {
  let questions = vec![
    QuestionType::text("Text question", None).unwrap(),
    QuestionType::multiple_choice(
      "MC question",
      vec!["A".to_string(), "B".to_string()],
      Some(0),
    )
    .unwrap(),
    QuestionType::boolean("Boolean question", None).unwrap(),
  ];

  for question in questions {
    let json = serde_json::to_string(&question);
    assert!(json.is_ok(), "Should serialize to JSON");

    let json_str = json.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_object(), "JSON should be an object");
    assert!(parsed.get("type").is_some(), "JSON should have type field");
    assert!(
      parsed.get("prompt").is_some(),
      "JSON should have prompt field"
    );
  }
}

// 7. test_question_type_should_deserialize_from_json
#[test]
fn test_question_type_should_deserialize_from_json() {
  let original = QuestionType::text("Test question", None).unwrap();
  let json = serde_json::to_string(&original).unwrap();

  let deserialized: Result<QuestionType, _> = serde_json::from_str(&json);
  assert!(deserialized.is_ok(), "Should deserialize from JSON");

  let parsed = deserialized.unwrap();
  assert_eq!(
    parsed, original,
    "Deserialized question should match original"
  );
}

// 8. test_question_type_should_support_boolean_question
#[test]
fn test_question_type_should_support_boolean_question() {
  let result = QuestionType::boolean("Do you agree?", Some(true));
  assert!(result.is_ok(), "Should create boolean question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Do you agree?");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 9. test_question_type_should_support_numeric_range_question
#[test]
fn test_question_type_should_support_numeric_range_question() {
  let result = QuestionType::numeric_range("Rate 1-5", 1, 5, Some(3));
  assert!(result.is_ok(), "Should create numeric range question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Rate 1-5");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 10. test_question_type_should_reject_invalid_range
#[test]
fn test_question_type_should_reject_invalid_range() {
  let result = QuestionType::numeric_range("Rate", 10, 1, Some(5));
  assert!(result.is_err(), "Should reject min > max");

  match result {
    Err(QuestionTypeError::Validation { reason }) => {
      assert!(
        reason.contains("min") || reason.contains("max"),
        "Error should mention min/max"
      );
      assert!(
        reason.contains("range") || reason.contains("greater"),
        "Error should indicate range issue"
      );
    }
    _ => panic!("Expected Validation error for invalid range"),
  }
}

// 11. test_question_type_should_support_date_question
#[test]
fn test_question_type_should_support_date_question() {
  let result = QuestionType::date("When is your birthday?", None);
  assert!(result.is_ok(), "Should create date question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "When is your birthday?");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 12. test_question_type_should_support_long_text_question
#[test]
fn test_question_type_should_support_long_text_question() {
  let result = QuestionType::long_text("Describe your experience", None, 500);
  assert!(result.is_ok(), "Should create long text question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Describe your experience");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 13. test_question_type_should_enforce_max_length
#[test]
fn test_question_type_should_enforce_max_length() {
  let question = QuestionType::long_text("Description", None, 10).unwrap();

  // Test validation with answer exceeding max length
  let long_answer = "This is way too long for the limit";
  let result = question.validate_answer(long_answer);
  assert!(result.is_err(), "Should reject answer exceeding max length");

  match result {
    Err(QuestionTypeError::Validation { reason }) => {
      assert!(
        reason.contains("length") || reason.contains("exceeds"),
        "Error should mention length"
      );
    }
    _ => panic!("Expected Validation error for exceeded max length"),
  }
}

// 14. test_question_type_should_support_rating_scale_question
#[test]
fn test_question_type_should_support_rating_scale_question() {
  let result = QuestionType::rating("Rate satisfaction", 1, 5);
  assert!(result.is_ok(), "Should create rating question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Rate satisfaction");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 15. test_question_type_should_display_human_readable_prompt
#[test]
fn test_question_type_should_display_human_readable_prompt() {
  let questions = vec![
    QuestionType::text("Text prompt", None).unwrap(),
    QuestionType::multiple_choice("MC prompt", vec!["A".to_string(), "B".to_string()], Some(0))
      .unwrap(),
    QuestionType::boolean("Boolean prompt", None).unwrap(),
    QuestionType::numeric_range("Range prompt", 1, 5, Some(3)).unwrap(),
  ];

  for question in questions {
    let display = question.display_prompt();
    assert!(!display.is_empty(), "Display prompt should not be empty");
    assert!(
      display.contains("prompt"),
      "Display should include prompt text"
    );
    assert!(
      display.contains('['),
      "Display should include type indicator in brackets"
    );
  }
}

// 16. test_question_type_should_support_open_ended_code_question
#[test]
fn test_question_type_should_support_open_ended_code_question() {
  let result = QuestionType::code("Write a function", "python", None);
  assert!(result.is_ok(), "Should create code question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Write a function");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");

  let parsed: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();
  assert_eq!(
    parsed["language"], "python",
    "JSON should include language field"
  );
}

// 17. test_question_type_should_be_equality_comparable
#[test]
fn test_question_type_should_be_equality_comparable() {
  let q1 = QuestionType::text("Same prompt", None).unwrap();
  let q2 = QuestionType::text("Same prompt", None).unwrap();
  let q3 = QuestionType::text("Different prompt", None).unwrap();

  assert_eq!(q1, q2, "Questions with same data should be equal");
  assert_ne!(q1, q3, "Questions with different data should not be equal");
}

// 18. test_question_type_should_support_file_upload_question
#[test]
fn test_question_type_should_support_file_upload_question() {
  let result = QuestionType::file_upload(
    "Attach resume",
    vec!["pdf".to_string(), "docx".to_string()],
    false,
  );
  assert!(result.is_ok(), "Should create file upload question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Attach resume");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");

  let parsed: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();
  assert!(
    parsed["allowed_types"].is_array(),
    "JSON should include allowed types array"
  );
}

// 19. test_question_type_should_support_ranking_question
#[test]
fn test_question_type_should_support_ranking_question() {
  let result = QuestionType::ranking(
    "Rank preferences",
    vec!["A".to_string(), "B".to_string(), "C".to_string()],
  );
  assert!(result.is_ok(), "Should create ranking question");

  let question = result.unwrap();
  assert_eq!(question.prompt(), "Rank preferences");
  assert!(question.validate().is_ok(), "Validation should pass");

  let json = serde_json::to_string(&question);
  assert!(json.is_ok(), "Should serialize to JSON");
}

// 20. test_question_type_should_reject_duplicate_options_in_multiple_choice
#[test]
fn test_question_type_should_reject_duplicate_options_in_multiple_choice() {
  let result = QuestionType::multiple_choice(
    "Choose",
    vec!["A".to_string(), "A".to_string(), "B".to_string()],
    None,
  );
  assert!(result.is_err(), "Should reject duplicate options");

  match result {
    Err(QuestionTypeError::Validation { reason }) => {
      assert!(
        reason.contains("duplicate") || reason.contains("unique"),
        "Error should mention duplicates"
      );
    }
    _ => panic!("Expected Validation error for duplicate options"),
  }
}

// Edge case: Empty prompt
#[test]
fn test_question_type_should_reject_empty_prompt() {
  let result = QuestionType::text("   ", None);
  assert!(result.is_err(), "Should reject whitespace-only prompt");
}

// Edge case: Very long prompt (1000+ chars)
#[test]
fn test_question_type_should_support_long_prompt() {
  let long_prompt = "A".repeat(1000);
  let result = QuestionType::text(&long_prompt, None);
  assert!(result.is_ok(), "Should support long prompts");
}

// Edge case: Unicode in prompts
#[test]
fn test_question_type_should_support_unicode_in_prompts() {
  let unicode_prompt = "¿Cómo te llamas? 日本語";
  let result = QuestionType::text(unicode_prompt, None);
  assert!(result.is_ok(), "Should support unicode in prompts");
}

// Edge case: Single option in multiple choice
#[test]
fn test_question_type_should_support_single_option_multiple_choice() {
  let result = QuestionType::multiple_choice("Only one", vec!["A".to_string()], Some(0));
  assert!(result.is_ok(), "Should support single option");
}

// Edge case: Negative numbers in ranges
#[test]
fn test_question_type_should_support_negative_range() {
  let result = QuestionType::numeric_range("Temperature", -10, 10, Some(0));
  assert!(result.is_ok(), "Should support negative ranges");
}

// Edge case: Zero in numeric ranges
#[test]
fn test_question_type_should_support_zero_in_range() {
  let result = QuestionType::numeric_range("Rating", 0, 5, Some(3));
  assert!(result.is_ok(), "Should support zero in range");
}
