#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::result_large_err)]
#![allow(clippy::missing_const_for_fn)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Redb-based implementation of `TranscriptStore`.
//!
//! Provides ACID-compliant persistent storage for `InterrogationTranscript`
//! using the redb embedded database.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use redb::ReadableTable;

use crate::storage::{InterrogationTranscript, StorageError, TranscriptResult, TranscriptStore};

/// Table name for transcript storage
const TRANSCRIPTS_TABLE: &str = "transcripts";

/// Redb-based implementation of `TranscriptStore`.
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
  pub const fn from_db(db: Arc<redb::Database>) -> Self {
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
  /// - Serialization fails (unlikely with `serde_json`)
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

    let table = match txn.open_table(table_def) {
      Ok(t) => t,
      Err(e) if e.to_string().contains("does not exist") => return Ok(None),
      Err(e) => return Err(StorageError::from(e)),
    };

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

    let table = match txn.open_table(table_def) {
      Ok(t) => t,
      Err(e) if e.to_string().contains("does not exist") => return Ok(Vec::new()),
      Err(e) => return Err(StorageError::from(e)),
    };

    let iter = table.iter()?;
    let sessions: Vec<String> = iter
      .filter_map(Result::ok)
      .map(|(key, _value)| key.value().to_string())
      .collect();

    Ok(sessions)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::storage::{AntithesisResponse, ExtractedField, StrawManValidation};

  fn assert_approx_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9);
  }

  fn require_some<T>(value: Option<T>, context: &str) -> Result<T, StorageError> {
    value.ok_or_else(|| StorageError::Database(format!("missing value: {context}")))
  }

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
  async fn test_open_in_memory() -> Result<(), StorageError> {
    let _store = RedbTranscriptStore::open_in_memory()?;
    Ok(())
  }

  #[tokio::test]
  async fn test_save_and_load_transcript() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;
    let transcript = create_test_transcript("I want to build a fitness app");

    store.save("session-1", &transcript).await?;
    let loaded = store.load("session-1").await?;
    let loaded_transcript = require_some(loaded, "session-1")?;

    assert_eq!(
      loaded_transcript.original_prompt,
      "I want to build a fitness app"
    );
    assert_eq!(loaded_transcript.problem.content, "Test problem");
    assert_approx_eq(loaded_transcript.persona.confidence, 0.9);
    Ok(())
  }

  #[tokio::test]
  async fn test_load_nonexistent_transcript() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let loaded = store.load("nonexistent").await?;
    assert!(loaded.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_delete_transcript() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;
    let transcript = create_test_transcript("Test prompt");

    store.save("session-to-delete", &transcript).await?;
    let loaded = store.load("session-to-delete").await?;
    assert!(loaded.is_some());

    store.delete("session-to-delete").await?;

    let loaded = store.load("session-to-delete").await?;
    assert!(loaded.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_delete_nonexistent_is_ok() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;
    store.delete("never-existed").await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_list_sessions_empty() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let sessions = store.list_sessions().await?;
    assert!(sessions.is_empty());
    Ok(())
  }

  #[tokio::test]
  async fn test_list_sessions_with_data() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let t1 = create_test_transcript("Prompt 1");
    let t2 = create_test_transcript("Prompt 2");
    let t3 = create_test_transcript("Prompt 3");

    store.save("session-a", &t1).await?;
    store.save("session-b", &t2).await?;
    store.save("session-c", &t3).await?;

    let sessions = store.list_sessions().await?;

    assert_eq!(sessions.len(), 3);
    assert!(sessions.iter().any(|session| session == "session-a"));
    assert!(sessions.iter().any(|session| session == "session-b"));
    assert!(sessions.iter().any(|session| session == "session-c"));
    Ok(())
  }

  #[tokio::test]
  async fn test_overwrite_transcript() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let t1 = create_test_transcript("Original prompt");
    store.save("overwrite-test", &t1).await?;

    let t2 = create_test_transcript("Updated prompt");
    store.save("overwrite-test", &t2).await?;

    let loaded = require_some(store.load("overwrite-test").await?, "overwrite-test")?;
    assert_eq!(loaded.original_prompt, "Updated prompt");
    Ok(())
  }

  #[tokio::test]
  async fn test_persistence_across_operations() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

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

    store.save("complete-test", &transcript).await?;

    let loaded = require_some(store.load("complete-test").await?, "complete-test")?;

    assert_eq!(loaded.original_prompt, "Build a meditation app");
    assert_eq!(loaded.problem.content, "Users struggle with stress");
    assert_approx_eq(loaded.problem.confidence, 0.95);
    assert_eq!(loaded.persona.content, "Stressed professionals");
    assert_eq!(loaded.solution.content, "5-minute daily meditation");
    assert_eq!(loaded.nonpersona.content, "People who hate apps");
    assert_eq!(loaded.antithesis.points[0], "Maybe they don't need an app");
    assert_approx_eq(loaded.antithesis.quality_score, 0.75);
    assert!(!loaded.straw_man_validation.passed);
    assert_eq!(loaded.straw_man_validation.traps_detected.len(), 1);
    Ok(())
  }

  #[tokio::test]
  async fn test_completed_transcript() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let transcript = InterrogationTranscript::from_prompt("Test".to_string()).complete();

    store.save("completed-test", &transcript).await?;

    let loaded = require_some(store.load("completed-test").await?, "completed-test")?;

    assert!(loaded.is_completed());
    assert!(loaded.completed_at.is_some());
    Ok(())
  }

  #[tokio::test]
  async fn test_multiple_sessions_isolated() -> Result<(), StorageError> {
    let store = RedbTranscriptStore::open_in_memory()?;

    let t1 = create_test_transcript("Session 1 prompt");
    let t2 = create_test_transcript("Session 2 prompt");
    let t3 = create_test_transcript("Session 3 prompt");

    store.save("isolated-1", &t1).await?;
    store.save("isolated-2", &t2).await?;
    store.save("isolated-3", &t3).await?;

    store.delete("isolated-2").await?;

    let l1 = require_some(store.load("isolated-1").await?, "isolated-1")?;
    let l3 = require_some(store.load("isolated-3").await?, "isolated-3")?;

    assert_eq!(l1.original_prompt, "Session 1 prompt");
    assert_eq!(l3.original_prompt, "Session 3 prompt");

    let sessions = store.list_sessions().await?;
    assert_eq!(sessions.len(), 2);
    Ok(())
  }

  #[test]
  fn test_from_db() -> Result<(), redb::DatabaseError> {
    let db = Arc::new(
      redb::Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?,
    );

    let store1 = RedbTranscriptStore::from_db(db.clone());
    let store2 = RedbTranscriptStore::from_db(db);

    // Both stores share the same database
    assert!(Arc::ptr_eq(&store1.db, &store2.db));
    Ok(())
  }
}
