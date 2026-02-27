#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod plan_emit_beads_core;
mod plan_emit_beads_profile;

use crate::intent::interview::types::{InterviewSession, Profile};
use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmissionResult {
    pub emitted: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl Default for EmissionResult {
    fn default() -> Self {
        Self::new()
    }
}

impl EmissionResult {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            emitted: 0,
            skipped: 0,
            errors: Vec::new(),
        }
    }

    pub fn add_skipped(&mut self, count: usize) {
        self.skipped += count;
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    #[must_use]
    pub const fn total_processed(&self) -> usize {
        self.emitted + self.skipped
    }
}

pub fn emit_beads(
    session: &InterviewSession,
    plan: &mut ExecutionPlan,
    dry_run: bool,
) -> Result<(Vec<PlanBead>, EmissionResult), PlanError> {
    plan_emit_beads_core::emit_beads(session, plan, dry_run)
}

#[must_use]
pub fn check_existing_beads(titles: &[String], existing: &[String]) -> Vec<String> {
    let existing_set: HashSet<&str> = existing.iter().map(String::as_str).collect();
    titles
        .iter()
        .filter(|title| !existing_set.contains(title.as_str()))
        .cloned()
        .collect()
}

#[must_use]
pub fn generate_profile_beads(profile: Profile, phase: u32) -> Vec<PlanBead> {
    plan_emit_beads_profile::generate_profile_beads(profile, phase)
}
