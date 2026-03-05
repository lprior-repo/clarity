#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Provider resolution module
//!
//! Resolves provider configuration to create concrete provider instances.

use super::r#trait::ExtractionError;
use serde::{Deserialize, Serialize};

/// Resolved provider configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProviderConfig {
  /// Provider type identifier
  pub provider_type: String,
  /// Model identifier
  pub model: Option<String>,
  /// API endpoint URL
  pub endpoint: Option<String>,
  /// Additional configuration
  pub extra: serde_json::Value,
}

/// Resolve provider configuration from a config value
///
/// # Errors
/// Returns an error if the configuration is invalid or missing required fields.
pub fn resolve_provider_config(
  config: &serde_json::Value,
) -> Result<ResolvedProviderConfig, ExtractionError> {
  let provider_type = config
    .get("provider_type")
    .and_then(|v| v.as_str())
    .unwrap_or("opencode")
    .to_string();

  let model = config.get("model").and_then(|v| v.as_str()).map(String::from);

  let endpoint = config.get("endpoint").and_then(|v| v.as_str()).map(String::from);

  let extra = config.get("extra").cloned().unwrap_or(serde_json::Value::Null);

  Ok(ResolvedProviderConfig {
    provider_type,
    model,
    endpoint,
    extra,
  })
}

/// Resolve provider from a provider configuration string
///
/// # Errors
/// Returns an error if the configuration string cannot be parsed or is invalid.
pub fn resolve_from_provider_config(
  config_str: &str,
) -> Result<ResolvedProviderConfig, ExtractionError> {
  let config: serde_json::Value = serde_json::from_str(config_str)
    .map_err(|e| ExtractionError::ConfigurationError(format!("Invalid JSON config: {e}")))?;

  resolve_provider_config(&config)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_resolve_provider_config_defaults() {
    let config = json!({});
    let resolved = resolve_provider_config(&config).unwrap();
    assert_eq!(resolved.provider_type, "opencode");
    assert!(resolved.model.is_none());
    assert!(resolved.endpoint.is_none());
  }

  #[test]
  fn test_resolve_provider_config_with_values() {
    let config = json!({
      "provider_type": "openai",
      "model": "gpt-4",
      "endpoint": "https://api.openai.com/v1"
    });
    let resolved = resolve_provider_config(&config).unwrap();
    assert_eq!(resolved.provider_type, "openai");
    assert_eq!(resolved.model, Some("gpt-4".to_string()));
    assert_eq!(
      resolved.endpoint,
      Some("https://api.openai.com/v1".to_string())
    );
  }

  #[test]
  fn test_resolve_from_provider_config() {
    let config_str = r#"{"provider_type": "test", "model": "test-model"}"#;
    let resolved = resolve_from_provider_config(config_str).unwrap();
    assert_eq!(resolved.provider_type, "test");
    assert_eq!(resolved.model, Some("test-model".to_string()));
  }
}
