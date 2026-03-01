#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::manual_let_else)]

use super::*;
use crate::intent::interview::types::{Answer, Gap, InterviewStage, Profile};
use std::collections::HashMap;
use tempfile::NamedTempFile;

fn create_test_session(id: &str) -> crate::intent::interview::types::InterviewSession {
  crate::intent::interview::types::InterviewSession::new(
    id.to_string(),
    Profile::Api,
    "2026-02-27T00:00:00Z".to_string(),
  )
}

fn create_test_session_with_answers(id: &str) -> crate::intent::interview::types::InterviewSession {
  let mut session = create_test_session(id);
  session.answers.push(Answer {
    question_id: "q1".to_string(),
    question_text: "What is the API?".to_string(),
    response: "REST API".to_string(),
    extracted: HashMap::new(),
    confidence: 0.9,
    ..Answer::default()
  });
  session.answers.push(Answer {
    question_id: "q2".to_string(),
    question_text: "What is auth?".to_string(),
    response: "Bearer token".to_string(),
    extracted: HashMap::new(),
    confidence: 0.8,
    ..Answer::default()
  });
  session
}

#[test]
fn test_jsonl_roundtrip() {
  let session = create_test_session_with_answers("roundtrip");
  let line_result = session_to_jsonl_line(&session);
  assert!(line_result.is_ok());
  let line = match line_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "serialization failed: {error}");
      return;
    }
  };

  let parsed_result =
    serde_json::from_str::<crate::intent::interview::types::InterviewSession>(&line);
  assert!(parsed_result.is_ok());
  let parsed = match parsed_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "deserialization failed: {error}");
      return;
    }
  };

  assert_eq!(parsed.id, "roundtrip");
  assert_eq!(parsed.answers.len(), 2);
}

#[test]
fn test_append_and_list_jsonl() {
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = match temp_file_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "temp file failed: {error}");
      return;
    }
  };
  let path = temp_file.path();

  let first = create_test_session("s1");
  let mut second = create_test_session("s2");

  let append_first = append_session_to_jsonl(&first, path);
  assert!(append_first.is_ok());
  let append_second = append_session_to_jsonl(&second, path);
  assert!(append_second.is_ok());

  second.raw_notes = "updated".to_string();
  let update_second = append_session_to_jsonl(&second, path);
  assert!(update_second.is_ok());

  let sessions_result = list_sessions_from_jsonl(path);
  assert!(sessions_result.is_ok());
  let sessions = match sessions_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "list sessions failed: {error}");
      return;
    }
  };
  assert_eq!(sessions.len(), 2);
  let updated_option = sessions.iter().find(|session| session.id == "s2");
  assert!(updated_option.is_some());
  let updated = if let Some(value) = updated_option {
    value
  } else {
    assert!(false, "updated session should exist");
    return;
  };
  assert_eq!(updated.raw_notes, "updated");
}

#[test]
fn test_get_session_not_found() {
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = match temp_file_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "temp file failed: {error}");
      return;
    }
  };
  let path = temp_file.path();

  let result = get_session_from_jsonl(path, "missing");
  assert!(matches!(result, Err(StorageError::SessionNotFound(_))));
}

#[test]
fn test_history_append_and_filter() {
  let temp_file_result = NamedTempFile::new();
  assert!(temp_file_result.is_ok());
  let temp_file = match temp_file_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "temp file failed: {error}");
      return;
    }
  };
  let path = temp_file.path();
  let first = create_test_session("s1");
  let second = create_test_session("s2");

  assert!(append_to_history(&first, "first", path).is_ok());
  assert!(append_to_history(&second, "second", path).is_ok());
  assert!(append_to_history(&first, "third", path).is_ok());

  let history_result = list_session_history(path, "s1");
  assert!(history_result.is_ok());
  let history = match history_result {
    Ok(value) => value,
    Err(error) => {
      assert!(false, "history list failed: {error}");
      return;
    }
  };
  assert_eq!(history.len(), 2);
  assert!(history.iter().all(|entry| entry.session_id == "s1"));
}

#[test]
fn test_create_snapshot_contents() {
  let mut session = create_test_session_with_answers("snap-1");
  session.gaps.push(Gap::default());
  let snapshot = create_snapshot(&session, "checkpoint");

  assert_eq!(snapshot.session_id, "snap-1");
  assert_eq!(snapshot.description, "checkpoint");
  assert_eq!(snapshot.answers.len(), 2);
  assert_eq!(snapshot.gaps_count, 1);
  assert_eq!(snapshot.stage, "discovery");
}

