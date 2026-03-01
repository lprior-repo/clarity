//! Interview session management and operations.
//!
//! This module provides the [`InterviewSession`] implementation with methods
//! for managing the interview lifecycle, including:
//!
//! - Phase completion tracking
//! - Round management
//! - Gap detection and resolution
//! - Conflict detection and resolution
//! - Answer collection
//!
//! See the [module-level documentation](../index.html#phase-and-stage-management)
//! for an overview of phase and stage management.

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use super::models::GapState;
use super::{
  conflict_detection, Answer, Conflict, ConflictDetectionError, Gap, InterviewError,
  InterviewSession, InterviewSessionError, InterviewStage, Profile,
};

impl InterviewSession {
  /// Create a new interview session.
  ///
  /// The session starts in the Discovery stage with `current_phase` set to 1.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use clarity_web::intent::interview::types::{InterviewSession, Profile};
  ///
  /// let session = InterviewSession::new(
  ///     "session-123".to_string(),
  ///     Profile::Api,
  ///     "2024-01-01T00:00:00Z".to_string(),
  /// );
  ///
  /// assert_eq!(session.id, "session-123");
  /// assert_eq!(session.profile, Profile::Api);
  /// assert_eq!(session.current_phase, 1);
  /// assert!(session.completed_phases.is_empty());
  /// ```
  #[must_use]
  pub fn new(id: String, profile: Profile, timestamp: String) -> Self {
    Self {
      id,
      profile,
      created_at: timestamp.clone(),
      updated_at: timestamp,
      stage: InterviewStage::Discovery,
      ..Self::default()
    }
  }

  /// Get the current round number (1-indexed).
  ///
  /// The current round is always `rounds_completed + 1`.
  #[must_use]
  pub const fn get_current_round(&self) -> u32 {
    self.rounds_completed + 1
  }

  /// Detect gaps in required fields based on the profile.
  ///
  /// Returns a list of gaps for required fields that have not been answered.
  #[must_use]
  pub fn detect_gaps(&self) -> Vec<Gap> {
    let required = self.profile.required_fields();
    let answered_fields: HashSet<&str> = self
      .answers
      .iter()
      .flat_map(|answer| answer.extracted.keys())
      .map(String::as_str)
      .collect();

    required
      .iter()
      .filter(|field| !answered_fields.contains(*field))
      .map(|&field| Gap {
        id: format!("gap-{field}"),
        field: field.to_string(),
        description: format!("Missing required field: {field}"),
        blocking: true,
        suggested_default: String::new(),
        why_needed: String::new(),
        round: self.get_current_round(),
        state: GapState::Open,
      })
      .collect()
  }

  /// Get all blocking gaps that are still unresolved.
  #[must_use]
  pub fn get_blocking_gaps(&self) -> Vec<&Gap> {
    self
      .gaps
      .iter()
      .filter(|gap| gap.blocking && !gap.is_resolved())
      .collect()
  }

  /// Resolve a gap by ID with a non-empty resolution.
  ///
  /// # Errors
  ///
  /// - Returns [`InterviewError::EmptyGapId`] if `gap_id` is empty or whitespace.
  /// - Returns [`InterviewError::EmptyResolution`] if `resolution` is empty or whitespace.
  /// - Returns [`InterviewError::GapNotFound`] if no gap matches the ID.
  pub fn resolve_gap(&mut self, gap_id: &str, resolution: &str) -> Result<(), InterviewError> {
    if gap_id.trim().is_empty() {
      return Err(InterviewError::EmptyGapId);
    }
    if resolution.trim().is_empty() {
      return Err(InterviewError::EmptyResolution);
    }

    let gap = self
      .gaps
      .iter_mut()
      .find(|gap| gap.id == gap_id)
      .ok_or_else(|| InterviewError::GapNotFound(gap_id.to_string()))?;

    gap.state = GapState::Resolved {
      resolution: resolution.to_string(),
    };
    self.updated_at = current_unix_timestamp();
    Ok(())
  }

  /// Add an answer to the current round.
  ///
  /// # Errors
  ///
  /// - Returns [`InterviewSessionError::SessionPaused`] if the session is paused.
  /// - Returns [`InterviewSessionError::AlreadyComplete`] if the session is complete.
  /// - Returns [`InterviewSessionError::EmptyQuestionId`] if the answer has no question ID.
  /// - Returns [`InterviewSessionError::EmptyTimestamp`] if the timestamp is empty.
  /// - Returns [`InterviewSessionError::RoundMismatch`] if the answer's round doesn't match.
  /// - Returns [`InterviewSessionError::DuplicateAnswer`] if this question was already answered in this round.
  pub fn add_answer(
    &mut self,
    answer: Answer,
    timestamp: &str,
  ) -> Result<(), InterviewSessionError> {
    if self.stage == InterviewStage::Paused {
      return Err(InterviewSessionError::SessionPaused);
    }
    if self.stage == InterviewStage::Complete {
      return Err(InterviewSessionError::AlreadyComplete);
    }
    if answer.question_id.is_empty() {
      return Err(InterviewSessionError::EmptyQuestionId);
    }
    if timestamp.is_empty() {
      return Err(InterviewSessionError::EmptyTimestamp);
    }

    let current_round = self.get_current_round();
    if answer.round != current_round {
      return Err(InterviewSessionError::RoundMismatch {
        answer_round: answer.round,
        current_round,
      });
    }

    if self
      .answers
      .iter()
      .any(|existing| existing.question_id == answer.question_id && existing.round == current_round)
    {
      return Err(InterviewSessionError::DuplicateAnswer {
        question_id: answer.question_id,
        round: current_round,
      });
    }

    self.answers.push(answer);
    self.updated_at = timestamp.to_string();
    Ok(())
  }

