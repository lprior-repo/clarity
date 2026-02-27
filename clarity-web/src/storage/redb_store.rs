#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::result_large_err)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::while_let_on_iterator)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![cfg(not(target_arch = "wasm32"))]

use redb::ReadableTable;
use std::path::Path;

use crate::storage::{
  tables, AnswerRecord, ExtractionCache, LatticeCache, ProjectMetadata, StorageError,
};
use crate::types::Answer;

pub type StoreResult<T> = Result<T, StorageError>;

#[derive(Debug)]
pub struct RedbStore {
  db: redb::Database,
}

impl RedbStore {
  /// Open (or create) a redb store at the provided path.
  ///
  /// # Errors
  ///
  /// Returns any `redb` error encountered while opening/creating the database.
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, redb::Error> {
    let db = redb::Database::create(path.as_ref())?;
    Ok(Self { db })
  }

  /// Open an in-memory redb store for tests or ephemeral data.
  ///
  /// # Errors
  ///
  /// Returns any `redb` error encountered while creating the in-memory database.
  pub fn open_in_memory() -> Result<Self, redb::Error> {
    let db =
      redb::Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?;
    Ok(Self { db })
  }

  #[allow(clippy::unused_self)]
  const fn ensure_tables(&self) -> StoreResult<()> {
    Ok(())
  }

  pub fn save_metadata(&self, metadata: &ProjectMetadata) -> StoreResult<()> {
    self.ensure_tables()?;
    let txn = self.db.begin_write()?;
    {
      let table_definition: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(tables::PROJECT_METADATA);
      let mut table = txn.open_table(table_definition)?;
      let json = serde_json::to_string(metadata).map_err(StorageError::serialization)?;
      table.insert("metadata", json.as_str())?;
    }
    txn.commit()?;
    Ok(())
  }

  pub fn get_metadata(&self) -> StoreResult<Option<ProjectMetadata>> {
    self.ensure_tables()?;
    let txn = self.db.begin_read()?;
    let table_definition: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(tables::PROJECT_METADATA);
    // Table may not exist yet if no metadata has been saved
    let table = match txn.open_table(table_definition) {
      Ok(t) => t,
      Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
      Err(e) => return Err(e.into()),
    };
    match table.get("metadata")? {
      Some(guard) => {
        let json: &str = guard.value();
        let metadata = serde_json::from_str(json).map_err(StorageError::deserialization)?;
        Ok(Some(metadata))
      }
      None => Ok(None),
    }
  }

  pub fn save_extraction_cache(&self, hash: &str, cache: &ExtractionCache) -> StoreResult<()> {
    self.ensure_tables()?;
    let txn = self.db.begin_write()?;
    {
      let table_definition: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(tables::EXTRACTIONS);
      let mut table = txn.open_table(table_definition)?;
      let json = serde_json::to_string(cache).map_err(StorageError::serialization)?;
      table.insert(hash, json.as_str())?;
    }
    txn.commit()?;
    Ok(())
  }

  pub fn get_extraction_cache(&self, hash: &str) -> StoreResult<Option<ExtractionCache>> {
    self.ensure_tables()?;
    let txn = self.db.begin_read()?;
    let table_definition: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(tables::EXTRACTIONS);
    // Table may not exist yet if no cache has been saved
    let table = match txn.open_table(table_definition) {
      Ok(t) => t,
      Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
      Err(e) => return Err(e.into()),
    };
    match table.get(hash)? {
      Some(guard) => {
        let json: &str = guard.value();
        let cache = serde_json::from_str(json).map_err(StorageError::deserialization)?;
        Ok(Some(cache))
      }
      None => Ok(None),
    }
  }

  pub fn save_lattice_cache(&self, phase: &str, cache: &LatticeCache) -> StoreResult<()> {
    self.ensure_tables()?;
    let txn = self.db.begin_write()?;
    {
      let table_definition: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(tables::LATTICE_CACHE);
      let mut table = txn.open_table(table_definition)?;
      let json = serde_json::to_string(cache).map_err(StorageError::serialization)?;
      table.insert(phase, json.as_str())?;
    }
    txn.commit()?;
    Ok(())
  }

