//! Analysis Results Types (test version)
//!
//! This is a minimal version without Dioxus imports for testing.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use std::fmt;

/// Confidence score for analysis findings (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
  const MIN: f64 = 0.0;
  const MAX: f64 = 1.0;
  const HIGH_THRESHOLD: f64 = 0.8;
  const MEDIUM_THRESHOLD: f64 = 0.5;

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

  pub const fn value(self) -> f64 {
    self.0
  }

  pub const fn is_high(self) -> bool {
    self.0 >= Self::HIGH_THRESHOLD
  }

  pub const fn is_medium(self) -> bool {
    self.0 >= Self::MEDIUM_THRESHOLD && self.0 < Self::HIGH_THRESHOLD
  }

  pub const fn is_low(self) -> bool {
    self.0 < Self::MEDIUM_THRESHOLD
  }

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

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
  pub title: String,
  pub description: String,
  pub confidence: ConfidenceScore,
  pub category: Option<String>,
}

impl Finding {
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

  pub fn category(&self) -> &str {
    self.category.as_deref().unwrap_or("general")
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisData {
  pub id: String,
  pub title: String,
  pub summary: String,
  pub findings: Vec<Finding>,
  pub created_at: i64,
  pub updated_at: i64,
}

impl AnalysisData {
  pub fn new(
    id: String,
    title: String,
    summary: String,
    findings: Vec<Finding>,
    created_at: i64,
    updated_at: i64,
  ) -> Result<Self, AnalysisError> {
    if title.trim().is_empty() {
      return Err(AnalysisError::InvalidInput("Title cannot be empty".to_string()));
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

  pub const fn finding_count(&self) -> usize {
    self.findings.len()
  }

  pub const fn has_findings(&self) -> bool {
    !self.findings.is_empty()
  }

  pub fn average_confidence(&self) -> Option<ConfidenceScore> {
    if self.findings.is_empty() {
      return None;
    }

    let sum: f64 = self.findings.iter().map(|f| f.confidence.value()).sum();
    let average = sum / self.findings.len() as f64;
    ConfidenceScore::new(average).ok()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisState {
  Loading,
  Success(AnalysisData),
  Empty,
  Error(String),
}

impl AnalysisState {
  pub const fn is_loading(&self) -> bool {
    matches!(self, Self::Loading)
  }

  pub const fn is_success(&self) -> bool {
    matches!(self, Self::Success(_))
  }

  pub const fn is_error(&self) -> bool {
    matches!(self, Self::Error(_))
  }

  pub const fn is_empty(&self) -> bool {
    matches!(self, Self::Empty)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisError {
  InvalidConfidence {
    score: f64,
    valid_range: (f64, f64),
  },
  InvalidInput(String),
  NotFound(String),
  NetworkError(String),
  ParseError(String),
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

  #[test]
  fn test_confidence_score_new_valid_range() {
    let result = ConfidenceScore::new(0.85);
    assert!(result.is_ok());
    let score = result.unwrap();
    assert_eq!(score.value(), 0.85);
    assert!(score.is_high());
  }

  #[test]
  fn test_confidence_score_new_invalid_high() {
    let result = ConfidenceScore::new(1.5);
    assert!(result.is_err());
  }

  #[test]
  fn test_analysis_data_new_valid() {
    let finding = Finding::new("F1".to_string(), "D1".to_string(), 0.8, None).unwrap();
    let result = AnalysisData::new(
      "id".to_string(),
      "Title".to_string(),
      "Summary".to_string(),
      vec![finding],
      0,
      0,
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_analysis_data_new_empty_title() {
    let result = AnalysisData::new(
      "id".to_string(),
      "".to_string(),
      "Summary".to_string(),
      vec![],
      0,
      0,
    );
    assert!(result.is_err());
  }

  #[test]
  fn test_analysis_results_page_component_exists() {
    // This test will fail until we implement the UI component
    // For now, we just document what we need
    let analysis = AnalysisData::new(
      "id".to_string(),
      "Title".to_string(),
      "Summary".to_string(),
      vec![],
      0,
      0,
    ).unwrap();

    // TODO: Create AnalysisResultsPage component
    // let component = AnalysisResultsPage { analysis };
    // assert!(component.is_some());
    panic!("AnalysisResultsPage component not yet implemented");
  }

  #[test]
  fn test_finding_card_component_exists() {
    // This test will fail until we implement the FindingCard component
    let finding = Finding::new("Test".to_string(), "Desc".to_string(), 0.8, None).unwrap();

    // TODO: Create FindingCard component
    // let card = FindingCard { finding };
    // assert!(card.is_some());
    panic!("FindingCard component not yet implemented");
  }
}
