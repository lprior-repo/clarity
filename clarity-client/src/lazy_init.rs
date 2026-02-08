//! Lazy initialization support for desktop optimizations
//!
//! This module provides OnceLock-based lazy initialization to defer
//! expensive operations until they're actually needed, improving startup time.

use std::sync::OnceLock;

/// Lazy-initialized state that's computed once and cached
///
/// # Type Parameters
/// * `T` - The type of value to lazily initialize
#[derive(Debug)]
pub struct LazyState<T> {
  /// The OnceLock that stores the initialized value
  init: OnceLock<T>,
}

impl<T> LazyState<T> {
  /// Create a new lazy state with the given factory function
  ///
  /// # Examples
  /// ```
  /// use clarity_client::lazy_init::LazyState;
  ///
  /// let lazy = LazyState::new();
  /// let value = lazy.get_or_init(|| "computed");
  /// assert_eq!(*value, "computed");
  /// ```
  #[must_use]
  pub const fn new() -> Self {
    Self {
      init: OnceLock::new(),
    }
  }

  /// Get the value, initializing it with the factory if needed
  ///
  /// # Arguments
  /// * `factory` - Function that creates the value if not initialized
  ///
  /// # Returns
  /// A reference to the initialized value
  ///
  /// # Examples
  /// ```
  /// use clarity_client::lazy_init::LazyState;
  ///
  /// let lazy = LazyState::new();
  ///
  /// // First call initializes the value
  /// let v1 = lazy.get_or_init(|| "computed");
  ///
  /// // Subsequent calls return the cached value
  /// let v2 = lazy.get_or_init(|| panic!("should not run"));
  ///
  /// assert!(std::ptr::eq(v1, v2));
  /// ```
  ///
  /// # Panics
  /// This function does not panic. The factory function may panic,
  /// but that's a bug in the factory, not this code.
  #[must_use]
  pub fn get_or_init(&self, factory: impl FnOnce() -> T) -> &T {
    self.init.get_or_init(factory)
  }

  /// Check if the value has been initialized
  ///
  /// # Returns
  /// `true` if the value has been initialized, `false` otherwise
  #[must_use]
  pub fn is_initialized(&self) -> bool {
    self.init.get().is_some()
  }
}

impl<T> Default for LazyState<T> {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// Martin Fowler Test Suite: Desktop Optimizations
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU32, Ordering};

  // Test 1: Lazy initialization defers computation

  #[test]
  fn test_lazy_state_defers_initialization_until_first_access() {
    // GIVEN: a lazy state
    let lazy: LazyState<&str> = LazyState::new();
    let counter = AtomicU32::new(0);

    // WHEN: the lazy state is created but not accessed
    // THEN: the factory should not have been called
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    assert!(!lazy.is_initialized());
  }

  #[test]
  fn test_lazy_state_initializes_on_first_access() {
    // GIVEN: a lazy state with a counter
    let counter = AtomicU32::new(0);
    let lazy = LazyState::new();

    // WHEN: accessing the value for the first time
    let value = lazy.get_or_init(|| {
      counter.fetch_add(1, Ordering::SeqCst);
      42
    });

    // THEN: the factory should be called exactly once
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(*value, 42);
    assert!(lazy.is_initialized());
  }

  #[test]
  fn test_lazy_state_caches_result() {
    // GIVEN: a lazy state that's been initialized
    let counter = AtomicU32::new(0);
    let lazy = LazyState::new();

    // WHEN: accessing the value multiple times
    let v1 = lazy.get_or_init(|| {
      counter.fetch_add(1, Ordering::SeqCst);
      "cached"
    });
    let v2 = lazy.get_or_init(|| {
      counter.fetch_add(1, Ordering::SeqCst);
      "never called"
    });
    let v3 = lazy.get_or_init(|| {
      counter.fetch_add(1, Ordering::SeqCst);
      "never called"
    });

    // THEN: the factory should only be called once
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(*v1, "cached");
    assert_eq!(*v2, "cached");
    assert_eq!(*v3, "cached");
    // All references point to the same value
    assert!(std::ptr::eq(v1, v2));
    assert!(std::ptr::eq(v2, v3));
  }

  #[test]
  fn test_lazy_state_is_initialized_returns_false_before_access() {
    // GIVEN: a newly created lazy state
    let lazy: LazyState<&str> = LazyState::new();

    // WHEN: checking if initialized before access
    // THEN: should return false
    assert!(!lazy.is_initialized());
  }

  #[test]
  fn test_lazy_state_is_initialized_returns_true_after_access() {
    // GIVEN: a lazy state that's been accessed
    let lazy = LazyState::new();
    let _ = lazy.get_or_init(|| "value");

    // WHEN: checking if initialized after access
    // THEN: should return true
    assert!(lazy.is_initialized());
  }

  #[test]
  fn test_lazy_state_with_expensive_computation() {
    // GIVEN: a lazy state with an expensive factory (simulated by sleep)
    use std::thread;
    use std::time::{Duration, Instant};

    let lazy: LazyState<&str> = LazyState::new();

    // WHEN: measuring first access time
    let start = Instant::now();
    let result1 = lazy.get_or_init(|| {
      thread::sleep(Duration::from_millis(100));
      "expensive_result"
    });
    let first_duration = start.elapsed();

    // THEN: first access should take ~100ms
    assert!(first_duration >= Duration::from_millis(90));

    // WHEN: measuring second access time
    let start = Instant::now();
    let result2 = lazy.get_or_init(|| "never called");
    let second_duration = start.elapsed();

    // THEN: second access should be instant (< 1ms)
    assert!(second_duration < Duration::from_millis(1));
    assert_eq!(*result1, "expensive_result");
    assert_eq!(*result2, "expensive_result");
    assert!(std::ptr::eq(result1, result2));
  }

  #[test]
  fn test_lazy_state_with_complex_type() {
    // GIVEN: a lazy state with a complex type (Vec)
    let lazy: LazyState<Vec<i32>> = LazyState::new();

    // WHEN: accessing the value
    let vec = lazy.get_or_init(|| vec![1, 2, 3, 4, 5]);

    // THEN: should get the expected vector
    assert_eq!(vec, &vec![1, 2, 3, 4, 5]);
    assert!(lazy.is_initialized());
  }

  #[test]
  fn test_lazy_state_multiple_independent_instances() {
    // GIVEN: multiple independent lazy states
    let lazy1 = LazyState::new();
    let lazy2 = LazyState::new();
    let lazy3 = LazyState::new();

    // WHEN: accessing them in different orders
    let v2 = lazy2.get_or_init(|| "second");
    let v1 = lazy1.get_or_init(|| "first");
    let v3 = lazy3.get_or_init(|| "third");

    // THEN: each should have its own value
    assert_eq!(*v1, "first");
    assert_eq!(*v2, "second");
    assert_eq!(*v3, "third");
  }

  #[test]
  fn test_lazy_state_default_trait() {
    // GIVEN: a lazy state created with Default
    let lazy = LazyState::<Vec<i32>>::default();

    // WHEN: accessing the value
    let vec = lazy.get_or_init(|| vec![1, 2, 3]);

    // THEN: should work the same as new()
    assert_eq!(vec, &vec![1, 2, 3]);
  }
}
