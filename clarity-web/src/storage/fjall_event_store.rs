#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]
#![cfg(not(target_arch = "wasm32"))]

//! Fjall-backed canonical event store foundation for the Clarity CLI.
//!
//! Redb modules in this crate are legacy UI/transcript persistence. This module
//! is the canonical storage foundation for `MASTER_DOC.md` event-sourced CLI
//! sessions.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const SESSION_HASH_BYTES: usize = 16;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum FjallStoreError {
  #[error("fjall operation failed: {message}")]
  Fjall { message: String },

  #[error("event serialization failed: {message}")]
  Serialization { message: String },
}

impl From<fjall::Error> for FjallStoreError {
  fn from(error: fjall::Error) -> Self {
    Self::Fjall {
      message: error.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
  pub session_id: String,
  pub seq: u64,
  pub event_id: String,
  pub event_type: String,
  pub payload: serde_json::Value,
  pub created_at: String,
  pub idempotency_key: String,
  pub schema_version: String,
  pub actor: String,
  pub prev_event_hash: Option<String>,
  pub event_hash: Option<String>,
}

pub struct FjallEventStore {
  db: fjall::Database,
  events: fjall::Keyspace,
  snapshots: fjall::Keyspace,
  locks: fjall::Keyspace,
  artifacts: fjall::Keyspace,
  gate_results: fjall::Keyspace,
  projection_status: fjall::Keyspace,
  bd_mappings: fjall::Keyspace,
}

impl FjallEventStore {
  /// Open the canonical Fjall database and all required Clarity keyspaces.
  ///
  /// # Errors
  ///
  /// Returns `FjallStoreError` when the database cannot be opened or any
  /// required keyspace cannot be created/opened.
  pub fn open(path: impl AsRef<Path>) -> Result<Self, FjallStoreError> {
    let db = fjall::Database::builder(path.as_ref()).open()?;
    Ok(Self {
      events: open_keyspace(&db, "events")?,
      snapshots: open_keyspace(&db, "snapshots")?,
      locks: open_keyspace(&db, "locks")?,
      artifacts: open_keyspace(&db, "artifacts")?,
      gate_results: open_keyspace(&db, "gate_results")?,
      projection_status: open_keyspace(&db, "projection_status")?,
      bd_mappings: open_keyspace(&db, "bd_mappings")?,
      db,
    })
  }

  /// Append one event and persist it with `SyncAll` durability.
  ///
  /// # Errors
  ///
  /// Returns `FjallStoreError` when serialization, batch commit, or fsync fails.
  pub fn append_event_sync_all(&self, event: &EventEnvelope) -> Result<(), FjallStoreError> {
    let key = event_key(&event.session_id, event.seq);
    let value = serde_json::to_vec(event).map_err(|error| FjallStoreError::Serialization {
      message: error.to_string(),
    })?;
    let mut batch = self.db.batch();
    batch.insert(&self.events, key, value);
    batch.commit()?;
    self.db.persist(fjall::PersistMode::SyncAll)?;
    Ok(())
  }

  /// Load all events for a session from the canonical event keyspace.
  ///
  /// # Errors
  ///
  /// Returns `FjallStoreError` when Fjall iteration or event deserialization
  /// fails.
  pub fn load_events(&self, session_id: &str) -> Result<Vec<EventEnvelope>, FjallStoreError> {
    let prefix = session_key_prefix(session_id);
    self
      .events
      .prefix(prefix)
      .map(|guard| {
        let value = guard.value().map_err(FjallStoreError::from)?;
        serde_json::from_slice(value.as_ref()).map_err(|error| FjallStoreError::Serialization {
          message: error.to_string(),
        })
      })
      .collect()
  }
}

#[must_use]
pub fn event_key(session_id: &str, seq: u64) -> Vec<u8> {
  session_key_prefix(session_id)
    .into_iter()
    .chain(seq.to_be_bytes())
    .collect()
}

#[must_use]
pub fn session_key_prefix(session_id: &str) -> Vec<u8> {
  let digest = Sha256::digest(session_id.as_bytes());
  digest.iter().take(SESSION_HASH_BYTES).copied().collect()
}

#[must_use]
pub fn canonical_sha256(bytes: &[u8]) -> String {
  let digest = Sha256::digest(bytes);
  format!("sha256:{digest:x}")
}

fn open_keyspace(db: &fjall::Database, name: &str) -> Result<fjall::Keyspace, FjallStoreError> {
  db.keyspace(name, fjall::KeyspaceCreateOptions::default)
    .map_err(FjallStoreError::from)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
  use super::*;

  #[test]
  fn event_key_uses_big_endian_sequence_suffix() {
    let key = event_key("session-a", 42);
    let expected_suffix = 42_u64.to_be_bytes();
    assert_eq!(key.len(), SESSION_HASH_BYTES + expected_suffix.len());
    assert!(key.ends_with(&expected_suffix));
  }

  #[test]
  fn append_event_persists_without_redb() -> Result<(), FjallStoreError> {
    let temp_dir = tempfile::tempdir().map_err(|error| FjallStoreError::Fjall {
      message: error.to_string(),
    })?;
    let store = FjallEventStore::open(temp_dir.path())?;
    let event = EventEnvelope {
      session_id: "clarity-test-session".to_string(),
      seq: 1,
      event_id: "018f0000-0000-7000-8000-000000000001".to_string(),
      event_type: "InterviewStarted".to_string(),
      payload: serde_json::json!({"kind":"session","command":"interview start"}),
      created_at: "2026-06-21T00:00:00Z".to_string(),
      idempotency_key: "session-start:clarity-test-session".to_string(),
      schema_version: "1.0.0".to_string(),
      actor: "System".to_string(),
      prev_event_hash: None,
      event_hash: None,
    };
    store.append_event_sync_all(&event)
  }
}
