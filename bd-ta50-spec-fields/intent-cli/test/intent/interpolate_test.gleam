import gleam/dict
import gleam/json
import gleam/option
import gleam/string
import gleeunit/should
import intent/interpolate

// =============================================================================
// Context Creation and Management Tests
// =============================================================================

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

pub fn set_multiple_variables_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "id", json.int(1))
  let ctx = interpolate.set_variable(ctx, "name", json.string("Alice"))
  let ctx = interpolate.set_variable(ctx, "active", json.bool(True))

  interpolate.get_variable(ctx, "id") |> should.not_equal(option.None)
  interpolate.get_variable(ctx, "name") |> should.not_equal(option.None)
  interpolate.get_variable(ctx, "active") |> should.not_equal(option.None)
}

pub fn get_variable_not_found_test() {
  let ctx = interpolate.new_context()

  interpolate.get_variable(ctx, "missing") |> should.equal(option.None)
}

pub fn set_request_body_test() {
  let ctx = interpolate.new_context()
  let body = json.object([#("id", json.int(42))])
  let ctx = interpolate.set_request_body(ctx, body)

  // Request body should be accessible via interpolation
  let result = interpolate.extract_capture(ctx, "request.body.id")
  case result {
    Ok(value) -> json.to_string(value) |> should.equal("42")
    Error(_) -> should.fail()
  }
}

pub fn set_response_body_test() {
  let ctx = interpolate.new_context()
  let body = json.object([#("status", json.string("ok"))])
  let ctx = interpolate.set_response_body(ctx, body)

  // Response body should be accessible via interpolation
  let result = interpolate.extract_capture(ctx, "response.body.status")
  case result {
    Ok(value) -> interpolate.json_to_string(value) |> should.equal("ok")
    Error(_) -> should.fail()
  }
}

// =============================================================================
// Basic String Interpolation Tests
// =============================================================================

pub fn interpolate_string_no_variables_test() {
  let ctx = interpolate.new_context()
  let result = interpolate.interpolate_string(ctx, "plain text")

  result |> should.equal(Ok("plain text"))
}

pub fn interpolate_string_simple_variable_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "name", json.string("Bob"))

  let result = interpolate.interpolate_string(ctx, "Hello ${name}")

  result |> should.equal(Ok("Hello Bob"))
}

pub fn interpolate_string_multiple_variables_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "first", json.string("Alice"))
  let ctx = interpolate.set_variable(ctx, "last", json.string("Smith"))

  let result = interpolate.interpolate_string(ctx, "${first} ${last}")

  result |> should.equal(Ok("Alice Smith"))
}

pub fn interpolate_string_variable_at_start_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "prefix", json.string("User"))

  let result = interpolate.interpolate_string(ctx, "${prefix}: data")

  result |> should.equal(Ok("User: data"))
}

pub fn interpolate_string_variable_at_end_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "suffix", json.string("end"))

  let result = interpolate.interpolate_string(ctx, "The ${suffix}")

  result |> should.equal(Ok("The end"))
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

// =============================================================================
// Different Value Type Tests
// =============================================================================

pub fn interpolate_string_integer_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "count", json.int(42))

  let result = interpolate.interpolate_string(ctx, "Count: ${count}")

  result |> should.equal(Ok("Count: 42"))
}

pub fn interpolate_string_boolean_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "active", json.bool(True))

  let result = interpolate.interpolate_string(ctx, "Active: ${active}")

  result |> should.equal(Ok("Active: true"))
}

pub fn interpolate_string_null_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "value", json.null())

  let result = interpolate.interpolate_string(ctx, "Value: ${value}")

  result |> should.equal(Ok("Value: null"))
}

pub fn interpolate_string_object_test() {
  let ctx = interpolate.new_context()
  let obj = json.object([#("id", json.int(1))])
  let ctx = interpolate.set_variable(ctx, "user", obj)

  let result = interpolate.interpolate_string(ctx, "User: ${user}")

  case result {
    Ok(str) -> {
      // Should get JSON representation of object
      string.contains(str, "{") |> should.be_true()
      string.contains(str, "id") |> should.be_true()
    }
    Error(_) -> should.fail()
  }
}

pub fn interpolate_string_array_test() {
  let ctx = interpolate.new_context()
  let arr = json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x })
  let ctx = interpolate.set_variable(ctx, "items", arr)

  let result = interpolate.interpolate_string(ctx, "Items: ${items}")

  case result {
    Ok(str) -> {
      // Should get JSON representation of array
      string.contains(str, "[") |> should.be_true()
    }
    Error(_) -> should.fail()
  }
}

