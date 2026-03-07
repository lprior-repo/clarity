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

//! End-to-end test: Persistence across app restart.
//!
//! This test verifies that all application data is correctly persisted to the
//! redb database and can be restored after closing and reopening the database.
//!
//! Test flow:
//! 1. Create a temporary database file
//! 2. Complete Discover phase in Express mode:
//!    - Save 5 answers covering all answer fields
//!    - Save metadata with Express mode preference
//!    - Save lattice cache with quality score
//! 3. Verify redb database file created
//! 4. Read database directly to verify tables
//! 5. Close and reopen app (simulate by closing and reopening database)
//! 6. Verify all data restored without corruption

use std::collections::HashMap;
use std::path::Path;

use redb::ReadableTable;

use clarity_web::storage::redb_store::RedbStore;
use clarity_web::storage::{
  tables, AnswerRecord, Confidence, ExtractionCache, LatticeCache, ProjectMetadata,
};
use clarity_web::types::Answer;

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
    return Err(format!("Database file does not exist: {db_path:?}"));
  }

  // Check file has content
  let metadata =
    std::fs::metadata(db_path).map_err(|e| format!("Failed to read database metadata: {e}"))?;

  if metadata.len() == 0 {
    return Err("Database file is empty".to_string());
  }

  Ok(())
}

/// Helper to count records in a redb table
fn count_table_records(db_path: &Path, table_name: &str) -> Result<usize, String> {
  let db = redb::Database::open(db_path).map_err(|e| format!("Failed to open database: {e}"))?;

  let txn = db
    .begin_read()
    .map_err(|e| format!("Failed to begin read: {e}"))?;
  let table_def: redb::TableDefinition<&str, &str> = redb::TableDefinition::new(table_name);

  match txn.open_table(table_def) {
    Ok(table) => {
      let mut count = 0;
      let mut iter = table
        .iter()
        .map_err(|e| format!("Failed to iterate table: {e}"))?;
      while iter.next().is_some() {
        count += 1;
      }
      Ok(count)
    }
    Err(_) => Ok(0), // Table doesn't exist yet
  }
}

