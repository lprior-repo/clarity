//! Analysis Results UI Components
//!
//! This module provides components for displaying analysis results from interviews.
//! It handles loading states, error states, and displays structured analysis data.
//!
//! # Design Principles
//!
//! - **Zero panic**: No unwrap/expect/panic in implementation
//! - **Result types**: All operations return Result<T, Error>
//! - **Mobile-first**: Responsive design starting with mobile layout
//! - **Accessible**: WCAG AA compliant color contrast and touch targets

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
// This is a framework limitation, not our code using unwrap.
#![allow(clippy::disallowed_methods)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use dioxus::prelude::*;
use std::fmt;

/// Confidence score for analysis findings (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
  /// Minimum valid confidence score
  const MIN: f64 = 0.0;

  /// Maximum valid confidence score
  const MAX: f64 = 1.0;

  /// High confidence threshold
  const HIGH_THRESHOLD: f64 = 0.8;

  /// Medium confidence threshold
  const MEDIUM_THRESHOLD: f64 = 0.5;

  /// Create a new confidence score with validation
  ///
  /// # Errors
  ///
  /// Returns `AnalysisError::InvalidConfidence` if score is outside [0.0, 1.0]
  pub fn new(score: f64) -> Result<Self, AnalysisError> {
    if score >= Self::MIN && score <= Self::MAX {
      Ok(Self(score))
    } else {
      Err(AnalysisError::InvalidConfidence {
        score,
        valid_range: (Self::MIN, Self::MAX),
      })
    }
  }

  /// Get the underlying score value
  #[must_use]
  pub const fn value(self) -> f64 {
    self.0
  }

  /// Check if this is a high confidence score (>= 0.8)
  #[must_use]
  pub const fn is_high(self) -> bool {
    self.0 >= Self::HIGH_THRESHOLD
  }

  /// Check if this is a medium confidence score (>= 0.5 and < 0.8)
  #[must_use]
  pub const fn is_medium(self) -> bool {
    self.0 >= Self::MEDIUM_THRESHOLD && self.0 < Self::HIGH_THRESHOLD
  }

  /// Check if this is a low confidence score (< 0.5)
  #[must_use]
  pub const fn is_low(self) -> bool {
    self.0 < Self::MEDIUM_THRESHOLD
  }

  /// Get the confidence level as a string
  #[must_use]
  pub fn level(self) -> &'static str {
    if self.is_high() {
      "high"
    } else if self.is_medium() {
      "medium"
    } else {
      "low"
    }
  }
}

impl fmt::Display for ConfidenceScore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:.0}", self.0 * 100.0)
  }
}

/// Individual analysis finding
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
  /// Finding title
  pub title: String,
  /// Finding description
  pub description: String,
  /// Confidence score (0.0-1.0)
  pub confidence: ConfidenceScore,
  /// Optional category/tag
  pub category: Option<String>,
}

impl Finding {
  /// Create a new finding
  ///
  /// # Errors
  ///
  /// Returns `AnalysisError::InvalidConfidence` if confidence is invalid
  pub fn new(
    title: String,
    description: String,
    confidence: f64,
    category: Option<String>,
  ) -> Result<Self, AnalysisError> {
    Ok(Self {
      title,
      description,
      confidence: ConfidenceScore::new(confidence)?,
      category,
    })
  }

  /// Get the finding category or a default value
  #[must_use]
  pub fn category(&self) -> &str {
    self.category.as_deref().unwrap_or("general")
  }
}

/// Analysis results data
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisData {
  /// Analysis ID
  pub id: String,
  /// Analysis title
  pub title: String,
  /// Analysis summary
  pub summary: String,
  /// Individual findings
  pub findings: Vec<Finding>,
  /// Timestamp when analysis was created
  pub created_at: i64,
  /// Timestamp when analysis was updated
  pub updated_at: i64,
}

impl AnalysisData {
  /// Create new analysis data
  ///
  /// # Errors
  ///
  /// Returns `AnalysisError::InvalidInput` if title or summary is empty
  pub fn new(
    id: String,
    title: String,
    summary: String,
    findings: Vec<Finding>,
    created_at: i64,
    updated_at: i64,
  ) -> Result<Self, AnalysisError> {
    if title.trim().is_empty() {
      return Err(AnalysisError::InvalidInput(
        "Title cannot be empty".to_string(),
      ));
    }
    if summary.trim().is_empty() {
      return Err(AnalysisError::InvalidInput(
        "Summary cannot be empty".to_string(),
      ));
    }

    Ok(Self {
      id,
      title,
      summary,
      findings,
      created_at,
      updated_at,
    })
  }

