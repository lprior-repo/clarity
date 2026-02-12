//! PME (Product-Market Engineer) Discover Phase Domain Types
//!
//! Scientific rigor types for the Double Diamond Phase 1 - Discover.
//! These types enforce evidence-based decision making and prevent
//! common pitfalls like straw man personas and jumping to conclusions.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// CONFIDENCE BOUNDS
// ============================================================================

/// Minimum confidence score (0.0)
pub const MIN_CONFIDENCE: f32 = 0.0;

/// Maximum confidence score (1.0)
pub const MAX_CONFIDENCE: f32 = 1.0;

/// Confidence threshold for "validated" status
pub const VALIDATED_THRESHOLD: f32 = 0.8;

/// Confidence threshold for "refuted" status
pub const REFUTED_THRESHOLD: f32 = 0.2;

// ============================================================================
// HYPOTHESIS TYPES
// ============================================================================

/// Hypothesis status in the scientific validation process
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
  /// Still being formulated
  Formulating,
  /// Currently being tested
  Testing,
  /// Validated with high confidence
  Validated,
  /// Refuted with high confidence
  Refuted,
  /// Results are inconclusive
  Inconclusive,
}

impl Default for HypothesisStatus {
  fn default() -> Self {
    Self::Formulating
  }
}

impl fmt::Display for HypothesisStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Formulating => write!(f, "Formulating"),
      Self::Testing => write!(f, "Testing"),
      Self::Validated => write!(f, "Validated"),
      Self::Refuted => write!(f, "Refuted"),
      Self::Inconclusive => write!(f, "Inconclusive"),
    }
  }
}

/// Scientific hypothesis with null hypothesis support
///
/// Enforces scientific rigor by requiring a null hypothesis.
/// This prevents jumping to conclusions without considering
/// the opposite possibility.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hypothesis {
  /// Unique identifier
  pub id: Uuid,
  /// The thesis statement being tested
  pub thesis_statement: String,
  /// Required null hypothesis - prevents confirmation bias
  pub null_hypothesis: String,
  /// Criteria used to validate or refute the hypothesis
  pub validation_criteria: Vec<String>,
  /// Current status in the validation process
  pub status: HypothesisStatus,
  /// Confidence score (0.0 to 1.0)
  pub confidence_score: f32,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last modification timestamp
  pub updated_at: DateTime<Utc>,
}

impl Hypothesis {
  /// Create a new hypothesis with thesis and null hypothesis
  ///
  /// # Errors
  /// Returns an error if thesis_statement or null_hypothesis is empty
  pub fn new(thesis_statement: String, null_hypothesis: String) -> Result<Self, PmeError> {
    if thesis_statement.trim().is_empty() {
      return Err(PmeError::EmptyField("thesis_statement".to_string()));
    }
    if null_hypothesis.trim().is_empty() {
      return Err(PmeError::EmptyField("null_hypothesis".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      thesis_statement,
      null_hypothesis,
      validation_criteria: Vec::new(),
      status: HypothesisStatus::default(),
      confidence_score: MIN_CONFIDENCE,
      created_at: now,
      updated_at: now,
    })
  }

  /// Add a validation criterion
  #[must_use]
  pub fn with_validation_criterion(mut self, criterion: String) -> Self {
    self.validation_criteria.push(criterion);
    self.updated_at = Utc::now();
    self
  }

  /// Set the status
  #[must_use]
  pub const fn with_status(mut self, status: HypothesisStatus) -> Self {
    self.status = status;
    self
  }

  /// Set the confidence score (clamped to valid range)
  #[must_use]
  pub const fn with_confidence(mut self, score: f32) -> Self {
    self.confidence_score = score.clamp(MIN_CONFIDENCE, MAX_CONFIDENCE);
    self
  }

  /// Update confidence and derive status from it
  #[must_use]
  pub fn update_confidence(mut self, score: f32) -> Self {
    self.confidence_score = score.clamp(MIN_CONFIDENCE, MAX_CONFIDENCE);
    self.status = self.derive_status();
    self.updated_at = Utc::now();
    self
  }

  /// Derive status from confidence score
  fn derive_status(&self) -> HypothesisStatus {
    if self.confidence_score >= VALIDATED_THRESHOLD {
      HypothesisStatus::Validated
    } else if self.confidence_score <= REFUTED_THRESHOLD {
      HypothesisStatus::Refuted
    } else if self.confidence_score > REFUTED_THRESHOLD && self.confidence_score < VALIDATED_THRESHOLD {
      HypothesisStatus::Inconclusive
    } else {
      self.status
    }
  }

  /// Check if hypothesis is ready for decision
  #[must_use]
  pub const fn is_decided(&self) -> bool {
    matches!(self.status, HypothesisStatus::Validated | HypothesisStatus::Refuted)
  }
}

// ============================================================================
// SIGNAL TYPES
// ============================================================================

/// Types of signals that can be detected in customer interviews
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
  /// User expressed pain or frustration
  PainPoint,
  /// User described a workaround they currently use
  Workaround,
  /// User described a moment of value or success
  ValueMoment,
  /// User expressed confusion or uncertainty
  Confusion,
  /// User expressed delight or satisfaction
  Delight,
}