  pub fn get_lattice_cache(&self, phase: &str) -> StoreResult<Option<LatticeCache>> {
    self.ensure_tables()?;
    let txn = self.db.begin_read()?;
    let table_definition: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(tables::LATTICE_CACHE);
    // Table may not exist yet if no cache has been saved
    let table = match txn.open_table(table_definition) {
      Ok(t) => t,
      Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
      Err(e) => return Err(e.into()),
    };
    match table.get(phase)? {
      Some(guard) => {
        let json: &str = guard.value();
        let cache = serde_json::from_str(json).map_err(StorageError::deserialization)?;
        Ok(Some(cache))
      }
      None => Ok(None),
    }
  }

  /// Save an answer to the database.
  ///
  /// # Errors
  /// Returns an error if serialization or database write fails.
  pub fn save_answer(&self, answer: &Answer) -> StoreResult<()> {
    self.ensure_tables()?;

    let record = AnswerRecord::from_answer(
      answer.step_id.clone(),
      answer.value.clone(),
      answer.timestamp.clone(),
    );

    let txn = self.db.begin_write()?;
    {
      let table_definition: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(tables::ANSWERS);
      let mut table = txn.open_table(table_definition)?;

      let json = serde_json::to_string(&record).map_err(StorageError::serialization)?;
      table.insert(answer.step_id.as_str(), json.as_str())?;
    }
    txn.commit()?;
    Ok(())
  }

  /// Load all answers from the database.
  ///
  /// # Errors
  /// Returns an error if deserialization or database read fails.
  pub fn get_all_answers(&self) -> StoreResult<Vec<Answer>> {
    self.ensure_tables()?;

    let txn = self.db.begin_read()?;
    let table_definition: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(tables::ANSWERS);
    // Table may not exist yet if no answers have been saved
    let table = match txn.open_table(table_definition) {
      Ok(t) => t,
      Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
      Err(e) => return Err(e.into()),
    };

    let mut answers = Vec::new();
    let mut iter = table.iter()?;
    while let Some(result) = iter.next() {
      let (_key, value) = result?;
      let json: &str = value.value();
      let record: AnswerRecord =
        serde_json::from_str(json).map_err(StorageError::deserialization)?;
      answers.push(Answer {
        step_id: record.step_id,
        value: record.value,
        timestamp: record.timestamp,
      });
    }
    Ok(answers)
  }

  /// Get a single answer by step_id.
  ///
  /// # Errors
  /// Returns an error if deserialization or database read fails.
  pub fn get_answer(&self, step_id: &str) -> StoreResult<Option<Answer>> {
    self.ensure_tables()?;

    let txn = self.db.begin_read()?;
    let table_definition: redb::TableDefinition<&str, &str> =
      redb::TableDefinition::new(tables::ANSWERS);
    // Table may not exist yet if no answers have been saved
    let table = match txn.open_table(table_definition) {
      Ok(t) => t,
      Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
      Err(e) => return Err(e.into()),
    };

    match table.get(step_id)? {
      Some(guard) => {
        let json: &str = guard.value();
        let record: AnswerRecord =
          serde_json::from_str(json).map_err(StorageError::deserialization)?;
        Ok(Some(Answer {
          step_id: record.step_id,
          value: record.value,
          timestamp: record.timestamp,
        }))
      }
      None => Ok(None),
    }
  }

  /// Delete an answer from the database.
  ///
  /// # Errors
  /// Returns an error if database operation fails.
  pub fn delete_answer(&self, step_id: &str) -> StoreResult<bool> {
    self.ensure_tables()?;

    let txn = self.db.begin_write()?;
    let deleted = {
      let table_definition: redb::TableDefinition<&str, &str> =
        redb::TableDefinition::new(tables::ANSWERS);
      let mut table = txn.open_table(table_definition)?;
      let removed = table.remove(step_id)?;
      removed.is_some()
    };
    txn.commit()?;
    Ok(deleted)
  }

