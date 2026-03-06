//! Validation Rule Engine (WP21)
//!
//! Provides a flexible validation rule system for checking values against criteria:
//! - Required: Value must be present and non-empty
//! - Pattern: Value must match a regex pattern
//! - Range: Numeric value must be within bounds
//! - Custom: User-defined validation logic
//!
//! ## Example
//!
//! ```ignore
//! use intent::validation::rule::{Rule, apply_rule, validate_with_rules};
//!
//! let rules = vec![
//!     Rule::Required,
//!     Rule::Pattern { pattern: r"^\d+$".into() },
//! ];
//!
//! let results = validate_with_rules("123", &rules)?;
//! assert!(results.iter().all(|r| r.passed));
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error taxonomy for rule validation
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuleError {
    /// A rule failed validation
    #[error("rule '{0}' failed: {1}")]
    RuleFailed(String, String),

    /// Pattern did not match the value
    #[error("pattern '{pattern}' did not match value '{value}'")]
    PatternMismatch {
        /// The regex pattern
        pattern: String,
        /// The value that didn't match
        value: String,
    },

    /// Value is out of specified range
    #[error("value {value} out of range [{min}, {max}]")]
    OutOfRange {
        /// The value
        value: f64,
        /// Minimum allowed value
        min: f64,
        /// Maximum allowed value
        max: f64,
    },

    /// Invalid regex pattern
    #[error("invalid regex pattern: {0}")]
    InvalidPattern(String),

    /// Value could not be parsed as a number
    #[error("not a number: {0}")]
    NotANumber(String),

    /// Custom rule check failed
    #[error("custom rule '{name}' failed: {message}")]
    CustomFailed {
        /// Rule name
        name: String,
        /// Failure message
        message: String,
    },
}

// =============================================================================
// Rule Types
// =============================================================================

/// Validation rule specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
    /// Value must be present and non-empty
    Required,

    /// Value must match the specified regex pattern
    Pattern {
        /// The regex pattern to match
        pattern: String,
    },

    /// Numeric value must be within the specified range (inclusive)
    Range {
        /// Minimum value (inclusive)
        min: f64,
        /// Maximum value (inclusive)
        max: f64,
    },

    /// Custom validation rule
    Custom {
        /// Name of the custom rule
        name: String,
        /// Check expression or description
        check: String,
    },
}

impl Rule {
    /// Get the name of this rule
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Rule::Required => "required",
            Rule::Pattern { .. } => "pattern",
            Rule::Range { .. } => "range",
            Rule::Custom { name, .. } => name,
        }
    }

    /// Create a required rule
    #[must_use]
    pub fn required() -> Self {
        Rule::Required
    }

    /// Create a pattern rule
    #[must_use]
    pub fn pattern(regex: impl Into<String>) -> Self {
        Rule::Pattern {
            pattern: regex.into(),
        }
    }

    /// Create a range rule
    #[must_use]
    pub fn range(min: f64, max: f64) -> Self {
        Rule::Range { min, max }
    }

    /// Create a custom rule
    #[must_use]
    pub fn custom(name: impl Into<String>, check: impl Into<String>) -> Self {
        Rule::Custom {
            name: name.into(),
            check: check.into(),
        }
    }
}

// =============================================================================
// Result Types
// =============================================================================

/// Result of applying a single rule
#[derive(Debug, Clone, PartialEq)]
pub struct RuleResult {
    /// Name of the rule that was applied
    pub rule_name: String,
    /// Whether the rule passed
    pub passed: bool,
    /// Optional message (usually for failures)
    pub message: Option<String>,
    /// The value that was validated
    pub value: Option<String>,
}

impl RuleResult {
    /// Create a passing result
    #[must_use]
    pub fn passed(rule_name: impl Into<String>, value: Option<String>) -> Self {
        Self {
            rule_name: rule_name.into(),
            passed: true,
            message: None,
            value,
        }
    }

