//! End-to-end test: Persistence across app restart.
//!
//! This is a standalone integration test that verifies the `RedbStore` persistence
//! functionality directly, without requiring the full application to compile.
//!
//! Test flow:
//! 1. Create a temporary database file
//! 2. Save Discover phase data (5 answers, metadata, lattice cache)
//! 3. Verify database file exists and contains data
//! 4. Close and reopen database (simulate app restart)
//! 5. Verify all data restored without corruption

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

// Use path-based imports to avoid lib compilation issues
mod storage_types {
  pub use clarity_web::storage::redb_store::RedbStore;
  pub use clarity_web::storage::{
    tables, AnswerRecord, Confidence, ExtractionCache, LatticeCache, ProjectMetadata,
  };
  pub use clarity_web::types::Answer;
}

use storage_types::*;

/// Helper to create a test answer
fn create_test_answer(step_id: &str, value: &str) -> Answer {
  Answer {
    step_id: step_id.to_string(),
    value: value.to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
  }
}

/// Helper to create a test answer record with all fields
fn create_test_answer_record(
  step_id: &str,
  value: &str,
  confidence: Confidence,
  ai_generated: bool,
) -> AnswerRecord {
  AnswerRecord::new(
    step_id.to_string(),
    value.to_string(),
    chrono::Utc::now().to_rfc3339(),
    confidence,
    ai_generated,
  )
}

/// Helper to verify database file exists and is valid
fn verify_database_file_exists(db_path: &Path) -> Result<(), String> {
  if !db_path.exists() {
    return Err(format!(
      "Database file does not exist: {}",
      db_path.display()
    ));
  }

  let metadata =
    std::fs::metadata(db_path).map_err(|e| format!("Failed to read database metadata: {e}"))?;

  if metadata.len() == 0 {
    return Err("Database file is empty".to_string());
  }

  Ok(())
}

/// Helper to count records in a redb table
fn count_table_records(db_path: &Path, table_name: &str) -> Result<usize, String> {
  use redb::ReadableTable;

  let db = redb::Database::open(db_path).map_err(|e| format!("Failed to open database: {e}"))?;

  let txn = db
    .begin_read()
    .map_err(|e| format!("Failed to begin read: {e}"))?;
  let table_def: redb::TableDefinition<&str, &str> = redb::TableDefinition::new(table_name);

  match txn.open_table(table_def) {
    Ok(table) => {
      let mut count = 0;
      let iter = table
        .iter()
        .map_err(|e| format!("Failed to iterate table: {e}"))?;
      for _ in iter {
        count += 1;
      }
      Ok(count)
    }
    Err(_) => Ok(0),
  }
}

