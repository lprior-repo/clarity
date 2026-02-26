#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Structured Logging Infrastructure for PME
//!
//! Provides structured, context-aware logging with:
//! - JSON-formatted output for production
//! - Pretty-printed output for development
//! - Context propagation across async boundaries
//! - Log level filtering by module
//!
//! # Example
//!
//! ```rust,ignore
//! use pme::infra::logging::{StructuredLogger, LogContext};
//!
//! let logger = StructuredLogger::new()
//!     .with_service("pme-api")
//!     .with_version("1.0.0");
//!
//! let ctx = LogContext::new()
//!     .with_request_id("req-123")
//!     .with_user_id("user-456");
//!
//! logger.info(ctx, "Request processed", &json!({"duration_ms": 42}));
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during logging operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LoggingError {
    /// Invalid log level specified
    #[error("invalid log level: {0}")]
    InvalidLevel(String),

    /// Serialization failed
    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    /// Output write failed
    #[error("output write failed: {0}")]
    OutputFailed(String),

    /// Context is missing required fields
    #[error("missing required context field: {0}")]
    MissingContext(String),
}

// ============================================================================
// Log Level
// ============================================================================

/// Log level with semantic meaning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    /// Detailed debug information (development only)
    Trace,
    /// Debug information for troubleshooting
    Debug,
    /// Normal operational messages
    Info,
    /// Warning conditions that might indicate problems
    Warn,
    /// Error conditions that need attention
    Error,
    /// Critical errors requiring immediate action
    Fatal,
}

impl LogLevel {
    /// Parse from string, returns error if invalid
    pub fn parse(s: &str) -> Result<Self, LoggingError> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" | "err" => Ok(Self::Error),
            "fatal" | "critical" => Ok(Self::Fatal),
            _ => Err(LoggingError::InvalidLevel(s.to_string())),
        }
    }

    /// Get all levels in order
    pub const fn all() -> [Self; 6] {
        [
            Self::Trace,
            Self::Debug,
            Self::Info,
            Self::Warn,
            Self::Error,
            Self::Fatal,
        ]
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Fatal => write!(f, "FATAL"),
        }
    }
}

// ============================================================================
// Log Context
// ============================================================================

/// Context information for structured logging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogContext {
    /// Unique request identifier for tracing
    pub request_id: Option<String>,
    /// User identifier (if authenticated)
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Tenant/organization identifier (multi-tenant)
    pub tenant_id: Option<String>,
    /// Component/module name
    pub component: Option<String>,
    /// Operation being performed
    pub operation: Option<String>,
    /// Additional custom fields
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

impl LogContext {
    /// Create an empty context
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_id: None,
            user_id: None,
            session_id: None,
            tenant_id: None,
            component: None,
            operation: None,
            extra: HashMap::new(),
        }
    }

    /// Add request ID
    #[must_use]
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Add user ID
    #[must_use]
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Add session ID
    #[must_use]
    pub fn with_session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Add tenant ID
    #[must_use]
    pub fn with_tenant_id(mut self, id: impl Into<String>) -> Self {
        self.tenant_id = Some(id.into());
        self
    }

    /// Add component name
    #[must_use]
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Add operation name
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Add custom field
    #[must_use]
    pub fn with_field(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Merge with another context (other takes precedence)
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();
        if other.request_id.is_some() {
            merged.request_id = other.request_id.clone();
        }
        if other.user_id.is_some() {
            merged.user_id = other.user_id.clone();
        }
        if other.session_id.is_some() {
            merged.session_id = other.session_id.clone();
        }
        if other.tenant_id.is_some() {
            merged.tenant_id = other.tenant_id.clone();
        }
        if other.component.is_some() {
            merged.component = other.component.clone();
        }
        if other.operation.is_some() {
            merged.operation = other.operation.clone();
        }
        merged.extra.extend(other.extra.clone());
        merged
    }

    /// Check if required fields are present
    pub fn validate_required(&self, required: &[&str]) -> Result<(), LoggingError> {
        let missing: Vec<String> = required
            .iter()
            .filter(|&field| match *field {
                "request_id" => self.request_id.is_none(),
                "user_id" => self.user_id.is_none(),
                "session_id" => self.session_id.is_none(),
                "tenant_id" => self.tenant_id.is_none(),
                "component" => self.component.is_none(),
                "operation" => self.operation.is_none(),
                _ => !self.extra.contains_key(*field),
            })
            .map(|s| s.to_string())
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(LoggingError::MissingContext(missing.join(", ")))
        }
    }
}

// ============================================================================
// Structured Log Entry
// ============================================================================