impl fmt::Display for SignalType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::PainPoint => write!(f, "Pain Point"),
      Self::Workaround => write!(f, "Workaround"),
      Self::ValueMoment => write!(f, "Value Moment"),
      Self::Confusion => write!(f, "Confusion"),
      Self::Delight => write!(f, "Delight"),
    }
  }
}

/// Intensity of an observed signal
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalIntensity {
  /// Weak signal - barely noticeable
  Weak,
  /// Moderate signal - clearly present
  Moderate,
  /// Strong signal - unmistakable
  Strong,
  /// Overwhelming signal - dominant theme
  Overwhelming,
}

impl Default for SignalIntensity {
  fn default() -> Self {
    Self::Moderate
  }
}

impl fmt::Display for SignalIntensity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Weak => write!(f, "Weak"),
      Self::Moderate => write!(f, "Moderate"),
      Self::Strong => write!(f, "Strong"),
      Self::Overwhelming => write!(f, "Overwhelming"),
    }
  }
}

impl SignalIntensity {
  /// Convert intensity to a numeric weight for aggregation
  #[must_use]
  pub const fn weight(&self) -> f32 {
    match self {
      Self::Weak => 0.25,
      Self::Moderate => 0.5,
      Self::Strong => 0.75,
      Self::Overwhelming => 1.0,
    }
  }
}

// ============================================================================
// INTERVIEW TYPES
// ============================================================================

/// Individual interview question with response tracking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InterviewQuestion {
  /// Unique identifier
  pub id: Uuid,
  /// The question text
  pub question: String,
  /// The participant's response (if any)
  pub response: Option<String>,
  /// Whether a signal was detected in the response
  pub signal_detected: bool,
  /// Type of signal detected (if any)
  pub signal_type: Option<SignalType>,
}

impl InterviewQuestion {
  /// Create a new interview question
  #[must_use]
  pub fn new(question: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      question,
      response: None,
      signal_detected: false,
      signal_type: None,
    }
  }

  /// Record a response
  #[must_use]
  pub fn with_response(mut self, response: String) -> Self {
    self.response = Some(response);
    self
  }

  /// Mark a signal as detected
  #[must_use]
  pub const fn with_signal(mut self, signal_type: SignalType) -> Self {
    self.signal_detected = true;
    self.signal_type = Some(signal_type);
    self
  }
}

/// Observed signal from an interview
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalObservation {
  /// Type of signal observed
  pub signal_type: SignalType,
  /// Description of the signal
  pub description: String,
  /// Intensity of the signal
  pub intensity: SignalIntensity,
  /// Raw quote from the participant (if available)
  pub raw_quote: Option<String>,
  /// When the signal was observed
  pub timestamp: DateTime<Utc>,
}

impl SignalObservation {
  /// Create a new signal observation
  #[must_use]
  pub fn new(signal_type: SignalType, description: String, intensity: SignalIntensity) -> Self {
    Self {
      signal_type,
      description,
      intensity,
      raw_quote: None,
      timestamp: Utc::now(),
    }
  }

  /// Add a raw quote
  #[must_use]
  pub fn with_quote(mut self, quote: String) -> Self {
    self.raw_quote = Some(quote);
    self
  }
}

/// Customer Discovery Interview with signal strength tracking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomerDiscoveryInterview {
  /// Unique identifier
  pub id: Uuid,
  /// Participant identifier (anonymized)
  pub participant_id: String,
  /// Date of the interview
  pub date: DateTime<Utc>,
  /// Questions asked during the interview
  pub questions_asked: Vec<InterviewQuestion>,
  /// Signals observed during the interview
  pub signals: Vec<SignalObservation>,
  /// Aggregated signal strength (0.0 to 1.0)
  pub signal_strength: f32,
  /// Key insights extracted from the interview
  pub key_insights: Vec<String>,
  /// When the interview record was created
  pub created_at: DateTime<Utc>,
}

