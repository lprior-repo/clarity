#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

//! Distributed Tracing Infrastructure for PME
//!
//! Provides distributed tracing support with:
//! - Span creation and propagation
//! - Trace context across service boundaries
//! - Timing and duration tracking
//! - Parent-child span relationships
//!
//! # Example
//!
//! ```rust,ignore
//! use pme::infra::tracing::{Tracer, SpanBuilder, SpanKind};
//!
//! let tracer = Tracer::new("pme-api");
//!
//! let span = SpanBuilder::new("process_request")
//!     .with_kind(SpanKind::Server)
//!     .with_parent(parent_context)
//!     .build();
//!
//! tracer.start(&span);
//! // ... do work ...
//! tracer.end(&span);
//! ```

use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during tracing operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TracingError {
  /// Invalid trace ID format
  #[error("invalid trace ID: {0}")]
  InvalidTraceId(String),

  /// Invalid span ID format
  #[error("invalid span ID: {0}")]
  InvalidSpanId(String),

  /// Span not found
  #[error("span not found: {0}")]
  SpanNotFound(String),

  /// Parent span not found
  #[error("parent span not found: {0}")]
  ParentNotFound(String),

  /// Invalid span state transition
  #[error("invalid state transition: {0}")]
  InvalidTransition(String),

  /// Trace context extraction failed
  #[error("context extraction failed: {0}")]
  ContextExtractionFailed(String),

  /// Trace context injection failed
  #[error("context injection failed: {0}")]
  ContextInjectionFailed(String),
}

// ============================================================================
// Trace and Span IDs
// ============================================================================

/// Trace identifier (128-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId([u8; 16]);

impl TraceId {
  /// Generate a new random trace ID
  #[must_use]
  pub fn generate() -> Self {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0u64, |d| d.as_nanos() as u64);

    // Combine timestamp with some randomness for uniqueness
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&timestamp.to_le_bytes());
    // Use a counter for the second half
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    bytes[8..].copy_from_slice(&counter.to_le_bytes());

    Self(bytes)
  }

  /// Create from bytes
  #[must_use]
  pub const fn from_bytes(bytes: [u8; 16]) -> Self {
    Self(bytes)
  }

  /// Get as bytes
  #[must_use]
  pub const fn as_bytes(&self) -> &[u8; 16] {
    &self.0
  }

  /// Parse from hex string
  /// # Errors
  ///
  pub fn parse(s: &str) -> Result<Self, TracingError> {
    if s.len() != 32 {
      return Err(TracingError::InvalidTraceId(s.to_string()));
    }

    let mut bytes = [0u8; 16];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
      if i >= 16 {
        break;
      }
      let hex =
        std::str::from_utf8(chunk).map_err(|_| TracingError::InvalidTraceId(s.to_string()))?;
      bytes[i] =
        u8::from_str_radix(hex, 16).map_err(|_| TracingError::InvalidTraceId(s.to_string()))?;
    }

    Ok(Self(bytes))
  }

  /// Convert to hex string
  #[must_use]
  pub fn to_hex(&self) -> String {
    self.0.iter().map(|b| format!("{b:02x}")).join("")
  }
}

impl fmt::Display for TraceId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_hex())
  }
}

impl Default for TraceId {
  fn default() -> Self {
    Self::generate()
  }
}

/// Span identifier (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId([u8; 8]);

impl SpanId {
  /// Generate a new random span ID
  #[must_use]
  pub fn generate() -> Self {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0u64, |d| d.as_nanos() as u64);

    // Mix with counter for uniqueness
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mixed = timestamp.wrapping_add(counter);

    Self(mixed.to_le_bytes())
  }

  /// Create from bytes
  #[must_use]
  pub const fn from_bytes(bytes: [u8; 8]) -> Self {
    Self(bytes)
  }

  /// Get as bytes
  #[must_use]
  pub const fn as_bytes(&self) -> &[u8; 8] {
    &self.0
  }

  /// Parse from hex string
  pub fn parse(s: &str) -> Result<Self, TracingError> {
    if s.len() != 16 {
      return Err(TracingError::InvalidSpanId(s.to_string()));
    }

    let mut bytes = [0u8; 8];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
      if i >= 8 {
        break;
      }
      let hex =
        std::str::from_utf8(chunk).map_err(|_| TracingError::InvalidSpanId(s.to_string()))?;
      bytes[i] =
        u8::from_str_radix(hex, 16).map_err(|_| TracingError::InvalidSpanId(s.to_string()))?;
    }

    Ok(Self(bytes))
  }

  /// Convert to hex string
  #[must_use]
  pub fn to_hex(&self) -> String {
    self.0.iter().map(|b| format!("{b:02x}")).join("")
  }
}

