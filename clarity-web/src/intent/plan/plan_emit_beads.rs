//! WP24: Plan Emit Beads - Generate work items with idempotency
//!
//! This module provides functionality to emit (generate) plan beads from
//! interview session data with idempotency guarantees to avoid duplicates.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::interview::types::{Answer, InterviewSession, InterviewStage, Profile};
use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError, PlanPhase};
use std::collections::HashSet;

/// Result of bead emission
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmissionResult {
    /// Number of beads successfully emitted
    pub emitted: usize,
    /// Number of beads skipped (already existed)
    pub skipped: usize,
    /// Error messages for failed emissions
    pub errors: Vec<String>,
}

impl Default for EmissionResult {
    fn default() -> Self {
        Self::new()
    }
}

impl EmissionResult {
    /// Create a new empty emission result
    #[must_use]
    pub const fn new() -> Self {
        Self {
            emitted: 0,
            skipped: 0,
            errors: Vec::new(),
        }
    }

    /// Add an emitted bead count
    pub fn add_emitted(&mut self, count: usize) {
        self.emitted += count;
    }

    /// Add a skipped bead count
    pub fn add_skipped(&mut self, count: usize) {
        self.skipped += count;
    }

    /// Add an error message
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if emission was successful (no errors)
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total beads processed
    #[must_use]
    pub const fn total_processed(&self) -> usize {
        self.emitted + self.skipped
    }
}

/// Emit beads from an interview session
///
/// Generates plan beads based on the session's answers, gaps, and conflicts.
/// Implements idempotency by checking existing beads in the plan.
///
/// # Arguments
/// * `session` - The interview session to generate beads from
/// * `plan` - The existing execution plan (for idempotency check)
/// * `dry_run` - If true, only compute what would be emitted without modifying
///
/// # Returns
/// A tuple of (emitted beads, emission result) on success
///
/// # Errors
/// Returns `PlanError` if session validation fails
pub fn emit_beads(
    session: &InterviewSession,
    plan: &mut ExecutionPlan,
    dry_run: bool,
) -> Result<(Vec<PlanBead>, EmissionResult), PlanError> {
    // Validate session
    if session.id.trim().is_empty() {
        return Err(PlanError::EmptySessionId);
    }

    // Session must be past discovery to emit beads
    if session.stage == InterviewStage::Discovery {
        return Err(PlanError::InvalidPhaseNumber {
            phase_number: 0,
        });
    }

    let mut result = EmissionResult::new();
    let mut emitted_beads = Vec::new();

    // Get existing bead titles for idempotency check
    let existing_titles: Vec<String> = plan.beads.iter().map(|b| b.title.clone()).collect();

    // Generate beads from answers
    let answer_beads = generate_beads_from_answers(session, &existing_titles, &mut result);

    // Generate beads from gaps (if any unresolved)
    let gap_beads = generate_beads_from_gaps(session, &existing_titles, &mut result);

    // Generate beads from conflicts (if any unresolved)
    let conflict_beads = generate_beads_from_conflicts(session, &existing_titles, &mut result);

    // Combine all beads
    emitted_beads.extend(answer_beads);
    emitted_beads.extend(gap_beads);
    emitted_beads.extend(conflict_beads);

    // If not dry run, add beads to plan
    if !dry_run {
        for bead in &emitted_beads {
            match plan.add_bead(bead.clone()) {
                Ok(()) => {}
                Err(PlanError::DuplicateBeadId(_)) => {
                    // This shouldn't happen due to idempotency check, but handle gracefully
                    result.add_skipped(1);
                }
                Err(e) => {
                    result.add_error(format!("Failed to add bead '{}': {}", bead.id, e));
                }
            }
        }

        // Update phases in the plan
        update_plan_phases(plan);
    }

    result.emitted = emitted_beads.len();

    Ok((emitted_beads, result))
}

/// Generate beads from session answers
fn generate_beads_from_answers(
    session: &InterviewSession,
    existing_titles: &[String],
    result: &mut EmissionResult,
) -> Vec<PlanBead> {
    let mut beads = Vec::new();

    // Group answers by phase
    let phases = group_answers_by_phase(session);

    for (phase, answers) in phases {
        let titles_to_create: Vec<String> = answers
            .iter()
            .map(|a| format!("Implement: {}", a.question_text))
            .collect();

        // Check idempotency
        let new_titles = check_existing_beads(&titles_to_create, existing_titles);
        result.add_skipped(titles_to_create.len() - new_titles.len());

        for answer in answers {
            let title = format!("Implement: {}", answer.question_text);

            // Only create if not already existing
            if new_titles.contains(&title) {
                if let Ok(bead) = create_bead_from_answer(answer, phase) {
                    beads.push(bead);
                }
            }
        }
    }

    beads
}

