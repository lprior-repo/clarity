//! Configuration file support for Intent CLI
//!
//! Loads and parses .intentrc.yaml configuration file.
//!
//! Ported from intent-cli/src/intent/config.gleam

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

/// Configuration file path
const CONFIG_FILE_PATH: &str = ".intentrc.yaml";

/// Intent CLI configuration from .intentrc.yaml
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
  pub default_profile: String,
  pub default_output_format: String,
  pub default_strategy: String,
  pub watch_debounce_ms: u32,
  pub max_cache_entries: u32,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      default_profile: "api".to_string(),
      default_output_format: "json".to_string(),
      default_strategy: "page_rank".to_string(),
      watch_debounce_ms: 500,
      max_cache_entries: 50,
    }
  }
}

/// Errors that can occur during configuration loading
#[derive(Debug, Clone, Error)]
pub enum ConfigError {
  #[error("failed to read config file '{path}': {reason}")]
  ReadError { path: String, reason: String },

  #[error("invalid numeric value for '{field}': '{value}'")]
  InvalidNumber { field: String, value: String },

  #[error("invalid default_profile '{input}'. Valid options: {valid_options}")]
  InvalidProfile {
    input: String,
    valid_options: String,
  },

  #[error("invalid default_output_format '{input}'. Valid options: {valid_options}")]
  InvalidOutputFormat {
    input: String,
    valid_options: String,
  },

  #[error("invalid default_strategy '{input}'. Valid options: {valid_options}")]
  InvalidStrategy {
    input: String,
    valid_options: String,
  },

  #[error("invalid watch_debounce_ms: {value}. Must be between 0 and {max}")]
  InvalidWatchDebounce { value: u32, max: u32 },

  #[error("invalid max_cache_entries: {value}. Must be between 0 and {max}")]
  InvalidMaxCacheEntries { value: u32, max: u32 },
}

/// Load configuration from .intentrc.yaml in current directory
///
/// Returns `Ok(Config)` if file exists and is valid, or `Ok(default_config())` if not found.
/// Returns `Err(ConfigError)` if file exists but is invalid.
///
/// # Errors
/// Returns `ConfigError` if the configuration file exists but cannot be parsed or validated.
pub fn load_config() -> Result<Config, ConfigError> {
  let path = std::path::Path::new(CONFIG_FILE_PATH);

  if !path.exists() {
    return Ok(Config::default());
  }

  let contents = std::fs::read_to_string(path).map_err(|error| ConfigError::ReadError {
    path: path.display().to_string(),
    reason: error.to_string(),
  })?;

  parse_yaml_config(&contents)
}

/// Parse YAML configuration content
fn parse_yaml_config(yaml_content: &str) -> Result<Config, ConfigError> {
  let mut config = Config::default();

  for line in yaml_content.lines() {
    let trimmed = line.trim();

    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    if let Some((key, value)) = parse_yaml_key_value(trimmed) {
      match key.as_str() {
        "default_profile" => config.default_profile = value,
        "default_output_format" => config.default_output_format = value,
        "default_strategy" => config.default_strategy = value,
        "watch_debounce_ms" => {
          config.watch_debounce_ms = parse_u32_config_value("watch_debounce_ms", &value)?;
        }
        "max_cache_entries" => {
          config.max_cache_entries = parse_u32_config_value("max_cache_entries", &value)?;
        }
        _ => {}
      }
    }
  }

  validate_config(&config)?;
  Ok(config)
}

/// Parse a YAML key: value line
fn parse_yaml_key_value(line: &str) -> Option<(String, String)> {
  let parts: Vec<&str> = line.splitn(2, ':').collect();
  if parts.len() == 2 {
    let key = parts[0].trim().to_string();
    let value = parts[1].trim().trim_matches('"').to_string();
    Some((key, value))
  } else {
    None
  }
}

/// Validate configuration values
fn validate_config(config: &Config) -> Result<(), ConfigError> {
  let valid_profiles = ["api", "cli", "event", "data", "workflow", "ui"];
  if !valid_profiles.contains(&config.default_profile.as_str()) {
    return Err(ConfigError::InvalidProfile {
      input: config.default_profile.clone(),
      valid_options: valid_profiles.join(", "),
    });
  }

  let valid_formats = ["json", "text", "markdown"];
  if !valid_formats.contains(&config.default_output_format.as_str()) {
    return Err(ConfigError::InvalidOutputFormat {
      input: config.default_output_format.clone(),
      valid_options: valid_formats.join(", "),
    });
  }

  let valid_strategies = ["page_rank", "effort_ease", "dependency_order"];
  if !valid_strategies.contains(&config.default_strategy.as_str()) {
    return Err(ConfigError::InvalidStrategy {
      input: config.default_strategy.clone(),
      valid_options: valid_strategies.join(", "),
    });
  }

  if config.watch_debounce_ms > 60000 {
    return Err(ConfigError::InvalidWatchDebounce {
      value: config.watch_debounce_ms,
      max: 60000,
    });
  }

  if config.max_cache_entries > 10000 {
    return Err(ConfigError::InvalidMaxCacheEntries {
      value: config.max_cache_entries,
      max: 10000,
    });
  }

  Ok(())
}

fn parse_u32_config_value(field: &str, value: &str) -> Result<u32, ConfigError> {
  value
    .parse::<u32>()
    .map_err(|_| ConfigError::InvalidNumber {
      field: field.to_string(),
      value: value.to_string(),
    })
}

/// Get default profile from config, falling back to CLI argument
#[must_use]
pub fn get_profile(config: &Config, cli_arg: &str) -> String {
  if cli_arg.is_empty() {
    config.default_profile.clone()
  } else {
    cli_arg.to_string()
  }
}

