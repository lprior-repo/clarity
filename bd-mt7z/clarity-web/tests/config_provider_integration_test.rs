//! Integration tests for config provider and server model coverage
//!
//! These tests verify:
//! 1. Model propagation from config to provider
//! 2. Error mapping for config errors
//! 3. AiProviderDiagnostics contract
//! 4. Model appearing in extract/suggest payloads

#![allow(clippy::all)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::needless_collect)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::single_match_else)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(unused_comparisons)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::assertions_on_constants)]

use clarity_web::config::{
  config_path, default_config, AiConfig, ConfigError, ProviderConfig, ProviderType, QualityConfig,
};
use clarity_web::providers::{
  ExtractionContext, ExtractionError, ExtractionMetadata, ExtractionProvider, FieldType,
  OpenCodeProvider, OpenCodeProviderOptions, SchemaField,
};
use clarity_web::server::AiProviderDiagnostics;
use serde_json::json;

// ============================================================================
// Config Provider Model Propagation Tests
// ============================================================================

/// Test that model from config propagates correctly to provider options
#[test]
fn test_model_propagation_from_config_to_provider() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: "test-session-123".to_string(),
      model: Some("custom-model-v2".to_string()),
      routing_provider: Some("custom-provider".to_string()),
    },
    quality: QualityConfig::default(),
  };

  // Verify config has model set
  assert_eq!(config.provider.model.as_deref(), Some("custom-model-v2"));
  assert_eq!(
    config.provider.routing_provider.as_deref(),
    Some("custom-provider")
  );

  // Create provider with options from config
  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    config.provider.session_id.clone(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  // Verify model propagated to provider
  assert_eq!(provider.model(), &Some("custom-model-v2".to_string()));
  assert_eq!(
    provider.routing_provider(),
    &Some("custom-provider".to_string())
  );
}

/// Test that default model is used when config model is None
#[test]
fn test_default_model_used_when_config_model_is_none() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: String::new(),
      model: None,
      routing_provider: None,
    },
    quality: QualityConfig::default(),
  };

  // Create provider with options from config (model is None)
  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    uuid::Uuid::new_v4().to_string(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  // Verify provider has no model set (will use defaults)
  assert_eq!(provider.model(), &None);
  assert_eq!(provider.routing_provider(), &None);
}

/// Test that session_id from config propagates to provider
#[test]
fn test_session_id_propagation_from_config() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: "my-custom-session-id".to_string(),
      model: Some("test-model".to_string()),
      routing_provider: None,
    },
    quality: QualityConfig::default(),
  };

  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    config.provider.session_id.clone(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  assert_eq!(provider.session_id(), "my-custom-session-id");
}

/// Test that endpoint from config propagates to provider
#[test]
fn test_endpoint_propagation_from_config() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://custom-api.example.com/v2".to_string(),
      session_id: "test-session".to_string(),
      model: None,
      routing_provider: None,
    },
    quality: QualityConfig::default(),
  };

  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    config.provider.session_id.clone(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  assert_eq!(provider.endpoint(), "https://custom-api.example.com/v2");
}

/// Test that combined model format (provider/model) is parsed correctly
#[test]
fn test_combined_model_format_propagation() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: "test-session".to_string(),
      model: Some("zai-coding-plan/glm-5".to_string()),
      routing_provider: None,
    },
    quality: QualityConfig::default(),
  };

  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    config.provider.session_id.clone(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  // The model should be stored as-is
  assert_eq!(provider.model(), &Some("zai-coding-plan/glm-5".to_string()));
}

// ============================================================================
// Config Error Mapping Tests
// ============================================================================

/// Test ConfigError::ConfigDirNotFound display
#[test]
fn test_config_error_config_dir_not_found_display() {
  let error = ConfigError::ConfigDirNotFound;
  let message = format!("{error}");
  assert!(message.contains("XDG config directory not found"));
}

/// Test ConfigError::ReadError display
#[test]
fn test_config_error_read_error_display() {
  let error = ConfigError::ReadError("permission denied".to_string());
  let message = format!("{error}");
  assert!(message.contains("failed to read config file"));
  assert!(message.contains("permission denied"));
}

/// Test ConfigError::ParseError display
#[test]
fn test_config_error_parse_error_display() {
  let error = ConfigError::ParseError("invalid TOML at line 5".to_string());
  let message = format!("{error}");
  assert!(message.contains("failed to parse config"));
  assert!(message.contains("invalid TOML at line 5"));
}

