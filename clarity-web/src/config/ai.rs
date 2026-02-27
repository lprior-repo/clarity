#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Domain errors for AI configuration
#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("XDG config directory not found")]
  ConfigDirNotFound,

  #[error("failed to read config file: {0}")]
  ReadError(String),

  #[error("failed to parse config: {0}")]
  ParseError(String),

  #[error("failed to create config directory: {0}")]
  CreateDirError(String),

  #[error("failed to write config file: {0}")]
  WriteError(String),
}

/// AI provider configuration
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
  Opencode,
  Other(String),
}

impl Default for ProviderType {
  fn default() -> Self {
    Self::Opencode
  }
}

impl Serialize for ProviderType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Self::Opencode => serializer.serialize_str("opencode"),
      Self::Other(s) => serializer.serialize_str(s),
    }
  }
}

impl<'de> Deserialize<'de> for ProviderType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct ProviderTypeVisitor;

    impl serde::de::Visitor<'_> for ProviderTypeVisitor {
      type Value = ProviderType;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing provider type")
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        match value {
          "opencode" => Ok(ProviderType::Opencode),
          other => Ok(ProviderType::Other(other.to_string())),
        }
      }
    }

    deserializer.deserialize_string(ProviderTypeVisitor)
  }
}

/// Provider connection details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
  #[serde(default)]
  pub provider: ProviderType,

  #[serde(default = "default_endpoint")]
  pub endpoint: String,

  #[serde(default)]
  pub session_id: String,
}

impl Default for ProviderConfig {
  fn default() -> Self {
    Self {
      provider: ProviderType::default(),
      endpoint: default_endpoint(),
      session_id: String::new(),
    }
  }
}

fn default_endpoint() -> String {
  "https://api.opencode.ai/v1".to_string()
}

/// Quality scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityConfig {
  #[serde(default = "default_quality_min_score")]
  pub min_score: u8,
}

impl Default for QualityConfig {
  fn default() -> Self {
    Self {
      min_score: default_quality_min_score(),
    }
  }
}

fn default_quality_min_score() -> u8 {
  70
}

/// Complete AI configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiConfig {
  #[serde(default)]
  pub provider: ProviderConfig,

  #[serde(default)]
  pub quality: QualityConfig,
}

impl Default for AiConfig {
  fn default() -> Self {
    Self {
      provider: ProviderConfig::default(),
      quality: QualityConfig::default(),
    }
  }
}

/// Get the path to the AI configuration file
///
/// Returns `None` if XDG config directory cannot be determined
pub fn config_path() -> Option<PathBuf> {
  dirs::config_dir()
    .map(|dir| dir.join("clarity"))
    .map(|dir| dir.join("ai.toml"))
}

/// Load AI configuration from XDG config directory
///
/// Creates default config with 0600 permissions if not found
pub fn load_ai_config() -> Result<AiConfig, ConfigError> {
  let config_file = config_path().ok_or(ConfigError::ConfigDirNotFound)?;

  // Try to read existing config
  match std::fs::read_to_string(&config_file) {
    Ok(content) => toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string())),
    Err(_) => {
      // Config doesn't exist, create default
      create_default_config(&config_file)
    }
  }
}

/// Create default configuration file with secure permissions
fn create_default_config(path: &PathBuf) -> Result<AiConfig, ConfigError> {
  let config = AiConfig::default();

  // Ensure parent directory exists
  let parent = path
    .parent()
    .ok_or_else(|| ConfigError::CreateDirError("no parent directory".to_string()))?;

  std::fs::create_dir_all(parent).map_err(|e| ConfigError::CreateDirError(e.to_string()))?;

  // Serialize to TOML
  let toml_content =
    toml::to_string_pretty(&config).map_err(|e| ConfigError::WriteError(e.to_string()))?;

  // Write with 0600 permissions (owner read/write only)
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;

    // Create file with restricted permissions using OpenOptionsExt
    std::fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .mode(0o600)
      .open(path)
      .and_then(|mut file| std::io::Write::write_all(&mut file, toml_content.as_bytes()))
      .map_err(|e| ConfigError::WriteError(e.to_string()))?;
  }

  #[cfg(not(unix))]
  {
    std::fs::write(path, toml_content).map_err(|e| ConfigError::WriteError(e.to_string()))?;
  }

  Ok(config)
}

