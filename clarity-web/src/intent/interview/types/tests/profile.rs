#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
use crate::intent::interview::types::{
  InterviewSession, InterviewSessionError, InterviewStage, Profile, ProfileParseError,
};
use std::str::FromStr;

#[test]
fn profile_roundtrip() {
  let profiles = [
    Profile::Api,
    Profile::Cli,
    Profile::Event,
    Profile::Data,
    Profile::Workflow,
    Profile::Ui,
  ];

  for profile in profiles {
    let parsed = profile.as_str().parse::<Profile>();
    assert_eq!(parsed, Ok(profile));
  }
}

#[test]
fn profile_parse_error_is_typed() {
  let parsed = Profile::from_str("unknown");
  assert!(matches!(
      parsed,
      Err(ProfileParseError::UnknownProfile { input }) if input == "unknown"
  ));
}

#[test]
fn required_fields_exist_for_all_profiles() {
  assert_eq!(Profile::Api.required_fields().len(), 5);
  assert_eq!(Profile::Cli.required_fields().len(), 4);
  assert_eq!(Profile::Event.required_fields().len(), 3);
  assert_eq!(Profile::Data.required_fields().len(), 3);
  assert_eq!(Profile::Workflow.required_fields().len(), 3);
  assert_eq!(Profile::Ui.required_fields().len(), 3);
}

#[test]
fn interview_session_new_sets_defaults() {
  let session = InterviewSession::new(
    "test-session".to_string(),
    Profile::Api,
    "2026-02-27T00:00:00Z".to_string(),
  );

  assert_eq!(session.id, "test-session");
  assert_eq!(session.profile, Profile::Api);
  assert_eq!(session.stage.as_str(), "discovery");
  assert_eq!(session.get_current_round(), 1);
  assert!(session.answers.is_empty());
  assert!(session.gaps.is_empty());
  assert!(session.conflicts.is_empty());
}

#[test]
fn interview_session_serde_roundtrip() {
  let session = InterviewSession::new(
    "serde-test".to_string(),
    Profile::Cli,
    "2026-02-27T00:00:00Z".to_string(),
  );

  let serialized = serde_json::to_string(&session);
  assert!(serialized.is_ok());

  let parsed = serialized
    .ok()
    .and_then(|json| serde_json::from_str::<InterviewSession>(&json).ok());
  assert_eq!(parsed, Some(session));
}

#[test]
fn interview_session_error_serde_roundtrip() {
  let error = InterviewSessionError::SessionNotModifiable {
    stage: InterviewStage::Complete,
  };

  let serialized = serde_json::to_string(&error);
  assert!(serialized.is_ok());

  let parsed = serialized
    .ok()
    .and_then(|json| serde_json::from_str::<InterviewSessionError>(&json).ok());
  assert_eq!(parsed, Some(error));
}