/// Generate beads from unresolved gaps
fn generate_beads_from_gaps(
    session: &InterviewSession,
    existing_titles: &[String],
    result: &mut EmissionResult,
) -> Vec<PlanBead> {
    let mut beads = Vec::new();

    for gap in &session.gaps {
        if gap.resolved {
            continue;
        }

        let title = format!("Address gap: {}", gap.field);

        // Check idempotency
        if check_existing_beads(&[title.clone()], existing_titles).is_empty() {
            result.add_skipped(1);
            continue;
        }

        let bead_result = PlanBead::new(
            format!("gap-{}", gap.id),
            title,
            gap.round,
        );

        let bead = match bead_result {
            Ok(b) => b
                .with_description(format!("Resolve gap: {} - {}", gap.field, gap.description))
                .with_effort(if gap.blocking { 3 } else { 1 })
                .with_tag("gap".to_string())
                .with_tag(if gap.blocking { "blocking".to_string() } else { "optional".to_string() }),
            Err(e) => {
                result.add_error(format!("Failed to create gap bead: {}", e));
                continue;
            }
        };

        beads.push(bead);
    }

    beads
}

/// Generate beads from unresolved conflicts
fn generate_beads_from_conflicts(
    session: &InterviewSession,
    existing_titles: &[String],
    result: &mut EmissionResult,
) -> Vec<PlanBead> {
    let mut beads = Vec::new();

    for conflict in &session.conflicts {
        if conflict.chosen.is_some() {
            continue;
        }

        let title = format!("Resolve conflict: {} vs {}", conflict.between.0, conflict.between.1);

        // Check idempotency
        if check_existing_beads(&[title.clone()], existing_titles).is_empty() {
            result.add_skipped(1);
            continue;
        }

        let bead_result = PlanBead::new(
            format!("conflict-{}", conflict.id),
            title,
            1, // Conflicts should be resolved early
        );

        let bead = match bead_result {
            Ok(b) => b
                .with_description(format!(
                    "Resolve conflict between '{}' and '{}': {}",
                    conflict.between.0, conflict.between.1, conflict.description
                ))
                .with_effort(2)
                .with_tag("conflict".to_string())
                .with_priority(1), // High priority
            Err(e) => {
                result.add_error(format!("Failed to create conflict bead: {}", e));
                continue;
            }
        };

        beads.push(bead);
    }

    beads
}

/// Create a bead from an answer
fn create_bead_from_answer(answer: &Answer, phase: u32) -> Result<PlanBead, PlanError> {
    let id = format!("answer-{}-{}", answer.round, answer.question_id);
    let title = format!("Implement: {}", answer.question_text);

    let bead = PlanBead::new(id, title, phase)?;

    Ok(bead
        .with_description(format!(
            "Implementation task from answer: {}",
            answer.response
        ))
        .with_effort(estimate_effort_from_confidence(answer.confidence))
        .with_tag(format!("round-{}", answer.round)))
}

/// Estimate effort based on confidence level
fn estimate_effort_from_confidence(confidence: f64) -> u32 {
    // Lower confidence = more effort needed
    if confidence >= 0.9 {
        1
    } else if confidence >= 0.7 {
        2
    } else if confidence >= 0.5 {
        3
    } else {
        5
    }
}

/// Group answers by phase number
fn group_answers_by_phase(session: &InterviewSession) -> Vec<(u32, Vec<&Answer>)> {
    let mut phase_map: std::collections::HashMap<u32, Vec<&Answer>> = std::collections::HashMap::new();

    for answer in &session.answers {
        // Map round to phase (simplified: round 1-2 = phase 1, round 3 = phase 2, etc.)
        let phase = match answer.round {
            1 | 2 => 1,
            3 => 2,
            4 => 3,
            _ => answer.round.saturating_sub(1),
        };

        phase_map.entry(phase).or_default().push(answer);
    }

    let mut phases: Vec<(u32, Vec<&Answer>)> = phase_map.into_iter().collect();
    phases.sort_by_key(|(phase, _)| *phase);
    phases
}

/// Check for existing beads (idempotency check)
///
/// Returns only titles that don't already exist in the existing list.
///
/// # Arguments
/// * `titles` - Titles to check
/// * `existing` - Existing bead titles
///
/// # Returns
/// Vector of titles that don't exist yet
#[must_use]
pub fn check_existing_beads(titles: &[String], existing: &[String]) -> Vec<String> {
    let existing_set: HashSet<&str> = existing.iter().map(String::as_str).collect();

    titles
        .iter()
        .filter(|title| !existing_set.contains(title.as_str()))
        .cloned()
        .collect()
}

