#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Testing Framework Infrastructure for PME
//!
//! Provides testing utilities with:
//! - Test fixture management
//! - Coverage tracking (target: 80%)
//! - Property-based testing helpers
//! - Mock implementations
//! - Test assertions with functional patterns
//!
//! # Example
//!
//! ```rust,ignore
//! use pme::infra::testing::{TestFixture, CoverageTracker, assert_ok, assert_err};
//!
//! let mut fixture = TestFixture::new("my_test");
//! fixture.setup().ok();
//!
//! let result = my_function();
//! assert_ok!(result);
//!
//! let coverage = CoverageTracker::global();
//! coverage.record("my_module", "function_name", true);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during testing operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TestingError {
  /// Fixture setup failed
  #[error("fixture setup failed: {0}")]
  FixtureSetupFailed(String),

  /// Fixture teardown failed
  #[error("fixture teardown failed: {0}")]
  FixtureTeardownFailed(String),

  /// Assertion failed
  #[error("assertion failed: {0}")]
  AssertionFailed(String),

  /// Test case not found
  #[error("test case not found: {0}")]
  TestCaseNotFound(String),

  /// Coverage target not met
  #[error("coverage target not met: {0:.1}% < {1:.1}%")]
  CoverageTargetNotMet(f64, f64),

  /// Invalid test configuration
  #[error("invalid test configuration: {0}")]
  InvalidConfiguration(String),
}

// ============================================================================
// Test Result Types
// ============================================================================

/// Result of a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
  /// Test name
  pub name: String,
  /// Module path
  pub module: String,
  /// Whether the test passed
  pub passed: bool,
  /// Duration in milliseconds
  pub duration_ms: u64,
  /// Error message (if failed)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
  /// Timestamp
  pub timestamp: DateTime<Utc>,
}

impl TestResult {
  /// Create a passing test result
  #[must_use]
  pub fn passed(name: impl Into<String>, module: impl Into<String>, duration_ms: u64) -> Self {
    Self {
      name: name.into(),
      module: module.into(),
      passed: true,
      duration_ms,
      error: None,
      timestamp: Utc::now(),
    }
  }

  /// Create a failing test result
  #[must_use]
  pub fn failed(
    name: impl Into<String>,
    module: impl Into<String>,
    duration_ms: u64,
    error: impl Into<String>,
  ) -> Self {
    Self {
      name: name.into(),
      module: module.into(),
      passed: false,
      duration_ms,
      error: Some(error.into()),
      timestamp: Utc::now(),
    }
  }
}

/// Summary of test run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestSummary {
  /// Total tests run
  pub total: usize,
  /// Tests passed
  pub passed: usize,
  /// Tests failed
  pub failed: usize,
  /// Total duration in milliseconds
  pub duration_ms: u64,
  /// Individual test results
  pub results: Vec<TestResult>,
}

impl TestSummary {
  /// Create an empty summary
  #[must_use]
  pub const fn new() -> Self {
    Self {
      total: 0,
      passed: 0,
      failed: 0,
      duration_ms: 0,
      results: Vec::new(),
    }
  }

  /// Add a test result
  pub fn add(&mut self, result: TestResult) {
    self.total += 1;
    self.duration_ms += result.duration_ms;
    if result.passed {
      self.passed += 1;
    } else {
      self.failed += 1;
    }
    self.results.push(result);
  }

  /// Merge another summary into this one
  pub fn merge(&mut self, other: &TestSummary) {
    self.total += other.total;
    self.passed += other.passed;
    self.failed += other.failed;
    self.duration_ms += other.duration_ms;
    self.results.extend(other.results.clone());
  }

  /// Get pass rate as percentage
  #[must_use]
  pub fn pass_rate(&self) -> f64 {
    if self.total == 0 {
      return 0.0;
    }
    (f64::from(u32::try_from(self.passed).unwrap_or(0))
      / f64::from(u32::try_from(self.total).unwrap_or(u32::MAX)))
      * 100.0
  }

  /// Check if all tests passed
  #[must_use]
  pub const fn all_passed(&self) -> bool {
    self.failed == 0 && self.total > 0
  }