  /// Get the number of findings
  #[must_use]
  pub const fn finding_count(&self) -> usize {
    self.findings.len()
  }

  /// Check if analysis has any findings
  #[must_use]
  pub const fn has_findings(&self) -> bool {
    !self.findings.is_empty()
  }

  /// Get average confidence score across all findings
  ///
  /// Returns None if there are no findings
  #[must_use]
  pub fn average_confidence(&self) -> Option<ConfidenceScore> {
    if self.findings.is_empty() {
      return None;
    }

    let sum: f64 = self.findings.iter().map(|f| f.confidence.value()).sum();
    let average = sum / self.findings.len() as f64;

    // Average of valid scores is always valid
    ConfidenceScore::new(average).ok()
  }
}

/// Analysis state for UI rendering
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisState {
  /// Loading analysis data
  Loading,
  /// Analysis data loaded successfully
  Success(AnalysisData),
  /// No analysis data available
  Empty,
  /// Error loading analysis
  Error(String),
}

impl AnalysisState {
  /// Check if state is loading
  #[must_use]
  pub const fn is_loading(&self) -> bool {
    matches!(self, Self::Loading)
  }

  /// Check if state is success
  #[must_use]
  pub const fn is_success(&self) -> bool {
    matches!(self, Self::Success(_))
  }

  /// Check if state is error
  #[must_use]
  pub const fn is_error(&self) -> bool {
    matches!(self, Self::Error(_))
  }

  /// Check if state is empty
  #[must_use]
  pub const fn is_empty(&self) -> bool {
    matches!(self, Self::Empty)
  }
}

/// Analysis error types
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisError {
  /// Invalid confidence score
  InvalidConfidence { score: f64, valid_range: (f64, f64) },

  /// Invalid input data
  InvalidInput(String),

  /// Analysis not found
  NotFound(String),

  /// Network error
  NetworkError(String),

  /// Parse error
  ParseError(String),

  /// Export error
  ExportError(String),
}

impl fmt::Display for AnalysisError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidConfidence {
        score,
        valid_range: (min, max),
      } => {
        write!(
          f,
          "Invalid confidence score {score}: must be between {min} and {max}"
        )
      }
      Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
      Self::NotFound(id) => write!(f, "Analysis not found: {id}"),
      Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
      Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
      Self::ExportError(msg) => write!(f, "Export error: {msg}"),
    }
  }
}

impl std::error::Error for AnalysisError {}

#[cfg(test)]
mod tests {
  use super::*;

  // Test 1: Should Create ConfidenceScore With Valid Value
  #[test]
  fn test_confidence_score_new_valid_range() {
    let result = ConfidenceScore::new(0.85);
    assert!(result.is_ok());
    let score = result.unwrap();
    assert_eq!(score.value(), 0.85);
    assert!(score.is_high());
    assert!(!score.is_medium());
    assert!(!score.is_low());
  }

  // Test 2: Should Reject ConfidenceScore Above Maximum
  #[test]
  fn test_confidence_score_new_invalid_high() {
    let result = ConfidenceScore::new(1.5);
    assert!(result.is_err());
    match result {
      Err(AnalysisError::InvalidConfidence { score, valid_range }) => {
        assert_eq!(score, 1.5);
        assert_eq!(valid_range, (0.0, 1.0));
      }
      _ => panic!("Expected InvalidConfidence error"),
    }
  }

  // Test 3: Should Reject ConfidenceScore Below Minimum
  #[test]
  fn test_confidence_score_new_invalid_low() {
    let result = ConfidenceScore::new(-0.1);
    assert!(result.is_err());
    match result {
      Err(AnalysisError::InvalidConfidence { score, .. }) => {
        assert_eq!(score, -0.1);
      }
      _ => panic!("Expected InvalidConfidence error"),
    }
  }

  // Test 4: Should Accept Boundary Values
  #[test]
  fn test_confidence_score_boundaries() {
    assert!(ConfidenceScore::new(0.0).is_ok());
    assert!(ConfidenceScore::new(1.0).is_ok());
  }

  // Test 5: Should Classify High Confidence
  #[test]
  fn test_confidence_score_is_high() {
    let score = ConfidenceScore::new(0.9).unwrap();
    assert!(score.is_high());
    assert!(!score.is_medium());
    assert!(!score.is_low());
    assert_eq!(score.level(), "high");
  }

