//// Comprehensive tests for Intent Rules Engine
//// Tests all rule types, edge cases, and error conditions

import gleam/dict
import gleam/io
import gleam/json
import gleam/list
import gleeunit/should
import intent/http_client
import intent/rules_engine.{
  BodyContains, BodyMissing, FieldMissing, FieldPresent, HeaderMissing,
  HeaderPresent, RuleFailed, RulePassed,
}
import intent/types.{
  type Method, type Rule, type RuleCheck, type When, Get, Post,
}

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Create a simple Rule
fn make_rule(
  name: String,
  description: String,
  when: When,
  check: RuleCheck,
) -> Rule {
  types.Rule(
    name: name,
    description: description,
    when: when,
    check: check,
    example: json.null(),
  )
}

/// Create a When condition
fn make_when(status: String, method: Method, path: String) -> When {
  types.When(status: status, method: method, path: path)
}

// ============================================================================
// Basic Rule Tests
// ============================================================================

/// Test rule that passes (empty violations)
pub fn test_rule_passes_when_no_violations() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule = make_rule("pass_rule", "Should pass", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("name", json.string("John"))]),
      raw_body: "{\"name\":\"John\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(rule_name)] -> should.equal("pass_rule", rule_name)
    _ -> should.fail()
  }
}

/// Test rule that fails due to missing required string in body
pub fn test_rule_fails_when_required_string_missing() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: ["success"],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "missing_string_rule",
      "Missing success string",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("name", json.string("John"))]),
      raw_body: "{\"name\":\"John\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(rule_name, description, violations)] -> {
      should.equal("missing_string_rule", rule_name)
      should.equal("Missing success string", description)
      should.equal(1, list.length(violations))

      case violations {
        [BodyMissing(required)] -> should.equal("success", required)
        _ -> should.fail()
      }
    }
    _ -> should.fail()
  }
}

/// Test rule that fails due to forbidden string in body
pub fn test_rule_fails_when_forbidden_string_present() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: ["error"],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "forbidden_string_rule",
      "Error string forbidden",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("message", json.string("There was an error"))]),
      raw_body: "{\"message\":\"There was an error\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] -> {
      case violations {
        [BodyContains(forbidden, _)] -> should.equal("error", forbidden)
        _ -> should.fail()
      }
    }
    _ -> should.fail()
  }
}

/// Test case insensitive string matching
pub fn test_rule_uses_case_insensitive_matching() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: ["ERROR"],
      body_must_contain: ["Success"],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "case_sensitive_rule",
      "Case insensitive matching",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("message", json.string("success! no error here"))]),
      raw_body: "{\"message\":\"success! no error here\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] ->
      // Should pass because "success" is found and "ERROR" is not found
      Nil
    _ -> should.fail()
  }
}

// ============================================================================
// Field Existence Tests
// ============================================================================

/// Test rule passes when required fields exist
pub fn test_rule_passes_when_fields_exist() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: ["user.id", "user.name"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "field_exist_rule",
      "Required fields must exist",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([
        #(
          "user",
          json.object([#("id", json.int(123)), #("name", json.string("John"))]),
        ),
      ]),
      raw_body: "{\"user\":{\"id\":123,\"name\":\"John\"}}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule fails when required fields are missing
pub fn test_rule_fails_when_fields_missing() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: ["user.id", "user.profile.email"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "missing_field_rule",
      "Missing required field",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("user", json.object([#("id", json.int(123))]))]),
      raw_body: "{\"user\":{\"id\":123}}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] -> {
      should.equal(1, list.length(violations))
      case violations {
        [FieldMissing(field)] -> should.equal("user.profile.email", field)
        _ -> should.fail()
      }
    }
    _ -> should.fail()
  }
}

/// Test rule passes when forbidden fields don't exist
pub fn test_rule_passes_when_fields_do_not_exist() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: ["password", "credit_card"],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "field_not_exist_rule",
      "Forbidden fields must not exist",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([
        #(
          "user",
          json.object([#("id", json.int(123)), #("name", json.string("John"))]),
        ),
      ]),
      raw_body: "{\"user\":{\"id\":123,\"name\":\"John\"}}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule fails when forbidden fields exist
pub fn test_rule_fails_when_fields_exist_forbidden() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: ["password"],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "field_forbidden_rule",
      "Forbidden field exists",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([
        #(
          "user",
          json.object([
            #("id", json.int(123)),
            #("password", json.string("secret123")),
          ]),
        ),
      ]),
      raw_body: "{\"user\":{\"id\":123,\"password\":\"secret123\"}}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      case violations {
        [FieldPresent(field)] -> should.equal("password", field)
        _ -> should.fail()
      }
    _ -> should.fail()
  }
}