#[test]
fn test_diff_sessions_counts() {
  let mut from = create_test_session_with_answers("diff-1");
  let mut to = from.clone();
  to.answers[0].response = "GraphQL".to_string();
  to.answers.push(Answer {
    question_id: "q3".to_string(),
    question_text: "New question".to_string(),
    response: "New response".to_string(),
    ..Answer::default()
  });
  from.answers.push(Answer {
    question_id: "q4".to_string(),
    question_text: "Removed".to_string(),
    response: "Removed response".to_string(),
    ..Answer::default()
  });
  to.gaps.push(Gap::default());

  let diff = diff_sessions(&from, &to);

  assert_eq!(diff.answers_added.len(), 1);
  assert_eq!(diff.answers_modified.len(), 1);
  assert_eq!(diff.answers_removed.len(), 1);
  assert_eq!(diff.gaps_added, 1);
}

#[test]
fn test_format_diff_has_sections() {
  let from = create_test_session("fmt-1");
  let mut to = from.clone();
  to.answers.push(Answer {
    question_id: "q1".to_string(),
    question_text: "What is the API?".to_string(),
    response: "response".to_string(),
    ..Answer::default()
  });

  let diff = diff_sessions(&from, &to);
  let rendered = format_diff(&diff);
  assert!(rendered.contains("Session Diff:"));
  assert!(rendered.contains("Answers Added"));
}

#[test]
fn test_diff_snapshots_uses_typed_stage_mapping() {
  let mut from = create_test_session("snap-stage");
  from.stage = InterviewStage::Discovery;
  let mut to = from.clone();
  to.stage = InterviewStage::Discovery;

  let mut from_snapshot = create_snapshot(&from, "from");
  let to_snapshot = create_snapshot(&to, "to");
  from_snapshot.stage = "Discovery".to_string();

  let diff = diff_snapshots(&from_snapshot, &to_snapshot);

  assert!(!diff.stage_changed);
  assert_eq!(diff.old_stage, Some("discovery".to_string()));
  assert_eq!(diff.new_stage, Some("discovery".to_string()));
}

#[test]
fn test_storage_error_display() {
  assert_eq!(
    StorageError::IoError("file missing".to_string()).to_string(),
    "I/O error: file missing"
  );

  assert_eq!(
    StorageError::InvalidJsonLine {
      line: 2,
      error: "bad json".to_string(),
    }
    .to_string(),
    "invalid JSON on line 2: bad json"
  );
}

#[test]
fn test_answer_version_creation() {
  let version = AnswerVersion::new(
    1,
    "Test response".to_string(),
    "q1".to_string(),
    "Initial answer".to_string(),
    "2026-02-28T00:00:00Z".to_string(),
  );

  assert_eq!(version.version, 1);
  assert_eq!(version.response, "Test response");
  assert_eq!(version.change_reason, "Initial answer");
}

#[test]
fn test_answer_with_history_new() {
  let history = AnswerWithHistory::new("q1", "First response", "Initial");

  assert_eq!(history.len(), 1);
  assert!(!history.is_empty());

  let current = history.current().expect("should have current version");
  assert_eq!(current.response, "First response");
  assert_eq!(current.version, 1);
}

#[test]
fn test_answer_with_history_add_version() {
  let mut history = AnswerWithHistory::new("q1", "First", "Initial");
  history.add_version("Second response", "User corrected");

  assert_eq!(history.len(), 2);

  let current = history.current().expect("should have current version");
  assert_eq!(current.response, "Second response");
  assert_eq!(current.version, 2);
}

#[test]
fn test_answer_with_history_get_version() {
  let mut history = AnswerWithHistory::new("q1", "v1", "init");
  history.add_version("v2", "fix1");
  history.add_version("v3", "fix2");

  let v1 = history.get_version(0).expect("version 1 should exist");
  assert_eq!(v1.response, "v1");
  assert_eq!(v1.version, 1);

  let v2 = history.get_version(1).expect("version 2 should exist");
  assert_eq!(v2.response, "v2");

  let v3 = history.get_version(2).expect("version 3 should exist");
  assert_eq!(v3.response, "v3");

  assert!(history.get_version(5).is_none());
}

#[test]
fn test_answer_with_history_serialization() {
  let history = AnswerWithHistory::new("q1", "response", "reason");

  let json = serde_json::to_string(&history).expect("should serialize");
  let deserialized: AnswerWithHistory = serde_json::from_str(&json).expect("should deserialize");

  assert_eq!(deserialized.len(), 1);
  assert_eq!(deserialized.current().unwrap().response, "response");
}