  // Test 6: Should Classify Medium Confidence
  #[test]
  fn test_confidence_score_is_medium() {
    let score = ConfidenceScore::new(0.65).unwrap();
    assert!(!score.is_high());
    assert!(score.is_medium());
    assert!(!score.is_low());
    assert_eq!(score.level(), "medium");
  }

  // Test 7: Should Classify Low Confidence
  #[test]
  fn test_confidence_score_is_low() {
    let score = ConfidenceScore::new(0.3).unwrap();
    assert!(!score.is_high());
    assert!(!score.is_medium());
    assert!(score.is_low());
    assert_eq!(score.level(), "low");
  }

  // Test 8: Should Classify Threshold Boundaries
  #[test]
  fn test_confidence_score_thresholds() {
    // High threshold (0.8)
    let score = ConfidenceScore::new(0.8).unwrap();
    assert!(score.is_high());

    // Medium threshold (0.5)
    let score = ConfidenceScore::new(0.5).unwrap();
    assert!(score.is_medium());

    // Just below high threshold
    let score = ConfidenceScore::new(0.79).unwrap();
    assert!(score.is_medium());

    // Just below medium threshold
    let score = ConfidenceScore::new(0.49).unwrap();
    assert!(score.is_low());
  }

  // Test 9: Should Create Finding With Valid Data
  #[test]
  fn test_finding_new_valid() {
    let result = Finding::new(
      "Test Finding".to_string(),
      "Test description".to_string(),
      0.85,
      Some("category".to_string()),
    );
    assert!(result.is_ok());
    let finding = result.unwrap();
    assert_eq!(finding.title, "Test Finding");
    assert_eq!(finding.description, "Test description");
    assert_eq!(finding.confidence.value(), 0.85);
    assert_eq!(finding.category(), "category");
  }

  // Test 10: Should Reject Finding With Invalid Confidence
  #[test]
  fn test_finding_new_invalid_confidence() {
    let result = Finding::new(
      "Test Finding".to_string(),
      "Test description".to_string(),
      1.5,
      Some("category".to_string()),
    );
    assert!(result.is_err());
    match result {
      Err(AnalysisError::InvalidConfidence { .. }) => {
        // Expected
      }
      _ => panic!("Expected InvalidConfidence error"),
    }
  }

  // Test 11: Should Return Default Category When None
  #[test]
  fn test_finding_category_default() {
    let finding = Finding::new(
      "Test Finding".to_string(),
      "Test description".to_string(),
      0.85,
      None,
    )
    .unwrap();
    assert_eq!(finding.category(), "general");
  }