    /// Create a failing result
    #[must_use]
    pub fn failed(rule_name: impl Into<String>, message: impl Into<String>, value: Option<String>) -> Self {
        Self {
            rule_name: rule_name.into(),
            passed: false,
            message: Some(message.into()),
            value,
        }
    }

    /// Check if this result passed
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        self.passed
    }

    /// Check if this result failed
    #[must_use]
    pub const fn is_fail(&self) -> bool {
        !self.passed
    }
}

// =============================================================================
// Validation Functions
// =============================================================================

/// Apply a single rule to a value
///
/// # Errors
///
/// Returns `RuleError` if the rule cannot be applied (e.g., invalid pattern)
/// or if the rule fails (for use in fallible contexts).
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::rule::{Rule, apply_rule, RuleResult};
/// let result = apply_rule(&Rule::Required, "hello")?;
/// assert!(result.passed);
///
/// let result = apply_rule(&Rule::Required, "")?;
/// assert!(!result.passed);
/// # Ok::<(), clarity_web::intent::validation::rule::RuleError>(())
/// ```
pub fn apply_rule(rule: &Rule, value: &str) -> Result<RuleResult, RuleError> {
    match rule {
        Rule::Required => validate_required(value),
        Rule::Pattern { pattern } => validate_pattern(value, pattern),
        Rule::Range { min, max } => validate_range(value, *min, *max),
        Rule::Custom { name, check } => validate_custom(value, name, check),
    }
}

/// Validate that a value is present and non-empty
fn validate_required(value: &str) -> Result<RuleResult, RuleError> {
    let trimmed = value.trim();
    let passed = !trimmed.is_empty();

    let result = if passed {
        RuleResult::passed("required", Some(value.to_string()))
    } else {
        RuleResult::failed("required", "value is required but was empty", Some(value.to_string()))
    };

    Ok(result)
}

/// Validate that a value matches a regex pattern
fn validate_pattern(value: &str, pattern: &str) -> Result<RuleResult, RuleError> {
    // We need to compile the regex without using unwrap
    let regex = regex::Regex::new(pattern)
        .map_err(|e| RuleError::InvalidPattern(format!("{}: {}", pattern, e)))?;

    let passed = regex.is_match(value);

    let result = if passed {
        RuleResult::passed("pattern", Some(value.to_string()))
    } else {
        RuleResult::failed(
            "pattern",
            format!("value '{}' does not match pattern '{}'", value, pattern),
            Some(value.to_string()),
        )
    };

    Ok(result)
}

/// Validate that a numeric value is within a range
fn validate_range(value: &str, min: f64, max: f64) -> Result<RuleResult, RuleError> {
    // Parse the value as a number
    let num: f64 = value
        .trim()
        .parse()
        .map_err(|_| RuleError::NotANumber(value.to_string()))?;

    let passed = num >= min && num <= max;

    let result = if passed {
        RuleResult::passed("range", Some(value.to_string()))
    } else {
        RuleResult::failed(
            "range",
            format!("value {} is outside range [{}, {}]", num, min, max),
            Some(value.to_string()),
        )
    };

    Ok(result)
}

/// Validate using a custom rule
///
/// Custom rules support simple check expressions:
/// - `length > N`: String length greater than N
/// - `length < N`: String length less than N
/// - `length >= N`: String length greater than or equal to N
/// - `length <= N`: String length less than or equal to N
/// - `starts_with "prefix"`: String starts with prefix
/// - `ends_with "suffix"`: String ends with suffix
/// - `contains "substr"`: String contains substring
/// - `one_of ["a", "b", "c"]`: Value is one of the listed values
fn validate_custom(value: &str, name: &str, check: &str) -> Result<RuleResult, RuleError> {
    let check_trimmed = check.trim();

    let passed = evaluate_custom_check(value, check_trimmed)?;

    let result = if passed {
        RuleResult::passed(name, Some(value.to_string()))
    } else {
        RuleResult::failed(
            name,
            format!("custom check failed: {}", check),
            Some(value.to_string()),
        )
    };

    Ok(result)
}