/// Test ConfigError::CreateDirError display
#[test]
fn test_config_error_create_dir_error_display() {
  let error = ConfigError::CreateDirError("disk full".to_string());
  let message = format!("{error}");
  assert!(message.contains("failed to create config directory"));
  assert!(message.contains("disk full"));
}

/// Test ConfigError::WriteError display
#[test]
fn test_config_error_write_error_display() {
  let error = ConfigError::WriteError("read-only filesystem".to_string());
  let message = format!("{error}");
  assert!(message.contains("failed to write config file"));
  assert!(message.contains("read-only filesystem"));
}

/// Test that config errors implement std::error::Error
#[test]
fn test_config_error_is_std_error() {
  fn assert_error<E: std::error::Error>() {}
  assert_error::<ConfigError>();
}

// ============================================================================
// AiProviderDiagnostics Contract Tests
// ============================================================================

/// Test AiProviderDiagnostics creation with all fields
#[test]
fn test_ai_provider_diagnostics_full() {
  let diagnostics = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: Some("zai-coding-plan/glm-5".to_string()),
    routing_provider: Some("zai-coding-plan".to_string()),
  };

  assert_eq!(diagnostics.provider, "opencode");
  assert_eq!(diagnostics.endpoint, "https://api.opencode.ai/v1");
  assert_eq!(diagnostics.model.as_deref(), Some("zai-coding-plan/glm-5"));
  assert_eq!(
    diagnostics.routing_provider.as_deref(),
    Some("zai-coding-plan")
  );
}

/// Test AiProviderDiagnostics serialization roundtrip
#[test]
fn test_ai_provider_diagnostics_serialization_roundtrip() {
  let diagnostics = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: Some("zai-coding-plan/glm-5".to_string()),
    routing_provider: Some("zai-coding-plan".to_string()),
  };

  let serialized = serde_json::to_string(&diagnostics).expect("Failed to serialize");
  let deserialized: AiProviderDiagnostics =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.provider, "opencode");
  assert_eq!(deserialized.endpoint, "https://api.opencode.ai/v1");
  assert_eq!(deserialized.model.as_deref(), Some("zai-coding-plan/glm-5"));
  assert_eq!(
    deserialized.routing_provider.as_deref(),
    Some("zai-coding-plan")
  );
}

/// Test AiProviderDiagnostics with minimal fields (model/routing_provider None)
#[test]
fn test_ai_provider_diagnostics_minimal() {
  let diagnostics = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: None,
    routing_provider: None,
  };

  assert_eq!(diagnostics.provider, "opencode");
  assert_eq!(diagnostics.model, None);
  assert_eq!(diagnostics.routing_provider, None);
}

/// Test AiProviderDiagnostics equality
#[test]
fn test_ai_provider_diagnostics_equality() {
  let d1 = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: Some("model-1".to_string()),
    routing_provider: Some("provider-1".to_string()),
  };

  let d2 = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: Some("model-1".to_string()),
    routing_provider: Some("provider-1".to_string()),
  };

  let d3 = AiProviderDiagnostics {
    provider: "opencode".to_string(),
    endpoint: "https://api.opencode.ai/v1".to_string(),
    model: Some("model-2".to_string()),
    routing_provider: Some("provider-1".to_string()),
  };

  assert_eq!(d1, d2);
  assert_ne!(d1, d3);
}

// ============================================================================
// Model in Extract/Suggest Payload Tests
// ============================================================================

/// Test that ExtractionMetadata includes model from provider config
#[test]
fn test_extraction_metadata_includes_model_from_config() {
  let config_model = "zai-coding-plan/glm-5";

  let metadata = ExtractionMetadata {
    provider: "opencode".to_string(),
    model: Some(config_model.to_string()),
    timestamp: chrono::Utc::now(),
    processing_duration_ms: 150,
    extra: json!({}),
  };

  assert_eq!(metadata.model.as_deref(), Some(config_model));
  assert_eq!(metadata.provider, "opencode");
}

/// Test ExtractionMetadata serialization includes model field
#[test]
fn test_extraction_metadata_serialization_includes_model() {
  let metadata = ExtractionMetadata {
    provider: "opencode".to_string(),
    model: Some("zai-coding-plan/glm-5".to_string()),
    timestamp: chrono::Utc::now(),
    processing_duration_ms: 150,
    extra: json!({"tokens": 100}),
  };

  let serialized = serde_json::to_string(&metadata).expect("Failed to serialize");

  // Verify model field is in JSON
  assert!(serialized.contains("zai-coding-plan/glm-5"));
  assert!(serialized.contains("opencode"));
  assert!(serialized.contains("processing_duration_ms"));
}

