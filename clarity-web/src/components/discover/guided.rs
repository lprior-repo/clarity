#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Represents a question state in guided mode
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionState {
  /// The question ID
  pub id: String,
  /// Whether the question has been answered
  pub answered: bool,
  /// Whether a suggestion is currently being loaded
  pub loading_suggestion: bool,
  /// Error message if suggestion loading failed
  pub suggestion_error: Option<String>,
}

/// Trait for providing AI suggestions
pub trait SuggestionProvider {
  /// Check if the provider is available
  fn is_available(&self) -> bool;
}

/// Represents the guided flow state
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidedFlow {
  /// Current question index
  pub current_question: usize,
  /// Total number of questions
  pub total_questions: usize,
  /// Whether the flow is complete
  pub complete: bool,
}

impl GuidedFlow {
  /// Create a new guided flow with the given number of questions
  #[must_use]
  pub const fn new(total_questions: usize) -> Self {
    Self {
      current_question: 0,
      total_questions,
      complete: false,
    }
  }

  /// Mark the current question as answered
  pub const fn mark_answered(&mut self) {
    self.current_question += 1;
    if self.current_question >= self.total_questions {
      self.complete = true;
    }
  }

  /// Check if the flow is complete
  #[must_use]
  pub const fn is_complete(&self) -> bool {
    self.complete
  }

  /// Get the current progress as a fraction
  #[must_use]
  #[allow(clippy::cast_precision_loss)]
  pub fn progress(&self) -> f64 {
    if self.total_questions == 0 {
      return 0.0;
    }
    self.current_question as f64 / self.total_questions as f64
  }
}

/// Test mode toggle for discover phase
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverMode {
  /// Express mode - user types freely
  Express,
  /// Guided mode - structured questions with AI suggestions
  #[default]
  Guided,
}
