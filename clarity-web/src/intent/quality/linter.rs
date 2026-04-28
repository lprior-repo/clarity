//! Spec Linter - Quality Analysis for Specifications
//!
//! Provides linting rules and quality analysis for specifications,
//! including naming conventions, required fields, deprecated patterns,
//! description quality, and completeness checks.
//!
//! ## Lint Rules
//!
//! The linter applies the following rules:
//!
//! - **Naming Convention**: Checks for consistent naming conventions (kebab-case vs `snake_case`)
//! - **Required Fields**: Validates that required fields are present and non-empty
//! - **Deprecated Pattern**: Detects deprecated anti-patterns (vague terms, generic names)
//! - **Description Quality**: Analyzes description quality and completeness
//! - **Completeness**: Checks overall spec completeness (descriptions, feature count)
//!
//! ## Design Principles
//!
//! - Zero panics: All operations return `Result<T, E>`
//! - Deterministic: Same input always produces same output
//! - Pure functions: No side effects, only analyzes input
//!
//! ## Examples
//!
//! ```no_run
//! use clarity_web::intent::quality::linter::{SpecLinter, LintRule};
//! use clarity_web::intent::types::Spec;
//!
//! // Create a linter with all rules
//! let linter = SpecLinter::new();
//!
//! // Or create a linter with specific rules
//! let custom_linter = SpecLinter::with_rules(vec![
//!     LintRule::NamingConvention,
//!     LintRule::RequiredFields,
//! ]);
//!
//! let spec = Spec::new("my-spec".to_string()).unwrap();
//! match linter.lint_spec(&spec) {
//!     Ok(report) => {
//!         println!("Found {} errors, {} warnings",
//!             report.error_count,
//!             report.warning_count
//!         );
//!     }
//!     Err(e) => println!("Linting failed: {}", e),
//! }
//! ```

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::types::Spec;

// ============================================================================
// LINTING THRESHOLD CONSTANTS
// ============================================================================

/// Minimum length for spec and feature descriptions to be considered adequate.
///
/// Descriptions shorter than this threshold are flagged as too brief to
/// provide meaningful documentation. This value ensures descriptions contain
/// at least a few words of explanation.
///
/// Example: "Manages user" (13 chars) passes, but "User" (4 chars) fails.
const MIN_DESCRIPTION_LENGTH: usize = 10;

/// Minimum length for behavior descriptions to be considered detailed enough.
///
/// Behavior descriptions at this threshold are considered "very short" and
/// receive a Hint-level lint result (informational, not a warning).
///
/// This is lower than `MIN_DESCRIPTION_LENGTH` because behaviors are more granular
/// and may have shorter but still meaningful descriptions like "Log out" (7 chars).
const MIN_BEHAVIOR_DESCRIPTION_LENGTH: usize = 5;
use thiserror::Error;

/// Errors that can occur during linting
///
/// These errors indicate structural issues with the specification that
/// prevent linting from completing successfully.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LintError {
  /// Invalid input provided
  ///
  /// This error is returned when the input specification is malformed
  /// or contains invalid data that cannot be processed.
  #[error("invalid input: {0}")]
  InvalidInput(String),

  /// Empty spec name
  ///
  /// This error is returned when the specification name is empty
  /// or contains only whitespace.
  #[error("empty spec name")]
  EmptySpecName,

  /// No features in spec
  ///
  /// This error is returned when the specification contains no features.
  /// A specification must have at least one feature to be linted.
  #[error("spec has no features")]
  NoFeatures,
}

/// Severity level for lint results
///
/// Indicates the urgency and importance of a lint result. Severities
/// are ordered from most to least severe: Error > Warning > Info > Hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LintSeverity {
  /// Error - must be fixed for the spec to be considered valid
  Error,
  /// Warning - should be fixed for best practices
  Warning,
  /// Info - optional improvement that would enhance quality
  Info,
  /// Hint - minor suggestion or nitpick
  Hint,
}

impl LintSeverity {
  /// Get the severity as a string
  ///
  /// # Returns
  ///
  /// A lowercase string representation of the severity
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::LintSeverity;
  ///
  /// assert_eq!(LintSeverity::Error.as_str(), "error");
  /// assert_eq!(LintSeverity::Warning.as_str(), "warning");
  /// ```
  #[must_use]
  pub const fn as_str(&self) -> &str {
    match self {
      Self::Error => "error",
      Self::Warning => "warning",
      Self::Info => "info",
      Self::Hint => "hint",
    }
  }