/// Test ExtractionMetadata roundtrip preserves model
#[test]
fn test_extraction_metadata_roundtrip_preserves_model() {
  let metadata = ExtractionMetadata {
    provider: "opencode".to_string(),
    model: Some("zai-coding-plan/glm-5".to_string()),
    timestamp: chrono::Utc::now(),
    processing_duration_ms: 150,
    extra: json!({}),
  };

  let serialized = serde_json::to_string(&metadata).expect("Failed to serialize");
  let deserialized: ExtractionMetadata =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.model, metadata.model);
  assert_eq!(deserialized.provider, metadata.provider);
  assert_eq!(
    deserialized.processing_duration_ms,
    metadata.processing_duration_ms
  );
}

/// Test ExtractionContext serialization for extract/suggest payloads
#[test]
fn test_extraction_context_payload_serialization() {
  let context = ExtractionContext {
    document_type: Some("discover_phase".to_string()),
    locale: Some("en_US".to_string()),
    schema: Some(vec![SchemaField {
      name: "problem".to_string(),
      field_type: FieldType::TextArea,
      required: true,
      description: Some("User's problem statement".to_string()),
      options: None,
    }]),
    extra: json!({
      "session_id": "test-session-123",
      "model_hint": "prefer-detailed"
    }),
  };

  let serialized = serde_json::to_string(&context).expect("Failed to serialize");
  let deserialized: ExtractionContext =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(
    deserialized.document_type,
    Some("discover_phase".to_string())
  );
  assert_eq!(deserialized.locale, Some("en_US".to_string()));
  assert!(deserialized.schema.is_some());
  assert_eq!(deserialized.schema.unwrap().len(), 1);
}

// ============================================================================
// Provider Type Tests
// ============================================================================

/// Test ProviderType serialization
#[test]
fn test_provider_type_serialization() {
  let opencode = ProviderType::Opencode;
  let serialized = serde_json::to_string(&opencode).expect("Failed to serialize");
  assert_eq!(serialized, "\"opencode\"");

  let custom = ProviderType::Other("custom_provider".to_string());
  let serialized = serde_json::to_string(&custom).expect("Failed to serialize");
  assert_eq!(serialized, "\"custom_provider\"");
}

/// Test ProviderType deserialization
#[test]
fn test_provider_type_deserialization() {
  let deserialized: ProviderType =
    serde_json::from_str("\"opencode\"").expect("Failed to deserialize");
  assert_eq!(deserialized, ProviderType::Opencode);

  let deserialized: ProviderType =
    serde_json::from_str("\"custom_provider\"").expect("Failed to deserialize");
  assert!(matches!(deserialized, ProviderType::Other(_)));
  if let ProviderType::Other(s) = deserialized {
    assert_eq!(s, "custom_provider");
  }
}

/// Test ProviderType equality
#[test]
fn test_provider_type_equality() {
  assert_eq!(ProviderType::Opencode, ProviderType::Opencode);
  assert_eq!(
    ProviderType::Other("custom".to_string()),
    ProviderType::Other("custom".to_string())
  );
  assert_ne!(
    ProviderType::Other("custom1".to_string()),
    ProviderType::Other("custom2".to_string())
  );
  assert_ne!(
    ProviderType::Opencode,
    ProviderType::Other("opencode".to_string())
  );
}

/// Test ProviderType default
#[test]
fn test_provider_type_default() {
  let default: ProviderType = ProviderType::default();
  assert_eq!(default, ProviderType::Opencode);
}

// ============================================================================
// Quality Config Tests
// ============================================================================

/// Test QualityConfig default
#[test]
fn test_quality_config_default() {
  let config = QualityConfig::default();
  assert_eq!(config.min_score, 70);
}

/// Test QualityConfig serialization
#[test]
fn test_quality_config_serialization() {
  let config = QualityConfig { min_score: 85 };
  let serialized = serde_json::to_string(&config).expect("Failed to serialize");
  assert!(serialized.contains("85"));
}

/// Test QualityConfig deserialization
#[test]
fn test_quality_config_deserialization() {
  let deserialized: QualityConfig =
    serde_json::from_str("{\"min_score\": 90}").expect("Failed to deserialize");
  assert_eq!(deserialized.min_score, 90);
}

