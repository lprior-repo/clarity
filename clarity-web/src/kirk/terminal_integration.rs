#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![allow(clippy::unnested_or_patterns)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::option_if_let_else)]
#![forbid(unsafe_code)]
#![allow(clippy::trivially_copy_pass_by_ref)]

//! Integration module connecting v2 terminal to real `OpenCode` server (bd-3us0).
//!
//! This module provides the connection layer between the Progressive Discover
//! terminal interface and the `OpenCode` backend server for AI-powered extraction.
//!
//! # Architecture
//!
//! The integration follows the functional core, imperative shell pattern:
//! - **Core**: Pure connection state machine and error types
//! - **Shell**: Async operations for actual network communication
//!
//! # Error Handling
//!
//! All operations return `Result<T, TerminalError>` and never panic.
//! Connection errors are handled gracefully with automatic retry logic.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::providers::{
  ExtractedFields, ExtractionContext, ExtractionError, ExtractionProvider, FieldType,
  OpenCodeProvider, SchemaField,
};
use crate::storage::transcript_store::InterrogationTranscript;

// ============================================================================
// Error Types (Core)
// ============================================================================

/// Errors that can occur during terminal-server communication.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalError {
  /// Connection to server failed
  #[error("Connection failed: {message}")]
  ConnectionFailed {
    /// Error message
    message: String,
    /// Whether this error is retryable
    retryable: bool,
  },

  /// Server is unreachable
  #[error("Server unreachable: {0}")]
  ServerUnreachable(String),

  /// Authentication failed
  #[error("Authentication failed: {0}")]
  AuthenticationFailed(String),

  /// Request timeout
  #[error("Request timed out after {timeout_ms}ms")]
  Timeout {
    /// Timeout duration in milliseconds
    timeout_ms: u64,
  },

  /// Rate limited by server
  #[error("Rate limited: retry after {retry_after_seconds}s")]
  RateLimited {
    /// Suggested retry time in seconds
    retry_after_seconds: u64,
  },

  /// Invalid request data
  #[error("Invalid request: {0}")]
  InvalidRequest(String),

  /// Server returned an error
  #[error("Server error: {message}")]
  ServerError {
    /// Error message
    message: String,
    /// HTTP status code
    status_code: Option<u16>,
  },

  /// Provider not initialized
  #[error("Provider not initialized")]
  ProviderNotInitialized,

  /// Session expired
  #[error("Session expired: {0}")]
  SessionExpired(String),

  /// Extraction failed
  #[error("Extraction failed: {0}")]
  ExtractionFailed(String),
}

impl From<ExtractionError> for TerminalError {
  fn from(error: ExtractionError) -> Self {
    match error {
      ExtractionError::NetworkError(msg) => Self::ConnectionFailed {
        message: msg,
        retryable: true,
      },
      ExtractionError::Timeout { timeout_ms } => Self::Timeout { timeout_ms },
      ExtractionError::RateLimited {
        retry_after_seconds,
      } => Self::RateLimited {
        retry_after_seconds,
      },
      ExtractionError::AuthenticationError(msg) => Self::AuthenticationFailed(msg),
      ExtractionError::InvalidInput(msg) | ExtractionError::ContentPolicy(msg) => {
        Self::InvalidRequest(msg)
      }
      ExtractionError::ApiError {
        message,
        status_code,
      } => Self::ServerError {
        message,
        status_code,
      },
      ExtractionError::ConfigurationError(msg) => Self::ConnectionFailed {
        message: msg,
        retryable: false,
      },
      ExtractionError::ParseError(msg)
      | ExtractionError::ProviderError(msg)
      | ExtractionError::Unknown(msg) => Self::ExtractionFailed(msg),
      ExtractionError::QuotaExceeded(msg) => Self::ServerError {
        message: msg,
        status_code: Some(402),
      },
    }
  }
}

// ============================================================================
// Connection State (Core)
// ============================================================================

/// State of the terminal connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionState {
  /// Not connected to server
  #[default]
  Disconnected,
  /// Attempting to connect
  Connecting,
  /// Connected and ready
  Connected,
  /// Connection lost, attempting reconnect
  Reconnecting,
  /// Connection failed permanently
  Failed,
}

impl ConnectionState {
  /// Check if the connection is active (can make requests).
  #[must_use]
  pub const fn is_active(self) -> bool {
    matches!(self, Self::Connected)
  }

  /// Check if currently connecting or reconnecting.
  #[must_use]
  pub const fn is_pending(self) -> bool {
    matches!(self, Self::Connecting | Self::Reconnecting)
  }

  /// Get the next state on successful connection.
  #[must_use]
  pub const fn on_success(self) -> Self {
    if self.is_active() {
      self
    } else {
      Self::Connected
    }
  }

