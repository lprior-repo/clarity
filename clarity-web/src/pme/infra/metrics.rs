#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Real User Monitoring (RUM) Metrics Infrastructure for PME
//!
//! Provides metrics collection with:
//! - Counter, Gauge, Histogram metric types
//! - Real-time aggregation
//! - Custom metric dimensions
//! - Performance percentile tracking
//!
//! # Example
//!
//! ```rust,ignore
//! use pme::infra::metrics::{MetricsRegistry, Counter, Gauge, Histogram};
//!
//! let registry = MetricsRegistry::new("pme-api");
//!
//! // Record a counter
//! registry.counter("requests_total")
//!     .with_dimension("method", "GET")
//!     .increment();
//!
//! // Record a gauge
//! registry.gauge("active_connections")
//!     .set(42.0);
//!
//! // Record a histogram
//! registry.histogram("request_duration_ms")
//!     .observe(123.0);
//! ```

use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during metrics operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum MetricsError {
    /// Invalid metric name
    #[error("invalid metric name: {0}")]
    InvalidName(String),

    /// Invalid metric value
    #[error("invalid metric value: {0}")]
    InvalidValue(String),

    /// Metric not found
    #[error("metric not found: {0}")]
    MetricNotFound(String),

    /// Invalid aggregation
    #[error("invalid aggregation: {0}")]
    InvalidAggregation(String),

    /// Dimension limit exceeded
    #[error("dimension limit exceeded: max {0}")]
    DimensionLimitExceeded(usize),
}

// ============================================================================
// Metric Types
// ============================================================================

/// Type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Counter - only increases (resets on restart)
    Counter,
    /// Gauge - can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Summary - similar to histogram with quantiles
    Summary,
}

// ============================================================================
// Metric Value
// ============================================================================

/// A metric value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The value
    pub value: f64,
    /// When this value was recorded
    pub timestamp: DateTime<Utc>,
}

impl MetricValue {
    /// Create a new metric value
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
        }
    }

    /// Create with explicit timestamp
    #[must_use]
    pub const fn with_timestamp(value: f64, timestamp: DateTime<Utc>) -> Self {
        Self { value, timestamp }
    }
}

// ============================================================================
// Metric Dimensions
// ============================================================================

/// Metric dimensions (labels/tags)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricDimensions {
    /// Key-value pairs
    pairs: Vec<(String, String)>,
}

impl MetricDimensions {
    /// Create empty dimensions
    #[must_use]
    pub const fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Create from a slice of key-value pairs
    #[must_use]
    pub fn from_slice(pairs: &[(&str, &str)]) -> Self {
        Self {
            pairs: pairs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    /// Add a dimension
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.pairs.push((key.into(), value.into()));
        self
    }

    /// Get a dimension value
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Get all pairs
    #[must_use]
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Get count
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Create a unique key for these dimensions
    #[must_use]
    pub fn to_key(&self) -> String {
        if self.pairs.is_empty() {
            String::new()
        } else {
            self.pairs
                .iter()
                .sorted_by(|a, b| a.0.cmp(&b.0))
                .map(|(k, v)| format!("{k}={v}"))
                .join(",")
        }
    }
}

impl Default for MetricDimensions {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Counter (Thread-safe)
// ============================================================================

/// Internal counter state
#[derive(Debug)]
struct CounterState {
    value: AtomicU64,
}

/// A counter metric that only increases
#[derive(Debug)]
pub struct Counter {
    name: String,
    help: Option<String>,
    state: Arc<CounterState>,
    dimensions: MetricDimensions,
}

impl Counter {
    /// Create a new counter
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: None,
            state: Arc::new(CounterState {
                value: AtomicU64::new(0),
            }),
            dimensions: MetricDimensions::new(),
        }
    }

