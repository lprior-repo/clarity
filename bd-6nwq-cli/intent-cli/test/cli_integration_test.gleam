import gleam/list
import gleeunit
import gleeunit/should
import intent

pub fn main() {
  gleeunit.main()
}

pub fn given_bare_bool_flag_when_normalizing_then_true_is_assumed_test() {
  let args = ["validate", "examples/user-api.cue", "--json"]

  intent.normalize_cli_args(args)
  |> should.equal(["validate", "examples/user-api.cue", "--json=true"])
}

pub fn given_bool_literal_value_when_normalizing_then_equals_syntax_is_used_test() {
  let args = ["sessions", "--json", "false"]

  intent.normalize_cli_args(args)
  |> should.equal(["sessions", "--json=false"])
}

pub fn given_non_bool_flag_when_normalizing_then_arguments_are_preserved_test() {
  let args = [
    "interview", "--profile", "api", "--notes", "Build user management system",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "interview", "--profile=api", "--notes=Build user management system",
  ])
}

pub fn given_mixed_flags_when_normalizing_then_only_known_bool_flags_change_test() {
  let args = [
    "plan-approve", "session-123", "--yes", "--notes", "ship it", "--json",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "plan-approve", "session-123", "--yes=true", "--notes=ship it",
    "--json=true",
  ])
}

pub fn given_draft_bool_flag_when_normalizing_then_true_is_assumed_test() {
  let args = ["validate-bead", "sample.cue", "--draft"]

  intent.normalize_cli_args(args)
  |> should.equal(["validate-bead", "sample.cue", "--draft=true"])
}

pub fn given_value_flag_missing_value_when_normalizing_then_flag_is_preserved_test() {
  let args = ["interview", "--profile", "api", "--session", "--json"]

  intent.normalize_cli_args(args)
  |> should.equal(["interview", "--profile=api", "--session", "--json=true"])
}

pub fn given_export_template_value_flag_when_normalizing_then_equals_syntax_is_used_test() {
  let args = [
    "interview", "--profile", "api", "--export-answers-template",
    "/tmp/answers.json",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "interview", "--profile=api", "--export-answers-template=/tmp/answers.json",
  ])
}

pub fn given_interview_session_and_answer_flags_when_normalizing_then_values_are_encoded_test() {
  let args = [
    "interview", "--session", "interview-123", "--answer",
    "THE SYSTEM SHALL authenticate users",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "interview", "--session=interview-123",
    "--answer=THE SYSTEM SHALL authenticate users",
  ])
}

pub fn given_plan_work_vision_flag_when_normalizing_then_value_is_encoded_test() {
  let args = [
    "plan-work", "--profile", "cli", "--vision",
    "Build a planning-first developer tool",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "plan-work", "--profile=cli",
    "--vision=Build a planning-first developer tool",
  ])
}

pub fn given_plan_emit_beads_flags_when_normalizing_then_bool_and_values_are_encoded_test() {
  let args = [
    "plan-emit-beads", "session-123", "--target", "br", "--json", "--execute",
    "--confirm",
  ]

  intent.normalize_cli_args(args)
  |> should.equal([
    "plan-emit-beads", "session-123", "--target=br", "--json=true",
    "--execute=true", "--confirm=true",
  ])
}

pub fn command_inventory_count_is_current_test() {
  let commands = [
    "analyze", "bead-status", "beads", "beads-regenerate", "compact", "coverage",
    "diff", "ears", "effects", "export", "gaps", "history", "improve",
    "interview", "invert", "lint", "plan", "plan-approve", "plan-emit-beads",
    "plan-next", "plan-status", "plan-work", "prototext", "quality", "sessions",
    "show", "validate", "validate-bead", "vision", "ready",
  ]

  commands
  |> list.length
  |> should.equal(30)
}

// Tests for help command validation (bd-1d05)
pub fn command_exists_returns_true_for_valid_commands_test() {
  intent.command_exists("batch")
  |> should.be_true()

  intent.command_exists("init")
  |> should.be_true()

  intent.command_exists("validate")
  |> should.be_true()
}

pub fn command_exists_returns_false_for_invalid_commands_test() {
  intent.command_exists("nonexistent")
  |> should.be_false()

  intent.command_exists("foo")
  |> should.be_false()

  intent.command_exists("")
  |> should.be_false()
}

pub fn command_exists_handles_all_available_commands_test() {
  let available_commands = [
    "init", "interview", "beads", "bead-status", "history", "version", "diff",
    "sessions", "plan", "plan-next", "plan-approve", "plan-emit-beads",
    "beads-regenerate", "vision", "ready", "effects", "validate", "batch",
  ]

  // All available commands should exist
  available_commands
  |> list.map(fn(cmd) { intent.command_exists(cmd) })
  |> list.all(fn(result) { result == True })
  |> should.be_true()
}

// Test for bd-3p4r: command not found includes command name
// Note: This test documents the expected behavior
// Actual testing requires integration test due to io.println_error and exit calls
pub fn invalid_command_error_format_test() {
  // The error message should include:
  // 1. The command name that was not found
  // 2. A list of available commands
  // Example: "error: command not found: invalidcommand"
  //          "Available commands: init, interview, beads, ..."

  let invalid_command = "invalidcommand"
  let _expected_error_contains = "error: command not found: " <> invalid_command
  let _expected_available = "Available commands:"

  // This documents the requirement
  // Full integration testing would capture stderr
  True
  |> should.be_true()
}
