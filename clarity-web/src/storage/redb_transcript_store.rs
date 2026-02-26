#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Redb-based implementation of TranscriptStore.
//!
//! Provides ACID-compliant persistent storage for InterrogationTranscript
//! using the redb embedded database.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use redb::ReadableTable;

use crate::storage::{InterrogationTranscript, StorageError, TranscriptResult, TranscriptStore};

/// Table name for transcript storage
const TRANSCRIPTS_TABLE: &str = "transcripts";

/// Redb-based implementation of TranscriptStore.
///
/// Provides persistent storage with ACID guarantees:
/// - **Atomicity**: Each save/delete is a single transaction
/// - **Consistency**: JSON serialization ensures valid data
/// - **Isolation**: redb handles concurrent access
/// - **Durability**: Data is written to disk on commit
#[derive(Debug)]
pub struct RedbTranscriptStore {
  db: Arc<redb::Database>,
}

impl RedbTranscriptStore {
  /// Open or create a transcript store at the given path.
  ///
  /// # Errors
  ///
  /// Returns `redb::Error` if the database cannot be created or opened.
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, redb::Error> {
    let db = redb::Database::create(path.as_ref())?;
    Ok(Self { db: Arc::new(db) })
  }

  /// Create an in-memory transcript store for testing.
  ///
  /// # Errors
  ///
  /// Returns `redb::Error` if the in-memory database cannot be created.
  pub fn open_in_memory() -> Result<Self, redb::Error> {
    let db =
      redb::Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?;
    Ok(Self { db: Arc::new(db) })
  }

  /// Create a store from an existing database.
  ///
  /// This is useful when sharing a database with other stores.
  #[must_use]
  pub fn from_db(db: Arc<redb::Database>) -> Self {
    Self { db }
  }

  /// Get a reference to the underlying database.
  #[must_use]
  pub fn db(&self) -> &redb::Database {
    &self.db
  }
}

#[async_trait]
impl TranscriptStore for RedbTranscriptStore {
  /// Save a transcript with the given session ID.
  ///
  /// The operation is atomic - either the full transcript is saved
  /// or no changes are made to the database.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if:
  /// - Serialization fails (unlikely with serde_json)
  /// - Database write fails
  /// - Transaction cannot be committed
  async fn save(
    &self,
    session_id: &str,
    transcript: &InterrogationTranscript,
  ) -> TranscriptResult<()> {
    // Serialize outside transaction to minimize lock time
    let json = serde_json::to_string(transcript).map_err(StorageError::serialization)?;

    let txn = self.db.begin_write()?;

    {
      let table_def: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(TRANSCRIPTS_TABLE);
      let mut table = txn.open_table(table_def)?;
      table.insert(session_id, json.as_str())?;
    }

    txn.commit()?;

    Ok(())
  }

  /// Load a transcript by session ID.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if:
  /// - Deserialization fails (corrupted data)
  /// - Database read fails
  async fn load(&self, session_id: &str) -> TranscriptResult<Option<InterrogationTranscript>> {
    let txn = self.db.begin_read()?;

    let table_def: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(TRANSCRIPTS_TABLE);

    let table = txn.open_table(table_def)?;

    match table.get(session_id)? {
      Some(guard) => {
        let json: &str = guard.value();
        let transcript = serde_json::from_str(json).map_err(StorageError::deserialization)?;
        Ok(Some(transcript))
      }
      None => Ok(None),
    }
  }

  /// Delete a transcript by session ID.
  ///
  /// This operation is idempotent - deleting a non-existent transcript
  /// returns `Ok(())`.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if the database operation fails.
  async fn delete(&self, session_id: &str) -> TranscriptResult<()> {
    let txn = self.db.begin_write()?;

    {
      let table_def: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(TRANSCRIPTS_TABLE);
      let mut table = txn.open_table(table_def)?;
      table.remove(session_id)?;
    }

    txn.commit()?;

    Ok(())
  }