  /// Get failed test names
  #[must_use]
  pub fn failed_tests(&self) -> Vec<&str> {
    self
      .results
      .iter()
      .filter(|r| !r.passed)
      .map(|r| r.name.as_str())
      .collect()
  }
}

// ============================================================================
// Test Fixture
// ============================================================================

/// A test fixture for setup and teardown
pub struct TestFixture {
  /// Fixture name
  name: String,
  /// Setup function
  setup_fn: Option<Box<dyn FnOnce() -> Result<(), TestingError> + Send>>,
  /// Teardown function
  teardown_fn: Option<Box<dyn FnOnce() -> Result<(), TestingError> + Send>>,
  /// Whether setup has been run
  setup_run: bool,
  /// Temporary paths to clean up
  temp_paths: Vec<PathBuf>,
}

impl TestFixture {
  /// Create a new test fixture
  #[must_use]
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      setup_fn: None,
      teardown_fn: None,
      setup_run: false,
      temp_paths: Vec::new(),
    }
  }

  /// Set the setup function
  #[must_use]
  pub fn with_setup<F>(mut self, f: F) -> Self
  where
    F: FnOnce() -> Result<(), TestingError> + Send + 'static,
  {
    self.setup_fn = Some(Box::new(f));
    self
  }

  /// Set the teardown function
  #[must_use]
  pub fn with_teardown<F>(mut self, f: F) -> Self
  where
    F: FnOnce() -> Result<(), TestingError> + Send + 'static,
  {
    self.teardown_fn = Some(Box::new(f));
    self
  }

  /// Add a temporary path to clean up
  #[must_use]
  pub fn with_temp_path(mut self, path: impl Into<PathBuf>) -> Self {
    self.temp_paths.push(path.into());
    self
  }

  /// Run setup
  pub fn setup(&mut self) -> Result<(), TestingError> {
    if self.setup_run {
      return Ok(());
    }

    if let Some(f) = self.setup_fn.take() {
      f().map_err(|e| TestingError::FixtureSetupFailed(format!("{}: {}", self.name, e)))?;
    }
    self.setup_run = true;
    Ok(())
  }

  /// Run teardown
  pub fn teardown(mut self) -> Result<(), TestingError> {
    // Clean up temporary paths
    for path in &self.temp_paths {
      if path.exists() {
        if path.is_dir() {
          std::fs::remove_dir_all(path).ok();
        } else {
          std::fs::remove_file(path).ok();
        }
      }
    }

    if let Some(f) = self.teardown_fn.take() {
      f().map_err(|e| TestingError::FixtureTeardownFailed(format!("{}: {}", self.name, e)))?;
    }
    Ok(())
  }

  /// Get fixture name
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl fmt::Debug for TestFixture {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TestFixture")
      .field("name", &self.name)
      .field("setup_run", &self.setup_run)
      .field("temp_paths", &self.temp_paths)
      .finish()
  }
}

// ============================================================================
// Coverage Tracker
// ============================================================================

/// Coverage information for a single item
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageItem {
  /// Item name (function, branch, etc.)
  pub name: String,
  /// Number of times covered
  pub hit_count: u64,
  /// Whether this item is covered
  pub covered: bool,
}

impl CoverageItem {
  /// Create a new coverage item
  #[must_use]
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      hit_count: 0,
      covered: false,
    }
  }

  /// Record a hit
  pub fn hit(&mut self) {
    self.hit_count += 1;
    self.covered = true;
  }
}

/// Coverage information for a module
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleCoverage {
  /// Module path
  pub path: String,
  /// Items and their coverage
  pub items: HashMap<String, CoverageItem>,
}

impl ModuleCoverage {
  /// Create new module coverage
  #[must_use]
  pub fn new(path: impl Into<String>) -> Self {
    Self {
      path: path.into(),
      items: HashMap::new(),
    }
  }

  /// Register an item without recording coverage
  pub fn register(&mut self, item_name: impl Into<String>) {
    let item_name = item_name.into();
    self
      .items
      .entry(item_name.clone())
      .or_insert_with(|| CoverageItem::new(&item_name));
  }

