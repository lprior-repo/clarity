#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for storage layer with Discover phase.
//!
//! These tests demonstrate the complete workflow of:
//! 1. Creating a RedbStore instance
//! 2. Loading answers on mount
//! 3. Saving each answer on input
//! 4. Saving mode preference to metadata
//! 5. Saving extraction cache
//! 6. Cleanup on unmount (close store)

use crate::storage::redb_store::RedbStore;
use crate::storage::types::{ExtractionCache, ProjectMetadata};
use crate::types::Answer;
use tempfile::TempDir;

/// Simulate the complete Discover phase workflow with storage.
#[test]
fn test_discover_storage_workflow() {
  // Create a temporary directory for the database
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  // 1. Create RedbStore instance on app init
  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // 2. Load answers on mount (initially empty)
  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 0, "Initial database should be empty");

  // 3. User enters answers - simulate input events
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "The system needs to handle real-time data processing".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  // Save immediately on input
  store.save_answer(&answer1).expect("Failed to save answer1");

  // Verify answer was persisted
  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].step_id, "problem");
  assert_eq!(loaded[0].value, answer1.value);

  // User enters another answer
  let answer2 = Answer {
    step_id: "solution".to_string(),
    value: "Use a streaming architecture with Kafka".to_string(),
    timestamp: "2024-02-25T12:01:00Z".to_string(),
  };

  // Save immediately on input
  store.save_answer(&answer2).expect("Failed to save answer2");

  // Verify both answers are persisted
  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 2);

  // 4. Save mode preference to metadata
  let now = chrono::Utc::now().to_rfc3339();
  let metadata = ProjectMetadata::new(
    "guided".to_string(),
    "discover".to_string(),
    now.clone(),
    now,
  );

  store
    .save_metadata(&metadata)
    .expect("Failed to save metadata");

  // Verify mode preference persisted
  let loaded_metadata = store.get_metadata().expect("Failed to get metadata");
  assert!(loaded_metadata.is_some());
  assert_eq!(loaded_metadata.as_ref().unwrap().mode_preference, "guided");
  assert_eq!(loaded_metadata.as_ref().unwrap().current_phase, "discover");

  // 5. Save extraction cache on successful extraction
  let cache = ExtractionCache::new(
    "input-hash-123".to_string(),
    r#"{"problem": "real-time data", "solution": "streaming"}"#.to_string(),
    chrono::Utc::now().to_rfc3339(),
  );

  store
    .save_extraction_cache("input-hash-123", &cache)
    .expect("Failed to save extraction cache");

  // Verify cache persisted
  let loaded_cache = store
    .get_extraction_cache("input-hash-123")
    .expect("Failed to get cache");
  assert!(loaded_cache.is_some());
  assert_eq!(
    loaded_cache.as_ref().unwrap().fields,
    r#"{"problem": "real-time data", "solution": "streaming"}"#
  );

  // 6. Cleanup on unmount (RedbStore closes on Drop)
  // The store will be dropped here, closing the database
  drop(store);

  // Verify persistence - reopen and check data
  let store2 = RedbStore::open(&db_path).expect("Failed to reopen store");

  let loaded = store2.load_answers().expect("Failed to load answers");
  assert_eq!(
    loaded.len(),
    2,
    "Answers should persist across store restarts"
  );

  let loaded_metadata = store2.get_metadata().expect("Failed to get metadata");
  assert!(loaded_metadata.is_some(), "Metadata should persist");

  let loaded_cache = store2
    .get_extraction_cache("input-hash-123")
    .expect("Failed to get cache");
  assert!(loaded_cache.is_some(), "Extraction cache should persist");
}

/// Test that answers update correctly on repeated inputs.
#[test]
fn test_answer_update_workflow() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // User enters initial answer
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "Initial problem statement".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store
    .save_answer(&answer1)
    .expect("Failed to save initial answer");

  // User revisits and updates the answer
  let answer2 = Answer {
    step_id: "problem".to_string(),
    value: "Revised problem statement with more detail".to_string(),
    timestamp: "2024-02-25T12:05:00Z".to_string(),
  };

  store
    .save_answer(&answer2)
    .expect("Failed to save updated answer");

  // Verify only the updated answer exists
  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 1);
  assert_eq!(
    loaded[0].value,
    "Revised problem statement with more detail"
  );
  assert_eq!(loaded[0].timestamp, "2024-02-25T12:05:00Z");
}

