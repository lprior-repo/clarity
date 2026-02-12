//! Meyer's Design by Contract framework.
//!
//! Captures and evaluates three contract layers:
//! - Preconditions: what must be true before execution
//! - Postconditions: what must be true after execution
//! - Invariants: what must always remain true

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub const MIN_SCORE: f32 = 0.0;
pub const MAX_SCORE: f32 = 1.0;
pub const ACCEPTABLE_CONTRACT_SCORE: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractLayer {
  Precondition,
  Postcondition,
  Invariant,
}

impl fmt::Display for ContractLayer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Precondition => write!(f, "Precondition"),
      Self::Postcondition => write!(f, "Postcondition"),
      Self::Invariant => write!(f, "Invariant"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractSeverity {
  Minor,
  Major,
  Critical,
}

impl fmt::Display for ContractSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Minor => write!(f, "Minor"),
      Self::Major => write!(f, "Major"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractClause {
  pub id: Uuid,
  pub layer: ContractLayer,
  pub statement: String,
  pub severity: ContractSeverity,
  pub satisfied: bool,
  pub confidence: f32,
  pub evidence: Option<String>,
}

impl ContractClause {
  pub fn new(
    layer: ContractLayer,
    statement: String,
    severity: ContractSeverity,
    satisfied: bool,
  ) -> Result<Self, DesignByContractError> {
    if statement.trim().is_empty() {
      return Err(DesignByContractError::EmptyField("statement".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      layer,
      statement: statement.trim().to_string(),
      severity,
      satisfied,
      confidence: if satisfied { 1.0 } else { 0.0 },
      evidence: None,
    })
  }

  #[must_use]
  pub fn with_confidence(self, confidence: f32) -> Self {
    Self {
      confidence: confidence.clamp(MIN_SCORE, MAX_SCORE),
      ..self
    }
  }

  #[must_use]
  pub fn with_evidence(self, evidence: String) -> Self {
    Self {
      evidence: Some(evidence.trim().to_string()),
      ..self
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractReport {
  pub id: Uuid,
  pub subject: String,
  pub clauses: Vec<ContractClause>,
  pub contract_score: f32,
  pub valid: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ContractReport {
  pub fn new(subject: String) -> Result<Self, DesignByContractError> {
    if subject.trim().is_empty() {
      return Err(DesignByContractError::EmptyField("subject".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      subject: subject.trim().to_string(),
      clauses: Vec::new(),
      contract_score: 0.0,
      valid: false,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_clause(self, clause: ContractClause) -> Self {
    let clauses: Vec<ContractClause> = self
      .clauses
      .into_iter()
      .chain(std::iter::once(clause))
      .collect();

    let (contract_score, valid) = compute_contract_state(&clauses);
    Self {
      clauses,
      contract_score,
      valid,
      updated_at: Utc::now(),
      ..self
    }
  }

  #[must_use]
  pub fn preconditions(&self) -> Vec<&ContractClause> {
    self
      .clauses
      .iter()
      .filter(|clause| clause.layer == ContractLayer::Precondition)
      .collect()
  }

  #[must_use]
  pub fn postconditions(&self) -> Vec<&ContractClause> {
    self
      .clauses
      .iter()
      .filter(|clause| clause.layer == ContractLayer::Postcondition)
      .collect()
  }

  #[must_use]
  pub fn invariants(&self) -> Vec<&ContractClause> {
    self
      .clauses
      .iter()
      .filter(|clause| clause.layer == ContractLayer::Invariant)
      .collect()
  }

  #[must_use]
  pub fn violated(&self) -> Vec<&ContractClause> {
    self
      .clauses
      .iter()
      .filter(|clause| !clause.satisfied)
      .collect()
  }

  pub fn validate(&self) -> Result<(), DesignByContractError> {
    if self.preconditions().is_empty() {
      return Err(DesignByContractError::MissingLayer(
        ContractLayer::Precondition,
      ));
    }
    if self.postconditions().is_empty() {
      return Err(DesignByContractError::MissingLayer(
        ContractLayer::Postcondition,
      ));
    }
    if self.invariants().is_empty() {
      return Err(DesignByContractError::MissingLayer(
        ContractLayer::Invariant,
      ));
    }

    let violations = self.violated();
    if let Some(critical) = violations
      .into_iter()
      .find(|clause| clause.severity == ContractSeverity::Critical)
    {
      return Err(DesignByContractError::CriticalViolation(
        critical.statement.clone(),
      ));
    }

    if !self.valid {
      return Err(DesignByContractError::ContractInvalid {
        score: self.contract_score,
      });
    }

    Ok(())
  }
}

fn compute_contract_state(clauses: &[ContractClause]) -> (f32, bool) {
  if clauses.is_empty() {
    return (0.0, false);
  }

  let sum: f32 = clauses.iter().map(|clause| clause.confidence).sum();
  let score = sum / clauses.len() as f32;
  let has_violations = clauses.iter().any(|clause| !clause.satisfied);
  (score, score >= ACCEPTABLE_CONTRACT_SCORE && !has_violations)
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum DesignByContractError {
  #[error("field cannot be empty: {0}")]
  EmptyField(String),

  #[error("missing required contract layer: {0}")]
  MissingLayer(ContractLayer),

  #[error("critical contract violation: {0}")]
  CriticalViolation(String),

  #[error("contract is invalid (score={score:.2})")]
  ContractInvalid { score: f32 },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn report_requires_subject() {
    let report = ContractReport::new(String::new());
    assert!(matches!(
      report,
      Err(DesignByContractError::EmptyField(field)) if field == "subject"
    ));
  }

  #[test]
  fn clause_requires_statement() {
    let clause = ContractClause::new(
      ContractLayer::Precondition,
      "   ".to_string(),
      ContractSeverity::Major,
      true,
    );
    assert!(matches!(
      clause,
      Err(DesignByContractError::EmptyField(field)) if field == "statement"
    ));
  }

  #[test]
  fn validate_fails_on_critical_violation() {
    let report_result = ContractReport::new("checkout".to_string());
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    let pre_result = ContractClause::new(
      ContractLayer::Precondition,
      "user is authenticated".to_string(),
      ContractSeverity::Critical,
      false,
    );
    let post_result = ContractClause::new(
      ContractLayer::Postcondition,
      "order has id".to_string(),
      ContractSeverity::Major,
      true,
    );
    let invariant_result = ContractClause::new(
      ContractLayer::Invariant,
      "price is never negative".to_string(),
      ContractSeverity::Major,
      true,
    );

    assert!(pre_result.is_ok());
    assert!(post_result.is_ok());
    assert!(invariant_result.is_ok());

    let report = report
      .with_clause(match pre_result {
        Ok(pre) => pre,
        Err(_) => return,
      })
      .with_clause(match post_result {
        Ok(post) => post,
        Err(_) => return,
      })
      .with_clause(match invariant_result {
        Ok(invariant) => invariant,
        Err(_) => return,
      });

    let result = report.validate();
    assert!(matches!(
      result,
      Err(DesignByContractError::CriticalViolation(_))
    ));
  }

  #[test]
  fn validate_passes_for_complete_and_satisfied_report() {
    let report_result = ContractReport::new("checkout".to_string());
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    let pre_result = ContractClause::new(
      ContractLayer::Precondition,
      "user is authenticated".to_string(),
      ContractSeverity::Major,
      true,
    );
    let post_result = ContractClause::new(
      ContractLayer::Postcondition,
      "order has id".to_string(),
      ContractSeverity::Major,
      true,
    );
    let invariant_result = ContractClause::new(
      ContractLayer::Invariant,
      "price is never negative".to_string(),
      ContractSeverity::Major,
      true,
    );

    assert!(pre_result.is_ok());
    assert!(post_result.is_ok());
    assert!(invariant_result.is_ok());

    let report = report
      .with_clause(match pre_result {
        Ok(pre) => pre.with_confidence(0.9),
        Err(_) => return,
      })
      .with_clause(match post_result {
        Ok(post) => post.with_confidence(0.9),
        Err(_) => return,
      })
      .with_clause(match invariant_result {
        Ok(invariant) => invariant.with_confidence(0.9),
        Err(_) => return,
      });

    assert!(report.validate().is_ok());
    assert!(report.valid);
  }
}
