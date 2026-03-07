//! Quality Algebra
//!
//! Unified quality reporting and evaluation traits.

use crate::domain::error::ClarityError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityReport {
  pub overall_score: u8,
}

pub trait QualityEvaluator<T> {
  /// Evaluate input quality and return a structured report.
  ///
  /// # Errors
  /// Returns [`ClarityError`] when evaluation cannot complete successfully.
  fn evaluate(&self, input: &T) -> Result<QualityReport, ClarityError>;
}
