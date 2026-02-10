#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Loading state management hook
//!
//! Provides composable loading state tracking for async operations.

use dioxus::prelude::*;
use futures_util as _;
use rpds::Vector;
use std::rc::Rc;

/// Type for callbacks (single-threaded UI context)
pub type AsyncCallback<T> = Rc<dyn Fn(T)>;

/// Loading state for a specific operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadingState {
  /// Unique identifier for this loading operation
  pub key: String,
  /// Whether the operation is currently loading
  pub loading: bool,
  /// Optional message to display while loading
  pub message: Option<String>,
  /// Progress value (0-100) if available
  pub progress: Option<u8>,
}

impl LoadingState {
  /// Create a new loading state
  #[must_use]
  pub const fn new(key: String) -> Self {
    Self {
      key,
      loading: false,
      message: None,
      progress: None,
    }
  }

  /// Set loading to true with optional message
  #[must_use]
  pub const fn with_loading(mut self, loading: bool) -> Self {
    self.loading = loading;
    self
  }

  /// Set the loading message
  #[must_use]
  pub fn with_message(mut self, message: String) -> Self {
    self.message = Some(message);
    self
  }

  /// Set progress (0-100)
  #[must_use]
  pub const fn with_progress(mut self, progress: u8) -> Self {
    self.progress = Some(progress);
    self
  }

  /// Clear the loading state
  #[must_use]
  pub fn clear(&self) -> Self {
    Self {
      key: self.key.clone(),
      loading: false,
      message: None,
      progress: None,
    }
  }
}

/// Global loading state manager
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadingManager {
  /// All tracked loading states
  states: Vector<LoadingState>,
}

impl LoadingManager {
  /// Create a new loading manager
  #[must_use]
  pub fn new() -> Self {
    Self {
      states: Vector::new(),
    }
  }

  /// Check if any operation is loading
  #[must_use]
  pub fn is_loading(&self) -> bool {
    self.states.iter().any(|s| s.loading)
  }

  /// Check if a specific key is loading
  #[must_use]
  pub fn is_loading_key(&self, key: &str) -> bool {
    self
      .states
      .iter()
      .find(|s| s.key == key)
      .is_some_and(|s| s.loading)
  }

  /// Get the number of active loading operations
  #[must_use]
  pub fn loading_count(&self) -> usize {
    self.states.iter().filter(|s| s.loading).count()
  }

  /// Get loading state for a specific key
  #[must_use]
  pub fn get(&self, key: &str) -> Option<&LoadingState> {
    self.states.iter().find(|s| s.key == key)
  }

  /// Get all loading messages
  #[must_use]
  pub fn loading_messages(&self) -> Vec<String> {
    self
      .states
      .iter()
      .filter(|s| s.loading)
      .filter_map(|s| s.message.clone())
      .collect()
  }

  /// Get the primary loading message (first active)
  #[must_use]
  pub fn primary_message(&self) -> Option<String> {
    self
      .states
      .iter()
      .find(|s| s.loading)
      .and_then(|s| s.message.clone())
  }

  /// Update or add a loading state (pure transformation)
  #[must_use]
  pub fn set(&self, key: String, loading: bool, message: Option<String>) -> Self {
    let state = LoadingState {
      key: key.clone(),
      loading,
      message,
      progress: None,
    };

    // Remove existing state if present, then add the new one
    let new_states = self.remove(&key).states.push_back(state);

    Self { states: new_states }
  }

  /// Set loading state with message (convenience method)
  #[must_use]
  pub fn set_loading(&self, key: String, message: String) -> Self {
    self.set(key, true, Some(message))
  }

  /// Clear loading for a key
  #[must_use]
  pub fn clear(&self, key: &str) -> Self {
    let new_states = self
      .states
      .iter()
      .map(|s| if s.key == key { s.clear() } else { s.clone() })
      .collect();

    Self { states: new_states }
  }

  /// Remove a loading state entirely
  #[must_use]
  pub fn remove(&self, key: &str) -> Self {
    let new_states = self
      .states
      .iter()
      .filter(|s| s.key != key)
      .cloned()
      .collect();
    Self { states: new_states }
  }

  /// Clear all loading states
  #[must_use]
  pub fn clear_all(&self) -> Self {
    Self {
      states: Vector::new(),
    }
  }
}

impl Default for LoadingManager {
  fn default() -> Self {
    Self::new()
  }
}

/// Hook to access loading state manager
///
/// Returns a signal containing the loading manager.
#[must_use]
pub fn use_loading_manager() -> Signal<LoadingManager> {
  use_signal(LoadingManager::new)
}

/// Hook to check if anything is loading
///
/// Returns a boolean signal that updates when loading state changes.
#[must_use]
pub fn use_is_loading() -> bool {
  let manager = use_loading_manager();
  let result = manager.read().is_loading();
  result
}