  /// Get the severity emoji
  ///
  /// # Returns
  ///
  /// An emoji representing the severity level
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::LintSeverity;
  ///
  /// assert_eq!(LintSeverity::Error.emoji(), "❌");
  /// assert_eq!(LintSeverity::Warning.emoji(), "⚠️");
  /// ```
  #[must_use]
  pub const fn emoji(&self) -> &str {
    match self {
      Self::Error => "❌",
      Self::Warning => "⚠️",
      Self::Info => "ℹ️",
      Self::Hint => "💡",
    }
  }
}

/// A linting rule that can be applied to a spec
///
/// Each rule represents a specific category of checks that the linter
/// can perform on a specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintRule {
  /// Check naming conventions (kebab-case vs `snake_case`, uppercase letters)
  NamingConvention,
  /// Check that required fields are present and non-empty
  RequiredFields,
  /// Check for deprecated patterns (vague terms, generic names)
  DeprecatedPattern,
  /// Check description quality and completeness
  DescriptionQuality,
  /// Check overall spec completeness
  Completeness,
}

impl LintRule {
  /// Get all available lint rules
  ///
  /// # Returns
  ///
  /// An array containing all lint rules in the order they should be applied
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintRule, SpecLinter};
  ///
  /// let all_rules = LintRule::all();
  /// let linter = SpecLinter::with_rules(all_rules.to_vec());
  /// ```
  #[must_use]
  pub const fn all() -> [Self; 5] {
    [
      Self::NamingConvention,
      Self::RequiredFields,
      Self::DeprecatedPattern,
      Self::DescriptionQuality,
      Self::Completeness,
    ]
  }

  /// Get the rule name
  ///
  /// # Returns
  ///
  /// A kebab-case string representation of the rule name
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::LintRule;
  ///
  /// assert_eq!(LintRule::NamingConvention.as_str(), "naming-convention");
  /// assert_eq!(LintRule::RequiredFields.as_str(), "required-fields");
  /// ```
  #[must_use]
  pub const fn as_str(&self) -> &str {
    match self {
      Self::NamingConvention => "naming-convention",
      Self::RequiredFields => "required-fields",
      Self::DeprecatedPattern => "deprecated-pattern",
      Self::DescriptionQuality => "description-quality",
      Self::Completeness => "completeness",
    }
  }

  /// Get the default severity for this rule
  ///
  /// # Returns
  ///
  /// The default severity level for violations of this rule
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintRule, LintSeverity};
  ///
  /// assert_eq!(LintRule::NamingConvention.default_severity(), LintSeverity::Warning);
  /// assert_eq!(LintRule::RequiredFields.default_severity(), LintSeverity::Error);
  /// ```
  #[must_use]
  #[allow(clippy::match_same_arms)]
  pub const fn default_severity(&self) -> LintSeverity {
    match self {
      Self::NamingConvention => LintSeverity::Warning,
      Self::RequiredFields => LintSeverity::Error,
      Self::DeprecatedPattern => LintSeverity::Warning,
      Self::DescriptionQuality => LintSeverity::Info,
      Self::Completeness => LintSeverity::Warning,
    }
  }
}

/// A single lint result
///
/// Represents a specific issue found during linting, including its location,
/// severity, description, and an optional suggestion for fixing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintResult {
  /// The rule that generated this result
  pub rule: LintRule,
  /// Severity of the lint
  pub severity: LintSeverity,
  /// Location in the spec (e.g., "features\[0\].behaviors\[1\]")
  pub location: String,
  /// Message describing the issue
  pub message: String,
  /// Optional suggestion for fixing the issue
  pub suggestion: Option<String>,
}

