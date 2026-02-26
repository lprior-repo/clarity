#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! OpenCode extraction provider implementation
//!
//! This module provides a client for the OpenCode extraction API,
//! which extracts structured fields from unstructured text.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::r#trait::{
    ExtractionContext, ExtractionError, ExtractionMetadata, ExtractionProvider,
    ExtractedFields, FieldExtraction, FieldType, SchemaField,
};

/// Default timeout for API requests in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Session ID header name
const SESSION_HEADER: &str = "X-Session-ID";

/// OpenCode extraction API client
///
/// This client communicates with the OpenCode extraction service
/// to extract structured fields from text input.
#[derive(Debug, Clone)]
pub struct OpenCodeProvider {
    /// API endpoint URL
    endpoint: String,
    /// Session identifier for request tracking
    session_id: String,
    /// HTTP client for making requests
    client: Client,
}

impl OpenCodeProvider {
    /// Create a new OpenCode provider
    ///
    /// # Arguments
    /// * `endpoint` - Base URL of the OpenCode API (e.g., "https://api.opencode.com")
    /// * `session_id` - Unique session identifier for tracking
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created provider
    /// * `Err(ExtractionError)` - Failed to create HTTP client
    pub fn new(
        endpoint: String,
        session_id: String,
    ) -> Result<Self, ExtractionError> {
        let timeout = std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS);

        Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| ExtractionError::ConfigurationError(format!(
                "Failed to create HTTP client: {e}"
            )))
            .map(|client| Self {
                endpoint,
                session_id,
                client,
            })
    }

    /// Get the session ID
    #[must_use]
    pub const fn session_id(&self) -> &String {
        &self.session_id
    }

    /// Build the full URL for an API endpoint
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.endpoint.trim_end_matches('/'), path)
    }

    /// Convert HTTP errors to ExtractionError
    fn map_http_error(&self, error: reqwest::Error) -> ExtractionError {
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
            return ExtractionError::NetworkError(format!(
                "Request failed: {error}"
            ));
        }

        ExtractionError::NetworkError(format!("HTTP error: {error}"))
    }

    /// Map HTTP status code to ExtractionError
    fn map_status_error(
        &self,
        status: reqwest::StatusCode,
        message: String,
    ) -> ExtractionError {
        match status.as_u16() {
            401 | 403 => ExtractionError::AuthenticationError(message),
            429 => ExtractionError::RateLimited {
                retry_after_seconds: 60,
            },
            400 | 422 => ExtractionError::InvalidInput(message),
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

    /// Parse API response into ExtractedFields
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
            model: response.model,
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

        let start = std::time::Instant::now();

        let request = ExtractRequest {
            text: text.to_string(),
            context: context.clone(),
        };

        let url = self.build_url("/extract");
        let response = self
            .client
            .post(&url)
            .header(SESSION_HEADER, &self.session_id)
            .json(&request)
            .send()
            .await
            .map_err(|e| self.map_http_error(e))?;

        let status = response.status();
        let duration_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error body: {e}"));

            return Err(self.map_status_error(status, error_body));
        }

        let extract_response: ExtractResponse = response
            .json()
            .await
            .map_err(|e| ExtractionError::ParseError(format!("Failed to parse response: {e}")))?;

        Ok(self.parse_response(extract_response, duration_ms))
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

        let start = std::time::Instant::now();

        let request = ExtractWithSchemaRequest {
            text: text.to_string(),
            schema: schema.to_vec(),
            context: context.clone(),
        };

        let url = self.build_url("/extract/schema");
        let response = self
            .client
            .post(&url)
            .header(SESSION_HEADER, &self.session_id)
            .json(&request)
            .send()
            .await
            .map_err(|e| self.map_http_error(e))?;

        let status = response.status();
        let duration_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error body: {e}"));

            return Err(self.map_status_error(status, error_body));
        }

        let extract_response: ExtractResponse = response
            .json()
            .await
            .map_err(|e| ExtractionError::ParseError(format!("Failed to parse response: {e}")))?;

        Ok(self.parse_response(extract_response, duration_ms))
    }

    fn provider_name(&self) -> &str {
        "opencode"
    }

    async fn health_check(&self) -> Result<(), ExtractionError> {
        let url = self.build_url("/health");

        let response = self
            .client
            .get(&url)
            .header(SESSION_HEADER, &self.session_id)
            .send()
            .await
            .map_err(|e| self.map_http_error(e))?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error body: {e}"));

            Err(self.map_status_error(status, error_body))
        }
    }
}

/// Request payload for /extract endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractRequest {
    text: String,
    context: ExtractionContext,
}

/// Request payload for /extract/schema endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractWithSchemaRequest {
    text: String,
    schema: Vec<SchemaField>,
    context: ExtractionContext,
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
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_provider() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
            "test-session".to_string(),
        );

        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.endpoint, "https://api.opencode.com");
        assert_eq!(provider.session_id, "test-session");
        assert_eq!(provider.provider_name(), "opencode");
    }

    #[test]
    fn test_build_url() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
            "test-session".to_string(),
        )
        .unwrap();

        assert_eq!(
            provider.build_url("/extract"),
            "https://api.opencode.com/extract"
        );

        assert_eq!(
            provider.build_url("/health"),
            "https://api.opencode.com/health"
        );

        // Test trailing slash handling
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com/".to_string(),
            "test-session".to_string(),
        )
        .unwrap();

        assert_eq!(
            provider.build_url("/extract"),
            "https://api.opencode.com/extract"
        );
    }

    #[test]
    fn test_map_status_error() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
            "test-session".to_string(),
        )
        .unwrap();

        // Test 401
        let error = provider.map_status_error(
            reqwest::StatusCode::UNAUTHORIZED,
            "Invalid token".to_string(),
        );
        assert!(matches!(
            error,
            ExtractionError::AuthenticationError(_)
        ));

        // Test 429
        let error = provider.map_status_error(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        );
        assert!(matches!(error, ExtractionError::RateLimited { .. }));

        // Test 400
        let error = provider.map_status_error(
            reqwest::StatusCode::BAD_REQUEST,
            "Invalid input".to_string(),
        );
        assert!(matches!(error, ExtractionError::InvalidInput(_)));

        // Test 500
        let error = provider.map_status_error(
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
    }

    #[test]
    fn test_parse_response() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
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
    fn test_extract_request_serialization() {
        let request = ExtractRequest {
            text: "Hello, world!".to_string(),
            context: ExtractionContext {
                document_type: Some("email".to_string()),
                locale: Some("en_US".to_string()),
                schema: None,
                extra: json!({}),
            },
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ExtractRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.text, "Hello, world!");
        assert_eq!(deserialized.context.document_type, Some("email".to_string()));
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
    fn test_session_id() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
            "my-session-123".to_string(),
        )
        .unwrap();

        assert_eq!(provider.session_id(), "my-session-123");
    }
}