  /// Get the next state on connection failure.
  #[must_use]
  pub const fn on_failure(self, retryable: bool) -> Self {
    match (self, retryable) {
      (Self::Connecting | Self::Reconnecting | Self::Connected, true) => Self::Reconnecting,
      _ => Self::Failed,
    }
  }
}

// ============================================================================
// Connection Configuration (Core)
// ============================================================================

/// Configuration for terminal connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalConfig {
  /// Server endpoint URL
  pub endpoint: String,
  /// Session identifier
  pub session_id: String,
  /// Request timeout in milliseconds
  pub timeout_ms: u64,
  /// Maximum retry attempts
  pub max_retries: u32,
  /// Delay between retries in milliseconds
  pub retry_delay_ms: u64,
}

impl TerminalConfig {
  /// Create a new terminal configuration.
  #[must_use]
  pub fn new(endpoint: String, session_id: String) -> Self {
    Self {
      endpoint,
      session_id,
      timeout_ms: 30_000,
      max_retries: 3,
      retry_delay_ms: 1000,
    }
  }

  /// Set the request timeout.
  #[must_use]
  pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
    self.timeout_ms = timeout_ms;
    self
  }

  /// Set the maximum retry attempts.
  #[must_use]
  pub fn with_max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  /// Set the retry delay.
  #[must_use]
  pub fn with_retry_delay(mut self, retry_delay_ms: u64) -> Self {
    self.retry_delay_ms = retry_delay_ms;
    self
  }

  /// Get the timeout as a Duration.
  #[must_use]
  pub const fn timeout_duration(&self) -> Duration {
    Duration::from_millis(self.timeout_ms)
  }

  /// Get the retry delay as a Duration.
  #[must_use]
  pub const fn retry_delay_duration(&self) -> Duration {
    Duration::from_millis(self.retry_delay_ms)
  }
}

// ============================================================================
// Connection Status (Core)
// ============================================================================

/// Status information about the terminal connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionStatus {
  /// Current connection state
  pub state: ConnectionState,
  /// Number of successful requests
  pub successful_requests: u64,
  /// Number of failed requests
  pub failed_requests: u64,
  /// Last error message (if any)
  pub last_error: Option<String>,
  /// Server latency in milliseconds (if connected)
  pub latency_ms: Option<u64>,
}

impl ConnectionStatus {
  /// Create a new connection status.
  #[must_use]
  pub fn new(state: ConnectionState) -> Self {
    Self {
      state,
      successful_requests: 0,
      failed_requests: 0,
      last_error: None,
      latency_ms: None,
    }
  }

  /// Create a disconnected status.
  #[must_use]
  pub fn disconnected() -> Self {
    Self::new(ConnectionState::Disconnected)
  }

  /// Record a successful request.
  #[must_use]
  pub fn with_success(mut self, latency_ms: u64) -> Self {
    self.successful_requests = self.successful_requests.saturating_add(1);
    self.latency_ms = Some(latency_ms);
    self.last_error = None;
    self
  }

  /// Record a failed request.
  #[must_use]
  pub fn with_failure(mut self, error: String) -> Self {
    self.failed_requests = self.failed_requests.saturating_add(1);
    self.last_error = Some(error);
    self
  }

  /// Update the connection state.
  #[must_use]
  pub fn with_state(mut self, state: ConnectionState) -> Self {
    self.state = state;
    self
  }

  /// Calculate the success rate (0.0 to 1.0).
  #[must_use]
  pub fn success_rate(&self) -> f64 {
    let total = self
      .successful_requests
      .saturating_add(self.failed_requests);
    if total == 0 {
      return 1.0;
    }
    f64::from(u32::try_from(self.successful_requests).unwrap_or(u32::MAX))
      / f64::from(u32::try_from(total).unwrap_or(u32::MAX))
  }
}

impl Default for ConnectionStatus {
  fn default() -> Self {
    Self::disconnected()
  }
}

// ============================================================================
// Terminal Client Trait (Shell Interface)
// ============================================================================

/// Trait for terminal-to-server communication.
///
/// This trait abstracts the communication layer, allowing for testing
/// with mock implementations.
#[async_trait]
pub trait TerminalClient: Send + Sync {
  /// Extract problem statement from text.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if extraction fails.
  async fn extract_problem(&self, text: &str) -> Result<ExtractedFields, TerminalError>;

  /// Extract target persona from text.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if extraction fails.
  async fn extract_persona(&self, text: &str) -> Result<ExtractedFields, TerminalError>;

  /// Extract solution description from text.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if extraction fails.
  async fn extract_solution(&self, text: &str) -> Result<ExtractedFields, TerminalError>;

  /// Extract non-persona description from text.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if extraction fails.
  async fn extract_nonpersona(&self, text: &str) -> Result<ExtractedFields, TerminalError>;

