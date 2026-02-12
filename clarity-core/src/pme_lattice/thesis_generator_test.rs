//! Tests for Thesis & Antithesis Generator
//!
//! Test quality doesn't matter - we test source code quality.

#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]

use super::thesis_generator::{ThesisAntithesisError, ThesisAntithesisGenerator};

#[test]
fn thesis_generator_new_requires_non_empty_thesis() {
  let result = ThesisAntithesisGenerator::new("", "Users don't want X");
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyThesis)));
}

#[test]
fn thesis_generator_new_requires_non_whitespace_thesis() {
  let result = ThesisAntithesisGenerator::new("   ", "Users don't want X");
  assert!(matches!(result, Err(ThesisAntithesisError::EmptyThesis)));
}

#[test]
fn thesis_generator_new_requires_non_empty_antithesis() {
  let result = ThesisAntithesisGenerator::new("Users want X", "");
  assert!(matches!(
    result,
    Err(ThesisAntithesisError::EmptyAntithesis)
  ));
}

#[test]
fn thesis_generator_new_requires_non_whitespace_antithesis() {
  let result = ThesisAntithesisGenerator::new("Users want X", "   ");
  assert!(matches!(
    result,
    Err(ThesisAntithesisError::EmptyAntithesis)
  ));
}

#[test]
fn thesis_generator_rejects_identical_thesis_and_antithesis() {
  let result = ThesisAntithesisGenerator::new("Users want X", "Users want X");
  assert!(matches!(
    result,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn thesis_generator_rejects_case_insensitive_identical_thesis_and_antithesis() {
  let result = ThesisAntithesisGenerator::new("Users want X", "USERS WANT X");
  assert!(matches!(
    result,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn thesis_generator_rejects_whitespace_normalized_identical() {
  let result = ThesisAntithesisGenerator::new("  Users want X  ", "Users want X");
  assert!(matches!(
    result,
    Err(ThesisAntithesisError::ThesisEqualsAntithesis)
  ));
}

#[test]
fn thesis_generator_new_succeeds_with_valid_input() {
  let result = ThesisAntithesisGenerator::new("Users want X", "Users don't want X");
  assert!(result.is_ok());
  let gen = result.expect("should have generator");
  assert_eq!(gen.thesis(), "Users want X");
  assert_eq!(gen.antithesis(), "Users don't want X");
  assert!(gen.failure_modes().is_empty());
}

#[test]
fn thesis_generator_with_failure_mode_adds_mode() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_failure_mode("Market doesn't exist");

  assert_eq!(gen.failure_modes().len(), 1);
  assert_eq!(gen.failure_modes()[0], "Market doesn't exist");
}

#[test]
fn thesis_generator_with_multiple_failure_modes() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_failure_mode("Market doesn't exist")
    .with_failure_mode("Price too high")
    .with_failure_mode("Competitors are better");

  assert_eq!(gen.failure_modes().len(), 3);
}

#[test]
fn thesis_generator_with_validation_criterion() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_validation_criterion("Interview 10 potential users");

  assert_eq!(gen.validation_criteria().len(), 1);
}

#[test]
fn thesis_generator_is_valid_requires_failure_modes() {
  let gen =
    ThesisAntithesisGenerator::new("Users want X", "Users don't want X").expect("valid input");

  assert!(!gen.is_valid());
  assert!(matches!(
    gen.validate(),
    Err(ThesisAntithesisError::NoFailureModes)
  ));
}

#[test]
fn thesis_generator_is_valid_with_failure_modes() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_failure_mode("Market doesn't exist");

  assert!(gen.is_valid());
  assert!(gen.validate().is_ok());
}

#[test]
fn thesis_generator_generate_antithesis_prompts_returns_helpful_prompts() {
  let prompts = ThesisAntithesisGenerator::generate_antithesis_prompts();

  assert!(!prompts.is_empty());
  assert!(prompts.iter().any(|p| p.contains("fail")));
}

#[test]
fn thesis_generator_has_id() {
  let gen =
    ThesisAntithesisGenerator::new("Users want X", "Users don't want X").expect("valid input");

  assert!(!gen.id().is_nil());
}

#[test]
fn thesis_generator_with_failure_mode_empty_is_ignored() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_failure_mode("")
    .with_failure_mode("Valid mode");

  assert_eq!(gen.failure_modes().len(), 1);
}

#[test]
fn thesis_generator_serialization() {
  let gen = ThesisAntithesisGenerator::new("Users want X", "Users don't want X")
    .expect("valid input")
    .with_failure_mode("Test failure");

  if let Ok(json) = serde_json::to_string(&gen) {
    let parsed: Result<ThesisAntithesisGenerator, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
  }
}

#[test]
fn thesis_generator_error_display() {
  let err = ThesisAntithesisError::EmptyThesis;
  assert!(!err.to_string().is_empty());

  let err = ThesisAntithesisError::EmptyAntithesis;
  assert!(!err.to_string().is_empty());

  let err = ThesisAntithesisError::ThesisEqualsAntithesis;
  assert!(!err.to_string().is_empty());

  let err = ThesisAntithesisError::NoFailureModes;
  assert!(!err.to_string().is_empty());
}
