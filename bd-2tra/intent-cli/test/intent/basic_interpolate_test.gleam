import gleam/json
import gleam/option
import gleeunit/should
import intent/interpolate

pub fn new_context_test() {
  let ctx = interpolate.new_context()

  // Should have empty variables dict
  interpolate.get_variable(ctx, "anything") |> should.equal(option.None)
}

pub fn set_variable_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "user_id", json.int(123))

  case interpolate.get_variable(ctx, "user_id") {
    option.Some(value) -> {
      json.to_string(value) |> should.equal("123")
    }
    option.None -> should.fail()
  }
}

pub fn interpolate_string_simple_variable_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "name", json.string("Bob"))

  let result = interpolate.interpolate_string(ctx, "Hello ${name}")

  result |> should.equal(Ok("Hello Bob"))
}

pub fn interpolate_string_missing_variable_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.interpolate_string(ctx, "Value: ${missing}")

  case result {
    Error(msg) -> {
      msg |> should.equal("Variable not found: missing")
    }
    Ok(_) -> should.fail()
  }
}