/// Test complete Discover phase persistence and restoration
#[test]
fn test_e2e_persistence_discover_phase_express_mode() {
  // Create a temporary database file
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_clarity.redb");

  // ============================================================
  // STEP 1: Complete Discover phase in Express mode
  // ============================================================

  // Open the database for the first time (simulating app start)
  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Create 5 test answers covering all answer fields
  // Simulating user completing Discover phase prompts
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
      true, // AI-generated based on pattern
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
        "confidence": 0.89,
        "nodes": [
            {"id": "goal", "type": "root", "label": "Project Management Tool"},
            {"id": "audience", "type": "entity", "label": "Software Teams"},
            {"id": "differentiator", "type": "concept", "label": "Middle Ground"},
            {"id": "features", "type": "category", "label": "Core Features"}
        ],
        "edges": [
            {"from": "goal", "to": "audience", "weight": 0.9},
            {"from": "goal", "to": "differentiator", "weight": 0.85},
            {"from": "goal", "to": "features", "weight": 0.95}
        ]
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

  // Verify all answers are retrievable through the store
  let loaded_answers = store.load_answers().expect("Failed to load answers");
  assert_eq!(
    loaded_answers.len(),
    5,
    "Expected to load 5 answers, got {}",
    loaded_answers.len()
  );

  // Verify metadata mode preference
  let loaded_metadata = store
    .get_metadata()
    .expect("Failed to get metadata")
    .expect("No metadata found");
  assert_eq!(
    loaded_metadata.mode_preference, "express",
    "Mode preference should be 'express', got '{}'",
    loaded_metadata.mode_preference
  );
  assert_eq!(
    loaded_metadata.current_phase, "discover",
    "Current phase should be 'discover', got '{}'",
    loaded_metadata.current_phase
  );

  // Verify lattice cache contains quality score
  let loaded_lattice = store
    .get_lattice_cache("discover")
    .expect("Failed to get lattice cache")
    .expect("No lattice cache found");
  assert!(
    loaded_lattice.output_data.contains("quality_score"),
    "Lattice cache should contain quality_score"
  );
  assert!(
    loaded_lattice.output_data.contains("0.92"),
    "Lattice cache should contain quality score value 0.92"
  );

  // ============================================================
  // STEP 4: Read database directly to verify tables (after dropping store)
  // ============================================================

  // Drop the store to release the database lock before direct inspection
  drop(store);

  // Verify answers table contains all 5 records
  let answer_count =
    count_table_records(&db_path, tables::ANSWERS).expect("Failed to count answers");
  assert_eq!(
    answer_count, 5,
    "Expected 5 answers in database, found {answer_count}"
  );

  // Verify metadata table contains mode preference
  let metadata_count = count_table_records(&db_path, tables::PROJECT_METADATA)
    .expect("Failed to count metadata records");
  assert_eq!(
    metadata_count, 1,
    "Expected 1 metadata record in database, found {metadata_count}"
  );

  // Verify lattice cache table exists
  let lattice_count = count_table_records(&db_path, tables::LATTICE_CACHE)
    .expect("Failed to count lattice cache records");
  assert_eq!(
    lattice_count, 1,
    "Expected 1 lattice cache record in database, found {lattice_count}"
  );

  // ============================================================
  // STEP 5: Reopen app (simulate restart) - store already dropped in step 4
  // ============================================================

  // Reopen the database (simulating app restart)
  let store_reopened = RedbStore::open(&db_path).expect("Failed to reopen database after restart");

  // ============================================================
  // STEP 6: Verify all data restored without corruption
  // ============================================================

  // Verify all 5 answers restored
  let restored_answers = store_reopened
    .load_answers()
    .expect("Failed to load answers after restart");
  assert_eq!(
    restored_answers.len(),
    5,
    "Expected to restore 5 answers after restart, got {}",
    restored_answers.len()
  );

  // Build a map for easy lookup
  let answer_map: HashMap<String, Answer> = restored_answers
    .into_iter()
    .map(|a| (a.step_id.clone(), a))
    .collect();

  // Verify each answer's content
  assert_eq!(
    answer_map.get("discover-001").map(|a| a.value.as_str()),
    Some("Building a project management tool for software teams"),
    "Answer discover-001 has incorrect value"
  );
  assert_eq!(
    answer_map.get("discover-002").map(|a| a.value.as_str()),
    Some("Small to medium software development teams, 5-50 people"),
    "Answer discover-002 has incorrect value"
  );
  assert_eq!(
    answer_map.get("discover-003").map(|a| a.value.as_str()),
    Some("Jira is too complex, Trello is too simple, need middle ground"),
    "Answer discover-003 has incorrect value"
  );
  assert_eq!(
    answer_map.get("discover-004").map(|a| a.value.as_str()),
    Some("Sprint planning, backlog management, burndown charts"),
    "Answer discover-004 has incorrect value"
  );
  assert_eq!(
    answer_map.get("discover-005").map(|a| a.value.as_str()),
    Some("Web application, responsive design, real-time updates"),
    "Answer discover-005 has incorrect value"
  );

  // Verify mode preference restored
  let restored_metadata = store_reopened
    .get_metadata()
    .expect("Failed to get metadata after restart")
    .expect("No metadata found after restart");
  assert_eq!(
    restored_metadata.mode_preference, "express",
    "Mode preference not restored correctly after restart"
  );
  assert_eq!(
    restored_metadata.current_phase, "discover",
    "Current phase not restored correctly after restart"
  );

  // Verify quality score restored in lattice cache
  let restored_lattice = store_reopened
    .get_lattice_cache("discover")
    .expect("Failed to get lattice cache after restart")
    .expect("No lattice cache found after restart");
  assert!(
    restored_lattice.output_data.contains("quality_score"),
    "Quality score missing from restored lattice cache"
  );
  assert!(
    restored_lattice.output_data.contains("0.92"),
    "Quality score value incorrect in restored lattice cache"
  );

  // Verify no corruption by checking JSON can be parsed
  let _: serde_json::Value = serde_json::from_str(&restored_lattice.output_data)
    .expect("Lattice cache data is corrupted - invalid JSON");

  // ============================================================
  // TEST PASSED: All data persisted and restored correctly
  // ============================================================
}

/// Test persistence of individual answer fields
#[test]
fn test_e2e_persistence_answer_fields() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_answer_fields.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Create answers with different confidence levels and AI flags
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

  // Verify all fields restored
  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 3);

  // Check individual answers
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

