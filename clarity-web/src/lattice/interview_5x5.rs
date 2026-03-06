#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Interview 5x5 module for structured requirements gathering.
//!
//! This module implements the 5x5 interview technique for comprehensive
//! requirements elicitation: 5 questions x 5 perspectives.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for 5x5 interview
#[derive(Debug, Error, PartialEq, Clone)]
pub enum InterviewError {
  #[error("question text is empty")]
  EmptyQuestion,

  #[error("answer text is empty")]
  EmptyAnswer,

  #[error("perspective not found: {0}")]
  PerspectiveNotFound(String),

  #[error("question type not found: {0}")]
  QuestionTypeNotFound(String),

  #[error("interview incomplete: {0} questions unanswered")]
  IncompleteInterview(usize),
}

/// The 5 perspectives for requirements gathering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Perspective {
  /// What: Functional requirements
  What,
  /// Why: Business rationale
  Why,
  /// Who: Stakeholders and users
  Who,
  /// When: Timing and sequencing
  When,
  /// Where: Context and environment
  Where,
}

impl Perspective {
  /// Get all perspectives
  #[must_use]
  pub const fn all() -> [Self; 5] {
    [Self::What, Self::Why, Self::Who, Self::When, Self::Where]
  }

  /// Get label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::What => "What",
      Self::Why => "Why",
      Self::Who => "Who",
      Self::When => "When",
      Self::Where => "Where",
    }
  }

  /// Get description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::What => "Functional requirements and capabilities",
      Self::Why => "Business rationale and objectives",
      Self::Who => "Stakeholders, users, and actors",
      Self::When => "Timing, sequencing, and conditions",
      Self::Where => "Context, environment, and constraints",
    }
  }

  /// Get suggested questions for this perspective
  #[must_use]
  pub fn suggested_questions(&self) -> Vec<String> {
    match self {
      Self::What => vec![
        "What must the system do?".to_string(),
        "What are the main functions?".to_string(),
        "What data is processed?".to_string(),
        "What outputs are expected?".to_string(),
        "What are the boundaries?".to_string(),
      ],
      Self::Why => vec![
        "Why is this system needed?".to_string(),
        "Why is this requirement important?".to_string(),
        "Why now?".to_string(),
        "Why this approach?".to_string(),
        "Why should we prioritize this?".to_string(),
      ],
      Self::Who => vec![
        "Who will use this system?".to_string(),
        "Who are the stakeholders?".to_string(),
        "Who maintains the system?".to_string(),
        "Who provides input?".to_string(),
        "Who receives output?".to_string(),
      ],
      Self::When => vec![
        "When does this need to happen?".to_string(),
        "When should the system be available?".to_string(),
        "When are the deadlines?".to_string(),
        "When do events trigger actions?".to_string(),
        "When is the system unavailable?".to_string(),
      ],
      Self::Where => vec![
        "Where will the system operate?".to_string(),
        "Where are the users located?".to_string(),
        "Where is data stored?".to_string(),
        "Where are the constraints?".to_string(),
        "Where does this fit in the ecosystem?".to_string(),
      ],
    }
  }
}

/// The 5 question types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestionType {
  /// Open-ended exploration
  Open,
  /// Specific details
  Detail,
  /// Hypothetical scenarios
  Scenario,
  /// Risks and concerns
  Risk,
  /// Success criteria
  Success,
}

impl QuestionType {
  /// Get all question types
  #[must_use]
  pub const fn all() -> [Self; 5] {
    [
      Self::Open,
      Self::Detail,
      Self::Scenario,
      Self::Risk,
      Self::Success,
    ]
  }

  /// Get label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Open => "Open",
      Self::Detail => "Detail",
      Self::Scenario => "Scenario",
      Self::Risk => "Risk",
      Self::Success => "Success",
    }
  }

  /// Get description
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Open => "Open-ended exploration questions",
      Self::Detail => "Specific detail questions",
      Self::Scenario => "Hypothetical scenario questions",
      Self::Risk => "Risk and concern questions",
      Self::Success => "Success criteria questions",
    }
  }
}

/// A single interview question
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
  /// Unique identifier
  pub id: String,
  /// The question text
  pub text: String,
  /// Associated perspective
  pub perspective: Perspective,
  /// Question type
  pub question_type: QuestionType,
  /// Whether the question is required
  pub required: bool,
}