// =============================================================================
// Nested Path Resolution Tests
// =============================================================================

pub fn interpolate_string_nested_field_test() {
  let ctx = interpolate.new_context()
  let user = json.object([#("name", json.string("Alice"))])
  let ctx = interpolate.set_variable(ctx, "user", user)

  let result = interpolate.interpolate_string(ctx, "Name: ${user.name}")

  result |> should.equal(Ok("Name: Alice"))
}

pub fn interpolate_string_deeply_nested_test() {
  let ctx = interpolate.new_context()
  let data =
    json.object([
      #(
        "user",
        json.object([
          #("profile", json.object([#("email", json.string("a@b.com"))])),
        ]),
      ),
    ])
  let ctx = interpolate.set_variable(ctx, "data", data)

  let result =
    interpolate.interpolate_string(ctx, "Email: ${data.user.profile.email}")

  result |> should.equal(Ok("Email: a@b.com"))
}

pub fn interpolate_string_request_body_test() {
  let ctx = interpolate.new_context()
  let body = json.object([#("token", json.string("abc123"))])
  let ctx = interpolate.set_request_body(ctx, body)

  let result =
    interpolate.interpolate_string(ctx, "Token: ${request.body.token}")

  result |> should.equal(Ok("Token: abc123"))
}

pub fn interpolate_string_response_body_test() {
  let ctx = interpolate.new_context()
  let body = json.object([#("id", json.int(999))])
  let ctx = interpolate.set_response_body(ctx, body)

  let result = interpolate.interpolate_string(ctx, "ID: ${response.body.id}")

  result |> should.equal(Ok("ID: 999"))
}

pub fn interpolate_string_missing_nested_field_test() {
  let ctx = interpolate.new_context()
  let user = json.object([#("name", json.string("Bob"))])
  let ctx = interpolate.set_variable(ctx, "user", user)

  let result = interpolate.interpolate_string(ctx, "Age: ${user.age}")

  case result {
    Error(msg) -> {
      // Should indicate field not found
      string.contains(msg, "not found") |> should.be_true()
    }
    Ok(_) -> should.fail()
  }
}

// =============================================================================
// Header Interpolation Tests
// =============================================================================

pub fn interpolate_headers_no_variables_test() {
  let ctx = interpolate.new_context()
  let headers =
    dict.from_list([
      #("Content-Type", "application/json"),
      #("Accept", "application/json"),
    ])

  let result = interpolate.interpolate_headers(ctx, headers)

  case result {
    Ok(new_headers) -> {
      dict.get(new_headers, "Content-Type")
      |> should.equal(Ok("application/json"))
    }
    Error(_) -> should.fail()
  }
}

pub fn interpolate_headers_with_variables_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "token", json.string("secret123"))

  let headers = dict.from_list([#("Authorization", "Bearer ${token}")])

  let result = interpolate.interpolate_headers(ctx, headers)

  case result {
    Ok(new_headers) -> {
      dict.get(new_headers, "Authorization")
      |> should.equal(Ok("Bearer secret123"))
    }
    Error(_) -> should.fail()
  }
}

pub fn interpolate_headers_multiple_values_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "api_key", json.string("key123"))
  let ctx = interpolate.set_variable(ctx, "version", json.string("v1"))

  let headers =
    dict.from_list([
      #("X-API-Key", "${api_key}"),
      #("X-API-Version", "${version}"),
    ])

  let result = interpolate.interpolate_headers(ctx, headers)

  case result {
    Ok(new_headers) -> {
      dict.get(new_headers, "X-API-Key") |> should.equal(Ok("key123"))
      dict.get(new_headers, "X-API-Version") |> should.equal(Ok("v1"))
    }
    Error(_) -> should.fail()
  }
}

pub fn interpolate_headers_missing_variable_test() {
  let ctx = interpolate.new_context()
  let headers = dict.from_list([#("X-Token", "${missing}")])

  let result = interpolate.interpolate_headers(ctx, headers)

  case result {
    Error(msg) -> {
      msg |> should.equal("Variable not found: missing")
    }
    Ok(_) -> should.fail()
  }
}

pub fn interpolate_headers_empty_dict_test() {
  let ctx = interpolate.new_context()
  let headers = dict.new()

  let result = interpolate.interpolate_headers(ctx, headers)

  case result {
    Ok(new_headers) -> {
      dict.size(new_headers) |> should.equal(0)
    }
    Error(_) -> should.fail()
  }
}

// =============================================================================
// Extract Capture Tests
// =============================================================================

pub fn extract_capture_simple_variable_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "user_id", json.int(42))

  let result = interpolate.extract_capture(ctx, "user_id")

  case result {
    Ok(value) -> json.to_string(value) |> should.equal("42")
    Error(_) -> should.fail()
  }
}

pub fn extract_capture_nested_path_test() {
  let ctx = interpolate.new_context()
  let user = json.object([#("id", json.int(123))])
  let ctx = interpolate.set_variable(ctx, "user", user)

  let result = interpolate.extract_capture(ctx, "user.id")

  case result {
    Ok(value) -> json.to_string(value) |> should.equal("123")
    Error(_) -> should.fail()
  }
}

pub fn extract_capture_response_body_test() {
  let ctx = interpolate.new_context()
  let body = json.object([#("token", json.string("xyz789"))])
  let ctx = interpolate.set_response_body(ctx, body)

  let result = interpolate.extract_capture(ctx, "response.body.token")

  case result {
    Ok(value) -> interpolate.json_to_string(value) |> should.equal("xyz789")
    Error(_) -> should.fail()
  }
}

pub fn extract_capture_missing_variable_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.extract_capture(ctx, "nonexistent")

  case result {
    Error(msg) -> msg |> should.equal("Variable not found: nonexistent")
    Ok(_) -> should.fail()
  }
}

pub fn extract_capture_no_request_body_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.extract_capture(ctx, "request.body.field")

  case result {
    Error(msg) -> msg |> should.equal("No request body in context")
    Ok(_) -> should.fail()
  }
}

pub fn extract_capture_no_response_body_test() {
  let ctx = interpolate.new_context()

  let result = interpolate.extract_capture(ctx, "response.body.field")

  case result {
    Error(msg) -> msg |> should.equal("No response body in context")
    Ok(_) -> should.fail()
  }
}

// =============================================================================
// JSON to String Conversion Tests
// =============================================================================

pub fn json_to_string_string_value_test() {
  let value = json.string("hello")

  let result = interpolate.json_to_string(value)

  result |> should.equal("hello")
}

pub fn json_to_string_integer_test() {
  let value = json.int(42)

  let result = interpolate.json_to_string(value)

  result |> should.equal("42")
}

pub fn json_to_string_boolean_test() {
  let value = json.bool(True)

  let result = interpolate.json_to_string(value)

  result |> should.equal("true")
}

pub fn json_to_string_null_test() {
  let value = json.null()

  let result = interpolate.json_to_string(value)

  result |> should.equal("null")
}

pub fn json_to_string_object_test() {
  let value = json.object([#("key", json.string("value"))])

  let result = interpolate.json_to_string(value)

  // Should return JSON representation
  string.contains(result, "{") |> should.be_true()
  string.contains(result, "key") |> should.be_true()
}

pub fn json_to_string_array_test() {
  let value = json.array([json.int(1), json.int(2)], fn(x) { x })

  let result = interpolate.json_to_string(value)

  // Should return JSON representation
  string.contains(result, "[") |> should.be_true()
}

// =============================================================================
// Edge Cases and Complex Scenarios
// =============================================================================

pub fn interpolate_string_same_variable_twice_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "word", json.string("echo"))

  let result = interpolate.interpolate_string(ctx, "${word} ${word}")

  result |> should.equal(Ok("echo echo"))
}

pub fn interpolate_string_adjacent_variables_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "first", json.string("hello"))
  let ctx = interpolate.set_variable(ctx, "second", json.string("world"))

  let result = interpolate.interpolate_string(ctx, "${first}${second}")

  result |> should.equal(Ok("helloworld"))
}

pub fn interpolate_string_with_special_chars_test() {
  let ctx = interpolate.new_context()
  let ctx =
    interpolate.set_variable(ctx, "email", json.string("test@example.com"))

  let result = interpolate.interpolate_string(ctx, "Email: ${email}")

  result |> should.equal(Ok("Email: test@example.com"))
}

pub fn interpolate_string_unicode_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "emoji", json.string("😀"))

  let result = interpolate.interpolate_string(ctx, "Face: ${emoji}")

  result |> should.equal(Ok("Face: 😀"))
}

pub fn interpolate_string_empty_string_variable_test() {
  let ctx = interpolate.new_context()
  let ctx = interpolate.set_variable(ctx, "empty", json.string(""))

  let result = interpolate.interpolate_string(ctx, "Value: ${empty}.")

  result |> should.equal(Ok("Value: ."))
}