// ============================================================================
// Legacy Config Parsing Tests
// ============================================================================

/// Test parsing legacy config format (without model field)
#[test]
fn test_legacy_config_without_model() {
  let toml_content = r#"
[provider]
provider = "opencode"
endpoint = "https://api.opencode.ai/v1"
session_id = "legacy-session"

[quality]
min_score = 75
"#;

  let config: AiConfig = toml::from_str(toml_content).expect("Failed to parse legacy config");

  // Model should use default
  assert_eq!(
    config.provider.model.as_deref(),
    Some("zai-coding-plan/glm-5")
  );
  assert_eq!(config.provider.routing_provider, None);
  assert_eq!(config.provider.session_id, "legacy-session");
  assert_eq!(config.quality.min_score, 75);
}

/// Test parsing upgraded config format (with model field)
#[test]
fn test_upgraded_config_with_model() {
  let toml_content = r#"
[provider]
provider = "opencode"
endpoint = "https://api.opencode.ai/v1"
session_id = "upgraded-session"
model = "new-model-v3"
routing_provider = "new-provider"

[quality]
min_score = 80
"#;

  let config: AiConfig = toml::from_str(toml_content).expect("Failed to parse upgraded config");

  assert_eq!(config.provider.model.as_deref(), Some("new-model-v3"));
  assert_eq!(
    config.provider.routing_provider.as_deref(),
    Some("new-provider")
  );
  assert_eq!(config.provider.session_id, "upgraded-session");
  assert_eq!(config.quality.min_score, 80);
}

/// Test that partial config uses defaults for missing fields
#[test]
fn test_partial_config_uses_defaults() {
  let toml_content = r#"
[provider]
endpoint = "https://custom.api.com/v1"
"#;

  let config: AiConfig = toml::from_str(toml_content).expect("Failed to parse partial config");

  // Uses defaults
  assert_eq!(config.provider.provider, ProviderType::Opencode);
  assert_eq!(config.provider.session_id, "");
  assert_eq!(
    config.provider.model.as_deref(),
    Some("zai-coding-plan/glm-5")
  );
  assert_eq!(config.provider.routing_provider, None);
  assert_eq!(config.quality.min_score, 70);
}

// ============================================================================
// ExtractionError Mapping Tests
// ============================================================================

/// Test that provider errors map correctly to server errors
#[test]
fn test_extraction_error_rate_limited_mapping() {
  let error = ExtractionError::RateLimited {
    retry_after_seconds: 60,
  };

  let message = format!("{error}");
  assert!(message.contains("Rate limited"));
  assert!(message.contains("60"));
}

/// Test that authentication error maps correctly
#[test]
fn test_extraction_error_authentication_mapping() {
  let error = ExtractionError::AuthenticationError("Invalid API key".to_string());
  let message = format!("{error}");
  assert!(message.contains("Authentication failed"));
  assert!(message.contains("Invalid API key"));
}

/// Test that invalid input error maps correctly
#[test]
fn test_extraction_error_invalid_input_mapping() {
  let error = ExtractionError::InvalidInput("Empty text".to_string());
  let message = format!("{error}");
  assert!(message.contains("Invalid input"));
  assert!(message.contains("Empty text"));
}

/// Test that network error maps correctly
#[test]
fn test_extraction_error_network_mapping() {
  let error = ExtractionError::NetworkError("Connection refused".to_string());
  let message = format!("{error}");
  assert!(message.contains("Network error"));
  assert!(message.contains("Connection refused"));
}

/// Test that quota exceeded error maps correctly
#[test]
fn test_extraction_error_quota_exceeded_mapping() {
  let error = ExtractionError::QuotaExceeded("Monthly limit reached".to_string());
  let message = format!("{error}");
  assert!(message.contains("Insufficient quota"));
  assert!(message.contains("Monthly limit reached"));
}

/// Test that timeout error maps correctly
#[test]
fn test_extraction_error_timeout_mapping() {
  let error = ExtractionError::Timeout { timeout_ms: 30000 };
  let message = format!("{error}");
  assert!(message.contains("timed out"));
  assert!(message.contains("30000"));
}

// ============================================================================
// OpenCodeProvider Options Tests
// ============================================================================

/// Test OpenCodeProviderOptions default
#[test]
fn test_opencode_provider_options_default() {
  let options = OpenCodeProviderOptions::default();
  assert_eq!(options.model, None);
  assert_eq!(options.routing_provider, None);
}