/// Test field navigation with nested JSON
pub fn test_field_navigation_with_deep_nesting() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: ["data.users[0].profile.address.city"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule("deep_field_rule", "Deep field navigation", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([
        #(
          "data",
          json.object([
            #(
              "users",
              json.array(
                [
                  json.object([
                    #(
                      "profile",
                      json.object([
                        #(
                          "address",
                          json.object([
                            #("city", json.string("New York")),
                            #("street", json.string("123 Main St")),
                          ]),
                        ),
                      ]),
                    ),
                  ]),
                ],
                fn(x) { x },
              ),
            ),
          ]),
        ),
      ]),
      raw_body: "{\"data\":{\"users\":[{\"profile\":{\"address\":{\"city\":\"New York\",\"street\":\"123 Main St\"}}}]}}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

// ============================================================================
// Header Tests
// ============================================================================

/// Test rule passes when required header exists
pub fn test_rule_passes_when_header_exists() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "Content-Type",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "header_exist_rule",
      "Required header must exist",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([
        #("Content-Type", "application/json"),
        #("X-Custom", "value"),
      ]),
      body: json.object([#("data", json.string("test"))]),
      raw_body: "{\"data\":\"test\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule fails when required header is missing
pub fn test_rule_fails_when_header_missing() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "Authorization",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "header_missing_rule",
      "Missing required header",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([#("Content-Type", "application/json")]),
      body: json.object([#("data", json.string("test"))]),
      raw_body: "{\"data\":\"test\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      case violations {
        [HeaderMissing(header)] -> should.equal("Authorization", header)
        _ -> should.fail()
      }
    _ -> should.fail()
  }
}

/// Test rule passes when forbidden header doesn't exist
pub fn test_rule_passes_when_header_not_forbidden() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "X-Secret",
    )
  let rule =
    make_rule(
      "header_not_forbidden_rule",
      "Forbidden header not present",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([#("Content-Type", "application/json")]),
      body: json.object([#("data", json.string("test"))]),
      raw_body: "{\"data\":\"test\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule fails when forbidden header exists
pub fn test_rule_fails_when_header_exists_forbidden() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "X-Secret",
    )
  let rule =
    make_rule(
      "header_forbidden_rule",
      "Forbidden header exists",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([
        #("X-Secret", "value"),
        #("Content-Type", "application/json"),
      ]),
      body: json.object([#("data", json.string("test"))]),
      raw_body: "{\"data\":\"test\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      case violations {
        [HeaderPresent(header)] -> should.equal("X-Secret", header)
        _ -> should.fail()
      }
    _ -> should.fail()
  }
}

/// Test case insensitive header matching
pub fn test_rule_uses_case_insensitive_header_matching() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "content-type",
      header_must_not_exist: "x-secret",
    )
  let rule =
    make_rule(
      "header_case_rule",
      "Case insensitive header matching",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([
        #("Content-Type", "application/json"),
        #("X-SECRET", "value"),
      ]),
      body: json.object([#("data", json.string("test"))]),
      raw_body: "{\"data\":\"test\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] -> {
      // Should fail because X-Secret header exists (case insensitive)
      should.equal(1, list.length(violations))
      case violations {
        [HeaderPresent(header)] -> should.equal("x-secret", header)
        _ -> should.fail()
      }
    }
    _ -> should.fail()
  }
}

// ============================================================================
// When Condition Tests
// ============================================================================

/// Test rule applies when all conditions match
pub fn test_rule_applies_when_all_conditions_match() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule = make_rule("apply_rule", "Should apply", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule doesn't apply when status doesn't match
pub fn test_rule_does_not_apply_when_status_mismatch() {
  let when_condition = make_when("404", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "not_apply_status_rule",
      "Should not apply due to status",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  should.equal(0, list.length(results))
}

/// Test rule doesn't apply when method doesn't match
pub fn test_rule_does_not_apply_when_method_mismatch() {
  let when_condition = make_when("200", Post, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "not_apply_method_rule",
      "Should not apply due to method",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  should.equal(0, list.length(results))
}

/// Test rule doesn't apply when path doesn't match
pub fn test_rule_does_not_apply_when_path_mismatch() {
  let when_condition = make_when("200", Get, "/posts")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "not_apply_path_rule",
      "Should not apply due to path",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  should.equal(0, list.length(results))
}

/// Test rule applies when path matches exactly
pub fn test_rule_applies_when_path_matches_exact() {
  let when_condition = make_when("200", Get, "/users/123")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "exact_path_rule",
      "Should apply with exact path",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users/123",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule applies when path matches regex
pub fn test_rule_applies_when_path_matches_regex() {
  let when_condition = make_when("200", Get, "/users/.+")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "regex_path_rule",
      "Should apply with regex path",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users/123",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RulePassed(_)] -> Nil
    _ -> should.fail()
  }
}

/// Test rule doesn't apply when path doesn't match regex
pub fn test_rule_does_not_apply_when_path_regex_mismatch() {
  let when_condition = make_when("200", Get, "/posts/.+")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "regex_path_fail_rule",
      "Should not apply with regex path",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users/123",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  should.equal(0, list.length(results))
}

// ============================================================================
// Status Condition Tests
// ============================================================================

/// Test exact status match
pub fn test_status_condition_exact_match() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("200", response.status)
  matches |> should.be_true()
}

/// Test greater than status condition
pub fn test_status_condition_greater_than() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("> 100", response.status)
  matches |> should.be_true()
}

/// Test greater than or equal status condition
pub fn test_status_condition_greater_than_or_equal() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition(">= 200", response.status)
  matches |> should.be_true()
}

/// Test less than status condition
pub fn test_status_condition_less_than() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("< 300", response.status)
  matches |> should.be_true()
}

/// Test less than or equal status condition
pub fn test_status_condition_less_than_or_equal() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("<= 200", response.status)
  matches |> should.be_true()
}

/// Test equals status condition
pub fn test_status_condition_equals() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("== 200", response.status)
  matches |> should.be_true()
}

/// Test status condition with invalid number
pub fn test_status_condition_invalid_number() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("> abc", response.status)
  matches |> should.be_false()
}

/// Test status condition with negative number
pub fn test_status_condition_negative_number() {
  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition(">= -1", response.status)
  matches |> should.be_true()
}

/// Test status condition with zero
pub fn test_status_condition_zero() {
  let response =
    http_client.ExecutionResult(
      status: 0,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/test",
    )
  let matches = rules_engine.check_status_condition("== 0", response.status)
  matches |> should.be_true()
}

// ============================================================================
// Edge Cases and Error Conditions
// ============================================================================

/// Test rule with empty string checks
pub fn test_rule_with_empty_string_checks() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [""],
      body_must_contain: [""],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule("empty_string_rule", "Empty string checks", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("message", json.string(""))]),
      raw_body: "{\"message\":\"\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      // Empty string in body_must_contain should fail
      should.equal(1, list.length(violations))
    [RulePassed(_)] -> should.fail()
    _ -> should.fail()
  }
}

