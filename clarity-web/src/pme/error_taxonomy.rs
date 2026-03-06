#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Error Taxonomy Engine for PME Develop Phase
//!
//! Implements a 5-category error classification system based on recoverability:
//! - SystemError: Unfixable by user (external dependencies, infrastructure)
//! - UserInvalidArgument: Fixable by user (invalid input they can correct)
//! - PreconditionNotMet: Fixable by user (missing prerequisites)
//! - DeveloperInvalidArgument: BUG (invalid API usage - developer fault)
//! - Assertion: CRITICAL BUG (invariant violation - immediate attention)
//!
//! Each category has specific routing and user messaging strategies.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Domain errors for error taxonomy operations
#[derive(Debug, Error, PartialEq, Clone)]
pub enum TaxonomyError {
    #[error("invalid error category: {0}")]
    InvalidCategory(String),

    #[error("remediation failed: {0}")]
    RemediationFailed(String),

    #[error("routing failed: {0}")]
    RoutingFailed(String),
}

/// The 5 error categories in the taxonomy
///
/// Ordered by severity and responsibility (System -> User -> Developer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Unfixable by user - external dependencies, infrastructure failures
    /// Routing: Log, alert ops team, show generic message
    SystemError,

    /// Fixable by user - invalid input they can correct
    /// Routing: Show specific error with correction guidance
    UserInvalidArgument,

    /// Fixable by user - missing prerequisites, wrong state
    /// Routing: Show what's missing and how to satisfy it
    PreconditionNotMet,

    /// BUG - invalid API usage, developer's fault
    /// Routing: Log as bug, show generic message, notify dev team
    DeveloperInvalidArgument,

    /// CRITICAL BUG - invariant violation, must be fixed immediately
    /// Routing: Alert immediately, block operation, require ack
    Assertion,
}

impl ErrorCategory {
    /// Get all categories in order
    pub fn all() -> &'static [ErrorCategory] {
        &[
            ErrorCategory::SystemError,
            ErrorCategory::UserInvalidArgument,
            ErrorCategory::PreconditionNotMet,
            ErrorCategory::DeveloperInvalidArgument,
            ErrorCategory::Assertion,
        ]
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            ErrorCategory::SystemError => "System Error",
            ErrorCategory::UserInvalidArgument => "Invalid Input",
            ErrorCategory::PreconditionNotMet => "Precondition Not Met",
            ErrorCategory::DeveloperInvalidArgument => "API Misuse",
            ErrorCategory::Assertion => "Assertion Failure",
        }
    }

    /// Who is responsible for fixing this error
    pub fn responsibility(&self) -> Responsibility {
        match self {
            ErrorCategory::SystemError => Responsibility::Operations,
            ErrorCategory::UserInvalidArgument => Responsibility::User,
            ErrorCategory::PreconditionNotMet => Responsibility::User,
            ErrorCategory::DeveloperInvalidArgument => Responsibility::Developer,
            ErrorCategory::Assertion => Responsibility::DeveloperCritical,
        }
    }

    /// Can the user fix this error?
    pub fn is_user_fixable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::UserInvalidArgument | ErrorCategory::PreconditionNotMet
        )
    }

    /// Is this a developer bug?
    pub fn is_bug(&self) -> bool {
        matches!(
            self,
            ErrorCategory::DeveloperInvalidArgument | ErrorCategory::Assertion
        )
    }

    /// Is this critical (requires immediate attention)?
    pub fn is_critical(&self) -> bool {
        matches!(self, ErrorCategory::Assertion)
    }

    /// Get routing strategy for this error category
    pub fn routing_strategy(&self) -> RoutingStrategy {
        match self {
            ErrorCategory::SystemError => RoutingStrategy {
                log_level: LogLevel::Error,
                alert_team: true,
                show_user: true,
                user_message_style: MessageStyle::Generic,
                block_operation: false,
            },
            ErrorCategory::UserInvalidArgument => RoutingStrategy {
                log_level: LogLevel::Info,
                alert_team: false,
                show_user: true,
                user_message_style: MessageStyle::SpecificWithGuidance,
                block_operation: true,
            },
            ErrorCategory::PreconditionNotMet => RoutingStrategy {
                log_level: LogLevel::Info,
                alert_team: false,
                show_user: true,
                user_message_style: MessageStyle::SpecificWithGuidance,
                block_operation: true,
            },
            ErrorCategory::DeveloperInvalidArgument => RoutingStrategy {
                log_level: LogLevel::Warn,
                alert_team: true,
                show_user: true,
                user_message_style: MessageStyle::Generic,
                block_operation: false,
            },
            ErrorCategory::Assertion => RoutingStrategy {
                log_level: LogLevel::Critical,
                alert_team: true,
                show_user: true,
                user_message_style: MessageStyle::Generic,
                block_operation: true,
            },
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Who is responsible for fixing an error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Responsibility {
    /// Operations team handles infrastructure
    Operations,
    /// User can correct their input
    User,
    /// Developer needs to fix code
    Developer,
    /// Developer must fix immediately (critical)
    DeveloperCritical,
}

/// Log level for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// How to style user-facing messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStyle {
    /// Generic "Something went wrong" message
    Generic,
    /// Specific error with guidance on how to fix
    SpecificWithGuidance,
}

/// Routing strategy for an error category
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingStrategy {
    /// What log level to use
    pub log_level: LogLevel,
    /// Should the team be alerted?
    pub alert_team: bool,
    /// Should this be shown to the user?
    pub show_user: bool,
    /// How to style the user message
    pub user_message_style: MessageStyle,
    /// Should this block the operation?
    pub block_operation: bool,
}

/// A classified error with context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassifiedError {
    /// The error category
    pub category: ErrorCategory,
    /// Technical error code
    pub code: String,
    /// Technical message (for logs)
    pub technical_message: String,
    /// User-friendly message
    pub user_message: String,
    /// Guidance on how to fix (for user-fixable errors)
    pub remediation: Option<Remediation>,
    /// Context where the error occurred
    pub context: ErrorContext,
    /// Timestamp when the error occurred
    pub timestamp: String,
}