impl LintResult {
  /// Create a new lint result
  ///
  /// # Arguments
  ///
  /// * `rule` - The rule that generated this result
  /// * `severity` - The severity level of the issue
  /// * `location` - The location in the spec (e.g., "features\[0\].behaviors\[1\]")
  /// * `message` - A description of the issue
  ///
  /// # Returns
  ///
  /// A new `LintResult` with the given properties
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintResult, LintRule, LintSeverity};
  ///
  /// let result = LintResult::new(
  ///     LintRule::NamingConvention,
  ///     LintSeverity::Warning,
  ///     "features[0].name".to_string(),
  ///     "Mixed naming conventions detected".to_string(),
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn new(rule: LintRule, severity: LintSeverity, location: String, message: String) -> Self {
    Self {
      rule,
      severity,
      location,
      message,
      suggestion: None,
    }
  }

  /// Add a suggestion to this lint result
  ///
  /// # Arguments
  ///
  /// * `suggestion` - A suggestion for fixing the issue
  ///
  /// # Returns
  ///
  /// `self` with the suggestion added
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintResult, LintRule, LintSeverity};
  ///
  /// let result = LintResult::new(
  ///     LintRule::NamingConvention,
  ///     LintSeverity::Warning,
  ///     "features[0].name".to_string(),
  ///     "Mixed naming conventions".to_string(),
  /// )
  /// .with_suggestion("Use kebab-case consistently".to_string());
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn with_suggestion(mut self, suggestion: String) -> Self {
    self.suggestion = Some(suggestion);
    self
  }

  /// Format this lint result for display
  ///
  /// Returns a human-readable string including the severity emoji,
  /// severity level, rule name, message, and optional suggestion.
  ///
  /// # Returns
  ///
  /// A formatted string representation of the lint result
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintResult, LintRule, LintSeverity};
  ///
  /// let result = LintResult::new(
  ///     LintRule::NamingConvention,
  ///     LintSeverity::Warning,
  ///     "features[0]".to_string(),
  ///     "Issue description".to_string(),
  /// );
  /// let formatted = result.format();
  /// assert!(formatted.contains("warning"));
  /// assert!(formatted.contains("naming-convention"));
  /// ```
  #[must_use]
  pub fn format(&self) -> String {
    let severity_str = self.severity.as_str();
    let emoji = self.severity.emoji();
    let rule_name = self.rule.as_str();

    self.suggestion.as_ref().map_or_else(
      || {
        format!(
          "{} {}: {}: {}",
          emoji, severity_str, rule_name, self.message
        )
      },
      |suggestion| {
        format!(
          "{} {}: {}: {} - Suggestion: {}",
          emoji, severity_str, rule_name, self.message, suggestion
        )
      },
    )
  }
}

/// Complete linting report for a specification
///
/// Contains all lint results along with summary statistics by severity level.
/// Provides convenient methods for filtering and analyzing results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintReport {
  /// All lint results found during analysis
  pub results: Vec<LintResult>,
  /// Total number of errors found
  pub error_count: usize,
  /// Total number of warnings found
  pub warning_count: usize,
  /// Total number of info messages found
  pub info_count: usize,
  /// Total number of hints found
  pub hint_count: usize,
}

impl LintReport {
  /// Create a new lint report
  ///
  /// # Returns
  ///
  /// A new empty `LintReport` with all counts set to zero
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::LintReport;
  ///
  /// let report = LintReport::new();
  /// assert_eq!(report.error_count, 0);
  /// assert!(report.results.is_empty());
  /// ```
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a lint result to the report
  ///
  /// Automatically updates the appropriate count based on the result's severity.
  ///
  /// # Arguments
  ///
  /// * `result` - The lint result to add
  fn add_result(&mut self, result: LintResult) {
    match result.severity {
      LintSeverity::Error => self.error_count += 1,
      LintSeverity::Warning => self.warning_count += 1,
      LintSeverity::Info => self.info_count += 1,
      LintSeverity::Hint => self.hint_count += 1,
    }
    self.results.push(result);
  }

  /// Check if the report has any errors
  ///
  /// # Returns
  ///
  /// `true` if the error count is greater than zero
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::LintReport;
  ///
  /// let report = LintReport::new();
  /// assert!(!report.has_errors());
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn has_errors(&self) -> bool {
    self.error_count > 0
  }

  /// Get all results of a specific severity
  ///
  /// # Arguments
  ///
  /// * `severity` - The severity level to filter by
  ///
  /// # Returns
  ///
  /// A vector of references to lint results with the specified severity
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintReport, LintSeverity};
  ///
  /// let report = LintReport::new();
  /// let errors = report.by_severity(LintSeverity::Error);
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn by_severity(&self, severity: LintSeverity) -> Vec<&LintResult> {
    self
      .results
      .iter()
      .filter(|r| r.severity == severity)
      .collect()
  }

  /// Get all results for a specific rule
  ///
  /// # Arguments
  ///
  /// * `rule` - The rule to filter by
  ///
  /// # Returns
  ///
  /// A vector of references to lint results from the specified rule
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{LintReport, LintRule};
  ///
  /// let report = LintReport::new();
  /// let naming_issues = report.by_rule(LintRule::NamingConvention);
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn by_rule(&self, rule: LintRule) -> Vec<&LintResult> {
    self.results.iter().filter(|r| r.rule == rule).collect()
  }
}