impl fmt::Display for SpanId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_hex())
  }
}

impl Default for SpanId {
  fn default() -> Self {
    Self::generate()
  }
}

// ============================================================================
// Span Kind
// ============================================================================

/// Kind of span (client, server, producer, consumer, internal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
  /// Client-side span (outgoing request)
  Client,
  /// Server-side span (incoming request)
  Server,
  /// Producer span (message sent to broker)
  Producer,
  /// Consumer span (message received from broker)
  Consumer,
  /// Internal span (not crossing boundaries)
  Internal,
}

impl SpanKind {
  /// Get string representation for `OpenTelemetry`
  #[must_use]
  pub const fn as_otlp(&self) -> &'static str {
    match self {
      Self::Client => "SPAN_KIND_CLIENT",
      Self::Server => "SPAN_KIND_SERVER",
      Self::Producer => "SPAN_KIND_PRODUCER",
      Self::Consumer => "SPAN_KIND_CONSUMER",
      Self::Internal => "SPAN_KIND_INTERNAL",
    }
  }
}

// ============================================================================
// Span Status
// ============================================================================

/// Status of a span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
  /// Span completed successfully
  Ok,
  /// Span was cancelled
  Cancelled,
  /// Span ended with an error
  Error,
}

// ============================================================================
// Span State
// ============================================================================

/// State of a span lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanState {
  /// Span has been created but not started
  Unstarted,
  /// Span is currently active
  Started,
  /// Span has ended
  Ended,
}

// ============================================================================
// Trace Context
// ============================================================================

/// Trace context for propagation across boundaries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceContext {
  /// Trace ID
  pub trace_id: TraceId,
  /// Current span ID
  pub span_id: SpanId,
  /// Parent span ID (if any)
  pub parent_span_id: Option<SpanId>,
  /// Trace flags (e.g., sampled)
  pub flags: TraceFlags,
  /// Baggage items (key-value pairs propagated with trace)
  pub baggage: HashMap<String, String>,
}

impl TraceContext {
  /// Create a new trace context
  #[must_use]
  pub fn new(trace_id: TraceId, span_id: SpanId) -> Self {
    Self {
      trace_id,
      span_id,
      parent_span_id: None,
      flags: TraceFlags::default(),
      baggage: HashMap::new(),
    }
  }

  /// Create a child context
  #[must_use]
  pub fn child(&self, new_span_id: SpanId) -> Self {
    Self {
      trace_id: self.trace_id,
      span_id: new_span_id,
      parent_span_id: Some(self.span_id),
      flags: self.flags,
      baggage: self.baggage.clone(),
    }
  }

  /// Set parent span
  #[must_use]
  pub const fn with_parent(mut self, parent_id: SpanId) -> Self {
    self.parent_span_id = Some(parent_id);
    self
  }

  /// Add baggage item
  #[must_use]
  pub fn with_baggage(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.baggage.insert(key.into(), value.into());
    self
  }

  /// Set sampled flag
  #[must_use]
  pub const fn with_sampled(mut self, sampled: bool) -> Self {
    self.flags.sampled = sampled;
    self
  }

  /// Extract from W3C traceparent header
  pub fn from_traceparent(header: &str) -> Result<Self, TracingError> {
    // Format: version-traceid-spanid-flags
    // Example: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
    let parts: Vec<&str> = header.split('-').collect();
    if parts.len() != 4 {
      return Err(TracingError::ContextExtractionFailed(
        "invalid traceparent format".to_string(),
      ));
    }

    let version = u8::from_str_radix(parts[0], 16)
      .map_err(|_| TracingError::ContextExtractionFailed("invalid version".to_string()))?;
    if version != 0 {
      return Err(TracingError::ContextExtractionFailed(
        "unsupported version".to_string(),
      ));
    }

    let trace_id = TraceId::parse(parts[1])?;
    let span_id = SpanId::parse(parts[2])?;
    let flags_byte = u8::from_str_radix(parts[3], 16)
      .map_err(|_| TracingError::ContextExtractionFailed("invalid flags".to_string()))?;

    Ok(Self {
      trace_id,
      span_id,
      parent_span_id: None,
      flags: TraceFlags {
        sampled: (flags_byte & 0x01) != 0,
      },
      baggage: HashMap::new(),
    })
  }

