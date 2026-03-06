//! CLI argument validation module for Intent CLI
//!
//! This module provides validation functions for CLI arguments including:
//! - Profile validation (api, cli, event, data, workflow, ui)
//! - Format validation (json, jsonl, markdown)
//! - Strategy validation (`page_rank`, `critical_path`, `shortest`, `risk_first`)
//! - Argument count validation
//! - Required flag validation
//!
//! All functions return `Result<T, String>` for pure, panic-free error handling.
//! Empty strings are handled according to each function's contract:
//! - Profile: Empty is an error (no default)
//! - Format: Empty returns `Ok("json")` as default
//! - Strategy: Empty returns `Ok("page_rank")` as default

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

/// Valid profile options for CLI validation
const VALID_PROFILES: [&str; 6] = ["api", "cli", "event", "data", "workflow", "ui"];

/// Valid format options for CLI validation
const VALID_FORMATS: [&str; 3] = ["json", "jsonl", "markdown"];

/// Valid strategy options for CLI validation
const VALID_STRATEGIES: [&str; 4] = ["page_rank", "critical_path", "shortest", "risk_first"];

/// Validate a profile string
///
/// # Errors
/// - Returns `Err` if the profile is empty
/// - Returns `Err` if the profile is not one of: api, cli, event, data, workflow, ui
///
/// # Normalization
/// Profile is normalized to lowercase before validation.
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_profile;
/// assert_eq!(validate_profile("API"), Ok("api".to_string()));
/// assert_eq!(validate_profile(""), Err("Profile is required. Valid options: api, cli, event, data, workflow, ui".to_string()));
/// ```
pub fn validate_profile(profile: &str) -> Result<String, String> {
  let normalized = profile.trim().to_lowercase();

  if normalized.is_empty() {
    return Err(format!(
      "Profile is required. Valid options: {}",
      VALID_PROFILES.join(", ")
    ));
  }

  if VALID_PROFILES.contains(&normalized.as_str()) {
    Ok(normalized)
  } else {
    Err(format!(
      "Invalid profile '{}'. Valid options: {}",
      profile,
      VALID_PROFILES.join(", ")
    ))
  }
}

/// Validate a format string
///
/// # Errors
/// Returns `Err` if the format is not one of: json, jsonl, markdown
///
/// # Default
/// Empty string returns `Ok("json")` as the default format.
///
/// # Normalization
/// Format is normalized to lowercase before validation.
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_format;
/// assert_eq!(validate_format(""), Ok("json".to_string()));
/// assert_eq!(validate_format("MARKDOWN"), Ok("markdown".to_string()));
/// assert_eq!(validate_format("xml"), Err("Invalid format 'xml'. Valid options: json, jsonl, markdown".to_string()));
/// ```
pub fn validate_format(format: &str) -> Result<String, String> {
  let normalized = format.trim().to_lowercase();

  if normalized.is_empty() {
    return Ok("json".to_string());
  }

  if VALID_FORMATS.contains(&normalized.as_str()) {
    Ok(normalized)
  } else {
    Err(format!(
      "Invalid format '{}'. Valid options: {}",
      format,
      VALID_FORMATS.join(", ")
    ))
  }
}

/// Validate a strategy string
///
/// # Errors
/// Returns `Err` if the strategy is not one of: `page_rank`, `critical_path`, `shortest`, `risk_first`
///
/// # Default
/// Empty string returns `Ok("page_rank")` as the default strategy.
///
/// # Normalization
/// Strategy is normalized to lowercase before validation.
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_strategy;
/// assert_eq!(validate_strategy(""), Ok("page_rank".to_string()));
/// assert_eq!(validate_strategy("CRITICAL_PATH"), Ok("critical_path".to_string()));
/// assert_eq!(validate_strategy("random"), Err("Invalid strategy 'random'. Valid options: page_rank, critical_path, shortest, risk_first".to_string()));
/// ```
pub fn validate_strategy(strategy: &str) -> Result<String, String> {
  let normalized = strategy.trim().to_lowercase();

  if normalized.is_empty() {
    return Ok("page_rank".to_string());
  }

  if VALID_STRATEGIES.contains(&normalized.as_str()) {
    Ok(normalized)
  } else {
    Err(format!(
      "Invalid strategy '{}'. Valid options: {}",
      strategy,
      VALID_STRATEGIES.join(", ")
    ))
  }
}

/// Validate that no arguments are provided
///
/// # Errors
/// Returns `Err` if the args slice is not empty, including the command name and count.
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_no_args;
/// assert_eq!(validate_no_args(&[], "help"), Ok(()));
/// assert!(validate_no_args(&["extra".to_string()], "help").is_err());
/// ```
pub fn validate_no_args(args: &[String], command_name: &str) -> Result<(), String> {
  if args.is_empty() {
    Ok(())
  } else {
    Err(format!(
      "Command '{}' does not accept arguments, but received {} argument(s)",
      command_name,
      args.len()
    ))
  }
}

