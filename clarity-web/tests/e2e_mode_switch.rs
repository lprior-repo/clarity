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
#![forbid(unsafe_code)]

//! End-to-end test: Mode switching with state preservation (bead e2e-004)

use clarity_web::storage::{ProjectMetadata, RedbStore};

/// Test: Mode preference persists to storage
#[tokio::test]
async fn test_mode_preference_persists() {
  let storage = RedbStore::open_in_memory().unwrap();

  // Save guided mode preference
  let metadata =
    ProjectMetadata::with_current_timestamp("guided".to_string(), "discover".to_string());
  storage.save_metadata(&metadata).unwrap();

  // Load and verify
  let loaded = storage.get_metadata().unwrap().unwrap();
  assert_eq!(loaded.mode_preference, "guided");
  assert_eq!(loaded.current_phase, "discover");
}

/// Test: Mode preference can be updated
#[tokio::test]
async fn test_mode_preference_update() {
  let storage = RedbStore::open_in_memory().unwrap();

  // Start with guided mode
  let metadata1 =
    ProjectMetadata::with_current_timestamp("guided".to_string(), "discover".to_string());
  storage.save_metadata(&metadata1).unwrap();

  // Update to express mode
  let metadata2 =
    ProjectMetadata::with_current_timestamp("express".to_string(), "discover".to_string());
  storage.save_metadata(&metadata2).unwrap();

  // Verify latest value
  let loaded = storage.get_metadata().unwrap().unwrap();
  assert_eq!(loaded.mode_preference, "express");
}

/// Test: Answer storage and retrieval
#[tokio::test]
async fn test_answer_storage() {
  let storage = RedbStore::open_in_memory().unwrap();

  // Save an answer
  let answer = clarity_web::types::Answer {
    step_id: "problem".to_string(),
    value: "Remote teams struggle with task management".to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
  };
  storage.save_answer(&answer).unwrap();

  // Load and verify
  let loaded = storage.get_answer("problem").unwrap().unwrap();
  assert_eq!(loaded.step_id, "problem");
  assert_eq!(loaded.value, "Remote teams struggle with task management");
}

/// Test: Multiple answers persist
#[tokio::test]
async fn test_multiple_answers_persist() {
  let storage = RedbStore::open_in_memory().unwrap();

  // Save multiple answers
  let answers = vec![
    clarity_web::types::Answer {
      step_id: "problem".to_string(),
      value: "Task management issue".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    },
    clarity_web::types::Answer {
      step_id: "user".to_string(),
      value: "Remote project managers".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    },
    clarity_web::types::Answer {
      step_id: "context".to_string(),
      value: "Distributed teams across time zones".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    },
  ];

  for answer in &answers {
    storage.save_answer(answer).unwrap();
  }

  // Load all and verify
  let loaded = storage.get_all_answers().unwrap();
  assert_eq!(loaded.len(), 3);

  // Verify each answer exists
  let step_ids: std::collections::HashSet<_> = loaded.iter().map(|a| a.step_id.as_str()).collect();
  assert!(step_ids.contains("problem"));
  assert!(step_ids.contains("user"));
  assert!(step_ids.contains("context"));
}

/// Test: Full round-trip mode switch scenario
#[tokio::test]
async fn test_full_mode_switch_round_trip() {
  let storage = RedbStore::open_in_memory().unwrap();

  // 1. Start in Express mode - enter freeform text
  let express_content = "Problem: Remote teams lose track of tasks. User: Project managers. Context: Multiple time zones.";
  // The literal above captures the expected express-mode prompt body so the
  // value is bound and cannot be flagged as a no-effect underscore binding
  // while still being available for downstream assertions if added later.
  let _ = express_content;

  // 2. Extract fields (simulated) - save to storage
  let extracted_answers = vec![
    clarity_web::types::Answer {
      step_id: "problem".to_string(),
      value: "Remote teams lose track of tasks".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    },
    clarity_web::types::Answer {
      step_id: "user".to_string(),
      value: "Project managers".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    },
  ];

  for answer in &extracted_answers {
    storage.save_answer(answer).unwrap();
  }

  // 3. Switch to Guided - save mode preference
  let guided_metadata =
    ProjectMetadata::with_current_timestamp("guided".to_string(), "discover".to_string());
  storage.save_metadata(&guided_metadata).unwrap();

  // Verify mode is guided
  let loaded_meta = storage.get_metadata().unwrap().unwrap();
  assert_eq!(loaded_meta.mode_preference, "guided");

  // 4. Answer remaining questions in Guided
  storage
    .save_answer(&clarity_web::types::Answer {
      step_id: "solution".to_string(),
      value: "Smart task aggregation".to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
    })
    .unwrap();

  // 5. Switch back to Express - verify all answers preserved
  let express_metadata =
    ProjectMetadata::with_current_timestamp("express".to_string(), "discover".to_string());
  storage.save_metadata(&express_metadata).unwrap();

  // Load all answers
  let all_answers = storage.get_all_answers().unwrap();
  assert_eq!(all_answers.len(), 3);

  // Verify no data loss
  let step_ids: Vec<_> = all_answers.iter().map(|a| a.step_id.as_str()).collect();
  assert!(step_ids.contains(&"problem"));
  assert!(step_ids.contains(&"user"));
  assert!(step_ids.contains(&"solution"));

  // Verify mode is express
  let final_meta = storage.get_metadata().unwrap().unwrap();
  assert_eq!(final_meta.mode_preference, "express");
}

/// Test: Mode preference survives storage reload
#[tokio::test]
async fn test_mode_preference_survives_reload() {
  let storage = std::sync::Arc::new(RedbStore::open_in_memory().unwrap());

  // First session: Set mode to Guided
  let metadata1 =
    ProjectMetadata::with_current_timestamp("guided".to_string(), "discover".to_string());
  storage.save_metadata(&metadata1).unwrap();

  // Simulate reload: Read from storage
  let loaded = storage.get_metadata().unwrap().unwrap();
  assert_eq!(loaded.mode_preference, "guided");

  // Second session: Update to Express
  let metadata2 =
    ProjectMetadata::with_current_timestamp("express".to_string(), "define".to_string());
  storage.save_metadata(&metadata2).unwrap();

  // Verify update persisted
  let final_state = storage.get_metadata().unwrap().unwrap();
  assert_eq!(final_state.mode_preference, "express");
  assert_eq!(final_state.current_phase, "define");
}
