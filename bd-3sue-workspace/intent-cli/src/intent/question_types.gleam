/// Question Types
/// Core types for interview questions
/// Used by both interview_questions.gleam and question_loader.gleam
pub type Perspective {
  User
  Developer
  Ops
  Security
  Business
}

pub type QuestionCategory {
  HappyPath
  ErrorCase
  EdgeCase
  Constraint
  Dependency
  NonFunctional
}

pub type QuestionPriority {
  Critical
  Important
  NiceTohave
}

pub type Question {
  Question(
    id: String,
    round: Int,
    perspective: Perspective,
    category: QuestionCategory,
    priority: QuestionPriority,
    question: String,
    context: String,
    example: String,
    expected_type: String,
    extract_into: List(String),
    depends_on: List(String),
    blocks: List(String),
  )
}
