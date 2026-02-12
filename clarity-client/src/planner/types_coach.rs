//! Coach types for the planning coach component

#![allow(dead_code)]

/// Coach answer - user response to a step
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachAnswer {
  pub step_id: String,
  pub value: String,
}

/// Coach step - a single question/prompt in the planning flow
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachStep {
  pub id: String,
  pub step_id: String,
  pub title: String,
  pub question: String,
  pub hint: Option<String>,
  pub follow_up: Option<String>,
}
