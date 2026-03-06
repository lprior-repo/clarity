//! Quality Algebra
//!
//! Unified quality reporting and evaluation traits.

use crate::domain::error::ClarityError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityReport {
  pub overall_score: u8,
}

pub trait QualityEvaluator<T> {
  fn evaluate(&self, input: &T) -> Result<QualityReport, ClarityError>;
}
