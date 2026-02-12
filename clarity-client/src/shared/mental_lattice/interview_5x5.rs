//! Interview 5x5 framework.
//!
//! A complete interview matrix with five perspectives and five questions each.

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

pub const QUESTIONS_PER_PERSPECTIVE: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterviewPerspective {
  User,
  Business,
  Technical,
  Operations,
  Risk,
}

impl InterviewPerspective {
  #[must_use]
  pub fn all() -> &'static [Self] {
    &[
      Self::User,
      Self::Business,
      Self::Technical,
      Self::Operations,
      Self::Risk,
    ]
  }
}

impl fmt::Display for InterviewPerspective {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::User => write!(f, "User"),
      Self::Business => write!(f, "Business"),
      Self::Technical => write!(f, "Technical"),
      Self::Operations => write!(f, "Operations"),
      Self::Risk => write!(f, "Risk"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
  High,
  Low,
  Mixed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterviewQuestion {
  pub id: Uuid,
  pub perspective: InterviewPerspective,
  pub prompt: String,
  pub answer: Option<String>,
  pub signal_strength: Option<SignalStrength>,
  pub asked_at: Option<DateTime<Utc>>,
}

impl InterviewQuestion {
  pub fn new(
    perspective: InterviewPerspective,
    prompt: String,
  ) -> Result<Self, InterviewMatrixError> {
    if prompt.trim().is_empty() {
      return Err(InterviewMatrixError::EmptyField("prompt".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      perspective,
      prompt: prompt.trim().to_string(),
      answer: None,
      signal_strength: None,
      asked_at: None,
    })
  }

  #[must_use]
  pub fn with_answer(self, answer: String, volunteered: bool) -> Self {
    let trimmed = answer.trim().to_string();
    let signal_strength = if trimmed.is_empty() {
      SignalStrength::Mixed
    } else if volunteered {
      SignalStrength::High
    } else {
      SignalStrength::Low
    };

    Self {
      signal_strength: Some(signal_strength),
      answer: Some(trimmed),
      asked_at: Some(Utc::now()),
      ..self
    }
  }

  #[must_use]
  pub fn is_answered(&self) -> bool {
    self
      .answer
      .as_ref()
      .is_some_and(|value| !value.trim().is_empty())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterviewMatrix {
  pub id: Uuid,
  pub topic: String,
  pub questions: Vec<InterviewQuestion>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl InterviewMatrix {
  pub fn new(topic: String) -> Result<Self, InterviewMatrixError> {
    if topic.trim().is_empty() {
      return Err(InterviewMatrixError::EmptyField("topic".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      topic: topic.trim().to_string(),
      questions: default_questions()?,
      created_at: now,
      updated_at: now,
    })
  }

  pub fn answer_question(
    &self,
    perspective: InterviewPerspective,
    index: usize,
    answer: String,
    volunteered: bool,
  ) -> Result<Self, InterviewMatrixError> {
    let scoped: Vec<&InterviewQuestion> = self
      .questions
      .iter()
      .filter(|question| question.perspective == perspective)
      .collect();

    if index >= scoped.len() {
      return Err(InterviewMatrixError::InvalidIndex { index });
    }

    let target_id = scoped[index].id;
    let updated_questions = self
      .questions
      .iter()
      .cloned()
      .map(|question| {
        if question.id == target_id {
          question.with_answer(answer.clone(), volunteered)
        } else {
          question
        }
      })
      .collect();

    Ok(Self {
      id: self.id,
      topic: self.topic.clone(),
      questions: updated_questions,
      created_at: self.created_at,
      updated_at: Utc::now(),
    })
  }

  #[must_use]
  pub fn completion_ratio(&self) -> f32 {
    if self.questions.is_empty() {
      return 0.0;
    }
    let answered = self
      .questions
      .iter()
      .filter(|question| question.is_answered())
      .count();
    answered as f32 / self.questions.len() as f32
  }

  pub fn validate_complete(&self) -> Result<(), InterviewMatrixError> {
    let missing = self
      .questions
      .iter()
      .filter(|question| !question.is_answered())
      .count();
    if missing > 0 {
      return Err(InterviewMatrixError::IncompleteMatrix { missing });
    }
    Ok(())
  }
}

fn default_questions() -> Result<Vec<InterviewQuestion>, InterviewMatrixError> {
  let prompts: [(InterviewPerspective, [&str; QUESTIONS_PER_PERSPECTIVE]); 5] = [
    (
      InterviewPerspective::User,
      [
        "What is the main job you are trying to complete?",
        "What currently blocks you most often?",
        "Which workaround do you use today?",
        "What outcome tells you this worked?",
        "What would make you abandon this solution?",
      ],
    ),
    (
      InterviewPerspective::Business,
      [
        "Which business metric should improve first?",
        "What is the cost of delay for this problem?",
        "What assumptions are currently unvalidated?",
        "What trade-off is unacceptable for the business?",
        "What is the minimum viable business impact?",
      ],
    ),
    (
      InterviewPerspective::Technical,
      [
        "What constraints define feasible implementation?",
        "Where is the highest technical risk?",
        "Which integration points are brittle today?",
        "What observability is required at launch?",
        "Which failure mode must be tested first?",
      ],
    ),
    (
      InterviewPerspective::Operations,
      [
        "Who supports this in production?",
        "What runbook update is required?",
        "How will incidents be detected quickly?",
        "What rollout control is needed?",
        "What rollback trigger is mandatory?",
      ],
    ),
    (
      InterviewPerspective::Risk,
      [
        "What abuse case is most likely?",
        "Where could data integrity degrade?",
        "What privacy concern must be resolved?",
        "What compliance requirement applies here?",
        "What is the worst plausible consequence?",
      ],
    ),
  ];

  prompts
    .into_iter()
    .flat_map(|(perspective, entries)| {
      entries
        .into_iter()
        .map(move |prompt| InterviewQuestion::new(perspective, prompt.to_string()))
    })
    .collect()
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum InterviewMatrixError {
  #[error("field cannot be empty: {0}")]
  EmptyField(String),

  #[error("invalid question index: {index}")]
  InvalidIndex { index: usize },

  #[error("interview matrix is incomplete: {missing} answers missing")]
  IncompleteMatrix { missing: usize },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_matrix_contains_twenty_five_questions() {
    let matrix_result = InterviewMatrix::new("onboarding".to_string());
    assert!(matrix_result.is_ok());
    let matrix = match matrix_result {
      Ok(matrix) => matrix,
      Err(_) => return,
    };
    assert_eq!(matrix.questions.len(), 25);
  }

  #[test]
  fn answering_question_updates_completion_ratio() {
    let matrix_result = InterviewMatrix::new("onboarding".to_string());
    assert!(matrix_result.is_ok());
    let matrix = match matrix_result {
      Ok(matrix) => matrix,
      Err(_) => return,
    };

    let updated_result = matrix.answer_question(
      InterviewPerspective::User,
      0,
      "I need to activate my account quickly".to_string(),
      true,
    );
    assert!(updated_result.is_ok());
    let updated = match updated_result {
      Ok(updated) => updated,
      Err(_) => return,
    };

    assert!(updated.completion_ratio() > matrix.completion_ratio());
  }

  #[test]
  fn validate_complete_fails_when_answers_missing() {
    let matrix_result = InterviewMatrix::new("onboarding".to_string());
    assert!(matrix_result.is_ok());
    let matrix = match matrix_result {
      Ok(matrix) => matrix,
      Err(_) => return,
    };

    let result = matrix.validate_complete();
    assert!(matches!(
      result,
      Err(InterviewMatrixError::IncompleteMatrix { missing }) if missing == 25
    ));
  }
}