impl Question {
  /// Create a new question
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::EmptyQuestion` if text is empty
  pub fn new(
    id: String,
    text: String,
    perspective: Perspective,
    question_type: QuestionType,
  ) -> Result<Self, InterviewError> {
    if text.trim().is_empty() {
      return Err(InterviewError::EmptyQuestion);
    }

    Ok(Self {
      id,
      text,
      perspective,
      question_type,
      required: true,
    })
  }

  /// Set required flag
  #[must_use]
  pub fn with_required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }
}

/// An answer to an interview question
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
  /// The question being answered
  pub question_id: String,
  /// The answer text
  pub text: String,
  /// Confidence level (0-100)
  pub confidence: u8,
  /// Whether this answer needs follow-up
  pub needs_follow_up: bool,
}

impl Answer {
  /// Create a new answer
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::EmptyAnswer` if text is empty
  pub fn new(question_id: String, text: String) -> Result<Self, InterviewError> {
    if text.trim().is_empty() {
      return Err(InterviewError::EmptyAnswer);
    }

    Ok(Self {
      question_id,
      text,
      confidence: 100,
      needs_follow_up: false,
    })
  }

  /// Set confidence level
  #[must_use]
  pub fn with_confidence(mut self, confidence: u8) -> Self {
    self.confidence = confidence.min(100);
    self
  }

  /// Mark as needing follow-up
  #[must_use]
  pub fn with_follow_up(mut self, needs_follow_up: bool) -> Self {
    self.needs_follow_up = needs_follow_up;
    self
  }
}

/// Complete 5x5 interview matrix
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interview5x5 {
  /// Interview topic/subject
  pub topic: String,
  /// All questions (5 perspectives x 5 types = 25)
  pub questions: Vec<Question>,
  /// Answers provided
  pub answers: Vec<Answer>,
  /// Interview completion status
  pub completion_percentage: u8,
  /// Overall coverage score
  pub coverage_score: u8,
}

