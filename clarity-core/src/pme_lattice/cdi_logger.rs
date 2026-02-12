//! CDI (Customer Data Insight) Logger - Bead bd-16qs.4
//!
//! Captures and analyzes customer signals with strength-based prioritization.
//! Part of the PME Lattice framework for product-market engineering.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::redundant_closure_for_method_calls)]

use chrono::{DateTime, Utc};
use rpds::Vector;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// SIGNAL STRENGTH
// ============================================================================

/// Strength levels for customer signals
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
  /// High-confidence, repeated signal (1.0)
  Strong,
  /// Moderate confidence signal (0.6)
  Medium,
  /// Low confidence or isolated signal (0.3)
  Weak,
  /// Likely noise, single occurrence (0.1)
  Noise,
}

impl SignalStrength {
  /// Returns the numeric value for this signal strength (0.0-1.0)
  #[must_use]
  pub const fn value(self) -> f32 {
    match self {
      Self::Strong => 1.0,
      Self::Medium => 0.6,
      Self::Weak => 0.3,
      Self::Noise => 0.1,
    }
  }

  /// Creates a SignalStrength from an f32 value (0.0-1.0)
  ///
  /// Values are mapped as:
  /// - 0.85-1.0: Strong
  /// - 0.50-0.84: Medium
  /// - 0.20-0.49: Weak
  /// - 0.00-0.19: Noise
  #[must_use]
  pub fn from_f32(value: f32) -> Self {
    let clamped = value.clamp(0.0, 1.0);
    match clamped {
      v if v >= 0.85 => Self::Strong,
      v if v >= 0.50 => Self::Medium,
      v if v >= 0.20 => Self::Weak,
      _ => Self::Noise,
    }
  }
}

impl fmt::Display for SignalStrength {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Strong => write!(f, "Strong"),
      Self::Medium => write!(f, "Medium"),
      Self::Weak => write!(f, "Weak"),
      Self::Noise => write!(f, "Noise"),
    }
  }
}

// ============================================================================
// SIGNAL SOURCE
// ============================================================================

/// Source of the customer signal
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
  /// Customer interviews
  Interview,
  /// Survey responses
  Survey,
  /// Product analytics data
  Analytics,
  /// Support tickets
  SupportTicket,
  /// Social media mentions
  SocialMedia,
  /// Sales call notes
  SalesCall,
  /// Other sources
  Other,
}

impl fmt::Display for SignalSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Interview => write!(f, "Interview"),
      Self::Survey => write!(f, "Survey"),
      Self::Analytics => write!(f, "Analytics"),
      Self::SupportTicket => write!(f, "Support Ticket"),
      Self::SocialMedia => write!(f, "Social Media"),
      Self::SalesCall => write!(f, "Sales Call"),
      Self::Other => write!(f, "Other"),
    }
  }
}

// ============================================================================
// CUSTOMER SIGNAL
// ============================================================================

/// A captured customer insight with metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomerSignal {
  /// Unique identifier
  pub id: Uuid,
  /// The insight text
  pub insight: String,
  /// Strength of the signal
  pub signal_strength: SignalStrength,
  /// Source of the insight
  pub source: SignalSource,
  /// Additional source context
  pub source_detail: Option<String>,
  /// Tags for categorization
  pub tags: Vec<String>,
  /// When the signal was captured
  pub captured_at: DateTime<Utc>,
}

impl CustomerSignal {
  /// Creates a new customer signal
  ///
  /// # Errors
  /// Returns `CDIError::EmptyInsight` if the insight is empty or whitespace
  pub fn new(
    insight: String,
    signal_strength: SignalStrength,
    source: SignalSource,
  ) -> Result<Self, CDIError> {
    if insight.trim().is_empty() {
      return Err(CDIError::EmptyInsight);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      insight,
      signal_strength,
      source,
      source_detail: None,
      tags: Vec::new(),
      captured_at: Utc::now(),
    })
  }

  /// Add source detail context
  #[must_use]
  pub fn with_source_detail(mut self, detail: String) -> Self {
    self.source_detail = Some(detail);
    self
  }

  /// Add a tag (duplicates are ignored)
  #[must_use]
  pub fn with_tag(mut self, tag: String) -> Self {
    if !self.tags.contains(&tag) {
      self.tags.push(tag);
    }
    self
  }

  /// Returns the numeric strength value
  #[must_use]
  pub fn strength_value(&self) -> f32 {
    self.signal_strength.value()
  }
}

// ============================================================================
// CDI LOGGER
// ============================================================================