impl CustomerDiscoveryInterview {
  /// Create a new interview record
  #[must_use]
  pub fn new(participant_id: String) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      participant_id,
      date: now,
      questions_asked: Vec::new(),
      signals: Vec::new(),
      signal_strength: MIN_CONFIDENCE,
      key_insights: Vec::new(),
      created_at: now,
    }
  }

  /// Set the interview date
  #[must_use]
  pub const fn with_date(mut self, date: DateTime<Utc>) -> Self {
    self.date = date;
    self
  }

  /// Add a question
  #[must_use]
  pub fn with_question(mut self, question: InterviewQuestion) -> Self {
    self.questions_asked.push(question);
    self
  }

  /// Add a signal observation
  #[must_use]
  pub fn with_signal(mut self, signal: SignalObservation) -> Self {
    self.signals.push(signal);
    self.recalculate_signal_strength();
    self
  }

  /// Add a key insight
  #[must_use]
  pub fn with_insight(mut self, insight: String) -> Self {
    self.key_insights.push(insight);
    self
  }

  /// Recalculate signal strength based on observed signals
  fn recalculate_signal_strength(&mut self) {
    if self.signals.is_empty() {
      self.signal_strength = MIN_CONFIDENCE;
      return;
    }

    let total_weight: f32 = self.signals.iter().map(|s| s.intensity.weight()).sum();
    let count = self.signals.len() as f32;
    // Average intensity, boosted by number of signals
    let boost = (count / 10.0).min(0.3);
    self.signal_strength = ((total_weight / count) + boost).clamp(MIN_CONFIDENCE, MAX_CONFIDENCE);
  }

  /// Get signal count by type
  #[must_use]
  pub fn signal_count_by_type(&self, signal_type: SignalType) -> usize {
    self.signals.iter().filter(|s| s.signal_type == signal_type).count()
  }

  /// Check if interview has sufficient signal strength
  #[must_use]
  pub const fn has_strong_signals(&self) -> bool {
    self.signal_strength >= 0.5
  }
}

// ============================================================================
// PERSONA VALIDATION TYPES
// ============================================================================

/// Types of validation checks for persona evidence
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCheckType {
  /// Persona is backed by actual interview data
  BackedByInterviews,
  /// Persona has verbatim quotes from users
  HasRawQuotes,
  /// Persona traits appear across multiple sources
  MultipleSources,
  /// Signals are consistent and not contradictory
  ConsistentSignals,
}

impl fmt::Display for ValidationCheckType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BackedByInterviews => write!(f, "Backed by Interviews"),
      Self::HasRawQuotes => write!(f, "Has Raw Quotes"),
      Self::MultipleSources => write!(f, "Multiple Sources"),
      Self::ConsistentSignals => write!(f, "Consistent Signals"),
    }
  }
}

/// Individual validation check for a persona
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaValidationCheck {
  /// Type of check performed
  pub check_type: ValidationCheckType,
  /// Whether the check passed
  pub passed: bool,
  /// Summary of evidence for this check
  pub evidence_summary: String,
  /// When the check was performed
  pub checked_at: DateTime<Utc>,
}

impl PersonaValidationCheck {
  /// Create a new validation check
  #[must_use]
  pub fn new(check_type: ValidationCheckType, passed: bool, evidence_summary: String) -> Self {
    Self {
      check_type,
      passed,
      evidence_summary,
      checked_at: Utc::now(),
    }
  }

  /// Create a passing check
  #[must_use]
  pub fn passed(check_type: ValidationCheckType, evidence_summary: String) -> Self {
    Self::new(check_type, true, evidence_summary)
  }

  /// Create a failing check
  #[must_use]
  pub fn failed(check_type: ValidationCheckType, evidence_summary: String) -> Self {
    Self::new(check_type, false, evidence_summary)
  }
}

