#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! CDI (Customer Discovery Interview) Logger with Signal Strength
//!
//! Tracks Customer Discovery Interviews with signal strength classification:
//! - **High Signal**: User volunteered information without prompting
//! - **Low Signal**: User responded to direct questions
//! - **Mixed Signal**: Combination of volunteered and prompted responses
//!
//! # Funnel Tracking
//!
//! The logger tracks the discovery interview funnel:
//! 1. Contact attempts
//! 2. Successful contacts
//! 3. Completed interviews
//! 4. High-signal insights
//!
//! # Example
//!
//! ```
//! use clarity_web::pme::discover::cdi_logger::{CdiLogger, CdiEntry, SignalStrength};
//!
//! let entry = CdiEntry::new("Interview #1".to_string())
//!     .with_participant("Enterprise Analyst".to_string())
//!     .with_signal(SignalStrength::High);
//!
//! let output = CdiLogger::log_entry(entry);
//! ```

use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Signal Types
// ============================================================================

/// Strength of signal from customer discovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalStrength {
    /// User volunteered information without prompting
    High,
    /// User responded to direct questions
    Low,
    /// Combination of volunteered and prompted responses
    Mixed,
}

impl SignalStrength {
    /// Get the numeric value of this signal strength.
    #[must_use]
    pub const fn value(&self) -> f64 {
        match self {
            Self::High => 1.0,
            Self::Mixed => 0.5,
            Self::Low => 0.2,
        }
    }

    /// Get a description of this signal strength.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::High => "User volunteered this information without prompting - strong validation signal",
            Self::Low => "User responded to direct questions - requires more validation",
            Self::Mixed => "Combination of volunteered and prompted responses",
        }
    }
}

// ============================================================================
// Signal Type (Content Category)
// ============================================================================

/// Type of signal content discovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Problem/pain point
    Problem,
    /// Current workaround/solution
    Workaround,
    /// Desired outcome/goal
    DesiredOutcome,
    /// Constraint/limitation
    Constraint,
    /// Budget/pricing information
    Budget,
    /// Decision-making process
    Decision,
    /// Competition/alternatives
    Competition,
    /// Use case/scenario
    UseCase,
    /// Objection/concern
    Objection,
}

impl SignalType {
    /// Get the name of this signal type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Problem => "Problem",
            Self::Workaround => "Workaround",
            Self::DesiredOutcome => "Desired Outcome",
            Self::Constraint => "Constraint",
            Self::Budget => "Budget",
            Self::Decision => "Decision",
            Self::Competition => "Competition",
            Self::UseCase => "Use Case",
            Self::Objection => "Objection",
        }
    }

    /// Get all signal types.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::Problem,
            Self::Workaround,
            Self::DesiredOutcome,
            Self::Constraint,
            Self::Budget,
            Self::Decision,
            Self::Competition,
            Self::UseCase,
            Self::Objection,
        ]
    }
}

// ============================================================================
// CDI Entry Types
// ============================================================================

/// A Customer Discovery Interview entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CdiEntry {
    /// Entry identifier
    pub id: String,
    /// Participant identifier/description
    pub participant: String,
    /// Interview date
    pub date: DateTime<Utc>,
    /// Signal strength
    pub signal_strength: SignalStrength,
    /// Signals collected
    pub signals: Vec<CdiSignal>,
    /// Interview outcome
    pub outcome: InterviewOutcome,
    /// Notes and observations
    pub notes: Vec<String>,
    /// Follow-up actions
    pub follow_ups: Vec<String>,
    /// Duration in minutes
    pub duration_minutes: Option<u32>,
    /// Segment/cohort
    pub segment: Option<String>,
}

/// A specific signal discovered during an interview.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CdiSignal {
    /// The signal content
    pub content: String,
    /// Type of signal
    pub signal_type: SignalType,
    /// Signal strength
    pub strength: SignalStrength,
    /// Quote from participant (if available)
    pub quote: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl CdiSignal {
    /// Create a new signal.
    #[must_use]
    pub fn new(content: String, signal_type: SignalType, strength: SignalStrength) -> Self {
        Self {
            content,
            signal_type,
            strength,
            quote: None,
            tags: Vec::new(),
        }
    }

    /// Add a quote.
    #[must_use]
    pub fn with_quote(mut self, quote: String) -> Self {
        self.quote = Some(quote);
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Calculate signal score (strength value with quote bonus).
    #[must_use]
    pub fn score(&self) -> f64 {
        let base = self.strength.value();
        let quote_bonus = if self.quote.is_some() { 0.1 } else { 0.0 };
        (base + quote_bonus).min(1.0)
    }
}

/// Outcome of an interview.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterviewOutcome {
    /// Interview completed successfully
    Completed,
    /// Interview partially completed
    Partial,
    /// Participant not qualified
    NotQualified,
    /// Participant declined
    Declined,
    /// No response
    NoResponse,
    /// Rescheduled
    Rescheduled,
}

