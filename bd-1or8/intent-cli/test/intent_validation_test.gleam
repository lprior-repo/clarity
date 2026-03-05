import gleam/list
import gleeunit
import gleeunit/should
import intent/validation

pub fn main() -> Nil {
  gleeunit.main()
}

// ============================================================================
// PROFILE FLAG VALIDATION TESTS
// ============================================================================

pub fn profile_flag_valid_values_test() {
  let valid_profiles = ["api", "cli", "event", "data", "workflow", "ui"]

  list.each(valid_profiles, fn(profile) {
    let result = validation.validate_profile(profile)

    result
    |> should.equal(Ok(profile))
  })
}

pub fn profile_flag_invalid_value_test() {
  let invalid_profile = "invalid"

  let result = validation.validate_profile(invalid_profile)

  result
  |> should.equal(Error(
    "Invalid profile: 'invalid'. Valid options: api, cli, event, data, workflow, ui

Run 'intent interview --help' for usage.",
  ))
}

pub fn profile_flag_empty_string_test() {
  let empty_profile = ""

  let result = validation.validate_profile(empty_profile)

  result
  |> should.equal(Error(
    "--profile is required when not resuming

Run 'intent interview --help' for usage.",
  ))
}

// ============================================================================
// FORMAT FLAG VALIDATION TESTS
// ============================================================================

pub fn format_flag_valid_values_test() {
  let valid_formats = ["json", "jsonl", "markdown"]

  list.each(valid_formats, fn(format) {
    let result = validation.validate_format(format)

    result
    |> should.equal(Ok(format))
  })
}

pub fn format_flag_empty_string_defaults_test() {
  let empty_format = ""

  let result = validation.validate_format(empty_format)

  result
  |> should.equal(Ok("json"))
}

pub fn format_flag_invalid_value_test() {
  let invalid_format = "xml"

  let result = validation.validate_format(invalid_format)

  result
  |> should.equal(Error(
    "Invalid format: 'xml'. Valid options: json, jsonl, markdown

Run 'intent beads --help' for usage.",
  ))
}

// ============================================================================
// STRATEGY FLAG VALIDATION TESTS
// ============================================================================

pub fn strategy_flag_valid_values_test() {
  let valid_strategies = [
    "page_rank", "critical_path", "shortest", "risk_first",
  ]

  list.each(valid_strategies, fn(strategy) {
    let result = validation.validate_strategy(strategy)

    result
    |> should.equal(Ok(strategy))
  })
}

pub fn strategy_flag_empty_string_defaults_test() {
  let empty_strategy = ""

  let result = validation.validate_strategy(empty_strategy)

  result
  |> should.equal(Ok("page_rank"))
}

pub fn strategy_flag_invalid_value_test() {
  let invalid_strategy = "invalid"

  let result = validation.validate_strategy(invalid_strategy)

  result
  |> should.equal(Error(
    "Invalid strategy: 'invalid'. Valid options: page_rank, critical_path, shortest, risk_first

Run 'intent plan-next --help' for usage.",
  ))
}

// ============================================================================
// COMMAND ARGUMENT VALIDATION TESTS
// ============================================================================

pub fn no_args_validation_accepts_empty_list_test() {
  let result = validation.validate_no_args([], "test")

  result
  |> should.equal(Ok(Nil))
}

pub fn no_args_validation_rejects_arguments_test() {
  let result = validation.validate_no_args(["extra", "arguments"], "test")

  result
  |> should.equal(Error(
    "Error: test command takes no arguments

Run 'intent test --help' for usage.",
  ))
}

pub fn single_arg_validation_accepts_one_arg_test() {
  let result = validation.validate_single_arg(["plan-123"], "plan-approve")

  result
  |> should.equal(Ok("plan-123"))
}

pub fn single_arg_validation_rejects_empty_list_test() {
  let result = validation.validate_single_arg([], "plan-approve")

  result
  |> should.equal(Error(
    "Error: plan ID required

Run 'intent plan-approve --help' for usage.",
  ))
}

pub fn single_arg_validation_rejects_multiple_args_test() {
  let result = validation.validate_single_arg(["arg1", "arg2"], "plan-approve")

  result
  |> should.equal(Error(
    "Error: plan-approve command takes exactly one argument

Run 'intent plan-approve --help' for usage.",
  ))
}

pub fn single_arg_validation_rejects_empty_string_test() {
  let result = validation.validate_single_arg([""], "plan-approve")

  result
  |> should.equal(Error(
    "Error: plan ID cannot be empty

Run 'intent plan-approve --help' for usage.",
  ))
}

pub fn single_arg_validation_trims_whitespace_test() {
  let result = validation.validate_single_arg(["  plan-123  "], "plan-approve")

  result
  |> should.equal(Ok("plan-123"))
}

// ============================================================================
// REQUIRED FLAG VALIDATION TESTS
// ============================================================================

pub fn required_flag_accepts_non_empty_string_test() {
  let result = validation.validate_required_flag("session", "test-session")

  result
  |> should.equal(Ok("test-session"))
}

pub fn required_flag_rejects_empty_string_test() {
  let result = validation.validate_required_flag("session", "")

  result
  |> should.equal(Error(
    "Error: --session required

Run 'intent beads --help' for usage.",
  ))
}

pub fn required_flag_rejects_whitespace_only_test() {
  let result = validation.validate_required_flag("session", "   ")

  result
  |> should.equal(Error(
    "Error: --session required

Run 'intent beads --help' for usage.",
  ))
}
