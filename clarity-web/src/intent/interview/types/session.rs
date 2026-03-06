use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
  conflict_detection, Answer, Conflict, ConflictDetectionError, Gap, InterviewError,
  InterviewSession, InterviewSessionError, InterviewStage, Profile,
};

impl InterviewSession {
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

  #[must_use]
  pub const fn get_current_round(&self) -> u32 {
    self.rounds_completed + 1
  }

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
        resolved: false,
        resolution: String::new(),
      })
      .collect()
  }

  #[must_use]
  pub fn get_blocking_gaps(&self) -> Vec<&Gap> {
    self
      .gaps
      .iter()
      .filter(|gap| gap.blocking && !gap.resolved)
      .collect()
  }

  /// # Errors
  /// Returns `InterviewError` if gap is not found or resolution is empty
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

    gap.resolved = true;
    gap.resolution = resolution.to_string();
    self.updated_at = current_unix_timestamp();
    Ok(())
  }

  /// # Errors
  /// Returns `InterviewSessionError` if session is not active or answer is invalid
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

  /// # Errors
  /// Returns `InterviewSessionError` if session is not active or timestamp is empty
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

  /// # Errors
  /// Returns `InterviewSessionError` if there are unresolved blocking gaps
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

  /// # Errors
  /// Returns `InterviewSessionError` if phase number is invalid or timestamp is empty
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

  /// # Errors
  /// Returns `ConflictDetectionError` if session ID is empty or question ID is empty
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

  /// # Errors
  /// Returns `ConflictDetectionError` if conflict is not found or option is invalid
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

    if conflict.chosen.is_some() {
      return Err(ConflictDetectionError::ConflictAlreadyResolved(
        conflict_id.to_string(),
      ));
    }

    let option_count = conflict.options.len();
    let chosen_index = usize::try_from(chosen_option)
      .map_err(|_| ConflictDetectionError::NegativeOptionIndex(chosen_option))?;
    if chosen_index >= option_count {
      return Err(ConflictDetectionError::InvalidOptionIndex {
        conflict_id: conflict_id.to_string(),
        index: chosen_option,
        option_count,
      });
    }

    conflict.chosen = Some(chosen_option);
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
