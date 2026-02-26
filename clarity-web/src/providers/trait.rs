#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Field type specification for extraction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Single-line text input
    Text,
    /// Multi-line text area
    TextArea,
    /// Numeric value
    Number,
    /// Date selection
    Date,
    /// Time selection
    Time,
    /// Boolean toggle
    Boolean,
    /// Selection from predefined options
    Select,
    /// Multiple selections from predefined options
    MultiSelect,
    /// Email address
    Email,
    /// Phone number
    Phone,
    /// URL/Link
    Url,
    /// File attachment
    File,
    /// Rich formatted text
    RichText,
    /// Currency amount
    Currency,
    /// Percentage value
    Percentage,
}

/// Individual field extraction result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldExtraction {
    /// Field identifier/name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Extracted value (JSON-serializable)
    pub value: serde_json::Value,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Optional justification or source location
    pub justification: Option<String>,
}

/// Complete extraction result with all fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedFields {
    /// All extracted fields
    pub fields: Vec<FieldExtraction>,
    /// Overall confidence score
    pub confidence: f64,
    /// Processing metadata
    pub metadata: ExtractionMetadata,
}

/// Metadata about the extraction process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    /// Provider that performed the extraction
    pub provider: String,
    /// Model/version used
    pub model: Option<String>,
    /// Timestamp of extraction
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Processing time in milliseconds
    pub processing_duration_ms: u64,
    /// Additional metadata
    pub extra: serde_json::Value,
}

/// Context information for extraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionContext {
    /// Optional context about the document type
    pub document_type: Option<String>,
    /// Locale for formatting
    pub locale: Option<String>,
    /// Custom schema to extract
    pub schema: Option<Vec<SchemaField>>,
    /// Additional provider-specific context
    pub extra: serde_json::Value,
}

/// Schema field definition for structured extraction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Whether field is required
    pub required: bool,
    /// Description for the LLM
    pub description: Option<String>,
    /// Options for select/multi-select fields
    pub options: Option<Vec<String>>,
}