/// A single structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Context information
    #[serde(flatten)]
    pub context: LogContext,
    /// Structured data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
    /// Error information (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// Source code location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

/// Error information for log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type/name
    pub r#type: String,
    /// Error message
    pub message: String,
    /// Stack trace (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Error code (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Module path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    #[must_use]
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        service: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            service: service.into(),
            version: version.into(),
            context: LogContext::new(),
            data: None,
            error: None,
            location: None,
        }
    }

    /// Add context to the entry
    #[must_use]
    pub fn with_context(mut self, context: LogContext) -> Self {
        self.context = context;
        self
    }

    /// Add structured data
    #[must_use]
    pub fn with_data(mut self, data: JsonValue) -> Self {
        self.data = Some(data);
        self
    }

    /// Add error information
    #[must_use]
    pub fn with_error(mut self, error: ErrorInfo) -> Self {
        self.error = Some(error);
        self
    }

    /// Add source location
    #[must_use]
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, LoggingError> {
        serde_json::to_string(self)
            .map_err(|e| LoggingError::SerializationFailed(e.to_string()))
    }

    /// Convert to pretty JSON string (for development)
    pub fn to_json_pretty(&self) -> Result<String, LoggingError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LoggingError::SerializationFailed(e.to_string()))
    }
}

// ============================================================================
// Log Output Format
// ============================================================================

/// Output format for logs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogFormat {
    /// JSON format (production)
    Json,
    /// Pretty-printed JSON (development)
    JsonPretty,
    /// Plain text (development)
    Plain,
}

// ============================================================================
// Structured Logger
// ============================================================================

/// Configuration for the structured logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Minimum log level
    pub min_level: LogLevel,
    /// Output format
    pub format: LogFormat,
    /// Include source location in logs
    pub include_location: bool,
    /// Modules to filter (module_path -> min_level)
    pub module_filters: HashMap<String, LogLevel>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            service: "pme".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            min_level: LogLevel::Info,
            format: LogFormat::Json,
            include_location: false,
            module_filters: HashMap::new(),
        }
    }
}

/// Structured logger with context support
#[derive(Debug, Clone)]
pub struct StructuredLogger {
    config: Arc<LoggerConfig>,
}

impl StructuredLogger {
    /// Create a new structured logger with defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(LoggerConfig::default()),
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: LoggerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Set service name
    #[must_use]
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        let config = Arc::make_mut(&mut self.config);
        config.service = service.into();
        self
    }

    /// Set version
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let config = Arc::make_mut(&mut self.config);
        config.version = version.into();
        self
    }

    /// Set minimum log level
    #[must_use]
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        let config = Arc::make_mut(&mut self.config);
        config.min_level = level;
        self
    }

    /// Set output format
    #[must_use]
    pub fn with_format(mut self, format: LogFormat) -> Self {
        let config = Arc::make_mut(&mut self.config);
        config.format = format;
        self
    }

    /// Check if a level should be logged
    #[must_use]
    pub fn should_log(&self, level: LogLevel, module: Option<&str>) -> bool {
        // Check module-specific filter first
        if let Some(module_path) = module {
            for (filter_module, filter_level) in &self.config.module_filters {
                if module_path.starts_with(filter_module) {
                    return level >= *filter_level;
                }
            }
        }
        level >= self.config.min_level
    }

    /// Create a log entry at the given level
    #[must_use]
    pub fn create_entry(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        context: LogContext,
    ) -> LogEntry {
        LogEntry::new(level, message, &self.config.service, &self.config.version)
            .with_context(context)
    }

    /// Log at trace level
    pub fn trace(&self, context: LogContext, message: impl Into<String>, data: &JsonValue) {
        if self.should_log(LogLevel::Trace, context.component.as_deref()) {
            let entry = self.create_entry(LogLevel::Trace, message, context).with_data(data.clone());
            self.output(entry);
        }
    }

    /// Log at debug level
    pub fn debug(&self, context: LogContext, message: impl Into<String>, data: &JsonValue) {
        if self.should_log(LogLevel::Debug, context.component.as_deref()) {
            let entry = self.create_entry(LogLevel::Debug, message, context).with_data(data.clone());
            self.output(entry);
        }
    }

    /// Log at info level
    pub fn info(&self, context: LogContext, message: impl Into<String>, data: &JsonValue) {
        if self.should_log(LogLevel::Info, context.component.as_deref()) {
            let entry = self.create_entry(LogLevel::Info, message, context).with_data(data.clone());
            self.output(entry);
        }
    }

    /// Log at warn level
    pub fn warn(&self, context: LogContext, message: impl Into<String>, data: &JsonValue) {
        if self.should_log(LogLevel::Warn, context.component.as_deref()) {
            let entry = self.create_entry(LogLevel::Warn, message, context).with_data(data.clone());
            self.output(entry);
        }
    }

    /// Log at error level
    pub fn error(&self, context: LogContext, message: impl Into<String>, data: &JsonValue) {
        if self.should_log(LogLevel::Error, context.component.as_deref()) {
            let entry = self.create_entry(LogLevel::Error, message, context).with_data(data.clone());
            self.output(entry);
        }
    }

    /// Log an error with error details
    pub fn log_error(
        &self,
        context: LogContext,
        message: impl Into<String>,
        error: &impl std::error::Error,
    ) {
        if self.should_log(LogLevel::Error, context.component.as_deref()) {
            let error_info = ErrorInfo {
                r#type: std::any::type_name_of_val(error).to_string(),
                message: error.to_string(),
                stack_trace: None,
                code: None,
            };
            let entry = self
                .create_entry(LogLevel::Error, message, context)
                .with_error(error_info);
            self.output(entry);
        }
    }

    /// Output a log entry (internal method)
    fn output(&self, entry: LogEntry) {
        let output = match self.config.format {
            LogFormat::Json => entry.to_json(),
            LogFormat::JsonPretty => entry.to_json_pretty(),
            LogFormat::Plain => Ok(format!(
                "[{}] {} {} - {}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                entry.level,
                entry.service,
                entry.message
            )),
        };

        if let Ok(text) = output {
            // Use eprintln for errors, println for others
            match entry.level {
                LogLevel::Error | LogLevel::Fatal => eprintln!("{text}"),
                _ => println!("{text}"),
            }
        }
    }

    /// Get the configuration
    #[must_use]
    pub fn config(&self) -> &LoggerConfig {
        &self.config
    }
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Log Aggregator for Testing
// ============================================================================