impl ClassifiedError {
    /// Create a new classified error
    pub fn new(
        category: ErrorCategory,
        code: impl Into<String>,
        technical_message: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            code: code.into(),
            technical_message: technical_message.into(),
            user_message: user_message.into(),
            remediation: None,
            context: ErrorContext::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add remediation guidance
    pub fn with_remediation(mut self, remediation: Remediation) -> Self {
        self.remediation = Some(remediation);
        self
    }

    /// Add context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Get the routing strategy for this error
    pub fn routing(&self) -> RoutingStrategy {
        self.category.routing_strategy()
    }

    /// Generate a user-facing message based on routing strategy
    pub fn user_display_message(&self) -> String {
        match self.routing().user_message_style {
            MessageStyle::Generic => {
                "Something went wrong. Our team has been notified.".to_string()
            }
            MessageStyle::SpecificWithGuidance => {
                let base = &self.user_message;
                match &self.remediation {
                    Some(rem) => format!("{}. {}", base, rem.guidance),
                    None => base.clone(),
                }
            }
        }
    }
}

/// Remediation guidance for user-fixable errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Remediation {
    /// What the user should do
    pub guidance: String,
    /// Optional link to documentation
    pub doc_link: Option<String>,
    /// Optional example of correct input
    pub example: Option<String>,
}

impl Remediation {
    /// Create new remediation guidance
    pub fn new(guidance: impl Into<String>) -> Self {
        Self {
            guidance: guidance.into(),
            doc_link: None,
            example: None,
        }
    }

    /// Add documentation link
    pub fn with_doc_link(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }

    /// Add example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
}

/// Context where an error occurred
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Component or module where error occurred
    pub component: String,
    /// Operation being performed
    pub operation: String,
    /// Additional metadata
    pub metadata: Vec<(String, String)>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            component: "unknown".to_string(),
            operation: "unknown".to_string(),
            metadata: Vec::new(),
        }
    }
}

impl ErrorContext {
    /// Create new error context
    pub fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            metadata: Vec::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// The Error Taxonomy Engine
///
/// Classifies errors and provides routing guidance
#[derive(Debug, Default)]
pub struct ErrorTaxonomyEngine {
    /// Custom classifiers for domain-specific errors
    classifiers: Vec<Box<dyn ErrorClassifier>>,
}

impl ErrorTaxonomyEngine {
    /// Create a new error taxonomy engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom error classifier
    pub fn with_classifier(mut self, classifier: Box<dyn ErrorClassifier>) -> Self {
        self.classifiers.push(classifier);
        self
    }

    /// Classify an error based on its characteristics
    pub fn classify(
        &self,
        error_code: &str,
        message: &str,
        context: ErrorContext,
    ) -> Result<ClassifiedError, TaxonomyError> {
        // Try custom classifiers first
        for classifier in &self.classifiers {
            if let Some(classified) = classifier.classify(error_code, message, &context) {
                return Ok(classified);
            }
        }

        // Default classification based on error code patterns
        let (category, user_message, remediation) = self.default_classification(error_code, message);

        let classified = ClassifiedError::new(category, error_code, message, user_message)
            .with_context(context);

        match remediation {
            Some(rem) => Ok(classified.with_remediation(rem)),
            None => Ok(classified),
        }
    }