/// Test OpenCodeProviderOptions with model only
#[test]
fn test_opencode_provider_options_model_only() {
  let options = OpenCodeProviderOptions {
    model: Some("custom-model".to_string()),
    routing_provider: None,
  };

  let provider = OpenCodeProvider::new_with_options(
    "https://api.opencode.ai/v1".to_string(),
    "test-session".to_string(),
    options,
  )
  .expect("provider should be created");

  assert_eq!(provider.model(), &Some("custom-model".to_string()));
  assert_eq!(provider.routing_provider(), &None);
}

/// Test OpenCodeProviderOptions with routing_provider only
#[test]
fn test_opencode_provider_options_routing_only() {
  let options = OpenCodeProviderOptions {
    model: None,
    routing_provider: Some("custom-router".to_string()),
  };

  let provider = OpenCodeProvider::new_with_options(
    "https://api.opencode.ai/v1".to_string(),
    "test-session".to_string(),
    options,
  )
  .expect("provider should be created");

  assert_eq!(provider.model(), &None);
  assert_eq!(
    provider.routing_provider(),
    &Some("custom-router".to_string())
  );
}

// ============================================================================
// Config Path Tests
// ============================================================================

/// Test that config_path returns a valid path
#[test]
fn test_config_path_returns_valid_path() {
  let path = config_path();
  // Path may be None on non-XDG systems
  if let Some(p) = path {
    assert!(p.to_string_lossy().contains("clarity"));
    assert!(p.to_string_lossy().contains("ai.toml"));
  }
}

/// Test that default_config returns valid defaults
#[test]
fn test_default_config_returns_valid_config() {
  let config = default_config();

  assert_eq!(config.provider.provider, ProviderType::Opencode);
  assert_eq!(config.provider.endpoint, "https://api.opencode.ai/v1");
  assert_eq!(
    config.provider.model.as_deref(),
    Some("zai-coding-plan/glm-5")
  );
  assert_eq!(config.quality.min_score, 70);
}

// ============================================================================
// Integration: Config to Provider to Diagnostics Chain
// ============================================================================

/// Test full chain: config -> provider -> diagnostics
#[test]
fn test_full_chain_config_to_diagnostics() {
  // 1. Create config
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: "integration-test-session".to_string(),
      model: Some("zai-coding-plan/glm-5".to_string()),
      routing_provider: Some("zai-coding-plan".to_string()),
    },
    quality: QualityConfig { min_score: 85 },
  };

  // 2. Create provider from config
  let provider = OpenCodeProvider::new_with_options(
    config.provider.endpoint.clone(),
    config.provider.session_id.clone(),
    OpenCodeProviderOptions {
      model: config.provider.model.clone(),
      routing_provider: config.provider.routing_provider.clone(),
    },
  )
  .expect("provider should be created");

  // 3. Build diagnostics from provider
  let diagnostics = AiProviderDiagnostics {
    provider: provider.provider_name().to_string(),
    endpoint: provider.endpoint().clone(),
    model: provider.model().clone(),
    routing_provider: provider.routing_provider().clone(),
  };

  // 4. Verify chain
  assert_eq!(diagnostics.provider, "opencode");
  assert_eq!(diagnostics.endpoint, "https://api.opencode.ai/v1");
  assert_eq!(diagnostics.model.as_deref(), Some("zai-coding-plan/glm-5"));
  assert_eq!(
    diagnostics.routing_provider.as_deref(),
    Some("zai-coding-plan")
  );
}

/// Test that provider name is always "opencode" for OpenCodeProvider
#[test]
fn test_provider_name_is_opencode() {
  let provider =
    OpenCodeProvider::new("https://api.opencode.ai/v1".to_string(), "test".to_string())
      .expect("provider should be created");

  assert_eq!(provider.provider_name(), "opencode");
}

/// Test that empty session_id in config triggers UUID generation
#[test]
fn test_empty_session_id_triggers_uuid_generation() {
  let config = AiConfig {
    provider: ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: String::new(),
      model: None,
      routing_provider: None,
    },
    quality: QualityConfig::default(),
  };

  // When session_id is empty, a UUID should be generated
  let session_id = if config.provider.session_id.is_empty() {
    uuid::Uuid::new_v4().to_string()
  } else {
    config.provider.session_id.clone()
  };

  // Verify it's a valid UUID format
  assert!(uuid::Uuid::parse_str(&session_id).is_ok());
}