  /// Validate straw man traps in persona text.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if validation fails.
  async fn validate_straw_man(&self, persona_text: &str) -> Result<ExtractedFields, TerminalError>;

  /// Validate VORP (Value, Obvious, Real, Possible).
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if validation fails.
  async fn validate_vorp(
    &self,
    value: &str,
    obvious: &str,
    real: &str,
    possible: &str,
  ) -> Result<ExtractedFields, TerminalError>;

  /// Check server health.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if health check fails.
  async fn health_check(&self) -> Result<(), TerminalError>;

  /// Get current connection status.
  fn status(&self) -> ConnectionStatus;
}

// ============================================================================
// OpenCode Terminal Client (Shell Implementation)
// ============================================================================

/// Real implementation of `TerminalClient` using `OpenCode` provider.
pub struct OpenCodeTerminalClient {
  /// The underlying `OpenCode` provider
  provider: Arc<OpenCodeProvider>,
  /// Connection status (shared for interior mutability)
  status: Arc<RwLock<ConnectionStatus>>,
  /// Configuration
  config: TerminalConfig,
}

impl OpenCodeTerminalClient {
  /// Create a new `OpenCode` terminal client.
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if the provider cannot be created.
  pub fn new(config: TerminalConfig) -> Result<Self, TerminalError> {
    let provider = OpenCodeProvider::new(config.endpoint.clone(), config.session_id.clone())
      .map_err(|e| TerminalError::ConnectionFailed {
        message: format!("Failed to create provider: {e}"),
        retryable: false,
      })?;

    Ok(Self {
      provider: Arc::new(provider),
      status: Arc::new(RwLock::new(ConnectionStatus::new(
        ConnectionState::Disconnected,
      ))),
      config,
    })
  }

  /// Create a client with an existing provider.
  #[must_use]
  pub fn with_provider(provider: Arc<OpenCodeProvider>, config: TerminalConfig) -> Self {
    Self {
      provider,
      status: Arc::new(RwLock::new(ConnectionStatus::new(
        ConnectionState::Disconnected,
      ))),
      config,
    }
  }

  /// Build extraction context for the given document type.
  fn build_context(&self, document_type: &str) -> ExtractionContext {
    ExtractionContext {
      document_type: Some(document_type.to_string()),
      locale: Some("en_US".to_string()),
      schema: None,
      extra: serde_json::json!({
          "session_id": self.config.session_id,
          "source": "terminal_v2"
      }),
    }
  }

