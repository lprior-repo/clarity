/// Comprehensive tests for checker rule validation
/// Following TDD RED-GREEN-REFACTOR workflow
///
/// RED Phase: Write failing tests for each rule type
/// GREEN Phase: Implement functionality to make tests pass
/// REFACTOR Phase: Clean up and optimize
import gleam/dict
import gleam/json
import gleam/list
import gleeunit/should
import intent/checker
import intent/http_client
import intent/interpolate
import intent/types

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

fn make_context() -> interpolate.Context {
  interpolate.new_context()
}

// =============================================================================
// RED Phase: Integer Comparison Tests
// =============================================================================

pub fn integer_gte_pass_test() {
  let checks =
    dict.from_list([
      #("age", types.Check(rule: "integer >= 18", why: "Age check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("age", json.int(25))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn integer_gte_fail_test() {
  let checks =
    dict.from_list([
      #("age", types.Check(rule: "integer >= 18", why: "Age check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("age", json.int(15))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn integer_gt_pass_test() {
  let checks =
    dict.from_list([
      #("score", types.Check(rule: "integer > 100", why: "Score check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("score", json.int(150))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn integer_gt_fail_test() {
  let checks =
    dict.from_list([
      #("score", types.Check(rule: "integer > 100", why: "Score check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("score", json.int(100))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn integer_lte_pass_test() {
  let checks =
    dict.from_list([
      #("temperature", types.Check(rule: "integer <= 100", why: "Temp check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("temperature", json.int(95))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn integer_lte_fail_test() {
  let checks =
    dict.from_list([
      #("temperature", types.Check(rule: "integer <= 100", why: "Temp check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("temperature", json.int(105))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn integer_lt_pass_test() {
  let checks =
    dict.from_list([
      #("age", types.Check(rule: "integer < 65", why: "Age check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("age", json.int(30))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn integer_lt_fail_test() {
  let checks =
    dict.from_list([
      #("age", types.Check(rule: "integer < 65", why: "Age check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("age", json.int(65))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn integer_between_pass_test() {
  let checks =
    dict.from_list([
      #("rating", types.Check(rule: "integer > 0 and < 6", why: "Rating check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("rating", json.int(3))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn integer_between_fail_low_test() {
  let checks =
    dict.from_list([
      #("rating", types.Check(rule: "integer > 0 and < 6", why: "Rating check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("rating", json.int(0))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn integer_between_fail_high_test() {
  let checks =
    dict.from_list([
      #("rating", types.Check(rule: "integer > 0 and < 6", why: "Rating check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("rating", json.int(6))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// RED Phase: Float Comparison Tests
// =============================================================================

pub fn number_between_pass_float_test() {
  let checks =
    dict.from_list([
      #(
        "price",
        types.Check(rule: "number between 10.5 and 99.9", why: "Price check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("price", json.float(50.0))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn number_between_pass_int_test() {
  let checks =
    dict.from_list([
      #(
        "price",
        types.Check(rule: "number between 10.0 and 100.0", why: "Price check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("price", json.int(50))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn number_between_fail_low_test() {
  let checks =
    dict.from_list([
      #(
        "price",
        types.Check(rule: "number between 10.0 and 100.0", why: "Price check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("price", json.float(5.0))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn number_between_fail_high_test() {
  let checks =
    dict.from_list([
      #(
        "price",
        types.Check(rule: "number between 10.0 and 100.0", why: "Price check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("price", json.float(150.0))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// RED Phase: String Format Tests
// =============================================================================

pub fn email_format_valid_test() {
  let checks =
    dict.from_list([
      #("email", types.Check(rule: "email", why: "Email format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("email", json.string("user@example.com"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn email_format_invalid_test() {
  let checks =
    dict.from_list([
      #("email", types.Check(rule: "email", why: "Email format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("email", json.string("not-an-email"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn uuid_format_valid_test() {
  let checks =
    dict.from_list([
      #("id", types.Check(rule: "uuid", why: "UUID format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("id", json.string("550e8400-e29b-41d4-a716-446655440000"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn uuid_format_invalid_test() {
  let checks =
    dict.from_list([
      #("id", types.Check(rule: "uuid", why: "UUID format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("id", json.string("not-a-uuid"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn uri_format_valid_test() {
  let checks =
    dict.from_list([#("url", types.Check(rule: "uri", why: "URI format check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("url", json.string("https://example.com/path"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn uri_format_invalid_test() {
  let checks =
    dict.from_list([#("url", types.Check(rule: "uri", why: "URI format check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("url", json.string("not-a-uri"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn jwt_format_valid_test() {
  let checks =
    dict.from_list([
      #("token", types.Check(rule: "jwt", why: "JWT format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  // Valid JWT with 3 parts (header.payload.signature)
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "token",
          json.string(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
          ),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn jwt_format_invalid_test() {
  let checks =
    dict.from_list([
      #("token", types.Check(rule: "jwt", why: "JWT format check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("token", json.string("not-a-jwt"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn iso8601_format_valid_date_test() {
  let checks =
    dict.from_list([
      #(
        "created_at",
        types.Check(rule: "iso8601 datetime", why: "ISO8601 format check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("created_at", json.string("2024-01-15"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn iso8601_format_valid_datetime_test() {
  let checks =
    dict.from_list([
      #(
        "created_at",
        types.Check(rule: "iso8601 datetime", why: "ISO8601 format check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("created_at", json.string("2024-01-15T10:30:00Z"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn iso8601_format_invalid_test() {
  let checks =
    dict.from_list([
      #(
        "created_at",
        types.Check(rule: "iso8601 datetime", why: "ISO8601 format check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("created_at", json.string("not-a-date"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// RED Phase: Array Validation Tests
// =============================================================================

pub fn array_length_exact_pass_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array of length 3", why: "Array length check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "items",
          json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_length_exact_fail_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array of length 3", why: "Array length check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("items", json.array([json.int(1), json.int(2)], fn(x) { x })),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn array_min_items_pass_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array with min 2 items", why: "Min items check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "items",
          json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_min_items_fail_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array with min 3 items", why: "Min items check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("items", json.array([json.int(1), json.int(2)], fn(x) { x })),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn array_max_items_pass_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array with max 5 items", why: "Max items check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "items",
          json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_max_items_fail_test() {
  let checks =
    dict.from_list([
      #(
        "items",
        types.Check(rule: "array with max 2 items", why: "Max items check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "items",
          json.array([json.int(1), json.int(2), json.int(3)], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn array_non_empty_pass_test() {
  let checks =
    dict.from_list([
      #("items", types.Check(rule: "non-empty array", why: "Non-empty check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("items", json.array([json.int(1)], fn(x) { x }))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_non_empty_fail_test() {
  let checks =
    dict.from_list([
      #("items", types.Check(rule: "non-empty array", why: "Non-empty check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("items", json.array([], fn(x) { x }))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn array_where_each_string_pass_test() {
  let checks =
    dict.from_list([
      #(
        "names",
        types.Check(
          rule: "array where each is string",
          why: "All strings check",
        ),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "names",
          json.array([json.string("Alice"), json.string("Bob")], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_where_each_string_fail_test() {
  let checks =
    dict.from_list([
      #(
        "values",
        types.Check(
          rule: "array where each is string",
          why: "All strings check",
        ),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("values", json.array([json.string("ok"), json.int(123)], fn(x) { x })),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn array_where_each_integer_pass_test() {
  let checks =
    dict.from_list([
      #(
        "scores",
        types.Check(
          rule: "array where each is integer",
          why: "All integers check",
        ),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "scores",
          json.array([json.int(10), json.int(20), json.int(30)], fn(x) { x }),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_where_each_email_pass_test() {
  let checks =
    dict.from_list([
      #(
        "emails",
        types.Check(rule: "array where each is email", why: "All emails check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "emails",
          json.array(
            [json.string("user1@example.com"), json.string("user2@example.com")],
            fn(x) { x },
          ),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn array_where_each_email_fail_test() {
  let checks =
    dict.from_list([
      #(
        "emails",
        types.Check(rule: "array where each is email", why: "All emails check"),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #(
          "emails",
          json.array(
            [json.string("valid@example.com"), json.string("invalid")],
            fn(x) { x },
          ),
        ),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// RED Phase: OneOf Validation Tests
// =============================================================================

pub fn one_of_pass_test() {
  let checks =
    dict.from_list([
      #(
        "status",
        types.Check(
          rule: "one of [\"active\", \"pending\", \"completed\"]",
          why: "Status check",
        ),
      ),
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
  list.length(result.failed) |> should.equal(0)
}

pub fn one_of_fail_test() {
  let checks =
    dict.from_list([
      #(
        "status",
        types.Check(
          rule: "one of [\"active\", \"pending\", \"completed\"]",
          why: "Status check",
        ),
      ),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("status", json.string("unknown"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

// =============================================================================
// RED Phase: Type Validation Tests
// =============================================================================

pub fn is_string_pass_test() {
  let checks =
    dict.from_list([#("name", types.Check(rule: "string", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("name", json.string("Alice"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_string_fail_test() {
  let checks =
    dict.from_list([#("name", types.Check(rule: "string", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("name", json.int(123))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_integer_pass_test() {
  let checks =
    dict.from_list([#("age", types.Check(rule: "integer", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual = make_execution_result(200, json.object([#("age", json.int(30))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_integer_fail_test() {
  let checks =
    dict.from_list([#("age", types.Check(rule: "integer", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("age", json.string("30"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_number_pass_int_test() {
  let checks =
    dict.from_list([#("value", types.Check(rule: "number", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.int(42))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_number_pass_float_test() {
  let checks =
    dict.from_list([#("value", types.Check(rule: "number", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.float(3.14))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_number_fail_test() {
  let checks =
    dict.from_list([#("value", types.Check(rule: "number", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.string("42"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_boolean_pass_test() {
  let checks =
    dict.from_list([
      #("active", types.Check(rule: "boolean", why: "Type check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("active", json.bool(True))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_boolean_fail_test() {
  let checks =
    dict.from_list([
      #("active", types.Check(rule: "boolean", why: "Type check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("active", json.string("true"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_array_pass_test() {
  let checks =
    dict.from_list([#("items", types.Check(rule: "array", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([
        #("items", json.array([json.int(1), json.int(2)], fn(x) { x })),
      ]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_array_fail_test() {
  let checks =
    dict.from_list([#("items", types.Check(rule: "array", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("items", json.string("not-array"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_object_pass_test() {
  let checks =
    dict.from_list([#("data", types.Check(rule: "object", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("data", json.object([#("key", json.string("value"))]))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_object_fail_test() {
  let checks =
    dict.from_list([#("data", types.Check(rule: "object", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("data", json.string("not-object"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn is_null_pass_test() {
  let checks =
    dict.from_list([#("value", types.Check(rule: "null", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.null())]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn is_null_fail_test() {
  let checks =
    dict.from_list([#("value", types.Check(rule: "null", why: "Type check"))])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(
      200,
      json.object([#("value", json.string("not-null"))]),
    )
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}

pub fn not_null_pass_test() {
  let checks =
    dict.from_list([
      #("value", types.Check(rule: "not null", why: "Not null check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.string("value"))]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(1)
  list.length(result.failed) |> should.equal(0)
}

pub fn not_null_fail_test() {
  let checks =
    dict.from_list([
      #("value", types.Check(rule: "not null", why: "Not null check")),
    ])
  let expected = make_response(200, checks, dict.new())
  let actual =
    make_execution_result(200, json.object([#("value", json.null())]))
  let ctx = make_context()

  let result = checker.check_response(expected, actual, ctx)

  list.length(result.passed) |> should.equal(0)
  list.length(result.failed) |> should.equal(1)
}
