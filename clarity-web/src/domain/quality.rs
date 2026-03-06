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
  ///
  /// # Errors
  ///
  /// Returns a `ClarityError` if the evaluation process fails.
  fn evaluate(&self, input: &T) -> Result<QualityReport, ClarityError>;
}
