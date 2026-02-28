//! Question Types
//!
//! Core types for interview questions.
//! Used by both `interview_questions` and `question_loader` modules.
//!
//! Ported from intent-cli/src/intent/question_types.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Perspective from which a question is asked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum QuestionPerspective {
  #[default]
  User,
  Developer,
  Ops,
  Security,
  Business,
}

impl QuestionPerspective {
  /// Convert to string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::User => "user",
      Self::Developer => "developer",
      Self::Ops => "ops",
      Self::Security => "security",
      Self::Business => "business",
    }
  }

  /// Parse from string
  ///
  /// # Errors
  /// Returns `QuestionParseError` if the input is not a valid perspective.
  pub fn parse(s: &str) -> Result<Self, QuestionParseError> {
    match s.to_lowercase().as_str() {
      "user" => Ok(Self::User),
      "developer" => Ok(Self::Developer),
      "ops" => Ok(Self::Ops),
      "security" => Ok(Self::Security),
      "business" => Ok(Self::Business),
      _ => Err(QuestionParseError::InvalidPerspective(s.to_string())),
    }
  }
}

/// Category of question
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategoryType {
  #[default]
  HappyPath,
  ErrorCase,
  EdgeCase,
  Constraint,
  Dependency,
  NonFunctional,
}

impl QuestionCategoryType {
  /// Convert to string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::HappyPath => "happy_path",
      Self::ErrorCase => "error_case",
      Self::EdgeCase => "edge_case",
      Self::Constraint => "constraint",
      Self::Dependency => "dependency",
      Self::NonFunctional => "non_functional",
    }
  }

  /// Parse from string
  ///
  /// # Errors
  /// Returns `QuestionParseError` if the input is not a valid category.
  pub fn parse(s: &str) -> Result<Self, QuestionParseError> {
    match s.to_lowercase().as_str() {
      "happy_path" => Ok(Self::HappyPath),
      "error_case" => Ok(Self::ErrorCase),
      "edge_case" => Ok(Self::EdgeCase),
      "constraint" => Ok(Self::Constraint),
      "dependency" => Ok(Self::Dependency),
      "non_functional" | "nonfunctional" => Ok(Self::NonFunctional),
      _ => Err(QuestionParseError::InvalidCategory(s.to_string())),
    }
  }
}

/// Priority of a question
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuestionPriorityType {
  Critical,
  #[default]
  Important,
  NiceToHave,
}

impl QuestionPriorityType {
  /// Convert to string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Critical => "critical",
      Self::Important => "important",
      Self::NiceToHave => "nice_to_have",
    }
  }

  /// Parse from string
  ///
  /// # Errors
  /// Returns `QuestionParseError` if the input is not a valid priority.
  pub fn parse(s: &str) -> Result<Self, QuestionParseError> {
    match s.to_lowercase().as_str() {
      "critical" => Ok(Self::Critical),
      "important" => Ok(Self::Important),
      "nice_to_have" | "nicetohave" => Ok(Self::NiceToHave),
      _ => Err(QuestionParseError::InvalidPriority(s.to_string())),
    }
  }
}

/// Errors during question parsing
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QuestionParseError {
  #[error("invalid perspective: {0}")]
  InvalidPerspective(String),

  #[error("invalid category: {0}")]
  InvalidCategory(String),

  #[error("invalid priority: {0}")]
  InvalidPriority(String),
}

impl FromStr for QuestionPerspective {
  type Err = QuestionParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl FromStr for QuestionCategoryType {
  type Err = QuestionParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl FromStr for QuestionPriorityType {
  type Err = QuestionParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_perspective_as_str() {
    assert_eq!(QuestionPerspective::User.as_str(), "user");
    assert_eq!(QuestionPerspective::Developer.as_str(), "developer");
    assert_eq!(QuestionPerspective::Ops.as_str(), "ops");
    assert_eq!(QuestionPerspective::Security.as_str(), "security");
    assert_eq!(QuestionPerspective::Business.as_str(), "business");
  }

  #[test]
  fn test_perspective_from_str() {
    assert_eq!(
      QuestionPerspective::parse("user"),
      Ok(QuestionPerspective::User)
    );
    assert_eq!(
      QuestionPerspective::parse("DEVELOPER"),
      Ok(QuestionPerspective::Developer)
    );
    assert!(QuestionPerspective::parse("invalid").is_err());
  }

  #[test]
  fn test_category_as_str() {
    assert_eq!(QuestionCategoryType::HappyPath.as_str(), "happy_path");
    assert_eq!(QuestionCategoryType::ErrorCase.as_str(), "error_case");
    assert_eq!(QuestionCategoryType::EdgeCase.as_str(), "edge_case");
    assert_eq!(QuestionCategoryType::Constraint.as_str(), "constraint");
    assert_eq!(QuestionCategoryType::Dependency.as_str(), "dependency");
    assert_eq!(
      QuestionCategoryType::NonFunctional.as_str(),
      "non_functional"
    );
  }

  #[test]
  fn test_category_from_str() {
    assert_eq!(
      QuestionCategoryType::parse("happy_path"),
      Ok(QuestionCategoryType::HappyPath)
    );
    assert_eq!(
      QuestionCategoryType::parse("ERROR_CASE"),
      Ok(QuestionCategoryType::ErrorCase)
    );
    assert_eq!(
      QuestionCategoryType::parse("nonfunctional"),
      Ok(QuestionCategoryType::NonFunctional)
    );
    assert!(QuestionCategoryType::parse("invalid").is_err());
  }

  #[test]
  fn test_priority_as_str() {
    assert_eq!(QuestionPriorityType::Critical.as_str(), "critical");
    assert_eq!(QuestionPriorityType::Important.as_str(), "important");
    assert_eq!(QuestionPriorityType::NiceToHave.as_str(), "nice_to_have");
  }

  #[test]
  fn test_priority_from_str() {
    assert_eq!(
      QuestionPriorityType::parse("critical"),
      Ok(QuestionPriorityType::Critical)
    );
    assert_eq!(
      QuestionPriorityType::parse("IMPORTANT"),
      Ok(QuestionPriorityType::Important)
    );
    assert_eq!(
      QuestionPriorityType::parse("nice_to_have"),
      Ok(QuestionPriorityType::NiceToHave)
    );
    assert_eq!(
      QuestionPriorityType::parse("nicetohave"),
      Ok(QuestionPriorityType::NiceToHave)
    );
    assert!(QuestionPriorityType::parse("invalid").is_err());
  }

  #[test]
  fn test_defaults() {
    assert_eq!(QuestionPerspective::default(), QuestionPerspective::User);
    assert_eq!(
      QuestionCategoryType::default(),
      QuestionCategoryType::HappyPath
    );
    assert_eq!(
      QuestionPriorityType::default(),
      QuestionPriorityType::Important
    );
  }
}