    /// Add help text
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add dimension
    #[must_use]
    pub fn with_dimension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.dimensions = self.dimensions.with(key, value);
        self
    }

    /// Increment by 1
    pub fn increment(&self) {
        self.state.value.fetch_add(1, Ordering::SeqCst);
    }

    /// Increment by a specific amount
    pub fn increment_by(&self, amount: u64) {
        self.state.value.fetch_add(amount, Ordering::SeqCst);
    }

    /// Get current value
    #[must_use]
    pub fn get(&self) -> u64 {
        self.state.value.load(Ordering::SeqCst)
    }

    /// Get metric name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Export as a snapshot
    #[must_use]
    pub fn snapshot(&self) -> MetricSnapshot {
        MetricSnapshot {
            name: self.name.clone(),
            metric_type: MetricType::Counter,
            value: self.get() as f64,
            dimensions: self.dimensions.clone(),
            timestamp: Utc::now(),
            help: self.help.clone(),
        }
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            state: Arc::clone(&self.state),
            dimensions: self.dimensions.clone(),
        }
    }
}

// ============================================================================
// Gauge (Thread-safe)
// ============================================================================

/// Internal gauge state
#[derive(Debug)]
struct GaugeState {
    /// Store as integer scaled by 10^6 for precision
    value: AtomicU64,
}

const GAUGE_SCALE: f64 = 1_000_000.0;

impl GaugeState {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    fn set(&self, val: f64) {
        let scaled = (val * GAUGE_SCALE) as i64;
        // Use two's complement representation for negative values
        self.value.store(scaled as u64, Ordering::SeqCst);
    }

    fn get(&self) -> f64 {
        let scaled = self.value.load(Ordering::SeqCst) as i64;
        scaled as f64 / GAUGE_SCALE
    }

    fn add(&self, val: f64) {
        let current = self.get();
        self.set(current + val);
    }

    fn sub(&self, val: f64) {
        let current = self.get();
        self.set(current - val);
    }
}

/// A gauge metric that can go up or down
#[derive(Debug)]
pub struct Gauge {
    name: String,
    help: Option<String>,
    state: Arc<GaugeState>,
    dimensions: MetricDimensions,
}

impl Gauge {
    /// Create a new gauge
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: None,
            state: Arc::new(GaugeState::new()),
            dimensions: MetricDimensions::new(),
        }
    }

    /// Add help text
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add dimension
    #[must_use]
    pub fn with_dimension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.dimensions = self.dimensions.with(key, value);
        self
    }

    /// Set the gauge value
    pub fn set(&self, value: f64) {
        self.state.set(value);
    }

    /// Increment by 1
    pub fn increment(&self) {
        self.add(1.0);
    }

    /// Decrement by 1
    pub fn decrement(&self) {
        self.sub(1.0);
    }

    /// Add to the gauge
    pub fn add(&self, amount: f64) {
        self.state.add(amount);
    }

    /// Subtract from the gauge
    pub fn sub(&self, amount: f64) {
        self.state.sub(amount);
    }

    /// Get current value
    #[must_use]
    pub fn get(&self) -> f64 {
        self.state.get()
    }

    /// Get metric name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Export as a snapshot
    #[must_use]
    pub fn snapshot(&self) -> MetricSnapshot {
        MetricSnapshot {
            name: self.name.clone(),
            metric_type: MetricType::Gauge,
            value: self.get(),
            dimensions: self.dimensions.clone(),
            timestamp: Utc::now(),
            help: self.help.clone(),
        }
    }
}

impl Clone for Gauge {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            state: Arc::clone(&self.state),
            dimensions: self.dimensions.clone(),
        }
    }
}

// ============================================================================
// Histogram (Thread-safe)
// ============================================================================

/// Internal histogram state
#[derive(Debug)]
struct HistogramState {
    buckets: Vec<f64>,
    counts: std::sync::Mutex<Vec<u64>>,
    sum: std::sync::Mutex<f64>,
    count: AtomicU64,
}

/// A histogram metric for distribution of values
#[derive(Debug)]
pub struct Histogram {
    name: String,
    help: Option<String>,
    state: Arc<HistogramState>,
    dimensions: MetricDimensions,
}

impl Histogram {
    /// Create a new histogram with default buckets
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_buckets(
            name,
            vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
    }