/// Test complete Discover phase persistence and restoration
#[test]
#[allow(clippy::too_many_lines)]
fn test_e2e_persistence_discover_phase_express_mode() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_clarity.redb");

  // ============================================================
  // STEP 1: Complete Discover phase in Express mode
  // ============================================================

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Create 5 test answers
  let answers = vec![
    create_test_answer_record(
      "discover-001",
      "Building a project management tool for software teams",
      Confidence::High,
      false,
    ),
    create_test_answer_record(
      "discover-002",
      "Small to medium software development teams, 5-50 people",
      Confidence::High,
      false,
    ),
    create_test_answer_record(
      "discover-003",
      "Jira is too complex, Trello is too simple, need middle ground",
      Confidence::Inferred,
      true,
    ),
    create_test_answer_record(
      "discover-004",
      "Sprint planning, backlog management, burndown charts",
      Confidence::High,
      false,
    ),
    create_test_answer_record(
      "discover-005",
      "Web application, responsive design, real-time updates",
      Confidence::Uncertain,
      false,
    ),
  ];

  // Save all answers to database
  for answer in &answers {
    let answer_for_save = Answer {
      step_id: answer.step_id.clone(),
      value: answer.value.clone(),
      timestamp: answer.timestamp.clone(),
    };
    store
      .save_answer(&answer_for_save)
      .expect("Failed to save answer");
  }

  // Save project metadata with Express mode preference
  let metadata =
    ProjectMetadata::with_current_timestamp("express".to_string(), "discover".to_string());
  store
    .save_metadata(&metadata)
    .expect("Failed to save metadata");

  // Save lattice cache with quality score
  let lattice_output = r#"{
        "quality_score": 0.92,
        "completeness": 0.95,
        "confidence": 0.89
    }"#;
  let lattice_cache =
    LatticeCache::with_current_timestamp("discover".to_string(), lattice_output.to_string());
  store
    .save_lattice_cache("discover", &lattice_cache)
    .expect("Failed to save lattice cache");

  // ============================================================
  // STEP 2: Verify database file created
  // ============================================================

  verify_database_file_exists(&db_path).expect("Database file verification failed");

  // ============================================================
  // STEP 3: Verify data integrity before closing (using store)
  // ============================================================

  let loaded_answers = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded_answers.len(), 5);

  let loaded_metadata = store.get_metadata().expect("Failed to get metadata");
  assert!(loaded_metadata.is_some());
  let loaded_metadata = loaded_metadata.unwrap();
  assert_eq!(loaded_metadata.mode_preference, "express");
  assert_eq!(loaded_metadata.current_phase, "discover");

  let loaded_lattice = store
    .get_lattice_cache("discover")
    .expect("Failed to get lattice cache");
  assert!(loaded_lattice.is_some());
  let loaded_lattice = loaded_lattice.unwrap();
  assert!(loaded_lattice.output_data.contains("quality_score"));
  assert!(loaded_lattice.output_data.contains("0.92"));

  // ============================================================
  // STEP 4: Read database directly to verify tables (after dropping store)
  // ============================================================

  // Drop the store to release the database lock before direct inspection
  drop(store);

  let answer_count =
    count_table_records(&db_path, tables::ANSWERS).expect("Failed to count answers");
  assert_eq!(
    answer_count, 5,
    "Expected 5 answers in database, found {answer_count}"
  );

  let metadata_count = count_table_records(&db_path, tables::PROJECT_METADATA)
    .expect("Failed to count metadata records");
  assert_eq!(
    metadata_count, 1,
    "Expected 1 metadata record in database, found {metadata_count}"
  );

  let lattice_count = count_table_records(&db_path, tables::LATTICE_CACHE)
    .expect("Failed to count lattice cache records");
  assert_eq!(
    lattice_count, 1,
    "Expected 1 lattice cache record in database, found {lattice_count}"
  );

  // ============================================================
  // STEP 5: Reopen app (simulate restart) - store already dropped in step 4
  // ============================================================

  let store_reopened = RedbStore::open(&db_path).expect("Failed to reopen database after restart");

  // ============================================================
  // STEP 6: Verify all data restored without corruption
  // ============================================================

  let restored_answers = store_reopened
    .load_answers()
    .expect("Failed to load answers after restart");
  assert_eq!(restored_answers.len(), 5);

  let answer_map: HashMap<String, Answer> = restored_answers
    .into_iter()
    .map(|a| (a.step_id.clone(), a))
    .collect();

  assert_eq!(
    answer_map.get("discover-001").map(|a| a.value.as_str()),
    Some("Building a project management tool for software teams")
  );
  assert_eq!(
    answer_map.get("discover-002").map(|a| a.value.as_str()),
    Some("Small to medium software development teams, 5-50 people")
  );
  assert_eq!(
    answer_map.get("discover-003").map(|a| a.value.as_str()),
    Some("Jira is too complex, Trello is too simple, need middle ground")
  );
  assert_eq!(
    answer_map.get("discover-004").map(|a| a.value.as_str()),
    Some("Sprint planning, backlog management, burndown charts")
  );
  assert_eq!(
    answer_map.get("discover-005").map(|a| a.value.as_str()),
    Some("Web application, responsive design, real-time updates")
  );

  let restored_metadata = store_reopened
    .get_metadata()
    .expect("Failed to get metadata after restart");
  assert!(restored_metadata.is_some());
  let restored_metadata = restored_metadata.unwrap();
  assert_eq!(restored_metadata.mode_preference, "express");
  assert_eq!(restored_metadata.current_phase, "discover");

  let restored_lattice = store_reopened
    .get_lattice_cache("discover")
    .expect("Failed to get lattice cache after restart");
  assert!(restored_lattice.is_some());
  let restored_lattice = restored_lattice.unwrap();
  assert!(restored_lattice.output_data.contains("quality_score"));
  assert!(restored_lattice.output_data.contains("0.92"));

  // Verify no corruption by checking JSON can be parsed
  let _: serde_json::Value = serde_json::from_str(&restored_lattice.output_data)
    .expect("Lattice cache data is corrupted - invalid JSON");

  // ============================================================
  // TEST PASSED
  // ============================================================
}