  /// Alias for register (for backwards compatibility)
  pub fn register_item(&mut self, item_name: impl Into<String>) {
    self.register(item_name);
  }

  /// Record coverage for an item
  pub fn record(&mut self, item_name: &str) {
    self
      .items
      .entry(item_name.to_string())
      .or_insert_with(|| CoverageItem::new(item_name))
      .hit();
  }

  /// Calculate coverage percentage
  #[must_use]
  pub fn coverage_percent(&self) -> f64 {
    if self.items.is_empty() {
      return 100.0;
    }

    let covered = self.items.values().filter(|i| i.covered).count();
    let total = self.items.len();

    (f64::from(u32::try_from(covered).unwrap_or(0))
      / f64::from(u32::try_from(total).unwrap_or(u32::MAX)))
      * 100.0
  }

  /// Get uncovered items
  #[must_use]
  pub fn uncovered_items(&self) -> Vec<&str> {
    self
      .items
      .iter()
      .filter(|(_, item)| !item.covered)
      .map(|(name, _)| name.as_str())
      .collect()
  }
}

/// Global coverage tracker
#[derive(Debug, Clone)]
pub struct CoverageTracker {
  modules: HashMap<String, ModuleCoverage>,
  target_percent: f64,
}

impl CoverageTracker {
  /// Create a new coverage tracker
  #[must_use]
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      target_percent: 80.0, // Default 80% target
    }
  }

  /// Create with custom target
  #[must_use]
  pub fn with_target(target_percent: f64) -> Self {
    Self {
      modules: HashMap::new(),
      target_percent,
    }
  }

  /// Set coverage target
  #[must_use]
  pub fn with_target_percent(mut self, target: f64) -> Self {
    self.target_percent = target;
    self
  }

  /// Register a module
  pub fn register_module(&mut self, module_path: impl Into<String>) {
    self
      .modules
      .entry(module_path.into())
      .or_insert_with(|| ModuleCoverage::new(""));
  }

  /// Register an item in a module
  pub fn register_item(&mut self, module_path: &str, item_name: impl Into<String>) {
    let item_name = item_name.into();
    self
      .modules
      .entry(module_path.to_string())
      .or_insert_with(|| ModuleCoverage::new(module_path))
      .items
      .entry(item_name.clone())
      .or_insert_with(|| CoverageItem::new(&item_name));
  }

  /// Record coverage for an item
  pub fn record(&mut self, module_path: &str, item_name: &str) {
    self
      .modules
      .entry(module_path.to_string())
      .or_insert_with(|| ModuleCoverage::new(module_path))
      .record(item_name);
  }

  /// Get coverage for a module
  #[must_use]
  pub fn module_coverage(&self, module_path: &str) -> Option<&ModuleCoverage> {
    self.modules.get(module_path)
  }

  /// Calculate total coverage percentage
  #[must_use]
  pub fn total_coverage(&self) -> f64 {
    let total_items: usize = self.modules.values().map(|m| m.items.len()).sum();
    let covered_items: usize = self
      .modules
      .values()
      .map(|m| m.items.values().filter(|i| i.covered).count())
      .sum();

    if total_items == 0 {
      return 100.0;
    }

    (f64::from(u32::try_from(covered_items).unwrap_or(0))
      / f64::from(u32::try_from(total_items).unwrap_or(u32::MAX)))
      * 100.0
  }

  /// Check if target is met
  #[must_use]
  pub fn target_met(&self) -> bool {
    self.total_coverage() >= self.target_percent
  }

  /// Get all uncovered items
  #[must_use]
  pub fn all_uncovered(&self) -> HashMap<String, Vec<&str>> {
    self
      .modules
      .iter()
      .map(|(path, module)| (path.clone(), module.uncovered_items()))
      .filter(|(_, items)| !items.is_empty())
      .collect()
  }

  /// Generate coverage report
  #[must_use]
  pub fn report(&self) -> CoverageReport {
    let module_reports: Vec<ModuleReport> = self
      .modules
      .iter()
      .map(|(path, module)| ModuleReport {
        path: path.clone(),
        coverage_percent: module.coverage_percent(),
        total_items: module.items.len(),
        covered_items: module.items.values().filter(|i| i.covered).count(),
        uncovered_items: module
          .uncovered_items()
          .into_iter()
          .map(String::from)
          .collect(),
      })
      .collect();

    CoverageReport {
      total_coverage: self.total_coverage(),
      target_coverage: self.target_percent,
      target_met: self.target_met(),
      modules: module_reports,
    }
  }

  /// Clear all coverage data
  pub fn clear(&mut self) {
    self.modules.clear();
  }
}

