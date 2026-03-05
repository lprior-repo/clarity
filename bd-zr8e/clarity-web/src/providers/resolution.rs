#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Provider resolution module for deterministic provider/model configuration.
//!
//! This module provides a shared resolution routine used by both:
//! - AI_PROVIDER singleton initialization
//! - get_ai_provider_status_server diagnostics
//!
//! ## Invariants
//! - Diagnostics responses NEVER include secret tokens or API keys
//! - Bootstrap and diagnostics use one shared resolution routine
//! - Supports both legacy config (model only) and upgraded config (model + routing_provider)

use serde::{Deserialize, Serialize};

use crate::config::ai::{AiConfig, ProviderConfig};

/// Resolved provider configuration for use in diagnostics and provider initialization.
///
/// This struct contains only non-sensitive information suitable for:
/// - UI display
/// - Logging
/// - Diagnostics endpoints
///
/// # Security
/// This type NEVER contains API keys, session tokens, or other secrets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedProviderConfig {
  /// Provider type (e.g., "opencode")
  pub provider_type: String,
  /// API endpoint URL (sanitized, no embedded credentials)
  pub endpoint: String,
  /// Resolved model identifier
  pub model: Option<String>,
  /// Resolved routing provider for model backends
  pub routing_provider: Option<String>,
}

impl Default for ResolvedProviderConfig {
  fn default() -> Self {
    Self {
      provider_type: "opencode".to_string(),
      endpoint: "https://api.opencode.ai/v1".to_string(),
      model: Some("zai-coding-plan/glm-5".to_string()),
      routing_provider: Some("zai-coding-plan".to_string()),
    }
  }
}

/// Resolve provider configuration from AiConfig.
///
/// This is the single source of truth for provider/model resolution,
/// used by both bootstrap and diagnostics.
///
/// # Arguments
/// * `config` - The loaded AI configuration
///
/// # Returns
/// `ResolvedProviderConfig` with sanitized, non-sensitive values
#[must_use]
pub fn resolve_provider_config(config: &AiConfig) -> ResolvedProviderConfig {
  resolve_from_provider_config(&config.provider)
}

/// Resolve provider configuration from ProviderConfig.
///
/// This handles both legacy config (model only) and upgraded config (model + routing_provider).
///
/// # Resolution Logic
/// 1. If `routing_provider` is set, use it directly
/// 2. If `model` contains a `/` (e.g., "provider/model"), extract provider from model string
/// 3. Otherwise, use defaults
#[must_use]
pub fn resolve_from_provider_config(config: &ProviderConfig) -> ResolvedProviderConfig {
  let provider_type = match &config.provider {
    crate::config::ai::ProviderType::Opencode => "opencode".to_string(),
    crate::config::ai::ProviderType::Other(s) => s.clone(),
  };

  // Sanitize endpoint - remove any embedded credentials
  let endpoint = sanitize_endpoint(&config.endpoint);

  // Resolve routing_provider and model
  let (model, routing_provider) = resolve_model_and_provider(&config.model, &config.routing_provider);

  ResolvedProviderConfig {
    provider_type,
    endpoint,
    model,
    routing_provider,
  }
}

/// Resolve model and routing_provider from config values.
///
/// Handles both legacy (model only) and upgraded (model + routing_provider) configs.
fn resolve_model_and_provider(
  model: &Option<String>,
  routing_provider: &Option<String>,
) -> (Option<String>, Option<String>) {
  match (model, routing_provider) {
    // Upgraded config: both specified
    (Some(m), Some(rp)) => (Some(m.clone()), Some(rp.clone())),

    // Legacy config: model contains provider/model format
    (Some(m), None) => {
      if let Some((provider, model_id)) = m.split_once('/') {
        // Model is in "provider/model" format
        (Some(model_id.to_string()), Some(provider.to_string()))
      } else {
        // Model is just the model ID, no routing provider
        (Some(m.clone()), None)
      }
    }

    // No model specified
    (None, Some(rp)) => (None, Some(rp.clone())),
    (None, None) => (None, None),
  }
}

