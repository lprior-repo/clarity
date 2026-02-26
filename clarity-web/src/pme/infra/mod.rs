#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Infrastructure Module for PME
//!
//! Production infrastructure for the Product Management Engine:
//! - **Logging**: Structured logging with tracing support
//! - **Tracing**: Distributed tracing across service boundaries
//! - **Metrics**: Real User Monitoring (RUM) metrics collection
//! - **Testing**: Testing framework with 80% coverage target
//!
//! # Architecture
//!
//! This module follows the Functional Core, Imperative Shell pattern:
//! - Core types are pure and immutable
//! - Side effects are isolated in shell adapters
//! - All operations return Result<T, E>
//!
//! # Example
//!
//! ```rust,ignore
//! use pme::infra::logging::{StructuredLogger, LogContext};
//! use pme::infra::tracing::{Tracer, SpanBuilder};
//! use pme::infra::metrics::{MetricsRegistry, RumCollector};
//!
//! // Set up logging
//! let logger = StructuredLogger::new()
//!     .with_service("pme-api")
//!     .with_min_level(LogLevel::Info);
//!
//! // Set up tracing
//! let tracer = Tracer::new("pme-api");
//! let span = tracer.start_trace("process_request");
//!
//! // Set up metrics
//! let rum = RumCollector::new("pme-web");
//! rum.record_page_load("/dashboard", 250.0);
//! ```

pub mod logging;
pub mod metrics;
pub mod testing;
pub mod tracing;

// Re-export logging types
pub use logging::{
    ErrorInfo, LogAggregator, LogContext, LogEntry, LogFormat, LogLevel, LoggerConfig,
    LogStats, LoggingError, SourceLocation, StructuredLogger,
};

// Re-export tracing types
pub use tracing::{
    AttributeValue, Span, SpanBuilder, SpanEvent, SpanId, SpanKind,
    SpanState, SpanStatus, TraceContext, TraceFlags, TraceId, TraceSummary, Tracer,
    TracerConfig, TracingError,
};

// Re-export metrics types
pub use metrics::{
    Counter, Gauge, Histogram, HistogramStats as MetricsHistogramStats, MetricDimensions,
    MetricsError, MetricSnapshot, MetricType, MetricValue, MetricsConfig, MetricsRegistry,
    MetricsSummary, RumCollector,
};

// Re-export testing types
pub use testing::{
    assert_contains, assert_empty, assert_eq, assert_err, assert_false, assert_in_range,
    assert_ne, assert_none, assert_not_empty, assert_ok, assert_some, assert_true,
    AssertionResult, CoverageItem, CoverageReport, CoverageTracker, ModuleCoverage,
    ModuleReport, TestDataGenerator, TestContext, TestFixture, TestResult, TestSummary,
    TestingError,
};

/// Infrastructure version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Infrastructure error type (union of all infra errors)
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    /// Logging error
    #[error("logging error: {0}")]
    Logging(#[from] LoggingError),

    /// Tracing error
    #[error("tracing error: {0}")]
    Tracing(#[from] TracingError),

    /// Metrics error
    #[error("metrics error: {0}")]
    Metrics(#[from] MetricsError),

    /// Testing error
    #[error("testing error: {0}")]
    Testing(#[from] TestingError),
}

/// Initialize infrastructure with sensible defaults
///
/// This sets up logging, tracing, and metrics with production-ready defaults.
/// Returns a tuple of (logger, tracer, metrics_registry).
#[must_use]
pub fn init_infra(service: &str) -> (StructuredLogger, Tracer, MetricsRegistry) {
    let logger = StructuredLogger::new()
        .with_service(service)
        .with_min_level(LogLevel::Info);

    let tracer = Tracer::new(service);

    let metrics = MetricsRegistry::new(service);

    (logger, tracer, metrics)
}

/// Infrastructure health check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Service name
    pub service: String,
    /// Whether logging is healthy
    pub logging_healthy: bool,
    /// Whether tracing is healthy
    pub tracing_healthy: bool,
    /// Whether metrics are healthy
    pub metrics_healthy: bool,
    /// Timestamp of health check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthStatus {
    /// Create a new health status
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            logging_healthy: true,
            tracing_healthy: true,
            metrics_healthy: true,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if all components are healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.logging_healthy && self.tracing_healthy && self.metrics_healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_infra() {
        let (logger, tracer, metrics) = init_infra("test-service");

        assert_eq!(logger.config().service, "test-service");
        assert_eq!(tracer.config().service, "test-service");
        assert_eq!(metrics.summary().service, "test-service");
    }

    #[test]
    fn test_health_status() {
        let health = HealthStatus::new("test-service");

        assert_eq!(health.service, "test-service");
        assert!(health.logging_healthy);
        assert!(health.tracing_healthy);
        assert!(health.metrics_healthy);
        assert!(health.is_healthy());
    }

    #[test]
    fn test_health_status_unhealthy() {
        let mut health = HealthStatus::new("test-service");
        health.logging_healthy = false;

        assert!(!health.is_healthy());
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_infra_error_from_logging() {
        let logging_err = LoggingError::InvalidLevel("bad".to_string());
        let infra_err: InfraError = logging_err.into();

        assert!(matches!(infra_err, InfraError::Logging(_)));
    }

    #[test]
    fn test_infra_error_from_tracing() {
        let tracing_err = TracingError::InvalidTraceId("bad".to_string());
        let infra_err: InfraError = tracing_err.into();

        assert!(matches!(infra_err, InfraError::Tracing(_)));
    }

    #[test]
    fn test_infra_error_from_testing() {
        let testing_err = TestingError::AssertionFailed("failed".to_string());
        let infra_err: InfraError = testing_err.into();

        assert!(matches!(infra_err, InfraError::Testing(_)));
    }
}