/// Errors that can occur during field extraction
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionError {
    /// API request failed
    #[error("API request failed: {message}")]
    ApiError {
        /// Error message
        message: String,
        /// HTTP status code if applicable
        status_code: Option<u16>,
    },

    /// Authentication/authorization failed
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Rate limiting occurred
    #[error("Rate limited: retry after {retry_after_seconds}s")]
    RateLimited {
        /// Suggested retry time
        retry_after_seconds: u64,
    },

    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Response parsing failed
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Timeout occurred
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Insufficient credits/quota
    #[error("Insufficient quota: {0}")]
    QuotaExceeded(String),

    /// Network connectivity issue
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Content policy violation
    #[error("Content policy violation: {0}")]
    ContentPolicy(String),

    /// Unknown/unexpected error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Trait for extraction providers (OpenAI, Claude, local models, etc.)
///
/// This trait defines the interface for extracting structured fields from
/// unstructured text using various AI providers.
#[async_trait]
pub trait ExtractionProvider: Send + Sync {
    /// Extract structured fields from text input
    ///
    /// # Arguments
    /// * `text` - The input text to extract fields from
    /// * `context` - Optional context about the extraction
    ///
    /// # Returns
    /// * `Ok(ExtractedFields)` - Successfully extracted fields
    /// * `Err(ExtractionError)` - Extraction failed
    async fn extract_fields(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<ExtractedFields, ExtractionError>;

    /// Extract structured fields with a custom schema
    ///
    /// # Arguments
    /// * `text` - The input text to extract fields from
    /// * `schema` - Schema defining fields to extract
    /// * `context` - Optional context about the extraction
    ///
    /// # Returns
    /// * `Ok(ExtractedFields)` - Successfully extracted fields
    /// * `Err(ExtractionError)` - Extraction failed
    async fn extract_fields_with_schema(
        &self,
        text: &str,
        schema: &[SchemaField],
        context: &ExtractionContext,
    ) -> Result<ExtractedFields, ExtractionError>;

    /// Get the provider name/identifier
    fn provider_name(&self) -> &str;

    /// Check if provider is configured and ready
    ///
    /// # Returns
    /// * `Ok(())` - Provider is ready
    /// * `Err(ExtractionError)` - Provider not configured
    async fn health_check(&self) -> Result<(), ExtractionError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_field_type_serialization() {
        // Test serialization
        let field_type = FieldType::Text;
        let serialized = serde_json::to_string(&field_type).unwrap();
        assert_eq!(serialized, r#""text""#);

        // Test deserialization
        let deserialized: FieldType = serde_json::from_str(r#""text""#).unwrap();
        assert_eq!(deserialized, FieldType::Text);

        // Test snake_case conversion
        let multi_select = FieldType::MultiSelect;
        let serialized = serde_json::to_string(&multi_select).unwrap();
        assert_eq!(serialized, r#""multi_select""#);
    }

    #[test]
    fn test_field_extraction_serialization() {
        let field = FieldExtraction {
            name: "email".to_string(),
            field_type: FieldType::Email,
            value: json!("user@example.com"),
            confidence: 0.95,
            justification: Some("Found in contact section".to_string()),
        };

        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: FieldExtraction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "email");
        assert_eq!(deserialized.field_type, FieldType::Email);
        assert_eq!(deserialized.value, json!("user@example.com"));
        assert!((deserialized.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(
            deserialized.justification,
            Some("Found in contact section".to_string())
        );
    }

    #[test]
    fn test_extracted_fields_serialization() {
        let fields = ExtractedFields {
            fields: vec![
                FieldExtraction {
                    name: "name".to_string(),
                    field_type: FieldType::Text,
                    value: json!("John Doe"),
                    confidence: 0.98,
                    justification: None,
                },
                FieldExtraction {
                    name: "age".to_string(),
                    field_type: FieldType::Number,
                    value: json!(42),
                    confidence: 0.90,
                    justification: Some("Explicitly stated".to_string()),
                },
            ],
            confidence: 0.94,
            metadata: ExtractionMetadata {
                provider: "test_provider".to_string(),
                model: Some("test-model-v1".to_string()),
                timestamp: chrono::Utc::now(),
                processing_duration_ms: 150,
                extra: json!({"test": true}),
            },
        };

        let serialized = serde_json::to_string(&fields).unwrap();
        let deserialized: ExtractedFields = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.fields.len(), 2);
        assert_eq!(deserialized.fields[0].name, "name");
        assert_eq!(deserialized.fields[1].name, "age");
        assert!((deserialized.confidence - 0.94).abs() < f64::EPSILON);
        assert_eq!(deserialized.metadata.provider, "test_provider");
    }

    #[test]
    fn test_extraction_context_serialization() {
        let context = ExtractionContext {
            document_type: Some("invoice".to_string()),
            locale: Some("en_US".to_string()),
            schema: Some(vec![
                SchemaField {
                    name: "total".to_string(),
                    field_type: FieldType::Currency,
                    required: true,
                    description: Some("Total amount due".to_string()),
                    options: None,
                },
            ]),
            extra: json!({"custom_key": "custom_value"}),
        };

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: ExtractionContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.document_type, Some("invoice".to_string()));
        assert_eq!(deserialized.locale, Some("en_US".to_string()));
        assert!(deserialized.schema.is_some());
        assert_eq!(deserialized.schema.unwrap().len(), 1);
    }

    #[test]
    fn test_extraction_error_serialization() {
        // Test ApiError
        let error = ExtractionError::ApiError {
            message: "Request failed".to_string(),
            status_code: Some(500),
        };
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: ExtractionError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized,
            ExtractionError::ApiError {
                message: "Request failed".to_string(),
                status_code: Some(500),
            }
        );

        // Test RateLimited
        let error = ExtractionError::RateLimited {
            retry_after_seconds: 60,
        };
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: ExtractionError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized,
            ExtractionError::RateLimited {
                retry_after_seconds: 60
            }
        );

        // Test InvalidInput
        let error = ExtractionError::InvalidInput("Empty text".to_string());
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: ExtractionError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized,
            ExtractionError::InvalidInput("Empty text".to_string())
        );
    }

    #[test]
    fn test_schema_field_serialization() {
        let field = SchemaField {
            name: "status".to_string(),
            field_type: FieldType::Select,
            required: true,
            description: Some("Order status".to_string()),
            options: Some(vec!["pending".to_string(), "complete".to_string()]),
        };

        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: SchemaField = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "status");
        assert_eq!(deserialized.field_type, FieldType::Select);
        assert!(deserialized.required);
        assert_eq!(deserialized.description, Some("Order status".to_string()));
        assert_eq!(
            deserialized.options,
            Some(vec!["pending".to_string(), "complete".to_string()])
        );
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = ExtractionMetadata {
            provider: "openai".to_string(),
            model: Some("gpt-4".to_string()),
            timestamp: chrono::Utc::now(),
            processing_duration_ms: 1234,
            extra: json!({"tokens_used": 150}),
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: ExtractionMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.provider, "openai");
        assert_eq!(deserialized.model, Some("gpt-4".to_string()));
        assert_eq!(deserialized.processing_duration_ms, 1234);
    }
}