impl CdiEntry {
    /// Create a new CDI entry.
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            participant: String::new(),
            date: Utc::now(),
            signal_strength: SignalStrength::Mixed,
            signals: Vec::new(),
            outcome: InterviewOutcome::Completed,
            notes: Vec::new(),
            follow_ups: Vec::new(),
            duration_minutes: None,
            segment: None,
        }
    }

    /// Set participant.
    #[must_use]
    pub fn with_participant(mut self, participant: String) -> Self {
        self.participant = participant;
        self
    }

    /// Set date.
    #[must_use]
    pub fn with_date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    /// Set signal strength.
    #[must_use]
    pub fn with_signal(mut self, strength: SignalStrength) -> Self {
        self.signal_strength = strength;
        self
    }

    /// Add a signal.
    #[must_use]
    pub fn with_signal_item(mut self, signal: CdiSignal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Set outcome.
    #[must_use]
    pub fn with_outcome(mut self, outcome: InterviewOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add a note.
    #[must_use]
    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    /// Add a follow-up action.
    #[must_use]
    pub fn with_follow_up(mut self, action: String) -> Self {
        self.follow_ups.push(action);
        self
    }

    /// Set duration.
    #[must_use]
    pub fn with_duration(mut self, minutes: u32) -> Self {
        self.duration_minutes = Some(minutes);
        self
    }

    /// Set segment.
    #[must_use]
    pub fn with_segment(mut self, segment: String) -> Self {
        self.segment = Some(segment);
        self
    }

    /// Calculate total signal score.
    #[must_use]
    pub fn total_signal_score(&self) -> f64 {
        if self.signals.is_empty() {
            return 0.0;
        }

        self.signals.iter().map(|s| s.score()).sum::<f64>()
            / f64::from(u8::try_from(self.signals.len()).map_or(1, |v| v))
    }

    /// Get signals by type.
    #[must_use]
    pub fn signals_by_type(&self, signal_type: SignalType) -> Vec<&CdiSignal> {
        self.signals.iter().filter(|s| s.signal_type == signal_type).collect()
    }

    /// Get high-signal items only.
    #[must_use]
    pub fn high_signal_items(&self) -> Vec<&CdiSignal> {
        self.signals
            .iter()
            .filter(|s| s.strength == SignalStrength::High)
            .collect()
    }

    /// Check if entry is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty() && !self.participant.is_empty()
    }
}

// ============================================================================
// Funnel Types
// ============================================================================

/// Customer Discovery Interview funnel metrics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CdiFunnel {
    /// Total contact attempts
    pub contact_attempts: u32,
    /// Successful contacts (response received)
    pub successful_contacts: u32,
    /// Completed interviews
    pub completed_interviews: u32,
    /// High-signal interviews
    pub high_signal_interviews: u32,
    /// Total signals collected
    pub total_signals: u32,
    /// High-signal items
    pub high_signal_count: u32,
}

