use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{InterviewStage, Perspective, Profile, QuestionCategory, QuestionPriority};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
  pub question_id: String,
  pub question_text: String,
  pub perspective: Perspective,
  pub round: u32,
  pub response: String,
  pub extracted: HashMap<String, String>,
  pub confidence: f64,
  pub notes: String,
  pub timestamp: String,
}

impl Default for Answer {
  fn default() -> Self {
    Self {
      question_id: String::new(),
      question_text: String::new(),
      perspective: Perspective::default(),
      round: 1,
      response: String::new(),
      extracted: HashMap::new(),
      confidence: 0.0,
      notes: String::new(),
      timestamp: String::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
  pub id: String,
  pub field: String,
  pub description: String,
  pub blocking: bool,
  pub suggested_default: String,
  pub why_needed: String,
  pub round: u32,
  pub resolved: bool,
  pub resolution: String,
}

impl Default for Gap {
  fn default() -> Self {
    Self {
      id: String::new(),
      field: String::new(),
      description: String::new(),
      blocking: true,
      suggested_default: String::new(),
      why_needed: String::new(),
      round: 1,
      resolved: false,
      resolution: String::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct Conflict {
  pub id: String,
  pub between: (String, String),
  pub description: String,
  pub impact: String,
  pub options: Vec<ConflictResolution>,
  pub chosen: Option<i32>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct ConflictResolution {
  pub option: String,
  pub description: String,
  pub tradeoffs: String,
  pub recommendation: bool,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
  pub id: String,
  pub round: u32,
  pub perspective: Perspective,
  pub category: QuestionCategory,
  pub priority: QuestionPriority,
  pub question: String,
  pub context: String,
  pub example: String,
  pub expected_type: String,
  pub extract_into: Vec<String>,
  pub depends_on: Vec<String>,
  pub blocks: Vec<String>,
}

impl Default for Question {
  fn default() -> Self {
    Self {
      id: String::new(),
      round: 1,
      perspective: Perspective::default(),
      category: QuestionCategory::default(),
      priority: QuestionPriority::default(),
      question: String::new(),
      context: String::new(),
      example: String::new(),
      expected_type: String::new(),
      extract_into: Vec::new(),
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewSession {
  pub id: String,
  pub profile: Profile,
  pub created_at: String,
  pub updated_at: String,
  pub completed_at: Option<String>,
  pub stage: InterviewStage,
  pub rounds_completed: u32,
  pub answers: Vec<Answer>,
  pub gaps: Vec<Gap>,
  pub conflicts: Vec<Conflict>,
  pub raw_notes: String,
  pub current_phase: u32,
  pub completed_phases: Vec<u32>,
}

impl Default for InterviewSession {
  fn default() -> Self {
    Self {
      id: String::new(),
      profile: Profile::default(),
      created_at: String::new(),
      updated_at: String::new(),
      completed_at: None,
      stage: InterviewStage::default(),
      rounds_completed: 0,
      answers: Vec::new(),
      gaps: Vec::new(),
      conflicts: Vec::new(),
      raw_notes: String::new(),
      current_phase: 1,
      completed_phases: Vec::new(),
    }
  }
}
