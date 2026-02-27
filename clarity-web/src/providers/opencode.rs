#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! `OpenCode` extraction provider implementation
//!
//! This module provides a client for OpenCode session APIs,
//! which extracts structured fields from unstructured text.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::r#trait::{
  ExtractedFields, ExtractionContext, ExtractionError, ExtractionMetadata, ExtractionProvider,
  FieldExtraction, FieldType, SchemaField,
};

/// Default timeout for API requests in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Session ID header name
const SESSION_HEADER: &str = "X-Session-ID";

/// Default agent used when invoking OpenCode session API.
const DEFAULT_AGENT: &str = "build";

/// `OpenCode` session API client
///
/// This client communicates with OpenCode
/// to extract structured fields from text input.
#[derive(Debug, Clone)]
pub struct OpenCodeProvider {
  /// API endpoint URL
  endpoint: String,
  /// Session identifier for request tracking
  session_id: String,
  /// Optional model identifier passed through to OpenCode
  model: Option<String>,
  /// Optional routed provider identifier for model backends
  routing_provider: Option<String>,
  /// HTTP client for making requests
  client: Client,
}

#[derive(Debug, Clone, Default)]
pub struct OpenCodeProviderOptions {
  pub model: Option<String>,
  pub routing_provider: Option<String>,
}

impl OpenCodeProvider {
  /// Create a new `OpenCode` provider
  ///
  /// # Arguments
  /// * `endpoint` - Base URL of the `OpenCode` API (e.g., `<https://api.opencode.ai/v1>`)
  /// * `session_id` - Unique session identifier for tracking
  ///
  /// # Returns
  /// * `Ok(Self)` - Successfully created provider
  /// * `Err(ExtractionError)` - Failed to create HTTP client
  ///
  /// # Errors
  ///
  /// Returns `ExtractionError::ConfigurationError` when the HTTP client cannot be built.
  pub fn new(endpoint: String, session_id: String) -> Result<Self, ExtractionError> {
    Self::new_with_options(endpoint, session_id, OpenCodeProviderOptions::default())
  }

  /// Create a new `OpenCode` provider with routing options.
  ///
  /// # Errors
  ///
  /// Returns `ExtractionError::ConfigurationError` when the HTTP client cannot be built.
  pub fn new_with_options(
    endpoint: String,
    session_id: String,
    options: OpenCodeProviderOptions,
  ) -> Result<Self, ExtractionError> {
    let timeout = std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS);