impl CdiFunnel {
    /// Create a new empty funnel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contact_attempts: 0,
            successful_contacts: 0,
            completed_interviews: 0,
            high_signal_interviews: 0,
            total_signals: 0,
            high_signal_count: 0,
        }
    }

    /// Calculate contact success rate.
    #[must_use]
    pub fn contact_success_rate(&self) -> f64 {
        if self.contact_attempts == 0 {
            return 0.0;
        }
        f64::from(self.successful_contacts) / f64::from(self.contact_attempts)
    }

    /// Calculate interview completion rate.
    #[must_use]
    pub fn interview_completion_rate(&self) -> f64 {
        if self.successful_contacts == 0 {
            return 0.0;
        }
        f64::from(self.completed_interviews) / f64::from(self.successful_contacts)
    }

    /// Calculate high signal rate.
    #[must_use]
    pub fn high_signal_rate(&self) -> f64 {
        if self.total_signals == 0 {
            return 0.0;
        }
        f64::from(self.high_signal_count) / f64::from(self.total_signals)
    }

    /// Calculate overall funnel efficiency.
    #[must_use]
    pub fn funnel_efficiency(&self) -> f64 {
        let contact_rate = self.contact_success_rate();
        let completion_rate = self.interview_completion_rate();
        let signal_rate = self.high_signal_rate();

        (contact_rate * 0.3 + completion_rate * 0.3 + signal_rate * 0.4).clamp(0.0, 1.0)
    }

    /// Add an entry to the funnel.
    #[must_use]
    pub fn with_entry(mut self, entry: &CdiEntry) -> Self {
        self.contact_attempts += 1;

        if entry.outcome != InterviewOutcome::NoResponse {
            self.successful_contacts += 1;
        }

        if entry.outcome == InterviewOutcome::Completed {
            self.completed_interviews += 1;

            let signal_count = u32::try_from(entry.signals.len()).map_or(0, |v| v);
            self.total_signals += signal_count;

            let high_count = u32::try_from(entry.high_signal_items().len()).map_or(0, |v| v);
            self.high_signal_count += high_count;

            if !entry.high_signal_items().is_empty() {
                self.high_signal_interviews += 1;
            }
        }

        self
    }

    /// Merge another funnel into this one.
    #[must_use]
    pub fn merge(&mut self, other: &CdiFunnel) {
        self.contact_attempts += other.contact_attempts;
        self.successful_contacts += other.successful_contacts;
        self.completed_interviews += other.completed_interviews;
        self.high_signal_interviews += other.high_signal_interviews;
        self.total_signals += other.total_signals;
        self.high_signal_count += other.high_signal_count;
    }
}