impl Default for CoverageTracker {
  fn default() -> Self {
    Self::new()
  }
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
  /// Total coverage percentage
  pub total_coverage: f64,
  /// Target coverage percentage
  pub target_coverage: f64,
  /// Whether target is met
  pub target_met: bool,
  /// Per-module reports
  pub modules: Vec<ModuleReport>,
}

/// Per-module coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleReport {
  /// Module path
  pub path: String,
  /// Coverage percentage
  pub coverage_percent: f64,
  /// Total items
  pub total_items: usize,
  /// Covered items
  pub covered_items: usize,
  /// Uncovered item names
  pub uncovered_items: Vec<String>,
}

// ============================================================================
// Test Assertions (Functional)
// ============================================================================

/// Assertion result type
#[derive(Debug, Clone, PartialEq)]
pub enum AssertionResult {
  /// Assertion passed
  Passed,
  /// Assertion failed with message
  Failed(String),
}

impl AssertionResult {
  /// Check if passed
  #[must_use]
  pub const fn is_passed(&self) -> bool {
    matches!(self, Self::Passed)
  }

  /// Check if failed
  #[must_use]
  pub const fn is_failed(&self) -> bool {
    matches!(self, Self::Failed(_))
  }

  /// Get error message if failed
  #[must_use]
  pub fn error(&self) -> Option<&str> {
    match self {
      Self::Failed(msg) => Some(msg),
      Self::Passed => None,
    }
  }
}

/// Assert that a Result is Ok
#[must_use]
pub fn assert_ok<T, E: std::fmt::Debug>(result: &Result<T, E>) -> AssertionResult {
  match result {
    Ok(_) => AssertionResult::Passed,
    Err(e) => AssertionResult::Failed(format!("Expected Ok, got Err({e:?})")),
  }
}

/// Assert that a Result is Err
#[must_use]
pub fn assert_err<T: std::fmt::Debug, E>(result: &Result<T, E>) -> AssertionResult {
  match result {
    Err(_) => AssertionResult::Passed,
    Ok(v) => AssertionResult::Failed(format!("Expected Err, got Ok({v:?})")),
  }
}

/// Assert that an Option is Some
#[must_use]
pub fn assert_some<T: std::fmt::Debug>(option: &Option<T>) -> AssertionResult {
  match option {
    Some(_) => AssertionResult::Passed,
    None => AssertionResult::Failed("Expected Some, got None".to_string()),
  }
}

/// Assert that an Option is None
#[must_use]
pub fn assert_none<T: std::fmt::Debug>(option: &Option<T>) -> AssertionResult {
  match option {
    None => AssertionResult::Passed,
    Some(v) => AssertionResult::Failed(format!("Expected None, got Some({v:?})")),
  }
}

/// Assert equality
#[must_use]
pub fn assert_eq<T: std::fmt::Debug + PartialEq>(left: &T, right: &T) -> AssertionResult {
  if left == right {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(format!("Assertion failed: {left:?} != {right:?}"))
  }
}

/// Assert inequality
#[must_use]
pub fn assert_ne<T: std::fmt::Debug + PartialEq>(left: &T, right: &T) -> AssertionResult {
  if left != right {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(format!("Assertion failed: {left:?} == {right:?}"))
  }
}

/// Assert that a condition is true
#[must_use]
pub fn assert_true(condition: bool, message: impl Into<String>) -> AssertionResult {
  if condition {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(message.into())
  }
}

/// Assert that a condition is false
#[must_use]
pub fn assert_false(condition: bool, message: impl Into<String>) -> AssertionResult {
  if !condition {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(message.into())
  }
}