/// Evaluate a custom check expression
fn evaluate_custom_check(value: &str, check: &str) -> Result<bool, RuleError> {
    // Length checks
    if let Some(rest) = check.strip_prefix("length ") {
        let rest = rest.trim();
        return evaluate_length_check(value.len(), rest);
    }

    // Starts with check
    if let Some(prefix) = check.strip_prefix("starts_with ") {
        let prefix = extract_quoted_string(prefix.trim())?;
        return Ok(value.starts_with(&prefix));
    }

    // Ends with check
    if let Some(suffix) = check.strip_prefix("ends_with ") {
        let suffix = extract_quoted_string(suffix.trim())?;
        return Ok(value.ends_with(&suffix));
    }

    // Contains check
    if let Some(substr) = check.strip_prefix("contains ") {
        let substr = extract_quoted_string(substr.trim())?;
        return Ok(value.contains(&substr));
    }

    // One of check
    if let Some(list) = check.strip_prefix("one_of ") {
        return evaluate_one_of(value, list.trim());
    }

    // Unknown check type - default to false with error
    Err(RuleError::CustomFailed {
        name: "unknown".into(),
        message: format!("unknown check expression: {}", check),
    })
}

/// Evaluate a length comparison
fn evaluate_length_check(len: usize, expr: &str) -> Result<bool, RuleError> {
    let expr = expr.trim();

    let (comparison, num_str) = if let Some(rest) = expr.strip_prefix(">=") {
        (Comparison::Gte, rest.trim())
    } else if let Some(rest) = expr.strip_prefix("<=") {
        (Comparison::Lte, rest.trim())
    } else if let Some(rest) = expr.strip_prefix(">") {
        (Comparison::Gt, rest.trim())
    } else if let Some(rest) = expr.strip_prefix("<") {
        (Comparison::Lt, rest.trim())
    } else if let Some(rest) = expr.strip_prefix("==") {
        (Comparison::Eq, rest.trim())
    } else if let Some(rest) = expr.strip_prefix("!=") {
        (Comparison::Ne, rest.trim())
    } else {
        return Err(RuleError::CustomFailed {
            name: "length".into(),
            message: format!("invalid comparison: {}", expr),
        });
    };

    let target: usize = num_str
        .parse()
        .map_err(|_| RuleError::CustomFailed {
            name: "length".into(),
            message: format!("not a valid number: {}", num_str),
        })?;

    let result = match comparison {
        Comparison::Gt => len > target,
        Comparison::Lt => len < target,
        Comparison::Gte => len >= target,
        Comparison::Lte => len <= target,
        Comparison::Eq => len == target,
        Comparison::Ne => len != target,
    };

    Ok(result)
}

#[derive(Debug, Clone, Copy)]
enum Comparison {
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Ne,
}

/// Extract a quoted string from an expression
fn extract_quoted_string(s: &str) -> Result<String, RuleError> {
    let s = s.trim();

    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Ok(s[1..s.len() - 1].to_string());
    }

    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return Ok(s[1..s.len() - 1].to_string());
    }

    // If not quoted, return as-is
    Ok(s.to_string())
}

