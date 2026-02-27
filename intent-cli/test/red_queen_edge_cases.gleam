import gleam/int
import gleam/io
import gleam/json
import gleam/list
import gleam/string
import intent/case_insensitive
import intent/formats
import intent/interpolate

pub fn main() {
  io.println("=== RED-07 Edge Case Testing ===")
  io.println("")

  // Test 1: interpolate.gleam - empty variable path
  let ctx = interpolate.new_context()
  let result = interpolate.interpolate_string(ctx, "${}")
  io.println("Test 1: Empty variable path: " <> format_result(result))

  // Test 2: interpolate.gleam - array index 0
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable(
      "items",
      json.array([json.string("first"), json.string("second")], fn(x) { x }),
    )
  let result = interpolate.interpolate_string(ctx, "${items[0]}")
  io.println("Test 2: Array index 0: " <> format_result(result))

  // Test 3: interpolate.gleam - array index -1
  let result = interpolate.interpolate_string(ctx, "${items[-1]}")
  io.println("Test 3: Array index -1: " <> format_result(result))

  // Test 4: interpolate.gleam - empty array
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("empty", json.array([], fn(x) { x }))
  let result = interpolate.interpolate_string(ctx, "${empty[0]}")
  io.println("Test 4: Empty array access: " <> format_result(result))

  // Test 5: interpolate.gleam - large string (10K)
  let large_str = list.repeat("x", 10_000) |> string.join("")
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("large", json.string(large_str))
  let result = interpolate.interpolate_string(ctx, "${large}")
  let test5_len = case result {
    Ok(s) -> string.length(s)
    Error(_) -> 0
  }
  io.println(
    "Test 5: Large string 10K: "
    <> case test5_len {
      10_000 -> "OK (length: 10000)"
      n ->
        case n == 0 {
          True -> "ERROR"
          False -> "OK but wrong length: " <> int.to_string(n)
        }
    },
  )

  // Test 6: case_insensitive.gleam - empty strings
  let result = case_insensitive.contains_ignore_case("", "")
  io.println("Test 6: Empty string contains: " <> bool_to_string(result))

  // Test 7: case_insensitive.gleam - Unicode case folding
  let german_s = "strasse"
  let german_upper = "STRASSE"
  let result = case_insensitive.equals_ignore_case(german_s, german_upper)
  io.println("Test 7: Case folding: " <> bool_to_string(result))

  // Test 8: formats.gleam - empty email
  let result = formats.validate_email("")
  io.println("Test 8: Empty email: " <> format_result3(result))

  // Test 9: formats.gleam - leap year Feb 29
  let result = formats.validate_iso8601("2024-02-29")
  io.println("Test 9: Leap year Feb 29: " <> format_result3(result))

  // Test 10: formats.gleam - non-leap year Feb 29
  let result = formats.validate_iso8601("2023-02-29")
  io.println("Test 10: Non-leap year Feb 29: " <> format_result3(result))

  // Test 11: formats.gleam - UUID with invalid version
  let result = formats.validate_uuid("550e8400-e29b-01d4-a716-446655440000")
  io.println("Test 11: UUID version 0: " <> format_result3(result))

  // Test 12: formats.gleam - email with consecutive dots
  let result = formats.validate_email("user..name@example.com")
  io.println("Test 12: Email consecutive dots: " <> format_result3(result))

  // Test 13: interpolate.gleam - null value
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("nothing", json.null())
  let result = interpolate.interpolate_string(ctx, "${nothing}")
  io.println("Test 13: Null value: " <> format_result(result))

  // Test 14: interpolate.gleam - boolean value
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("flag", json.bool(True))
  let result = interpolate.interpolate_string(ctx, "${flag}")
  io.println("Test 14: Boolean value: " <> format_result(result))

  // Test 15: formats.gleam - min date (0000-01-01)
  let result = formats.validate_iso8601("0000-01-01")
  io.println("Test 15: Min date (0000-01-01): " <> format_result3(result))

  // Test 16: formats.gleam - max date (9999-12-31)
  let result = formats.validate_iso8601("9999-12-31")
  io.println("Test 16: Max date (9999-12-31): " <> format_result3(result))

  // Test 17: formats.gleam - UUID max values
  let result = formats.validate_uuid("FFFFFFFF-FFFF-5FFF-BFFF-FFFFFFFFFFFF")
  io.println("Test 17: UUID max values: " <> format_result3(result))

  // Test 18: interpolate.gleam - special chars
  let ctx =
    interpolate.new_context()
    |> interpolate.set_variable("special", json.string("a\u{200B}b"))
  let result = interpolate.interpolate_string(ctx, "${special}")
  let test18_len = case result {
    Ok(s) -> string.length(s)
    Error(_) -> 0
  }
  io.println(
    "Test 18: Zero-width chars: "
    <> case test18_len {
      3 -> "OK (length: 3)"
      n ->
        case n == 0 {
          True -> "ERROR"
          False -> "OK but wrong length: " <> int.to_string(n)
        }
    },
  )

  // Test 19: formats.gleam - hour 24 (should fail)
  let result = formats.validate_iso8601("2024-01-01T24:00:00")
  io.println("Test 19: Hour 24 (invalid): " <> format_result3(result))

  // Test 20: formats.gleam - minute 60 (should fail)
  let result = formats.validate_iso8601("2024-01-01T12:60:00")
  io.println("Test 20: Minute 60 (invalid): " <> format_result3(result))

  io.println("")
  io.println("=== Testing Complete ===")
}

fn format_result(result: Result(String, String)) -> String {
  case result {
    Ok(s) -> "OK: " <> s
    Error(e) -> "ERROR: " <> e
  }
}

fn format_result3(result: Result(Nil, String)) -> String {
  case result {
    Ok(_) -> "OK"
    Error(e) -> "ERROR: " <> e
  }
}

fn bool_to_string(b: Bool) -> String {
  case b {
    True -> "TRUE"
    False -> "FALSE"
  }
}