/// Assert that a value is within a range
#[must_use]
pub fn assert_in_range<T: std::fmt::Debug + PartialOrd>(
  value: &T,
  min: &T,
  max: &T,
) -> AssertionResult {
  if value >= min && value <= max {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(format!(
      "Assertion failed: {value:?} is not in range [{min:?}, {max:?}]"
    ))
  }
}

/// Assert that a string contains a substring
#[must_use]
pub fn assert_contains(haystack: &str, needle: &str) -> AssertionResult {
  if haystack.contains(needle) {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(format!(
      "Assertion failed: '{haystack}' does not contain '{needle}'"
    ))
  }
}

/// Assert that a collection is empty
#[must_use]
pub fn assert_empty<T: std::fmt::Debug>(collection: &[T]) -> AssertionResult {
  if collection.is_empty() {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed(format!(
      "Assertion failed: collection is not empty, has {} items",
      collection.len()
    ))
  }
}

/// Assert that a collection is not empty
#[must_use]
pub fn assert_not_empty<T: std::fmt::Debug>(collection: &[T]) -> AssertionResult {
  if !collection.is_empty() {
    AssertionResult::Passed
  } else {
    AssertionResult::Failed("Assertion failed: collection is empty".to_string())
  }
}

// ============================================================================
// Test Data Generator
// ============================================================================

/// Generator for test data
#[derive(Debug, Clone)]
pub struct TestDataGenerator {
  seed: u64,
}

impl TestDataGenerator {
  /// Create a new generator with a seed
  #[must_use]
  pub const fn new(seed: u64) -> Self {
    Self { seed }
  }

  /// Create with current time as seed
  #[must_use]
  pub fn from_time() -> Self {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |d| d.as_secs());
    Self { seed }
  }

  /// Generate a random u64
  #[must_use]
  pub fn next_u64(&mut self) -> u64 {
    // Simple xorshift64
    self.seed ^= self.seed << 13;
    self.seed ^= self.seed >> 7;
    self.seed ^= self.seed << 17;
    self.seed
  }

  /// Generate a random i64 in range
  #[must_use]
  pub fn next_i64_in_range(&mut self, min: i64, max: i64) -> i64 {
    let range = (max - min + 1) as u64;
    if range == 0 {
      return min;
    }
    min + i64::try_from(self.next_u64() % range).unwrap_or(0)
  }

  /// Generate a random f64 in range [0, 1)
  #[must_use]
  pub fn next_f64(&mut self) -> f64 {
    const MAX_SAFE_INT: f64 = 9007199254740992.0; // 2^53
    f64::from_bits(self.next_u64() >> 11) / MAX_SAFE_INT
  }

  /// Generate a random f64 in range
  #[must_use]
  pub fn next_f64_in_range(&mut self, min: f64, max: f64) -> f64 {
    min + self.next_f64() * (max - min)
  }

  /// Generate a random bool
  #[must_use]
  pub fn next_bool(&mut self) -> bool {
    self.next_u64() % 2 == 0
  }

  /// Generate a random string
  #[must_use]
  pub fn next_string(&mut self, length: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..length)
      .map(|_| {
        let idx =
          usize::try_from(self.next_u64() % u64::try_from(CHARS.len()).unwrap_or(1)).unwrap_or(0);
        char::from(CHARS[idx])
      })
      .collect()
  }

  /// Choose a random element from a slice
  #[must_use]
  pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
    if slice.is_empty() {
      return None;
    }
    let idx =
      usize::try_from(self.next_u64() % u64::try_from(slice.len()).unwrap_or(1)).unwrap_or(0);
    slice.get(idx)
  }

  /// Shuffle a vector
  pub fn shuffle<T>(&mut self, vec: &mut Vec<T>) {
    if vec.len() <= 1 {
      return;
    }
    for i in (1..vec.len()).rev() {
      let j = usize::try_from(self.next_u64() % u64::try_from(i + 1).unwrap_or(1)).unwrap_or(0);
      vec.swap(i, j);
    }
  }

  /// Get current seed
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }
}

impl Default for TestDataGenerator {
  fn default() -> Self {
    Self::from_time()
  }
}

// ============================================================================
// Test Context
// ============================================================================