impl Default for CdiFunnel {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Output Types
// ============================================================================

/// Output from the CDI Logger.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CdiLoggerOutput {
    /// All entries
    pub entries: Vec<CdiEntry>,
    /// Funnel metrics
    pub funnel: CdiFunnel,
    /// Signal analysis
    pub signal_analysis: SignalAnalysis,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Analysis of collected signals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalAnalysis {
    /// Signal counts by type
    pub by_type: Vec<(SignalType, usize)>,
    /// Signal counts by strength
    pub by_strength: Vec<(SignalStrength, usize)>,
    /// Average signal score
    pub avg_score: f64,
    /// Top problems identified
    pub top_problems: Vec<String>,
    /// Top desired outcomes
    pub top_outcomes: Vec<String>,
    /// Common objections
    pub common_objections: Vec<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the CDI Logger.
#[derive(Debug, Error)]
pub enum CdiLoggerError {
    /// Entry ID is empty
    #[error("Entry ID cannot be empty")]
    EmptyEntryId,

    /// Participant is empty
    #[error("Participant cannot be empty")]
    EmptyParticipant,

    /// No entries to analyze
    #[error("No entries provided for analysis")]
    NoEntries,
}

// ============================================================================
// CDI Logger Implementation
// ============================================================================

/// CDI Logger - Tracks Customer Discovery Interviews with signal strength.
pub struct CdiLogger;

impl CdiLogger {
    /// Log a single entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry is invalid.
    pub fn log_entry(entry: CdiEntry) -> Result<CdiLoggerOutput, CdiLoggerError> {
        if entry.id.is_empty() {
            return Err(CdiLoggerError::EmptyEntryId);
        }
        if entry.participant.is_empty() {
            return Err(CdiLoggerError::EmptyParticipant);
        }

        let entries = vec![entry.clone()];
        let funnel = CdiFunnel::new().with_entry(&entry);
        let signal_analysis = Self::analyze_signals(&entries);
        let recommendations = Self::generate_recommendations(&funnel, &signal_analysis);

        Ok(CdiLoggerOutput {
            entries,
            funnel,
            signal_analysis,
            recommendations,
        })
    }

    /// Log multiple entries and aggregate results.
    ///
    /// # Errors
    ///
    /// Returns an error if no entries are provided or any entry is invalid.
    pub fn log_entries(entries: Vec<CdiEntry>) -> Result<CdiLoggerOutput, CdiLoggerError> {
        if entries.is_empty() {
            return Err(CdiLoggerError::NoEntries);
        }

        // Validate all entries
        for entry in &entries {
            if entry.id.is_empty() {
                return Err(CdiLoggerError::EmptyEntryId);
            }
            if entry.participant.is_empty() {
                return Err(CdiLoggerError::EmptyParticipant);
            }
        }

        // Build funnel from all entries
        let funnel = entries.iter().fold(CdiFunnel::new(), |f, e| f.with_entry(e));

        // Analyze signals
        let signal_analysis = Self::analyze_signals(&entries);

        // Generate recommendations
        let recommendations = Self::generate_recommendations(&funnel, &signal_analysis);

        Ok(CdiLoggerOutput {
            entries,
            funnel,
            signal_analysis,
            recommendations,
        })
    }

    /// Analyze signals from entries.
    fn analyze_signals(entries: &[CdiEntry]) -> SignalAnalysis {
        // Count by type
        let by_type: Vec<(SignalType, usize)> = SignalType::all()
            .iter()
            .map(|&t| {
                let count = entries
                    .iter()
                    .flat_map(|e| &e.signals)
                    .filter(|s| s.signal_type == t)
                    .count();
                (t, count)
            })
            .filter(|(_, c)| *c > 0)
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .collect();

        // Count by strength
        let by_strength: Vec<(SignalStrength, usize)> = [
            (SignalStrength::High, entries.iter().flat_map(|e| &e.signals).filter(|s| s.strength == SignalStrength::High).count()),
            (SignalStrength::Mixed, entries.iter().flat_map(|e| &e.signals).filter(|s| s.strength == SignalStrength::Mixed).count()),
            (SignalStrength::Low, entries.iter().flat_map(|e| &e.signals).filter(|s| s.strength == SignalStrength::Low).count()),
        ].into_iter().filter(|(_, c)| *c > 0).collect();

        // Calculate average score
        let all_signals: Vec<_> = entries.iter().flat_map(|e| &e.signals).collect();
        let avg_score = if all_signals.is_empty() {
            0.0
        } else {
            all_signals.iter().map(|s| s.score()).sum::<f64>()
                / f64::from(u8::try_from(all_signals.len()).map_or(1, |v| v))
        };

        // Extract top problems
        let top_problems: Vec<String> = entries
            .iter()
            .flat_map(|e| &e.signals)
            .filter(|s| s.signal_type == SignalType::Problem)
            .sorted_by(|a, b| b.score().partial_cmp(&a.score()).map_or(std::cmp::Ordering::Equal, |v| v))
            .take(5)
            .map(|s| s.content.clone())
            .collect();

        // Extract top outcomes
        let top_outcomes: Vec<String> = entries
            .iter()
            .flat_map(|e| &e.signals)
            .filter(|s| s.signal_type == SignalType::DesiredOutcome)
            .sorted_by(|a, b| b.score().partial_cmp(&a.score()).map_or(std::cmp::Ordering::Equal, |v| v))
            .take(5)
            .map(|s| s.content.clone())
            .collect();

        // Extract common objections
        let common_objections: Vec<String> = entries
            .iter()
            .flat_map(|e| &e.signals)
            .filter(|s| s.signal_type == SignalType::Objection)
            .take(5)
            .map(|s| s.content.clone())
            .collect();

        SignalAnalysis {
            by_type,
            by_strength,
            avg_score,
            top_problems,
            top_outcomes,
            common_objections,
        }
    }

    /// Generate recommendations based on funnel and signal analysis.
    fn generate_recommendations(funnel: &CdiFunnel, analysis: &SignalAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Funnel recommendations
        if funnel.contact_success_rate() < 0.3 {
            recommendations.push(
                "Low contact success rate - review outreach messaging and channels".to_string(),
            );
        }

        if funnel.interview_completion_rate() < 0.5 {
            recommendations.push(
                "Low interview completion rate - consider shorter interview format or better scheduling".to_string(),
            );
        }

        if funnel.high_signal_rate() < 0.3 {
            recommendations.push(
                "Low high-signal rate - focus on open-ended questions to elicit volunteered information".to_string(),
            );
        }

        // Signal type recommendations
        let problem_count = analysis
            .by_type
            .iter()
            .find(|(t, _)| *t == SignalType::Problem)
            .map(|(_, c)| *c)
            .map_or(0, |v| v);

        let outcome_count = analysis
            .by_type
            .iter()
            .find(|(t, _)| *t == SignalType::DesiredOutcome)
            .map(|(_, c)| *c)
            .map_or(0, |v| v);

        if problem_count > 0 && outcome_count == 0 {
            recommendations.push(
                "Problems identified but no desired outcomes - ask users what success looks like".to_string(),
            );
        }

        // Sample size recommendations
        if funnel.completed_interviews < 5 {
            recommendations.push(
                format!(
                    "Only {} completed interviews - continue discovery to reach statistical significance",
                    funnel.completed_interviews
                ),
            );
        } else if funnel.completed_interviews >= 10 && funnel.high_signal_interviews < 5 {
            recommendations.push(
                "10+ interviews but few high-signal results - may need to refine target segment".to_string(),
            );
        }

        // Analysis quality
        if analysis.avg_score < 0.4 {
            recommendations.push(
                "Low average signal score - focus on observation over direct questioning".to_string(),
            );
        }

        recommendations
    }

    /// Get signal summary for a specific type.
    #[must_use]
    pub fn get_signals_by_type(entries: &[CdiEntry], signal_type: SignalType) -> Vec<&CdiSignal> {
        entries
            .iter()
            .flat_map(|e| &e.signals)
            .filter(|s| s.signal_type == signal_type)
            .collect()
    }

    /// Get high-signal quotes.
    #[must_use]
    pub fn get_high_signal_quotes(entries: &[CdiEntry]) -> Vec<&str> {
        entries
            .iter()
            .flat_map(|e| &e.signals)
            .filter(|s| s.strength == SignalStrength::High)
            .filter_map(|s| s.quote.as_deref())
            .collect()
    }

    /// Calculate segment performance.
    #[must_use]
    pub fn segment_performance(entries: &[CdiEntry]) -> Vec<(String, f64)> {
        entries
            .iter()
            .filter_map(|e| e.segment.as_ref().map(|s| (s.clone(), e.total_signal_score())))
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .chunk_by(|(segment, _)| segment.clone())
            .into_iter()
            .map(|(segment, group)| {
                let scores: Vec<f64> = group.map(|(_, score)| score).collect();
                let avg = scores.iter().sum::<f64>() / f64::from(u8::try_from(scores.len()).map_or(1, |v| v));
                (segment, avg)
            })
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).map_or(std::cmp::Ordering::Equal, |v| v))
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
    use super::*;

    fn create_test_signal() -> CdiSignal {
        CdiSignal::new(
            "Takes too long to generate reports".to_string(),
            SignalType::Problem,
            SignalStrength::High,
        )
        .with_quote("I spend 10 hours a week on this!".to_string())
        .with_tag("pain-point".to_string())
    }

    fn create_test_entry() -> CdiEntry {
        CdiEntry::new("interview-001".to_string())
            .with_participant("Enterprise Analyst".to_string())
            .with_signal(SignalStrength::High)
            .with_signal_item(create_test_signal())
            .with_signal_item(CdiSignal::new(
                "Want faster reporting".to_string(),
                SignalType::DesiredOutcome,
                SignalStrength::High,
            ))
            .with_outcome(InterviewOutcome::Completed)
            .with_note("Great interview".to_string())
            .with_duration(30)
            .with_segment("Enterprise".to_string())
    }

    #[test]
    fn test_signal_strength_value() {
        assert!((SignalStrength::High.value() - 1.0).abs() < 0.01);
        assert!((SignalStrength::Mixed.value() - 0.5).abs() < 0.01);
        assert!((SignalStrength::Low.value() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_signal_type_all() {
        let all = SignalType::all();
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn test_cdi_signal_creation() {
        let signal = create_test_signal();

        assert_eq!(signal.signal_type, SignalType::Problem);
        assert_eq!(signal.strength, SignalStrength::High);
        assert!(signal.quote.is_some());
        assert!(!signal.tags.is_empty());
    }

    #[test]
    fn test_cdi_signal_score() {
        let with_quote = CdiSignal::new(
            "Test".to_string(),
            SignalType::Problem,
            SignalStrength::High,
        )
        .with_quote("Quote".to_string());

        let without_quote = CdiSignal::new(
            "Test".to_string(),
            SignalType::Problem,
            SignalStrength::High,
        );

        // High signal (1.0) + quote bonus (0.1) = 1.0 (capped)
        assert!((with_quote.score() - 1.0).abs() < 0.01);
        // High signal without quote = 1.0
        assert!((without_quote.score() - 1.0).abs() < 0.01);
        // Both are capped at 1.0, so they should be equal
        assert!((with_quote.score() - without_quote.score()).abs() < 0.01);
    }

    #[test]
    fn test_cdi_entry_creation() {
        let entry = create_test_entry();

        assert_eq!(entry.id, "interview-001");
        assert_eq!(entry.participant, "Enterprise Analyst");
        assert_eq!(entry.signals.len(), 2);
        assert!(entry.is_valid());
    }

    #[test]
    fn test_cdi_entry_invalid() {
        let no_id = CdiEntry::new("".to_string()).with_participant("Test".to_string());
        assert!(!no_id.is_valid());

        let no_participant = CdiEntry::new("id".to_string());
        assert!(!no_participant.is_valid());
    }

    #[test]
    fn test_cdi_entry_total_signal_score() {
        let entry = create_test_entry();
        let score = entry.total_signal_score();

        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_cdi_entry_signals_by_type() {
        let entry = create_test_entry();

        let problems = entry.signals_by_type(SignalType::Problem);
        assert_eq!(problems.len(), 1);

        let outcomes = entry.signals_by_type(SignalType::DesiredOutcome);
        assert_eq!(outcomes.len(), 1);
    }

    #[test]
    fn test_cdi_entry_high_signal_items() {
        let entry = create_test_entry();
        let high_signal = entry.high_signal_items();

        assert_eq!(high_signal.len(), 2); // Both are high signal
    }

    #[test]
    fn test_cdi_funnel_creation() {
        let funnel = CdiFunnel::new();

        assert_eq!(funnel.contact_attempts, 0);
        assert_eq!(funnel.successful_contacts, 0);
        assert_eq!(funnel.completed_interviews, 0);
    }

    #[test]
    fn test_cdi_funnel_with_entry() {
        let entry = create_test_entry();
        let funnel = CdiFunnel::new().with_entry(&entry);

        assert_eq!(funnel.contact_attempts, 1);
        assert_eq!(funnel.successful_contacts, 1);
        assert_eq!(funnel.completed_interviews, 1);
        assert_eq!(funnel.total_signals, 2);
        assert_eq!(funnel.high_signal_count, 2);
    }

    #[test]
    fn test_cdi_funnel_rates() {
        let entry = create_test_entry();
        let funnel = CdiFunnel::new().with_entry(&entry);

        assert!((funnel.contact_success_rate() - 1.0).abs() < 0.01);
        assert!((funnel.interview_completion_rate() - 1.0).abs() < 0.01);
        assert!((funnel.high_signal_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cdi_funnel_efficiency() {
        let entry = create_test_entry();
        let funnel = CdiFunnel::new().with_entry(&entry);

        let efficiency = funnel.funnel_efficiency();
        assert!(efficiency > 0.8);
    }

    #[test]
    fn test_cdi_logger_empty_entry_id() {
        let entry = CdiEntry::new("".to_string()).with_participant("Test".to_string());
        let result = CdiLogger::log_entry(entry);

        assert!(result.is_err());
        assert!(matches!(result, Err(CdiLoggerError::EmptyEntryId)));
    }

    #[test]
    fn test_cdi_logger_empty_participant() {
        let entry = CdiEntry::new("id".to_string());
        let result = CdiLogger::log_entry(entry);

        assert!(result.is_err());
        assert!(matches!(result, Err(CdiLoggerError::EmptyParticipant)));
    }

    #[test]
    fn test_cdi_logger_success() {
        let entry = create_test_entry();
        let result = CdiLogger::log_entry(entry);

        assert!(result.is_ok());
        let output = result.expect("Should succeed");

        assert_eq!(output.entries.len(), 1);
        assert_eq!(output.funnel.completed_interviews, 1);
    }

    #[test]
    fn test_cdi_logger_no_entries() {
        let result = CdiLogger::log_entries(vec![]);

        assert!(result.is_err());
        assert!(matches!(result, Err(CdiLoggerError::NoEntries)));
    }

    #[test]
    fn test_cdi_logger_multiple_entries() {
        let entry1 = create_test_entry();
        let entry2 = CdiEntry::new("interview-002".to_string())
            .with_participant("Manager".to_string())
            .with_signal(SignalStrength::Mixed)
            .with_signal_item(CdiSignal::new(
                "Budget constraints".to_string(),
                SignalType::Budget,
                SignalStrength::Low,
            ))
            .with_outcome(InterviewOutcome::Completed);

        let result = CdiLogger::log_entries(vec![entry1, entry2]);

        assert!(result.is_ok());
        let output = result.expect("Should succeed");

        assert_eq!(output.entries.len(), 2);
        assert_eq!(output.funnel.completed_interviews, 2);
        assert_eq!(output.funnel.total_signals, 3);
    }

    #[test]
    fn test_signal_analysis() {
        let entry = create_test_entry();
        let output = CdiLogger::log_entry(entry).expect("Should succeed");

        let analysis = &output.signal_analysis;

        assert!(!analysis.by_type.is_empty());
        assert!(analysis.avg_score > 0.0);
        assert!(!analysis.top_problems.is_empty());
        assert!(!analysis.top_outcomes.is_empty());
    }

    #[test]
    fn test_recommendations_low_contact_rate() {
        // Add more no-response entries to trigger recommendation
        let entries: Vec<CdiEntry> = (0..5)
            .map(|i| {
                CdiEntry::new(format!("interview-{:03}", i))
                    .with_participant("Test".to_string())
                    .with_outcome(InterviewOutcome::NoResponse)
            })
            .collect();

        let result = CdiLogger::log_entries(entries).expect("Should succeed");

        assert!(
            result
                .recommendations
                .iter()
                .any(|r| r.contains("contact success rate"))
        );
    }

    #[test]
    fn test_get_signals_by_type() {
        let entry = create_test_entry();
        let entries = vec![entry];
        let problems = CdiLogger::get_signals_by_type(&entries, SignalType::Problem);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].content, "Takes too long to generate reports");
    }

    #[test]
    fn test_get_high_signal_quotes() {
        let entry = create_test_entry();
        let entries = vec![entry];
        let quotes = CdiLogger::get_high_signal_quotes(&entries);

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], "I spend 10 hours a week on this!");
    }

    #[test]
    fn test_segment_performance() {
        let entry1 = create_test_entry().with_segment("Enterprise".to_string());
        let entry2 = CdiEntry::new("interview-002".to_string())
            .with_participant("SMB".to_string())
            .with_signal_item(CdiSignal::new(
                "Test".to_string(),
                SignalType::Problem,
                SignalStrength::Low, // Lower signal
            ))
            .with_outcome(InterviewOutcome::Completed)
            .with_segment("SMB".to_string());

        let performance = CdiLogger::segment_performance(&[entry1, entry2]);

        assert_eq!(performance.len(), 2);
        // Enterprise should have higher performance (high signal)
        assert!(performance[0].1 >= performance[1].1);
    }

    #[test]
    fn test_funnel_merge() {
        let mut funnel1 = CdiFunnel::new().with_entry(&create_test_entry());
        let funnel2 = CdiFunnel::new().with_entry(&create_test_entry());

        funnel1.merge(&funnel2);

        assert_eq!(funnel1.contact_attempts, 2);
        assert_eq!(funnel1.completed_interviews, 2);
    }

    #[test]
    fn test_funnel_zero_rates() {
        let funnel = CdiFunnel::new();

        assert!((funnel.contact_success_rate() - 0.0).abs() < 0.01);
        assert!((funnel.interview_completion_rate() - 0.0).abs() < 0.01);
        assert!((funnel.high_signal_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_cdi_entry_with_various_outcomes() {
        let completed = CdiEntry::new("c".to_string())
            .with_participant("Test".to_string())
            .with_outcome(InterviewOutcome::Completed);

        let declined = CdiEntry::new("d".to_string())
            .with_participant("Test".to_string())
            .with_outcome(InterviewOutcome::Declined);

        let no_response = CdiEntry::new("n".to_string())
            .with_participant("Test".to_string())
            .with_outcome(InterviewOutcome::NoResponse);

        let funnel_completed = CdiFunnel::new().with_entry(&completed);
        let funnel_declined = CdiFunnel::new().with_entry(&declined);
        let funnel_no_response = CdiFunnel::new().with_entry(&no_response);

        // Completed should increment all relevant counters
        assert_eq!(funnel_completed.successful_contacts, 1);
        assert_eq!(funnel_completed.completed_interviews, 1);

        // Declined should count as contact but not completion
        assert_eq!(funnel_declined.successful_contacts, 1);
        assert_eq!(funnel_declined.completed_interviews, 0);

        // No response should not count as successful contact
        assert_eq!(funnel_no_response.successful_contacts, 0);
        assert_eq!(funnel_no_response.completed_interviews, 0);
    }
}