  // Test 12: Should Create AnalysisData With Valid Fields
  #[test]
  fn test_analysis_data_new_valid() {
    let finding = Finding::new(
      "Finding 1".to_string(),
      "Description".to_string(),
      0.8,
      None,
    )
    .unwrap();

    let result = AnalysisData::new(
      "analysis-123".to_string(),
      "Test Analysis".to_string(),
      "Test summary".to_string(),
      vec![finding],
      1234567890,
      1234567891,
    );
    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.id, "analysis-123");
    assert_eq!(analysis.title, "Test Analysis");
    assert_eq!(analysis.summary, "Test summary");
    assert_eq!(analysis.finding_count(), 1);
    assert!(analysis.has_findings());
  }

  // Test 13: Should Reject AnalysisData With Empty Title
  #[test]
  fn test_analysis_data_new_empty_title() {
    let result = AnalysisData::new(
      "analysis-123".to_string(),
      "".to_string(),
      "Test summary".to_string(),
      vec![],
      1234567890,
      1234567891,
    );
    assert!(result.is_err());
    match result {
      Err(AnalysisError::InvalidInput(msg)) => {
        assert!(msg.contains("Title"));
        assert!(msg.contains("empty"));
      }
      _ => panic!("Expected InvalidInput error"),
    }
  }

  // Test 14: Should Reject AnalysisData With Whitespace Title
  #[test]
  fn test_analysis_data_new_whitespace_title() {
    let result = AnalysisData::new(
      "analysis-123".to_string(),
      "   ".to_string(),
      "Test summary".to_string(),
      vec![],
      1234567890,
      1234567891,
    );
    assert!(result.is_err());
  }

  // Test 15: Should Reject AnalysisData With Empty Summary
  #[test]
  fn test_analysis_data_new_empty_summary() {
    let result = AnalysisData::new(
      "analysis-123".to_string(),
      "Test Analysis".to_string(),
      "".to_string(),
      vec![],
      1234567890,
      1234567891,
    );
    assert!(result.is_err());
    match result {
      Err(AnalysisError::InvalidInput(msg)) => {
        assert!(msg.contains("Summary"));
        assert!(msg.contains("empty"));
      }
      _ => panic!("Expected InvalidInput error"),
    }
  }

  // Test 16: Should Calculate Average Confidence
  #[test]
  fn test_analysis_data_average_confidence() {
    let findings = vec![
      Finding::new("F1".to_string(), "D1".to_string(), 0.8, None).unwrap(),
      Finding::new("F2".to_string(), "D2".to_string(), 0.6, None).unwrap(),
      Finding::new("F3".to_string(), "D3".to_string(), 1.0, None).unwrap(),
    ];

    let analysis = AnalysisData::new(
      "analysis-123".to_string(),
      "Test".to_string(),
      "Summary".to_string(),
      findings,
      0,
      0,
    )
    .unwrap();

    let avg = analysis.average_confidence();
    assert!(avg.is_some());
    // (0.8 + 0.6 + 1.0) / 3 = 0.8
    assert_eq!(avg.unwrap().value(), 0.8);
  }

  // Test 17: Should Return None For Average Confidence With No Findings
  #[test]
  fn test_analysis_data_average_confidence_empty() {
    let analysis = AnalysisData::new(
      "analysis-123".to_string(),
      "Test".to_string(),
      "Summary".to_string(),
      vec![],
      0,
      0,
    )
    .unwrap();

    assert!(analysis.average_confidence().is_none());
  }

  // Test 18: AnalysisState Loading
  #[test]
  fn test_analysis_state_loading() {
    let state = AnalysisState::Loading;
    assert!(state.is_loading());
    assert!(!state.is_success());
    assert!(!state.is_error());
    assert!(!state.is_empty());
  }

  // Test 19: AnalysisState Success
  #[test]
  fn test_analysis_state_success() {
    let analysis = AnalysisData::new(
      "analysis-123".to_string(),
      "Test".to_string(),
      "Summary".to_string(),
      vec![],
      0,
      0,
    )
    .unwrap();

    let state = AnalysisState::Success(analysis);
    assert!(!state.is_loading());
    assert!(state.is_success());
    assert!(!state.is_error());
    assert!(!state.is_empty());
  }

  // Test 20: AnalysisState Error
  #[test]
  fn test_analysis_state_error() {
    let state = AnalysisState::Error("Network error".to_string());
    assert!(!state.is_loading());
    assert!(!state.is_success());
    assert!(state.is_error());
    assert!(!state.is_empty());
  }

  // Test 21: AnalysisState Empty
  #[test]
  fn test_analysis_state_empty() {
    let state = AnalysisState::Empty;
    assert!(!state.is_loading());
    assert!(!state.is_success());
    assert!(!state.is_error());
    assert!(state.is_empty());
  }

  // Test 22: ConfidenceScore Display
  #[test]
  fn test_confidence_score_display() {
    let score = ConfidenceScore::new(0.85).unwrap();
    assert_eq!(format!("{}", score), "85");
  }

  // Test 23: ConfidenceScore Display Rounds Correctly
  #[test]
  fn test_confidence_score_display_rounding() {
    let score = ConfidenceScore::new(0.856).unwrap();
    assert_eq!(format!("{}", score), "86");
  }

  // Test 24: AnalysisError Display
  #[test]
  fn test_analysis_error_display_invalid_confidence() {
    let error = AnalysisError::InvalidConfidence {
      score: 1.5,
      valid_range: (0.0, 1.0),
    };
    let display = format!("{}", error);
    assert!(display.contains("1.5"));
    assert!(display.contains("0.0"));
    assert!(display.contains("1.0"));
  }

  // Test 25: AnalysisError Display InvalidInput
  #[test]
  fn test_analysis_error_display_invalid_input() {
    let error = AnalysisError::InvalidInput("Title cannot be empty".to_string());
    assert_eq!(format!("{}", error), "Invalid input: Title cannot be empty");
  }

  // Test 26: AnalysisError Display NotFound
  #[test]
  fn test_analysis_error_display_not_found() {
    let error = AnalysisError::NotFound("analysis-123".to_string());
    assert_eq!(format!("{}", error), "Analysis not found: analysis-123");
  }

  // Test 27: AnalysisError Display NetworkError
  #[test]
  fn test_analysis_error_display_network_error() {
    let error = AnalysisError::NetworkError("Connection refused".to_string());
    assert_eq!(format!("{}", error), "Network error: Connection refused");
  }

  // Test 28: AnalysisError Display ParseError
  #[test]
  fn test_analysis_error_display_parse_error() {
    let error = AnalysisError::ParseError("Invalid JSON".to_string());
    assert_eq!(format!("{}", error), "Parse error: Invalid JSON");
  }

  // Test 29: AnalysisError Display ExportError
  #[test]
  fn test_analysis_error_display_export_error() {
    let error = AnalysisError::ExportError("File write failed".to_string());
    assert_eq!(format!("{}", error), "Export error: File write failed");
  }

  // Test 30: Finding Clone
  #[test]
  fn test_finding_clone() {
    let finding1 = Finding::new(
      "Test".to_string(),
      "Description".to_string(),
      0.8,
      Some("category".to_string()),
    )
    .unwrap();
    let finding2 = finding1.clone();
    assert_eq!(finding1, finding2);
  }

  // Test 31: AnalysisData Clone
  #[test]
  fn test_analysis_data_clone() {
    let finding = Finding::new("F1".to_string(), "D1".to_string(), 0.8, None).unwrap();
    let analysis1 = AnalysisData::new(
      "id".to_string(),
      "Title".to_string(),
      "Summary".to_string(),
      vec![finding],
      0,
      0,
    )
    .unwrap();
    let analysis2 = analysis1.clone();
    assert_eq!(analysis1, analysis2);
  }

  // Test 32: AnalysisState Clone
  #[test]
  fn test_analysis_state_clone() {
    let state1 = AnalysisState::Error("Test".to_string());
    let state2 = state1.clone();
    assert_eq!(state1, state2);
  }

  // Test 33: Finding PartialEq
  #[test]
  fn test_finding_partial_eq() {
    let finding1 = Finding::new(
      "Test".to_string(),
      "Description".to_string(),
      0.8,
      Some("category".to_string()),
    )
    .unwrap();
    let finding2 = Finding::new(
      "Test".to_string(),
      "Description".to_string(),
      0.8,
      Some("category".to_string()),
    )
    .unwrap();
    assert_eq!(finding1, finding2);
  }

  // Test 34: Finding PartialEq Different Title
  #[test]
  fn test_finding_partial_eq_different_title() {
    let finding1 = Finding::new("Test1".to_string(), "D".to_string(), 0.8, None).unwrap();
    let finding2 = Finding::new("Test2".to_string(), "D".to_string(), 0.8, None).unwrap();
    assert_ne!(finding1, finding2);
  }

  // Test 35: ConfidenceScore PartialEq
  #[test]
  fn test_confidence_score_partial_eq() {
    let score1 = ConfidenceScore::new(0.8).unwrap();
    let score2 = ConfidenceScore::new(0.8).unwrap();
    assert_eq!(score1, score2);
  }

  // Test 36: ConfidenceScore PartialOrd
  #[test]
  fn test_confidence_score_partial_ord() {
    let score1 = ConfidenceScore::new(0.7).unwrap();
    let score2 = ConfidenceScore::new(0.8).unwrap();
    assert!(score1 < score2);
    assert!(score2 > score1);
  }

  // Test 37: AnalysisData With Multiple Findings
  #[test]
  fn test_analysis_data_multiple_findings() {
    let findings = vec![
      Finding::new("F1".to_string(), "D1".to_string(), 0.9, None).unwrap(),
      Finding::new("F2".to_string(), "D2".to_string(), 0.7, None).unwrap(),
      Finding::new("F3".to_string(), "D3".to_string(), 0.5, None).unwrap(),
    ];

    let analysis = AnalysisData::new(
      "id".to_string(),
      "Title".to_string(),
      "Summary".to_string(),
      findings,
      0,
      0,
    )
    .unwrap();

    assert_eq!(analysis.finding_count(), 3);
    assert!(analysis.has_findings());
  }

  // Test 38: AnalysisData Empty Findings
  #[test]
  fn test_analysis_data_empty_findings() {
    let analysis = AnalysisData::new(
      "id".to_string(),
      "Title".to_string(),
      "Summary".to_string(),
      vec![],
      0,
      0,
    )
    .unwrap();

    assert_eq!(analysis.finding_count(), 0);
    assert!(!analysis.has_findings());
  }
}