/// Test that mode preference changes persist.
#[test]
fn test_mode_preference_persistence() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // Initial mode: guided
  let now1 = chrono::Utc::now().to_rfc3339();
  let metadata1 = ProjectMetadata::new(
    "guided".to_string(),
    "discover".to_string(),
    now1.clone(),
    now1,
  );

  store
    .save_metadata(&metadata1)
    .expect("Failed to save initial metadata");

  let loaded1 = store.get_metadata().expect("Failed to get metadata");
  assert_eq!(loaded1.as_ref().unwrap().mode_preference, "guided");

  // User switches mode to express
  let now2 = chrono::Utc::now().to_rfc3339();
  let metadata2 = ProjectMetadata::new(
    "express".to_string(),
    "discover".to_string(),
    now2.clone(),
    now2,
  );

  store
    .save_metadata(&metadata2)
    .expect("Failed to save updated metadata");

  let loaded2 = store.get_metadata().expect("Failed to get metadata");
  assert_eq!(loaded2.as_ref().unwrap().mode_preference, "express");

  // Verify persistence across store restarts
  drop(store);
  let store2 = RedbStore::open(&db_path).expect("Failed to reopen store");

  let loaded3 = store2.get_metadata().expect("Failed to get metadata");
  assert_eq!(loaded3.as_ref().unwrap().mode_preference, "express");
}

/// Test that extraction cache improves performance (simulated).
#[test]
fn test_extraction_cache_workflow() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // First extraction - cache miss
  let hash1 = "input-hash-abc";
  let cache1 = store
    .get_extraction_cache(hash1)
    .expect("Failed to check cache");

  assert!(cache1.is_none(), "Initial cache should be empty");

  // Save extraction result to cache
  let extraction = ExtractionCache::new(
    hash1.to_string(),
    r#"{"field": "extracted value"}"#.to_string(),
    chrono::Utc::now().to_rfc3339(),
  );

  store
    .save_extraction_cache(hash1, &extraction)
    .expect("Failed to save to cache");

  // Second extraction - cache hit
  let cache2 = store
    .get_extraction_cache(hash1)
    .expect("Failed to check cache");

  assert!(cache2.is_some(), "Cache should contain the extraction");
  assert_eq!(
    cache2.as_ref().unwrap().fields,
    r#"{"field": "extracted value"}"#
  );
}

/// Test deletion workflow.
#[test]
fn test_answer_deletion_workflow() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // Save some answers
  let answers = vec![
    Answer {
      step_id: "problem".to_string(),
      value: "Problem".to_string(),
      timestamp: "2024-02-25T12:00:00Z".to_string(),
    },
    Answer {
      step_id: "solution".to_string(),
      value: "Solution".to_string(),
      timestamp: "2024-02-25T12:01:00Z".to_string(),
    },
  ];

  for answer in &answers {
    store.save_answer(answer).expect("Failed to save answer");
  }

  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 2);

  // User deletes one answer
  let deleted = store
    .delete_answer("problem")
    .expect("Failed to delete answer");

  assert!(deleted, "Delete should succeed");

  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].step_id, "solution");
}

/// Test multiple projects with separate stores.
#[test]
fn test_multiple_projects_isolation() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");

  let project1_db = temp_dir.path().join("project1.redb");
  let project2_db = temp_dir.path().join("project2.redb");

  let store1 = RedbStore::open(&project1_db).expect("Failed to open store1");
  let store2 = RedbStore::open(&project2_db).expect("Failed to open store2");

  // Store data in project1
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "Project 1 problem".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store1
    .save_answer(&answer1)
    .expect("Failed to save to store1");

  // Store data in project2
  let answer2 = Answer {
    step_id: "problem".to_string(),
    value: "Project 2 problem".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store2
    .save_answer(&answer2)
    .expect("Failed to save to store2");

  // Verify isolation
  let loaded1 = store1.load_answers().expect("Failed to load from store1");
  let loaded2 = store2.load_answers().expect("Failed to load from store2");

  assert_eq!(loaded1.len(), 1);
  assert_eq!(loaded2.len(), 1);
  assert_eq!(loaded1[0].value, "Project 1 problem");
  assert_eq!(loaded2[0].value, "Project 2 problem");
}

/// Test clearing all answers.
#[test]
fn test_clear_all_answers() {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open store");

  // Save multiple answers
  for i in 0..5 {
    let answer = Answer {
      step_id: format!("step-{}", i),
      value: format!("Answer {}", i),
      timestamp: "2024-02-25T12:00:00Z".to_string(),
    };

    store.save_answer(&answer).expect("Failed to save answer");
  }

  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 5);

  // Clear all
  store.clear_answers().expect("Failed to clear answers");

  let loaded = store.get_all_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 0, "All answers should be cleared");
}