  /// List all session IDs in the store.
  ///
  /// Sessions are returned in no particular order.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if the database operation fails.
  async fn list_sessions(&self) -> TranscriptResult<Vec<String>> {
    let txn = self.db.begin_read()?;

    let table_def: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(TRANSCRIPTS_TABLE);

    let table = txn.open_table(table_def)?;

    let iter = table.iter()?;
    let sessions: Vec<String> = iter
      .filter_map(|result| result.ok())
      .map(|(key, _value)| key.value().to_string())
      .collect();

    Ok(sessions)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::storage::{AntithesisResponse, ExtractedField, StrawManValidation};

  /// Create a test transcript with given content.
  fn create_test_transcript(prompt: &str) -> InterrogationTranscript {
    let mut transcript = InterrogationTranscript::from_prompt(prompt.to_string());
    transcript.problem = ExtractedField::new("Test problem".to_string(), 0.8, "ai".to_string());
    transcript.persona = ExtractedField::new("Test user".to_string(), 0.9, "ai".to_string());
    transcript.antithesis = AntithesisResponse::new(
      "Counter 1".to_string(),
      "Counter 2".to_string(),
      "Counter 3".to_string(),
      0.7,
    );
    transcript.straw_man_validation = StrawManValidation::passed();
    transcript
  }

  #[tokio::test]
  async fn test_open_in_memory() {
    let result = RedbTranscriptStore::open_in_memory();
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_save_and_load_transcript() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();
    let transcript = create_test_transcript("I want to build a fitness app");

    // Save
    let save_result = store.save("session-1", &transcript).await;
    assert!(save_result.is_ok());

    // Load
    let load_result = store.load("session-1").await;
    assert!(load_result.is_ok());

    let loaded = load_result.unwrap();
    assert!(loaded.is_some());

    let loaded_transcript = loaded.unwrap();
    assert_eq!(
      loaded_transcript.original_prompt,
      "I want to build a fitness app"
    );
    assert_eq!(loaded_transcript.problem.content, "Test problem");
    assert_eq!(loaded_transcript.persona.confidence, 0.9);
  }

  #[tokio::test]
  async fn test_load_nonexistent_transcript() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let result = store.load("nonexistent").await;
    assert!(result.is_ok());

    let loaded = result.unwrap();
    assert!(loaded.is_none());
  }

  #[tokio::test]
  async fn test_delete_transcript() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();
    let transcript = create_test_transcript("Test prompt");

    // Save
    store.save("session-to-delete", &transcript).await.unwrap();

    // Verify exists
    let loaded = store.load("session-to-delete").await.unwrap();
    assert!(loaded.is_some());

    // Delete
    let delete_result = store.delete("session-to-delete").await;
    assert!(delete_result.is_ok());

