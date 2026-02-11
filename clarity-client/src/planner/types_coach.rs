//! Stub module for coach types
//!
//! TODO: Implement proper coach types

#![allow(dead_code)]

/// Coach answer - stub type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachAnswer {
    pub step_id: String,
    pub value: String,
}

/// Coach step - stub type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoachStep {
    pub id: String,
    pub step_id: String,
    pub title: String,
    pub question: String,
    pub follow_up: Option<String>,
}