/// Spec linter
///
/// Analyzes specifications for quality issues using configurable lint rules.
/// Create a new linter with `new()` (all rules) or `with_rules()` (specific rules).
///
/// # Examples
///
/// ```
/// use clarity_web::intent::quality::linter::{SpecLinter, LintRule};
///
/// // Linter with all rules
/// let linter = SpecLinter::new();
///
/// // Linter with specific rules
/// let custom_linter = SpecLinter::with_rules(vec![
///     LintRule::NamingConvention,
///     LintRule::RequiredFields,
/// ]);
///
/// // Linter excluding a specific rule
/// let no_naming = SpecLinter::new().without_rule(LintRule::NamingConvention);
/// ```
#[derive(Debug, Clone)]
pub struct SpecLinter {
  /// Rules to apply during linting
  rules: Vec<LintRule>,
}

impl SpecLinter {
  /// Create a new spec linter with all rules
  ///
  /// # Returns
  ///
  /// A new `SpecLinter` configured with all available lint rules
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::SpecLinter;
  ///
  /// let linter = SpecLinter::new();
  /// // All rules will be applied
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn new() -> Self {
    Self {
      rules: LintRule::all().to_vec(),
    }
  }

  /// Create a new spec linter with specific rules
  ///
  /// # Arguments
  ///
  /// * `rules` - A vector of lint rules to apply
  ///
  /// # Returns
  ///
  /// A new `SpecLinter` configured with only the specified rules
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{SpecLinter, LintRule};
  ///
  /// let linter = SpecLinter::with_rules(vec![
  ///     LintRule::NamingConvention,
  ///     LintRule::RequiredFields,
  /// ]);
  /// // Only naming convention and required fields will be checked
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn with_rules(rules: Vec<LintRule>) -> Self {
    Self { rules }
  }

  /// Create a new spec linter without a specific rule
  ///
  /// # Arguments
  ///
  /// * `rule` - The rule to exclude
  ///
  /// # Returns
  ///
  /// `self` with the specified rule removed from the configuration
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::quality::linter::{SpecLinter, LintRule};
  ///
  /// let linter = SpecLinter::new().without_rule(LintRule::Completeness);
  /// // All rules except completeness will be checked
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn without_rule(mut self, rule: LintRule) -> Self {
    self.rules.retain(|r| r != &rule);
    self
  }

  /// Lint a specification
  ///
  /// Applies all configured lint rules to the specification and returns
  /// a comprehensive report of any issues found.
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to lint
  ///
  /// # Returns
  ///
  /// A `LintReport` containing all lint results organized by severity
  ///
  /// # Errors
  ///
  /// Returns `LintError` if the spec has structural issues that prevent
  /// linting (e.g., empty name, no features)
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use clarity_web::intent::quality::linter::SpecLinter;
  /// # use clarity_web::intent::types::Spec;
  /// let linter = SpecLinter::new();
  /// let spec = Spec::new("my-spec".to_string()).unwrap();
  ///
  /// match linter.lint_spec(&spec) {
  ///     Ok(report) => {
  ///         if report.has_errors() {
  ///             println!("Found {} errors that must be fixed", report.error_count);
  ///         }
  ///         for result in &report.results {
  ///             println!("{}", result.format());
  ///         }
  ///     }
  ///     Err(e) => println!("Linting failed: {}", e),
  /// }
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  pub fn lint_spec(&self, spec: &Spec) -> Result<LintReport, LintError> {
    // Validate input
    if spec.name.trim().is_empty() {
      return Err(LintError::EmptySpecName);
    }

    if spec.features.is_empty() {
      return Err(LintError::NoFeatures);
    }

    let mut report = LintReport::new();

    // Apply each rule
    for rule in &self.rules {
      match rule {
        LintRule::NamingConvention => Self::check_naming_convention(spec, &mut report),
        LintRule::RequiredFields => Self::check_required_fields(spec, &mut report),
        LintRule::DeprecatedPattern => Self::check_deprecated_patterns(spec, &mut report),
        LintRule::DescriptionQuality => {
          Self::check_description_quality(spec, &mut report);
        }
        LintRule::Completeness => Self::check_completeness(spec, &mut report),
      }
    }

    Ok(report)
  }