/// Sanitize endpoint URL to remove any embedded credentials.
///
/// # Security
/// Ensures no API keys or credentials are embedded in the URL.
fn sanitize_endpoint(endpoint: &str) -> String {
  // Remove any credentials from the URL
  // e.g., "https://user:pass@api.example.com" -> "https://api.example.com"

  if let Ok(mut url) = url::Url::parse(endpoint) {
    // Remove username and password
    // set_username returns Result<(), ()> which we ignore
    let _: Result<(), ()> = url.set_username("");
    let _: Result<(), ()> = url.set_password(None);
    url.to_string()
  } else {
    // If parsing fails, return as-is but this shouldn't happen with valid URLs
    endpoint.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::ai::{AiConfig, ProviderConfig, ProviderType};

  #[test]
  fn test_default_resolved_config() {
    let config = ResolvedProviderConfig::default();

    assert_eq!(config.provider_type, "opencode");
    assert_eq!(config.endpoint, "https://api.opencode.ai/v1");
    assert_eq!(config.model, Some("zai-coding-plan/glm-5".to_string()));
    assert_eq!(config.routing_provider, Some("zai-coding-plan".to_string()));
  }

  #[test]
  fn test_resolve_with_full_config() {
    let provider_config = ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.example.com/v1".to_string(),
      session_id: "secret-session".to_string(), // Should NOT appear in resolved
      model: Some("my-provider/my-model".to_string()),
      routing_provider: Some("my-provider".to_string()),
    };

    let resolved = resolve_from_provider_config(&provider_config);

    assert_eq!(resolved.provider_type, "opencode");
    assert_eq!(resolved.endpoint, "https://api.example.com/v1");
    assert_eq!(resolved.model, Some("my-model".to_string()));
    assert_eq!(resolved.routing_provider, Some("my-provider".to_string()));
    // session_id should NOT be in resolved config
    assert!(!format!("{resolved:?}").contains("secret-session"));
  }

  #[test]
  fn test_resolve_legacy_config_with_slash_model() {
    let provider_config = ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: String::new(),
      model: Some("provider-id/model-name".to_string()),
      routing_provider: None,
    };

    let resolved = resolve_from_provider_config(&provider_config);

    // Should extract provider from model string
    assert_eq!(resolved.model, Some("model-name".to_string()));
    assert_eq!(resolved.routing_provider, Some("provider-id".to_string()));
  }

  #[test]
  fn test_resolve_legacy_config_without_slash() {
    let provider_config = ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: String::new(),
      model: Some("just-a-model".to_string()),
      routing_provider: None,
    };

    let resolved = resolve_from_provider_config(&provider_config);

    assert_eq!(resolved.model, Some("just-a-model".to_string()));
    assert_eq!(resolved.routing_provider, None);
  }

  #[test]
  fn test_resolve_with_other_provider_type() {
    let provider_config = ProviderConfig {
      provider: ProviderType::Other("custom-provider".to_string()),
      endpoint: "https://custom.api.com/v1".to_string(),
      session_id: String::new(),
      model: Some("model-v1".to_string()),
      routing_provider: None,
    };

    let resolved = resolve_from_provider_config(&provider_config);

    assert_eq!(resolved.provider_type, "custom-provider");
    assert_eq!(resolved.endpoint, "https://custom.api.com/v1");
  }

  #[test]
  fn test_sanitize_endpoint_removes_credentials() {
    // URL with embedded credentials
    let endpoint = "https://user:secret@api.example.com/v1";
    let sanitized = sanitize_endpoint(endpoint);

    assert_eq!(sanitized, "https://api.example.com/v1");
    assert!(!sanitized.contains("user"));
    assert!(!sanitized.contains("secret"));
  }

  #[test]
  fn test_sanitize_endpoint_preserves_clean_url() {
    let endpoint = "https://api.opencode.ai/v1";
    let sanitized = sanitize_endpoint(endpoint);

    assert_eq!(sanitized, endpoint);
  }

  #[test]
  fn test_resolved_config_serialization() {
    let config = ResolvedProviderConfig {
      provider_type: "opencode".to_string(),
      endpoint: "https://api.example.com/v1".to_string(),
      model: Some("model-v1".to_string()),
      routing_provider: Some("provider-x".to_string()),
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: ResolvedProviderConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.provider_type, config.provider_type);
    assert_eq!(parsed.endpoint, config.endpoint);
    assert_eq!(parsed.model, config.model);
    assert_eq!(parsed.routing_provider, config.routing_provider);
  }

  #[test]
  fn test_resolved_config_no_secrets_in_debug() {
    let config = ResolvedProviderConfig {
      provider_type: "opencode".to_string(),
      endpoint: "https://api.example.com/v1".to_string(),
      model: Some("model-v1".to_string()),
      routing_provider: Some("provider-x".to_string()),
    };

    let debug_str = format!("{config:?}");

    // Should not contain any secret-related field names
    assert!(!debug_str.contains("api_key"));
    assert!(!debug_str.contains("session_id"));
    assert!(!debug_str.contains("token"));
    assert!(!debug_str.contains("secret"));
  }

  #[test]
  fn test_resolve_from_full_ai_config() {
    let ai_config = AiConfig::default();
    let resolved = resolve_provider_config(&ai_config);

    assert_eq!(resolved.provider_type, "opencode");
    // Default model has "zai-coding-plan/glm-5" format
    assert_eq!(resolved.model, Some("glm-5".to_string()));
    assert_eq!(resolved.routing_provider, Some("zai-coding-plan".to_string()));
  }

  #[test]
  fn test_empty_model_and_routing() {
    let provider_config = ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.opencode.ai/v1".to_string(),
      session_id: String::new(),
      model: None,
      routing_provider: None,
    };

    let resolved = resolve_from_provider_config(&provider_config);

    assert_eq!(resolved.model, None);
    assert_eq!(resolved.routing_provider, None);
  }
}
