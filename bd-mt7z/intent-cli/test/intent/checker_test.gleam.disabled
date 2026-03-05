import gleam/dict
import gleam/json
import gleam/list
import gleeunit/should
import intent/checker
import intent/http_client
import intent/interpolate
import intent/types

// =============================================================================
// Status Code Checking Tests
// =============================================================================

pub fn check_response_status_match_test() {
  let expected = make_response(200, dict.new(), dict.new())
  let actual = make_execution_result(200, json.object([]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  result.status_ok |> should.be_true()
  result.status_expected |> should.equal(200)
  result.status_actual |> should.equal(200)
}

pub fn check_response_status_mismatch_test() {
  let expected = make_response(200, dict.new(), dict.new())
  let actual = make_execution_result(404, json.object([]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  result.status_ok |> should.be_false()
  result.status_expected |> should.equal(200)
  result.status_actual |> should.equal(404)
}

pub fn check_response_various_status_codes_test() {
  let test_cases = [
    #(200, 200, True),
    #(201, 201, True),
    #(204, 204, True),
    #(400, 400, True),
    #(404, 404, True),
    #(500, 500, True),
    #(200, 404, False),
    #(201, 200, False),
  ]

  list.each(test_cases, fn(test_case) {
    let #(expected_status, actual_status, should_pass) = test_case
    let expected = make_response(expected_status, dict.new(), dict.new())
    let actual = make_execution_result(actual_status, json.object([]))
    let ctx = make_context()

    let result = checker.check_response(expected, actual, ctx)

    result.status_ok |> should.equal(should_pass)
  })
}

// =============================================================================
// Field Rule Checking Tests
// =============================================================================

pub fn check_response_no_checks_test() {
  let expected = make_response(200, dict.new(), dict.new())
  let actual =
    make_execution_result(200, json.object([#("data", json.string("test"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_single_passing_check_test() {
  let checks =
    dict.from_list([
      #("name", types.Check(rule: "equals John", why: "Name should be John")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("name", json.string("John"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_single_failing_check_test() {
  let checks =
    dict.from_list([
      #("name", types.Check(rule: "equals John", why: "Name should be John")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("name", json.string("Jane"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn check_response_multiple_checks_all_pass_test() {
  let checks =
    dict.from_list([
      #("name", types.Check(rule: "equals John", why: "Check name")),
      #("age", types.Check(rule: "equals 30", why: "Check age")),
      #(
        "email",
        types.Check(rule: "string containing @", why: "Check email format"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("name", json.string("John")),
        #("age", json.int(30)),
        #("email", json.string("john@example.com")),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(3)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_multiple_checks_some_fail_test() {
  let checks =
    dict.from_list([
      #("name", types.Check(rule: "equals John", why: "Check name")),
      #("age", types.Check(rule: "equals 25", why: "Check age")),
      #(
        "email",
        types.Check(rule: "string containing @", why: "Check email format"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("name", json.string("John")),
        #("age", json.int(30)),
        // Wrong age
        #("email", json.string("invalid-email")),
        // No @
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(2)
}

// =============================================================================
// Header Checking Tests
// =============================================================================

pub fn check_response_header_exact_match_test() {
  let expected_headers = dict.from_list([#("content-type", "application/json")])
  let expected = make_response(200, dict.new(), expected_headers)

  let actual_headers = dict.from_list([#("content-type", "application/json")])
  let actual =
    make_execution_result_with_headers(200, json.object([]), actual_headers)
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_header_mismatch_test() {
  let expected_headers = dict.from_list([#("content-type", "application/json")])
  let expected = make_response(200, dict.new(), expected_headers)

  let actual_headers = dict.from_list([#("content-type", "text/html")])
  let actual =
    make_execution_result_with_headers(200, json.object([]), actual_headers)
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn check_response_multiple_headers_test() {
  let expected_headers =
    dict.from_list([
      #("content-type", "application/json"),
      #("x-api-version", "v1"),
      #("cache-control", "no-cache"),
    ])
  let expected = make_response(200, dict.new(), expected_headers)

  let actual_headers =
    dict.from_list([
      #("content-type", "application/json"),
      #("x-api-version", "v1"),
      #("cache-control", "no-cache"),
    ])
  let actual =
    make_execution_result_with_headers(200, json.object([]), actual_headers)
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(3)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_missing_header_test() {
  let expected_headers = dict.from_list([#("x-required-header", "value")])
  let expected = make_response(200, dict.new(), expected_headers)

  let actual_headers = dict.new()
  let actual =
    make_execution_result_with_headers(200, json.object([]), actual_headers)
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// Combined Status, Headers, and Body Tests
// =============================================================================

pub fn check_response_all_aspects_pass_test() {
  let checks =
    dict.from_list([
      #("success", types.Check(rule: "equals true", why: "Check success")),
    ])
  let expected_headers = dict.from_list([#("content-type", "application/json")])
  let expected = make_response(200, checks, expected_headers)

  let actual_headers = dict.from_list([#("content-type", "application/json")])
  let actual =
    make_execution_result_with_headers(
      200,
      json.object([#("success", json.bool(True))]),
      actual_headers,
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  result.status_ok |> should.be_true()
  list.length(result.passed) |> should.equal(2)
  // 1 body check + 1 header
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_all_aspects_fail_test() {
  let checks =
    dict.from_list([
      #("success", types.Check(rule: "equals true", why: "Check success")),
    ])
  let expected_headers = dict.from_list([#("content-type", "application/json")])
  let expected = make_response(200, checks, expected_headers)

  let actual_headers = dict.from_list([#("content-type", "text/html")])
  let actual =
    make_execution_result_with_headers(
      500,
      json.object([#("success", json.bool(False))]),
      actual_headers,
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  result.status_ok |> should.be_false()
  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(2)
  // 1 body check + 1 header
}

// =============================================================================
// Nested JSON Path Tests
// =============================================================================

pub fn check_response_nested_field_access_test() {
  let checks =
    dict.from_list([
      #(
        "user.name",
        types.Check(rule: "equals Alice", why: "Check nested name"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("user", json.object([#("name", json.string("Alice"))]))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn check_response_deeply_nested_field_test() {
  let checks =
    dict.from_list([
      #(
        "data.user.profile.email",
        types.Check(rule: "string containing @", why: "Check email"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "data",
          json.object([
            #(
              "user",
              json.object([
                #(
                  "profile",
                  json.object([#("email", json.string("test@example.com"))]),
                ),
              ]),
            ),
          ]),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

// =============================================================================
// Array Access Tests
// =============================================================================
// NOTE: Array indexing (items[0], items[-1]) is not yet implemented in the
// checker module. These tests are removed until that feature is added.
// See src/intent/checker/json.gleam - navigate_json_path() only handles
// dot notation, not bracket notation.

// =============================================================================
// Different Rule Types Tests
// =============================================================================

pub fn check_response_equals_rule_test() {
  let checks =
    dict.from_list([
      #("status", types.Check(rule: "equals active", why: "Check status")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("status", json.string("active"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

pub fn check_response_contains_rule_test() {
  let checks =
    dict.from_list([
      #(
        "message",
        types.Check(rule: "string containing success", why: "Check message"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("message", json.string("Operation completed successfully")),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

pub fn check_response_exists_rule_test() {
  let checks =
    dict.from_list([
      #("id", types.Check(rule: "present", why: "ID must be present")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("id", json.int(123))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

pub fn check_response_absent_rule_test() {
  let checks =
    dict.from_list([
      #("error", types.Check(rule: "absent", why: "Should not have error")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("success", json.bool(True))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

pub fn check_response_missing_field_test() {
  let checks =
    dict.from_list([
      #("nonexistent", types.Check(rule: "equals value", why: "Check field")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.failed) |> should.equal(1)
}

pub fn check_response_null_value_test() {
  let checks =
    dict.from_list([
      #("value", types.Check(rule: "equals null", why: "Should be null")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.null())]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

pub fn check_response_empty_array_test() {
  let checks =
    dict.from_list([
      #("items", types.Check(rule: "equals []", why: "Should be empty array")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("items", json.array([], fn(x) { x }))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

pub fn check_response_empty_object_test() {
  let checks =
    dict.from_list([
      #("data", types.Check(rule: "equals {}", why: "Should be empty object")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("data", json.object([]))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
}

// =============================================================================
// Helper Functions
// =============================================================================

fn make_response(
  status: Int,
  checks: dict.Dict(String, types.Check),
  headers: dict.Dict(String, String),
) -> types.Response {
  types.Response(
    status: status,
    example: json.null(),
    checks: checks,
    headers: headers,
  )
}

fn make_execution_result(
  status: Int,
  body: json.Json,
) -> http_client.ExecutionResult {
  http_client.ExecutionResult(
    status: status,
    headers: dict.new(),
    body: body,
    raw_body: json.to_string(body),
    elapsed_ms: 100,
    request_method: types.Get,
    request_path: "/test",
  )
}

fn make_execution_result_with_headers(
  status: Int,
  body: json.Json,
  headers: dict.Dict(String, String),
) -> http_client.ExecutionResult {
  http_client.ExecutionResult(
    status: status,
    headers: headers,
    body: body,
    raw_body: json.to_string(body),
    elapsed_ms: 100,
    request_method: types.Get,
    request_path: "/test",
  )
}

fn make_context() -> interpolate.Context {
  interpolate.new_context()
}