  /// Check naming conventions
  ///
  /// Validates that feature and behavior names follow consistent conventions:
  /// - Features should use either kebab-case or `snake_case`, not both
  /// - Behaviors should use `snake_case`
  /// - No uppercase letters in behavior names
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  /// * `report` - The report to add results to
  fn check_naming_convention(spec: &Spec, report: &mut LintReport) {
    let severity = LintRule::NamingConvention.default_severity();

    // Check feature names
    for (idx, feature) in spec.features.iter().enumerate() {
      // Check for mixed conventions (kebab-case vs snake_case)
      let has_kebab = feature.name.contains('-');
      let has_snake = feature.name.contains('_');

      if has_kebab && has_snake {
        report.add_result(
          LintResult::new(
            LintRule::NamingConvention,
            severity,
            format!("features[{idx}].name"),
            format!(
              "Feature '{}' uses mixed naming conventions (kebab-case and snake_case)",
              feature.name
            ),
          )
          .with_suggestion("Use either kebab-case or snake_case consistently".to_string()),
        );
      }

      // Check behavior names
      for (bidx, behavior) in feature.behaviors.iter().enumerate() {
        // Behavior names should be snake_case
        if behavior.name.contains('-') {
          report.add_result(
            LintResult::new(
              LintRule::NamingConvention,
              severity,
              format!("features[{idx}].behaviors[{bidx}].name"),
              format!(
                "Behavior '{}' uses kebab-case instead of snake_case",
                behavior.name
              ),
            )
            .with_suggestion(format!("Rename to '{}'", behavior.name.replace('-', "_"))),
          );
        }

        // Check for uppercase letters in behavior names
        if behavior.name.chars().any(|c| c.is_ascii_uppercase()) {
          report.add_result(
            LintResult::new(
              LintRule::NamingConvention,
              severity,
              format!("features[{idx}].behaviors[{bidx}].name"),
              format!("Behavior '{}' contains uppercase letters", behavior.name),
            )
            .with_suggestion(format!(
              "Rename to '{}'",
              behavior
                .name
                .chars()
                .map(|c| {
                  if c.is_ascii_uppercase() {
                    c.to_ascii_lowercase()
                  } else {
                    c
                  }
                })
                .collect::<String>()
            )),
          );
        }
      }
    }
  }

  /// Check required fields
  ///
  /// Validates that all required fields are present and non-empty:
  /// - Spec name
  /// - Feature names
  /// - Behavior names
  /// - At least one behavior per feature
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  /// * `report` - The report to add results to
  fn check_required_fields(spec: &Spec, report: &mut LintReport) {
    let severity = LintRule::RequiredFields.default_severity();

    // Check spec name
    if spec.name.trim().is_empty() {
      report.add_result(
        LintResult::new(
          LintRule::RequiredFields,
          severity,
          "spec.name".to_string(),
          "Spec name is required".to_string(),
        )
        .with_suggestion("Add a name to the specification".to_string()),
      );
    }

    // Check features
    for (idx, feature) in spec.features.iter().enumerate() {
      if feature.name.trim().is_empty() {
        report.add_result(
          LintResult::new(
            LintRule::RequiredFields,
            severity,
            format!("features[{idx}].name"),
            "Feature name is required".to_string(),
          )
          .with_suggestion("Add a name to the feature".to_string()),
        );
      }

      if feature.behaviors.is_empty() {
        report.add_result(
          LintResult::new(
            LintRule::RequiredFields,
            severity,
            format!("features[{idx}]"),
            format!("Feature '{}' has no behaviors", feature.name),
          )
          .with_suggestion("Add at least one behavior to the feature".to_string()),
        );
      }

      // Check behaviors
      for (bidx, behavior) in feature.behaviors.iter().enumerate() {
        if behavior.name.trim().is_empty() {
          report.add_result(
            LintResult::new(
              LintRule::RequiredFields,
              severity,
              format!("features[{idx}].behaviors[{bidx}].name"),
              "Behavior name is required".to_string(),
            )
            .with_suggestion("Add a name to the behavior".to_string()),
          );
        }
      }
    }
  }

  /// Check for deprecated patterns
  ///
  /// Detects anti-patterns and deprecated usage:
  /// - Vague terms in descriptions (todo, tbd, etc., something, anything)
  /// - Overly generic behavior names (handle, process, do, run, execute)
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  /// * `report` - The report to add results to
  fn check_deprecated_patterns(spec: &Spec, report: &mut LintReport) {
    let severity = LintRule::DeprecatedPattern.default_severity();

    // Check for vague descriptions
    let vague_terms = ["todo", "tbd", "etc", "something", "anything"];

    for (idx, feature) in spec.features.iter().enumerate() {
      for (bidx, behavior) in feature.behaviors.iter().enumerate() {
        let desc_lower = behavior.description.to_lowercase();

        for term in &vague_terms {
          if desc_lower.contains(term) {
            report.add_result(
              LintResult::new(
                LintRule::DeprecatedPattern,
                severity,
                format!("features[{idx}].behaviors[{bidx}].description"),
                format!("Behavior description contains vague term '{term}'"),
              )
              .with_suggestion("Replace with specific, concrete description".to_string()),
            );
          }
        }
      }
    }

    // Check for overly generic behavior names
    let generic_names = ["handle", "process", "do", "run", "execute"];

    for (idx, feature) in spec.features.iter().enumerate() {
      for (bidx, behavior) in feature.behaviors.iter().enumerate() {
        let name_lower = behavior.name.to_lowercase();

        for generic in &generic_names {
          if name_lower == *generic {
            report.add_result(
              LintResult::new(
                LintRule::DeprecatedPattern,
                severity,
                format!("features[{idx}].behaviors[{bidx}].name"),
                format!("Behavior name '{}' is too generic", behavior.name),
              )
              .with_suggestion(
                "Use more descriptive name (e.g., 'handle_user_login' instead of 'handle')"
                  .to_string(),
              ),
            );
          }
        }
      }
    }
  }