/// Log aggregator for capturing logs in tests
#[derive(Debug, Clone, Default)]
pub struct LogAggregator {
    entries: Vec<LogEntry>,
}

impl LogAggregator {
    /// Create a new log aggregator
    #[must_use]
    pub const fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add a log entry
    pub fn add(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    /// Get all entries
    #[must_use]
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Filter entries by level
    #[must_use]
    pub fn by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.level == level)
            .collect()
    }

    /// Filter entries containing text in message
    #[must_use]
    pub fn containing(&self, text: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.message.contains(text))
            .collect()
    }

    /// Get entry count
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get statistics about logged entries
    #[must_use]
    pub fn stats(&self) -> LogStats {
        let by_level = LogLevel::all()
            .iter()
            .map(|&level| (level, self.by_level(level).len()))
            .collect();

        LogStats {
            total: self.entries.len(),
            by_level,
        }
    }
}

/// Statistics about logged entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    /// Total number of entries
    pub total: usize,
    /// Count by level
    pub by_level: HashMap<LogLevel, usize>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parse() {
        assert_eq!(LogLevel::parse("info"), Ok(LogLevel::Info));
        assert_eq!(LogLevel::parse("INFO"), Ok(LogLevel::Info));
        assert_eq!(LogLevel::parse("debug"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::parse("warn"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::parse("warning"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::parse("error"), Ok(LogLevel::Error));
        assert!(LogLevel::parse("invalid").is_err());
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Fatal > LogLevel::Error);
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
    }

    #[test]
    fn test_log_context_builder() {
        let ctx = LogContext::new()
            .with_request_id("req-123")
            .with_user_id("user-456")
            .with_component("test-module");

        assert_eq!(ctx.request_id, Some("req-123".to_string()));
        assert_eq!(ctx.user_id, Some("user-456".to_string()));
        assert_eq!(ctx.component, Some("test-module".to_string()));
    }

    #[test]
    fn test_log_context_merge() {
        let ctx1 = LogContext::new()
            .with_request_id("req-1")
            .with_user_id("user-1");

        let ctx2 = LogContext::new()
            .with_request_id("req-2")
            .with_component("comp");

        let merged = ctx1.merge(&ctx2);

        assert_eq!(merged.request_id, Some("req-2".to_string())); // ctx2 wins
        assert_eq!(merged.user_id, Some("user-1".to_string())); // from ctx1
        assert_eq!(merged.component, Some("comp".to_string())); // from ctx2
    }

    #[test]
    fn test_log_context_validate() {
        let ctx = LogContext::new()
            .with_request_id("req-1");

        assert!(ctx.validate_required(&["request_id"]).is_ok());
        assert!(ctx.validate_required(&["user_id"]).is_err());
        assert!(ctx.validate_required(&["request_id", "user_id"]).is_err());
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "Test message", "test-service", "1.0.0")
            .with_context(LogContext::new().with_request_id("req-1"))
            .with_data(serde_json::json!({ "key": "value" }));

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.service, "test-service");
        assert_eq!(entry.context.request_id, Some("req-1".to_string()));
        assert!(entry.data.is_some());
    }

    #[test]
    fn test_log_entry_json() {
        let entry = LogEntry::new(LogLevel::Info, "Test", "svc", "1.0");
        let json = entry.to_json();

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"level\":\"INFO\""));
        assert!(json_str.contains("\"message\":\"Test\""));
    }

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();

        assert_eq!(config.service, "pme");
        assert_eq!(config.min_level, LogLevel::Info);
        assert_eq!(config.format, LogFormat::Json);
    }

    #[test]
    fn test_structured_logger_creation() {
        let logger = StructuredLogger::new()
            .with_service("my-service")
            .with_version("2.0.0")
            .with_min_level(LogLevel::Debug)
            .with_format(LogFormat::JsonPretty);

        assert_eq!(logger.config().service, "my-service");
        assert_eq!(logger.config().version, "2.0.0");
        assert_eq!(logger.config().min_level, LogLevel::Debug);
    }

    #[test]
    fn test_logger_should_log() {
        let logger = StructuredLogger::new()
            .with_min_level(LogLevel::Info);

        assert!(!logger.should_log(LogLevel::Trace, None));
        assert!(!logger.should_log(LogLevel::Debug, None));
        assert!(logger.should_log(LogLevel::Info, None));
        assert!(logger.should_log(LogLevel::Warn, None));
        assert!(logger.should_log(LogLevel::Error, None));
    }

    #[test]
    fn test_logger_module_filters() {
        let mut module_filters = HashMap::new();
        module_filters.insert("pme::debug".to_string(), LogLevel::Debug);

        let config = LoggerConfig {
            module_filters,
            ..LoggerConfig::default()
        };
        let logger = StructuredLogger::with_config(config);

        // Global level is Info, but pme::debug module allows Debug
        assert!(logger.should_log(LogLevel::Debug, Some("pme::debug::module")));
        assert!(!logger.should_log(LogLevel::Debug, Some("pme::other")));
    }

    #[test]
    fn test_log_aggregator() {
        let mut aggregator = LogAggregator::new();

        aggregator.add(LogEntry::new(LogLevel::Info, "Info 1", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Error, "Error 1", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Info, "Info 2", "svc", "1.0"));

        assert_eq!(aggregator.len(), 3);
        assert_eq!(aggregator.by_level(LogLevel::Info).len(), 2);
        assert_eq!(aggregator.by_level(LogLevel::Error).len(), 1);
    }

    #[test]
    fn test_log_aggregator_containing() {
        let mut aggregator = LogAggregator::new();

        aggregator.add(LogEntry::new(LogLevel::Info, "User login", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Info, "User logout", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Error, "System error", "svc", "1.0"));

        let user_entries = aggregator.containing("User");
        assert_eq!(user_entries.len(), 2);
    }

    #[test]
    fn test_log_aggregator_stats() {
        let mut aggregator = LogAggregator::new();

        aggregator.add(LogEntry::new(LogLevel::Info, "Info", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Info, "Info", "svc", "1.0"));
        aggregator.add(LogEntry::new(LogLevel::Error, "Error", "svc", "1.0"));

        let stats = aggregator.stats();

        assert_eq!(stats.total, 3);
        assert_eq!(*stats.by_level.get(&LogLevel::Info).unwrap_or(&0), 2);
        assert_eq!(*stats.by_level.get(&LogLevel::Error).unwrap_or(&0), 1);
    }

    #[test]
    fn test_error_info() {
        let error_info = ErrorInfo {
            r#type: "TestError".to_string(),
            message: "Something went wrong".to_string(),
            stack_trace: Some("at line 1\nat line 2".to_string()),
            code: Some("E001".to_string()),
        };

        let entry = LogEntry::new(LogLevel::Error, "Error occurred", "svc", "1.0")
            .with_error(error_info);

        assert!(entry.error.is_some());
        let e = entry.error.unwrap();
        assert_eq!(e.r#type, "TestError");
        assert_eq!(e.code, Some("E001".to_string()));
    }

    #[test]
    fn test_log_context_extra_fields() {
        let ctx = LogContext::new()
            .with_field("custom_field", serde_json::json!("value"))
            .with_field("count", serde_json::json!(42));

        assert_eq!(ctx.extra.get("custom_field"), Some(&serde_json::json!("value")));
        assert_eq!(ctx.extra.get("count"), Some(&serde_json::json!(42)));
    }
}