    Client::builder()
      .timeout(timeout)
      .build()
      .map_err(|e| {
        ExtractionError::ConfigurationError(format!("Failed to create HTTP client: {e}"))
      })
      .map(|client| Self {
        endpoint,
        session_id,
        model: options.model,
        routing_provider: options.routing_provider,
        client,
      })
  }

  /// Get the session ID
  #[must_use]
  pub const fn session_id(&self) -> &String {
    &self.session_id
  }

  #[must_use]
  pub const fn endpoint(&self) -> &String {
    &self.endpoint
  }

  #[must_use]
  pub const fn model(&self) -> &Option<String> {
    &self.model
  }

  #[must_use]
  pub const fn routing_provider(&self) -> &Option<String> {
    &self.routing_provider
  }

  /// Build the full URL for an API endpoint
  fn build_url(&self, path: &str) -> String {
    format!("{}{}", self.endpoint.trim_end_matches('/'), path)
  }

  /// Convert HTTP errors to `ExtractionError`
  fn map_http_error(&self, error: &reqwest::Error) -> ExtractionError {
    if error.is_timeout() {
      return ExtractionError::Timeout {
        timeout_ms: DEFAULT_TIMEOUT_SECS * 1000,
      };
    }

    if error.is_connect() {
      return ExtractionError::NetworkError(format!(
        "Failed to connect to {}: {}",
        self.endpoint, error
      ));
    }

    if error.is_request() {
      return ExtractionError::NetworkError(format!("Request failed: {error}"));
    }

    ExtractionError::NetworkError(format!("HTTP error: {error}"))
  }

  /// Map HTTP status code to `ExtractionError`
  fn map_status_error(status: reqwest::StatusCode, message: String) -> ExtractionError {
    match status.as_u16() {
      401 | 403 => ExtractionError::AuthenticationError(message),
      429 => ExtractionError::RateLimited {
        retry_after_seconds: 60,
      },
      400 | 422 => ExtractionError::InvalidInput(message),
      404 => ExtractionError::ApiError {
        message,
        status_code: Some(404),
      },
      402 => ExtractionError::QuotaExceeded(message),
      500..=599 => ExtractionError::ApiError {
        message,
        status_code: Some(status.as_u16()),
      },
      _ => ExtractionError::Unknown(format!(
        "Unexpected status code {}: {}",
        status.as_u16(),
        message
      )),
    }
  }

  /// Parse API response into `ExtractedFields`
  fn parse_response(
    &self,
    response: ExtractResponse,
    processing_duration_ms: u64,
  ) -> ExtractedFields {
    let fields = response
      .fields
      .into_iter()
      .map(|field| FieldExtraction {
        name: field.name,
        field_type: field.field_type,
        value: field.value,
        confidence: field.confidence,
        justification: field.justification,
      })
      .collect();

    let metadata = ExtractionMetadata {
      provider: self.provider_name().to_string(),
      model: response.model.or_else(|| self.model.clone()),
      timestamp: chrono::Utc::now(),
      processing_duration_ms,
      extra: response.extra.unwrap_or_default(),
    };

    ExtractedFields {
      fields,
      confidence: response.confidence,
      metadata,
    }
  }

  async fn extract_fields_via_session_api(
    &self,
    text: &str,
    context: &ExtractionContext,
    schema: Option<&[SchemaField]>,
  ) -> Result<ExtractedFields, ExtractionError> {
    let start = std::time::Instant::now();
    let session_id = self.create_opencode_session().await?;
    let prompt = Self::build_session_prompt(text, context, schema);
    let request = self.build_session_prompt_request(prompt);
    let url = self.build_url(&format!("/session/{session_id}/message"));

    let response = self
      .client
      .post(&url)
      .json(&request)
      .send()
      .await
      .map_err(|e| self.map_http_error(&e))?;

    let status = response.status();
    if !status.is_success() {
      let error_body = response
        .text()
        .await
        .unwrap_or_else(|e| format!("Failed to read error body: {e}"));

      return Err(Self::map_status_error(status, error_body));
    }

    let response_body: Value = response
      .json()
      .await
      .map_err(|e| ExtractionError::ParseError(format!("Failed to parse response: {e}")))?;

    let assistant_text = Self::extract_assistant_text(&response_body).ok_or_else(|| {
      ExtractionError::ParseError("Session API response missing assistant text part".to_string())
    })?;

    let extraction = Self::parse_extraction_from_assistant_text(&assistant_text)?;
    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    Ok(self.parse_response(extraction, duration_ms))
  }

  async fn create_opencode_session(&self) -> Result<String, ExtractionError> {
    let url = self.build_url("/session");
    let response = self
      .client
      .post(&url)
      .json(&serde_json::json!({"title": "clarity-extraction"}))
      .send()
      .await
      .map_err(|e| self.map_http_error(&e))?;

    let status = response.status();
    if !status.is_success() {
      let error_body = response
        .text()
        .await
        .unwrap_or_else(|e| format!("Failed to read error body: {e}"));
      return Err(Self::map_status_error(status, error_body));
    }

    let body: Value = response
      .json()
      .await
      .map_err(|e| ExtractionError::ParseError(format!("Failed to parse response: {e}")))?;

    Self::extract_session_id(&body).ok_or_else(|| {
      ExtractionError::ParseError("Could not find session id in OpenCode response".to_string())
    })
  }

  fn build_session_prompt_request(&self, prompt: String) -> SessionPromptRequest {
    let resolved_model = self.resolve_model_for_session_api();
    let provider_id = resolved_model.as_ref().map(|m| m.provider_id.clone());
    let model_id = resolved_model.as_ref().map(|m| m.model_id.clone());

    SessionPromptRequest {
      model: resolved_model,
      provider_id,
      model_id,
      agent: Some(DEFAULT_AGENT.to_string()),
      parts: vec![SessionPromptPart {
        part_type: "text".to_string(),
        text: prompt,
      }],
    }
  }

  fn resolve_model_for_session_api(&self) -> Option<SessionModelRef> {
    self
      .model
      .as_deref()
      .and_then(|model| {
        model
          .split_once('/')
          .map(|(provider_id, model_id)| (provider_id.to_string(), model_id.to_string()))
          .or_else(|| {
            self
              .routing_provider
              .as_ref()
              .map(|provider_id| (provider_id.clone(), model.to_string()))
          })
      })
      .map(|(provider_id, model_id)| SessionModelRef {
        provider_id,
        model_id,
      })
  }

  fn build_session_prompt(
    text: &str,
    context: &ExtractionContext,
    schema: Option<&[SchemaField]>,
  ) -> String {
    let context_json = serde_json::to_string(context).unwrap_or_else(|_| "{}".to_string());
    let schema_json = schema
      .map(serde_json::to_string)
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_else(|| "null".to_string());

    format!(
      "Extract structured fields from the input. Return ONLY valid JSON with this exact shape:\n\
{{\"fields\":[{{\"name\":\"...\",\"field_type\":\"text\",\"value\":\"...\",\"confidence\":0.0,\"justification\":null}}],\
\"confidence\":0.0,\"model\":null,\"extra\":{{}}}}\n\
Rules:\n\
- confidence values must be between 0.0 and 1.0\n\
- field_type must be snake_case and match supported types\n\
- if schema is provided, return only fields from schema\n\
- do not include markdown or explanation outside JSON\n\
Context JSON: {context_json}\n\
Schema JSON: {schema_json}\n\
Input:\n{text}"
    )
  }

  fn extract_session_id(value: &Value) -> Option<String> {
    value
      .get("id")
      .and_then(Value::as_str)
      .map(ToOwned::to_owned)
      .or_else(|| {
        value
          .get("data")
          .and_then(|data| data.get("id"))
          .and_then(Value::as_str)
          .map(ToOwned::to_owned)
      })
      .or_else(|| {
        value
          .get("session")
          .and_then(|session| session.get("id"))
          .and_then(Value::as_str)
          .map(ToOwned::to_owned)
      })
  }

  fn extract_assistant_text(value: &Value) -> Option<String> {
    Self::find_text_part(value)
      .or_else(|| value.get("data").and_then(Self::find_text_part))
      .or_else(|| value.get("message").and_then(Self::find_text_part))
      .or_else(|| {
        value
          .get("messages")
          .and_then(Value::as_array)
          .and_then(|messages| messages.iter().rev().find_map(Self::find_text_part))
      })
  }

  fn find_text_part(value: &Value) -> Option<String> {
    match value {
      Value::Object(map) => {
        let current = map
          .get("type")
          .and_then(Value::as_str)
          .zip(map.get("text").and_then(Value::as_str))
          .filter(|(part_type, _)| *part_type == "text")
          .map(|(_, text)| text.to_string());

        current.or_else(|| map.values().find_map(Self::find_text_part))
      }
      Value::Array(items) => items.iter().find_map(Self::find_text_part),
      _ => None,
    }
  }

  fn parse_extraction_from_assistant_text(text: &str) -> Result<ExtractResponse, ExtractionError> {
    serde_json::from_str::<ExtractResponse>(text)
      .or_else(|_| {
        Self::extract_json_block(text)
          .and_then(|json| serde_json::from_str::<ExtractResponse>(&json))
      })
      .map_err(|e| {
        ExtractionError::ParseError(format!(
          "Failed to parse extraction JSON from assistant text: {e}"
        ))
      })
  }

  fn extract_json_block(text: &str) -> Result<String, serde_json::Error> {
    let fenced = text
      .split("```")
      .map(str::trim)
      .find(|part| part.starts_with('{') || part.starts_with("json\n{"));

    let candidate = fenced
      .map(|part| {
        if let Some(stripped) = part.strip_prefix("json\n") {
          stripped.trim().to_string()
        } else {
          part.to_string()
        }
      })
      .or_else(|| {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        text.get(start..=end).map(ToOwned::to_owned)
      })
      .unwrap_or_default();

    serde_json::from_str::<Value>(&candidate).map(|_| candidate)
  }
}

