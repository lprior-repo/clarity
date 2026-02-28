//! Provider configuration resolution
//!
//! This module handles resolving provider configuration from various sources
//! (environment variables, config files, defaults).

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Resolved provider configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProviderConfig {
  /// Provider name (e.g., "opencode", "openai")
  pub provider: String,
  /// Model to use (provider-specific)
  pub model: Option<String>,
  /// API endpoint (for custom deployments)
  pub endpoint: Option<String>,
  /// API key (if required)
  pub api_key: Option<String>,
  /// Additional configuration options
  pub options: Vec<String>,
}

impl Default for ResolvedProviderConfig {
  fn default() -> Self {
    Self {
      provider: "opencode".to_string(),
      model: None,
      endpoint: None,
      api_key: None,
      options: Vec::new(),
    }
  }
}

impl ResolvedProviderConfig {
  /// Create a new resolved config with the given provider
  #[must_use]
  pub fn new(provider: String) -> Self {
    Self {
      provider,
      ..Self::default()
    }
  }

  /// Create a resolved config with provider and model
  #[must_use]
  pub fn with_model(provider: String, model: String) -> Self {
    Self {
      provider,
      model: Some(model),
      ..Self::default()
    }
  }

  /// Check if the configuration is valid
  #[must_use]
  pub fn is_valid(&self) -> bool {
    !self.provider.is_empty()
  }
}

/// Resolve provider configuration from environment and defaults.
///
/// This function attempts to resolve the provider configuration by:
/// 1. Checking environment variables
/// 2. Falling back to defaults
///
/// # Returns
/// A resolved provider configuration
#[must_use]
pub fn resolve_provider_config() -> ResolvedProviderConfig {
  // Check for environment variable overrides
  if let Ok(provider) = std::env::var("CLARITY_AI_PROVIDER") {
    let model = std::env::var("CLARITY_AI_MODEL").ok();
    let endpoint = std::env::var("CLARITY_AI_ENDPOINT").ok();
    let api_key = std::env::var("CLARITY_AI_API_KEY").ok();

    return ResolvedProviderConfig {
      provider,
      model,
      endpoint,
      api_key,
      options: Vec::new(),
    };
  }

  // Default to opencode provider
  ResolvedProviderConfig::default()
}

/// Resolve provider configuration from an existing config structure.
///
/// This function takes a pre-existing configuration and resolves it
/// into a standardized `ResolvedProviderConfig`.
///
/// # Arguments
/// * `config` - Optional provider configuration (may be from app state)
///
/// # Returns
/// A resolved provider configuration
#[must_use]
pub fn resolve_from_provider_config(config: Option<&ProviderConfigSource>) -> ResolvedProviderConfig {
  match config {
    Some(cfg) => ResolvedProviderConfig {
      provider: cfg.provider.clone(),
      model: cfg.model.clone(),
      endpoint: cfg.endpoint.clone(),
      api_key: cfg.api_key.clone(),
      options: cfg.options.clone(),
    },
    None => resolve_provider_config(),
  }
}

/// Source configuration for provider resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderConfigSource {
  /// Provider name
  pub provider: String,
  /// Model to use
  pub model: Option<String>,
  /// API endpoint
  pub endpoint: Option<String>,
  /// API key
  pub api_key: Option<String>,
  /// Additional options
  pub options: Vec<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resolved_config_default() {
    let config = ResolvedProviderConfig::default();
    assert_eq!(config.provider, "opencode");
    assert!(config.model.is_none());
    assert!(config.is_valid());
  }

  #[test]
  fn test_resolved_config_new() {
    let config = ResolvedProviderConfig::new("openai".to_string());
    assert_eq!(config.provider, "openai");
    assert!(config.model.is_none());
    assert!(config.is_valid());
  }

  #[test]
  fn test_resolved_config_with_model() {
    let config = ResolvedProviderConfig::with_model("openai".to_string(), "gpt-4".to_string());
    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, Some("gpt-4".to_string()));
    assert!(config.is_valid());
  }

  #[test]
  fn test_resolved_config_invalid_empty_provider() {
    let config = ResolvedProviderConfig {
      provider: String::new(),
      ..Default::default()
    };
    assert!(!config.is_valid());
  }

  #[test]
  fn test_resolve_from_provider_config_none() {
    let config = resolve_from_provider_config(None);
    assert_eq!(config.provider, "opencode");
  }

  #[test]
  fn test_resolve_from_provider_config_some() {
    let source = ProviderConfigSource {
      provider: "anthropic".to_string(),
      model: Some("claude-3".to_string()),
      endpoint: None,
      api_key: Some("test-key".to_string()),
      options: Vec::new(),
    };
    let config = resolve_from_provider_config(Some(&source));
    assert_eq!(config.provider, "anthropic");
    assert_eq!(config.model, Some("claude-3".to_string()));
    assert_eq!(config.api_key, Some("test-key".to_string()));
  }
}