/// Test rule with null JSON body
pub fn test_rule_with_null_json_body() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: ["error"],
      body_must_contain: ["success"],
      fields_must_exist: ["data"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule("null_body_rule", "Null JSON body", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.null(),
      raw_body: "null",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      // Should fail because null JSON has no fields
      should.equal(1, list.length(violations))
    _ -> should.fail()
  }
}

/// Test rule with empty JSON object body
pub fn test_rule_with_empty_json_body() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: ["error"],
      body_must_contain: ["success"],
      fields_must_exist: ["data"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule("empty_json_rule", "Empty JSON body", when_condition, check)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([]),
      raw_body: "{}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      // Should fail because empty JSON has no fields
      should.equal(1, list.length(violations))
    _ -> should.fail()
  }
}

/// Test rule with invalid JSON field path
pub fn test_rule_with_invalid_json_field_path() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: [],
      fields_must_exist: ["invalid.path[0].field"],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule =
    make_rule(
      "invalid_path_rule",
      "Invalid JSON field path",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("valid", json.string("field"))]),
      raw_body: "{\"valid\":\"field\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      // Should fail because invalid path doesn't exist
      should.equal(1, list.length(violations))
    _ -> should.fail()
  }
}

/// Test rule with multiple violations
pub fn test_rule_with_multiple_violations() {
  let when_condition = make_when("200", Get, "/users")
  let check =
    types.RuleCheck(
      body_must_not_contain: ["error", "fail"],
      body_must_contain: ["success"],
      fields_must_exist: ["id", "name"],
      fields_must_not_exist: ["password"],
      header_must_exist: "Authorization",
      header_must_not_exist: "X-Secret",
    )
  let rule =
    make_rule(
      "multiple_violations_rule",
      "Multiple violations",
      when_condition,
      check,
    )

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.from_list([
        #("Content-Type", "application/json"),
        #("X-Secret", "value"),
      ]),
      body: json.object([
        #("error", json.string("Something failed")),
        #("id", json.int(123)),
      ]),
      raw_body: "{\"error\":\"Something failed\",\"id\":123}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results = rules_engine.check_rules([rule], response, "test_behavior")

  case results {
    [RuleFailed(_, _, violations)] ->
      // Should have 5 violations:
      // 1. BodyContains "error"
      // 2. BodyMissing "success"
      // 3. FieldMissing "name"
      // 4. FieldPresent "password" (not actually present, so this should not be a violation)
      // 5. HeaderMissing "Authorization"
      // 6. HeaderPresent "X-Secret"
      // Actually, let me count carefully:
      // 1. error string found -> BodyContains
      // 2. success string missing -> BodyMissing
      // 3. name field missing -> FieldMissing
      // 4. Authorization header missing -> HeaderMissing
      // 5. X-Secret header present -> HeaderPresent
      // That's 5 violations total
      should.equal(5, list.length(violations))
    _ -> should.fail()
  }
}