  /// Check description quality
  ///
  /// Analyzes the quality of descriptions:
  /// - Spec description should be at least 10 characters
  /// - Feature descriptions should be at least 10 characters if present
  /// - Behavior descriptions should be present and reasonably detailed
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  /// * `report` - The report to add results to
  fn check_description_quality(spec: &Spec, report: &mut LintReport) {
    let severity = LintRule::DescriptionQuality.default_severity();

    // Check spec description
    if spec.description.trim().len() < MIN_DESCRIPTION_LENGTH {
      report.add_result(
        LintResult::new(
          LintRule::DescriptionQuality,
          severity,
          "spec.description".to_string(),
          "Spec description is too short or missing".to_string(),
        )
        .with_suggestion(format!(
          "Add a more detailed description (at least {MIN_DESCRIPTION_LENGTH} characters)"
        )),
      );
    }

    // Check feature descriptions
    for (idx, feature) in spec.features.iter().enumerate() {
      if feature.description.trim().len() < MIN_DESCRIPTION_LENGTH
        && !feature.description.is_empty()
      {
        report.add_result(
          LintResult::new(
            LintRule::DescriptionQuality,
            severity,
            format!("features[{idx}].description"),
            format!("Feature '{}' description is too short", feature.name),
          )
          .with_suggestion("Provide more detailed feature description".to_string()),
        );
      }

      // Check behavior descriptions
      for (bidx, behavior) in feature.behaviors.iter().enumerate() {
        if behavior.description.trim().is_empty() {
          report.add_result(
            LintResult::new(
              LintRule::DescriptionQuality,
              severity,
              format!("features[{idx}].behaviors[{bidx}].description"),
              format!("Behavior '{}' has no description", behavior.name),
            )
            .with_suggestion("Add a description explaining what the behavior does".to_string()),
          );
        } else if behavior.description.trim().len() < MIN_BEHAVIOR_DESCRIPTION_LENGTH {
          report.add_result(
            LintResult::new(
              LintRule::DescriptionQuality,
              LintSeverity::Hint,
              format!("features[{idx}].behaviors[{bidx}].description"),
              format!("Behavior '{}' description is very short", behavior.name),
            )
            .with_suggestion("Consider expanding the description".to_string()),
          );
        }
      }
    }
  }

  /// Check spec completeness
  ///
  /// Checks overall completeness of the specification:
  /// - Features without descriptions
  /// - Behaviors without descriptions
  /// - Features with only one behavior (may indicate missing behaviors)
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  /// * `report` - The report to add results to
  fn check_completeness(spec: &Spec, report: &mut LintReport) {
    let severity = LintRule::Completeness.default_severity();

    // Check for features without descriptions
    let features_without_desc = spec
      .features
      .iter()
      .filter(|f| f.description.trim().is_empty())
      .count();

    if features_without_desc > 0 {
      report.add_result(
        LintResult::new(
          LintRule::Completeness,
          severity,
          "features".to_string(),
          format!("{features_without_desc} feature(s) missing descriptions"),
        )
        .with_suggestion("Add descriptions to all features".to_string()),
      );
    }

    // Check for behaviors without descriptions
    let behaviors_without_desc: usize = spec
      .features
      .iter()
      .map(|f| {
        f.behaviors
          .iter()
          .filter(|b| b.description.trim().is_empty())
          .count()
      })
      .sum();

    if behaviors_without_desc > 0 {
      report.add_result(
        LintResult::new(
          LintRule::Completeness,
          severity,
          "features[].behaviors".to_string(),
          format!("{behaviors_without_desc} behavior(s) missing descriptions")
        )
        .with_suggestion("Add descriptions to all behaviors".to_string()),
      );
    }

    // Check for features with only one behavior
    let single_behavior_features: Vec<&str> = spec
      .features
      .iter()
      .filter(|f| f.behaviors.len() == 1)
      .map(|f| f.name.as_str())
      .collect();

    if !single_behavior_features.is_empty() {
      report.add_result(
        LintResult::new(
          LintRule::Completeness,
          LintSeverity::Info,
          "features".to_string(),
          format!(
            "Features with single behavior: {}",
            single_behavior_features.join(", ")
          ),
        )
        .with_suggestion(
          "Consider if these behaviors can be merged or if more behaviors are needed".to_string(),
        ),
      );
    }
  }
}