  /// Convert to W3C traceparent header
  #[must_use]
  pub fn to_traceparent(&self) -> String {
    let flags = if self.flags.sampled { "01" } else { "00" };
    format!(
      "00-{}-{}-{}",
      self.trace_id.to_hex(),
      self.span_id.to_hex(),
      flags
    )
  }
}

impl Default for TraceContext {
  fn default() -> Self {
    Self::new(TraceId::generate(), SpanId::generate())
  }
}

/// Trace flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceFlags {
  /// Whether this trace should be sampled
  pub sampled: bool,
}

impl Default for TraceFlags {
  fn default() -> Self {
    Self { sampled: true }
  }
}

// ============================================================================
// Span
// ============================================================================

/// A span represents a unit of work in a distributed trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
  /// Unique span identifier
  pub span_id: SpanId,
  /// Trace this span belongs to
  pub trace_id: TraceId,
  /// Parent span ID (if nested)
  pub parent_span_id: Option<SpanId>,
  /// Span name (operation name)
  pub name: String,
  /// Kind of span
  pub kind: SpanKind,
  /// Start time
  pub start_time: Option<DateTime<Utc>>,
  /// End time
  pub end_time: Option<DateTime<Utc>>,
  /// Duration in milliseconds
  pub duration_ms: Option<u64>,
  /// Span status
  pub status: SpanStatus,
  /// Span state
  pub state: SpanState,
  /// Attributes (key-value pairs)
  pub attributes: HashMap<String, AttributeValue>,
  /// Events (logs/annotations)
  pub events: Vec<SpanEvent>,
  /// Service name
  pub service: String,
}

/// Attribute value (supports multiple types)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
  /// String value
  String(String),
  /// Integer value
  Int(i64),
  /// Float value
  Float(f64),
  /// Boolean value
  Bool(bool),
  /// String array
  StringArray(Vec<String>),
}

/// An event that occurred during a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
  /// Event name
  pub name: String,
  /// Event timestamp
  pub timestamp: DateTime<Utc>,
  /// Event attributes
  pub attributes: HashMap<String, AttributeValue>,
}

impl Span {
  /// Create a new span
  #[must_use]
  pub fn new(
    name: impl Into<String>,
    trace_id: TraceId,
    span_id: SpanId,
    service: impl Into<String>,
  ) -> Self {
    Self {
      span_id,
      trace_id,
      parent_span_id: None,
      name: name.into(),
      kind: SpanKind::Internal,
      start_time: None,
      end_time: None,
      duration_ms: None,
      status: SpanStatus::Ok,
      state: SpanState::Unstarted,
      attributes: HashMap::new(),
      events: Vec::new(),
      service: service.into(),
    }
  }

  /// Set parent span
  #[must_use]
  pub const fn with_parent(mut self, parent_id: SpanId) -> Self {
    self.parent_span_id = Some(parent_id);
    self
  }

  /// Set span kind
  #[must_use]
  pub const fn with_kind(mut self, kind: SpanKind) -> Self {
    self.kind = kind;
    self
  }

  /// Add attribute
  #[must_use]
  pub fn with_attribute(mut self, key: impl Into<String>, value: AttributeValue) -> Self {
    self.attributes.insert(key.into(), value);
    self
  }

  /// Add event
  #[must_use]
  pub fn with_event(mut self, event: SpanEvent) -> Self {
    self.events.push(event);
    self
  }

  /// Start the span
  pub fn start(&mut self) -> Result<(), TracingError> {
    if self.state != SpanState::Unstarted {
      return Err(TracingError::InvalidTransition(format!(
        "cannot start span in state {:?}",
        self.state
      )));
    }
    self.start_time = Some(Utc::now());
    self.state = SpanState::Started;
    Ok(())
  }

  /// End the span
  pub fn end(&mut self) -> Result<(), TracingError> {
    if self.state != SpanState::Started {
      return Err(TracingError::InvalidTransition(format!(
        "cannot end span in state {:?}",
        self.state
      )));
    }
    self.end_time = Some(Utc::now());

    // Calculate duration
    if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
      let duration = end.signed_duration_since(start);
      self.duration_ms = Some(duration.num_milliseconds().try_into().unwrap_or(0));
    }