    /// Create with custom buckets
    #[must_use]
    pub fn with_buckets(name: impl Into<String>, buckets: Vec<f64>) -> Self {
        let bucket_count = buckets.len();
        Self {
            name: name.into(),
            help: None,
            state: Arc::new(HistogramState {
                buckets,
                counts: std::sync::Mutex::new(vec![0; bucket_count + 1]), // +1 for +Inf bucket
                sum: std::sync::Mutex::new(0.0),
                count: AtomicU64::new(0),
            }),
            dimensions: MetricDimensions::new(),
        }
    }

    /// Add help text
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add dimension
    #[must_use]
    pub fn with_dimension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.dimensions = self.dimensions.with(key, value);
        self
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        self.state.count.fetch_add(1, Ordering::SeqCst);

        if let Ok(mut sum) = self.state.sum.lock() {
            *sum += value;
        }

        if let Ok(mut counts) = self.state.counts.lock() {
            for (i, &bucket) in self.state.buckets.iter().enumerate() {
                if value <= bucket {
                    counts[i] += 1;
                }
            }
            // Always increment the +Inf bucket
            if let Some(last) = counts.last_mut() {
                *last += 1;
            }
        }
    }

    /// Get histogram statistics
    #[must_use]
    pub fn stats(&self) -> Option<HistogramStats> {
        let counts = self.state.counts.lock().ok()?;
        let sum = *self.state.sum.lock().ok()?;
        let count = self.state.count.load(Ordering::SeqCst);

        if count == 0 {
            return None;
        }

        let mean = sum / f64::from(u32::try_from(count).unwrap_or(u32::MAX));

        Some(HistogramStats {
            sum,
            count,
            mean,
            buckets: self.state.buckets.clone(),
            bucket_counts: counts.clone(),
        })
    }

    /// Get metric name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Export as a snapshot
    #[must_use]
    pub fn snapshot(&self) -> Option<MetricSnapshot> {
        let stats = self.stats()?;
        Some(MetricSnapshot {
            name: self.name.clone(),
            metric_type: MetricType::Histogram,
            value: stats.mean,
            dimensions: self.dimensions.clone(),
            timestamp: Utc::now(),
            help: self.help.clone(),
        })
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            state: Arc::clone(&self.state),
            dimensions: self.dimensions.clone(),
        }
    }
}

/// Statistics from a histogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    /// Sum of all values
    pub sum: f64,
    /// Count of observations
    pub count: u64,
    /// Mean value
    pub mean: f64,
    /// Bucket boundaries
    pub buckets: Vec<f64>,
    /// Cumulative counts per bucket
    pub bucket_counts: Vec<u64>,
}

impl HistogramStats {
    /// Calculate percentile
    #[must_use]
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.count == 0 || self.buckets.is_empty() {
            return None;
        }

        let target = f64::from(u32::try_from(self.count).unwrap_or(u32::MAX)) * (p / 100.0);

        for (i, &bucket_count) in self.bucket_counts.iter().enumerate() {
            if f64::from(u32::try_from(bucket_count).unwrap_or(u32::MAX)) >= target {
                if i < self.buckets.len() {
                    return Some(self.buckets[i]);
                }
                // +Inf bucket
                return self.buckets.last().copied();
            }
        }

        self.buckets.last().copied()
    }

    /// Get median (50th percentile)
    #[must_use]
    pub fn median(&self) -> Option<f64> {
        self.percentile(50.0)
    }

    /// Get 95th percentile
    #[must_use]
    pub fn p95(&self) -> Option<f64> {
        self.percentile(95.0)
    }

    /// Get 99th percentile
    #[must_use]
    pub fn p99(&self) -> Option<f64> {
        self.percentile(99.0)
    }
}

// ============================================================================
// Metric Snapshot
// ============================================================================

/// A snapshot of a metric at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Current value
    pub value: f64,
    /// Dimensions
    pub dimensions: MetricDimensions,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Help text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

// ============================================================================
// Metrics Registry
// ============================================================================

/// Configuration for metrics registry
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Service name (used as prefix)
    pub service: String,
    /// Maximum dimensions per metric
    pub max_dimensions: usize,
    /// Default histogram buckets
    pub default_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            service: "pme".to_string(),
            max_dimensions: 10,
            default_buckets: vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        }
    }
}