  /// Calculate confidence score for an answer.
  ///
  /// Returns a confidence score based on response length and extracted fields:
  /// - 0.85 if response is > 50 characters and has extracted fields
  /// - 0.60 otherwise
  #[must_use]
  pub fn calculate_confidence(
    response: &str,
    extracted_fields: &std::collections::HashMap<String, String>,
  ) -> f64 {
    let response_length = response.chars().count();
    let has_extracted_fields = !extracted_fields.is_empty();

    if response_length > 50 && has_extracted_fields {
      0.85
    } else {
      0.6
    }
  }

  /// Mark the current round as complete and advance stage if needed.
  ///
  /// This method:
  /// 1. Increments `rounds_completed`
  /// 2. Updates the stage based on rounds completed:
  ///    - Rounds 1-2: Discovery
  ///    - Round 3: Refinement
  ///    - Round 4: Validation
  ///    - Round 5+: Complete
  /// 3. Sets `completed_at` if the session is now Complete
  ///
  /// # Errors
  ///
  /// - Returns [`InterviewSessionError::SessionPaused`] if the session is paused.
  /// - Returns [`InterviewSessionError::AlreadyComplete`] if the session is complete.
  /// - Returns [`InterviewSessionError::EmptyTimestamp`] if the timestamp is empty.
  pub fn complete_round(&mut self, timestamp: &str) -> Result<(), InterviewSessionError> {
    if self.stage == InterviewStage::Paused {
      return Err(InterviewSessionError::SessionPaused);
    }
    if self.stage == InterviewStage::Complete {
      return Err(InterviewSessionError::AlreadyComplete);
    }
    if timestamp.is_empty() {
      return Err(InterviewSessionError::EmptyTimestamp);
    }

    self.rounds_completed += 1;
    self.stage = match self.rounds_completed {
      1 | 2 => InterviewStage::Discovery,
      3 => InterviewStage::Refinement,
      4 => InterviewStage::Validation,
      _ => InterviewStage::Complete,
    };
    self.updated_at = timestamp.to_string();

    if self.stage == InterviewStage::Complete {
      self.completed_at = Some(timestamp.to_string());
    }
    Ok(())
  }

  /// Validate that no unresolved blocking gaps remain.
  ///
  /// # Errors
  ///
  /// Returns [`InterviewSessionError::BlockingGapsUnresolved`] if any blocking
  /// gaps are still open, containing the count and IDs of unresolved gaps.
  pub fn can_proceed(&self) -> Result<(), InterviewSessionError> {
    let gap_ids: Vec<String> = self
      .get_blocking_gaps()
      .into_iter()
      .map(|gap| gap.id.clone())
      .collect();

    if gap_ids.is_empty() {
      Ok(())
    } else {
      Err(InterviewSessionError::BlockingGapsUnresolved {
        count: gap_ids.len(),
        gap_ids,
      })
    }
  }