/// Evaluate one_of check
fn evaluate_one_of(value: &str, list: &str) -> Result<bool, RuleError> {
    // Parse JSON-like array: ["a", "b", "c"]
    let list = list.trim();

    if !list.starts_with('[') || !list.ends_with(']') {
        return Err(RuleError::CustomFailed {
            name: "one_of".into(),
            message: format!("expected array format: {}", list),
        });
    }

    let inner = &list[1..list.len() - 1];

    // Simple parsing: split by comma and clean up quotes
    let values: Vec<String> = inner
        .split(',')
        .map(|s| {
            let s = s.trim();
            // Remove surrounding quotes
            if (s.starts_with('"') && s.ends_with('"'))
                || (s.starts_with('\'') && s.ends_with('\''))
            {
                s[1..s.len() - 1].to_string()
            } else {
                s.to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    Ok(values.iter().any(|v| v == value))
}

/// Validate a value against multiple rules
///
/// Returns all rule results, allowing you to see which rules passed and failed.
///
/// # Errors
///
/// Returns `RuleError` if any rule cannot be applied (e.g., invalid pattern).
/// Note: Rule failures (value doesn't match) are reported in results, not as errors.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::rule::{Rule, validate_with_rules, RuleResult};
/// let rules = vec![
///     Rule::Required,
///     Rule::range(0.0, 100.0),
/// ];
///
/// let results = validate_with_rules("50", &rules)?;
/// assert!(results.iter().all(|r| r.passed));
///
/// let results = validate_with_rules("150", &rules)?;
/// assert!(results.iter().any(|r| !r.passed));
/// # Ok::<(), clarity_web::intent::validation::rule::RuleError>(())
/// ```
pub fn validate_with_rules(value: &str, rules: &[Rule]) -> Result<Vec<RuleResult>, RuleError> {
    rules
        .iter()
        .map(|rule| apply_rule(rule, value))
        .try_collect()
}

/// Check if all rules pass for a value
///
/// # Errors
///
/// Returns `RuleError` if any rule cannot be applied.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::rule::{Rule, all_rules_pass};
/// let rules = vec![Rule::Required, Rule::pattern(r"^\d+$")];
///
/// assert!(all_rules_pass("123", &rules)?);
/// assert!(!all_rules_pass("abc", &rules)?);
/// # Ok::<(), clarity_web::intent::validation::rule::RuleError>(())
/// ```
pub fn all_rules_pass(value: &str, rules: &[Rule]) -> Result<bool, RuleError> {
    let results = validate_with_rules(value, rules)?;
    Ok(results.iter().all(|r| r.passed))
}

/// Get all failing rules for a value
///
/// # Errors
///
/// Returns `RuleError` if any rule cannot be applied.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::rule::{Rule, failing_rules};
/// let rules = vec![
///     Rule::Required,
///     Rule::range(0.0, 10.0),
/// ];
///
/// let failures = failing_rules("20", &rules)?;
/// assert_eq!(failures.len(), 1);
/// assert_eq!(failures[0].rule_name, "range");
/// # Ok::<(), clarity_web::intent::validation::rule::RuleError>(())
/// ```
pub fn failing_rules(value: &str, rules: &[Rule]) -> Result<Vec<RuleResult>, RuleError> {
    let results = validate_with_rules(value, rules)?;
    Ok(results.into_iter().filter(|r| !r.passed).collect())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Rule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rule_name() {
        assert_eq!(Rule::Required.name(), "required");
        assert_eq!(Rule::pattern(r"\d+").name(), "pattern");
        assert_eq!(Rule::range(0.0, 10.0).name(), "range");
        assert_eq!(Rule::custom("my_rule", "check").name(), "my_rule");
    }

    #[test]
    fn test_rule_constructors() {
        let r1 = Rule::required();
        assert_eq!(r1, Rule::Required);

        let r2 = Rule::pattern(r"\d+");
        assert!(matches!(r2, Rule::Pattern { .. }));

        let r3 = Rule::range(0.0, 100.0);
        assert!(matches!(r3, Rule::Range { .. }));

        let r4 = Rule::custom("test", "length > 5");
        assert!(matches!(r4, Rule::Custom { .. }));
    }

    #[test]
    fn test_rule_serialization() {
        let rule = Rule::Required;
        let json = serde_json::to_string(&rule).expect("should serialize");
        assert!(json.contains("required"));

        let rule = Rule::pattern(r"\d+");
        let json = serde_json::to_string(&rule).expect("should serialize");
        assert!(json.contains("pattern"));

        let rule = Rule::range(0.0, 10.0);
        let json = serde_json::to_string(&rule).expect("should serialize");
        assert!(json.contains("range"));
    }

    // -------------------------------------------------------------------------
    // RuleResult Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rule_result_passed() {
        let result = RuleResult::passed("test", Some("value".into()));
        assert!(result.passed);
        assert!(result.message.is_none());
        assert_eq!(result.value, Some("value".into()));
        assert!(result.is_pass());
        assert!(!result.is_fail());
    }

    #[test]
    fn test_rule_result_failed() {
        let result = RuleResult::failed("test", "error message", Some("value".into()));
        assert!(!result.passed);
        assert_eq!(result.message, Some("error message".into()));
        assert!(!result.is_pass());
        assert!(result.is_fail());
    }

    // -------------------------------------------------------------------------
    // Required Rule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_required_with_value() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::Required, "hello")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_required_with_whitespace() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::Required, "   hello   ")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_required_empty() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::Required, "")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_required_whitespace_only() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::Required, "   ")?;
        assert!(!result.passed);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Pattern Rule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_pattern_match() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::pattern(r"^\d+$"), "123")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_pattern_no_match() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::pattern(r"^\d+$"), "abc")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_pattern_email() -> Result<(), RuleError> {
        let rule = Rule::pattern(r"^[^@\s]+@[^@\s]+\.[^@\s]+$");
        let result = apply_rule(&rule, "test@example.com")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "invalid-email")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_pattern_invalid_regex() {
        let result = apply_rule(&Rule::pattern(r"[invalid"), "test");
        assert!(matches!(result, Err(RuleError::InvalidPattern(_))));
    }

    // -------------------------------------------------------------------------
    // Range Rule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_range_in_bounds() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 100.0), "50")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_range_at_min() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 100.0), "0")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_range_at_max() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 100.0), "100")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_range_below_min() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 100.0), "-1")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_range_above_max() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 100.0), "101")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_range_not_a_number() {
        let result = apply_rule(&Rule::range(0.0, 100.0), "not a number");
        assert!(matches!(result, Err(RuleError::NotANumber(_))));
    }

    #[test]
    fn test_range_decimal() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(0.0, 1.0), "0.5")?;
        assert!(result.passed);

        let result = apply_rule(&Rule::range(0.0, 1.0), "1.5")?;
        assert!(!result.passed);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Custom Rule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_custom_length_greater_than() -> Result<(), RuleError> {
        let rule = Rule::custom("length_check", "length > 5");
        let result = apply_rule(&rule, "hello world")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "hi")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_length_less_than() -> Result<(), RuleError> {
        let rule = Rule::custom("length_check", "length < 10");
        let result = apply_rule(&rule, "short")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "this is a very long string")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_length_gte() -> Result<(), RuleError> {
        let rule = Rule::custom("length_check", "length >= 3");
        let result = apply_rule(&rule, "abc")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "ab")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_length_lte() -> Result<(), RuleError> {
        let rule = Rule::custom("length_check", "length <= 5");
        let result = apply_rule(&rule, "abc")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "abcdef")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_starts_with() -> Result<(), RuleError> {
        let rule = Rule::custom("prefix_check", r#"starts_with "hello""#);
        let result = apply_rule(&rule, "hello world")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "goodbye")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_ends_with() -> Result<(), RuleError> {
        let rule = Rule::custom("suffix_check", r#"ends_with ".com""#);
        let result = apply_rule(&rule, "example.com")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "example.org")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_contains() -> Result<(), RuleError> {
        let rule = Rule::custom("contains_check", r"contains @");
        let result = apply_rule(&rule, "test@example.com")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "test.example.com")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_one_of() -> Result<(), RuleError> {
        let rule = Rule::custom("enum_check", r#"one_of ["red", "green", "blue"]"#);
        let result = apply_rule(&rule, "red")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "yellow")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_unknown_check() {
        let rule = Rule::custom("unknown", "something_unknown");
        let result = apply_rule(&rule, "test");
        assert!(matches!(result, Err(RuleError::CustomFailed { .. })));
    }

    // -------------------------------------------------------------------------
    // validate_with_rules Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_with_rules_all_pass() -> Result<(), RuleError> {
        let rules = vec![
            Rule::Required,
            Rule::pattern(r"^\d+$"),
            Rule::range(0.0, 100.0),
        ];

        let results = validate_with_rules("50", &rules)?;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.passed));
        Ok(())
    }

    #[test]
    fn test_validate_with_rules_some_fail() -> Result<(), RuleError> {
        let rules = vec![
            Rule::Required,
            Rule::pattern(r"^\d+$"),
        ];

        let results = validate_with_rules("abc", &rules)?;
        assert_eq!(results.len(), 2);

        let passed: Vec<_> = results.iter().filter(|r| r.passed).collect();
        let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();

        assert_eq!(passed.len(), 1); // Required passes (non-empty)
        assert_eq!(failed.len(), 1); // Pattern fails
        Ok(())
    }

    #[test]
    fn test_validate_with_rules_empty() -> Result<(), RuleError> {
        let rules: Vec<Rule> = vec![];
        let results = validate_with_rules("test", &rules)?;
        assert!(results.is_empty());
        Ok(())
    }

    // -------------------------------------------------------------------------
    // all_rules_pass Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_rules_pass_true() -> Result<(), RuleError> {
        let rules = vec![Rule::Required, Rule::pattern(r"^\w+$")];
        assert!(all_rules_pass("hello", &rules)?);
        Ok(())
    }

    #[test]
    fn test_all_rules_pass_false() -> Result<(), RuleError> {
        let rules = vec![Rule::Required, Rule::pattern(r"^\d+$")];
        assert!(!all_rules_pass("hello", &rules)?);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // failing_rules Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_failing_rules_none() -> Result<(), RuleError> {
        let rules = vec![Rule::Required];
        let failures = failing_rules("test", &rules)?;
        assert!(failures.is_empty());
        Ok(())
    }

    #[test]
    fn test_failing_rules_some() -> Result<(), RuleError> {
        let rules = vec![
            Rule::Required,
            Rule::range(0.0, 10.0),
        ];
        let failures = failing_rules("100", &rules)?;
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].rule_name, "range");
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Error Display Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_display() {
        let err = RuleError::RuleFailed("test".into(), "failed".into());
        assert!(format!("{}", err).contains("test"));

        let err = RuleError::PatternMismatch {
            pattern: r"\d+".into(),
            value: "abc".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains(r"\d+"));
        assert!(msg.contains("abc"));

        let err = RuleError::OutOfRange {
            value: 150.0,
            min: 0.0,
            max: 100.0,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("150"));
        assert!(msg.contains("0"));
        assert!(msg.contains("100"));
    }

    // -------------------------------------------------------------------------
    // Edge Cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_range_negative_numbers() -> Result<(), RuleError> {
        let result = apply_rule(&Rule::range(-100.0, 0.0), "-50")?;
        assert!(result.passed);

        let result = apply_rule(&Rule::range(-100.0, 0.0), "50")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_length_equals() -> Result<(), RuleError> {
        let rule = Rule::custom("exact_length", "length == 5");
        let result = apply_rule(&rule, "hello")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "hi")?;
        assert!(!result.passed);
        Ok(())
    }

    #[test]
    fn test_custom_length_not_equals() -> Result<(), RuleError> {
        let rule = Rule::custom("not_five", "length != 5");
        let result = apply_rule(&rule, "hello")?;
        assert!(!result.passed);

        let result = apply_rule(&rule, "hi")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_pattern_case_insensitive() -> Result<(), RuleError> {
        // Using (?i) flag for case insensitive
        let rule = Rule::pattern(r"(?i)^hello$");
        let result = apply_rule(&rule, "HELLO")?;
        assert!(result.passed);

        let result = apply_rule(&rule, "Hello")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_one_of_with_single_quotes() -> Result<(), RuleError> {
        let rule = Rule::custom("check", "one_of ['a', 'b', 'c']");
        let result = apply_rule(&rule, "a")?;
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_one_of_empty_list() -> Result<(), RuleError> {
        let rule = Rule::custom("check", "one_of []");
        let result = apply_rule(&rule, "anything")?;
        assert!(!result.passed);
        Ok(())
    }
}
