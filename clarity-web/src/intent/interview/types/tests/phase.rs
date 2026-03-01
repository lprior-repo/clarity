//! Comprehensive unit tests for phase management in InterviewSession.
//!
//! Tests cover:
//! - Phase transitions
//! - Phase completion tracking
//! - Edge cases: invalid phases, out-of-order transitions
//! - Error handling

use crate::intent::interview::types::{
  InterviewSession, InterviewSessionError, InterviewStage, Profile,
};

fn make_session() -> InterviewSession {
  InterviewSession::new(
    "test-session".to_string(),
    Profile::Api,
    "2026-02-27T00:00:00Z".to_string(),
  )
}

// ============================================
// Basic Phase State Tests
// ============================================

#[test]
fn session_default_current_phase_is_one() {
  let session = InterviewSession::default();
  assert_eq!(session.current_phase, 1);
  assert!(session.completed_phases.is_empty());
}

#[test]
fn new_session_current_phase_is_one() {
  let session = make_session();
  assert_eq!(session.current_phase, 1);
  assert!(session.completed_phases.is_empty());
}

// ============================================
// Phase Completion Tests
// ============================================

#[test]
fn complete_phase_succeeds_for_valid_phase() {
  let mut session = make_session();

  let result = session.complete_phase(1, "2026-02-27T00:01:00Z");
  assert!(result.is_ok());
  assert_eq!(session.current_phase, 2);
  assert_eq!(session.completed_phases, vec![1]);
}

#[test]
fn complete_phase_tracks_multiple_phases() {
  let mut session = make_session();

  session.complete_phase(1, "t1").ok();
  session.complete_phase(2, "t2").ok();
  let result = session.complete_phase(3, "t3");

  assert!(result.is_ok());
  assert_eq!(session.current_phase, 4);
  assert!(session.completed_phases.contains(&1));
  assert!(session.completed_phases.contains(&2));
  assert!(session.completed_phases.contains(&3));
}

#[test]
fn complete_phase_updates_timestamp() {
  let mut session = make_session();
  assert_eq!(session.updated_at, "2026-02-27T00:00:00Z");

  session.complete_phase(1, "2026-02-27T00:05:00Z").ok();
  assert_eq!(session.updated_at, "2026-02-27T00:05:00Z");
}

// ============================================
// Out-of-Order Phase Transitions
// ============================================

#[test]
fn complete_phase_allows_out_of_order_completion() {
  let mut session = make_session();

  // Complete phase 3 before phases 1 and 2
  let result = session.complete_phase(3, "t1");
  assert!(result.is_ok());

  // Phase 3 is marked complete
  assert!(session.completed_phases.contains(&3));

  // current_phase only advances if completing the current phase
  assert_eq!(session.current_phase, 1);
}

#[test]
fn complete_phase_advances_current_only_when_matching() {
  let mut session = make_session();

  // Complete phase 2 (not current)
  session.complete_phase(2, "t1").ok();
  assert_eq!(session.current_phase, 1);

  // Complete phase 1 (current)
  session.complete_phase(1, "t2").ok();
  assert_eq!(session.current_phase, 2);

  // Complete phase 2 again (now current)
  session.complete_phase(2, "t3").ok();
  assert_eq!(session.current_phase, 3);
}

#[test]
fn complete_phase_handles_skipped_phases() {
  let mut session = make_session();

  // Skip directly to phase 5
  session.complete_phase(5, "t1").ok();
  assert!(session.completed_phases.contains(&5));
  assert_eq!(session.current_phase, 1); // current_phase unchanged

  // Now complete phase 1
  session.complete_phase(1, "t2").ok();
  assert_eq!(session.current_phase, 2);

  // Complete phase 2
  session.complete_phase(2, "t3").ok();
  assert_eq!(session.current_phase, 3);
}

// ============================================
// Idempotency Tests
// ============================================

#[test]
fn complete_phase_is_idempotent() {
  let mut session = make_session();

  session.complete_phase(1, "t1").ok();
  let first_state = session.clone();

  session.complete_phase(1, "t2").ok();

  // Phase 1 appears only once in completed_phases
  let count = session.completed_phases.iter().filter(|&&p| p == 1).count();
  assert_eq!(count, 1);

  // current_phase doesn't change on re-completion
  assert_eq!(session.current_phase, first_state.current_phase);

  // But timestamp should update
  assert_eq!(session.updated_at, "t2");
}

// ============================================
// Edge Cases: Invalid Phases
// ============================================

#[test]
fn complete_phase_rejects_phase_zero() {
  let mut session = make_session();

  let result = session.complete_phase(0, "t1");
  assert!(matches!(
    result,
    Err(InterviewSessionError::InvalidPhaseNumber { phase_number: 0 })
  ));
}

#[test]
fn complete_phase_rejects_empty_timestamp() {
  let mut session = make_session();

  let result = session.complete_phase(1, "");
  assert!(matches!(result, Err(InterviewSessionError::EmptyTimestamp)));
}

#[test]
fn complete_phase_rejects_empty_timestamp_for_any_phase() {
  let mut session = make_session();

  let result = session.complete_phase(5, "");
  assert!(matches!(result, Err(InterviewSessionError::EmptyTimestamp)));
}

