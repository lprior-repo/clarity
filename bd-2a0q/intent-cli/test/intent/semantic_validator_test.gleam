import gleam/dict
import gleam/list
import gleam/option
import gleam/string
import gleeunit
import gleeunit/should
import intent/semantic_validator

pub fn main() -> Nil {
  gleeunit.main()
}

// Helper function to create a valid spec for testing
fn valid_spec() -> semantic_validator.Spec {
  semantic_validator.Spec(
    name: "test-api",
    description: "Test API description",
    audience: "developers",
    version: "1.0.0",
    success_criteria: ["Users can CRUD"],
    config: semantic_validator.Config(
      base_url: "http://localhost:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    ),
    features: [
      semantic_validator.Feature(
        name: "test-feature",
        description: "Test feature",
        behaviors: [
          semantic_validator.Behavior(
            name: "test-behavior",
            intent: "Test intent",
            notes: "",
            requires: [],
            tags: [],
            request: semantic_validator.Request(
              method: "GET",
              path: "/api/test",
              headers: dict.new(),
              query: dict.new(),
              body: option.None,
            ),
            response: semantic_validator.Response(
              status: 200,
              example: option.None,
              checks: dict.from_list([
                #(
                  "test",
                  semantic_validator.Check(rule: "present", why: "Test check"),
                ),
              ]),
              headers: option.None,
            ),
            captures: dict.new(),
          ),
        ],
      ),
    ],
    rules: [],
    anti_patterns: [],
    ai_hints: semantic_validator.AIHints(
      implementation: option.None,
      entities: option.None,
      security: option.None,
      pitfalls: option.None,
    ),
  )
}

// Test: Valid spec passes all validation
pub fn valid_spec_passes_validation_test() {
  let spec = valid_spec()

  let result = semantic_validator.validate_spec_semantics(spec)

  result
  |> should.equal(semantic_validator.Valid)
}

// Test: Empty name fails validation
pub fn empty_name_fails_validation_test() {
  let spec = valid_spec()
  let spec = semantic_validator.Spec(..spec, name: "")

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        e.field == "name"
        && e.rule == "non_empty_string"
        && string.contains(e.explanation, "cannot be empty")
      })
      |> should.be_true
    }
  }
}

// Test: Empty description fails validation
pub fn empty_description_fails_validation_test() {
  let spec = valid_spec()
  let spec = semantic_validator.Spec(..spec, description: "")

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        e.field == "description" && e.rule == "non_empty_string"
      })
      |> should.be_true
    }
  }
}

// Test: Empty version fails validation
pub fn empty_version_fails_validation_test() {
  let spec = valid_spec()
  let spec = semantic_validator.Spec(..spec, version: "")

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) { e.field == "version" && e.rule == "non_empty_string" })
      |> should.be_true
    }
  }
}

// Test: Negative timeout fails validation
pub fn negative_timeout_fails_validation_test() {
  let spec = valid_spec()
  let config = semantic_validator.Config(..spec.config, timeout_ms: -1000)
  let spec = semantic_validator.Spec(..spec, config: config)

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        e.field == "config.timeout_ms"
        && e.rule == "positive_integer"
        && string.contains(e.expected, "> 0")
      })
      |> should.be_true
    }
  }
}

// Test: Zero timeout fails validation
pub fn zero_timeout_fails_validation_test() {
  let spec = valid_spec()
  let config = semantic_validator.Config(..spec.config, timeout_ms: 0)
  let spec = semantic_validator.Spec(..spec, config: config)

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        e.field == "config.timeout_ms" && e.rule == "positive_integer"
      })
      |> should.be_true
    }
  }
}

// Test: Empty features list fails validation
pub fn empty_features_fails_validation_test() {
  let spec = valid_spec()
  let spec = semantic_validator.Spec(..spec, features: [])

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        e.field == "features"
        && e.rule == "non_empty_list"
        && string.contains(e.explanation, "at least one feature")
      })
      |> should.be_true
    }
  }
}

// Test: Path traversal fails validation
pub fn path_traversal_fails_validation_test() {
  let spec = valid_spec()

  let behavior =
    semantic_validator.Behavior(
      name: "traversal-test",
      intent: "Test path traversal detection",
      notes: "",
      requires: [],
      tags: [],
      request: semantic_validator.Request(
        method: "GET",
        path: "../../../etc/passwd",
        headers: dict.new(),
        query: dict.new(),
        body: option.None,
      ),
      response: semantic_validator.Response(
        status: 200,
        example: option.None,
        checks: dict.new(),
        headers: option.None,
      ),
      captures: dict.new(),
    )

  let feature =
    semantic_validator.Feature(
      name: "test-feature",
      description: "Test feature",
      behaviors: [behavior],
    )

  let spec = semantic_validator.Spec(..spec, features: [feature])

  case semantic_validator.validate_spec_semantics(spec) {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      errors
      |> list.any(fn(e) {
        string.contains(e.field, "path")
        && e.rule == "no_path_traversal"
        && string.contains(e.actual, "../")
      })
      |> should.be_true
    }
  }
}

// Test: Format validation errors
pub fn format_validation_errors_test() {
  let error =
    semantic_validator.ValidationError(
      field: "name",
      rule: "non_empty_string",
      expected: "non-empty string",
      actual: "\"\"",
      explanation: "Name cannot be empty",
    )

  let formatted = semantic_validator.format_validation_errors([error])

  formatted
  |> string.contains("1 error")
  |> should.be_true

  formatted
  |> string.contains("name")
  |> should.be_true

  formatted
  |> string.contains("non_empty_string")
  |> should.be_true
}