/// Validate that exactly one argument is provided
///
/// # Errors
/// - Returns `Err` if args is empty
/// - Returns `Err` if args contains more than one element
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_single_arg;
/// assert_eq!(validate_single_arg(&["bead-id".to_string()], "get"), Ok("bead-id".to_string()));
/// assert!(validate_single_arg(&[], "get").is_err());
/// assert!(validate_single_arg(&["a".to_string(), "b".to_string()], "get").is_err());
/// ```
pub fn validate_single_arg(args: &[String], command_name: &str) -> Result<String, String> {
  match args.len() {
    0 => Err(format!(
      "Command '{command_name}' requires exactly 1 argument, but received 0"
    )),
    1 => Ok(args[0].clone()),
    n => Err(format!(
      "Command '{command_name}' requires exactly 1 argument, but received {n}"
    )),
  }
}

/// Validate that a required flag has a non-empty, non-whitespace value
///
/// # Errors
/// Returns `Err` if the value is empty or contains only whitespace.
///
/// # Return Value
/// Returns `Ok(trimmed_value)` with leading/trailing whitespace removed.
///
/// # Examples
/// ```
/// use clarity_web::intent::cli::validation::validate_required_flag;
/// assert_eq!(validate_required_flag("name", "  value  "), Ok("value".to_string()));
/// assert!(validate_required_flag("name", "").is_err());
/// assert!(validate_required_flag("name", "   ").is_err());
/// ```
pub fn validate_required_flag(flag_name: &str, value: &str) -> Result<String, String> {
  let trimmed = value.trim();

  if trimmed.is_empty() {
    Err(format!("Required flag '--{flag_name}' is missing or empty"))
  } else {
    Ok(trimmed.to_string())
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {

  use super::*;

  // ============================================================
  // validate_profile tests
  // ============================================================

  #[test]
  fn test_validate_profile_valid_lowercase() {
    let profiles = vec!["api", "cli", "event", "data", "workflow", "ui"];
    for profile in profiles {
      let result = validate_profile(profile);
      assert!(result.is_ok(), "Profile '{}' should be valid", profile);
      assert_eq!(result.unwrap(), profile);
    }
  }

  #[test]
  fn test_validate_profile_valid_uppercase() {
    let result = validate_profile("API");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "api");
  }

  #[test]
  fn test_validate_profile_valid_mixed_case() {
    let result = validate_profile("Cli");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cli");
  }

  #[test]
  fn test_validate_profile_trims_whitespace() {
    let result = validate_profile("  api  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "api");
  }

  #[test]
  fn test_validate_profile_empty_string() {
    let result = validate_profile("");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Profile is required"));
    assert!(error.contains("api, cli, event, data, workflow, ui"));
  }

  #[test]
  fn test_validate_profile_whitespace_only() {
    let result = validate_profile("   ");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Profile is required"));
  }

  #[test]
  fn test_validate_profile_invalid_value() {
    let result = validate_profile("invalid");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid profile"));
    assert!(error.contains("invalid"));
    assert!(error.contains("api, cli, event, data, workflow, ui"));
  }

  // ============================================================
  // validate_format tests
  // ============================================================

  #[test]
  fn test_validate_format_valid_lowercase() {
    let formats = vec!["json", "jsonl", "markdown"];
    for format in formats {
      let result = validate_format(format);
      assert!(result.is_ok(), "Format '{}' should be valid", format);
      assert_eq!(result.unwrap(), format);
    }
  }

  #[test]
  fn test_validate_format_valid_uppercase() {
    let result = validate_format("MARKDOWN");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "markdown");
  }

  #[test]
  fn test_validate_format_valid_mixed_case() {
    let result = validate_format("JsOnL");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "jsonl");
  }

  #[test]
  fn test_validate_format_trims_whitespace() {
    let result = validate_format("  json  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "json");
  }

  #[test]
  fn test_validate_format_empty_string_returns_default() {
    let result = validate_format("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "json");
  }

  #[test]
  fn test_validate_format_whitespace_only_returns_default() {
    let result = validate_format("   ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "json");
  }

  #[test]
  fn test_validate_format_invalid_value() {
    let result = validate_format("xml");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid format"));
    assert!(error.contains("xml"));
    assert!(error.contains("json, jsonl, markdown"));
  }

  // ============================================================
  // validate_strategy tests
  // ============================================================

  #[test]
  fn test_validate_strategy_valid_lowercase() {
    let strategies = vec!["page_rank", "critical_path", "shortest", "risk_first"];
    for strategy in strategies {
      let result = validate_strategy(strategy);
      assert!(result.is_ok(), "Strategy '{}' should be valid", strategy);
      assert_eq!(result.unwrap(), strategy);
    }
  }

  #[test]
  fn test_validate_strategy_valid_uppercase() {
    let result = validate_strategy("CRITICAL_PATH");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "critical_path");
  }

  #[test]
  fn test_validate_strategy_valid_mixed_case() {
    let result = validate_strategy("Risk_First");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "risk_first");
  }

  #[test]
  fn test_validate_strategy_trims_whitespace() {
    let result = validate_strategy("  shortest  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "shortest");
  }

  #[test]
  fn test_validate_strategy_empty_string_returns_default() {
    let result = validate_strategy("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "page_rank");
  }

  #[test]
  fn test_validate_strategy_whitespace_only_returns_default() {
    let result = validate_strategy("   ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "page_rank");
  }

  #[test]
  fn test_validate_strategy_invalid_value() {
    let result = validate_strategy("random");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid strategy"));
    assert!(error.contains("random"));
    assert!(error.contains("page_rank, critical_path, shortest, risk_first"));
  }

  // ============================================================
  // validate_no_args tests
  // ============================================================

  #[test]
  fn test_validate_no_args_empty() {
    let result = validate_no_args(&[], "help");
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_no_args_single_arg() {
    let args = vec!["extra".to_string()];
    let result = validate_no_args(&args, "help");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("help"));
    assert!(error.contains("does not accept arguments"));
    assert!(error.contains("1 argument"));
  }

  #[test]
  fn test_validate_no_args_multiple_args() {
    let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = validate_no_args(&args, "version");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("version"));
    assert!(error.contains("3 argument"));
  }

  // ============================================================
  // validate_single_arg tests
  // ============================================================

  #[test]
  fn test_validate_single_arg_exactly_one() {
    let args = vec!["bead-id".to_string()];
    let result = validate_single_arg(&args, "get");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "bead-id");
  }

  #[test]
  fn test_validate_single_arg_empty() {
    let result = validate_single_arg(&[], "get");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("get"));
    assert!(error.contains("requires exactly 1 argument"));
    assert!(error.contains("received 0"));
  }

  #[test]
  fn test_validate_single_arg_multiple() {
    let args = vec!["a".to_string(), "b".to_string()];
    let result = validate_single_arg(&args, "get");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("get"));
    assert!(error.contains("requires exactly 1 argument"));
    assert!(error.contains("received 2"));
  }

  #[test]
  fn test_validate_single_arg_many() {
    let args = vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string(),
      "d".to_string(),
    ];
    let result = validate_single_arg(&args, "edit");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("received 4"));
  }

  // ============================================================
  // validate_required_flag tests
  // ============================================================

  #[test]
  fn test_validate_required_flag_valid_value() {
    let result = validate_required_flag("name", "value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "value");
  }

  #[test]
  fn test_validate_required_flag_trims_whitespace() {
    let result = validate_required_flag("name", "  trimmed-value  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "trimmed-value");
  }

  #[test]
  fn test_validate_required_flag_empty_string() {
    let result = validate_required_flag("name", "");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("--name"));
    assert!(error.contains("missing or empty"));
  }

  #[test]
  fn test_validate_required_flag_whitespace_only() {
    let result = validate_required_flag("config", "   ");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("--config"));
    assert!(error.contains("missing or empty"));
  }

  #[test]
  fn test_validate_required_flag_preserves_internal_whitespace() {
    let result = validate_required_flag("title", "  hello world  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello world");
  }

  // ============================================================
  // Edge case and integration tests
  // ============================================================

  #[test]
  fn test_all_valid_profiles_are_accepted() {
    // Verify all documented profiles work
    for profile in &["api", "cli", "event", "data", "workflow", "ui"] {
      let result = validate_profile(profile);
      assert!(result.is_ok(), "Profile {} should be valid", profile);
    }
  }

  #[test]
  fn test_all_valid_formats_are_accepted() {
    for format in &["json", "jsonl", "markdown"] {
      let result = validate_format(format);
      assert!(result.is_ok(), "Format {} should be valid", format);
    }
  }

  #[test]
  fn test_all_valid_strategies_are_accepted() {
    for strategy in &["page_rank", "critical_path", "shortest", "risk_first"] {
      let result = validate_strategy(strategy);
      assert!(result.is_ok(), "Strategy {} should be valid", strategy);
    }
  }

  #[test]
  fn test_error_messages_are_descriptive() {
    // Profile error should list valid options
    let profile_error = validate_profile("invalid").unwrap_err();
    assert!(profile_error.contains("Invalid profile"));
    assert!(profile_error.contains("api"));

    // Format error should list valid options
    let format_error = validate_format("xml").unwrap_err();
    assert!(format_error.contains("Invalid format"));
    assert!(format_error.contains("json"));

    // Strategy error should list valid options
    let strategy_error = validate_strategy("random").unwrap_err();
    assert!(strategy_error.contains("Invalid strategy"));
    assert!(strategy_error.contains("page_rank"));

    // Argument count errors should include counts
    let no_args_error = validate_no_args(&["extra".to_string()], "help").unwrap_err();
    assert!(no_args_error.contains("1 argument"));

    let single_arg_error = validate_single_arg(&[], "get").unwrap_err();
    assert!(single_arg_error.contains("0"));
  }
}
