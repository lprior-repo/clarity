import gleam/string

/// ============================================================================
/// VALIDATION TYPES
/// ============================================================================
/// Valid profile types for interview
pub type Profile {
  ApiProfile
  CliProfile
  EventProfile
  DataProfile
  WorkflowProfile
  UiProfile
}

/// Valid output formats for beads
pub type Format {
  JsonFormat
  JsonlFormat
  MarkdownFormat
}

/// Valid selection strategies for plan-next
pub type Strategy {
  PageRank
  CriticalPath
  Shortest
  RiskFirst
}

/// ============================================================================
/// PROFILE VALIDATION
/// ============================================================================
/// Validate profile flag value
/// Returns Ok(profile) if valid, Error(message) otherwise
pub fn validate_profile(profile: String) -> Result(String, String) {
  case profile {
    "" ->
      Error(
        "--profile is required when not resuming\n\nRun 'intent interview --help' for usage.",
      )
    "api" -> Ok("api")
    "cli" -> Ok("cli")
    "event" -> Ok("event")
    "data" -> Ok("data")
    "workflow" -> Ok("workflow")
    "ui" -> Ok("ui")
    invalid ->
      Error(
        "Invalid profile: '"
        <> invalid
        <> "'. Valid options: api, cli, event, data, workflow, ui\n\nRun 'intent interview --help' for usage.",
      )
  }
}

/// ============================================================================
/// FORMAT VALIDATION
/// ============================================================================
/// Validate format flag value
/// Returns Ok(format) if valid, Error(message) otherwise
/// Empty string defaults to "json"
pub fn validate_format(format: String) -> Result(String, String) {
  case format {
    "" -> Ok("json")
    // Default value
    "json" -> Ok("json")
    "jsonl" -> Ok("jsonl")
    "markdown" -> Ok("markdown")
    invalid ->
      Error(
        "Invalid format: '"
        <> invalid
        <> "'. Valid options: json, jsonl, markdown\n\nRun 'intent beads --help' for usage.",
      )
  }
}

/// ============================================================================
/// STRATEGY VALIDATION
/// ============================================================================
/// Validate strategy flag value
/// Returns Ok(strategy) if valid, Error(message) otherwise
/// Empty string defaults to "page_rank"
pub fn validate_strategy(strategy: String) -> Result(String, String) {
  case strategy {
    "" -> Ok("page_rank")
    // Default value
    "page_rank" -> Ok("page_rank")
    "critical_path" -> Ok("critical_path")
    "shortest" -> Ok("shortest")
    "risk_first" -> Ok("risk_first")
    invalid ->
      Error(
        "Invalid strategy: '"
        <> invalid
        <> "'. Valid options: page_rank, critical_path, shortest, risk_first\n\nRun 'intent plan-next --help' for usage.",
      )
  }
}

/// ============================================================================
/// COMMAND ARGUMENT VALIDATION
/// ============================================================================
/// Validate that a command takes no arguments
/// Returns Ok(Nil) if args is empty, Error(message) otherwise
pub fn validate_no_args(
  args: List(String),
  command_name: String,
) -> Result(Nil, String) {
  case args {
    [] -> Ok(Nil)
    _ ->
      Error(
        "Error: "
        <> command_name
        <> " command takes no arguments\n\nRun 'intent "
        <> command_name
        <> " --help' for usage.",
      )
  }
}

/// Validate that a command takes exactly one argument
/// Returns Ok(arg) if valid, Error(message) otherwise
pub fn validate_single_arg(
  args: List(String),
  command_name: String,
) -> Result(String, String) {
  case args {
    [] ->
      Error(
        "Error: "
        <> get_arg_name(command_name)
        <> " required\n\nRun 'intent "
        <> command_name
        <> " --help' for usage.",
      )
    [arg] -> {
      let trimmed = string.trim(arg)
      case trimmed {
        "" ->
          Error(
            "Error: "
            <> get_arg_name(command_name)
            <> " cannot be empty\n\nRun 'intent "
            <> command_name
            <> " --help' for usage.",
          )
        _ -> Ok(trimmed)
      }
    }
    _ ->
      Error(
        "Error: "
        <> command_name
        <> " command takes exactly one argument\n\nRun 'intent "
        <> command_name
        <> " --help' for usage.",
      )
  }
}

/// Get the argument name for a command
fn get_arg_name(command_name: String) -> String {
  case command_name {
    "plan-approve" -> "plan ID"
    _ -> "argument"
  }
}

/// ============================================================================
/// REQUIRED FLAG VALIDATION
/// ============================================================================
/// Validate that a required flag is not empty
/// Returns Ok(value) if valid, Error(message) otherwise
pub fn validate_required_flag(
  flag_name: String,
  value: String,
) -> Result(String, String) {
  let trimmed = string.trim(value)
  case trimmed {
    "" ->
      Error(
        "Error: --"
        <> flag_name
        <> " required\n\nRun 'intent "
        <> get_command_for_flag(flag_name)
        <> " --help' for usage.",
      )
    _ -> Ok(trimmed)
  }
}

/// Get the command name for a flag
fn get_command_for_flag(flag_name: String) -> String {
  case flag_name {
    "session" -> "beads"
    "bead-id" -> "bead-status"
    _ -> "intent"
  }
}