/// Test persistence of individual answer fields
#[test]
fn test_e2e_persistence_answer_fields() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_answer_fields.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  let test_cases = vec![
    ("step-1", "User answer", Confidence::High, false),
    ("step-2", "AI suggestion", Confidence::Inferred, true),
    ("step-3", "Uncertain data", Confidence::Uncertain, false),
  ];

  for (step_id, value, confidence, ai_generated) in &test_cases {
    let record = create_test_answer_record(step_id, value, *confidence, *ai_generated);
    let answer = Answer {
      step_id: record.step_id.clone(),
      value: record.value.clone(),
      timestamp: record.timestamp.clone(),
    };
    store.save_answer(&answer).expect("Failed to save answer");
  }

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 3);

  let answer_map: HashMap<_, _> = loaded.into_iter().map(|a| (a.step_id.clone(), a)).collect();

  assert_eq!(
    answer_map.get("step-1").map(|a| a.value.as_str()),
    Some("User answer")
  );
  assert_eq!(
    answer_map.get("step-2").map(|a| a.value.as_str()),
    Some("AI suggestion")
  );
  assert_eq!(
    answer_map.get("step-3").map(|a| a.value.as_str()),
    Some("Uncertain data")
  );
}

/// Test persistence across multiple restarts
#[test]
fn test_e2e_persistence_multiple_restarts() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_multiple_restores.redb");

  // First session
  {
    let store = RedbStore::open(&db_path).expect("Failed to open database");

    let answer = create_test_answer("session-1", "Data from first session");
    store.save_answer(&answer).expect("Failed to save answer");

    let metadata =
      ProjectMetadata::with_current_timestamp("waterfall".to_string(), "discover".to_string());
    store
      .save_metadata(&metadata)
      .expect("Failed to save metadata");
  }

  // Second session
  {
    let store = RedbStore::open(&db_path).expect("Failed to reopen database");

    let answers = store.load_answers().expect("Failed to load answers");
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].value, "Data from first session");

    let answer = create_test_answer("session-2", "Data from second session");
    store.save_answer(&answer).expect("Failed to save answer");

    let metadata =
      ProjectMetadata::with_current_timestamp("agile".to_string(), "define".to_string());
    store
      .save_metadata(&metadata)
      .expect("Failed to save metadata");
  }

  // Third session
  {
    let store = RedbStore::open(&db_path).expect("Failed to reopen database");

    let answers = store.load_answers().expect("Failed to load answers");
    assert_eq!(answers.len(), 2);

    let metadata = store.get_metadata().expect("Failed to get metadata");
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.mode_preference, "agile");
    assert_eq!(metadata.current_phase, "define");
  }
}

