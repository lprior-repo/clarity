use crate::intent::interview::types::InterviewStage;
use crate::intent::interview::types::enums::InterviewStageError;

#[test]
fn interview_stage_default_is_discovery() {
  let stage = InterviewStage::default();
  assert_eq!(stage, InterviewStage::Discovery);
  assert!(!stage.is_terminal());
  assert!(stage.is_active());
  assert!(!stage.is_paused());
}

// ============================================
// P0: State Machine Transition Tests
// ============================================

#[test]
fn interview_stage_can_transition_to_valid_paths() {
  // Discovery can go to Refinement, Validation, Paused
  assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Refinement));
  assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Validation));
  assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Paused));

  // Refinement can go to Validation, Complete, Paused
  assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Validation));
  assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Complete));
  assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Paused));

  // Validation can go to Complete, Paused
  assert!(InterviewStage::Validation.can_transition_to(InterviewStage::Complete));
  assert!(InterviewStage::Validation.can_transition_to(InterviewStage::Paused));

  // Paused can resume to any active stage
  assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Discovery));
  assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Refinement));
  assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Validation));

  // Same state is always valid (no-op)
  assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Discovery));
  assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Refinement));
  assert!(InterviewStage::Validation.can_transition_to(InterviewStage::Validation));
  assert!(InterviewStage::Complete.can_transition_to(InterviewStage::Complete));
  assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Paused));
}

#[test]
fn interview_stage_cannot_transition_invalid_paths() {
  // Cannot go backwards from Refinement
  assert!(!InterviewStage::Refinement.can_transition_to(InterviewStage::Discovery));

  // Cannot go backwards from Validation
  assert!(!InterviewStage::Validation.can_transition_to(InterviewStage::Discovery));
  assert!(!InterviewStage::Validation.can_transition_to(InterviewStage::Refinement));

  // Complete is terminal - no transitions out
  assert!(!InterviewStage::Complete.can_transition_to(InterviewStage::Discovery));
  assert!(!InterviewStage::Complete.can_transition_to(InterviewStage::Refinement));
  assert!(!InterviewStage::Complete.can_transition_to(InterviewStage::Validation));
  assert!(!InterviewStage::Complete.can_transition_to(InterviewStage::Paused));
}

#[test]
fn interview_stage_transition_to_success_for_valid() {
  let discovery = InterviewStage::Discovery;

  let result = discovery.transition_to(InterviewStage::Refinement);
  assert!(result.is_ok());
  assert_eq!(result, Ok(InterviewStage::Refinement));

  let result2 = discovery.transition_to(InterviewStage::Paused);
  assert!(result2.is_ok());
  assert_eq!(result2, Ok(InterviewStage::Paused));
}

#[test]
fn interview_stage_transition_to_error_for_invalid() {
  let complete = InterviewStage::Complete;

  let result = complete.transition_to(InterviewStage::Discovery);
  assert!(result.is_err());
  assert_eq!(
    result,
    Err(InterviewStageError::InvalidTransition {
      from: "complete".to_string(),
      to: "discovery".to_string(),
    })
  );

  let refinement = InterviewStage::Refinement;
  let result2 = refinement.transition_to(InterviewStage::Discovery);
  assert!(result2.is_err());
  assert_eq!(
    result2,
    Err(InterviewStageError::InvalidTransition {
      from: "refinement".to_string(),
      to: "discovery".to_string(),
    })
  );
}

#[test]
fn interview_stage_is_terminal() {
  assert!(!InterviewStage::Discovery.is_terminal());
  assert!(!InterviewStage::Refinement.is_terminal());
  assert!(!InterviewStage::Validation.is_terminal());
  assert!(InterviewStage::Complete.is_terminal());
  assert!(!InterviewStage::Paused.is_terminal());
}

#[test]
fn interview_stage_is_active() {
  assert!(InterviewStage::Discovery.is_active());
  assert!(InterviewStage::Refinement.is_active());
  assert!(InterviewStage::Validation.is_active());
  assert!(!InterviewStage::Complete.is_active());
  assert!(!InterviewStage::Paused.is_active());
}

#[test]
fn interview_stage_is_paused() {
  assert!(!InterviewStage::Discovery.is_paused());
  assert!(!InterviewStage::Refinement.is_paused());
  assert!(!InterviewStage::Validation.is_paused());
  assert!(!InterviewStage::Complete.is_paused());
  assert!(InterviewStage::Paused.is_paused());
}

#[test]
fn interview_stage_as_str() {
  assert_eq!(InterviewStage::Discovery.as_str(), "discovery");
  assert_eq!(InterviewStage::Refinement.as_str(), "refinement");
  assert_eq!(InterviewStage::Validation.as_str(), "validation");
  assert_eq!(InterviewStage::Complete.as_str(), "complete");
  assert_eq!(InterviewStage::Paused.as_str(), "paused");
}

// ============================================
// Full Workflow Tests
// ============================================

#[test]
fn interview_stage_normal_workflow() {
  // Simulate a normal workflow: Discovery -> Refinement -> Validation -> Complete
  let stage = InterviewStage::Discovery;

  let stage = stage.transition_to(InterviewStage::Refinement).expect("discovery -> refinement");
  assert_eq!(stage, InterviewStage::Refinement);

  let stage = stage.transition_to(InterviewStage::Validation).expect("refinement -> validation");
  assert_eq!(stage, InterviewStage::Validation);

  let stage = stage.transition_to(InterviewStage::Complete).expect("validation -> complete");
  assert_eq!(stage, InterviewStage::Complete);

  // Cannot leave Complete
  assert!(stage.transition_to(InterviewStage::Discovery).is_err());
}

#[test]
fn interview_stage_pause_resume_workflow() {
  // Start at Discovery
  let stage = InterviewStage::Discovery;

  // Pause
  let stage = stage.transition_to(InterviewStage::Paused).expect("discovery -> paused");
  assert!(stage.is_paused());

  // Resume to Refinement
  let stage = stage.transition_to(InterviewStage::Refinement).expect("paused -> refinement");
  assert_eq!(stage, InterviewStage::Refinement);

  // Pause again
  let stage = stage.transition_to(InterviewStage::Paused).expect("refinement -> paused");
  assert!(stage.is_paused());

  // Resume to Validation
  let stage = stage.transition_to(InterviewStage::Validation).expect("paused -> validation");
  assert_eq!(stage, InterviewStage::Validation);

  // Complete
  let stage = stage.transition_to(InterviewStage::Complete).expect("validation -> complete");
  assert!(stage.is_terminal());
}