  /// Execute an extraction with retry logic.
  async fn extract_with_retry(
    &self,
    text: &str,
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, TerminalError> {
    // Update status to connecting
    {
      let status = self.status.read().await.clone();
      if !status.state.is_active() && !status.state.is_pending() {
        self.status.write().await.state = ConnectionState::Connecting;
      }
    }

    let mut last_error: Option<TerminalError> = None;
    let mut attempts = 0u32;

    while attempts < self.config.max_retries {
      attempts = attempts.saturating_add(1);

      let start = std::time::Instant::now();

      match self.provider.extract_fields(text, context).await {
        Ok(fields) => {
          let latency = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

          // Update status on success
          let status_guard = self.status.read().await.clone();
          *self.status.write().await = status_guard
            .with_state(ConnectionState::Connected)
            .with_success(latency);

          debug!(
            attempts = attempts,
            latency_ms = latency,
            "Extraction succeeded"
          );

          return Ok(fields);
        }
        Err(e) => {
          let terminal_error = TerminalError::from(e.clone());
          let retryable = matches!(
            terminal_error,
            TerminalError::ConnectionFailed {
              retryable: true,
              ..
            } | TerminalError::Timeout { .. }
              | TerminalError::RateLimited { .. }
          );

          // Update status on failure
          let current_state = self.status.read().await.state;
          let new_state = current_state.on_failure(retryable);
          let error_msg = terminal_error.to_string();
          *self.status.write().await = ConnectionStatus::new(new_state).with_failure(error_msg);

          last_error = Some(terminal_error);

          if !retryable || attempts >= self.config.max_retries {
            break;
          }

          info!(
            attempt = attempts,
            max_retries = self.config.max_retries,
            delay_ms = self.config.retry_delay_ms,
            "Retrying extraction"
          );

          tokio::time::sleep(self.config.retry_delay_duration()).await;
        }
      }
    }

    // All retries exhausted
    last_error.map_or_else(
      || {
        Err(TerminalError::ExtractionFailed(
          "Unknown error after retries".to_string(),
        ))
      },
      |e| {
        warn!(
            attempts = attempts,
            error = %e,
            "All extraction attempts failed"
        );
        Err(e)
      },
    )
  }

  /// Execute extraction with schema.
  async fn extract_with_schema(
    &self,
    text: &str,
    schema: &[SchemaField],
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, TerminalError> {
    // Update status to connecting
    {
      let status = self.status.read().await.clone();
      if !status.state.is_active() && !status.state.is_pending() {
        self.status.write().await.state = ConnectionState::Connecting;
      }
    }

    let mut last_error: Option<TerminalError> = None;
    let mut attempts = 0u32;

    while attempts < self.config.max_retries {
      attempts = attempts.saturating_add(1);

      let start = std::time::Instant::now();

      match self
        .provider
        .extract_fields_with_schema(text, schema, context)
        .await
      {
        Ok(fields) => {
          let latency = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

          // Update status on success
          let status_guard = self.status.read().await.clone();
          *self.status.write().await = status_guard
            .with_state(ConnectionState::Connected)
            .with_success(latency);

          debug!(
            attempts = attempts,
            latency_ms = latency,
            "Schema extraction succeeded"
          );

          return Ok(fields);
        }
        Err(e) => {
          let terminal_error = TerminalError::from(e.clone());
          let retryable = matches!(
            terminal_error,
            TerminalError::ConnectionFailed {
              retryable: true,
              ..
            } | TerminalError::Timeout { .. }
              | TerminalError::RateLimited { .. }
          );

          // Update status on failure
          let current_state = self.status.read().await.state;
          let new_state = current_state.on_failure(retryable);
          let error_msg = terminal_error.to_string();
          *self.status.write().await = ConnectionStatus::new(new_state).with_failure(error_msg);

          last_error = Some(terminal_error);

          if !retryable || attempts >= self.config.max_retries {
            break;
          }

          info!(
            attempt = attempts,
            max_retries = self.config.max_retries,
            delay_ms = self.config.retry_delay_ms,
            "Retrying schema extraction"
          );

          tokio::time::sleep(self.config.retry_delay_duration()).await;
        }
      }
    }

    last_error.map_or_else(
      || {
        Err(TerminalError::ExtractionFailed(
          "Unknown error after retries".to_string(),
        ))
      },
      |e| {
        warn!(
            attempts = attempts,
            error = %e,
            "All schema extraction attempts failed"
        );
        Err(e)
      },
    )
  }
}

#[async_trait]
impl TerminalClient for OpenCodeTerminalClient {
  async fn extract_problem(&self, text: &str) -> Result<ExtractedFields, TerminalError> {
    let context = self.build_context("problem_statement");
    self.extract_with_retry(text, &context).await
  }

  async fn extract_persona(&self, text: &str) -> Result<ExtractedFields, TerminalError> {
    let context = self.build_context("target_persona");
    self.extract_with_retry(text, &context).await
  }

  async fn extract_solution(&self, text: &str) -> Result<ExtractedFields, TerminalError> {
    let context = self.build_context("solution_description");
    self.extract_with_retry(text, &context).await
  }

  async fn extract_nonpersona(&self, text: &str) -> Result<ExtractedFields, TerminalError> {
    let context = self.build_context("non_persona");
    self.extract_with_retry(text, &context).await
  }

  async fn validate_straw_man(&self, persona_text: &str) -> Result<ExtractedFields, TerminalError> {
    let schema = vec![
      SchemaField {
        name: "irrational_actor_detected".to_string(),
        field_type: FieldType::Boolean,
        required: true,
        description: Some(
          "True if the persona acts against their own motivations or self-interest".to_string(),
        ),
        options: None,
      },
      SchemaField {
        name: "manic_pixie_dream_user_detected".to_string(),
        field_type: FieldType::Boolean,
        required: true,
        description: Some(
          "True if the persona magically loves everything without discernment".to_string(),
        ),
        options: None,
      },
      SchemaField {
        name: "stoic_monk_detected".to_string(),
        field_type: FieldType::Boolean,
        required: true,
        description: Some(
          "True if the persona tolerates immense friction or difficulty without complaint"
            .to_string(),
        ),
        options: None,
      },
      SchemaField {
        name: "your_clone_detected".to_string(),
        field_type: FieldType::Boolean,
        required: true,
        description: Some(
          "True if the persona has developer-level system knowledge or mental models".to_string(),
        ),
        options: None,
      },
      SchemaField {
        name: "suggestions".to_string(),
        field_type: FieldType::TextArea,
        required: false,
        description: Some(
          "Specific suggestions for fixing detected traps. Be concrete and actionable.".to_string(),
        ),
        options: None,
      },
    ];

    let context = self.build_context("straw_man_validation");
    self
      .extract_with_schema(persona_text, &schema, &context)
      .await
  }

  async fn validate_vorp(
    &self,
    value: &str,
    obvious: &str,
    real: &str,
    possible: &str,
  ) -> Result<ExtractedFields, TerminalError> {
    let prompt = format!(
      "Analyze the VORP dimensions for this solution:\n\n\
             Value: {value}\n\
             Obvious: {obvious}\n\
             Real: {real}\n\
             Possible: {possible}\n\n\
             Score each dimension from 0.0 to 1.0.",
    );

    let schema = vec![
      SchemaField {
        name: "value_score".to_string(),
        field_type: FieldType::Number,
        required: true,
        description: Some("Score for Value dimension (0.0-1.0)".to_string()),
        options: None,
      },
      SchemaField {
        name: "obvious_score".to_string(),
        field_type: FieldType::Number,
        required: true,
        description: Some("Score for Obvious dimension (0.0-1.0)".to_string()),
        options: None,
      },
      SchemaField {
        name: "real_score".to_string(),
        field_type: FieldType::Number,
        required: true,
        description: Some("Score for Real dimension (0.0-1.0)".to_string()),
        options: None,
      },
      SchemaField {
        name: "possible_score".to_string(),
        field_type: FieldType::Number,
        required: true,
        description: Some("Score for Possible dimension (0.0-1.0)".to_string()),
        options: None,
      },
      SchemaField {
        name: "suggestions".to_string(),
        field_type: FieldType::TextArea,
        required: false,
        description: Some("Suggestions for improvement".to_string()),
        options: None,
      },
    ];

    let context = self.build_context("vorp_validation");
    self.extract_with_schema(&prompt, &schema, &context).await
  }

  async fn health_check(&self) -> Result<(), TerminalError> {
    // Update status to connecting
    {
      let status = self.status.read().await.clone();
      if !status.state.is_active() && !status.state.is_pending() {
        self.status.write().await.state = ConnectionState::Connecting;
      }
    }

    let result = self.provider.health_check().await;

    match result {
      Ok(()) => {
        let status_guard = self.status.read().await.clone();
        *self.status.write().await = status_guard.with_state(ConnectionState::Connected);

        debug!("Health check passed");
        Ok(())
      }
      Err(e) => {
        let terminal_error = TerminalError::from(e);
        let status_guard = self.status.read().await.clone();
        *self.status.write().await = status_guard
          .with_state(ConnectionState::Failed)
          .with_failure(terminal_error.to_string());

        warn!(error = %terminal_error, "Health check failed");
        Err(terminal_error)
      }
    }
  }

  fn status(&self) -> ConnectionStatus {
    // Note: This is a synchronous method, so we can't await the lock.
    // We return a snapshot from a try_read or default.
    self
      .status
      .try_read()
      .map_or_else(|_| ConnectionStatus::default(), |guard| guard.clone())
  }
}

// ============================================================================
// Transcript Processor (Core + Shell)
// ============================================================================

/// Processor for extracting fields from interrogation transcripts.
pub struct TranscriptProcessor {
  /// The terminal client
  client: Arc<dyn TerminalClient>,
}

impl TranscriptProcessor {
  /// Create a new transcript processor.
  #[must_use]
  pub fn new(client: Arc<dyn TerminalClient>) -> Self {
    Self { client }
  }

  /// Extract all fields from a transcript.
  ///
  /// This method processes the transcript in order, extracting:
  /// 1. Problem statement
  /// 2. Target persona
  /// 3. Solution description
  /// 4. Non-persona description
  ///
  /// # Errors
  ///
  /// Returns `TerminalError` if any extraction fails.
  pub async fn process_transcript(
    &self,
    transcript: &InterrogationTranscript,
  ) -> Result<ProcessedTranscript, TerminalError> {
    let original_prompt = transcript.original_prompt.clone();

    // Extract problem
    let problem_result = self.client.extract_problem(&original_prompt).await?;
    let problem =
      extract_field_value(&problem_result, "problem").unwrap_or_else(|| original_prompt.clone());

    // Extract persona
    let persona_result = self.client.extract_persona(&original_prompt).await?;
    let persona = extract_field_value(&persona_result, "persona").unwrap_or_default();

    // Extract solution
    let solution_result = self.client.extract_solution(&original_prompt).await?;
    let solution = extract_field_value(&solution_result, "solution").unwrap_or_default();

    // Extract non-persona
    let nonpersona_result = self.client.extract_nonpersona(&original_prompt).await?;
    let nonpersona = extract_field_value(&nonpersona_result, "nonpersona").unwrap_or_default();

    Ok(ProcessedTranscript {
      original_prompt,
      problem,
      persona,
      solution,
      nonpersona,
      problem_confidence: problem_result.confidence,
      persona_confidence: persona_result.confidence,
      solution_confidence: solution_result.confidence,
      nonpersona_confidence: nonpersona_result.confidence,
    })
  }
}

/// Result of processing a transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedTranscript {
  /// Original user prompt
  pub original_prompt: String,
  /// Extracted problem statement
  pub problem: String,
  /// Extracted target persona
  pub persona: String,
  /// Extracted solution description
  pub solution: String,
  /// Extracted non-persona description
  pub nonpersona: String,
  /// Confidence score for problem extraction
  pub problem_confidence: f64,
  /// Confidence score for persona extraction
  pub persona_confidence: f64,
  /// Confidence score for solution extraction
  pub solution_confidence: f64,
  /// Confidence score for nonpersona extraction
  pub nonpersona_confidence: f64,
}

/// Extract a field value from extracted fields by name.
fn extract_field_value(fields: &ExtractedFields, name: &str) -> Option<String> {
  fields
    .fields
    .iter()
    .find(|f| f.name == name)
    .and_then(|f| f.value.as_str().map(std::string::ToString::to_string))
}

// ============================================================================
// Tests
// ============================================================================

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

