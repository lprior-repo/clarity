#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

//! Adversarial Generation 2 Testing - Storage Corruption & Race Conditions
//!
//! Tests concurrent operations, corruption scenarios, and recovery

use clarity_web::storage::types::*;
use clarity_web::storage::RedbStore;
use clarity_web::types::Answer;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Helper to create test answers
fn create_answer(step_id: &str, value: &str) -> Answer {
  Answer {
    step_id: step_id.to_string(),
    value: value.to_string(),
    timestamp: "2024-01-01T00:00:00Z".to_string(),
  }
}

#[test]
fn test_storage_concurrent_write_same_key() {
  // BUG TEST: Multiple threads writing to same key
  // Last write should win (no corruption)
  let store = Arc::new(RedbStore::open_in_memory().expect("Failed to open store"));

  let answer1 = create_answer("key1", "value1");
  let answer2 = create_answer("key1", "value2");

  let store1 = Arc::clone(&store);
  let store2 = Arc::clone(&store);

  let handle1 = thread::spawn(move || {
    thread::sleep(Duration::from_millis(10));
    let _ = store1.save_answer(&answer1);
  });

  let handle2 = thread::spawn(move || {
    let _ = store2.save_answer(&answer2);
  });

  handle1.join().expect("Thread 1 panicked");
  handle2.join().expect("Thread 2 panicked");

  // Give transactions time to commit
  thread::sleep(Duration::from_millis(50));

  // One of the values should persist
  let result = store.get_answer("key1");
  assert!(result.is_ok());

  match result.unwrap() {
    Some(answer) => {
      // Should be one of the two values
      assert!(answer.value == "value1" || answer.value == "value2");
    }
    None => {
      panic!("Answer should exist after concurrent writes");
    }
  }
}

#[test]
fn test_storage_concurrent_read_write() {
  // BUG TEST: Concurrent reads and writes
  let store = Arc::new(RedbStore::open_in_memory().expect("Failed to open store"));

  // Write initial data
  for i in 0..10 {
    let answer = create_answer(&format!("key{i}"), &format!("value{i}"));
    store
      .save_answer(&answer)
      .expect("Failed to save initial answer");
  }

  let store_read = Arc::clone(&store);
  let store_write = Arc::clone(&store);

  // Spawn read thread
  let handle_read = thread::spawn(move || {
    for _ in 0..100 {
      let _ = store_read.get_all_answers();
      thread::sleep(Duration::from_millis(1));
    }
  });

  // Spawn write thread
  let handle_write = thread::spawn(move || {
    for i in 10..20 {
      let answer = create_answer(&format!("key{i}"), &format!("value{i}"));
      let _ = store_write.save_answer(&answer);
      thread::sleep(Duration::from_millis(2));
    }
  });

  handle_read.join().expect("Read thread panicked");
  handle_write.join().expect("Write thread panicked");

  // Verify no corruption
  let all_answers = store.get_all_answers().expect("Failed to get all answers");
  assert!(
    all_answers.len() >= 10,
    "Should have at least initial answers"
  );
  assert!(all_answers.len() <= 20, "Should not exceed total writes");
}

#[test]
fn test_storage_serialized_corruption_recovery() {
  // BUG TEST: Corrupted JSON data should not crash
  // Note: We can't directly insert corrupted data since db is private
  // This test documents the expectation for corruption handling

  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Insert valid data
  let valid = create_answer("valid", "valid value");
  store.save_answer(&valid).expect("Failed to save valid");

  // Valid read should work
  let valid_result = store.get_answer("valid");
  assert!(valid_result.is_ok());
  assert!(valid_result.unwrap().is_some());

  // Document: If corrupted data existed, it should:
  // 1. Return Deserialization error for that key
  // 2. Not panic
  // 3. Allow other keys to be read

  // get_all_answers demonstrates graceful handling
  let all_result = store.get_all_answers();
  assert!(all_result.is_ok());
  assert!(!all_result.unwrap().is_empty());
}

#[test]
fn test_storage_empty_value_handling() {
  // Edge case: Empty string values
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let empty_answer = create_answer("empty", "");
  store
    .save_answer(&empty_answer)
    .expect("Failed to save empty answer");

  let result = store.get_answer("empty");
  assert!(result.is_ok());

  match result.unwrap() {
    Some(answer) => {
      assert_eq!(answer.value, "");
    }
    None => {
      panic!("Empty answer should be stored");
    }
  }
}