/// Evidence-based persona with validation tracking
///
/// Prevents "straw man" personas by requiring evidence from
/// actual customer discovery interviews.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaEvidence {
  /// Reference to the base persona
  pub persona_id: Uuid,
  /// Interviews that provide evidence for this persona
  pub interviews_referenced: Vec<Uuid>,
  /// Total count of evidence items
  pub evidence_count: usize,
  /// Confidence level in persona accuracy (0.0 to 1.0)
  pub confidence_level: f32,
  /// Validation checks performed
  pub validation_checks: Vec<PersonaValidationCheck>,
  /// When the persona was last validated
  pub last_validated: Option<DateTime<Utc>>,
}

impl PersonaEvidence {
  /// Create new persona evidence for a persona
  #[must_use]
  pub fn new(persona_id: Uuid) -> Self {
    Self {
      persona_id,
      interviews_referenced: Vec::new(),
      evidence_count: 0,
      confidence_level: MIN_CONFIDENCE,
      validation_checks: Vec::new(),
      last_validated: None,
    }
  }

  /// Add an interview reference
  #[must_use]
  pub fn with_interview(mut self, interview_id: Uuid) -> Self {
    if !self.interviews_referenced.contains(&interview_id) {
      self.interviews_referenced.push(interview_id);
      self.evidence_count += 1;
      self.recalculate_confidence();
    }
    self
  }

  /// Add a validation check
  #[must_use]
  pub fn with_validation_check(mut self, check: PersonaValidationCheck) -> Self {
    self.validation_checks.push(check);
    self.recalculate_confidence();
    self.last_validated = Some(Utc::now());
    self
  }

  /// Recalculate confidence based on evidence
  fn recalculate_confidence(&mut self) {
    if self.validation_checks.is_empty() {
      self.confidence_level = MIN_CONFIDENCE;
      return;
    }

    let passed_count = self.validation_checks.iter().filter(|c| c.passed).count();
    let total = self.validation_checks.len();

    // Base confidence on validation pass rate
    let base_confidence = passed_count as f32 / total as f32;

    // Boost for multiple interviews
    let interview_boost = (self.interviews_referenced.len() as f32 / 5.0).min(0.2);

    self.confidence_level = (base_confidence + interview_boost).clamp(MIN_CONFIDENCE, MAX_CONFIDENCE);
  }

  /// Check if persona is validated (all checks passing)
  #[must_use]
  pub fn is_validated(&self) -> bool {
    !self.validation_checks.is_empty()
      && self.validation_checks.iter().all(|c| c.passed)
  }

  /// Check if persona is a potential "straw man" (low evidence)
  #[must_use]
  pub fn is_straw_man(&self) -> bool {
    self.interviews_referenced.len() < 2 || self.confidence_level < 0.4
  }

  /// Get failing checks
  #[must_use]
  pub fn failing_checks(&self) -> Vec<&PersonaValidationCheck> {
    self.validation_checks.iter().filter(|c| !c.passed).collect()
  }
}

// ============================================================================
// SCENARIO PLOT HOLE TYPES
// ============================================================================

/// Types of plot holes that can be detected in scenarios
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotHoleType {
  /// A step in the user journey is missing
  MissingStep,
  /// Two parts of the scenario contradict each other
  Contradiction,
  /// An assumption is unrealistic or unvalidated
  UnrealisticAssumption,
  /// User takes an action without clear motivation
  UnmotivatedAction,
  /// A solution appears without explanation (deus ex machina)
  DeusExMachina,
}

impl fmt::Display for PlotHoleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MissingStep => write!(f, "Missing Step"),
      Self::Contradiction => write!(f, "Contradiction"),
      Self::UnrealisticAssumption => write!(f, "Unrealistic Assumption"),
      Self::UnmotivatedAction => write!(f, "Unmotivated Action"),
      Self::DeusExMachina => write!(f, "Deus Ex Machina"),
    }
  }
}

/// Severity of a detected plot hole
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotHoleSeverity {
  /// Minor issue - doesn't break the scenario
  Minor,
  /// Moderate issue - weakens the scenario
  Moderate,
  /// Major issue - significantly impacts scenario validity
  Major,
  /// Fatal issue - scenario is invalid as written
  Fatal,
}

impl Default for PlotHoleSeverity {
  fn default() -> Self {
    Self::Moderate
  }
}

impl fmt::Display for PlotHoleSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Minor => write!(f, "Minor"),
      Self::Moderate => write!(f, "Moderate"),
      Self::Major => write!(f, "Major"),
      Self::Fatal => write!(f, "Fatal"),
    }
  }
}