    self.state = SpanState::Ended;
    Ok(())
  }

  /// End the span with error status
  pub fn end_with_error(&mut self, error_message: &str) -> Result<(), TracingError> {
    self.status = SpanStatus::Error;
    self.attributes.insert(
      "error.message".to_string(),
      AttributeValue::String(error_message.to_string()),
    );
    self.end()
  }

  /// Get trace context from this span
  #[must_use]
  pub fn context(&self) -> TraceContext {
    TraceContext {
      trace_id: self.trace_id,
      span_id: self.span_id,
      parent_span_id: self.parent_span_id,
      flags: TraceFlags::default(),
      baggage: HashMap::new(),
    }
  }

  /// Check if this span is a root span (no parent)
  #[must_use]
  pub const fn is_root(&self) -> bool {
    self.parent_span_id.is_none()
  }
}

// ============================================================================
// Span Builder
// ============================================================================

/// Builder for creating spans
#[derive(Debug, Clone)]
pub struct SpanBuilder {
  name: String,
  kind: SpanKind,
  parent_context: Option<TraceContext>,
  attributes: HashMap<String, AttributeValue>,
  service: String,
}

impl SpanBuilder {
  /// Create a new span builder
  #[must_use]
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      kind: SpanKind::Internal,
      parent_context: None,
      attributes: HashMap::new(),
      service: "pme".to_string(),
    }
  }

  /// Set span kind
  #[must_use]
  pub const fn with_kind(mut self, kind: SpanKind) -> Self {
    self.kind = kind;
    self
  }

  /// Set parent context
  #[must_use]
  pub fn with_parent(mut self, context: TraceContext) -> Self {
    self.parent_context = Some(context);
    self
  }

  /// Add attribute
  #[must_use]
  pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self
      .attributes
      .insert(key.into(), AttributeValue::String(value.into()));
    self
  }

  /// Add integer attribute
  #[must_use]
  pub fn with_int_attribute(mut self, key: impl Into<String>, value: i64) -> Self {
    self
      .attributes
      .insert(key.into(), AttributeValue::Int(value));
    self
  }

  /// Set service name
  #[must_use]
  pub fn with_service(mut self, service: impl Into<String>) -> Self {
    self.service = service.into();
    self
  }

  /// Build the span
  #[must_use]
  pub fn build(self) -> Span {
    let span_id = SpanId::generate();

    let (trace_id, parent_span_id) = match self.parent_context {
      Some(ctx) => (ctx.trace_id, Some(ctx.span_id)),
      None => (TraceId::generate(), None),
    };

    let mut span = Span::new(self.name, trace_id, span_id, self.service);
    span.kind = self.kind;
    span.parent_span_id = parent_span_id;
    span.attributes = self.attributes;
    span
  }

  /// Build as child of another span
  #[must_use]
  pub fn build_child(self, parent: &Span) -> Span {
    let span_id = SpanId::generate();
    let mut span = Span::new(self.name, parent.trace_id, span_id, &self.service);
    span.kind = self.kind;
    span.parent_span_id = Some(parent.span_id);
    span.attributes = self.attributes;
    span
  }
}

// ============================================================================
// Tracer
// ============================================================================

/// Configuration for the tracer
#[derive(Debug, Clone)]
pub struct TracerConfig {
  /// Service name
  pub service: String,
  /// Sampling rate (0.0 to 1.0)
  pub sampling_rate: f64,
  /// Maximum attributes per span
  pub max_attributes: usize,
  /// Maximum events per span
  pub max_events: usize,
}

impl Default for TracerConfig {
  fn default() -> Self {
    Self {
      service: "pme".to_string(),
      sampling_rate: 1.0,
      max_attributes: 128,
      max_events: 128,
    }
  }
}

/// Tracer for creating and managing spans
#[derive(Debug, Clone)]
pub struct Tracer {
  config: Arc<TracerConfig>,
  spans: Arc<std::sync::Mutex<Vec<Span>>>,
}

impl Tracer {
  /// Create a new tracer with defaults
  #[must_use]
  pub fn new(service: impl Into<String>) -> Self {
    Self {
      config: Arc::new(TracerConfig {
        service: service.into(),
        ..TracerConfig::default()
      }),
      spans: Arc::new(std::sync::Mutex::new(Vec::new())),
    }
  }