/// Hook to check if a specific key is loading
///
/// # Arguments
/// * `key` - The loading operation key to check
///
/// # Returns
/// A boolean signal indicating if the key is loading
#[must_use]
pub fn use_is_loading_key(key: String) -> bool {
  let manager = use_loading_manager();
  let result = manager.read().is_loading_key(&key);
  result
}

/// Hook to get the primary loading message
///
/// Returns the message of the first active loading operation.
#[must_use]
pub fn use_loading_message() -> Option<String> {
  let manager = use_loading_manager();
  let result = manager.read().primary_message();
  result
}

/// Hook to get all loading messages
///
/// Returns a vector of all active loading messages.
#[must_use]
pub fn use_loading_messages() -> Vec<String> {
  let manager = use_loading_manager();
  let result = manager.read().loading_messages();
  result
}

/// Hook to get loading state for a specific key
///
/// # Arguments
/// * `key` - The loading operation key
///
/// # Returns
/// Option containing the loading state if it exists
#[must_use]
pub fn use_loading_state(key: String) -> Option<LoadingState> {
  let manager = use_loading_manager();
  let result = manager.read().get(&key).cloned();
  result
}

/// Hook to create loading operations
///
/// Returns callbacks for starting and stopping loading operations.
#[must_use]
pub fn use_loading_operations() -> LoadingOperations {
  let manager = use_loading_manager();

  LoadingOperations {
    start: {
      let manager = manager;
      Rc::new(move |(key, message): (String, String)| {
        let current = manager.read().clone();
        let mut mgr = manager;
        mgr.set(current.set_loading(key, message));
      })
    },
    stop: {
      let manager = manager;
      Rc::new(move |key: String| {
        let current = manager.read().clone();
        let mut mgr = manager;
        mgr.set(current.clear(&key));
      })
    },
    set: {
      let manager = manager;
      Rc::new(
        move |(key, loading, message): (String, bool, Option<String>)| {
          let current = manager.read().clone();
          let mut mgr = manager;
          mgr.set(current.set(key, loading, message));
        },
      )
    },
    remove: {
      let manager = manager;
      Rc::new(move |key: String| {
        let current = manager.read().clone();
        let mut mgr = manager;
        mgr.set(current.remove(&key));
      })
    },
  }
}

/// Loading operation callbacks
#[derive(Clone)]
pub struct LoadingOperations {
  /// Start a loading operation with a message
  pub start: AsyncCallback<(String, String)>,
  /// Stop a loading operation
  pub stop: AsyncCallback<String>,
  /// Set loading state with custom parameters
  pub set: AsyncCallback<(String, bool, Option<String>)>,
  /// Remove a loading state entirely
  pub remove: AsyncCallback<String>,
}

/// Hook to run an async operation with automatic loading state management
///
/// # Arguments
/// * `key` - The loading operation key
/// * `message` - Loading message to display
/// * `operation` - The async operation to run
///
/// # Returns
/// A future that resolves when the operation completes
pub fn use_loading_operation<T, E, F, Fut>(
  key: String,
  message: String,
  operation: F,
) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>> + Clone + 'static
where
  F: Fn() -> Fut + Clone + 'static,
  Fut: std::future::Future<Output = Result<T, E>> + 'static,
  T: 'static,
  E: 'static,
{
  let ops = use_loading_operations();

  move || {
    let key = key.clone();
    let message = message.clone();
    let operation = operation.clone();
    let ops = ops.clone();

    (ops.start)((key.clone(), message));

    Box::pin(async move {
      let result = operation().await;
      (ops.stop)(key);
      result
    })
  }
}