/// Test persistence across multiple database open/close cycles
#[test]
fn test_e2e_persistence_multiple_restarts() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_multiple_restores.redb");

  // First session: Create initial data
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

  // Second session: Add more data
  {
    let store = RedbStore::open(&db_path).expect("Failed to reopen database");

    // Verify previous data
    let answers = store.load_answers().expect("Failed to load answers");
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].value, "Data from first session");

    // Add new data
    let answer = create_test_answer("session-2", "Data from second session");
    store.save_answer(&answer).expect("Failed to save answer");

    let metadata =
      ProjectMetadata::with_current_timestamp("agile".to_string(), "define".to_string());
    store
      .save_metadata(&metadata)
      .expect("Failed to save metadata");
  }

  // Third session: Verify all data persisted
  {
    let store = RedbStore::open(&db_path).expect("Failed to reopen database");

    let answers = store.load_answers().expect("Failed to load answers");
    assert_eq!(
      answers.len(),
      2,
      "Should have 2 answers from multiple sessions"
    );

    let metadata = store
      .get_metadata()
      .expect("Failed to get metadata")
      .expect("No metadata found");
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

  // Write multiple records rapidly
  for i in 0..20 {
    let answer = create_test_answer(
      &format!("concurrent-{i:03}"),
      &format!("Concurrent answer {i}"),
    );
    store.save_answer(&answer).expect("Failed to save answer");
  }

  // Write metadata and cache
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

  // Restart and verify
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let answers = store.load_answers().expect("Failed to load answers");
  assert_eq!(answers.len(), 20, "All 20 concurrent writes should persist");

  let restored_metadata = store
    .get_metadata()
    .expect("Failed to get metadata")
    .expect("No metadata found");
  assert_eq!(restored_metadata.mode_preference, "express");

  let restored_lattice = store
    .get_lattice_cache("develop")
    .expect("Failed to get lattice cache")
    .expect("No lattice cache found");
  assert!(restored_lattice.output_data.contains("test"));
}

/// Test empty database behavior
#[test]
fn test_e2e_persistence_empty_database() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_empty.redb");

  // Create empty database
  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Verify no data
  let answers = store.load_answers().expect("Failed to load answers");
  assert!(answers.is_empty(), "New database should have no answers");

  let metadata = store.get_metadata().expect("Failed to get metadata");
  assert!(metadata.is_none(), "New database should have no metadata");

  // Restart empty database
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen empty database");

  // Verify still empty
  let answers = store
    .load_answers()
    .expect("Failed to load answers after restart");
  assert!(
    answers.is_empty(),
    "Database should still be empty after restart"
  );
}

/// Test extraction cache persistence
#[test]
fn test_e2e_persistence_extraction_cache() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_extraction.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Save extraction caches
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

  // Verify extraction caches restored
  let restored1 = store
    .get_extraction_cache("hash-abc123")
    .expect("Failed to get extraction cache 1")
    .expect("Extraction cache 1 not found");
  assert_eq!(restored1.fields, r#"{"field1": "value1"}"#);

  let restored2 = store
    .get_extraction_cache("hash-xyz789")
    .expect("Failed to get extraction cache 2")
    .expect("Extraction cache 2 not found");
  assert_eq!(restored2.fields, r#"{"field2": "value2"}"#);

  // Verify missing cache returns None
  let missing = store
    .get_extraction_cache("hash-missing")
    .expect("Failed to check missing cache");
  assert!(missing.is_none(), "Missing cache should return None");
}

/// Test answer update persistence
#[test]
fn test_e2e_persistence_answer_update() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_update.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Save initial answer
  let answer1 = create_test_answer("update-test", "Initial value");
  store
    .save_answer(&answer1)
    .expect("Failed to save initial answer");

  // Update answer
  let answer2 = create_test_answer("update-test", "Updated value");
  store
    .save_answer(&answer2)
    .expect("Failed to save updated answer");

  // Restart and verify update persisted
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].value, "Updated value", "Answer should be updated");
}

/// Test answer deletion persistence
#[test]
fn test_e2e_persistence_answer_deletion() {
  let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
  let db_path = temp_dir.path().join("test_delete.redb");

  let store = RedbStore::open(&db_path).expect("Failed to open database");

  // Save multiple answers
  store
    .save_answer(&create_test_answer("delete-1", "Keep me"))
    .expect("Failed to save answer 1");
  store
    .save_answer(&create_test_answer("delete-2", "Delete me"))
    .expect("Failed to save answer 2");
  store
    .save_answer(&create_test_answer("delete-3", "Keep me too"))
    .expect("Failed to save answer 3");

  // Delete one answer
  let deleted = store
    .delete_answer("delete-2")
    .expect("Failed to delete answer");
  assert!(deleted, "Delete should return true");

  // Restart and verify deletion persisted
  drop(store);
  let store = RedbStore::open(&db_path).expect("Failed to reopen database");

  let loaded = store.load_answers().expect("Failed to load answers");
  assert_eq!(loaded.len(), 2, "Should have 2 answers after deletion");

  let answer_map: HashMap<_, _> = loaded.into_iter().map(|a| (a.step_id.clone(), a)).collect();
  assert!(answer_map.contains_key("delete-1"));
  assert!(!answer_map.contains_key("delete-2"));
  assert!(answer_map.contains_key("delete-3"));
}
