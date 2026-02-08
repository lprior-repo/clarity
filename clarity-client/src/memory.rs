//! Memory optimization utilities for desktop applications
//!
//! This module provides memory-efficient data structures and patterns
//! for reducing memory usage and preventing leaks.

use std::sync::Arc;

/// Share large data structures across components using Arc
///
/// # Type Parameters
/// * `T` - The type of data to share
#[derive(Clone, Debug)]
pub struct SharedState<T> {
  /// The Arc-wrapped data
  inner: Arc<T>,
}

impl<T> SharedState<T> {
  /// Create a new shared state
  ///
  /// # Arguments
  /// * `data` - The data to wrap
  ///
  /// # Returns
  /// A new SharedState instance
  #[must_use]
  pub fn new(data: T) -> Self {
    Self {
      inner: Arc::new(data),
    }
  }

  /// Get a reference to the shared data
  ///
  /// # Returns
  /// A reference to the inner data
  #[must_use]
  pub fn get(&self) -> &T {
    &self.inner
  }

  /// Get the Arc for more advanced usage
  ///
  /// # Returns
  /// A clone of the inner Arc
  #[must_use]
  pub fn arc(&self) -> Arc<T> {
    Arc::clone(&self.inner)
  }

  /// Check if two SharedState instances share the same data
  ///
  /// # Arguments
  /// * `other` - Another SharedState instance
  ///
  /// # Returns
  /// `true` if both instances point to the same data
  #[must_use]
  pub fn same_data(&self, other: &Self) -> bool {
    std::ptr::eq(&*self.inner as *const T, &*other.inner as *const T)
  }
}

impl<T> AsRef<T> for SharedState<T> {
  fn as_ref(&self) -> &T {
    self.get()
  }
}

impl<T: Default> Default for SharedState<T> {
  fn default() -> Self {
    Self::new(T::default())
  }
}

// ============================================================================
// Martin Fowler Test Suite: Memory Optimization
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // Test 1: Shared state reduces memory usage

  #[test]
  fn test_shared_state_new_creates_arc_wrapped_data() {
    // GIVEN: some data
    let data = vec![1, 2, 3, 4, 5];

    // WHEN: creating shared state
    let shared = SharedState::new(data.clone());

    // THEN: should be able to access the data
    assert_eq!(shared.get(), &data);
  }

  #[test]
  fn test_shared_state_clone_shares_same_data() {
    // GIVEN: a shared state with large data
    let large_data = vec![0u8; 10_000_000]; // 10 MB
    let shared1 = SharedState::new(large_data);

    // WHEN: cloning the shared state
    let shared2 = shared1.clone();

    // THEN: both should point to the same data
    assert!(shared1.same_data(&shared2));
    assert!(std::ptr::eq(shared1.get(), shared2.get()));
  }

  #[test]
  fn test_shared_state_multiple_clones_single_allocation() {
    // GIVEN: a shared state
    let data = "shared string data".to_string();
    let shared1 = SharedState::new(data);

    // WHEN: creating multiple clones
    let shared2 = shared1.clone();
    let shared3 = shared2.clone();
    let shared4 = shared3.clone();

    // THEN: all should share the same data
    assert!(shared1.same_data(&shared2));
    assert!(shared2.same_data(&shared3));
    assert!(shared3.same_data(&shared4));
    assert_eq!(shared1.get(), shared4.get());
  }

  #[test]
  fn test_shared_state_arc_method_returns_arc() {
    // GIVEN: a shared state
    let data = vec![1, 2, 3];
    let shared = SharedState::new(data);

    // WHEN: getting the Arc
    let arc = shared.arc();

    // THEN: should be able to access data through Arc
    assert_eq!(*arc, vec![1, 2, 3]);
  }

  #[test]
  fn test_shared_state_as_ref_trait() {
    // GIVEN: a shared state
    let data = "test data".to_string();
    let shared = SharedState::new(data);

    // WHEN: using as_ref
    let ref_data: &str = shared.as_ref();

    // THEN: should get reference to inner data
    assert_eq!(ref_data, "test data");
  }

  #[test]
  fn test_shared_state_with_complex_data() {
    // GIVEN: complex nested data structure
    #[derive(Debug, PartialEq, Eq)]
    struct ComplexData {
      id: u32,
      name: String,
      values: Vec<u32>,
    }

    let data = ComplexData {
      id: 42,
      name: "test".to_string(),
      values: vec![1, 2, 3, 4, 5],
    };

    // WHEN: creating shared state
    let shared = SharedState::new(data);

    // THEN: should access all fields
    assert_eq!(shared.get().id, 42);
    assert_eq!(shared.get().name, "test");
    assert_eq!(shared.get().values, vec![1, 2, 3, 4, 5]);
  }
}