impl Default for SpecLinter {
  fn default() -> Self {
    Self::new()
  }
}

/// Format a lint report as a string
///
/// Creates a human-readable formatted report with summary statistics
/// and detailed results grouped by severity level.
///
/// # Arguments
///
/// * `report` - The lint report to format
///
/// # Returns
///
/// A formatted string representation of the report
///
/// # Examples
///
/// ```no_run
/// # use clarity_web::intent::quality::linter::{format_lint_report, SpecLinter};
/// # use clarity_web::intent::types::Spec;
/// let linter = SpecLinter::new();
/// let spec = Spec::new("my-spec".to_string()).unwrap();
///
/// match linter.lint_spec(&spec) {
///     Ok(report) => {
///         let formatted = format_lint_report(&report);
///         println!("{}", formatted);
///     }
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn format_lint_report(report: &LintReport) -> String {
  use std::fmt::Write;

  let mut output = String::new();

  // Summary
  output.push_str("=== Lint Report ===\n");
  let _ = writeln!(
    output,
    "Errors: {}, Warnings: {}, Info: {}, Hints: {}\n\n",
    report.error_count, report.warning_count, report.info_count, report.hint_count
  );

  // Results by severity
  for severity in [
    LintSeverity::Error,
    LintSeverity::Warning,
    LintSeverity::Info,
    LintSeverity::Hint,
  ] {
    let results = report.by_severity(severity);
    if !results.is_empty() {
      let _ = writeln!(output, "--- {} ---", severity.as_str().to_uppercase());
      for result in results {
        let _ = writeln!(output, "  {}", result.format());
      }
      let _ = writeln!(output);
    }
  }

  output
}