/// Registry for managing metrics
#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    config: Arc<MetricsConfig>,
    counters: Arc<std::sync::Mutex<HashMap<String, Counter>>>,
    gauges: Arc<std::sync::Mutex<HashMap<String, Gauge>>>,
    histograms: Arc<std::sync::Mutex<HashMap<String, Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            config: Arc::new(MetricsConfig {
                service: service.into(),
                ..MetricsConfig::default()
            }),
            counters: Arc::new(std::sync::Mutex::new(HashMap::new())),
            gauges: Arc::new(std::sync::Mutex::new(HashMap::new())),
            histograms: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Create with configuration
    #[must_use]
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            config: Arc::new(config),
            counters: Arc::new(std::sync::Mutex::new(HashMap::new())),
            gauges: Arc::new(std::sync::Mutex::new(HashMap::new())),
            histograms: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a counter
    #[must_use]
    pub fn counter(&self, name: &str) -> Counter {
        let full_name = self.full_name(name);
        if let Ok(mut counters) = self.counters.lock() {
            if let Some(counter) = counters.get(&full_name) {
                return counter.clone();
            }
            let counter = Counter::new(&full_name);
            counters.insert(full_name.clone(), counter.clone());
            counter
        } else {
            Counter::new(&full_name)
        }
    }

    /// Get or create a gauge
    #[must_use]
    pub fn gauge(&self, name: &str) -> Gauge {
        let full_name = self.full_name(name);
        if let Ok(mut gauges) = self.gauges.lock() {
            if let Some(gauge) = gauges.get(&full_name) {
                return gauge.clone();
            }
            let gauge = Gauge::new(&full_name);
            gauges.insert(full_name.clone(), gauge.clone());
            gauge
        } else {
            Gauge::new(&full_name)
        }
    }

    /// Get or create a histogram
    #[must_use]
    pub fn histogram(&self, name: &str) -> Histogram {
        let full_name = self.full_name(name);
        if let Ok(mut histograms) = self.histograms.lock() {
            if let Some(histogram) = histograms.get(&full_name) {
                return histogram.clone();
            }
            let histogram = Histogram::new(&full_name);
            histograms.insert(full_name.clone(), histogram.clone());
            histogram
        } else {
            Histogram::new(&full_name)
        }
    }

    /// Create full metric name with prefix
    fn full_name(&self, name: &str) -> String {
        format!("{}_{}", self.config.service, name)
    }

    /// Get all metric snapshots
    #[must_use]
    pub fn snapshots(&self) -> Vec<MetricSnapshot> {
        let mut snapshots = Vec::new();

        if let Ok(counters) = self.counters.lock() {
            snapshots.extend(counters.values().map(|c| c.snapshot()));
        }

        if let Ok(gauges) = self.gauges.lock() {
            snapshots.extend(gauges.values().map(|g| g.snapshot()));
        }

        if let Ok(histograms) = self.histograms.lock() {
            snapshots.extend(histograms.values().filter_map(|h| h.snapshot()));
        }

        snapshots
    }

    /// Clear all metrics
    pub fn clear(&self) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.clear();
        }
        if let Ok(mut gauges) = self.gauges.lock() {
            gauges.clear();
        }
        if let Ok(mut histograms) = self.histograms.lock() {
            histograms.clear();
        }
    }

    /// Get metrics summary
    #[must_use]
    pub fn summary(&self) -> MetricsSummary {
        let counters = if let Ok(c) = self.counters.lock() {
            c.len()
        } else {
            0
        };
        let gauges = if let Ok(g) = self.gauges.lock() {
            g.len()
        } else {
            0
        };
        let histograms = if let Ok(h) = self.histograms.lock() {
            h.len()
        } else {
            0
        };

        MetricsSummary {
            service: self.config.service.clone(),
            counters,
            gauges,
            histograms,
            total: counters + gauges + histograms,
        }
    }
}