  /// Create with configuration
  #[must_use]
  pub fn with_config(config: TracerConfig) -> Self {
    Self {
      config: Arc::new(config),
      spans: Arc::new(std::sync::Mutex::new(Vec::new())),
    }
  }

  /// Create a new trace (root span)
  #[must_use]
  pub fn start_trace(&self, name: impl Into<String>) -> Span {
    let mut span = SpanBuilder::new(name)
      .with_service(self.config.service.clone())
      .build();
    span.start().ok();
    self.record_span(&span);
    span
  }

  /// Create a child span
  #[must_use]
  pub fn start_child(&self, name: impl Into<String>, parent: &Span) -> Span {
    let mut span = SpanBuilder::new(name)
      .with_service(self.config.service.clone())
      .build_child(parent);
    span.start().ok();
    self.record_span(&span);
    span
  }

  /// Start a span from context
  #[must_use]
  pub fn start_from_context(&self, name: impl Into<String>, context: &TraceContext) -> Span {
    let mut span = SpanBuilder::new(name)
      .with_service(self.config.service.clone())
      .with_parent(context.clone())
      .build();
    span.start().ok();
    self.record_span(&span);
    span
  }

  /// End a span
  pub fn end_span(&self, span: &mut Span) -> Result<(), TracingError> {
    span.end()
  }

  /// Record a span
  fn record_span(&self, span: &Span) {
    if let Ok(mut spans) = self.spans.lock() {
      spans.push(span.clone());
    }
  }

  /// Get all recorded spans
  #[must_use]
  pub fn get_spans(&self) -> Vec<Span> {
    if let Ok(spans) = self.spans.lock() {
      spans.clone()
    } else {
      Vec::new()
    }
  }

  /// Get spans by trace ID
  #[must_use]
  pub fn get_spans_by_trace(&self, trace_id: TraceId) -> Vec<Span> {
    self
      .get_spans()
      .into_iter()
      .filter(|s| s.trace_id == trace_id)
      .collect()
  }

  /// Clear all recorded spans
  pub fn clear(&self) {
    if let Ok(mut spans) = self.spans.lock() {
      spans.clear();
    }
  }

  /// Check if should sample based on sampling rate
  #[must_use]
  pub fn should_sample(&self) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0u128, |d| d.as_nanos());
    let sampled = (nanos as f64 % 100.0) / 100.0;
    sampled < self.config.sampling_rate
  }

  /// Get configuration
  #[must_use]
  pub fn config(&self) -> &TracerConfig {
    &self.config
  }
}

// ============================================================================
// Trace Summary
// ============================================================================

/// Summary of a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
  /// Trace ID
  pub trace_id: TraceId,
  /// Root span name
  pub root_span: String,
  /// Total number of spans
  pub span_count: usize,
  /// Total duration in milliseconds
  pub total_duration_ms: u64,
  /// Number of error spans
  pub error_count: usize,
  /// Service names involved
  pub services: Vec<String>,
}

impl TraceSummary {
  /// Create a summary from spans
  #[must_use]
  pub fn from_spans(spans: &[Span]) -> Option<Self> {
    if spans.is_empty() {
      return None;
    }

    let trace_id = spans[0].trace_id;
    let root_span = spans
      .iter()
      .find(|s| s.is_root())
      .map_or_else(|| spans[0].name.clone(), |s| s.name.clone());

    let span_count = spans.len();
    let error_count = spans
      .iter()
      .filter(|s| s.status == SpanStatus::Error)
      .count();

    let total_duration_ms = spans
      .iter()
      .filter_map(|s| s.duration_ms)
      .max()
      .unwrap_or(0);

    let services = spans.iter().map(|s| s.service.clone()).unique().collect();

    Some(Self {
      trace_id,
      root_span,
      span_count,
      total_duration_ms,
      error_count,
      services,
    })
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trace_id_generation() {
    let id1 = TraceId::generate();
    let id2 = TraceId::generate();
    assert_ne!(id1, id2, "Trace IDs should be unique");
  }

  #[test]
  fn test_trace_id_hex() {
    let id = TraceId::from_bytes([0x12; 16]);
    let hex = id.to_hex();
    assert_eq!(hex.len(), 32);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn test_trace_id_parse() {
    let id = TraceId::generate();
    let hex = id.to_hex();
    let parsed = TraceId::parse(&hex);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), id);
  }