#[async_trait]
impl ExtractionProvider for OpenCodeProvider {
  async fn extract_fields(
    &self,
    text: &str,
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    if text.trim().is_empty() {
      return Err(ExtractionError::InvalidInput(
        "Input text cannot be empty".to_string(),
      ));
    }

    self
      .extract_fields_via_session_api(text, context, None)
      .await
  }

  async fn extract_fields_with_schema(
    &self,
    text: &str,
    schema: &[SchemaField],
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    if text.trim().is_empty() {
      return Err(ExtractionError::InvalidInput(
        "Input text cannot be empty".to_string(),
      ));
    }

    if schema.is_empty() {
      return Err(ExtractionError::InvalidInput(
        "Schema cannot be empty".to_string(),
      ));
    }

    self
      .extract_fields_via_session_api(text, context, Some(schema))
      .await
  }

  fn provider_name(&self) -> &'static str {
    "opencode"
  }

  async fn health_check(&self) -> Result<(), ExtractionError> {
    let primary = self.check_health_path("/health").await;
    if primary.is_ok() {
      return Ok(());
    }

    let fallback = self.check_health_path("/global/health").await;
    if fallback.is_ok() {
      return Ok(());
    }

    fallback.or(primary)
  }
}

impl OpenCodeProvider {
  async fn check_health_path(&self, path: &str) -> Result<(), ExtractionError> {
    let url = self.build_url(path);

    let response = self
      .client
      .get(&url)
      .header(SESSION_HEADER, &self.session_id)
      .send()
      .await
      .map_err(|e| self.map_http_error(&e))?;

    let status = response.status();

    if status.is_success() {
      return Ok(());
    }

    let error_body: String = response
      .text()
      .await
      .unwrap_or_else(|e| format!("Failed to read error body: {e}"));

    Err(Self::map_status_error(status, error_body))
  }
}