/// Detected plot hole in a scenario
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioPlotHole {
  /// The scenario with the plot hole
  pub scenario_id: Uuid,
  /// Type of plot hole
  pub hole_type: PlotHoleType,
  /// Description of the issue
  pub description: String,
  /// Severity of the issue
  pub severity: PlotHoleSeverity,
  /// Suggested fix (if any)
  pub suggested_fix: Option<String>,
  /// When the plot hole was detected
  pub detected_at: DateTime<Utc>,
}

impl ScenarioPlotHole {
  /// Create a new plot hole detection
  #[must_use]
  pub fn new(
    scenario_id: Uuid,
    hole_type: PlotHoleType,
    description: String,
    severity: PlotHoleSeverity,
  ) -> Self {
    Self {
      scenario_id,
      hole_type,
      description,
      severity,
      suggested_fix: None,
      detected_at: Utc::now(),
    }
  }

  /// Add a suggested fix
  #[must_use]
  pub fn with_suggested_fix(mut self, fix: String) -> Self {
    self.suggested_fix = Some(fix);
    self
  }

  /// Check if plot hole is blocking (major or fatal)
  #[must_use]
  pub const fn is_blocking(&self) -> bool {
    matches!(self.severity, PlotHoleSeverity::Major | PlotHoleSeverity::Fatal)
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// PME domain errors
#[derive(Debug, Error, PartialEq)]
pub enum PmeError {
  /// A required field was empty
  #[error("required field is empty: {0}")]
  EmptyField(String),

  /// Invalid confidence score
  #[error("confidence score {0} is out of range [0.0, 1.0]")]
  InvalidConfidence(f32),

  /// Not enough evidence
  #[error("not enough evidence: need at least {required} but have {actual}")]
  InsufficientEvidence {
    required: usize,
    actual: usize,
  },

  /// Validation failed
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

// ============================================================================
// PME DISCOVER STATE
// ============================================================================

/// Complete PME Discover phase state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PmeDiscoverState {
  /// Hypotheses being tested
  pub hypotheses: Vec<Hypothesis>,
  /// Customer discovery interviews conducted
  pub interviews: Vec<CustomerDiscoveryInterview>,
  /// Persona evidence tracking
  pub persona_evidence: Vec<PersonaEvidence>,
  /// Detected plot holes in scenarios
  pub plot_holes: Vec<ScenarioPlotHole>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl PmeDiscoverState {
  /// Create a new empty PME Discover state
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      hypotheses: Vec::new(),
      interviews: Vec::new(),
      persona_evidence: Vec::new(),
      plot_holes: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a hypothesis
  #[must_use]
  pub fn with_hypothesis(mut self, hypothesis: Hypothesis) -> Self {
    self.hypotheses.push(hypothesis);
    self.updated_at = Utc::now();
    self
  }

  /// Add an interview
  #[must_use]
  pub fn with_interview(mut self, interview: CustomerDiscoveryInterview) -> Self {
    self.interviews.push(interview);
    self.updated_at = Utc::now();
    self
  }

  /// Add persona evidence
  #[must_use]
  pub fn with_persona_evidence(mut self, evidence: PersonaEvidence) -> Self {
    self.persona_evidence.push(evidence);
    self.updated_at = Utc::now();
    self
  }

  /// Add a detected plot hole
  #[must_use]
  pub fn with_plot_hole(mut self, plot_hole: ScenarioPlotHole) -> Self {
    self.plot_holes.push(plot_hole);
    self.updated_at = Utc::now();
    self
  }

  /// Get validated hypotheses
  #[must_use]
  pub fn validated_hypotheses(&self) -> Vec<&Hypothesis> {
    self.hypotheses.iter().filter(|h| h.status == HypothesisStatus::Validated).collect()
  }

  /// Get refuted hypotheses
  #[must_use]
  pub fn refuted_hypotheses(&self) -> Vec<&Hypothesis> {
    self.hypotheses.iter().filter(|h| h.status == HypothesisStatus::Refuted).collect()
  }

  /// Get blocking plot holes
  #[must_use]
  pub fn blocking_plot_holes(&self) -> Vec<&ScenarioPlotHole> {
    self.plot_holes.iter().filter(|p| p.is_blocking()).collect()
  }

  /// Get straw man personas (low evidence)
  #[must_use]
  pub fn straw_man_personas(&self) -> Vec<&PersonaEvidence> {
    self.persona_evidence.iter().filter(|p| p.is_straw_man()).collect()
  }

  /// Calculate overall discovery health score
  #[must_use]
  pub fn health_score(&self) -> f32 {
    if self.hypotheses.is_empty() {
      return MIN_CONFIDENCE;
    }

    let validated = self.validated_hypotheses().len();
    let total = self.hypotheses.len();

    let hypothesis_score = validated as f32 / total as f32;

    let blocking_holes = self.blocking_plot_holes().len();
    let hole_penalty = (blocking_holes as f32 * 0.1).min(0.3);

    let straw_men = self.straw_man_personas().len();
    let straw_penalty = (straw_men as f32 * 0.1).min(0.2);

    (hypothesis_score - hole_penalty - straw_penalty).clamp(MIN_CONFIDENCE, MAX_CONFIDENCE)
  }
}

impl Default for PmeDiscoverState {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hypothesis_new_requires_non_empty_thesis() {
    let result = Hypothesis::new("".to_string(), "null".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn hypothesis_new_requires_non_empty_null() {
    let result = Hypothesis::new("thesis".to_string(), "".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn hypothesis_new_succeeds_with_valid_input() {
    let result = Hypothesis::new("Users want X".to_string(), "Users do not want X".to_string());
    assert!(result.is_ok());
    let h = result.unwrap();
    assert_eq!(h.thesis_statement, "Users want X");
    assert_eq!(h.null_hypothesis, "Users do not want X");
    assert_eq!(h.status, HypothesisStatus::Formulating);
  }

  #[test]
  fn hypothesis_confidence_updates_status() {
    let h = Hypothesis::new("t".to_string(), "n".to_string())
      .update_confidence(0.9);
    assert_eq!(h.status, HypothesisStatus::Validated);

    let h = Hypothesis::new("t".to_string(), "n".to_string())
      .update_confidence(0.1);
    assert_eq!(h.status, HypothesisStatus::Refuted);
  }

  #[test]
  fn signal_intensity_weights() {
    assert!((SignalIntensity::Weak.weight() - 0.25).abs() < f32::EPSILON);
    assert!((SignalIntensity::Moderate.weight() - 0.5).abs() < f32::EPSILON);
    assert!((SignalIntensity::Strong.weight() - 0.75).abs() < f32::EPSILON);
    assert!((SignalIntensity::Overwhelming.weight() - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn interview_signal_strength_aggregates() {
    let interview = CustomerDiscoveryInterview::new("p1".to_string())
      .with_signal(SignalObservation::new(
        SignalType::PainPoint,
        "test".to_string(),
        SignalIntensity::Strong,
      ))
      .with_signal(SignalObservation::new(
        SignalType::Delight,
        "test2".to_string(),
        SignalIntensity::Moderate,
      ));

    assert!(interview.signal_strength > MIN_CONFIDENCE);
    assert!(interview.has_strong_signals());
  }

  #[test]
  fn persona_evidence_detects_straw_man() {
    let evidence = PersonaEvidence::new(Uuid::new_v4());
    assert!(evidence.is_straw_man());
    assert!(!evidence.is_validated());
  }

  #[test]
  fn persona_evidence_with_interviews_not_straw_man() {
    let evidence = PersonaEvidence::new(Uuid::new_v4())
      .with_interview(Uuid::new_v4())
      .with_interview(Uuid::new_v4())
      .with_interview(Uuid::new_v4())
      .with_validation_check(PersonaValidationCheck::passed(
        ValidationCheckType::BackedByInterviews,
        "3 interviews support this".to_string(),
      ));

    assert!(!evidence.is_straw_man());
  }

  #[test]
  fn plot_hole_blocking_detection() {
    let minor = ScenarioPlotHole::new(
      Uuid::new_v4(),
      PlotHoleType::MissingStep,
      "minor".to_string(),
      PlotHoleSeverity::Minor,
    );
    assert!(!minor.is_blocking());

    let fatal = ScenarioPlotHole::new(
      Uuid::new_v4(),
      PlotHoleType::Contradiction,
      "fatal".to_string(),
      PlotHoleSeverity::Fatal,
    );
    assert!(fatal.is_blocking());
  }

  #[test]
  fn pme_discover_state_health_score() {
    let state = PmeDiscoverState::new()
      .with_hypothesis(
        Hypothesis::new("t".to_string(), "n".to_string())
          .update_confidence(0.9),
      );

    assert!(state.health_score() > MIN_CONFIDENCE);
    assert_eq!(state.validated_hypotheses().len(), 1);
  }
}
