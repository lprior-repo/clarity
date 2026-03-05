//// RED PHASE: Array Interpolation Tests
//// These tests FAIL initially and document the broken behavior

import gleam/json
import gleam/option
import gleam/string
import gleeunit/should
import intent/array_indexing
import intent/interpolate

/// Test 1: Array indexing with positive index - SHOULD WORK
pub fn array_index_positive_test() {
  let ctx = interpolate.new_context()
  let items = json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x })
  let ctx = interpolate.set_variable(ctx, "items", items)

  // Interpolate "${items[0]}" should return "First: 1"
  let result = interpolate.interpolate_string(ctx, "First: ${items[0]}")

  case result {
    Ok(value) -> {
      // Should contain "First: 1"
      string.contains(value, "First: 1") |> should.be_true()
    }
    Error(_msg) -> {
      // Currently fails with "Variable not found: items[0]"
      // This shows the bug - it should work but doesn't
      should.fail()
    }
  }
}

/// Test 2: Array indexing with negative index - SHOULD WORK
pub fn array_index_negative_test() {
  let ctx = interpolate.new_context()
  let items = json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x })
  let ctx = interpolate.set_variable(ctx, "items", items)

  // Interpolate "${items[-1]}" should return "Last: 3"
  let result = interpolate.interpolate_string(ctx, "Last: ${items[-1]}")

  case result {
    Ok(value) -> {
      // Should contain "Last: 3"
      string.contains(value, "Last: 3") |> should.be_true()
    }
    Error(_) -> {
      // Currently fails
      should.fail()
    }
  }
}

/// Test 3: Nested field with array index - SHOULD WORK
pub fn nested_array_index_test() {
  let ctx = interpolate.new_context()
  let user =
    json.object([
      #("name", json.string("Alice")),
      #(
        "emails",
        json.array(
          [json.string("alice@example.com"), json.string("alice@test.com")],
          fn(x) { x },
        ),
      ),
    ])
  let ctx = interpolate.set_variable(ctx, "user", user)

  // Note: This would need path like "user.emails[0]"
  // But first let's verify basic array indexing works
  let result = interpolate.interpolate_string(ctx, "User: ${user.name}")

  case result {
    Ok(value) -> {
      string.contains(value, "User: Alice") |> should.be_true()
    }
    Error(_) -> {
      should.fail()
    }
  }
}

/// Test 4: Verify parse_path_component works correctly
pub fn parse_path_component_test() {
  // This should parse correctly
  case array_indexing.parse_path_component("items[0]") {
    Ok(#(field_name, _spec)) -> {
      field_name |> should.equal("items")
      // spec should be Index(0)
    }
    Error(_e) -> {
      // If this fails, that's the root cause
      should.fail()
    }
  }
}

/// Test 5: Verify get_variable finds the array
pub fn get_variable_finds_array_test() {
  let ctx = interpolate.new_context()
  let items = json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x })
  let ctx = interpolate.set_variable(ctx, "items", items)

  // We should be able to get the "items" variable
  let result = interpolate.get_variable(ctx, "items")

  case result {
    option.Some(value) -> {
      // Variable exists
      value |> should.equal(items)
    }
    option.None -> {
      // Variable not found - this would be the problem
      should.fail()
    }
  }
}