/// Get default output format from config, falling back to CLI argument
#[must_use]
pub fn get_output_format(config: &Config, cli_arg: &str) -> String {
  if cli_arg.is_empty() {
    config.default_output_format.clone()
  } else {
    cli_arg.to_string()
  }
}

/// Get default strategy from config, falling back to CLI argument
#[must_use]
pub fn get_strategy(config: &Config, cli_arg: &str) -> String {
  if cli_arg.is_empty() {
    config.default_strategy.clone()
  } else {
    cli_arg.to_string()
  }
}

/// Check if config file exists
#[must_use]
pub fn config_file_exists() -> bool {
  std::path::Path::new(CONFIG_FILE_PATH).exists()
}

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
  fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.default_profile, "api");
    assert_eq!(config.default_output_format, "json");
    assert_eq!(config.default_strategy, "page_rank");
    assert_eq!(config.watch_debounce_ms, 500);
    assert_eq!(config.max_cache_entries, 50);
  }

  #[test]
  fn test_parse_yaml_key_value() {
    let (key, value) = parse_yaml_key_value("default_profile: cli").unwrap();
    assert_eq!(key, "default_profile");
    assert_eq!(value, "cli");
  }

  #[test]
  fn test_parse_yaml_key_value_quoted() {
    let (key, value) = parse_yaml_key_value("default_profile: \"cli\"").unwrap();
    assert_eq!(key, "default_profile");
    assert_eq!(value, "cli");
  }

  #[test]
  fn test_parse_yaml_key_value_invalid() {
    assert!(parse_yaml_key_value("no colon here").is_none());
  }

  #[test]
  fn test_parse_yaml_config_simple() {
    let yaml = r"
default_profile: cli
default_output_format: markdown
";
    let config = parse_yaml_config(yaml).unwrap();
    assert_eq!(config.default_profile, "cli");
    assert_eq!(config.default_output_format, "markdown");
  }

  #[test]
  fn test_parse_yaml_config_with_comments() {
    let yaml = r"
# This is a comment
default_profile: cli
";
    let config = parse_yaml_config(yaml).unwrap();
    assert_eq!(config.default_profile, "cli");
  }

  #[test]
  fn test_validate_config_valid() {
    let config = Config::default();
    assert!(validate_config(&config).is_ok());
  }

  #[test]
  fn test_validate_config_invalid_profile() {
    let config = Config {
      default_profile: "invalid".to_string(),
      ..Config::default()
    };
    assert!(matches!(
      validate_config(&config),
      Err(ConfigError::InvalidProfile { .. })
    ));
  }

  #[test]
  fn test_validate_config_invalid_format() {
    let config = Config {
      default_output_format: "xml".to_string(),
      ..Config::default()
    };
    assert!(matches!(
      validate_config(&config),
      Err(ConfigError::InvalidOutputFormat { .. })
    ));
  }

  #[test]
  fn test_validate_config_invalid_strategy() {
    let config = Config {
      default_strategy: "random".to_string(),
      ..Config::default()
    };
    assert!(matches!(
      validate_config(&config),
      Err(ConfigError::InvalidStrategy { .. })
    ));
  }

  #[test]
  fn test_validate_config_debounce_too_high() {
    let config = Config {
      watch_debounce_ms: 70000,
      ..Config::default()
    };
    assert!(matches!(
      validate_config(&config),
      Err(ConfigError::InvalidWatchDebounce { .. })
    ));
  }

  #[test]
  fn test_validate_config_cache_too_high() {
    let config = Config {
      max_cache_entries: 20000,
      ..Config::default()
    };
    assert!(matches!(
      validate_config(&config),
      Err(ConfigError::InvalidMaxCacheEntries { .. })
    ));
  }

  #[test]
  fn test_parse_yaml_config_invalid_number_is_typed_error() {
    let yaml = r"
watch_debounce_ms: nope
";

    assert!(matches!(
      parse_yaml_config(yaml),
      Err(ConfigError::InvalidNumber { field, value })
        if field == "watch_debounce_ms" && value == "nope"
    ));
  }

  #[test]
  fn test_get_profile_uses_cli_arg() {
    let config = Config::default();
    assert_eq!(get_profile(&config, "cli"), "cli");
  }

  #[test]
  fn test_get_profile_uses_config_when_empty() {
    let config = Config {
      default_profile: "workflow".to_string(),
      ..Config::default()
    };
    assert_eq!(get_profile(&config, ""), "workflow");
  }

  #[test]
  fn test_get_output_format_uses_cli_arg() {
    let config = Config::default();
    assert_eq!(get_output_format(&config, "markdown"), "markdown");
  }

  #[test]
  fn test_get_output_format_uses_config_when_empty() {
    let config = Config {
      default_output_format: "text".to_string(),
      ..Config::default()
    };
    assert_eq!(get_output_format(&config, ""), "text");
  }

  #[test]
  fn test_get_strategy_uses_cli_arg() {
    let config = Config::default();
    assert_eq!(
      get_strategy(&config, "dependency_order"),
      "dependency_order"
    );
  }

  #[test]
  fn test_get_strategy_uses_config_when_empty() {
    let config = Config {
      default_strategy: "effort_ease".to_string(),
      ..Config::default()
    };
    assert_eq!(get_strategy(&config, ""), "effort_ease");
  }

  #[test]
  fn test_config_file_exists_false() {
    // This test assumes no .intentrc.yaml in the test environment
    // The result depends on the actual file system state
    let _exists = config_file_exists();
  }
}