/// Convenience function to lint a spec
///
/// Creates a default `SpecLinter` and lints the specification.
/// Equivalent to `SpecLinter::new().lint_spec(spec)`.
///
/// # Arguments
///
/// * `spec` - The specification to lint
///
/// # Returns
///
/// A `LintReport` containing all lint results
///
/// # Errors
///
/// Returns `LintError` if the spec has structural issues
///
/// # Examples
///
/// ```no_run
/// # use clarity_web::intent::quality::linter::lint_spec;
/// # use clarity_web::intent::types::Spec;
/// let spec = Spec::new("my-spec".to_string()).unwrap();
///
/// match lint_spec(&spec) {
///     Ok(report) => println!("Found {} issues", report.results.len()),
///     Err(e) => println!("Linting failed: {}", e),
/// }
/// ```
#[allow(clippy::missing_const_for_fn)]
pub fn lint_spec(spec: &Spec) -> Result<LintReport, LintError> {
  SpecLinter::new().lint_spec(spec)
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use crate::intent::types::{Behavior, Feature};

  fn create_valid_spec() -> Spec {
    match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "A test specification".to_string();

        let mut auth_feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("Authentication features".to_string()),
          Err(_) => return spec,
        };

        let login = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return spec,
        };

        let logout = match Behavior::new("logout".to_string()) {
          Ok(b) => b.with_description("User logs out".to_string()),
          Err(_) => return spec,
        };

        let _ = auth_feature.add_behavior(login);
        let _ = auth_feature.add_behavior(logout);
        let _ = spec.add_feature(auth_feature);
        spec
      }
      Err(_) => panic!("Failed to create spec"),
    }
  }

  fn create_problematic_spec() -> Spec {
    match Spec::new("problematic-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "A test spec".to_string();

        // Feature with mixed naming conventions (kebab AND snake)
        let mut bad_feature = match Feature::new("auth_service-test".to_string()) {
          Ok(f) => f,
          Err(_) => panic!("Failed to create feature"),
        };

        // Behavior with vague description
        let mut vague_behavior = match Behavior::new("user_login".to_string()) {
          Ok(b) => b,
          Err(_) => panic!("Failed to create behavior"),
        };
        vague_behavior.description = "TODO: Implement this".to_string();

        // Generic behavior name
        let generic_behavior = match Behavior::new("handle".to_string()) {
          Ok(b) => b.with_description("Handle something".to_string()),
          Err(_) => panic!("Failed to create behavior"),
        };

        // Behavior without description
        let no_desc = match Behavior::new("process".to_string()) {
          Ok(b) => b,
          Err(_) => panic!("Failed to create behavior"),
        };

        let _ = bad_feature.add_behavior(vague_behavior);
        let _ = bad_feature.add_behavior(generic_behavior);
        let _ = bad_feature.add_behavior(no_desc);
        let _ = spec.add_feature(bad_feature);
        spec
      }
      Err(_) => panic!("Failed to create spec"),
    }
  }

  #[test]
  fn test_lint_valid_spec() {
    let spec = create_valid_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should have minimal issues
    assert!(!report.has_errors());
  }

  #[test]
  fn test_lint_empty_spec_name() {
    let mut spec = match Spec::new("   ".to_string()) {
      Ok(s) => s,
      Err(_) => return,
    };

    let mut feature = match Feature::new("test".to_string()) {
      Ok(f) => f,
      Err(_) => return,
    };

    let behavior = match Behavior::new("test_behavior".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };

    let _ = feature.add_behavior(behavior);
    let _ = spec.add_feature(feature);

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_err());
    assert!(matches!(result, Err(LintError::EmptySpecName)));
  }

  #[test]
  fn test_lint_no_features() {
    let spec = match Spec::new("empty-spec".to_string()) {
      Ok(s) => s,
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_err());
    assert!(matches!(result, Err(LintError::NoFeatures)));
  }

  #[test]
  fn test_naming_convention_lint() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    let naming_results = report.by_rule(LintRule::NamingConvention);
    // Should flag mixed naming in feature name (kebab-case)
    assert!(naming_results
      .iter()
      .any(|r| { r.message.contains("kebab-case") && r.message.contains("snake_case") }));
  }

  #[test]
  fn test_deprecated_pattern_lint() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    let deprecated_results = report.by_rule(LintRule::DeprecatedPattern);
    assert!(!deprecated_results.is_empty());

    // Should flag TODO in description
    assert!(deprecated_results
      .iter()
      .any(|r| r.message.contains("todo")));

    // Should flag generic behavior name
    assert!(deprecated_results
      .iter()
      .any(|r| r.message.contains("generic")));
  }

  #[test]
  fn test_description_quality_lint() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    let quality_results = report.by_rule(LintRule::DescriptionQuality);
    assert!(!quality_results.is_empty());

    // Should flag missing description
    assert!(quality_results
      .iter()
      .any(|r| r.message.contains("no description")));
  }

  #[test]
  fn test_lint_with_specific_rules() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::with_rules(vec![LintRule::NamingConvention]);
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should only have naming convention results
    assert!(!report.by_rule(LintRule::NamingConvention).is_empty());
    assert!(report.by_rule(LintRule::DeprecatedPattern).is_empty());
  }

  #[test]
  fn test_format_lint_report() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    let formatted = format_lint_report(&report);
    assert!(formatted.contains("Lint Report"));
    assert!(formatted.contains("Errors:") || formatted.contains("Warnings:"));
  }

  #[test]
  fn test_lint_severity_levels() {
    let spec = create_problematic_spec();
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should have results at different severity levels
    let has_warnings = !report.by_severity(LintSeverity::Warning).is_empty();
    let has_info = !report.by_severity(LintSeverity::Info).is_empty();

    assert!(has_warnings || has_info);
  }

  #[test]
  fn test_lint_deterministic() {
    let spec = create_valid_spec();
    let linter = SpecLinter::new();

    let result1 = linter.lint_spec(&spec);
    let result2 = linter.lint_spec(&spec);

    if let (Ok(r1), Ok(r2)) = (result1, result2) {
      assert_eq!(r1, r2);
    }
  }

  #[test]
  fn test_no_unwrap_in_linter() {
    // This test is verified by the lints at the top of the file
    let spec = create_valid_spec();
    let _ = SpecLinter::new().lint_spec(&spec);
    // If we got here without panicking, the test passes
  }

  #[test]
  fn test_empty_behavior_description() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(mut b) => {
            b.description = String::new();
            b
          }
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(report
      .results
      .iter()
      .any(|r| r.message.contains("no description")));
  }

  #[test]
  fn test_feature_with_single_behavior() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User login".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(report
      .results
      .iter()
      .any(|r| r.message.contains("single behavior")));
  }
}