#[test]
fn test_storage_special_characters_in_values() {
  // Edge case: Special characters that might break serialization
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let special_cases = vec![
    (
      "quotes",
      create_answer("quotes", "Text with \"quotes\" and 'apostrophes'"),
    ),
    (
      "newlines",
      create_answer("newlines", "Line 1\nLine 2\rLine 3"),
    ),
    (
      "unicode",
      create_answer("unicode", "Unicode: 🎉 中文 العربية"),
    ),
    ("null", create_answer("null", "Text with \0 null byte")),
    (
      "backslash",
      create_answer("backslash", "Path: C:\\Users\\Test"),
    ),
  ];

  for (key, answer) in &special_cases {
    store
      .save_answer(answer)
      .unwrap_or_else(|_| panic!("Failed to save {key}"));
  }

  for (key, original) in &special_cases {
    let result = store.get_answer(key);
    assert!(result.is_ok(), "Should read {key}");

    match result.unwrap() {
      Some(retrieved) => {
        assert_eq!(retrieved.step_id, original.step_id);
        assert_eq!(retrieved.value, original.value);
      }
      None => {
        panic!("Should retrieve {key}");
      }
    }
  }
}

#[test]
fn test_storage_overwrite_metadata() {
  // Test that metadata overwrite works correctly
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let metadata1 = ProjectMetadata::new(
    "waterfall".to_string(),
    "discover".to_string(),
    "2024-01-01T00:00:00Z".to_string(),
    "2024-01-01T01:00:00Z".to_string(),
  );

  let metadata2 = ProjectMetadata::new(
    "agile".to_string(),
    "define".to_string(),
    "2024-01-02T00:00:00Z".to_string(),
    "2024-01-02T01:00:00Z".to_string(),
  );

  store
    .save_metadata(&metadata1)
    .expect("Failed to save first metadata");
  store
    .save_metadata(&metadata2)
    .expect("Failed to save second metadata");

  let result = store.get_metadata().expect("Failed to get metadata");

  match result {
    Some(metadata) => {
      // Should have second metadata (last write wins)
      assert_eq!(metadata.mode_preference, "agile");
      assert_eq!(metadata.current_phase, "define");
    }
    None => {
      panic!("Metadata should exist");
    }
  }
}

#[test]
fn test_storage_delete_nonexistent() {
  // Edge case: Deleting a key that doesn't exist
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let result = store.delete_answer("nonexistent");
  assert!(result.is_ok());
  assert!(!result.unwrap()); // Should return false (not deleted)
}

#[test]
fn test_storage_cache_key_collisions() {
  // Edge case: Different hash values (if hash function has collisions)
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let cache1 = ExtractionCache::new(
    "hash1".to_string(),
    "{\"data\":\"value1\"}".to_string(),
    "2024-01-01T00:00:00Z".to_string(),
  );

  let cache2 = ExtractionCache::new(
    "hash2".to_string(),
    "{\"data\":\"value2\"}".to_string(),
    "2024-01-01T02:00:00Z".to_string(),
  );

  store
    .save_extraction_cache("hash1", &cache1)
    .expect("Failed to save cache1");
  store
    .save_extraction_cache("hash2", &cache2)
    .expect("Failed to save cache2");

  // Both should be retrievable
  let result1 = store.get_extraction_cache("hash1");
  let result2 = store.get_extraction_cache("hash2");

  assert!(result1.is_ok());
  assert!(result2.is_ok());

  match (result1.unwrap(), result2.unwrap()) {
    (Some(c1), Some(c2)) => {
      assert_ne!(c1.fields, c2.fields);
    }
    _ => {
      panic!("Both caches should exist");
    }
  }
}