impl Interview5x5 {
  /// Create a new 5x5 interview structure
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::EmptyQuestion` if topic is empty
  pub fn new(topic: String) -> Result<Self, InterviewError> {
    if topic.trim().is_empty() {
      return Err(InterviewError::EmptyQuestion);
    }

    let questions = generate_default_questions();
    let answers = Vec::new();

    Ok(Self {
      topic,
      questions,
      answers,
      completion_percentage: 0,
      coverage_score: 0,
    })
  }

  /// Add an answer to the interview
  pub fn add_answer(&mut self, answer: Answer) {
    // Remove existing answer for same question
    self.answers.retain(|a| a.question_id != answer.question_id);

    self.answers.push(answer);
    self.update_scores();
  }

  /// Update completion and coverage scores
  fn update_scores(&mut self) {
    let required_questions: Vec<&Question> = self.questions.iter().filter(|q| q.required).collect();

    let answered_required = required_questions
      .iter()
      .filter(|q| self.answers.iter().any(|a| a.question_id == q.id))
      .count();

    let total = required_questions.len();
    self.completion_percentage = if total > 0 {
      ((answered_required * 100) / total) as u8
    } else {
      0
    };

    // Coverage: weighted by perspective and question type coverage
    let perspectives_covered = Perspective::all()
      .iter()
      .filter(|p| self.is_perspective_covered(**p))
      .count();

    let types_covered = QuestionType::all()
      .iter()
      .filter(|t| self.is_question_type_covered(**t))
      .count();

    self.coverage_score = (((perspectives_covered * 10) + (types_covered * 10)) / 2) as u8;
  }

  /// Check if a perspective has at least one answer
  fn is_perspective_covered(&self, perspective: Perspective) -> bool {
    self
      .questions
      .iter()
      .filter(|q| q.perspective == perspective)
      .any(|q| self.answers.iter().any(|a| a.question_id == q.id))
  }

  /// Check if a question type has at least one answer
  fn is_question_type_covered(&self, question_type: QuestionType) -> bool {
    self
      .questions
      .iter()
      .filter(|q| q.question_type == question_type)
      .any(|q| self.answers.iter().any(|a| a.question_id == q.id))
  }

  /// Get questions for a specific perspective
  #[must_use]
  pub fn questions_by_perspective(&self, perspective: Perspective) -> Vec<&Question> {
    self
      .questions
      .iter()
      .filter(|q| q.perspective == perspective)
      .collect()
  }

  /// Get questions for a specific type
  #[must_use]
  pub fn questions_by_type(&self, question_type: QuestionType) -> Vec<&Question> {
    self
      .questions
      .iter()
      .filter(|q| q.question_type == question_type)
      .collect()
  }

  /// Get unanswered questions
  #[must_use]
  pub fn unanswered_questions(&self) -> Vec<&Question> {
    self
      .questions
      .iter()
      .filter(|q| !self.answers.iter().any(|a| a.question_id == q.id))
      .collect()
  }

  /// Get answers needing follow-up
  #[must_use]
  pub fn answers_needing_follow_up(&self) -> Vec<&Answer> {
    self.answers.iter().filter(|a| a.needs_follow_up).collect()
  }

  /// Check if interview is complete
  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.completion_percentage == 100
  }

  /// Get coverage gaps
  #[must_use]
  pub fn coverage_gaps(&self) -> Vec<CoverageGap> {
    let mut gaps = Vec::new();

    // Check perspective coverage
    for perspective in Perspective::all() {
      if !self.is_perspective_covered(perspective) {
        gaps.push(CoverageGap::Perspective(perspective));
      }
    }

    // Check question type coverage
    for question_type in QuestionType::all() {
      if !self.is_question_type_covered(question_type) {
        gaps.push(CoverageGap::QuestionType(question_type));
      }
    }

    gaps
  }

  /// Export as requirements text
  #[must_use]
  pub fn to_requirements(&self) -> String {
    self
      .answers
      .iter()
      .sorted_by(|a1, a2| {
        let q1 = self.questions.iter().find(|q| q.id == a1.question_id);
        let q2 = self.questions.iter().find(|q| q.id == a2.question_id);

        match (q1, q2) {
          (Some(q1), Some(q2)) => {
            let p1 = q1.perspective as u8;
            let p2 = q2.perspective as u8;
            p1.cmp(&p2)
          }
          _ => std::cmp::Ordering::Equal,
        }
      })
      .filter_map(|a| {
        self
          .questions
          .iter()
          .find(|q| q.id == a.question_id)
          .map(|q| format!("[{}] {}: {}", q.perspective.label(), q.text, a.text))
      })
      .join("\n\n")
  }
}

/// Coverage gap types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageGap {
  /// Missing perspective coverage
  Perspective(Perspective),
  /// Missing question type coverage
  QuestionType(QuestionType),
}

impl CoverageGap {
  /// Get description of the gap
  #[must_use]
  pub fn description(&self) -> String {
    match self {
      Self::Perspective(p) => format!("No answers for {} perspective", p.label()),
      Self::QuestionType(t) => format!("No answers for {} questions", t.label()),
    }
  }
}

/// Generate default 25 questions (5x5)
fn generate_default_questions() -> Vec<Question> {
  let mut questions = Vec::new();
  let mut id = 0;

  for perspective in Perspective::all() {
    for question_type in QuestionType::all() {
      id += 1;
      let text = generate_question_text(perspective, question_type);

      if let Ok(question) = Question::new(format!("Q-{id:02}"), text, perspective, question_type) {
        questions.push(question);
      }
    }
  }

  questions
}

/// Generate question text based on perspective and type
fn generate_question_text(perspective: Perspective, question_type: QuestionType) -> String {
  match (perspective, question_type) {
    // What questions
    (Perspective::What, QuestionType::Open) => "What is the primary function?".to_string(),
    (Perspective::What, QuestionType::Detail) => "What specific features are required?".to_string(),
    (Perspective::What, QuestionType::Scenario) => "What happens in edge cases?".to_string(),
    (Perspective::What, QuestionType::Risk) => "What could go wrong?".to_string(),
    (Perspective::What, QuestionType::Success) => "What defines success?".to_string(),

    // Why questions
    (Perspective::Why, QuestionType::Open) => "Why is this needed?".to_string(),
    (Perspective::Why, QuestionType::Detail) => {
      "Why are these specific requirements important?".to_string()
    }
    (Perspective::Why, QuestionType::Scenario) => "Why would stakeholders use this?".to_string(),
    (Perspective::Why, QuestionType::Risk) => "Why might this fail?".to_string(),
    (Perspective::Why, QuestionType::Success) => "Why would this be successful?".to_string(),

    // Who questions
    (Perspective::Who, QuestionType::Open) => "Who are the users?".to_string(),
    (Perspective::Who, QuestionType::Detail) => "Who specifically needs this feature?".to_string(),
    (Perspective::Who, QuestionType::Scenario) => "Who would be affected by problems?".to_string(),
    (Perspective::Who, QuestionType::Risk) => "Who might oppose this?".to_string(),
    (Perspective::Who, QuestionType::Success) => "Who benefits most from success?".to_string(),

    // When questions
    (Perspective::When, QuestionType::Open) => "When does this need to happen?".to_string(),
    (Perspective::When, QuestionType::Detail) => {
      "When specifically must each phase complete?".to_string()
    }
    (Perspective::When, QuestionType::Scenario) => "When would usage peak?".to_string(),
    (Perspective::When, QuestionType::Risk) => "When are we most vulnerable?".to_string(),
    (Perspective::When, QuestionType::Success) => "When will we know we succeeded?".to_string(),

    // Where questions
    (Perspective::Where, QuestionType::Open) => "Where will this operate?".to_string(),
    (Perspective::Where, QuestionType::Detail) => {
      "Where are the specific deployment targets?".to_string()
    }
    (Perspective::Where, QuestionType::Scenario) => "Where might context change?".to_string(),
    (Perspective::Where, QuestionType::Risk) => "Where are the weak points?".to_string(),
    (Perspective::Where, QuestionType::Success) => "Where should we focus testing?".to_string(),
  }
}

/// Conduct a quick 5x5 analysis on requirements text
///
/// # Arguments
/// * `text` - Requirements text to analyze
///
/// # Returns
/// Coverage analysis showing which perspectives are addressed
#[must_use]
pub fn analyze_coverage(text: &str) -> PerspectiveCoverage {
  let lower = text.to_lowercase();
  let mut covered = Vec::new();

  // Check for What indicators
  let what_indicators = [
    "shall",
    "must",
    "will",
    "function",
    "feature",
    "requirement",
  ];
  if what_indicators.iter().any(|i| lower.contains(i)) {
    covered.push(Perspective::What);
  }

  // Check for Why indicators
  let why_indicators = [
    "because",
    "reason",
    "objective",
    "goal",
    "purpose",
    "business",
  ];
  if why_indicators.iter().any(|i| lower.contains(i)) {
    covered.push(Perspective::Why);
  }

  // Check for Who indicators
  let actor_indicators = [
    "user",
    "stakeholder",
    "actor",
    "customer",
    "admin",
    "person",
  ];
  if actor_indicators.iter().any(|i| lower.contains(i)) {
    covered.push(Perspective::Who);
  }

  // Check for When indicators
  let when_indicators = [
    "when",
    "timing",
    "schedule",
    "deadline",
    "phase",
    "milestone",
  ];
  if when_indicators.iter().any(|i| lower.contains(i)) {
    covered.push(Perspective::When);
  }

  // Check for Where indicators
  let where_indicators = [
    "where",
    "environment",
    "platform",
    "location",
    "context",
    "infrastructure",
  ];
  if where_indicators.iter().any(|i| lower.contains(i)) {
    covered.push(Perspective::Where);
  }

  let missing: Vec<Perspective> = Perspective::all()
    .iter()
    .filter(|p| !covered.contains(p))
    .copied()
    .collect();

  let coverage_percentage = ((covered.len() * 100) / 5) as u8;

  PerspectiveCoverage {
    covered,
    missing,
    coverage_percentage,
  }
}

/// Perspective coverage analysis result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerspectiveCoverage {
  /// Perspectives covered
  pub covered: Vec<Perspective>,
  /// Perspectives missing
  pub missing: Vec<Perspective>,
  /// Coverage percentage (0-100)
  pub coverage_percentage: u8,
}

impl PerspectiveCoverage {
  /// Check if complete
  #[must_use]
  pub const fn is_complete(&self) -> bool {
    self.coverage_percentage == 100
  }

  /// Get suggestions for missing perspectives
  #[must_use]
  pub fn suggestions(&self) -> Vec<String> {
    self
      .missing
      .iter()
      .map(|p| {
        let questions = p.suggested_questions();
        format!(
          "Consider {} perspective: {}",
          p.label(),
          questions
            .first()
            .map_or("Add relevant details", |q| q.as_str())
        )
      })
      .collect()
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  #[test]
  fn test_perspective_labels() {
    assert_eq!(Perspective::What.label(), "What");
    assert_eq!(Perspective::Why.label(), "Why");
    assert_eq!(Perspective::Who.label(), "Who");
    assert_eq!(Perspective::When.label(), "When");
    assert_eq!(Perspective::Where.label(), "Where");
  }

  #[test]
  fn test_perspective_suggested_questions() {
    for perspective in Perspective::all() {
      let questions = perspective.suggested_questions();
      assert_eq!(questions.len(), 5);
    }
  }

  #[test]
  fn test_question_type_labels() {
    assert_eq!(QuestionType::Open.label(), "Open");
    assert_eq!(QuestionType::Detail.label(), "Detail");
  }

  #[test]
  fn test_question_new_valid() {
    let question = Question::new(
      "Q-01".to_string(),
      "What is the purpose?".to_string(),
      Perspective::What,
      QuestionType::Open,
    );

    assert!(question.is_ok());
    let q = question.unwrap();
    assert_eq!(q.id, "Q-01");
    assert!(q.required);
  }

  #[test]
  fn test_question_new_empty_text() {
    let result = Question::new(
      "Q-01".to_string(),
      "".to_string(),
      Perspective::What,
      QuestionType::Open,
    );

    assert!(matches!(result, Err(InterviewError::EmptyQuestion)));
  }

  #[test]
  fn test_answer_new_valid() {
    let answer = Answer::new("Q-01".to_string(), "This is the answer.".to_string());

    assert!(answer.is_ok());
    let a = answer.unwrap();
    assert_eq!(a.confidence, 100);
    assert!(!a.needs_follow_up);
  }

  #[test]
  fn test_answer_new_empty_text() {
    let result = Answer::new("Q-01".to_string(), "   ".to_string());

    assert!(matches!(result, Err(InterviewError::EmptyAnswer)));
  }

  #[test]
  fn test_answer_builder_methods() {
    let answer = Answer::new("Q-01".to_string(), "Answer text.".to_string())
      .unwrap()
      .with_confidence(75)
      .with_follow_up(true);

    assert_eq!(answer.confidence, 75);
    assert!(answer.needs_follow_up);
  }

  #[test]
  fn test_interview_5x5_new() {
    let interview = Interview5x5::new("Test Topic".to_string());

    assert!(interview.is_ok());
    let i = interview.unwrap();
    assert_eq!(i.questions.len(), 25); // 5x5
    assert!(i.answers.is_empty());
  }

  #[test]
  fn test_interview_5x5_empty_topic() {
    let result = Interview5x5::new("".to_string());
    assert!(matches!(result, Err(InterviewError::EmptyQuestion)));
  }

  #[test]
  fn test_interview_add_answer() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    let answer = Answer::new("Q-01".to_string(), "Answer.".to_string()).unwrap();
    interview.add_answer(answer);

    assert_eq!(interview.answers.len(), 1);
    assert!(interview.completion_percentage > 0);
  }

  #[test]
  fn test_interview_questions_by_perspective() {
    let interview = Interview5x5::new("Test".to_string()).unwrap();

    let what_questions = interview.questions_by_perspective(Perspective::What);
    assert_eq!(what_questions.len(), 5); // 5 question types
  }

  #[test]
  fn test_interview_questions_by_type() {
    let interview = Interview5x5::new("Test".to_string()).unwrap();

    let open_questions = interview.questions_by_type(QuestionType::Open);
    assert_eq!(open_questions.len(), 5); // 5 perspectives
  }

  #[test]
  fn test_interview_unanswered_questions() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    assert_eq!(interview.unanswered_questions().len(), 25);

    let answer = Answer::new("Q-01".to_string(), "Answer.".to_string()).unwrap();
    interview.add_answer(answer);

    assert_eq!(interview.unanswered_questions().len(), 24);
  }

  #[test]
  fn test_interview_completion_percentage() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    // Initially 0%
    assert_eq!(interview.completion_percentage, 0);

    // Answer all questions
    for question in interview.questions.clone() {
      let answer = Answer::new(question.id.clone(), "Answer.".to_string()).unwrap();
      interview.add_answer(answer);
    }

    assert_eq!(interview.completion_percentage, 100);
  }

  #[test]
  fn test_interview_coverage_gaps() {
    let interview = Interview5x5::new("Test".to_string()).unwrap();

    let gaps = interview.coverage_gaps();
    assert!(!gaps.is_empty()); // All perspectives and types missing
  }

  #[test]
  fn test_interview_to_requirements() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    let answer = Answer::new("Q-01".to_string(), "Test answer.".to_string()).unwrap();
    interview.add_answer(answer);

    let requirements = interview.to_requirements();
    assert!(requirements.contains("Test answer."));
  }

  #[test]
  fn test_analyze_coverage_all_perspectives() {
    let text = "The user shall authenticate. The reason is security. \
                    The system must function when needed. The platform is cloud-based.";

    let coverage = analyze_coverage(text);

    assert!(!coverage.covered.is_empty());
  }

  #[test]
  fn test_analyze_coverage_empty_text() {
    let coverage = analyze_coverage("");

    assert!(coverage.covered.is_empty());
    assert_eq!(coverage.coverage_percentage, 0);
  }

  #[test]
  fn test_analyze_coverage_what_only() {
    let text = "The system shall process data.";

    let coverage = analyze_coverage(text);

    assert!(coverage.covered.contains(&Perspective::What));
  }

  #[test]
  fn test_perspective_coverage_suggestions() {
    let coverage = PerspectiveCoverage {
      covered: vec![Perspective::What],
      missing: vec![Perspective::Why, Perspective::Who],
      coverage_percentage: 20,
    };

    let suggestions = coverage.suggestions();
    assert_eq!(suggestions.len(), 2);
  }

  #[test]
  fn test_perspective_coverage_is_complete() {
    let complete = PerspectiveCoverage {
      covered: Perspective::all().to_vec(),
      missing: vec![],
      coverage_percentage: 100,
    };

    assert!(complete.is_complete());

    let incomplete = PerspectiveCoverage {
      covered: vec![Perspective::What],
      missing: vec![Perspective::Why],
      coverage_percentage: 20,
    };

    assert!(!incomplete.is_complete());
  }

  #[test]
  fn test_coverage_gap_description() {
    let gap = CoverageGap::Perspective(Perspective::Why);
    assert!(gap.description().contains("Why"));

    let gap = CoverageGap::QuestionType(QuestionType::Risk);
    assert!(gap.description().contains("Risk"));
  }

  #[test]
  fn test_answers_needing_follow_up() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    let answer1 = Answer::new("Q-01".to_string(), "Answer 1.".to_string())
      .unwrap()
      .with_follow_up(true);
    let answer2 = Answer::new("Q-02".to_string(), "Answer 2.".to_string())
      .unwrap()
      .with_follow_up(false);

    interview.add_answer(answer1);
    interview.add_answer(answer2);

    let follow_ups = interview.answers_needing_follow_up();
    assert_eq!(follow_ups.len(), 1);
  }

  #[test]
  fn test_interview_is_complete() {
    let mut interview = Interview5x5::new("Test".to_string()).unwrap();

    assert!(!interview.is_complete());

    // Answer all questions
    for question in interview.questions.clone() {
      let answer = Answer::new(question.id.clone(), "Answer.".to_string()).unwrap();
      interview.add_answer(answer);
    }

    assert!(interview.is_complete());
  }

  #[test]
  fn test_default_questions_generation() {
    let questions = generate_default_questions();

    assert_eq!(questions.len(), 25);

    // Check each perspective has 5 questions
    for perspective in Perspective::all() {
      let count = questions
        .iter()
        .filter(|q| q.perspective == perspective)
        .count();
      assert_eq!(count, 5);
    }

    // Check each type has 5 questions
    for question_type in QuestionType::all() {
      let count = questions
        .iter()
        .filter(|q| q.question_type == question_type)
        .count();
      assert_eq!(count, 5);
    }
  }
}