/// Update plan phases based on beads
fn update_plan_phases(plan: &mut ExecutionPlan) {
    // Clear existing phases
    plan.phases.clear();

    // Get unique phase numbers
    let mut phase_numbers: HashSet<u32> = plan
        .beads
        .iter()
        .map(|b| b.phase)
        .collect();

    // Create phases
    let mut sorted_phases: Vec<u32> = phase_numbers.drain().collect();
    sorted_phases.sort();

    for phase_num in sorted_phases {
        let mut phase = PlanPhase::new(phase_num, format!("Phase {}", phase_num));

        // Add beads to phase
        for bead in &plan.beads {
            if bead.phase == phase_num {
                phase.add_bead(bead.id.clone());
            }
        }

        plan.phases.push(phase);
    }
}

/// Generate beads specific to a profile type
///
/// # Arguments
/// * `profile` - The profile type
/// * `phase` - The phase to generate beads for
///
/// # Returns
/// Vector of profile-specific beads
#[must_use]
pub fn generate_profile_beads(profile: Profile, phase: u32) -> Vec<PlanBead> {
    match profile {
        Profile::Api => generate_api_beads(phase),
        Profile::Cli => generate_cli_beads(phase),
        Profile::Event => generate_event_beads(phase),
        Profile::Data => generate_data_beads(phase),
        Profile::Workflow => generate_workflow_beads(phase),
        Profile::Ui => generate_ui_beads(phase),
    }
}

fn generate_api_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("api-design", "Design API endpoints", 2),
        ("api-auth", "Implement authentication", 3),
        ("api-validation", "Add input validation", 2),
        ("api-error-handling", "Implement error handling", 2),
        ("api-docs", "Generate API documentation", 1),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("api-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("API task: {}", title))
                    .with_effort(*effort)
                    .with_tag("api".to_string())
            })
        })
        .collect()
}

fn generate_cli_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("cli-args", "Parse command-line arguments", 2),
        ("cli-help", "Implement help system", 1),
        ("cli-output", "Format output", 2),
        ("cli-errors", "Handle errors gracefully", 2),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("cli-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("CLI task: {}", title))
                    .with_effort(*effort)
                    .with_tag("cli".to_string())
            })
        })
        .collect()
}

fn generate_event_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("event-schema", "Define event schemas", 2),
        ("event-producer", "Implement event producer", 3),
        ("event-consumer", "Implement event consumer", 3),
        ("event-error", "Handle event errors", 2),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("event-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("Event task: {}", title))
                    .with_effort(*effort)
                    .with_tag("event".to_string())
            })
        })
        .collect()
}

fn generate_data_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("data-model", "Design data model", 3),
        ("data-migration", "Create migrations", 2),
        ("data-access", "Implement data access layer", 3),
        ("data-validation", "Add data validation", 2),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("data-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("Data task: {}", title))
                    .with_effort(*effort)
                    .with_tag("data".to_string())
            })
        })
        .collect()
}

fn generate_workflow_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("workflow-design", "Design workflow", 3),
        ("workflow-steps", "Implement workflow steps", 3),
        ("workflow-error", "Handle workflow errors", 2),
        ("workflow-monitor", "Add workflow monitoring", 2),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("workflow-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("Workflow task: {}", title))
                    .with_effort(*effort)
                    .with_tag("workflow".to_string())
            })
        })
        .collect()
}