/// Test multiple rules with some passing and some failing
pub fn test_multiple_rules_mixed_results() {
  let when_condition1 = make_when("200", Get, "/users")
  let check1 =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: ["success"],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule1 = make_rule("pass_rule", "Should pass", when_condition1, check1)

  let when_condition2 = make_when("200", Get, "/users")
  let check2 =
    types.RuleCheck(
      body_must_not_contain: [],
      body_must_contain: ["failure"],
      fields_must_exist: [],
      fields_must_not_exist: [],
      header_must_exist: "",
      header_must_not_exist: "",
    )
  let rule2 = make_rule("fail_rule", "Should fail", when_condition2, check2)

  let response =
    http_client.ExecutionResult(
      status: 200,
      headers: dict.new(),
      body: json.object([#("message", json.string("success"))]),
      raw_body: "{\"message\":\"success\"}",
      elapsed_ms: 100,
      request_method: Get,
      request_path: "/users",
    )

  let results =
    rules_engine.check_rules([rule1, rule2], response, "test_behavior")

  case results {
    [RulePassed(pass_name), RuleFailed(fail_name, _, _)] -> {
      should.equal("pass_rule", pass_name)
      should.equal("fail_rule", fail_name)
    }
    _ -> should.fail()
  }
}

// ============================================================================
// Format Violation Tests
// ============================================================================

/// Test BodyContains violation formatting
pub fn test_format_body_contains_violation() {
  let violation = BodyContains("error", "response body")
  let formatted = rules_engine.format_violation(violation)
  should.equal("Found forbidden string 'error' in response body", formatted)
}

/// Test BodyMissing violation formatting
pub fn test_format_body_missing_violation() {
  let violation = BodyMissing("success")
  let formatted = rules_engine.format_violation(violation)
  should.equal(
    "Required string 'success' not found in response body",
    formatted,
  )
}

/// Test FieldMissing violation formatting
pub fn test_format_field_missing_violation() {
  let violation = FieldMissing("user.id")
  let formatted = rules_engine.format_violation(violation)
  should.equal("Required field 'user.id' not found", formatted)
}

/// Test FieldPresent violation formatting
pub fn test_format_field_present_violation() {
  let violation = FieldPresent("password")
  let formatted = rules_engine.format_violation(violation)
  should.equal("Forbidden field 'password' is present in response", formatted)
}

/// Test HeaderMissing violation formatting
pub fn test_format_header_missing_violation() {
  let violation = HeaderMissing("Authorization")
  let formatted = rules_engine.format_violation(violation)
  should.equal("Required header 'Authorization' not found", formatted)
}

/// Test HeaderPresent violation formatting
pub fn test_format_header_present_violation() {
  let violation = HeaderPresent("X-Secret")
  let formatted = rules_engine.format_violation(violation)
  should.equal("Forbidden header 'X-Secret' is present in response", formatted)
}

// ============================================================================
// Test Suite Entry Points
// ============================================================================

pub fn all_tests() {
  test_rule_passes_when_no_violations()
  test_rule_fails_when_required_string_missing()
  test_rule_fails_when_forbidden_string_present()
  test_rule_uses_case_insensitive_matching()

  test_rule_passes_when_fields_exist()
  test_rule_fails_when_fields_missing()
  test_rule_passes_when_fields_do_not_exist()
  test_rule_fails_when_fields_exist_forbidden()
  test_field_navigation_with_deep_nesting()

  test_rule_passes_when_header_exists()
  test_rule_fails_when_header_missing()
  test_rule_passes_when_header_not_forbidden()
  test_rule_fails_when_header_exists_forbidden()
  test_rule_uses_case_insensitive_header_matching()

  test_rule_applies_when_all_conditions_match()
  test_rule_does_not_apply_when_status_mismatch()
  test_rule_does_not_apply_when_method_mismatch()
  test_rule_does_not_apply_when_path_mismatch()
  test_rule_applies_when_path_matches_exact()
  test_rule_applies_when_path_matches_regex()
  test_rule_does_not_apply_when_path_regex_mismatch()

  test_status_condition_exact_match()
  test_status_condition_greater_than()
  test_status_condition_greater_than_or_equal()
  test_status_condition_less_than()
  test_status_condition_less_than_or_equal()
  test_status_condition_equals()
  test_status_condition_invalid_number()
  test_status_condition_negative_number()
  test_status_condition_zero()

  test_rule_with_empty_string_checks()
  test_rule_with_null_json_body()
  test_rule_with_empty_json_body()
  test_rule_with_invalid_json_field_path()
  test_rule_with_multiple_violations()
  test_multiple_rules_mixed_results()

  test_format_body_contains_violation()
  test_format_body_missing_violation()
  test_format_field_missing_violation()
  test_format_field_present_violation()
  test_format_header_missing_violation()
  test_format_header_present_violation()

  io.println("All rules engine tests passed!")
}