/// Context for running tests
#[derive(Debug, Clone)]
pub struct TestContext {
  /// Test name
  pub name: String,
  /// Test module
  pub module: String,
  /// Start time
  pub start_time: DateTime<Utc>,
  /// Custom properties
  pub properties: HashMap<String, String>,
}

impl TestContext {
  /// Create a new test context
  #[must_use]
  pub fn new(name: impl Into<String>, module: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      module: module.into(),
      start_time: Utc::now(),
      properties: HashMap::new(),
    }
  }

  /// Add a property
  #[must_use]
  pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.properties.insert(key.into(), value.into());
    self
  }

  /// Get elapsed time in milliseconds
  #[must_use]
  pub fn elapsed_ms(&self) -> u64 {
    let elapsed = Utc::now().signed_duration_since(self.start_time);
    elapsed.num_milliseconds().try_into().unwrap_or(0)
  }

  /// Create a passing result
  #[must_use]
  pub fn pass(&self) -> TestResult {
    TestResult::passed(&self.name, &self.module, self.elapsed_ms())
  }

  /// Create a failing result
  #[must_use]
  pub fn fail(&self, error: impl Into<String>) -> TestResult {
    TestResult::failed(&self.name, &self.module, self.elapsed_ms(), error)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_test_result_passed() {
    let result = TestResult::passed("test1", "module1", 100);
    assert!(result.passed);
    assert!(result.error.is_none());
  }

  #[test]
  fn test_test_result_failed() {
    let result = TestResult::failed("test1", "module1", 50, "Something went wrong");
    assert!(!result.passed);
    assert_eq!(result.error, Some("Something went wrong".to_string()));
  }

  #[test]
  fn test_test_summary() {
    let mut summary = TestSummary::new();

    summary.add(TestResult::passed("t1", "m1", 10));
    summary.add(TestResult::passed("t2", "m1", 20));
    summary.add(TestResult::failed("t3", "m1", 5, "error"));

    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.duration_ms, 35);
  }

  #[test]
  fn test_test_summary_pass_rate() {
    let mut summary = TestSummary::new();

    summary.add(TestResult::passed("t1", "m1", 10));
    summary.add(TestResult::passed("t2", "m1", 10));
    summary.add(TestResult::failed("t3", "m1", 10, "e"));

    let rate = summary.pass_rate();
    assert!((rate - 66.66666666666666).abs() < 1.0);
  }

  #[test]
  fn test_test_summary_all_passed() {
    let mut summary = TestSummary::new();

    assert!(!summary.all_passed()); // No tests

    summary.add(TestResult::passed("t1", "m1", 10));
    assert!(summary.all_passed());

    summary.add(TestResult::failed("t2", "m1", 10, "e"));
    assert!(!summary.all_passed());
  }

  #[test]
  fn test_test_summary_failed_tests() {
    let mut summary = TestSummary::new();

    summary.add(TestResult::passed("t1", "m1", 10));
    summary.add(TestResult::failed("t2", "m1", 10, "e1"));
    summary.add(TestResult::failed("t3", "m1", 10, "e2"));

    let failed = summary.failed_tests();
    assert_eq!(failed.len(), 2);
    assert!(failed.contains(&"t2"));
    assert!(failed.contains(&"t3"));
  }

  #[test]
  fn test_fixture_creation() {
    let fixture = TestFixture::new("test_fixture");
    assert_eq!(fixture.name(), "test_fixture");
  }

  #[test]
  fn test_fixture_setup_teardown() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let setup_called = Arc::new(AtomicBool::new(false));
    let teardown_called = Arc::new(AtomicBool::new(false));

    let setup_flag = Arc::clone(&setup_called);
    let teardown_flag = Arc::clone(&teardown_called);

    let mut fixture = TestFixture::new("test")
      .with_setup(move || {
        setup_flag.store(true, Ordering::SeqCst);
        Ok(())
      })
      .with_teardown(move || {
        teardown_flag.store(true, Ordering::SeqCst);
        Ok(())
      });

    assert!(fixture.setup().is_ok());
    assert!(setup_called.load(Ordering::SeqCst));

    assert!(fixture.teardown().is_ok());
    assert!(teardown_called.load(Ordering::SeqCst));
  }

  #[test]
  fn test_coverage_item() {
    let mut item = CoverageItem::new("test_func");
    assert_eq!(item.hit_count, 0);
    assert!(!item.covered);

    item.hit();
    assert_eq!(item.hit_count, 1);
    assert!(item.covered);

    item.hit();
    assert_eq!(item.hit_count, 2);
  }

  #[test]
  fn test_module_coverage() {
    let mut module = ModuleCoverage::new("test::module");

    module.record("func1");
    module.record("func1"); // Double hit
                            // func2 is not recorded (not covered)

    assert_eq!(module.items.len(), 1);
    assert!((module.coverage_percent() - 100.0).abs() < f64::EPSILON);

    // Register but don't hit
    module.register_item("func2");
    assert!((module.coverage_percent() - 50.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_module_coverage_uncovered() {
    let mut module = ModuleCoverage::new("test");
    module.register_item("func1");
    module.register_item("func2");
    module.record("func1");

    let uncovered = module.uncovered_items();
    assert_eq!(uncovered.len(), 1);
    assert!(uncovered.contains(&"func2"));
  }

  #[test]
  fn test_coverage_tracker() {
    let mut tracker = CoverageTracker::new();

    tracker.register_item("module1", "func1");
    tracker.register_item("module1", "func2");
    tracker.register_item("module2", "func3");

    tracker.record("module1", "func1");
    tracker.record("module2", "func3");

    let coverage = tracker.total_coverage();
    assert!((coverage - 66.66666666666666).abs() < 1.0);
  }

  #[test]
  fn test_coverage_tracker_target() {
    let tracker = CoverageTracker::new().with_target_percent(80.0);
    assert!((tracker.target_percent - 80.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_coverage_tracker_report() {
    let mut tracker = CoverageTracker::new();
    tracker.register_item("module1", "func1");
    tracker.record("module1", "func1");

    let report = tracker.report();
    assert!((report.total_coverage - 100.0).abs() < f64::EPSILON);
    assert!(report.target_met);
  }

  #[test]
  fn test_coverage_tracker_clear() {
    let mut tracker = CoverageTracker::new();
    tracker.register_item("module1", "func1");
    tracker.record("module1", "func1");

    assert!(!tracker.modules.is_empty());

    tracker.clear();
    assert!(tracker.modules.is_empty());
  }

  #[test]
  fn test_assert_ok() {
    let ok: Result<i32, &str> = Ok(42);
    let err: Result<i32, &str> = Err("error");

    assert!(assert_ok(&ok).is_passed());
    assert!(assert_ok(&err).is_failed());
  }

  #[test]
  fn test_assert_err() {
    let ok: Result<i32, &str> = Ok(42);
    let err: Result<i32, &str> = Err("error");

    assert!(assert_err(&err).is_passed());
    assert!(assert_err(&ok).is_failed());
  }

  #[test]
  fn test_assert_some_none() {
    let some: Option<i32> = Some(42);
    let none: Option<i32> = None;

    assert!(assert_some(&some).is_passed());
    assert!(assert_some(&none).is_failed());
    assert!(assert_none(&none).is_passed());
    assert!(assert_none(&some).is_failed());
  }

  #[test]
  fn test_assert_eq_ne() {
    assert!(assert_eq(&1, &1).is_passed());
    assert!(assert_eq(&1, &2).is_failed());
    assert!(assert_ne(&1, &2).is_passed());
    assert!(assert_ne(&1, &1).is_failed());
  }

  #[test]
  fn test_assert_true_false() {
    assert!(assert_true(true, "should be true").is_passed());
    assert!(assert_true(false, "should be true").is_failed());
    assert!(assert_false(false, "should be false").is_passed());
    assert!(assert_false(true, "should be false").is_failed());
  }

  #[test]
  fn test_assert_in_range() {
    assert!(assert_in_range(&5, &1, &10).is_passed());
    assert!(assert_in_range(&1, &1, &10).is_passed());
    assert!(assert_in_range(&10, &1, &10).is_passed());
    assert!(assert_in_range(&0, &1, &10).is_failed());
    assert!(assert_in_range(&11, &1, &10).is_failed());
  }

  #[test]
  fn test_assert_contains() {
    assert!(assert_contains("hello world", "world").is_passed());
    assert!(assert_contains("hello world", "foo").is_failed());
  }

  #[test]
  fn test_assert_empty_not_empty() {
    let empty: Vec<i32> = Vec::new();
    let not_empty = vec![1, 2, 3];

    assert!(assert_empty(&empty).is_passed());
    assert!(assert_empty(&not_empty).is_failed());
    assert!(assert_not_empty(&not_empty).is_passed());
    assert!(assert_not_empty(&empty).is_failed());
  }

  #[test]
  fn test_assertion_result_error() {
    let passed = AssertionResult::Passed;
    let failed = AssertionResult::Failed("error message".to_string());

    assert!(passed.error().is_none());
    assert_eq!(failed.error(), Some("error message"));
  }

  #[test]
  fn test_data_generator_deterministic() {
    let mut gen1 = TestDataGenerator::new(42);
    let mut gen2 = TestDataGenerator::new(42);

    assert_eq!(gen1.next_u64(), gen2.next_u64());
    assert_eq!(gen1.next_u64(), gen2.next_u64());
  }

  #[test]
  fn test_data_generator_range() {
    let mut gen = TestDataGenerator::new(12345);

    for _ in 0..100 {
      let val = gen.next_i64_in_range(0, 10);
      assert!(val >= 0 && val <= 10);
    }
  }

  #[test]
  fn test_data_generator_f64_range() {
    let mut gen = TestDataGenerator::new(12345);

    for _ in 0..100 {
      let val = gen.next_f64();
      assert!(val >= 0.0 && val < 1.0);
    }
  }

  #[test]
  fn test_data_generator_string() {
    let mut gen = TestDataGenerator::new(42);
    let s = gen.next_string(10);

    assert_eq!(s.len(), 10);
    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_data_generator_choose() {
    let mut gen = TestDataGenerator::new(42);
    let items = vec![1, 2, 3, 4, 5];

    for _ in 0..10 {
      let choice = gen.choose(&items);
      assert!(choice.is_some());
      assert!(items.contains(choice.unwrap()));
    }
  }

  #[test]
  fn test_data_generator_choose_empty() {
    let mut gen = TestDataGenerator::new(42);
    let empty: Vec<i32> = Vec::new();

    assert!(gen.choose(&empty).is_none());
  }

  #[test]
  fn test_data_generator_shuffle() {
    let mut gen = TestDataGenerator::new(42);
    let mut vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let original = vec.clone();

    gen.shuffle(&mut vec);

    // Should have same elements
    let mut sorted_orig: Vec<_> = original.into_iter().collect();
    let mut sorted_shuffled: Vec<_> = vec.clone().into_iter().collect();
    sorted_orig.sort();
    sorted_shuffled.sort();
    assert_eq!(sorted_orig, sorted_shuffled);
  }

  #[test]
  fn test_test_context() {
    let ctx = TestContext::new("test1", "module1").with_property("key", "value");

    assert_eq!(ctx.name, "test1");
    assert_eq!(ctx.module, "module1");
    assert_eq!(ctx.properties.get("key"), Some(&"value".to_string()));
  }

  #[test]
  fn test_test_context_result() {
    let ctx = TestContext::new("test1", "module1");

    let passed = ctx.pass();
    assert!(passed.passed);
    assert_eq!(passed.name, "test1");

    let failed = ctx.fail("error message");
    assert!(!failed.passed);
    assert_eq!(failed.error, Some("error message".to_string()));
  }

  #[test]
  fn test_test_summary_merge() {
    let mut summary1 = TestSummary::new();
    summary1.add(TestResult::passed("t1", "m1", 10));

    let mut summary2 = TestSummary::new();
    summary2.add(TestResult::failed("t2", "m2", 20, "e"));

    summary1.merge(&summary2);

    assert_eq!(summary1.total, 2);
    assert_eq!(summary1.passed, 1);
    assert_eq!(summary1.failed, 1);
    assert_eq!(summary1.duration_ms, 30);
  }
}
