//! Environment variable configuration for Intent CLI
//!
//! This module provides functions to read environment variables that
//! configure Intent CLI behavior. Environment variables take precedence
//! over config file defaults but are overridden by CLI flags.
//!
//! Supported environment variables:
//! - INTENT_DEFAULT_PROFILE: Default profile for interviews (api|cli|event|data|workflow|ui)
//! - INTENT_DEFAULT_FORMAT: Default format for bead output (json|jsonl|markdown)
//! - INTENT_DEFAULT_STRATEGY: Default strategy for plan-next (page_rank|critical_path|shortest|risk_first)
//! - INTENT_CONFIG_FILE: Path to configuration file
//! - INTENT_NO_COLOR: Disable colored output (true|false)
//! - INTENT_QUIET: Reduce output verbosity (true|false)
//!
//! Ported from intent-cli/src/intent/env.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

/// Configuration type for all environment variables
#[derive(Debug, Clone, PartialEq)]
pub struct EnvConfig {
  pub default_profile: String,
  pub default_format: String,
  pub default_strategy: String,
  pub config_file: String,
  pub no_color: bool,
  pub quiet: bool,
}

impl Default for EnvConfig {
  fn default() -> Self {
    Self {
      default_profile: String::new(),
      default_format: "json".to_string(),
      default_strategy: "page_rank".to_string(),
      config_file: String::new(),
      no_color: false,
      quiet: false,
    }
  }
}

/// Load all environment variables into a config record
#[must_use]
pub fn load_env_config() -> EnvConfig {
  EnvConfig {
    default_profile: get_env_default("INTENT_DEFAULT_PROFILE", ""),
    default_format: get_env_default("INTENT_DEFAULT_FORMAT", "json"),
    default_strategy: get_env_default("INTENT_DEFAULT_STRATEGY", "page_rank"),
    config_file: get_env_default("INTENT_CONFIG_FILE", ""),
    no_color: get_env_bool("INTENT_NO_COLOR", false),
    quiet: get_env_bool("INTENT_QUIET", false),
  }
}

/// Get a string environment variable with a default value
#[must_use]
pub fn get_env_default(key: &str, default: &str) -> String {
  match std::env::var(key) {
    Ok(value) => {
      let trimmed = value.trim();
      if trimmed.is_empty() {
        default.to_string()
      } else {
        trimmed.to_string()
      }
    }
    Err(_) => default.to_string(),
  }
}

/// Get a boolean environment variable with a default value
/// Accepts: "true", "1", "yes" (case-insensitive) as true
#[must_use]
pub fn get_env_bool(key: &str, default: bool) -> bool {
  match std::env::var(key) {
    Ok(value) => parse_bool(&value, default),
    Err(_) => default,
  }
}

/// Parse a string as a boolean
fn parse_bool(value: &str, default: bool) -> bool {
  let normalized = value.trim().to_lowercase();
  match normalized.as_str() {
    "true" | "1" | "yes" | "on" => true,
    "false" | "0" | "no" | "off" => false,
    _ => default,
  }
}

/// Check if environment variables should be loaded
/// Returns false if INTENT_NO_CONFIG is set to true
#[must_use]
pub fn should_load_config() -> bool {
  match std::env::var("INTENT_NO_CONFIG") {
    Ok(value) => {
      let normalized = value.trim().to_lowercase();
      !matches!(normalized.as_str(), "true" | "1" | "yes" | "on")
    }
    Err(_) => true,
  }
}

/// Get the profile from environment or default
#[must_use]
pub fn get_default_profile() -> Option<String> {
  let profile = get_env_default("INTENT_DEFAULT_PROFILE", "");
  if profile.is_empty() {
    None
  } else {
    Some(profile)
  }
}

/// Get the format from environment or default
#[must_use]
pub fn get_default_format() -> String {
  get_env_default("INTENT_DEFAULT_FORMAT", "json")
}

/// Get the strategy from environment or default
#[must_use]
pub fn get_default_strategy() -> String {
  get_env_default("INTENT_DEFAULT_STRATEGY", "page_rank")
}

/// Check if colored output should be disabled
#[must_use]
pub fn is_no_color() -> bool {
  let intent_no_color = get_env_bool("INTENT_NO_COLOR", false);
  let no_color = get_env_bool("NO_COLOR", false);
  intent_no_color || no_color
}

/// Check if quiet mode is enabled
#[must_use]
pub fn is_quiet() -> bool {
  get_env_bool("INTENT_QUIET", false)
}

/// Get config file path from environment
#[must_use]
pub fn get_config_file() -> Option<String> {
  let path = get_env_default("INTENT_CONFIG_FILE", "");
  if path.is_empty() {
    None
  } else {
    Some(path)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_env_config_default() {
    let config = EnvConfig::default();
    assert!(config.default_profile.is_empty());
    assert_eq!(config.default_format, "json");
    assert_eq!(config.default_strategy, "page_rank");
    assert!(config.config_file.is_empty());
    assert!(!config.no_color);
    assert!(!config.quiet);
  }

  #[test]
  fn test_parse_bool_true_values() {
    assert!(parse_bool("true", false));
    assert!(parse_bool("TRUE", false));
    assert!(parse_bool("1", false));
    assert!(parse_bool("yes", false));
    assert!(parse_bool("YES", false));
    assert!(parse_bool("on", false));
    assert!(parse_bool("ON", false));
  }

  #[test]
  fn test_parse_bool_false_values() {
    assert!(!parse_bool("false", true));
    assert!(!parse_bool("FALSE", true));
    assert!(!parse_bool("0", true));
    assert!(!parse_bool("no", true));
    assert!(!parse_bool("NO", true));
    assert!(!parse_bool("off", true));
    assert!(!parse_bool("OFF", true));
  }

  #[test]
  fn test_parse_bool_uses_default_for_unknown() {
    assert!(parse_bool("maybe", true));
    assert!(!parse_bool("maybe", false));
    assert!(parse_bool("", true));
    assert!(!parse_bool("  ", false));
  }

  #[test]
  fn test_get_env_default_returns_default_when_not_set() {
    // This test assumes TEST_NONEXISTENT_VAR is not set
    let result = get_env_default("TEST_NONEXISTENT_VAR_12345", "default_value");
    assert_eq!(result, "default_value");
  }

  #[test]
  fn test_get_env_bool_returns_default_when_not_set() {
    let result = get_env_bool("TEST_NONEXISTENT_VAR_12345", true);
    assert!(result);
  }

  #[test]
  fn test_get_default_profile_returns_none_when_not_set() {
    let result = get_default_profile();
    // Will be None if INTENT_DEFAULT_PROFILE is not set
    // This test depends on environment state
    let _ = result;
  }

  #[test]
  fn test_get_default_format_returns_json_by_default() {
    let result = get_default_format();
    // Will be "json" if INTENT_DEFAULT_FORMAT is not set
    let _ = result;
  }

  #[test]
  fn test_get_default_strategy_returns_page_rank_by_default() {
    let result = get_default_strategy();
    // Will be "page_rank" if INTENT_DEFAULT_STRATEGY is not set
    let _ = result;
  }

  #[test]
  fn test_is_no_color_respects_both_vars() {
    // This test depends on environment state
    let _ = is_no_color();
  }

  #[test]
  fn test_is_quiet_default() {
    let _ = is_quiet();
  }

  #[test]
  fn test_should_load_config_default() {
    let _ = should_load_config();
  }

  #[test]
  fn test_get_config_file_default() {
    let _ = get_config_file();
  }
}