    /// Default classification based on common patterns
    fn default_classification(
        &self,
        error_code: &str,
        message: &str,
    ) -> (ErrorCategory, String, Option<Remediation>) {
        let lower_code = error_code.to_lowercase();
        let lower_msg = message.to_lowercase();

        // System errors - infrastructure, external services
        if lower_code.contains("econnrefused")
            || lower_code.contains("etimedout")
            || lower_code.contains("enotfound")
            || lower_msg.contains("connection refused")
            || lower_msg.contains("service unavailable")
            || lower_msg.contains("timeout")
            || lower_code.starts_with("sys_")
            || lower_code.starts_with("infra_")
        {
            return (
                ErrorCategory::SystemError,
                "A system error occurred. Please try again later.".to_string(),
                None,
            );
        }

        // Precondition errors - missing prerequisites
        if lower_code.contains("precondition")
            || lower_code.contains("prerequisite")
            || lower_code.starts_with("pre_")
            || lower_msg.contains("must be")
            || lower_msg.contains("required before")
            || lower_msg.contains("not initialized")
        {
            return (
                ErrorCategory::PreconditionNotMet,
                "A required condition was not met.".to_string(),
                Some(Remediation::new("Complete the required steps before proceeding")),
            );
        }

        // Assertion errors - invariant violations
        if lower_code.contains("assert")
            || lower_code.contains("invariant")
            || lower_code.starts_with("assert_")
            || lower_msg.contains("assertion failed")
            || lower_msg.contains("invariant violated")
        {
            return (
                ErrorCategory::Assertion,
                "An unexpected state was encountered.".to_string(),
                None,
            );
        }

        // Developer errors - API misuse patterns
        if lower_code.starts_with("api_")
            || lower_code.starts_with("dev_")
            || lower_msg.contains("invalid argument")
            || lower_msg.contains("not implemented")
        {
            return (
                ErrorCategory::DeveloperInvalidArgument,
                "An internal error occurred.".to_string(),
                None,
            );
        }

        // Default to user-invalid-argument (most user-fixable)
        (
            ErrorCategory::UserInvalidArgument,
            "Invalid input provided.".to_string(),
            Some(Remediation::new("Please check your input and try again")),
        )
    }

    /// Route an error to appropriate handlers
    pub fn route(&self, error: &ClassifiedError) -> RoutingResult {
        let strategy = error.routing();

        RoutingResult {
            should_log: true,
            log_level: strategy.log_level,
            should_alert_team: strategy.alert_team,
            user_message: error.user_display_message(),
            should_block: strategy.block_operation,
        }
    }

