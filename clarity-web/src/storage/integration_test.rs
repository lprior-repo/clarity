#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for storage layer with Discover phase.
//!
//! These tests demonstrate the complete workflow of:
//! 1. Creating a `RedbStore` instance
//! 2. Loading answers on mount
//! 3. Saving each answer on input
//! 4. Saving mode preference to metadata
//! 5. Saving extraction cache
//! 6. Cleanup on unmount (close store)

use crate::storage::redb_store::RedbStore;
use crate::storage::types::{ExtractionCache, ProjectMetadata};
use crate::types::Answer;
use tempfile::TempDir;

fn require_some<T>(value: Option<T>, context: &str) -> Result<T, std::io::Error> {
  value.ok_or_else(|| std::io::Error::other(format!("missing value: {context}")))
}

/// Simulate the complete Discover phase workflow with storage.
#[test]
fn test_discover_storage_workflow() -> Result<(), Box<dyn std::error::Error>> {
  // Create a temporary directory for the database
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  // 1. Create RedbStore instance on app init
  let store = RedbStore::open(&db_path)?;

  // 2. Load answers on mount (initially empty)
  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 0, "Initial database should be empty");

  // 3. User enters answers - simulate input events
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "The system needs to handle real-time data processing".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  // Save immediately on input
  store.save_answer(&answer1)?;

  // Verify answer was persisted
  let loaded = store.get_all_answers()?;
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
  store.save_answer(&answer2)?;

  // Verify both answers are persisted
  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 2);

  // 4. Save mode preference to metadata
  let now = chrono::Utc::now().to_rfc3339();
  let metadata = ProjectMetadata::new(
    "guided".to_string(),
    "discover".to_string(),
    now.clone(),
    now,
  );

  store.save_metadata(&metadata)?;

  // Verify mode preference persisted
  let loaded_metadata = require_some(store.get_metadata()?, "metadata")?;
  assert_eq!(loaded_metadata.mode_preference, "guided");
  assert_eq!(loaded_metadata.current_phase, "discover");

  // 5. Save extraction cache on successful extraction
  let cache = ExtractionCache::new(
    "input-hash-123".to_string(),
    r#"{"problem": "real-time data", "solution": "streaming"}"#.to_string(),
    chrono::Utc::now().to_rfc3339(),
  );

  store.save_extraction_cache("input-hash-123", &cache)?;

  // Verify cache persisted
  let loaded_cache = require_some(
    store.get_extraction_cache("input-hash-123")?,
    "extraction cache input-hash-123",
  )?;
  assert_eq!(
    loaded_cache.fields,
    r#"{"problem": "real-time data", "solution": "streaming"}"#
  );

  // 6. Cleanup on unmount (RedbStore closes on Drop)
  // The store will be dropped here, closing the database
  drop(store);

  // Verify persistence - reopen and check data
  let store2 = RedbStore::open(&db_path)?;

  let loaded = store2.load_answers()?;
  assert_eq!(
    loaded.len(),
    2,
    "Answers should persist across store restarts"
  );

  let loaded_metadata = store2.get_metadata()?;
  assert!(loaded_metadata.is_some(), "Metadata should persist");

  let loaded_cache = store2.get_extraction_cache("input-hash-123")?;
  assert!(loaded_cache.is_some(), "Extraction cache should persist");

  Ok(())
}

/// Test that answers update correctly on repeated inputs.
#[test]
fn test_answer_update_workflow() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path)?;

  // User enters initial answer
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "Initial problem statement".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store.save_answer(&answer1)?;

  // User revisits and updates the answer
  let answer2 = Answer {
    step_id: "problem".to_string(),
    value: "Revised problem statement with more detail".to_string(),
    timestamp: "2024-02-25T12:05:00Z".to_string(),
  };

  store.save_answer(&answer2)?;

  // Verify only the updated answer exists
  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 1);
  assert_eq!(
    loaded[0].value,
    "Revised problem statement with more detail"
  );
  assert_eq!(loaded[0].timestamp, "2024-02-25T12:05:00Z");

  Ok(())
}