/// Hook to wrap multiple operations in a single loading state
///
/// Useful for batch operations where you want to show one loading state
/// for multiple async operations.
///
/// # Arguments
/// * `key` - The loading operation key
/// * `message` - Loading message to display
/// * `operations` - Vector of (key, operation) tuples
///
/// # Returns
/// A future that resolves when all operations complete
pub fn use_loading_batch<T, E, F, Fut>(
  key: String,
  message: String,
  operations: Vec<(String, F)>,
) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Result<T, E>>>>> + Clone + 'static
where
  F: Fn() -> Fut + Clone + 'static,
  Fut: std::future::Future<Output = Result<T, E>> + 'static,
  T: 'static,
  E: 'static,
{
  let ops = use_loading_operations();

  move || {
    let key = key.clone();
    let message = message.clone();
    let operations = operations.clone();
    let ops = ops.clone();

    (ops.start)((key.clone(), message));

    Box::pin(async move {
      // Start individual operations without messages (silent)
      for (op_key, _) in &operations {
        (ops.start)((op_key.clone(), String::new()));
      }

      // Run all operations
      let results: Vec<_> =
        futures_util::future::join_all(operations.iter().map(|(_, op)| op())).await;

      // Stop all individual operations
      for (op_key, _) in &operations {
        (ops.stop)(op_key.clone());
      }

      // Stop the batch loading
      (ops.stop)(key);

      // Return results (errors are in the vector)
      results
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_loading_state_new() {
    let state = LoadingState::new("test".to_string());
    assert!(!state.loading);
    assert!(state.message.is_none());
    assert!(state.progress.is_none());
  }

  #[test]
  fn test_loading_state_builder() {
    let state = LoadingState::new("test".to_string())
      .with_loading(true)
      .with_message("Loading...".to_string())
      .with_progress(50);

    assert!(state.loading);
    assert_eq!(state.message, Some("Loading...".to_string()));
    assert_eq!(state.progress, Some(50));
  }

  #[test]
  fn test_loading_state_clear() {
    let state = LoadingState::new("test".to_string())
      .with_loading(true)
      .with_message("Loading...".to_string());

    let cleared = state.clear();
    assert!(!cleared.loading);
    assert!(cleared.message.is_none());
    assert!(cleared.progress.is_none());
    assert_eq!(cleared.key, state.key);
  }

  #[test]
  fn test_loading_manager_new() {
    let manager = LoadingManager::new();
    assert!(!manager.is_loading());
    assert_eq!(manager.loading_count(), 0);
  }

  #[test]
  fn test_loading_manager_set_loading() {
    let manager = LoadingManager::new();
    let updated = manager.set_loading("test".to_string(), "Loading...".to_string());

    assert!(updated.is_loading());
    assert!(updated.is_loading_key("test"));
    assert_eq!(updated.loading_count(), 1);
    assert_eq!(updated.primary_message(), Some("Loading...".to_string()));
  }

  #[test]
  fn test_loading_manager_clear() {
    let manager = LoadingManager::new();
    let loaded = manager.set_loading("test".to_string(), "Loading...".to_string());
    assert!(loaded.is_loading());

    let cleared = loaded.clear("test");
    assert!(!cleared.is_loading());
    assert!(!cleared.is_loading_key("test"));
  }

  #[test]
  fn test_loading_manager_multiple_states() {
    let manager = LoadingManager::new();
    let updated = manager
      .set_loading("first".to_string(), "Loading first...".to_string())
      .set_loading("second".to_string(), "Loading second...".to_string());

    assert!(updated.is_loading());
    assert_eq!(updated.loading_count(), 2);

    let messages = updated.loading_messages();
    assert_eq!(messages.len(), 2);
    assert!(messages.contains(&"Loading first...".to_string()));
    assert!(messages.contains(&"Loading second...".to_string()));
  }

  #[test]
  fn test_loading_manager_clear_all() {
    let manager = LoadingManager::new();
    let loaded = manager
      .set_loading("first".to_string(), "Loading first...".to_string())
      .set_loading("second".to_string(), "Loading second...".to_string());

    assert!(loaded.is_loading());

    let cleared = loaded.clear_all();
    assert!(!cleared.is_loading());
    assert_eq!(cleared.loading_count(), 0);
  }

  #[test]
  fn test_loading_manager_remove() {
    let manager = LoadingManager::new();
    let loaded = manager.set_loading("test".to_string(), "Loading...".to_string());
    assert!(loaded.is_loading());

    let removed = loaded.remove("test");
    assert!(!removed.is_loading());
    assert!(removed.get("test").is_none());
  }

  #[test]
  fn test_loading_manager_update_existing() {
    let manager = LoadingManager::new();
    let updated = manager.set_loading("test".to_string(), "Loading...".to_string());
    assert_eq!(updated.loading_count(), 1);

    // Update the same key to stop loading
    let stopped = updated.set("test".to_string(), false, None);
    assert!(!stopped.is_loading());
    // Key should still exist
    assert!(stopped.get("test").is_some());
  }

  #[test]
  fn test_loading_manager_primary_message() {
    let manager = LoadingManager::new();
    let updated = manager
      .set_loading("first".to_string(), "First message".to_string())
      .set_loading("second".to_string(), "Second message".to_string());

    // Should return the first message
    assert_eq!(updated.primary_message(), Some("First message".to_string()));
  }

  #[test]
  fn test_loading_manager_default() {
    let manager = LoadingManager::default();
    assert!(!manager.is_loading());
    assert_eq!(manager.loading_count(), 0);
  }

  #[test]
  fn test_loading_operations_clone() {
    let ops: LoadingOperations = LoadingOperations {
      start: Rc::new(|(_, _)| {}),
      stop: Rc::new(|_| {}),
      set: Rc::new(|(_, _, _)| {}),
      remove: Rc::new(|_| {}),
    };

    let _ = ops.clone();
  }
}