/// Collects and analyzes customer signals using persistent data structures
#[derive(Clone, Debug, Default)]
pub struct CDILogger {
  signals: Vector<CustomerSignal>,
}

impl CDILogger {
  /// Creates a new empty CDI logger
  #[must_use]
  pub fn new() -> Self {
    Self {
      signals: rpds::Vector::new(),
    }
  }

  /// Add a signal to the logger (returns new logger, original unchanged)
  #[must_use]
  pub fn add_signal(self, signal: CustomerSignal) -> Self {
    Self {
      signals: self.signals.push_back(signal),
    }
  }

  /// Returns true if no signals are logged
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.signals.is_empty()
  }

  /// Returns the count of logged signals
  #[must_use]
  pub fn signal_count(&self) -> usize {
    self.signals.len()
  }

  /// Returns signals with Strong strength
  #[must_use]
  pub fn get_strong_signals(&self) -> Vec<&CustomerSignal> {
    self
      .signals
      .iter()
      .filter(|signal| signal.signal_strength == SignalStrength::Strong)
      .collect()
  }

  /// Returns signals filtered by source
  #[must_use]
  pub fn get_signals_by_source(&self, source: SignalSource) -> Vec<&CustomerSignal> {
    self
      .signals
      .iter()
      .filter(|signal| signal.source == source)
      .collect()
  }

  /// Returns signals with strength value above the threshold
  #[must_use]
  pub fn get_signals_above_threshold(&self, threshold: f32) -> Vec<&CustomerSignal> {
    self
      .signals
      .iter()
      .filter(|signal| signal.strength_value() >= threshold)
      .collect()
  }

  /// Find a signal by ID
  #[must_use]
  pub fn get_signal_by_id(&self, id: Uuid) -> Option<&CustomerSignal> {
    self.signals.iter().find(|signal| signal.id == id)
  }

  /// Returns all signals as a vector
  #[must_use]
  pub fn all_signals(&self) -> Vec<&CustomerSignal> {
    self.signals.iter().collect()
  }

  /// Returns signals sorted by strength (strongest first)
  #[must_use]
  pub fn get_signals_sorted_by_strength(&self) -> Vec<&CustomerSignal> {
    let mut signals: Vec<_> = self.signals.iter().collect();
    signals.sort_by(|a, b| {
      b.strength_value()
        .partial_cmp(&a.strength_value())
        .unwrap_or(std::cmp::Ordering::Equal)
    });
    signals
  }
}

// ============================================================================
// AGGREGATE STRENGTH CALCULATION
// ============================================================================

/// Errors for aggregate strength calculation
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AggregateStrengthError {
  /// No signals to calculate from
  #[error("no signals available to calculate aggregate strength")]
  NoSignals,
}

/// Calculates the aggregate signal strength from all signals in the logger
///
/// # Errors
/// Returns `AggregateStrengthError::NoSignals` if the logger is empty
pub fn calculate_aggregate_strength(logger: &CDILogger) -> Result<f32, AggregateStrengthError> {
  if logger.is_empty() {
    return Err(AggregateStrengthError::NoSignals);
  }

  let total: f32 = logger
    .signals
    .iter()
    .map(|signal| signal.strength_value())
    .sum();

  let count = logger.signal_count() as f32;

  Ok(total / count)
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for the CDI logger module
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CDIError {
  /// The insight text was empty
  #[error("insight cannot be empty")]
  EmptyInsight,

  /// Validation failed
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signal_strength_all_variants_have_values() {
    assert!((SignalStrength::Strong.value() - 1.0).abs() < f32::EPSILON);
    assert!((SignalStrength::Medium.value() - 0.6).abs() < f32::EPSILON);
    assert!((SignalStrength::Weak.value() - 0.3).abs() < f32::EPSILON);
    assert!((SignalStrength::Noise.value() - 0.1).abs() < f32::EPSILON);
  }

  #[test]
  fn signal_strength_from_f32_boundary_values() {
    assert_eq!(SignalStrength::from_f32(0.85), SignalStrength::Strong);
    assert_eq!(SignalStrength::from_f32(0.84), SignalStrength::Medium);
    assert_eq!(SignalStrength::from_f32(0.50), SignalStrength::Medium);
    assert_eq!(SignalStrength::from_f32(0.49), SignalStrength::Weak);
    assert_eq!(SignalStrength::from_f32(0.20), SignalStrength::Weak);
    assert_eq!(SignalStrength::from_f32(0.19), SignalStrength::Noise);
  }
}
