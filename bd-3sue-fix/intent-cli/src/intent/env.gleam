/// Environment variable configuration for Intent CLI
///
/// This module provides functions to read environment variables that
/// configure Intent CLI behavior. Environment variables take precedence
/// over config file defaults but are overridden by CLI flags.
///
/// Supported environment variables:
/// - INTENT_DEFAULT_PROFILE: Default profile for interviews (api|cli|event|data|workflow|ui)
/// - INTENT_DEFAULT_FORMAT: Default format for bead output (json|jsonl|markdown)
/// - INTENT_DEFAULT_STRATEGY: Default strategy for plan-next (page_rank|critical_path|shortest|risk_first)
/// - INTENT_CONFIG_FILE: Path to configuration file
/// - INTENT_NO_COLOR: Disable colored output (true|false)
/// - INTENT_QUIET: Reduce output verbosity (true|false)

import gleam/string

/// Configuration type for all environment variables
pub type EnvConfig {
  EnvConfig(
    default_profile: String,
    default_format: String,
    default_strategy: String,
    config_file: String,
    no_color: Bool,
    quiet: Bool,
  )
}

/// Load all environment variables into a config record
pub fn load_env_config() -> EnvConfig {
  EnvConfig(
    default_profile: get_env_default("INTENT_DEFAULT_PROFILE", ""),
    default_format: get_env_default("INTENT_DEFAULT_FORMAT", "json"),
    default_strategy: get_env_default("INTENT_DEFAULT_STRATEGY", "page_rank"),
    config_file: get_env_default("INTENT_CONFIG_FILE", ""),
    no_color: get_env_bool("INTENT_NO_COLOR", False),
    quiet: get_env_bool("INTENT_QUIET", False),
  )
}

/// Get a string environment variable with a default value
pub fn get_env_default(key: String, default: String) -> String {
  case get_env(key) {
    Ok(value) -> {
      case string.trim(value) {
        "" -> default
        val -> val
      }
    }
    Error(_) -> default
  }
}

/// Get a boolean environment variable with a default value
/// Accepts: "true", "1", "yes" (case-insensitive) as true
pub fn get_env_bool(key: String, default: Bool) -> Bool {
  case get_env(key) {
    Ok(value) -> parse_bool(value, default)
    Error(_) -> default
  }
}

/// Parse a string as a boolean
fn parse_bool(value: String, default: Bool) -> Bool {
  let normalized = string.lowercase(string.trim(value))
  case normalized {
    "true" | "1" | "yes" | "on" -> True
    "false" | "0" | "no" | "off" -> False
    _ -> default
  }
}

/// Check if environment variables should be loaded
/// Returns False if INTENT_NO_CONFIG is set to true
pub fn should_load_config() -> Bool {
  case get_env("INTENT_NO_CONFIG") {
    Ok(value) -> {
      let normalized = string.lowercase(string.trim(value))
      case normalized {
        "true" | "1" | "yes" | "on" -> False
        _ -> True
      }
    }
    Error(_) -> True
  }
}

/// Get the profile from environment or default
pub fn get_default_profile() -> Result(String, Nil) {
  let profile = get_env_default("INTENT_DEFAULT_PROFILE", "")
  case profile {
    "" -> Error(Nil)
    val -> Ok(val)
  }
}

/// Get the format from environment or default
pub fn get_default_format() -> String {
  get_env_default("INTENT_DEFAULT_FORMAT", "json")
}

/// Get the strategy from environment or default
pub fn get_default_strategy() -> String {
  get_env_default("INTENT_DEFAULT_STRATEGY", "page_rank")
}

/// Check if colored output should be disabled
pub fn is_no_color() -> Bool {
  // Check INTENT_NO_COLOR or standard NO_COLOR environment variable
  let intent_no_color = get_env_bool("INTENT_NO_COLOR", False)
  let no_color = get_env_bool("NO_COLOR", False)

  intent_no_color || no_color
}

/// Check if quiet mode is enabled
pub fn is_quiet() -> Bool {
  get_env_bool("INTENT_QUIET", False)
}

/// Get config file path from environment
pub fn get_config_file() -> Result(String, Nil) {
  let path = get_env_default("INTENT_CONFIG_FILE", "")
  case path {
    "" -> Error(Nil)
    val -> Ok(val)
  }
}

@external(erlang, "intent_ffi", "get_env")
fn ffi_get_env(key: String) -> Result(String, Nil)

/// Get an environment variable value
/// Returns Ok(value) if set, Error(Nil) if not set
fn get_env(key: String) -> Result(String, Nil) {
  ffi_get_env(key)
}