#[test]
fn complete_phase_allows_large_phase_numbers() {
  let mut session = make_session();

  // Large phase numbers should be allowed (no upper bound in current implementation)
  let result = session.complete_phase(1000, "t1");
  assert!(result.is_ok());
  assert!(session.completed_phases.contains(&1000));
}

#[test]
fn complete_phase_unchanged_on_error() {
  let mut session = make_session();
  let original_phase = session.current_phase;
  let original_completed = session.completed_phases.clone();

  let _err = session.complete_phase(0, "t1");

  assert_eq!(session.current_phase, original_phase);
  assert_eq!(session.completed_phases, original_completed);
}

// ============================================
// Phase and Stage Interaction Tests
// ============================================

#[test]
fn complete_phase_works_in_discovery_stage() {
  let mut session = make_session();
  session.stage = InterviewStage::Discovery;

  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
}

#[test]
fn complete_phase_works_in_refinement_stage() {
  let mut session = make_session();
  session.stage = InterviewStage::Refinement;

  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
}

#[test]
fn complete_phase_works_in_validation_stage() {
  let mut session = make_session();
  session.stage = InterviewStage::Validation;

  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
}

#[test]
fn complete_phase_works_in_complete_stage() {
  let mut session = make_session();
  session.stage = InterviewStage::Complete;

  // Phase completion should still work even when session is complete
  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
}

#[test]
fn complete_phase_works_in_paused_stage() {
  let mut session = make_session();
  session.stage = InterviewStage::Paused;

  // Phase completion should still work when paused
  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
}

// ============================================
// Multiple Phase Operations
// ============================================

#[test]
fn complete_phase_sequential_order() {
  let mut session = make_session();

  for phase in 1..=5 {
    let result = session.complete_phase(phase, &format!("t{phase}"));
    assert!(result.is_ok());
    assert_eq!(session.current_phase, phase + 1);
  }

  assert_eq!(session.completed_phases, vec![1, 2, 3, 4, 5]);
}

#[test]
fn complete_phase_reverse_order() {
  let mut session = make_session();

  // Complete phases in reverse order
  session.complete_phase(5, "t1").ok();
  session.complete_phase(4, "t2").ok();
  session.complete_phase(3, "t3").ok();
  session.complete_phase(2, "t4").ok();
  session.complete_phase(1, "t5").ok();

  // All should be marked complete
  assert!(session.completed_phases.contains(&1));
  assert!(session.completed_phases.contains(&2));
  assert!(session.completed_phases.contains(&3));
  assert!(session.completed_phases.contains(&4));
  assert!(session.completed_phases.contains(&5));

  // current_phase only advances when completing the CURRENT phase
  // We started at 1, so only completing phase 1 advances current_phase to 2
  // The other phases (5,4,3,2) were marked complete but didn't advance current_phase
  assert_eq!(session.current_phase, 2);
}

#[test]
fn complete_phase_random_order() {
  let mut session = make_session();

  // Complete in random order: 3, 1, 5, 2, 4
  session.complete_phase(3, "t1").ok();
  session.complete_phase(1, "t2").ok();
  session.complete_phase(5, "t3").ok();
  session.complete_phase(2, "t4").ok();
  session.complete_phase(4, "t5").ok();

  assert_eq!(session.completed_phases.len(), 5);
  assert!(session.completed_phases.contains(&1));
  assert!(session.completed_phases.contains(&2));
  assert!(session.completed_phases.contains(&3));
  assert!(session.completed_phases.contains(&4));
  assert!(session.completed_phases.contains(&5));
}

// ============================================
// State Consistency Tests
// ============================================

#[test]
fn complete_phase_maintains_consistency_after_multiple_calls() {
  let mut session = make_session();

  // Complete phase 1 multiple times
  for _ in 0..5 {
    session.complete_phase(1, "t1").ok();
  }

  // Should only appear once
  assert_eq!(session.completed_phases.len(), 1);

  // current_phase should have advanced once
  assert_eq!(session.current_phase, 2);
}

#[test]
fn complete_phase_current_phase_never_decreases() {
  let mut session = make_session();

  let initial = session.current_phase;

  session.complete_phase(5, "t1").ok();
  assert!(session.current_phase >= initial);

  session.complete_phase(1, "t2").ok();
  assert!(session.current_phase >= initial);

  session.complete_phase(10, "t3").ok();
  assert!(session.current_phase >= initial);
}

// ============================================
// Boundary Tests
// ============================================

#[test]
fn complete_phase_with_max_u32() {
  let mut session = make_session();

  // u32::MAX should be a valid phase number
  let result = session.complete_phase(u32::MAX, "t1");
  assert!(result.is_ok());
  assert!(session.completed_phases.contains(&u32::MAX));
}

#[test]
fn complete_phase_with_one() {
  let mut session = make_session();

  // Phase 1 is the minimum valid phase
  let result = session.complete_phase(1, "t1");
  assert!(result.is_ok());
  assert_eq!(session.current_phase, 2);
}