/// Summary of metrics in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Service name
    pub service: String,
    /// Number of counters
    pub counters: usize,
    /// Number of gauges
    pub gauges: usize,
    /// Number of histograms
    pub histograms: usize,
    /// Total metrics
    pub total: usize,
}

// ============================================================================
// RUM Metrics Collector
// ============================================================================

/// Real User Monitoring metrics collector
#[derive(Debug, Clone)]
pub struct RumCollector {
    registry: MetricsRegistry,
}

impl RumCollector {
    /// Create a new RUM collector
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            registry: MetricsRegistry::new(service),
        }
    }

    /// Record page load timing
    pub fn record_page_load(&self, page: &str, duration_ms: f64) {
        self.registry
            .histogram("page_load_duration_ms")
            .with_dimension("page", page)
            .observe(duration_ms);

        self.registry
            .counter("page_loads_total")
            .with_dimension("page", page)
            .increment();
    }

    /// Record user interaction
    pub fn record_interaction(&self, interaction_type: &str, element: &str) {
        self.registry
            .counter("user_interactions_total")
            .with_dimension("type", interaction_type)
            .with_dimension("element", element)
            .increment();
    }

    /// Record API call timing
    pub fn record_api_call(&self, endpoint: &str, method: &str, duration_ms: f64, success: bool) {
        self.registry
            .histogram("api_call_duration_ms")
            .with_dimension("endpoint", endpoint)
            .with_dimension("method", method)
            .observe(duration_ms);

        self.registry
            .counter("api_calls_total")
            .with_dimension("endpoint", endpoint)
            .with_dimension("method", method)
            .with_dimension("success", if success { "true" } else { "false" })
            .increment();

        if !success {
            self.registry
                .counter("api_errors_total")
                .with_dimension("endpoint", endpoint)
                .with_dimension("method", method)
                .increment();
        }
    }

    /// Record Web Vitals
    pub fn record_web_vital(&self, metric_name: &str, value: f64) {
        self.registry
            .histogram("web_vitals")
            .with_dimension("metric", metric_name)
            .observe(value);
    }

    /// Record LCP (Largest Contentful Paint)
    pub fn record_lcp(&self, value: f64) {
        self.record_web_vital("lcp", value);
    }

    /// Record FID (First Input Delay)
    pub fn record_fid(&self, value: f64) {
        self.record_web_vital("fid", value);
    }

    /// Record CLS (Cumulative Layout Shift)
    pub fn record_cls(&self, value: f64) {
        self.record_web_vital("cls", value);
    }

    /// Record TTFB (Time to First Byte)
    pub fn record_ttfb(&self, value: f64) {
        self.record_web_vital("ttfb", value);
    }

    /// Record error
    pub fn record_error(&self, error_type: &str, message: &str) {
        self.registry
            .counter("errors_total")
            .with_dimension("type", error_type)
            .with_dimension("message", message)
            .increment();
    }

    /// Set active users gauge
    pub fn set_active_users(&self, count: u64) {
        self.registry.gauge("active_users").set(count as f64);
    }

    /// Record session duration
    pub fn record_session_duration(&self, duration_seconds: f64) {
        self.registry
            .histogram("session_duration_seconds")
            .observe(duration_seconds);
    }

    /// Get the underlying registry
    #[must_use]
    pub const fn registry(&self) -> &MetricsRegistry {
        &self.registry
    }

    /// Get all metrics
    #[must_use]
    pub fn snapshots(&self) -> Vec<MetricSnapshot> {
        self.registry.snapshots()
    }

    /// Get summary
    #[must_use]
    pub fn summary(&self) -> MetricsSummary {
        self.registry.summary()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_dimensions() {
        let dims = MetricDimensions::new()
            .with("method", "GET")
            .with("status", "200");

        assert_eq!(dims.get("method"), Some("GET"));
        assert_eq!(dims.get("status"), Some("200"));
        assert_eq!(dims.get("other"), None);
        assert_eq!(dims.len(), 2);
    }

    #[test]
    fn test_metric_dimensions_key() {
        let dims1 = MetricDimensions::new()
            .with("a", "1")
            .with("b", "2");

        let dims2 = MetricDimensions::new()
            .with("b", "2")
            .with("a", "1");

        // Keys should be the same regardless of order
        assert_eq!(dims1.to_key(), dims2.to_key());
    }

    #[test]
    fn test_counter_increment() {
        let counter = Counter::new("test_counter");

        assert_eq!(counter.get(), 0);
        counter.increment();
        assert_eq!(counter.get(), 1);
        counter.increment_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_with_dimensions() {
        let counter = Counter::new("test")
            .with_dimension("method", "GET")
            .with_help("Test counter");

        assert_eq!(counter.dimensions.get("method"), Some("GET"));
        assert_eq!(counter.help, Some("Test counter".to_string()));
    }

    #[test]
    fn test_counter_snapshot() {
        let counter = Counter::new("test_counter");
        counter.increment_by(10);

        let snapshot = counter.snapshot();
        assert_eq!(snapshot.name, "test_counter");
        assert_eq!(snapshot.metric_type, MetricType::Counter);
        assert!((snapshot.value - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gauge_operations() {
        let gauge = Gauge::new("test_gauge");

        gauge.set(10.0);
        assert!((gauge.get() - 10.0).abs() < 0.001);

        gauge.increment();
        assert!((gauge.get() - 11.0).abs() < 0.001);

        gauge.decrement();
        assert!((gauge.get() - 10.0).abs() < 0.001);

        gauge.add(5.0);
        assert!((gauge.get() - 15.0).abs() < 0.001);

        gauge.sub(3.0);
        assert!((gauge.get() - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_gauge_negative() {
        let gauge = Gauge::new("test_gauge");
        gauge.set(-10.0);
        assert!((gauge.get() - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_gauge_snapshot() {
        let gauge = Gauge::new("test_gauge")
            .with_dimension("host", "localhost");
        gauge.set(42.0);

        let snapshot = gauge.snapshot();
        assert_eq!(snapshot.name, "test_gauge");
        assert_eq!(snapshot.metric_type, MetricType::Gauge);
        assert!((snapshot.value - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_histogram_observe() {
        let hist = Histogram::new("test_histogram");

        hist.observe(1.0);
        hist.observe(2.0);
        hist.observe(3.0);

        let stats = hist.stats();
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert!((stats.sum - 6.0).abs() < f64::EPSILON);
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_histogram_percentiles() {
        let buckets = vec![1.0, 5.0, 10.0, 50.0, 100.0];
        let hist = Histogram::with_buckets("test", buckets);

        // Add values: 1, 2, 3, ..., 10
        for i in 1..=10 {
            hist.observe(f64::from(i));
        }

        let stats = hist.stats().unwrap();

        let median = stats.median();
        assert!(median.is_some());

        let p95 = stats.p95();
        assert!(p95.is_some());
    }

    #[test]
    fn test_histogram_empty() {
        let hist = Histogram::new("test");
        assert!(hist.stats().is_none());
    }

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new("test_service");

        let counter = registry.counter("requests");
        counter.increment();

        let gauge = registry.gauge("connections");
        gauge.set(5.0);

        let hist = registry.histogram("latency");
        hist.observe(100.0);

        let summary = registry.summary();
        assert_eq!(summary.counters, 1);
        assert_eq!(summary.gauges, 1);
        assert_eq!(summary.histograms, 1);
        assert_eq!(summary.total, 3);
    }

    #[test]
    fn test_metrics_registry_full_name() {
        let registry = MetricsRegistry::new("myapp");
        let counter = registry.counter("requests");

        assert!(counter.name().starts_with("myapp_"));
    }

    #[test]
    fn test_metrics_registry_snapshots() {
        let registry = MetricsRegistry::new("test");

        registry.counter("c1").increment();
        registry.gauge("g1").set(10.0);

        let snapshots = registry.snapshots();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_metrics_registry_clear() {
        let registry = MetricsRegistry::new("test");

        registry.counter("c1").increment();
        assert!(!registry.snapshots().is_empty());

        registry.clear();
        assert!(registry.snapshots().is_empty());
    }

    #[test]
    fn test_rum_collector_page_load() {
        let rum = RumCollector::new("test");

        rum.record_page_load("/home", 250.0);
        rum.record_page_load("/about", 150.0);

        let snapshots = rum.snapshots();
        assert!(snapshots.iter().any(|s| s.name.contains("page_load")));
    }

    #[test]
    fn test_rum_collector_api_call() {
        let rum = RumCollector::new("test");

        rum.record_api_call("/api/users", "GET", 50.0, true);
        rum.record_api_call("/api/users", "POST", 100.0, false);

        let snapshots = rum.snapshots();
        assert!(snapshots.iter().any(|s| s.name.contains("api_call")));
        assert!(snapshots.iter().any(|s| s.name.contains("api_error")));
    }

    #[test]
    fn test_rum_collector_web_vitals() {
        let rum = RumCollector::new("test");

        rum.record_lcp(2.5);
        rum.record_fid(0.1);
        rum.record_cls(0.05);
        rum.record_ttfb(0.3);

        let snapshots = rum.snapshots();
        let vitals = snapshots.iter().filter(|s| s.name.contains("web_vitals"));
        // Should have observed all 4 vitals in one histogram
        assert!(vitals.count() >= 1);
    }

    #[test]
    fn test_rum_collector_errors() {
        let rum = RumCollector::new("test");

        rum.record_error("TypeError", "Cannot read property");
        rum.record_error("NetworkError", "Failed to fetch");

        let snapshots = rum.snapshots();
        assert!(snapshots.iter().any(|s| s.name.contains("errors")));
    }

    #[test]
    fn test_rum_collector_active_users() {
        let rum = RumCollector::new("test");

        rum.set_active_users(42);

        let snapshots = rum.snapshots();
        let active_users = snapshots.iter().find(|s| s.name.contains("active_users"));
        assert!(active_users.is_some());
        assert!((active_users.unwrap().value - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rum_collector_session_duration() {
        let rum = RumCollector::new("test");

        rum.record_session_duration(300.0); // 5 minutes

        let snapshots = rum.snapshots();
        assert!(snapshots.iter().any(|s| s.name.contains("session_duration")));
    }

    #[test]
    fn test_metric_value() {
        let mv = MetricValue::new(42.0);
        assert!((mv.value - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_metric_dimensions_from_slice() {
        let dims = MetricDimensions::from_slice(&[
            ("method", "GET"),
            ("status", "200"),
        ]);

        assert_eq!(dims.get("method"), Some("GET"));
        assert_eq!(dims.get("status"), Some("200"));
    }

    #[test]
    fn test_histogram_stats_percentile_edge_cases() {
        let stats = HistogramStats {
            sum: 0.0,
            count: 0,
            mean: 0.0,
            buckets: vec![1.0, 5.0, 10.0],
            bucket_counts: vec![0, 0, 0, 0],
        };

        // Empty histogram should return None
        assert!(stats.percentile(50.0).is_none());
    }

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();

        assert_eq!(config.service, "pme");
        assert_eq!(config.max_dimensions, 10);
        assert!(!config.default_buckets.is_empty());
    }

    #[test]
    fn test_counter_clone_shares_state() {
        let counter1 = Counter::new("test");
        counter1.increment();

        let counter2 = counter1.clone();
        assert_eq!(counter2.get(), 1);

        counter2.increment();
        assert_eq!(counter1.get(), 2); // Shared state
    }

    #[test]
    fn test_gauge_clone_shares_state() {
        let gauge1 = Gauge::new("test");
        gauge1.set(10.0);

        let gauge2 = gauge1.clone();
        assert!((gauge2.get() - 10.0).abs() < 0.001);

        gauge2.set(20.0);
        assert!((gauge1.get() - 20.0).abs() < 0.001); // Shared state
    }

    #[test]
    fn test_histogram_clone_shares_state() {
        let hist1 = Histogram::new("test");
        hist1.observe(5.0);

        let hist2 = hist1.clone();
        let stats = hist2.stats();
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().count, 1);

        hist2.observe(10.0);
        let stats = hist1.stats().unwrap();
        assert_eq!(stats.count, 2); // Shared state
    }
}