  /// Mark a phase as complete and optionally advance `current_phase`.
  ///
  /// This method tracks fine-grained progress within an interview session.
  /// Unlike rounds, phases can be completed in any order.
  ///
  /// # Behavior
  ///
  /// - Adds `phase_number` to `completed_phases` (idempotent - won't duplicate)
  /// - Advances `current_phase` only if completing the current phase
  /// - Updates `updated_at` timestamp
  /// - Works regardless of the current [`InterviewStage`]
  ///
  /// # Phase Rules
  ///
  /// - Phases are 1-indexed (phase 0 is invalid)
  /// - Phases can be completed out of order
  /// - `current_phase` only advances when the *current* phase is completed
  /// - Completing the same phase multiple times is idempotent
  ///
  /// # Errors
  ///
  /// - Returns [`InterviewSessionError::InvalidPhaseNumber`] if `phase_number` is 0.
  /// - Returns [`InterviewSessionError::EmptyTimestamp`] if `timestamp` is empty.
  ///
  /// # Examples
  ///
  /// ## Sequential Phase Completion
  ///
  /// ```ignore
  /// use clarity_web::intent::interview::types::{InterviewSession, Profile};
  ///
  /// let mut session = InterviewSession::new(
  ///     "session-1".into(),
  ///     Profile::Api,
  ///     "t1".into(),
  /// );
  ///
  /// // Initially at phase 1
  /// assert_eq!(session.current_phase, 1);
  /// assert!(session.completed_phases.is_empty());
  ///
  /// // Complete phase 1
  /// session.complete_phase(1, "t2").unwrap();
  /// assert_eq!(session.current_phase, 2);
  /// assert_eq!(session.completed_phases, vec![1]);
  /// ```
  ///
  /// ## Out-of-Order Phase Completion
  ///
  /// ```ignore
  /// // Complete phase 3 before phase 1
  /// session.complete_phase(3, "t1").unwrap();
  /// assert!(session.completed_phases.contains(&3));
  /// assert_eq!(session.current_phase, 1); // unchanged - not the current phase
  ///
  /// // Now complete phase 1 (current)
  /// session.complete_phase(1, "t2").unwrap();
  /// assert_eq!(session.current_phase, 2); // advances now
  /// ```
  pub fn complete_phase(
    &mut self,
    phase_number: u32,
    timestamp: &str,
  ) -> Result<(), InterviewSessionError> {
    if phase_number == 0 {
      return Err(InterviewSessionError::InvalidPhaseNumber { phase_number });
    }
    if timestamp.is_empty() {
      return Err(InterviewSessionError::EmptyTimestamp);
    }

    if !self.completed_phases.contains(&phase_number) {
      self.completed_phases.push(phase_number);
    }
    if self.current_phase == phase_number {
      self.current_phase = phase_number + 1;
    }

    self.updated_at = timestamp.to_string();
    Ok(())
  }

  /// Detect and append newly found conflicts.
  ///
  /// Analyzes all answers for conflicts and appends any new ones to the session.
  ///
  /// # Errors
  ///
  /// - Returns [`ConflictDetectionError::EmptySessionId`] if the session ID is empty.
  /// - Returns [`ConflictDetectionError::EmptyQuestionId`] if an answer has no question ID.
  pub fn detect_conflicts(&mut self) -> Result<Vec<Conflict>, ConflictDetectionError> {
    if self.id.is_empty() {
      return Err(ConflictDetectionError::EmptySessionId);
    }

    if let Some(index) = self
      .answers
      .iter()
      .position(|answer| answer.question_id.is_empty())
    {
      return Err(ConflictDetectionError::EmptyQuestionId(index));
    }

    let new_conflicts = conflict_detection::detect_conflicts(&self.answers);
    if !new_conflicts.is_empty() {
      self.updated_at = current_unix_timestamp();
      self.conflicts.extend(new_conflicts.clone());
    }

    Ok(new_conflicts)
  }

  /// Resolve a conflict by selecting an option index.
  ///
  /// # Errors
  ///
  /// - Returns [`ConflictDetectionError::EmptyConflictId`] if `conflict_id` is empty.
  /// - Returns [`ConflictDetectionError::NegativeOptionIndex`] if `chosen_option` is negative.
  /// - Returns [`ConflictDetectionError::ConflictNotFound`] if no conflict matches the ID.
  /// - Returns [`ConflictDetectionError::ConflictAlreadyResolved`] if the conflict is already resolved.
  /// - Returns [`ConflictDetectionError::InvalidOptionIndex`] if the index is out of bounds.
  /// - Returns [`ConflictDetectionError::EmptyOptions`] if the conflict has no options.
  pub fn resolve_conflict(
    &mut self,
    conflict_id: &str,
    chosen_option: i32,
  ) -> Result<(), ConflictDetectionError> {
    if conflict_id.trim().is_empty() {
      return Err(ConflictDetectionError::EmptyConflictId);
    }
    if chosen_option < 0 {
      return Err(ConflictDetectionError::NegativeOptionIndex(chosen_option));
    }

    let conflict = self
      .conflicts
      .iter_mut()
      .find(|conflict| conflict.id == conflict_id)
      .ok_or_else(|| ConflictDetectionError::ConflictNotFound(conflict_id.to_string()))?;

    if conflict.is_resolved() {
      return Err(ConflictDetectionError::ConflictAlreadyResolved(
        conflict_id.to_string(),
      ));
    }

    let option_count = conflict.options.len();
    let new_state = conflict
      .state
      .resolve(chosen_option, option_count)
      .map_err(|e| match e {
        super::models::ConflictStateError::NegativeIndex(idx) => {
          ConflictDetectionError::NegativeOptionIndex(idx)
        }
        super::models::ConflictStateError::AlreadyResolved => {
          ConflictDetectionError::ConflictAlreadyResolved(conflict_id.to_string())
        }
        super::models::ConflictStateError::InvalidIndex {
          index,
          option_count,
        } => ConflictDetectionError::InvalidOptionIndex {
          conflict_id: conflict_id.to_string(),
          index,
          option_count,
        },
        super::models::ConflictStateError::EmptyOptions => ConflictDetectionError::EmptyOptions,
      })?;

    conflict.state = new_state;
    self.updated_at = current_unix_timestamp();
    Ok(())
  }
}

fn current_unix_timestamp() -> String {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |duration| duration.as_secs())
    .to_string()
}