#[test]
fn test_storage_lattice_cache_overwrite() {
  // Test overwriting lattice cache for same phase
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let cache1 = LatticeCache::new(
    "discover".to_string(),
    "{\"version\":1}".to_string(),
    "2024-01-01T00:00:00Z".to_string(),
  );

  let cache2 = LatticeCache::new(
    "discover".to_string(),
    "{\"version\":2}".to_string(),
    "2024-01-01T02:00:00Z".to_string(),
  );

  store
    .save_lattice_cache("discover", &cache1)
    .expect("Failed to save cache1");
  store
    .save_lattice_cache("discover", &cache2)
    .expect("Failed to save cache2");

  let result = store.get_lattice_cache("discover");
  assert!(result.is_ok());

  match result.unwrap() {
    Some(cache) => {
      // Should have version 2 (last write wins)
      assert!(cache.output_data.contains("\"version\":2"));
    }
    None => {
      panic!("Cache should exist");
    }
  }
}

#[test]
fn test_storage_very_long_values() {
  // Edge case: Very long string values
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let long_value = "A".repeat(1_000_000); // 1MB of 'A's
  let answer = create_answer("long", &long_value);

  let save_result = store.save_answer(&answer);
  assert!(save_result.is_ok(), "Should save very long value");

  let load_result = store.get_answer("long");
  assert!(load_result.is_ok());

  match load_result.unwrap() {
    Some(retrieved) => {
      assert_eq!(retrieved.value.len(), 1_000_000);
    }
    None => {
      panic!("Should retrieve long value");
    }
  }
}

#[test]
fn test_storage_unicode_keys() {
  // Edge case: Unicode in step_id (keys)
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let answer = create_answer("用户目标", "User goal in Chinese");
  let save_result = store.save_answer(&answer);

  // May fail depending on redb's unicode support
  if let Ok(()) = save_result {
    let load_result = store.get_answer("用户目标");
    assert!(load_result.is_ok());
    assert!(load_result.unwrap().is_some());
  } else {
    // Acceptable if unicode keys aren't supported
  }
}

#[test]
fn test_storage_null_byte_in_key() {
  // BUG TEST: Null byte in key
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let answer = create_answer("key\0with\0null", "value");
  let save_result = store.save_answer(&answer);

  // Should handle gracefully (likely fail or sanitize)
  if let Ok(()) = save_result {
    // If saved, should be retrievable
    let load_result = store.get_answer("key\0with\0null");
    assert!(load_result.is_ok());
  } else {
    // Acceptable to reject null bytes
  }
}

#[test]
fn test_storage_rapid_fire_operations() {
  // Stress test: Rapid successive operations
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  for i in 0..1000 {
    let answer = create_answer(&format!("key{i}"), &format!("value{i}"));
    store
      .save_answer(&answer)
      .unwrap_or_else(|_| panic!("Failed to save iteration {i}"));
  }

  let all_answers = store.get_all_answers().expect("Failed to get all answers");

  assert_eq!(all_answers.len(), 1000);

  // Verify all values
  for i in 0..1000 {
    let key = format!("key{i}");
    let result = store.get_answer(&key);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some(), "Key {i} should exist");
  }
}

#[test]
fn test_storage_transaction_isolation() {
  // BUG TEST: Verify transaction isolation
  // Note: Can't directly access transactions due to private db field
  // This test documents the expectation for isolation

  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let answer1 = create_answer("key1", "value1");
  let answer2 = create_answer("key2", "value2");

  store.save_answer(&answer1).expect("Failed to save answer1");

  // Read first value
  let result1 = store.get_answer("key1");
  assert!(result1.is_ok());
  assert!(result1.unwrap().is_some());

  // Write second value
  store.save_answer(&answer2).expect("Failed to save answer2");

  // Both should be visible
  let result2 = store.get_answer("key2");
  assert!(result2.is_ok());
  assert!(result2.unwrap().is_some());

  let all = store.get_all_answers().expect("Failed to get all");
  assert_eq!(all.len(), 2);
}

#[test]
fn test_storage_empty_database_reads() {
  // Edge case: Reading from empty database
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  let metadata = store.get_metadata();
  assert!(metadata.is_ok());
  assert!(metadata.unwrap().is_none());

  let answer = store.get_answer("nonexistent");
  assert!(answer.is_ok());
  assert!(answer.unwrap().is_none());

  let all_answers = store.get_all_answers();
  assert!(all_answers.is_ok());
  assert!(all_answers.unwrap().is_empty());

  let cache = store.get_extraction_cache("hash");
  assert!(cache.is_ok());
  assert!(cache.unwrap().is_none());

  let lattice = store.get_lattice_cache("phase");
  assert!(lattice.is_ok());
  assert!(lattice.unwrap().is_none());
}
