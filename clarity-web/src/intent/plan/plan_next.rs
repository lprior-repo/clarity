#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod plan_next_logic;

use crate::intent::interview::types::{Gap, InterviewSession};
use crate::intent::plan::types::{ExecutionPlan, PlanBead};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AnswerQuestion,
    ResolveGap,
    ResolveConflict,
    CompletePhase,
    ReviewPlan,
}

impl Default for ActionType {
    fn default() -> Self {
        Self::AnswerQuestion
    }
}

impl ActionType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AnswerQuestion => "answer_question",
            Self::ResolveGap => "resolve_gap",
            Self::ResolveConflict => "resolve_conflict",
            Self::CompletePhase => "complete_phase",
            Self::ReviewPlan => "review_plan",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AnswerQuestion => "Answer pending interview question",
            Self::ResolveGap => "Resolve missing requirement",
            Self::ResolveConflict => "Resolve conflicting requirements",
            Self::CompletePhase => "Complete the current phase",
            Self::ReviewPlan => "Review and approve execution plan",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction {
    pub action_type: ActionType,
    pub target_id: String,
    pub description: String,
    pub reason: String,
    #[serde(default)]
    pub priority: u32,
}

impl Default for NextAction {
    fn default() -> Self {
        Self {
            action_type: ActionType::default(),
            target_id: String::new(),
            description: String::new(),
            reason: String::new(),
            priority: 0,
        }
    }
}

impl NextAction {
    #[must_use]
    pub fn new(action_type: ActionType, target_id: String, description: String, reason: String) -> Self {
        Self {
            action_type,
            target_id,
            description,
            reason,
            priority: 0,
        }
    }

    #[must_use]
    pub fn with_priority(self, priority: u32) -> Self {
        Self { priority, ..self }
    }
}

#[must_use]
pub fn get_next_action(session: &InterviewSession, plan: &ExecutionPlan) -> Option<NextAction> {
    plan_next_logic::get_next_action(session, plan)
}

#[must_use]
pub fn determine_next_phase(plan: &ExecutionPlan) -> Option<u32> {
    plan_next_logic::determine_next_phase(plan)
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
    plan_next_logic::get_actionable_beads(plan)
}

#[must_use]
pub fn get_blocking_gaps(session: &InterviewSession) -> Vec<&Gap> {
    session.get_blocking_gaps()
}

#[must_use]
pub fn can_proceed(session: &InterviewSession) -> bool {
    session.can_proceed().is_ok()
}