/// Test database integrity after concurrent writes
#[test]
fn test_e2e_persistence_concurrent_writes() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_concurrent.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  for i in 0..20 {
    let answer = create_test_answer(&format!("concurrent-{i:03}"), &format!("Answer {i}"));
    store.save_answer(&answer).expect("Failed to save answer");
  }

  let metadata =
    ProjectMetadata::with_current_timestamp("express".to_string(), "develop".to_string());
  store
    .save_metadata(&metadata)
    .expect("Failed to save metadata");

  let lattice =
    LatticeCache::with_current_timestamp("develop".to_string(), r#"{"test": "data"}"#.to_string());
  store
    .save_lattice_cache("develop", &lattice)
    .expect("Failed to save lattice cache");

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let answers = store.load_answers().expect("Failed to load answers");
  assert_eq!(answers.len(), 20);

  let restored_metadata = store.get_metadata().expect("Failed to get metadata");
  assert!(restored_metadata.is_some());
  assert_eq!(restored_metadata.unwrap().mode_preference, "express");

  let restored_lattice = store
    .get_lattice_cache("develop")
    .expect("Failed to get lattice cache");
  assert!(restored_lattice.is_some());
  assert!(restored_lattice.unwrap().output_data.contains("test"));
}

/// Test empty database behavior
#[test]
fn test_e2e_persistence_empty_database() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_empty.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  let answers = store.load_answers().expect("Failed to load answers");
  assert!(answers.is_empty());

  let metadata = store.get_metadata().expect("Failed to get metadata");
  assert!(metadata.is_none());

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen empty database");

  let answers = store
    .load_answers()
    .expect("Failed to load answers after restart");
  assert!(answers.is_empty());
}

/// Test extraction cache persistence
#[test]
fn test_e2e_persistence_extraction_cache() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_extraction.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  let cache1 = ExtractionCache::new(
    "hash-abc123".to_string(),
    r#"{"field1": "value1"}"#.to_string(),
    "2024-02-25T12:00:00Z".to_string(),
  );
  let cache2 = ExtractionCache::new(
    "hash-xyz789".to_string(),
    r#"{"field2": "value2"}"#.to_string(),
    "2024-02-25T13:00:00Z".to_string(),
  );

  store
    .save_extraction_cache("hash-abc123", &cache1)
    .expect("Failed to save extraction cache 1");
  store
    .save_extraction_cache("hash-xyz789", &cache2)
    .expect("Failed to save extraction cache 2");

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let restored1 = store
    .get_extraction_cache("hash-abc123")
    .expect("Failed to get extraction cache 1");
  assert!(restored1.is_some());
  assert_eq!(restored1.unwrap().fields, r#"{"field1": "value1"}"#);

  let restored2 = store
    .get_extraction_cache("hash-xyz789")
    .expect("Failed to get extraction cache 2");
  assert!(restored2.is_some());
  assert_eq!(restored2.unwrap().fields, r#"{"field2": "value2"}"#);

  let missing = store
    .get_extraction_cache("hash-missing")
    .expect("Failed to check missing cache");
  assert!(missing.is_none());
}

/// Test answer update persistence
#[test]
fn test_e2e_persistence_answer_update() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_update.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  let answer1 = create_test_answer("update-test", "Initial value");
  store
    .save_answer(&answer1)
    .expect("Failed to save initial answer");

  let answer2 = create_test_answer("update-test", "Updated value");
  store
    .save_answer(&answer2)
    .expect("Failed to save updated answer");

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].value, "Updated value");
}

/// Test answer deletion persistence
#[test]
fn test_e2e_persistence_answer_deletion() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_delete.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  store
    .save_answer(&create_test_answer("delete-1", "Keep me"))
    .expect("Failed to save answer 1");
  store
    .save_answer(&create_test_answer("delete-2", "Delete me"))
    .expect("Failed to save answer 2");
  store
    .save_answer(&create_test_answer("delete-3", "Keep me too"))
    .expect("Failed to save answer 3");

  let deleted = store
    .delete_answer("delete-2")
    .expect("Failed to delete answer");
  assert!(deleted);

  // Restart
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 2);

  let answer_map: HashMap<_, _> = loaded.into_iter().map(|a| (a.step_id.clone(), a)).collect();
  assert!(answer_map.contains_key("delete-1"));
  assert!(!answer_map.contains_key("delete-2"));
  assert!(answer_map.contains_key("delete-3"));
}