  /// Load all answers from the database (alias for get_all_answers).
  ///
  /// # Errors
  /// Returns an error if deserialization or database read fails.
  pub fn load_answers(&self) -> StoreResult<Vec<Answer>> {
    self.get_all_answers()
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn test_open_in_memory() {
    assert!(RedbStore::open_in_memory().is_ok());
  }

  #[test]
  fn test_save_and_get_metadata() {
    let store = RedbStore::open_in_memory().expect("Failed to open in-memory store");
    let metadata = ProjectMetadata::new(
      "agile".to_string(),
      "discover".to_string(),
      "2024-02-25T10:00:00Z".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    assert!(store.save_metadata(&metadata).is_ok());
    let result = store.get_metadata().expect("Failed to get metadata");
    assert_eq!(result, Some(metadata));
  }

  #[test]
  fn test_metadata_persistence_mode_preference() {
    let store = RedbStore::open_in_memory().expect("Failed to open in-memory store");
    let metadata1 = ProjectMetadata::new(
      "waterfall".to_string(),
      "define".to_string(),
      "2024-02-25T10:00:00Z".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    store
      .save_metadata(&metadata1)
      .expect("Failed to save metadata");

    let metadata2 = ProjectMetadata::new(
      "agile".to_string(),
      "discover".to_string(),
      "2024-02-25T10:00:00Z".to_string(),
      "2024-02-25T13:00:00Z".to_string(),
    );
    store
      .save_metadata(&metadata2)
      .expect("Failed to save metadata");

    let result = store.get_metadata().expect("Failed to get metadata");
    assert_eq!(result, Some(metadata2));
    let mode = result.map(|m| m.mode_preference).unwrap_or_default();
    assert_eq!(mode, "agile");
  }

  #[test]
  fn test_save_and_get_extraction_cache() {
    let store = RedbStore::open_in_memory().expect("Failed to open store");
    let cache = ExtractionCache::new(
      "hash-abc123".to_string(),
      r#"{"field": "value"}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    assert!(store.save_extraction_cache("hash-abc123", &cache).is_ok());
    let result = store
      .get_extraction_cache("hash-abc123")
      .expect("Failed to get cache");
    assert_eq!(result, Some(cache));
  }

  #[test]
  fn test_save_and_get_lattice_cache() {
    let store = RedbStore::open_in_memory().expect("Failed to open store");
    let cache = LatticeCache::new(
      "discover".to_string(),
      r#"{"nodes": []}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    assert!(store.save_lattice_cache("discover", &cache).is_ok());
    let result = store
      .get_lattice_cache("discover")
      .expect("Failed to get cache");
    assert_eq!(result, Some(cache));
  }

  #[test]
  fn test_all_tables_concurrent_operations() {
    let store = RedbStore::open_in_memory().expect("Failed to open store");
    let metadata = ProjectMetadata::new(
      "agile".to_string(),
      "discover".to_string(),
      "2024-02-25T10:00:00Z".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    let extraction = ExtractionCache::new(
      "hash-xyz".to_string(),
      r#"{"extracted": true}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );
    let lattice = LatticeCache::new(
      "discover".to_string(),
      r#"{"lattice": "data"}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );

    store
      .save_metadata(&metadata)
      .expect("Failed to save metadata");
    store
      .save_extraction_cache("hash-xyz", &extraction)
      .expect("Failed to save extraction");
    store
      .save_lattice_cache("discover", &lattice)
      .expect("Failed to save lattice");

    assert_eq!(
      store.get_metadata().expect("Failed to get metadata"),
      Some(metadata)
    );
    assert_eq!(
      store
        .get_extraction_cache("hash-xyz")
        .expect("Failed to get extraction"),
      Some(extraction)
    );
    assert_eq!(
      store
        .get_lattice_cache("discover")
        .expect("Failed to get lattice"),
      Some(lattice)
    );
  }
}
