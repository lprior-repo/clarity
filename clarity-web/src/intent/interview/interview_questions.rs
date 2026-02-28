//! Interview Questions Library
//!
//! Types and loader for interview questions.
//! Questions are defined in schema/questions.cue
//!
//! Ported from intent-cli/src/intent/interview_questions.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::question_loader::{get_questions, load_default_questions, QuestionsDatabase};
use super::types::{Perspective, Question, QuestionCategory, QuestionPriority};

/// Get all questions for a specific profile and round
/// Loads questions from CUE file on each call - for repeated calls,
/// use `get_questions_for_round_with_db` with a cached database
#[must_use]
pub fn get_questions_for_round(profile: &str, round: u32) -> Vec<Question> {
  load_default_questions().map_or_else(
    |_| fallback_questions(profile, round),
    |db| get_questions(&db, profile, round),
  )
}

/// Get questions with explicit database (for testing or to avoid reloading)
#[must_use]
pub fn get_questions_for_round_with_db(
  profile: &str,
  round: u32,
  db: &QuestionsDatabase,
) -> Vec<Question> {
  get_questions(db, profile, round)
}

/// Get the next unasked question in the current round
#[must_use]
pub fn get_next_question(profile: &str, round: u32, answered_ids: &[String]) -> Option<Question> {
  let questions = get_questions_for_round(profile, round);
  find_first_unanswered(&questions, answered_ids)
}

/// Find the first unanswered question
fn find_first_unanswered(questions: &[Question], answered: &[String]) -> Option<Question> {
  questions
    .iter()
    .find(|q| !answered.contains(&q.id))
    .cloned()
}

/// Fallback questions if CUE loading fails
fn fallback_questions(profile: &str, round: u32) -> Vec<Question> {
  match round {
    1 => vec![Question {
      id: "fallback-1".to_string(),
      round: 1,
      perspective: Perspective::User,
      category: QuestionCategory::HappyPath,
      priority: QuestionPriority::Critical,
      question: format!("In one sentence, what should this {profile} do?"),
      context: "Questions could not be loaded from CUE. Using fallback.".to_string(),
      example: "Describe the core purpose".to_string(),
      expected_type: "text".to_string(),
      extract_into: vec!["name".to_string()],
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }],
    _ => Vec::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_fallback_questions_round_1() {
    let questions = fallback_questions("api", 1);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].id, "fallback-1");
    assert!(questions[0].question.contains("api"));
  }

  #[test]
  fn test_fallback_questions_other_rounds() {
    let questions = fallback_questions("api", 2);
    assert!(questions.is_empty());

    let questions = fallback_questions("cli", 5);
    assert!(questions.is_empty());
  }

  #[test]
  fn test_find_first_unanswered_empty_questions() {
    let answered = vec!["q1".to_string()];
    let result = find_first_unanswered(&[], &answered);
    assert!(result.is_none());
  }

  #[test]
  fn test_find_first_unanswered_all_answered() {
    let questions = vec![Question {
      id: "q1".to_string(),
      ..Question::default()
    }];
    let answered = vec!["q1".to_string()];
    let result = find_first_unanswered(&questions, &answered);
    assert!(result.is_none());
  }

  #[test]
  fn test_find_first_unanswered_finds_first() {
    let questions = vec![
      Question {
        id: "q1".to_string(),
        ..Question::default()
      },
      Question {
        id: "q2".to_string(),
        ..Question::default()
      },
    ];
    let answered = vec!["q1".to_string()];
    let result = find_first_unanswered(&questions, &answered);
    assert!(result.is_some());
    assert_eq!(result.map(|q| q.id), Some("q2".to_string()));
  }

  #[test]
  fn test_get_questions_for_round_with_db() {
    let mut db = QuestionsDatabase::default();
    db.api.round_1 = vec![Question {
      id: "test-q".to_string(),
      ..Question::default()
    }];

    let questions = get_questions_for_round_with_db("api", 1, &db);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].id, "test-q");
  }

  #[test]
  fn test_get_next_question_returns_none_when_all_answered() {
    let mut db = QuestionsDatabase::default();
    db.api.round_1 = vec![Question {
      id: "q1".to_string(),
      ..Question::default()
    }];

    let answered = vec!["q1".to_string()];
    let result = get_next_question_with_db_test("api", 1, &answered, &db);
    assert!(result.is_none());
  }

  // Helper to test with explicit db
  fn get_next_question_with_db_test(
    profile: &str,
    round: u32,
    answered_ids: &[String],
    db: &QuestionsDatabase,
  ) -> Option<Question> {
    let questions = get_questions_for_round_with_db(profile, round, db);
    find_first_unanswered(&questions, answered_ids)
  }
}