fn generate_ui_beads(phase: u32) -> Vec<PlanBead> {
    let tasks = [
        ("ui-components", "Create UI components", 3),
        ("ui-state", "Implement state management", 3),
        ("ui-events", "Handle user events", 2),
        ("ui-styling", "Apply styling", 1),
    ];

    tasks
        .iter()
        .filter_map(|(id, title, effort)| {
            PlanBead::new(format!("ui-{}", id), title.to_string(), phase).ok().map(|b| {
                b.with_description(format!("UI task: {}", title))
                    .with_effort(*effort)
                    .with_tag("ui".to_string())
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::interview::types::{Conflict, ConflictResolution, Gap, Perspective};
    use std::collections::HashMap;

    fn create_test_session() -> InterviewSession {
        let mut session = InterviewSession::new(
            "test-session".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        // Advance to refinement stage
        session.stage = InterviewStage::Refinement;
        session
    }

    fn create_test_plan() -> ExecutionPlan {
        ExecutionPlan::new("test-session".to_string())
    }

    #[test]
    fn test_emission_result_new() {
        let result = EmissionResult::new();
        assert_eq!(result.emitted, 0);
        assert_eq!(result.skipped, 0);
        assert!(result.errors.is_empty());
        assert!(result.is_success());
    }

    #[test]
    fn test_emission_result_counts() {
        let mut result = EmissionResult::new();
        result.add_emitted(5);
        result.add_skipped(3);
        result.add_error("Test error".to_string());

        assert_eq!(result.emitted, 5);
        assert_eq!(result.skipped, 3);
        assert_eq!(result.total_processed(), 8);
        assert!(!result.is_success());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_check_existing_beads_empty() {
        let titles = vec!["Bead 1".to_string(), "Bead 2".to_string()];
        let existing: Vec<String> = Vec::new();

        let new_titles = check_existing_beads(&titles, &existing);
        assert_eq!(new_titles.len(), 2);
    }

    #[test]
    fn test_check_existing_beads_all_exist() {
        let titles = vec!["Bead 1".to_string(), "Bead 2".to_string()];
        let existing = vec!["Bead 1".to_string(), "Bead 2".to_string()];

        let new_titles = check_existing_beads(&titles, &existing);
        assert!(new_titles.is_empty());
    }

    #[test]
    fn test_check_existing_beads_partial() {
        let titles = vec!["Bead 1".to_string(), "Bead 2".to_string(), "Bead 3".to_string()];
        let existing = vec!["Bead 1".to_string(), "Bead 3".to_string()];

        let new_titles = check_existing_beads(&titles, &existing);
        assert_eq!(new_titles.len(), 1);
        assert_eq!(new_titles[0], "Bead 2");
    }

    #[test]
    fn test_emit_beads_empty_session_id() {
        let session = InterviewSession::default();
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(matches!(result, Err(PlanError::EmptySessionId)));
    }

    #[test]
    fn test_emit_beads_discovery_stage() {
        let session = InterviewSession::new(
            "test".to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        );
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(matches!(result, Err(PlanError::InvalidPhaseNumber { .. })));
    }

    #[test]
    fn test_emit_beads_empty_session() {
        let session = create_test_session();
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, emission_result) = result.expect("ok");
        assert!(beads.is_empty());
        assert!(emission_result.is_success());
    }

    #[test]
    fn test_emit_beads_with_answers() {
        let mut session = create_test_session();
        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "What is the API?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "REST API".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:00:00Z".to_string(),
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");
        assert!(!beads.is_empty());
    }

    #[test]
    fn test_emit_beads_with_gaps() {
        let mut session = create_test_session();
        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "base_url".to_string(),
            description: "Missing base URL".to_string(),
            blocking: true,
            resolved: false,
            ..Gap::default()
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");

        let gap_beads: Vec<&PlanBead> = beads.iter().filter(|b| b.tags.contains(&"gap".to_string())).collect();
        assert_eq!(gap_beads.len(), 1);
    }

    #[test]
    fn test_emit_beads_with_resolved_gaps() {
        let mut session = create_test_session();
        session.gaps.push(Gap {
            id: "gap-1".to_string(),
            field: "base_url".to_string(),
            description: "Missing base URL".to_string(),
            blocking: true,
            resolved: true, // Already resolved
            ..Gap::default()
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");

        let gap_beads: Vec<&PlanBead> = beads.iter().filter(|b| b.tags.contains(&"gap".to_string())).collect();
        assert!(gap_beads.is_empty());
    }

    #[test]
    fn test_emit_beads_with_conflicts() {
        let mut session = create_test_session();
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "CAP conflict".to_string(),
            impact: "High".to_string(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");

        let conflict_beads: Vec<&PlanBead> = beads
            .iter()
            .filter(|b| b.tags.contains(&"conflict".to_string()))
            .collect();
        assert_eq!(conflict_beads.len(), 1);
    }

    #[test]
    fn test_emit_beads_with_resolved_conflicts() {
        let mut session = create_test_session();
        session.conflicts.push(Conflict {
            id: "conflict-1".to_string(),
            between: ("a".to_string(), "b".to_string()),
            description: "CAP conflict".to_string(),
            impact: "High".to_string(),
            options: vec![ConflictResolution::default()],
            chosen: Some(0), // Already resolved
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, false);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");

        let conflict_beads: Vec<&PlanBead> = beads
            .iter()
            .filter(|b| b.tags.contains(&"conflict".to_string()))
            .collect();
        assert!(conflict_beads.is_empty());
    }

    #[test]
    fn test_emit_beads_dry_run() {
        let mut session = create_test_session();
        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "Question?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:00:00Z".to_string(),
        });
        let mut plan = create_test_plan();

        let result = emit_beads(&session, &mut plan, true);
        assert!(result.is_ok());
        let (beads, _) = result.expect("ok");

        // Dry run should compute beads but not add to plan
        assert!(!beads.is_empty());
        assert!(plan.beads.is_empty());
    }

    #[test]
    fn test_emit_beads_idempotency() {
        let mut session = create_test_session();
        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "Question?".to_string(),
            perspective: Perspective::User,
            round: 1,
            response: "Answer".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            notes: String::new(),
            timestamp: "2026-02-27T00:00:00Z".to_string(),
        });

        let mut plan = create_test_plan();

        // First emission
        let result1 = emit_beads(&session, &mut plan, false);
        assert!(result1.is_ok());
        let (beads1, _) = result1.expect("ok");
        let _initial_count = beads1.len();

        // Second emission (should be idempotent)
        let result2 = emit_beads(&session, &mut plan, false);
        assert!(result2.is_ok());
        let (beads2, emission_result) = result2.expect("ok");

        // Should skip existing beads
        assert!(beads2.is_empty() || emission_result.skipped > 0);
    }

    #[test]
    fn test_estimate_effort_from_confidence() {
        assert_eq!(estimate_effort_from_confidence(0.95), 1);
        assert_eq!(estimate_effort_from_confidence(0.9), 1);
        assert_eq!(estimate_effort_from_confidence(0.85), 2);
        assert_eq!(estimate_effort_from_confidence(0.7), 2);
        assert_eq!(estimate_effort_from_confidence(0.6), 3);
        assert_eq!(estimate_effort_from_confidence(0.5), 3);
        assert_eq!(estimate_effort_from_confidence(0.4), 5);
        assert_eq!(estimate_effort_from_confidence(0.0), 5);
    }

    #[test]
    fn test_generate_profile_beads_api() {
        let beads = generate_profile_beads(Profile::Api, 1);
        assert!(!beads.is_empty());

        for bead in &beads {
            assert!(bead.tags.contains(&"api".to_string()));
            assert_eq!(bead.phase, 1);
        }
    }

    #[test]
    fn test_generate_profile_beads_cli() {
        let beads = generate_profile_beads(Profile::Cli, 2);
        assert!(!beads.is_empty());

        for bead in &beads {
            assert!(bead.tags.contains(&"cli".to_string()));
            assert_eq!(bead.phase, 2);
        }
    }

    #[test]
    fn test_generate_profile_beads_all_profiles() {
        let profiles = [
            Profile::Api,
            Profile::Cli,
            Profile::Event,
            Profile::Data,
            Profile::Workflow,
            Profile::Ui,
        ];

        for profile in profiles {
            let beads = generate_profile_beads(profile, 1);
            assert!(!beads.is_empty(), "Profile {:?} should generate beads", profile);
        }
    }

    #[test]
    fn test_update_plan_phases() {
        let mut plan = create_test_plan();

        plan.add_bead(
            PlanBead::new("b1".to_string(), "First".to_string(), 1).expect("valid"),
        )
        .expect("add");
        plan.add_bead(
            PlanBead::new("b2".to_string(), "Second".to_string(), 1).expect("valid"),
        )
        .expect("add");
        plan.add_bead(
            PlanBead::new("b3".to_string(), "Third".to_string(), 2).expect("valid"),
        )
        .expect("add");

        update_plan_phases(&mut plan);

        assert_eq!(plan.phases.len(), 2);
        assert_eq!(plan.phases[0].number, 1);
        assert_eq!(plan.phases[0].beads.len(), 2);
        assert_eq!(plan.phases[1].number, 2);
        assert_eq!(plan.phases[1].beads.len(), 1);
    }

    #[test]
    fn test_group_answers_by_phase() {
        let mut session = create_test_session();

        // Add answers in different rounds
        for round in 1..=4 {
            session.answers.push(Answer {
                question_id: format!("q{}", round),
                question_text: format!("Question {}", round),
                perspective: Perspective::User,
                round,
                response: format!("Answer {}", round),
                extracted: HashMap::new(),
                confidence: 0.8,
                notes: String::new(),
                timestamp: "2026-02-27T00:00:00Z".to_string(),
            });
        }

        let phases = group_answers_by_phase(&session);

        // Should have answers grouped by phase
        assert!(!phases.is_empty());

        // Verify phases are sorted
        for i in 1..phases.len() {
            assert!(phases[i - 1].0 <= phases[i].0);
        }
    }
}