/// Test that mode preference changes persist.
#[test]
fn test_mode_preference_persistence() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path)?;

  // Initial mode: guided
  let now1 = chrono::Utc::now().to_rfc3339();
  let metadata1 = ProjectMetadata::new(
    "guided".to_string(),
    "discover".to_string(),
    now1.clone(),
    now1,
  );

  store.save_metadata(&metadata1)?;

  let loaded1 = require_some(store.get_metadata()?, "initial metadata")?;
  assert_eq!(loaded1.mode_preference, "guided");

  // User switches mode to express
  let now2 = chrono::Utc::now().to_rfc3339();
  let metadata2 = ProjectMetadata::new(
    "express".to_string(),
    "discover".to_string(),
    now2.clone(),
    now2,
  );

  store.save_metadata(&metadata2)?;

  let loaded2 = require_some(store.get_metadata()?, "updated metadata")?;
  assert_eq!(loaded2.mode_preference, "express");

  // Verify persistence across store restarts
  drop(store);
  let store2 = RedbStore::open(&db_path)?;

  let loaded3 = require_some(store2.get_metadata()?, "reloaded metadata")?;
  assert_eq!(loaded3.mode_preference, "express");

  Ok(())
}

/// Test that extraction cache improves performance (simulated).
#[test]
fn test_extraction_cache_workflow() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path)?;

  // First extraction - cache miss
  let hash1 = "input-hash-abc";
  let cache1 = store.get_extraction_cache(hash1)?;

  assert!(cache1.is_none(), "Initial cache should be empty");

  // Save extraction result to cache
  let extraction = ExtractionCache::new(
    hash1.to_string(),
    r#"{"field": "extracted value"}"#.to_string(),
    chrono::Utc::now().to_rfc3339(),
  );

  store.save_extraction_cache(hash1, &extraction)?;

  // Second extraction - cache hit
  let cache2 = require_some(store.get_extraction_cache(hash1)?, "cached extraction")?;

  assert_eq!(cache2.fields, r#"{"field": "extracted value"}"#);

  Ok(())
}

/// Test deletion workflow.
#[test]
fn test_answer_deletion_workflow() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path)?;

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
    store.save_answer(answer)?;
  }

  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 2);

  // User deletes one answer
  let deleted = store.delete_answer("problem")?;

  assert!(deleted, "Delete should succeed");

  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].step_id, "solution");

  Ok(())
}

/// Test multiple projects with separate stores.
#[test]
fn test_multiple_projects_isolation() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;

  let project1_db = temp_dir.path().join("project1.redb");
  let project2_db = temp_dir.path().join("project2.redb");

  let store1 = RedbStore::open(&project1_db)?;
  let store2 = RedbStore::open(&project2_db)?;

  // Store data in project1
  let answer1 = Answer {
    step_id: "problem".to_string(),
    value: "Project 1 problem".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store1.save_answer(&answer1)?;

  // Store data in project2
  let answer2 = Answer {
    step_id: "problem".to_string(),
    value: "Project 2 problem".to_string(),
    timestamp: "2024-02-25T12:00:00Z".to_string(),
  };

  store2.save_answer(&answer2)?;

  // Verify isolation
  let loaded1 = store1.load_answers()?;
  let loaded2 = store2.load_answers()?;

  assert_eq!(loaded1.len(), 1);
  assert_eq!(loaded2.len(), 1);
  assert_eq!(loaded1[0].value, "Project 1 problem");
  assert_eq!(loaded2[0].value, "Project 2 problem");

  Ok(())
}

/// Test clearing all answers.
#[test]
fn test_clear_all_answers() -> Result<(), Box<dyn std::error::Error>> {
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.redb");

  let store = RedbStore::open(&db_path)?;

  // Save multiple answers
  for i in 0..5 {
    let answer = Answer {
      step_id: format!("step-{i}"),
      value: format!("Answer {i}"),
      timestamp: "2024-02-25T12:00:00Z".to_string(),
    };

    store.save_answer(&answer)?;
  }

  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 5);

  // Clear all by deleting each answer
  for answer in &loaded {
    store.delete_answer(&answer.step_id)?;
  }

  let loaded = store.get_all_answers()?;
  assert_eq!(loaded.len(), 0, "All answers should be cleared");

  Ok(())
}