    /// Get a summary of errors by category
    pub fn summarize_errors(errors: &[ClassifiedError]) -> ErrorSummary {
        let by_category = errors
            .iter()
            .chunk_by(|e| e.category)
            .into_iter()
            .map(|(cat, group)| (cat, group.count()))
            .collect();

        let user_fixable_count = errors
            .iter()
            .filter(|e| e.category.is_user_fixable())
            .count();

        let bug_count = errors.iter().filter(|e| e.category.is_bug()).count();

        let critical_count = errors.iter().filter(|e| e.category.is_critical()).count();

        ErrorSummary {
            total: errors.len(),
            by_category,
            user_fixable_count,
            bug_count,
            critical_count,
        }
    }
}

/// Trait for custom error classifiers
pub trait ErrorClassifier: std::fmt::Debug {
    /// Classify an error, returning None if this classifier doesn't handle it
    fn classify(
        &self,
        error_code: &str,
        message: &str,
        context: &ErrorContext,
    ) -> Option<ClassifiedError>;
}

/// Result of routing an error
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingResult {
    /// Should this be logged?
    pub should_log: bool,
    /// What log level to use
    pub log_level: LogLevel,
    /// Should the team be alerted?
    pub should_alert_team: bool,
    /// Message to show the user
    pub user_message: String,
    /// Should the operation be blocked?
    pub should_block: bool,
}

/// Summary of errors by category
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorSummary {
    /// Total error count
    pub total: usize,
    /// Count by category
    pub by_category: std::collections::HashMap<ErrorCategory, usize>,
    /// Number of user-fixable errors
    pub user_fixable_count: usize,
    /// Number of bugs
    pub bug_count: usize,
    /// Number of critical bugs
    pub critical_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_properties() {
        assert!(ErrorCategory::UserInvalidArgument.is_user_fixable());
        assert!(ErrorCategory::PreconditionNotMet.is_user_fixable());
        assert!(!ErrorCategory::SystemError.is_user_fixable());

        assert!(ErrorCategory::DeveloperInvalidArgument.is_bug());
        assert!(ErrorCategory::Assertion.is_bug());
        assert!(!ErrorCategory::UserInvalidArgument.is_bug());

        assert!(ErrorCategory::Assertion.is_critical());
        assert!(!ErrorCategory::DeveloperInvalidArgument.is_critical());
    }

    #[test]
    fn test_routing_strategy() {
        let user_strategy = ErrorCategory::UserInvalidArgument.routing_strategy();
        assert!(!user_strategy.alert_team);
        assert_eq!(
            user_strategy.user_message_style,
            MessageStyle::SpecificWithGuidance
        );

        let assert_strategy = ErrorCategory::Assertion.routing_strategy();
        assert!(assert_strategy.alert_team);
        assert!(assert_strategy.block_operation);
        assert_eq!(assert_strategy.log_level, LogLevel::Critical);
    }

    #[test]
    fn test_classify_system_error() {
        let engine = ErrorTaxonomyEngine::new();
        let context = ErrorContext::new("network", "connect");

        let result = engine
            .classify("ECONNREFUSED", "Connection refused", context)
            .map_err(|e| e.to_string());

        assert!(result.is_ok());
        let classified = result.map_err(|e| e.to_string()).map_err(|_| "").ok();
        if let Some(c) = classified {
            assert_eq!(c.category, ErrorCategory::SystemError);
        }
    }

    #[test]
    fn test_classify_user_error() {
        let engine = ErrorTaxonomyEngine::new();
        let context = ErrorContext::new("validation", "validate_email");

        let result = engine
            .classify("invalid_email", "Email address is not valid", context)
            .map_err(|e| e.to_string());

        assert!(result.is_ok());
        let classified = result.map_err(|e| e.to_string()).map_err(|_| "").ok();
        if let Some(c) = classified {
            assert!(c.category.is_user_fixable());
            assert!(c.remediation.is_some());
        }
    }

    #[test]
    fn test_classify_assertion_error() {
        let engine = ErrorTaxonomyEngine::new();
        let context = ErrorContext::new("core", "process");

        let result = engine
            .classify("assert_failed", "Assertion failed: balance >= 0", context)
            .map_err(|e| e.to_string());

        assert!(result.is_ok());
        let classified = result.map_err(|e| e.to_string()).map_err(|_| "").ok();
        if let Some(c) = classified {
            assert_eq!(c.category, ErrorCategory::Assertion);
            assert!(c.category.is_critical());
        }
    }

    #[test]
    fn test_user_display_message() {
        let user_error = ClassifiedError::new(
            ErrorCategory::UserInvalidArgument,
            "invalid_email",
            "Email validation failed",
            "Please enter a valid email address",
        )
        .with_remediation(Remediation::new("Use format: user@example.com"));

        let message = user_error.user_display_message();
        assert!(message.contains("valid email"));
        assert!(message.contains("user@example.com"));

        let system_error = ClassifiedError::new(
            ErrorCategory::SystemError,
            "sys_down",
            "Database connection failed",
            "Service temporarily unavailable",
        );

        let system_message = system_error.user_display_message();
        assert!(system_message.contains("Something went wrong"));
    }

    #[test]
    fn test_error_summary() {
        let errors = vec![
            ClassifiedError::new(
                ErrorCategory::UserInvalidArgument,
                "e1",
                "Error 1",
                "User error",
            ),
            ClassifiedError::new(
                ErrorCategory::UserInvalidArgument,
                "e2",
                "Error 2",
                "User error",
            ),
            ClassifiedError::new(
                ErrorCategory::DeveloperInvalidArgument,
                "e3",
                "Error 3",
                "Dev error",
            ),
            ClassifiedError::new(ErrorCategory::Assertion, "e4", "Error 4", "Critical"),
        ];

        let summary = ErrorTaxonomyEngine::summarize_errors(&errors);

        assert_eq!(summary.total, 4);
        assert_eq!(summary.user_fixable_count, 2);
        assert_eq!(summary.bug_count, 2);
        assert_eq!(summary.critical_count, 1);
    }

    #[test]
    fn test_remediation_builder() {
        let rem = Remediation::new("Fix the input")
            .with_doc_link("https://docs.example.com/errors")
            .with_example("valid-input-here");

        assert_eq!(rem.guidance, "Fix the input");
        assert_eq!(
            rem.doc_link,
            Some("https://docs.example.com/errors".to_string())
        );
        assert_eq!(rem.example, Some("valid-input-here".to_string()));
    }
}
