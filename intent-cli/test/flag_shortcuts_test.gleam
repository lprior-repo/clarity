import gleam/list
import gleeunit
import gleeunit/should
import shellout

pub fn main() {
  gleeunit.main()
}

/// Test: validate command help includes flag shortcuts documentation
pub fn validate_help_includes_shortcuts_test() {
  let result =
    shellout.command("gleam", ["run", "--", "validate", "--help"], ".", [])

  result
  |> should.be_ok()

  let assert Ok(output) = result
  output
  |> string.contains("Flag shortcuts:")
  |> should.be_true()
}

/// Test: batch command help includes flag shortcuts documentation
pub fn batch_help_includes_shortcuts_test() {
  let result =
    shellout.command("gleam", ["run", "--", "batch", "--help"], ".", [])

  result
  |> should.be_ok()

  let assert Ok(output) = result
  output
  |> string.contains("Flag shortcuts:")
  |> should.be_true()
}

/// Test: history command help includes flag shortcuts documentation
pub fn history_help_includes_shortcuts_test() {
  let result =
    shellout.command("gleam", ["run", "--", "history", "--help"], ".", [])

  result
  |> should.be_ok()

  let assert Ok(output) = result
  output
  |> string.contains("Flag shortcuts:")
  |> should.be_true()
}

/// Test: sessions command help includes flag shortcuts documentation
pub fn sessions_help_includes_shortcuts_test() {
  let result =
    shellout.command("gleam", ["run", "--", "sessions", "--help"], ".", [])

  result
  |> should.be_ok()

  let assert Ok(output) = result
  output
  |> string.contains("Flag shortcuts:")
  |> should.be_true()
}

/// Test: All commands document -j for --json
pub fn commands_document_json_shortcut_test() {
  let commands = [["validate", "--help"], ["batch", "--help"]]

  let results =
    commands
    |> list.map(fn(cmd) {
      shellout.command("gleam", ["run", "--", ..cmd], ".", [])
    })

  results
  |> list.each(fn(result) {
    let assert Ok(output) = result
    output
    |> string.contains("-j, --json")
    |> should.be_true()
  })
}

// Import string module
import gleam/string