    // Verify deleted
    let loaded = store.load("session-to-delete").await.unwrap();
    assert!(loaded.is_none());
  }

  #[tokio::test]
  async fn test_delete_nonexistent_is_ok() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    // Deleting non-existent should be Ok
    let result = store.delete("never-existed").await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_list_sessions_empty() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let sessions = store.list_sessions().await.unwrap();
    assert!(sessions.is_empty());
  }

  #[tokio::test]
  async fn test_list_sessions_with_data() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let t1 = create_test_transcript("Prompt 1");
    let t2 = create_test_transcript("Prompt 2");
    let t3 = create_test_transcript("Prompt 3");

    store.save("session-a", &t1).await.unwrap();
    store.save("session-b", &t2).await.unwrap();
    store.save("session-c", &t3).await.unwrap();

    let sessions = store.list_sessions().await.unwrap();

    assert_eq!(sessions.len(), 3);
    assert!(sessions.contains(&"session-a".to_string()));
    assert!(sessions.contains(&"session-b".to_string()));
    assert!(sessions.contains(&"session-c".to_string()));
  }

  #[tokio::test]
  async fn test_overwrite_transcript() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let t1 = create_test_transcript("Original prompt");
    store.save("overwrite-test", &t1).await.unwrap();

    let t2 = create_test_transcript("Updated prompt");
    store.save("overwrite-test", &t2).await.unwrap();

    let loaded = store.load("overwrite-test").await.unwrap().unwrap();
    assert_eq!(loaded.original_prompt, "Updated prompt");
  }

  #[tokio::test]
  async fn test_persistence_across_operations() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    // Create and save a complete transcript
    let mut transcript = InterrogationTranscript::from_prompt("Build a meditation app".to_string());
    transcript.problem = ExtractedField::new(
      "Users struggle with stress".to_string(),
      0.95,
      "ai".to_string(),
    );
    transcript.persona =
      ExtractedField::new("Stressed professionals".to_string(), 0.9, "ai".to_string());
    transcript.solution = ExtractedField::new(
      "5-minute daily meditation".to_string(),
      0.85,
      "ai".to_string(),
    );
    transcript.nonpersona =
      ExtractedField::new("People who hate apps".to_string(), 0.7, "ai".to_string());
    transcript.antithesis = AntithesisResponse::new(
      "Maybe they don't need an app".to_string(),
      "Existing apps are good enough".to_string(),
      "Users won't stick to it".to_string(),
      0.75,
    );
    transcript.straw_man_validation =
      StrawManValidation::new(vec![crate::storage::StrawManTrap::ManicPixieDreamUser]);

    store.save("complete-test", &transcript).await.unwrap();

    // Load and verify all fields
    let loaded = store.load("complete-test").await.unwrap().unwrap();

    assert_eq!(loaded.original_prompt, "Build a meditation app");
    assert_eq!(loaded.problem.content, "Users struggle with stress");
    assert_eq!(loaded.problem.confidence, 0.95);
    assert_eq!(loaded.persona.content, "Stressed professionals");
    assert_eq!(loaded.solution.content, "5-minute daily meditation");
    assert_eq!(loaded.nonpersona.content, "People who hate apps");
    assert_eq!(loaded.antithesis.points[0], "Maybe they don't need an app");
    assert_eq!(loaded.antithesis.quality_score, 0.75);
    assert!(!loaded.straw_man_validation.passed);
    assert_eq!(loaded.straw_man_validation.traps_detected.len(), 1);
  }

  #[tokio::test]
  async fn test_completed_transcript() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let transcript = InterrogationTranscript::from_prompt("Test".to_string()).complete();

    store.save("completed-test", &transcript).await.unwrap();

    let loaded = store.load("completed-test").await.unwrap().unwrap();

    assert!(loaded.is_completed());
    assert!(loaded.completed_at.is_some());
  }

  #[tokio::test]
  async fn test_multiple_sessions_isolated() {
    let store = RedbTranscriptStore::open_in_memory().unwrap();

    let t1 = create_test_transcript("Session 1 prompt");
    let t2 = create_test_transcript("Session 2 prompt");
    let t3 = create_test_transcript("Session 3 prompt");

    store.save("isolated-1", &t1).await.unwrap();
    store.save("isolated-2", &t2).await.unwrap();
    store.save("isolated-3", &t3).await.unwrap();

    // Delete one
    store.delete("isolated-2").await.unwrap();

    // Verify others are intact
    let l1 = store.load("isolated-1").await.unwrap().unwrap();
    let l3 = store.load("isolated-3").await.unwrap().unwrap();

    assert_eq!(l1.original_prompt, "Session 1 prompt");
    assert_eq!(l3.original_prompt, "Session 3 prompt");

    let sessions = store.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 2);
  }

  #[test]
  fn test_from_db() {
    let db = Arc::new(
      redb::Database::builder()
        .create_with_backend(redb::backends::InMemoryBackend::new())
        .unwrap(),
    );

    let store1 = RedbTranscriptStore::from_db(db.clone());
    let store2 = RedbTranscriptStore::from_db(db);

    // Both stores share the same database
    assert!(Arc::ptr_eq(&store1.db, &store2.db));
  }
}