  #[test]
  fn test_connection_state_is_active() {
    assert!(ConnectionState::Connected.is_active());
    assert!(!ConnectionState::Disconnected.is_active());
    assert!(!ConnectionState::Connecting.is_active());
    assert!(!ConnectionState::Reconnecting.is_active());
    assert!(!ConnectionState::Failed.is_active());
  }

  #[test]
  fn test_connection_state_is_pending() {
    assert!(ConnectionState::Connecting.is_pending());
    assert!(ConnectionState::Reconnecting.is_pending());
    assert!(!ConnectionState::Connected.is_pending());
    assert!(!ConnectionState::Disconnected.is_pending());
    assert!(!ConnectionState::Failed.is_pending());
  }

  #[test]
  fn test_connection_state_on_success() {
    assert_eq!(
      ConnectionState::Connecting.on_success(),
      ConnectionState::Connected
    );
    assert_eq!(
      ConnectionState::Reconnecting.on_success(),
      ConnectionState::Connected
    );
    assert_eq!(
      ConnectionState::Connected.on_success(),
      ConnectionState::Connected
    );
  }

  #[test]
  fn test_connection_state_on_failure_retryable() {
    assert_eq!(
      ConnectionState::Connecting.on_failure(true),
      ConnectionState::Reconnecting
    );
    assert_eq!(
      ConnectionState::Reconnecting.on_failure(true),
      ConnectionState::Reconnecting
    );
    assert_eq!(
      ConnectionState::Connected.on_failure(true),
      ConnectionState::Reconnecting
    );
  }