  #[test]
  fn test_trace_id_parse_invalid() {
    assert!(TraceId::parse("invalid").is_err());
    assert!(TraceId::parse("00112233445566778899aabbccddeeff").is_ok());
    assert!(TraceId::parse("00112233445566778899aabbccddee").is_err()); // Too short
  }

  #[test]
  fn test_span_id_generation() {
    let id1 = SpanId::generate();
    let id2 = SpanId::generate();
    assert_ne!(id1, id2, "Span IDs should be unique");
  }

  #[test]
  fn test_span_id_hex() {
    let id = SpanId::from_bytes([0xab; 8]);
    let hex = id.to_hex();
    assert_eq!(hex.len(), 16);
  }

  #[test]
  fn test_span_id_parse() {
    let id = SpanId::generate();
    let hex = id.to_hex();
    let parsed = SpanId::parse(&hex);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), id);
  }

  #[test]
  fn test_trace_context_creation() {
    let trace_id = TraceId::generate();
    let span_id = SpanId::generate();
    let ctx = TraceContext::new(trace_id, span_id);

    assert_eq!(ctx.trace_id, trace_id);
    assert_eq!(ctx.span_id, span_id);
    assert!(ctx.parent_span_id.is_none());
  }

  #[test]
  fn test_trace_context_child() {
    let parent = TraceContext::default();
    let child_span_id = SpanId::generate();
    let child = parent.child(child_span_id);

    assert_eq!(child.trace_id, parent.trace_id);
    assert_eq!(child.span_id, child_span_id);
    assert_eq!(child.parent_span_id, Some(parent.span_id));
  }

  #[test]
  fn test_trace_context_traceparent() {
    let ctx = TraceContext::default().with_sampled(true);
    let header = ctx.to_traceparent();

    assert!(header.starts_with("00-"));
    assert_eq!(header.len(), 55); // "00-" + 32 + "-" + 16 + "-" + 2

    let parsed = TraceContext::from_traceparent(&header);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert_eq!(parsed.trace_id, ctx.trace_id);
    assert_eq!(parsed.span_id, ctx.span_id);
  }

  #[test]
  fn test_trace_context_traceparent_invalid() {
    assert!(TraceContext::from_traceparent("invalid").is_err());
    assert!(TraceContext::from_traceparent(
      "01-00112233445566778899aabbccddeeff-0011223344556677-01"
    )
    .is_err()); // Invalid version
  }

  #[test]
  fn test_span_creation() {
    let trace_id = TraceId::generate();
    let span_id = SpanId::generate();
    let span = Span::new("test_operation", trace_id, span_id, "test-service");

    assert_eq!(span.name, "test_operation");
    assert_eq!(span.trace_id, trace_id);
    assert_eq!(span.span_id, span_id);
    assert!(span.is_root());
    assert_eq!(span.state, SpanState::Unstarted);
  }

  #[test]
  fn test_span_lifecycle() {
    let mut span = Span::new("test", TraceId::generate(), SpanId::generate(), "svc");

    assert!(span.start().is_ok());
    assert_eq!(span.state, SpanState::Started);
    assert!(span.start_time.is_some());

    // Can't start again
    assert!(span.start().is_err());

    assert!(span.end().is_ok());
    assert_eq!(span.state, SpanState::Ended);
    assert!(span.end_time.is_some());
    assert!(span.duration_ms.is_some());

    // Can't end again
    assert!(span.end().is_err());
  }

  #[test]
  fn test_span_with_parent() {
    let parent_id = SpanId::generate();
    let span =
      Span::new("child", TraceId::generate(), SpanId::generate(), "svc").with_parent(parent_id);

    assert_eq!(span.parent_span_id, Some(parent_id));
    assert!(!span.is_root());
  }

  #[test]
  fn test_span_with_error() {
    let mut span = Span::new("test", TraceId::generate(), SpanId::generate(), "svc");
    span.start().ok();

    assert!(span.end_with_error("Something went wrong").is_ok());
    assert_eq!(span.status, SpanStatus::Error);
    assert!(span.attributes.contains_key("error.message"));
  }

  #[test]
  fn test_span_builder() {
    let span = SpanBuilder::new("operation")
      .with_kind(SpanKind::Server)
      .with_attribute("key", "value")
      .with_int_attribute("count", 42)
      .with_service("my-service")
      .build();

    assert_eq!(span.name, "operation");
    assert_eq!(span.kind, SpanKind::Server);
    assert_eq!(span.service, "my-service");
    assert_eq!(
      span.attributes.get("key"),
      Some(&AttributeValue::String("value".to_string()))
    );
    assert_eq!(span.attributes.get("count"), Some(&AttributeValue::Int(42)));
  }

  #[test]
  fn test_span_builder_child() {
    let parent = Span::new("parent", TraceId::generate(), SpanId::generate(), "svc");
    let child = SpanBuilder::new("child").build_child(&parent);

    assert_eq!(child.trace_id, parent.trace_id);
    assert_eq!(child.parent_span_id, Some(parent.span_id));
  }

  #[test]
  fn test_tracer_start_trace() {
    let tracer = Tracer::new("test-service");
    let span = tracer.start_trace("operation");

    assert_eq!(span.name, "operation");
    assert_eq!(span.state, SpanState::Started);
    assert!(span.is_root());
  }

  #[test]
  fn test_tracer_child_span() {
    let tracer = Tracer::new("test-service");
    let parent = tracer.start_trace("parent");
    let child = tracer.start_child("child", &parent);

    assert_eq!(child.trace_id, parent.trace_id);
    assert_eq!(child.parent_span_id, Some(parent.span_id));
  }

  #[test]
  fn test_tracer_get_spans() {
    let tracer = Tracer::new("test-service");
    let parent = tracer.start_trace("parent");
    let child = tracer.start_child("child", &parent);

    let spans = tracer.get_spans();
    assert_eq!(spans.len(), 2);
  }

  #[test]
  fn test_tracer_get_spans_by_trace() {
    let tracer = Tracer::new("test-service");
    let trace1_span = tracer.start_trace("trace1");
    let trace2_span = tracer.start_trace("trace2");

    let trace1_spans = tracer.get_spans_by_trace(trace1_span.trace_id);
    assert_eq!(trace1_spans.len(), 1);
    assert_eq!(trace1_spans[0].name, "trace1");
  }

  #[test]
  fn test_tracer_clear() {
    let tracer = Tracer::new("test-service");
    tracer.start_trace("test");
    assert!(!tracer.get_spans().is_empty());

    tracer.clear();
    assert!(tracer.get_spans().is_empty());
  }

  #[test]
  fn test_trace_summary() {
    let tracer = Tracer::new("test-service");
    let _parent = tracer.start_trace("parent");
    let _child = tracer.start_child(
      "child",
      &Span::new("dummy", TraceId::generate(), SpanId::generate(), "test"),
    );

    let spans = tracer.get_spans();
    // Spans are recorded when started, not when modified
    assert!(spans.len() >= 2);

    let summary = TraceSummary::from_spans(&spans);
    assert!(summary.is_some());
    let summary = summary.unwrap();
    assert!(summary.span_count >= 2);
  }

  #[test]
  fn test_span_kind_otlp() {
    assert_eq!(SpanKind::Client.as_otlp(), "SPAN_KIND_CLIENT");
    assert_eq!(SpanKind::Server.as_otlp(), "SPAN_KIND_SERVER");
    assert_eq!(SpanKind::Internal.as_otlp(), "SPAN_KIND_INTERNAL");
  }

  #[test]
  fn test_span_event() {
    let event = SpanEvent {
      name: "exception".to_string(),
      timestamp: Utc::now(),
      attributes: HashMap::new(),
    };

    let mut span = Span::new("test", TraceId::generate(), SpanId::generate(), "svc");
    span = span.with_event(event);

    assert_eq!(span.events.len(), 1);
    assert_eq!(span.events[0].name, "exception");
  }

  #[test]
  fn test_trace_context_baggage() {
    let ctx = TraceContext::default()
      .with_baggage("user_id", "123")
      .with_baggage("tenant", "acme");

    assert_eq!(ctx.baggage.get("user_id"), Some(&"123".to_string()));
    assert_eq!(ctx.baggage.get("tenant"), Some(&"acme".to_string()));
  }

  #[test]
  fn test_tracer_from_context() {
    let tracer = Tracer::new("test-service");
    let parent_ctx = TraceContext::default();
    let span = tracer.start_from_context("child", &parent_ctx);

    assert_eq!(span.trace_id, parent_ctx.trace_id);
    assert_eq!(span.parent_span_id, Some(parent_ctx.span_id));
  }
}