/// Get default configuration without creating files
pub fn default_config() -> AiConfig {
  AiConfig::default()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  /// Test default configuration values
  #[test]
  fn test_default_config() {
    let config = default_config();

    assert_eq!(config.provider.provider, ProviderType::Opencode);
    assert_eq!(config.provider.endpoint, "https://api.opencode.ai/v1");
    assert_eq!(config.provider.session_id, "");
    assert_eq!(config.quality.min_score, 70);
  }

  /// Test ProviderConfig default
  #[test]
  fn test_provider_config_default() {
    let provider = ProviderConfig::default();

    assert_eq!(provider.provider, ProviderType::Opencode);
    assert_eq!(provider.endpoint, "https://api.opencode.ai/v1");
    assert!(provider.session_id.is_empty());
  }

  /// Test QualityConfig default
  #[test]
  fn test_quality_config_default() {
    let quality = QualityConfig::default();

    assert_eq!(quality.min_score, 70);
  }

  /// Test ProviderType serialization/deserialization
  #[test]
  fn test_provider_type_serde() {
    // Test deserialization via serde_json (TOML requires key=value)
    let deserialized: ProviderType = serde_json::from_str("\"opencode\"").unwrap();
    assert_eq!(deserialized, ProviderType::Opencode);

    // Test deserialization of unknown provider
    let custom: ProviderType = serde_json::from_str("\"custom_provider\"").unwrap();
    assert!(matches!(custom, ProviderType::Other(_)));
    if let ProviderType::Other(s) = custom {
      assert_eq!(s, "custom_provider");
    }

    // Test serialization within config struct
    let config = ProviderConfig {
      provider: ProviderType::Opencode,
      ..Default::default()
    };
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("opencode"));

    // Test round-trip through TOML
    let parsed: ProviderConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(parsed.provider, ProviderType::Opencode);

    // Test custom provider in TOML config
    let toml_content = r#"
[provider]
provider = "custom_openai"
endpoint = "https://api.openai.com/v1"
"#;
    let parsed_config: AiConfig = toml::from_str(toml_content).unwrap();
    assert!(matches!(
      parsed_config.provider.provider,
      ProviderType::Other(_)
    ));
  }

  /// Test AiConfig serialization/deserialization
  #[test]
  fn test_ai_config_serde() {
    let config = AiConfig::default();

    // Serialize
    let toml_str = toml::to_string_pretty(&config).unwrap();

    // Deserialize
    let parsed: AiConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed, config);
  }

  /// Test config parsing from TOML
  #[test]
  fn test_parse_toml_config() {
    let toml_content = r#"
[provider]
provider = "opencode"
endpoint = "https://api.example.com/v1"
session_id = "test-session-123"

[quality]
min_score = 85
"#;

    let config: AiConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.provider.provider, ProviderType::Opencode);
    assert_eq!(config.provider.endpoint, "https://api.example.com/v1");
    assert_eq!(config.provider.session_id, "test-session-123");
    assert_eq!(config.quality.min_score, 85);
  }

  /// Test partial config with defaults
  #[test]
  fn test_partial_config_uses_defaults() {
    let toml_content = r#"
[provider]
provider = "opencode"
"#;

    let config: AiConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.provider.provider, ProviderType::Opencode);
    assert_eq!(
      config.provider.endpoint,
      "https://api.opencode.ai/v1" // default
    );
    assert_eq!(config.quality.min_score, 70); // default
  }

  /// Test config file creation in temp directory
  #[test]
  fn test_create_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("ai.toml");

    let config = create_default_config(&config_path).unwrap();

    // Verify config values
    assert_eq!(config.provider.provider, ProviderType::Opencode);
    assert_eq!(config.quality.min_score, 70);

    // Verify file was created
    assert!(config_path.exists());

    // Verify file contents can be parsed
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: AiConfig = toml::from_str(&content).unwrap();
    assert_eq!(parsed, config);

    // Verify file permissions on Unix
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let metadata = fs::metadata(&config_path).unwrap();
      let mode = metadata.permissions().mode();
      assert_eq!(mode & 0o777, 0o600);
    }
  }

  /// Test loading existing config
  #[test]
  fn test_load_existing_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("ai.toml");

    // Write config file
    let toml_content = r#"
[provider]
provider = "opencode"
endpoint = "https://test.example.com/v1"
session_id = "test-session"

[quality]
min_score = 90
"#;
    fs::write(&config_path, toml_content).unwrap();

    // Load and verify
    let config = load_ai_config_from_path(&config_path).unwrap();

    assert_eq!(config.provider.endpoint, "https://test.example.com/v1");
    assert_eq!(config.provider.session_id, "test-session");
    assert_eq!(config.quality.min_score, 90);
  }

  /// Test config creation when file doesn't exist
  #[test]
  fn test_creates_default_when_missing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("ai.toml");

    // Ensure file doesn't exist
    assert!(!config_path.exists());

    // Load should create default
    let config = load_ai_config_from_path(&config_path).unwrap();

    assert_eq!(config.provider.provider, ProviderType::Opencode);
    assert_eq!(config.quality.min_score, 70);
    assert!(config_path.exists());
  }

  // Helper function to test loading from specific path
  fn load_ai_config_from_path(path: &PathBuf) -> Result<AiConfig, ConfigError> {
    match std::fs::read_to_string(path) {
      Ok(content) => toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string())),
      Err(_) => create_default_config(path),
    }
  }

  /// Test quality score boundary
  #[test]
  fn test_quality_score_range() {
    let toml_content = r"
[quality]
min_score = 100
";

    let config: AiConfig = toml::from_str(toml_content).unwrap();
    assert_eq!(config.quality.min_score, 100);
  }

  /// Test empty session_id handling
  #[test]
  fn test_empty_session_id() {
    let config = ProviderConfig {
      provider: ProviderType::Opencode,
      endpoint: "https://api.example.com".to_string(),
      session_id: String::new(),
    };

    assert!(config.session_id.is_empty());
    assert_eq!(config.provider, ProviderType::Opencode);
  }
}
