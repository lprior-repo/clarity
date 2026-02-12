//! Gap detection framework for anti-pattern and OWASP coverage.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
  Low,
  Medium,
  High,
  Critical,
}

impl fmt::Display for GapSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Low => write!(f, "Low"),
      Self::Medium => write!(f, "Medium"),
      Self::High => write!(f, "High"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwaspCategory {
  BrokenAccessControl,
  CryptographicFailures,
  Injection,
  InsecureDesign,
  SecurityMisconfiguration,
  VulnerableComponents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductAntiPattern {
  UndefinedUserOutcome,
  MissingFailureMode,
  HiddenCoupling,
  VanityMetricOnly,
  IrreversibleWorkflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapKind {
  Owasp(OwaspCategory),
  AntiPattern(ProductAntiPattern),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignSignal {
  pub key: String,
  pub present: bool,
}

impl DesignSignal {
  pub fn new(key: String, present: bool) -> Result<Self, GapDetectionError> {
    if key.trim().is_empty() {
      return Err(GapDetectionError::EmptyField("key".to_string()));
    }
    Ok(Self {
      key: key.trim().to_string(),
      present,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
  pub id: Uuid,
  pub kind: GapKind,
  pub severity: GapSeverity,
  pub title: String,
  pub remediation: String,
}

impl Gap {
  fn new(
    kind: GapKind,
    severity: GapSeverity,
    title: &str,
    remediation: &str,
  ) -> Result<Self, GapDetectionError> {
    if title.trim().is_empty() {
      return Err(GapDetectionError::EmptyField("title".to_string()));
    }
    if remediation.trim().is_empty() {
      return Err(GapDetectionError::EmptyField("remediation".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      kind,
      severity,
      title: title.to_string(),
      remediation: remediation.to_string(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapReport {
  pub id: Uuid,
  pub subject: String,
  pub gaps: Vec<Gap>,
  pub created_at: DateTime<Utc>,
}

impl GapReport {
  pub fn detect(subject: String, signals: &[DesignSignal]) -> Result<Self, GapDetectionError> {
    if subject.trim().is_empty() {
      return Err(GapDetectionError::EmptyField("subject".to_string()));
    }

    let has = |key: &str| {
      signals
        .iter()
        .any(|signal| signal.key == key && signal.present)
    };

    let maybe_gaps: [Option<Result<Gap, GapDetectionError>>; 8] = [
      (!has("authz")).then(|| {
        Gap::new(
          GapKind::Owasp(OwaspCategory::BrokenAccessControl),
          GapSeverity::Critical,
          "Authorization policy missing",
          "Define least-privilege authorization rules and denial tests.",
        )
      }),
      (!has("input_validation")).then(|| {
        Gap::new(
          GapKind::Owasp(OwaspCategory::Injection),
          GapSeverity::High,
          "Input validation coverage missing",
          "Add validation and adversarial tests for malformed and hostile input.",
        )
      }),
      (!has("secure_defaults")).then(|| {
        Gap::new(
          GapKind::Owasp(OwaspCategory::SecurityMisconfiguration),
          GapSeverity::High,
          "Secure defaults are undefined",
          "Harden default settings and document secure baseline configuration.",
        )
      }),
      (!has("dependency_review")).then(|| {
        Gap::new(
          GapKind::Owasp(OwaspCategory::VulnerableComponents),
          GapSeverity::Medium,
          "Dependency risk review missing",
          "Track dependency advisories and pin/update vulnerable packages.",
        )
      }),
      (!has("user_outcome")).then(|| {
        Gap::new(
          GapKind::AntiPattern(ProductAntiPattern::UndefinedUserOutcome),
          GapSeverity::High,
          "User outcome is undefined",
          "Define measurable user outcomes before implementation.",
        )
      }),
      (!has("failure_modes")).then(|| {
        Gap::new(
          GapKind::AntiPattern(ProductAntiPattern::MissingFailureMode),
          GapSeverity::High,
          "Failure modes are not documented",
          "List failure paths and graceful degradation strategy.",
        )
      }),
      (!has("dependency_map")).then(|| {
        Gap::new(
          GapKind::AntiPattern(ProductAntiPattern::HiddenCoupling),
          GapSeverity::Medium,
          "Hidden coupling risk",
          "Map explicit dependencies and isolate side effects.",
        )
      }),
      (!has("value_metric")).then(|| {
        Gap::new(
          GapKind::AntiPattern(ProductAntiPattern::VanityMetricOnly),
          GapSeverity::Medium,
          "Value metric missing",
          "Define a value metric beyond traffic or signups.",
        )
      }),
    ];

    let gaps = maybe_gaps
      .into_iter()
      .flatten()
      .collect::<Result<Vec<_>, _>>()?;

    Ok(Self {
      id: Uuid::new_v4(),
      subject: subject.trim().to_string(),
      gaps,
      created_at: Utc::now(),
    })
  }

  #[must_use]
  pub fn critical_count(&self) -> usize {
    self
      .gaps
      .iter()
      .filter(|gap| gap.severity == GapSeverity::Critical)
      .count()
  }

  #[must_use]
  pub fn by_kind(&self, kind: GapKind) -> Vec<&Gap> {
    self.gaps.iter().filter(|gap| gap.kind == kind).collect()
  }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum GapDetectionError {
  #[error("field cannot be empty: {0}")]
  EmptyField(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detect_flags_missing_security_signals() {
    let report_result = GapReport::detect("checkout".to_string(), &[]);
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    assert!(report.critical_count() >= 1);
    assert!(!report
      .by_kind(GapKind::Owasp(OwaspCategory::Injection))
      .is_empty());
  }

  #[test]
  fn detect_reduces_gaps_when_signals_present() {
    let signals_result = vec![
      DesignSignal::new("authz".to_string(), true),
      DesignSignal::new("input_validation".to_string(), true),
      DesignSignal::new("secure_defaults".to_string(), true),
      DesignSignal::new("dependency_review".to_string(), true),
      DesignSignal::new("user_outcome".to_string(), true),
      DesignSignal::new("failure_modes".to_string(), true),
      DesignSignal::new("dependency_map".to_string(), true),
      DesignSignal::new("value_metric".to_string(), true),
    ];

    assert!(signals_result.iter().all(Result::is_ok));
    let signals: Vec<DesignSignal> = signals_result.into_iter().filter_map(Result::ok).collect();

    let report_result = GapReport::detect("checkout".to_string(), &signals);
    assert!(report_result.is_ok());
    let report = match report_result {
      Ok(report) => report,
      Err(_) => return,
    };

    assert!(report.gaps.is_empty());
  }
}
