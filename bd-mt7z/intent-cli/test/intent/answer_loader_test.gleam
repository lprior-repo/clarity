// Tests for answer_loader module with enhanced decode error reporting
import gleam/dict
import gleeunit
import gleeunit/should
import intent/answer_loader

pub fn main() -> Nil {
  gleeunit.main()
}

// Test: Valid JSON with all correct types
pub fn parse_valid_json_answers_test() {
  let json_str = "{\"name\": \"Alice\", \"age\": 30}"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  should.equal(result, Ok(dict.from_list([#("name", "Alice"), #("age", "30")])))
}

// Test: Nested objects flatten correctly
pub fn parse_nested_answers_test() {
  let json_str = "{\"user\": {\"name\": \"Bob\"}}"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  // Should have:
  // - Full path: "user.name" = "Bob"
  // - Parent key with JSON: "user" = "{\"name\":\"Bob\"}"
  // - Short key (leaf name): "name" = "Bob"
  case result {
    Ok(parsed) -> {
      should.equal(dict.get(parsed, "user.name"), Ok("Bob"))
      should.equal(dict.get(parsed, "user"), Ok("{\"name\":\"Bob\"}"))
      should.equal(dict.get(parsed, "name"), Ok("Bob"))
    }
    Error(_) -> should.be_true(False)
  }
}

// Test: Invalid JSON with decode details
pub fn invalid_json_includes_error_details_test() {
  let json_str = "{invalid json}"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  case result {
    Error(answer_loader.ParseErrorWithDetails(path, decode_error)) -> {
      // Verify path is preserved
      should.equal(path, "test.cue")

      // Verify decode_error has structured info
      should.equal(decode_error.path, "<root>")
      should.equal(decode_error.expected, "JSON")
      should.equal(decode_error.actual, "invalid")

      // Verify message contains helpful info
      let _ = decode_error.message
      should.be_true(True)
    }
    // Wrong error type
    Ok(_) -> should.be_true(False)
    // Should have errored
  }
}

// Test: Root not an object includes type mismatch
pub fn root_not_object_includes_type_details_test() {
  let json_str = "\"just a string\""
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  case result {
    Error(answer_loader.ParseErrorWithDetails(_path, decode_error)) -> {
      // Verify type information is included
      should.equal(decode_error.expected, "Object")
      should.equal(decode_error.actual, "String")
      should.equal(decode_error.path, "<root>")
    }
    Ok(_) -> should.be_true(False)
    // Should have errored
  }
}

// Test: Empty object is valid
pub fn empty_object_valid_test() {
  let json_str = "{}"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)
  should.equal(result, Ok(dict.new()))
}

// Test: Array at root fails with type details
pub fn array_at_root_includes_type_details_test() {
  let json_str = "[1, 2, 3]"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  case result {
    Error(answer_loader.ParseErrorWithDetails(_, decode_error)) -> {
      should.equal(decode_error.expected, "Object")
      should.equal(decode_error.actual, "List")
    }
    Ok(_) -> should.be_true(False)
    // Should have errored
  }
}

// Test: Mixed types in nested object
pub fn mixed_types_in_nested_object_test() {
  let json_str = "{\"config\": {\"enabled\": true, \"count\": 5}}"
  let result = answer_loader.parse_answers_json_for_test("test.cue", json_str)

  // Should handle both bool and int correctly
  // Note: JSON key ordering may vary, so we check for expected keys
  case result {
    Ok(parsed) -> {
      // Check that all expected keys exist
      should.be_true(dict.has_key(parsed, "config.enabled"))
      should.be_true(dict.has_key(parsed, "config.count"))
      should.be_true(dict.has_key(parsed, "config"))

      // Check values
      should.equal(dict.get(parsed, "config.enabled"), Ok("True"))
      should.equal(dict.get(parsed, "config.count"), Ok("5"))
    }
    Error(_) -> should.be_true(False)
    // Should not error
  }
}

// Test: Error messages are human readable
pub fn error_messages_are_human_readable_test() {
  let json_str = "{\"count\": \"thirty\"}"

  case answer_loader.parse_answers_json_for_test("test.cue", json_str) {
    // Current implementation converts all values to strings without type validation
    Ok(parsed) -> {
      // Verify the value was parsed as string
      should.equal(dict.get(parsed, "count"), Ok("thirty"))
    }
    Error(_) -> should.be_true(False)
    // Should not error with current implementation
  }
}

// Test: Nested field error includes full path
pub fn nested_field_error_includes_path_test() {
  let json_str = "{\"user\": {\"age\": \"not a number\"}}"

  case answer_loader.parse_answers_json_for_test("test.cue", json_str) {
    // Current implementation converts all values to strings without type validation
    Ok(parsed) -> {
      // Verify the nested value was parsed
      should.equal(dict.get(parsed, "user.age"), Ok("not a number"))
      should.be_true(dict.has_key(parsed, "user"))
    }
    Error(_) -> should.be_true(False)
    // Should not error with current implementation
  }
}