/// Session API model descriptor for OpenCode `/session/:id/message`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionModelRef {
  #[serde(rename = "providerID")]
  provider_id: String,
  #[serde(rename = "modelID")]
  model_id: String,
}

/// Session API prompt part.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionPromptPart {
  #[serde(rename = "type")]
  part_type: String,
  text: String,
}

/// Session API prompt request.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionPromptRequest {
  model: Option<SessionModelRef>,
  #[serde(rename = "providerID")]
  provider_id: Option<String>,
  #[serde(rename = "modelID")]
  model_id: Option<String>,
  agent: Option<String>,
  parts: Vec<SessionPromptPart>,
}

/// Response from extract endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractResponse {
  fields: Vec<ExtractResponseField>,
  confidence: f64,
  model: Option<String>,
  extra: Option<serde_json::Value>,
}

/// Individual field in extract response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractResponseField {
  name: String,
  field_type: FieldType,
  value: serde_json::Value,
  confidence: f64,
  justification: Option<String>,
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]

  use super::*;
  use serde_json::json;

  #[test]
  fn test_new_provider() {
    let provider = OpenCodeProvider::new(
      "https://api.opencode.ai/v1".to_string(),
      "test-session".to_string(),
    );

    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.endpoint, "https://api.opencode.ai/v1");
    assert_eq!(provider.session_id, "test-session");
    assert_eq!(provider.model, None);
    assert_eq!(provider.routing_provider, None);
    assert_eq!(provider.provider_name(), "opencode");
  }

  #[test]
  fn test_new_provider_with_options() {
    let provider = OpenCodeProvider::new_with_options(
      "https://api.opencode.ai/v1".to_string(),
      "test-session".to_string(),
      OpenCodeProviderOptions {
        model: Some("zai-coding-plan/glm-5".to_string()),
        routing_provider: Some("zai-coding-plan".to_string()),
      },
    );

    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.model(), &Some("zai-coding-plan/glm-5".to_string()));
    assert_eq!(
      provider.routing_provider(),
      &Some("zai-coding-plan".to_string())
    );
  }

  #[test]
  fn test_build_url() {
    let provider = OpenCodeProvider::new(
      "https://api.opencode.ai/v1".to_string(),
      "test-session".to_string(),
    )
    .unwrap();

    assert_eq!(
      provider.build_url("/session"),
      "https://api.opencode.ai/v1/session"
    );

    assert_eq!(
      provider.build_url("/health"),
      "https://api.opencode.ai/v1/health"
    );

    // Test trailing slash handling
    let provider = OpenCodeProvider::new(
      "https://api.opencode.ai/v1/".to_string(),
      "test-session".to_string(),
    )
    .unwrap();

    assert_eq!(
      provider.build_url("/session"),
      "https://api.opencode.ai/v1/session"
    );
  }

  #[test]
  fn test_map_status_error() {
    // Test 401
    let error = OpenCodeProvider::map_status_error(
      reqwest::StatusCode::UNAUTHORIZED,
      "Invalid token".to_string(),
    );
    assert!(matches!(error, ExtractionError::AuthenticationError(_)));

    // Test 429
    let error = OpenCodeProvider::map_status_error(
      reqwest::StatusCode::TOO_MANY_REQUESTS,
      "Rate limit exceeded".to_string(),
    );
    assert!(matches!(error, ExtractionError::RateLimited { .. }));

    // Test 400
    let error = OpenCodeProvider::map_status_error(
      reqwest::StatusCode::BAD_REQUEST,
      "Invalid input".to_string(),
    );
    assert!(matches!(error, ExtractionError::InvalidInput(_)));

    // Test 500
    let error = OpenCodeProvider::map_status_error(
      reqwest::StatusCode::INTERNAL_SERVER_ERROR,
      "Server error".to_string(),
    );
    assert!(matches!(
      error,
      ExtractionError::ApiError {
        message: _,
        status_code: Some(500)
      }
    ));

    // Test 404
    let error =
      OpenCodeProvider::map_status_error(reqwest::StatusCode::NOT_FOUND, "Not found".to_string());
    assert!(matches!(
      error,
      ExtractionError::ApiError {
        message: _,
        status_code: Some(404)
      }
    ));
  }

  #[test]
  fn test_parse_response() {
    let provider = OpenCodeProvider::new(
      "https://api.opencode.ai/v1".to_string(),
      "test-session".to_string(),
    )
    .unwrap();

    let response = ExtractResponse {
      fields: vec![
        ExtractResponseField {
          name: "email".to_string(),
          field_type: FieldType::Email,
          value: json!("user@example.com"),
          confidence: 0.95,
          justification: Some("Found in contact section".to_string()),
        },
        ExtractResponseField {
          name: "name".to_string(),
          field_type: FieldType::Text,
          value: json!("John Doe"),
          confidence: 0.98,
          justification: None,
        },
      ],
      confidence: 0.965,
      model: Some("opencode-v1".to_string()),
      extra: Some(json!({"tokens_used": 100})),
    };

    let result = provider.parse_response(response, 150);

    assert_eq!(result.fields.len(), 2);
    assert_eq!(result.fields[0].name, "email");
    assert_eq!(result.fields[0].field_type, FieldType::Email);
    assert_eq!(result.fields[0].value, json!("user@example.com"));
    assert!((result.fields[0].confidence - 0.95).abs() < f64::EPSILON);
    assert_eq!(
      result.fields[0].justification,
      Some("Found in contact section".to_string())
    );

    assert_eq!(result.fields[1].name, "name");
    assert_eq!(result.fields[1].field_type, FieldType::Text);

    assert!((result.confidence - 0.965).abs() < f64::EPSILON);
    assert_eq!(result.metadata.provider, "opencode");
    assert_eq!(result.metadata.model, Some("opencode-v1".to_string()));
    assert_eq!(result.metadata.processing_duration_ms, 150);
    assert_eq!(result.metadata.extra, json!({"tokens_used": 100}));
  }

  #[test]
  fn test_parse_response_falls_back_to_configured_model() {
    let provider = OpenCodeProvider::new_with_options(
      "https://api.opencode.ai/v1".to_string(),
      "test-session".to_string(),
      OpenCodeProviderOptions {
        model: Some("zai-coding-plan/glm-5".to_string()),
        routing_provider: Some("zai-coding-plan".to_string()),
      },
    )
    .unwrap();

    let response = ExtractResponse {
      fields: vec![ExtractResponseField {
        name: "problem".to_string(),
        field_type: FieldType::TextArea,
        value: json!("Need faster extraction"),
        confidence: 0.82,
        justification: None,
      }],
      confidence: 0.82,
      model: None,
      extra: None,
    };

    let result = provider.parse_response(response, 42);
    assert_eq!(
      result.metadata.model.as_deref(),
      Some("zai-coding-plan/glm-5")
    );
    assert_eq!(result.metadata.provider, "opencode");
  }

  #[test]
  fn test_session_prompt_request_serialization() {
    let request = SessionPromptRequest {
      model: Some(SessionModelRef {
        provider_id: "zai-coding-plan".to_string(),
        model_id: "glm-5".to_string(),
      }),
      provider_id: Some("zai-coding-plan".to_string()),
      model_id: Some("glm-5".to_string()),
      agent: Some("build".to_string()),
      parts: vec![SessionPromptPart {
        part_type: "text".to_string(),
        text: "Hello, world!".to_string(),
      }],
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: SessionPromptRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.parts.len(), 1);
    assert_eq!(deserialized.parts[0].text, "Hello, world!");
    assert_eq!(deserialized.agent.as_deref(), Some("build"));
    assert_eq!(deserialized.provider_id.as_deref(), Some("zai-coding-plan"));
    assert_eq!(deserialized.model_id.as_deref(), Some("glm-5"));
  }

  #[test]
  fn test_extract_response_serialization() {
    let response = ExtractResponse {
      fields: vec![ExtractResponseField {
        name: "test".to_string(),
        field_type: FieldType::Text,
        value: json!("value"),
        confidence: 1.0,
        justification: None,
      }],
      confidence: 1.0,
      model: Some("model-v1".to_string()),
      extra: None,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: ExtractResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.fields.len(), 1);
    assert_eq!(deserialized.fields[0].name, "test");
    assert_eq!(deserialized.model, Some("model-v1".to_string()));
  }

  #[test]
  fn test_resolve_model_for_session_api_from_combined_model_id() {
    let provider = OpenCodeProvider::new_with_options(
      "https://api.opencode.ai/v1".to_string(),
      "my-session".to_string(),
      OpenCodeProviderOptions {
        model: Some("zai-coding-plan/glm-5".to_string()),
        routing_provider: None,
      },
    )
    .unwrap();

    let resolved = provider.resolve_model_for_session_api();
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(resolved.provider_id, "zai-coding-plan");
    assert_eq!(resolved.model_id, "glm-5");
  }

  #[test]
  fn test_extract_session_id_from_wrapped_response() {
    let response = json!({"data": {"id": "ses_123"}});
    let session_id = OpenCodeProvider::extract_session_id(&response);
    assert_eq!(session_id.as_deref(), Some("ses_123"));
  }

  #[test]
  fn test_parse_extraction_from_fenced_json_text() {
    let assistant_text =
      "```json\n{\"fields\":[],\"confidence\":0.9,\"model\":null,\"extra\":{}}\n```";
    let parsed = OpenCodeProvider::parse_extraction_from_assistant_text(assistant_text);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert!(parsed.fields.is_empty());
    assert!((parsed.confidence - 0.9).abs() < f64::EPSILON);
  }

  #[test]
  fn test_session_id() {
    let provider = OpenCodeProvider::new(
      "https://api.opencode.ai/v1".to_string(),
      "my-session-123".to_string(),
    )
    .unwrap();

    assert_eq!(provider.session_id(), "my-session-123");
  }
}