  #[test]
  fn test_connection_state_on_failure_non_retryable() {
    assert_eq!(
      ConnectionState::Connecting.on_failure(false),
      ConnectionState::Failed
    );
    assert_eq!(
      ConnectionState::Connected.on_failure(false),
      ConnectionState::Failed
    );
  }

  #[test]
  fn test_terminal_config_new() {
    let config = TerminalConfig::new(
      "https://api.example.com".to_string(),
      "session-123".to_string(),
    );

    assert_eq!(config.endpoint, "https://api.example.com");
    assert_eq!(config.session_id, "session-123");
    assert_eq!(config.timeout_ms, 30_000);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay_ms, 1000);
  }

  #[test]
  fn test_terminal_config_builders() {
    let config = TerminalConfig::new("endpoint".to_string(), "session".to_string())
      .with_timeout(60_000)
      .with_max_retries(5)
      .with_retry_delay(2000);

    assert_eq!(config.timeout_ms, 60_000);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay_ms, 2000);
  }

  #[test]
  fn test_terminal_config_durations() {
    let config = TerminalConfig::new("endpoint".to_string(), "session".to_string())
      .with_timeout(5000)
      .with_retry_delay(500);

    assert_eq!(config.timeout_duration(), Duration::from_secs(5));
    assert_eq!(config.retry_delay_duration(), Duration::from_millis(500));
  }

  #[test]
  fn test_connection_status_new() {
    let status = ConnectionStatus::new(ConnectionState::Connecting);

    assert_eq!(status.state, ConnectionState::Connecting);
    assert_eq!(status.successful_requests, 0);
    assert_eq!(status.failed_requests, 0);
    assert!(status.last_error.is_none());
    assert!(status.latency_ms.is_none());
  }

  #[test]
  fn test_connection_status_disconnected() {
    let status = ConnectionStatus::disconnected();

    assert_eq!(status.state, ConnectionState::Disconnected);
  }

  #[test]
  fn test_connection_status_with_success() {
    let status = ConnectionStatus::disconnected()
      .with_state(ConnectionState::Connected)
      .with_success(150);

    assert_eq!(status.state, ConnectionState::Connected);
    assert_eq!(status.successful_requests, 1);
    assert_eq!(status.latency_ms, Some(150));
    assert!(status.last_error.is_none());
  }

  #[test]
  fn test_connection_status_with_failure() {
    let status = ConnectionStatus::disconnected().with_failure("Connection refused".to_string());

    assert_eq!(status.failed_requests, 1);
    assert_eq!(status.last_error, Some("Connection refused".to_string()));
  }

  #[test]
  fn test_connection_status_success_rate() {
    // No requests
    let status = ConnectionStatus::disconnected();
    assert!((status.success_rate() - 1.0).abs() < f64::EPSILON);

    // All success
    let status = ConnectionStatus::disconnected()
      .with_success(100)
      .with_success(100);
    assert!((status.success_rate() - 1.0).abs() < f64::EPSILON);

    // Mixed
    let status = ConnectionStatus::disconnected()
      .with_success(100)
      .with_failure("error".to_string());
    assert!((status.success_rate() - 0.5).abs() < f64::EPSILON);

    // All failure
    let status = ConnectionStatus::disconnected()
      .with_failure("error".to_string())
      .with_failure("error".to_string());
    assert!((status.success_rate() - 0.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_terminal_error_from_extraction_error() {
    let extraction_error = ExtractionError::NetworkError("Connection refused".to_string());
    let terminal_error = TerminalError::from(extraction_error);
    assert!(matches!(
      terminal_error,
      TerminalError::ConnectionFailed {
        retryable: true,
        ..
      }
    ));

    let extraction_error = ExtractionError::Timeout { timeout_ms: 5000 };
    let terminal_error = TerminalError::from(extraction_error);
    assert!(matches!(terminal_error, TerminalError::Timeout { .. }));

    let extraction_error = ExtractionError::RateLimited {
      retry_after_seconds: 60,
    };
    let terminal_error = TerminalError::from(extraction_error);
    assert!(matches!(terminal_error, TerminalError::RateLimited { .. }));

    let extraction_error = ExtractionError::AuthenticationError("Invalid token".to_string());
    let terminal_error = TerminalError::from(extraction_error);
    assert!(matches!(
      terminal_error,
      TerminalError::AuthenticationFailed(_)
    ));
  }

  #[test]
  fn test_processed_transcript_serialization() -> Result<(), serde_json::Error> {
    let processed = ProcessedTranscript {
      original_prompt: "Build a todo app".to_string(),
      problem: "Users can't track tasks".to_string(),
      persona: "Busy professionals".to_string(),
      solution: "Simple task tracker".to_string(),
      nonpersona: "Enterprise teams".to_string(),
      problem_confidence: 0.95,
      persona_confidence: 0.85,
      solution_confidence: 0.90,
      nonpersona_confidence: 0.80,
    };

    let json = serde_json::to_string(&processed)?;

    let restored: ProcessedTranscript = serde_json::from_str(&json)?;

    assert_eq!(restored.original_prompt, "Build a todo app");
    assert_eq!(restored.problem, "Users can't track tasks");
    assert!((restored.problem_confidence - 0.95).abs() < f64::EPSILON);
    Ok(())
  }

  #[test]
  fn test_extract_field_value() {
    use crate::providers::FieldExtraction;
    use serde_json::json;

    let fields = ExtractedFields {
      fields: vec![
        FieldExtraction {
          name: "problem".to_string(),
          field_type: FieldType::TextArea,
          value: json!("Users struggle with tasks"),
          confidence: 0.9,
          justification: None,
        },
        FieldExtraction {
          name: "persona".to_string(),
          field_type: FieldType::TextArea,
          value: json!("Busy professionals"),
          confidence: 0.85,
          justification: None,
        },
      ],
      confidence: 0.875,
      metadata: crate::providers::ExtractionMetadata {
        provider: "opencode".to_string(),
        model: Some("test".to_string()),
        timestamp: chrono::Utc::now(),
        processing_duration_ms: 100,
        extra: json!({}),
      },
    };

    let problem = extract_field_value(&fields, "problem");
    assert_eq!(problem, Some("Users struggle with tasks".to_string()));

    let persona = extract_field_value(&fields, "persona");
    assert_eq!(persona, Some("Busy professionals".to_string()));

    let missing = extract_field_value(&fields, "missing");
    assert!(missing.is_none());
  }
}

// ============================================================================
// Mock Client for Testing
// ============================================================================

#[cfg(test)]
mod mock_client {

  use super::*;

  /// Mock terminal client for testing.
  pub struct MockTerminalClient {
    status: Arc<RwLock<ConnectionStatus>>,
    should_fail: bool,
  }

  impl MockTerminalClient {
    /// Create a new mock client.
    #[must_use]
    pub fn new() -> Self {
      Self {
        status: Arc::new(RwLock::new(ConnectionStatus::disconnected())),
        should_fail: false,
      }
    }

    /// Create a mock client that fails all requests.
    #[must_use]
    pub fn failing() -> Self {
      Self {
        status: Arc::new(RwLock::new(ConnectionStatus::disconnected())),
        should_fail: true,
      }
    }
  }

  #[async_trait]
  impl TerminalClient for MockTerminalClient {
    async fn extract_problem(&self, _text: &str) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      let status_guard = self.status.read().await.clone();
      *self.status.write().await = status_guard
        .with_state(ConnectionState::Connected)
        .with_success(100);

      Ok(ExtractedFields {
        fields: vec![crate::providers::FieldExtraction {
          name: "problem".to_string(),
          field_type: FieldType::TextArea,
          value: serde_json::json!("Mock problem"),
          confidence: 0.9,
          justification: None,
        }],
        confidence: 0.9,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn extract_persona(&self, _text: &str) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      Ok(ExtractedFields {
        fields: vec![crate::providers::FieldExtraction {
          name: "persona".to_string(),
          field_type: FieldType::TextArea,
          value: serde_json::json!("Mock persona"),
          confidence: 0.85,
          justification: None,
        }],
        confidence: 0.85,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn extract_solution(&self, _text: &str) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      Ok(ExtractedFields {
        fields: vec![crate::providers::FieldExtraction {
          name: "solution".to_string(),
          field_type: FieldType::TextArea,
          value: serde_json::json!("Mock solution"),
          confidence: 0.88,
          justification: None,
        }],
        confidence: 0.88,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn extract_nonpersona(&self, _text: &str) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      Ok(ExtractedFields {
        fields: vec![crate::providers::FieldExtraction {
          name: "nonpersona".to_string(),
          field_type: FieldType::TextArea,
          value: serde_json::json!("Mock nonpersona"),
          confidence: 0.80,
          justification: None,
        }],
        confidence: 0.80,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn validate_straw_man(
      &self,
      _persona_text: &str,
    ) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      Ok(ExtractedFields {
        fields: vec![
          crate::providers::FieldExtraction {
            name: "irrational_actor_detected".to_string(),
            field_type: FieldType::Boolean,
            value: serde_json::json!(false),
            confidence: 0.95,
            justification: None,
          },
          crate::providers::FieldExtraction {
            name: "manic_pixie_dream_user_detected".to_string(),
            field_type: FieldType::Boolean,
            value: serde_json::json!(false),
            confidence: 0.95,
            justification: None,
          },
        ],
        confidence: 0.95,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn validate_vorp(
      &self,
      _value: &str,
      _obvious: &str,
      _real: &str,
      _possible: &str,
    ) -> Result<ExtractedFields, TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ConnectionFailed {
          message: "Mock failure".to_string(),
          retryable: false,
        });
      }

      Ok(ExtractedFields {
        fields: vec![
          crate::providers::FieldExtraction {
            name: "value_score".to_string(),
            field_type: FieldType::Number,
            value: serde_json::json!(0.85),
            confidence: 0.9,
            justification: None,
          },
          crate::providers::FieldExtraction {
            name: "obvious_score".to_string(),
            field_type: FieldType::Number,
            value: serde_json::json!(0.80),
            confidence: 0.9,
            justification: None,
          },
        ],
        confidence: 0.825,
        metadata: crate::providers::ExtractionMetadata {
          provider: "mock".to_string(),
          model: Some("mock-v1".to_string()),
          timestamp: chrono::Utc::now(),
          processing_duration_ms: 100,
          extra: serde_json::json!({}),
        },
      })
    }

    async fn health_check(&self) -> Result<(), TerminalError> {
      if self.should_fail {
        return Err(TerminalError::ServerUnreachable("Mock failure".to_string()));
      }
      Ok(())
    }

    fn status(&self) -> ConnectionStatus {
      match self.status.try_read() {
        Ok(guard) => guard.clone(),
        Err(_) => ConnectionStatus::default(),
      }
    }
  }

  #[tokio::test]
  async fn test_mock_client_success() -> Result<(), TerminalError> {
    let client = MockTerminalClient::new();
    let fields = client.extract_problem("Test input").await?;

    assert_eq!(fields.fields.len(), 1);
    assert_eq!(fields.fields[0].name, "problem");
    Ok(())
  }

  #[tokio::test]
  async fn test_mock_client_failure() {
    let client = MockTerminalClient::failing();
    let result = client.extract_problem("Test input").await;

    assert!(result.is_err());
    if let Err(error) = result {
      assert!(matches!(
        error,
        TerminalError::ConnectionFailed {
          retryable: false,
          ..
        }
      ));
    }
  }

  #[tokio::test]
  async fn test_transcript_processor() -> Result<(), TerminalError> {
    let client = Arc::new(MockTerminalClient::new());
    let processor = TranscriptProcessor::new(client);

    let transcript =
      InterrogationTranscript::from_prompt("Build a todo app for busy professionals".to_string());

    let processed = processor.process_transcript(&transcript).await?;

    assert_eq!(
      processed.original_prompt,
      "Build a todo app for busy professionals"
    );
    assert_eq!(processed.problem, "Mock problem");
    assert_eq!(processed.persona, "Mock persona");
    assert_eq!(processed.solution, "Mock solution");
    assert_eq!(processed.nonpersona, "Mock nonpersona");
    Ok(())
  }
}
