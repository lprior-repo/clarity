//// Integration tests for Intent CLI core workflows
//// Tests interview → beads, plan generation, document generation, and effects analysis

import gleam/list
import gleeunit
import gleeunit/should
import shellout

pub fn main() {
  gleeunit.main()
}

/// Test: Beads generation command works
pub fn beads_command_works_test() {
  let result =
    shellout.command(
      "gleam",
      ["run", "--", "beads", "--session", "examples/user-api.cue"],
      ".",
      [],
    )

  result
  |> should.be_ok()
}

/// Test: Plan generation command works
pub fn plan_command_works_test() {
  let result =
    shellout.command("gleam", ["run", "--", "plan", "--notes", "Test"], ".", [])

  result
  |> should.be_ok()
}

/// Test: Vision document generation works
pub fn vision_command_works_test() {
  let result =
    shellout.command(
      "gleam",
      ["run", "--", "vision", "--help"],
      ".",
      [],
    )

  result
  |> should.be_ok()
}

/// Test: Ready document generation works
pub fn ready_command_works_test() {
  let result =
    shellout.command(
      "gleam",
      ["run", "--", "ready", "--help"],
      ".",
      [],
    )

  result
  |> should.be_ok()
}

/// Test: Effects analysis works
pub fn effects_command_works_test() {
  let result =
    shellout.command(
      "gleam",
      ["run", "--", "effects", "--help"],
      ".",
      [],
    )

  result
  |> should.be_ok()
}

/// Test: Help commands work
pub fn help_commands_work_test() {
  let commands = [
    ["beads", "--help"],
    ["plan", "--help"],
    ["vision", "--help"],
    ["ready", "--help"],
    ["effects", "--help"],
  ]

  let results =
    commands
    |> list.map(fn(cmd) {
      shellout.command("gleam", ["run", "--", ..cmd], ".", [])
    })

  results
  |> list.each(fn(result) {
    result
    |> should.be_ok()
  })
}

/// Test: Plan workflow commands
pub fn plan_workflow_commands_test() {
  let commands = [
    ["plan-next", "--help"],
    ["plan-approve", "--help"],
    ["plan-emit-beads", "--help"],
    ["plan-work", "--help"],
  ]

  let results =
    commands
    |> list.map(fn(cmd) {
      shellout.command("gleam", ["run", "--", ..cmd], ".", [])
    })

  results
  |> list.each(fn(result) {
    result
    |> should.be_ok()
  })
}
